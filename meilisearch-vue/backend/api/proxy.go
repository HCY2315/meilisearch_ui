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
)

// HandleProxy 拦截并转发所有发往 Meilisearch 的请求
func HandleProxy(c *gin.Context) {
	// 目前简易取第一台实例，多实例可根据前台 App 或 User 绑定关联查询
	var instance model.MeiliInstance
	repository.DB.First(&instance)

	proxyPath := c.Param("proxyPath")
	// 鉴权：拦截 URL 路径进行安全审查 (如 /indexes/books/search)
	parts := strings.Split(strings.TrimPrefix(proxyPath, "/"), "/")
	if len(parts) >= 2 && parts[0] == "indexes" {
		requestedIndex := parts[1]
		role, _ := c.Get("role")
		if role != "admin" {
			userId, _ := c.Get("userId")
			var user model.User
			repository.DB.First(&user, userId)

			allowed := false
			var allowedArr []string
			json.Unmarshal([]byte(user.AllowIndexes), &allowedArr)
			for _, a := range allowedArr {
				if a == requestedIndex || a == "*" {
					allowed = true
					break
				}
			}
			if !allowed {
				c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "You do not have access to index: " + requestedIndex})
				return
			}
		}
	}

	// 转发到真实的 Meilisearch 节点
	target, _ := url.Parse(instance.Host)
	proxy := httputil.NewSingleHostReverseProxy(target)

	originalDirector := proxy.Director
	proxy.Director = func(req *http.Request) {
		originalDirector(req)
		req.Host = target.Host
		// 剥离请求携带的 JWT Token，换成真实的 Meilisearch Master_Key
		req.Header.Set("Authorization", "Bearer "+instance.APIKey)
		req.Header.Set("X-Meili-API-Key", instance.APIKey)
	}

	proxy.ServeHTTP(c.Writer, c.Request)
}
