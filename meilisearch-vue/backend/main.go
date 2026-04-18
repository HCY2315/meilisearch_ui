package main

import (
	"log"
	"net/http"
	"time"

	"backend/model"
	"backend/repository"

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
		}
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

	var instance model.MeiliInstance
	if err := repository.DB.Where("id = ?", app.InstanceID).First(&instance).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Instance not properly configured"})
		return
	}

	// TODO: 利用 instance.APIKey 去 Meilisearch 换取 Tenant Token，现在临时直接返回假的 token 和真实的 Host
	// NOTE: 在生产环境中，前端不应拿到 Master Key
	
	c.JSON(http.StatusOK, gin.H{
		"uiConfig": app.UIConfig,
		"allowIndexes": app.AllowIndexes,
		"meili": gin.H{
			"host": instance.Host,
			// HACK: 暂时返回 apiKey 给前端测试连接，后续应替换为 Tenant Token
			"searchToken": instance.APIKey, 
		},
	})
}
