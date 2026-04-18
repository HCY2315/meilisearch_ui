package api

import (
	"encoding/json"
	"net/http"
	"net/http/httputil"
	"net/url"
	"strings"

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

	// 安全鉴权：拦截请求验证其对目标索引的访问权限
	if len(parts) >= 2 && parts[0] == "indexes" {
		requestedIndex := parts[1]

		if !isAdmin {
			// 普通访客，检查该 Index 是否被上锁
			var indexConf model.IndexConfig
			isLocked := false
			if err := repository.DB.Where("uid = ?", requestedIndex).First(&indexConf).Error; err == nil {
				isLocked = indexConf.IsLocked
			}

			if isLocked {
				// 如果被上锁，需要验证 App-Token 是否拥有权限
				userToken := c.GetHeader("App-Token")
				hasAccess := false

				if userToken != "" {
					var tok model.AccessToken
					if err := repository.DB.Where("token = ?", userToken).First(&tok).Error; err == nil {
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
