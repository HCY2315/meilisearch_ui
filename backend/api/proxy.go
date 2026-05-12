package api

import (
	"encoding/json"
	"net/http"
	"net/http/httputil"
	"net/url"
	"strings"
	"time"

	"backend/model"
	"backend/repository"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
)

// HandleProxy 拦截并转发所有发往 Meilisearch 的请求
func HandleProxy(c *gin.Context) {
	var instance model.MeiliInstance
	repository.DB.First(&instance)

	proxyPath := c.Param("proxyPath")
	parts := strings.Split(strings.TrimPrefix(proxyPath, "/"), "/")

	// 尝试解析可选的系统管理员 JWT 以便让管理员无视锁限制
	isAdmin := false
	authHdr := c.GetHeader("Authorization")
	if strings.HasPrefix(authHdr, "Bearer ") {
		tokenStr := strings.TrimPrefix(authHdr, "Bearer ")
		token, _ := jwt.Parse(tokenStr, func(token *jwt.Token) (interface{}, error) { return JwtSecret, nil })
		if token != nil && token.Valid {
			if claims, ok := token.Claims.(jwt.MapClaims); ok {
				if role, _ := claims["role"].(string); role == "admin" {
					isAdmin = true
				}
			}
		}
	}

	// 安全鉴权与只读(Read-Only)限制
	if !isAdmin {
		// 阻断普通的写入/删除请求 (只允许 GET/OPTIONS 或者 POST查询)
		method := c.Request.Method
		pathSuffix := c.Request.URL.Path
		isSearchPost := method == "POST" && (strings.HasSuffix(pathSuffix, "/search") || strings.HasSuffix(pathSuffix, "/multi-search"))
		if method != "GET" && method != "OPTIONS" && !isSearchPost {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "只读模式 (Read-Only Mode): 缺乏系统管理员权限，禁止写入或修改索引配置。"})
			return
		}

		if len(parts) >= 2 && parts[0] == "indexes" {
			requestedIndex := parts[1]

			// 普通访客，检查该 Index 是否在前台隐藏
			var indexConf model.IndexConfig
			isVisible := true
			isLocked := false
			if err := repository.DB.Where("uid = ?", requestedIndex).First(&indexConf).Error; err == nil {
				isVisible = indexConf.IsVisible
				isLocked = indexConf.IsLocked
			}

			if !isVisible {
				c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "该索引已被前台隐藏"})
				return
			}

			if isLocked {
				// 如果被上锁，需要验证 App-Token 是否拥有权限
				userToken := c.GetHeader("App-Token")
				hasAccess := false

				if userToken != "" {
					var tok model.AccessToken
					if err := repository.DB.Where("token = ?", userToken).First(&tok).Error; err == nil {
						if tok.ExpiresAt != nil && tok.ExpiresAt.Before(time.Now()) {
							// NOTE: Token 已过期，执行软删除（model 已配置 DeletedAt）
							repository.DB.Delete(&tok)
						} else {
							var allowedArr []string
							json.Unmarshal([]byte(tok.AllowIndexes), &allowedArr)
							for _, a := range allowedArr {
								if a == requestedIndex || a == "*" {
									hasAccess = true
									break
								}
							}
						}
					}
				}

				if !hasAccess {
					c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "该库已被锁定，请输入正确的访问凭证(Token)"})
					return
				}
			}
		}
	}

	target, _ := url.Parse(instance.Host)
	proxy := httputil.NewSingleHostReverseProxy(target)

	originalDirector := proxy.Director
	proxy.Director = func(req *http.Request) {
		originalDirector(req)
		req.Host = target.Host
		req.URL.Path = proxyPath
		req.Header.Set("Authorization", "Bearer "+instance.APIKey)
		req.Header.Set("X-Meili-API-Key", instance.APIKey)
	}

	proxy.ServeHTTP(c.Writer, c.Request)
}
