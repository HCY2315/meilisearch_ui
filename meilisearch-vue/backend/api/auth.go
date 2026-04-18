package api

import (
	"encoding/json"
	"io"
	"net/http"
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
	if userToken != "" {
		var tok model.AccessToken
		if err := repository.DB.Where("token = ?", userToken).First(&tok).Error; err == nil {
			json.Unmarshal([]byte(tok.AllowIndexes), &allowedByToken)
		}
	} // 就算 token 为空，也不报错，只下发公开项

	// Load DB config map
	var configs []model.IndexConfig
	repository.DB.Find(&configs)
	lockedMap := make(map[string]bool)
	for _, conf := range configs {
		lockedMap[conf.Uid] = conf.IsLocked
	}

	// 1. 直连 Meilisearch 获取真实全部的 indexes
	client := &http.Client{}
	req, _ := http.NewRequest("GET", instance.Host+"/indexes", nil)
	req.Header.Set("Authorization", "Bearer "+instance.APIKey)
	res, err := client.Do(req)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "无法连接到底层 Meilisearch"})
		return
	}
	defer res.Body.Close()

	if res.StatusCode != 200 {
		c.Status(res.StatusCode)
		return
	}

	var payload struct {
		Results []map[string]interface{} `json:"results"`
		Offset  int                      `json:"offset"`
		Limit   int                      `json:"limit"`
		Total   int                      `json:"total"`
	}
	bodyBytes, _ := io.ReadAll(res.Body)
	json.Unmarshal(bodyBytes, &payload)

	// 2. 过滤
	filteredResults := []map[string]interface{}{}
	for _, idx := range payload.Results {
		uid, ok := idx["uid"].(string)
		if !ok {
			continue
		}

		isLocked := lockedMap[uid]
		canView := false

		if !isLocked {
			// 公开的
			canView = true
		} else {
			// 加锁了，检查 token
			for _, allowed := range allowedByToken {
				if allowed == uid || allowed == "*" {
					canView = true
					break
				}
			}
		}

		if canView {
			filteredResults = append(filteredResults, idx)
		}
	}

	payload.Results = filteredResults
	payload.Total = len(filteredResults)

	c.JSON(http.StatusOK, payload)
}
