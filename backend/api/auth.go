package api

import (
	"encoding/json"
	"io"
	"log"
	"net/http"
	"strings"
	"time"

	"backend/model"
	"backend/repository"
	"backend/schema"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/crypto/bcrypt"
)

func HandleLogin(c *gin.Context) {
	var req schema.LoginRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid params"})
		return
	}

	var user model.User
	if err := repository.DB.Where("username = ?", req.Username).First(&user).Error; err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not found"})
		return
	}

	// 校验密码
	if err := bcrypt.CompareHashAndPassword([]byte(user.PasswordHash), []byte(req.Password)); err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid password"})
		return
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"userId":   user.ID,
		"username": user.Username,
		"role":     user.Role,
		"exp":      time.Now().Add(time.Hour * 24).Unix(),
	})

	tokenString, _ := token.SignedString(JwtSecret)

	c.JSON(http.StatusOK, gin.H{
		"token": tokenString,
		"user": gin.H{
			"id":       user.ID,
			"username": user.Username,
			"role":     user.Role,
		},
	})
}

func HandleAppConfig(c *gin.Context) {
	appKey := c.GetHeader("App-Key")
	if appKey == "" {
		appKey = "default-app-key"
	}

	var app model.Application
	if err := repository.DB.Where("app_key = ?", appKey).First(&app).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "App Config not found"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"uiConfig": app.UIConfig,
		"meili": gin.H{
			"host":        "/api/v1/proxy",
			"searchToken": c.GetHeader("Authorization"),
		},
	})
}

// HandleGetVisibleIndexes 代理请求 Meilisearch，然后根据后台的 lock 以及前端传来的 token 过滤数据
func HandleGetVisibleIndexes(c *gin.Context) {
	var instance model.MeiliInstance
	repository.DB.First(&instance)

	userToken := c.GetHeader("App-Token")
	var allowedByToken []string

	// ---- 特权检查：如果是系统管理员登录，无视一切锁，显示全部 ----
	isAdmin := false
	authHdr := c.GetHeader("Authorization")
	if strings.HasPrefix(authHdr, "Bearer ") {
		tStr := strings.TrimPrefix(authHdr, "Bearer ")
		t, _ := jwt.Parse(tStr, func(token *jwt.Token) (interface{}, error) { return JwtSecret, nil })
		if t != nil && t.Valid {
			if claims, ok := t.Claims.(jwt.MapClaims); ok {
				if r, _ := claims["role"].(string); r == "admin" {
					isAdmin = true
				}
			}
		}
	}

	if userToken != "" {
		var tok model.AccessToken
		if err := repository.DB.Where("token = ?", userToken).First(&tok).Error; err == nil {
			if tok.ExpiresAt != nil && tok.ExpiresAt.Before(time.Now()) {
				repository.DB.Delete(&tok)
			} else {
				json.Unmarshal([]byte(tok.AllowIndexes), &allowedByToken)
			}
		}
	}

	// Load DB config map
	var configs []model.IndexConfig
	repository.DB.Find(&configs)
	configMap := make(map[string]model.IndexConfig)
	for _, conf := range configs {
		configMap[conf.Uid] = conf
	}
	log.Printf("[DEBUG] configMap keys: %v", configMap)

	// 1. 直连 Meilisearch 获取真实全部的 indexes
	client := &http.Client{}
	req, _ := http.NewRequest("GET", instance.Host+"/indexes", nil)
	req.Header.Set("Authorization", "Bearer "+instance.APIKey)
	res, err := client.Do(req)
	if err != nil {
		log.Printf("[Proxy Error] Failed to connect to Meilisearch: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "无法连接到底层 Meilisearch"})
		return
	}
	defer res.Body.Close()

	var payload struct {
		Results []map[string]interface{} `json:"results"`
		Total   int                      `json:"total"`
	}
	bodyBytes, _ := io.ReadAll(res.Body)
	json.Unmarshal(bodyBytes, &payload)

	// 2. 增强逻辑：合并配置，标记锁定状态
	enhancedResults := []map[string]interface{}{}
	for _, idx := range payload.Results {
		uid, _ := idx["uid"].(string)

		dbConf, exists := configMap[uid]
		log.Printf("[DEBUG] uid=%s exists=%v drawer=%q", uid, exists, dbConf.DrawerFieldOrder)
		log.Printf("[DEBUG] uid=%s, exists=%v, drawerFieldOrder=%q", uid, exists, dbConf.DrawerFieldOrder)
		isLocked := false
		alias := uid
		if exists {
			isLocked = dbConf.IsLocked
			if dbConf.Alias != "" {
				alias = dbConf.Alias
			}
		}

		isUnlocked := false
		if isAdmin {
			isUnlocked = true
		} else if isLocked {
			// 加锁了，检查 token 是否有权访问该 UID
			for _, allowed := range allowedByToken {
				if allowed == uid || allowed == "*" {
					isUnlocked = true
					break
				}
			}
		} else {
			// 未上锁的默认即为已解锁状态
			isUnlocked = true
		}

		// 注入前台所需状态
		idx["isLocked"] = isLocked
		idx["isUnlocked"] = isUnlocked
		idx["displayName"] = alias

        if exists {
            idx["fieldConfigs"] = dbConf.FieldConfigs
            idx["viewConfigs"] = dbConf.ViewConfigs
            idx["tableConfigs"] = dbConf.TableConfigs
            idx["canEdit"] = dbConf.CanEdit
            idx["nestedFieldConfigs"] = dbConf.NestedFieldConfigs
            if dbConf.DrawerFieldOrder != "" {
                idx["drawerFieldOrder"] = dbConf.DrawerFieldOrder
            } else {
                idx["drawerFieldOrder"] = "{}"
            }
        } else {
            idx["fieldConfigs"] = ""
            idx["viewConfigs"] = ""
            idx["tableConfigs"] = ""
            idx["canEdit"] = false
            idx["nestedFieldConfigs"] = ""
            idx["drawerFieldOrder"] = "{}"
        }

		enhancedResults = append(enhancedResults, idx)
	}

	payload.Results = enhancedResults
	payload.Total = len(enhancedResults)

	c.JSON(http.StatusOK, payload)
}
