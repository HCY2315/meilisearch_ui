package main

import (
	"log"
	"net/http"
	"time"

	"backend/model"
	"backend/repository"

	"encoding/json"
	"net/http/httputil"
	"net/url"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/crypto/bcrypt"
)

var jwtSecret = []byte("super-secret-key-change-me-later")

func main() {
	// 初始化数据库
	repository.InitDB()

	r := gin.Default()

	// 允许跨域 (CORS) 简单配置
	r.Use(func(c *gin.Context) {
		c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
		c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, GET, OPTIONS, PUT, DELETE")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization, App-Key")
		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(204)
			return
		}
		c.Next()
	})

	api := r.Group("/api/v1")
	{
		// 1. 登录接口
		api.POST("/auth/login", handleLogin)

		// 2. 前台获取配置接口
		api.GET("/app/config", handleAppConfig)

		// 3. 后台管理接口 (需鉴权且必须是 admin)
		adminGroup := api.Group("/admin")
		adminGroup.Use(AuthMiddleware())
		{
			adminGroup.GET("/instances", handleGetInstances)
			adminGroup.GET("/apps", handleGetApps)
			adminGroup.PUT("/apps/:id", handleUpdateApp)
			
			// User management endpoints
			adminGroup.GET("/users", handleGetUsers)
			adminGroup.POST("/users", handleCreateUser)
			adminGroup.PUT("/users/:id/permissions", handleUpdateUserPermissions)
		}

		// 4. Meilisearch 透明转发网关 (Proxy)
		proxyGroup := api.Group("/proxy")
		proxyGroup.Use(AuthMiddleware()) // 网关统一经过 JWT 验证
		proxyGroup.Any("/*proxyPath", handleProxy)
	}

	log.Println("Server mapping to port :8080")
	r.Run(":8080")
}

type LoginRequest struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

func handleLogin(c *gin.Context) {
	var req LoginRequest
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

	// 签发 JWT
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"userId":   user.ID,
		"username": user.Username,
		"role":     user.Role,
		"exp":      time.Now().Add(time.Hour * 24).Unix(),
	})

	tokenString, _ := token.SignedString(jwtSecret)

	c.JSON(http.StatusOK, gin.H{
		"token": tokenString,
		"user": gin.H{
			"id":       user.ID,
			"username": user.Username,
			"role":     user.Role,
		},
	})
}

func handleAppConfig(c *gin.Context) {
	// 期望在头部传递 App-Key 或者通过鉴权拿到绑定 App
	appKey := c.GetHeader("App-Key")
	if appKey == "" {
		// fallback to default locally
		appKey = "default-app-key"
	}

	var app model.Application
	if err := repository.DB.Where("app_key = ?", appKey).First(&app).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "App Config not found"})
		return
	}

	// Default allowIndexes limit from the Application config itself
	allowIndexes := app.AllowIndexes

	// Check if a user is logged in
	authHeader := c.GetHeader("Authorization")
	if len(authHeader) > 7 && authHeader[:7] == "Bearer " {
		tokenStr := authHeader[7:]
		// try parse and get userId
		token, err := jwt.Parse(tokenStr, func(token *jwt.Token) (interface{}, error) { return jwtSecret, nil })
		if err == nil && token.Valid {
			if claims, ok := token.Claims.(jwt.MapClaims); ok {
				var user model.User
				if err := repository.DB.First(&user, claims["userId"]).Error; err == nil {
					// User's own allowance overrides app level definition
					if user.Role == "admin" {
						allowIndexes = `["*"]` // Admin can see all indexes natively
					} else if user.AllowIndexes != "" {
						allowIndexes = user.AllowIndexes
					}
				}
			}
		}
	}

	var instance model.MeiliInstance
	if err := repository.DB.Where("id = ?", app.InstanceID).First(&instance).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Instance not properly configured"})
		return
	}
	
	c.JSON(http.StatusOK, gin.H{
		"uiConfig": app.UIConfig,
		"allowIndexes": allowIndexes,
		"meili": gin.H{
			// 返回网关地址，而非真实 Meili host
			"host": "http://localhost:8080/api/v1/proxy",
			// 前台使用 JWT Token 作为身份凭证伪装成 apiKey 请求网关
			"searchToken": c.GetHeader("Authorization"), 
		},
	})
}

// handleProxy 拦截并转发所有发往 Meilisearch 的请求
func handleProxy(c *gin.Context) {
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
		// 剥离请求携带的 JWT Token，擦除痕迹，换成真实的 Meilisearch Master_Key
		req.Header.Set("Authorization", "Bearer "+instance.APIKey)
		req.Header.Set("X-Meili-API-Key", instance.APIKey)
	}

	proxy.ServeHTTP(c.Writer, c.Request)
}
