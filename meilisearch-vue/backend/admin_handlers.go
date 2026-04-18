package main

import (
	"net/http"
	"strings"

	"backend/model"
	"backend/repository"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
)

// AuthMiddleware 校验 JWT token
func AuthMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		tokenStr := c.GetHeader("Authorization")
		if tokenStr == "" || !strings.HasPrefix(tokenStr, "Bearer ") {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "Missing or invalid token"})
			return
		}
		tokenStr = strings.TrimPrefix(tokenStr, "Bearer ")

		token, err := jwt.Parse(tokenStr, func(token *jwt.Token) (interface{}, error) {
			return jwtSecret, nil
		})

		if err != nil || !token.Valid {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "Invalid token"})
			return
		}

		claims, _ := token.Claims.(jwt.MapClaims)
		// 校验后台权限
		role, ok := claims["role"].(string)
		if !ok || role != "admin" {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "Admin access required"})
			return
		}

		c.Set("userId", claims["userId"])
		c.Set("role", claims["role"])
		c.Next()
	}
}

// handleGetInstances 获取所有实例信息
func handleGetInstances(c *gin.Context) {
	var instances []model.MeiliInstance
	repository.DB.Find(&instances)
	// 在管理列表，我们通常不显示完整的 API Key 而是脱敏
	c.JSON(http.StatusOK, instances)
}

// handleGetApps 获取所有前台项目
func handleGetApps(c *gin.Context) {
	var apps []model.Application
	repository.DB.Find(&apps)
	c.JSON(http.StatusOK, apps)
}

// handleUpdateApp 更新应用配置（演示简化版：主要是UI配置与绑定的实例）
func handleUpdateApp(c *gin.Context) {
    id := c.Param("id")
    var req struct {
        UIConfig string `json:"uiConfig"`
    }
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid fields"})
        return
    }

    var app model.Application
    if err := repository.DB.First(&app, id).Error; err != nil {
        c.JSON(http.StatusNotFound, gin.H{"error": "App not found"})
        return
    }

    app.UIConfig = req.UIConfig
    repository.DB.Save(&app)
    c.JSON(http.StatusOK, app)
}
