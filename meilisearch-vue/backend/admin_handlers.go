package main

import (
	"net/http"
	"strings"

	"backend/model"
	"backend/repository"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/crypto/bcrypt"
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

// handleGetUsers 获取系统用户
func handleGetUsers(c *gin.Context) {
	var users []model.User
	repository.DB.Find(&users)
	c.JSON(http.StatusOK, users)
}

// handleCreateUser 创建普通用户
func handleCreateUser(c *gin.Context) {
	var req struct {
		Username     string `json:"username"`
		Password     string `json:"password"`
		Role         string `json:"role"`
		AllowIndexes string `json:"allowIndexes"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid input"})
		return
	}

	hash, _ := bcrypt.GenerateFromPassword([]byte(req.Password), bcrypt.DefaultCost)
	user := model.User{
		Username:     req.Username,
		PasswordHash: string(hash),
		Role:         req.Role,
		AllowIndexes: req.AllowIndexes,
	}
	if err := repository.DB.Create(&user).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Username might exist"})
		return
	}
	c.JSON(http.StatusOK, user)
}

// handleUpdateUserPermissions 更新用户权限
func handleUpdateUserPermissions(c *gin.Context) {
	id := c.Param("id")
	var req struct {
		Role         string `json:"role"`
		AllowIndexes string `json:"allowIndexes"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid payload"})
		return
	}
	var user model.User
	if err := repository.DB.First(&user, id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "User not found"})
		return
	}
	user.Role = req.Role
	user.AllowIndexes = req.AllowIndexes
	repository.DB.Save(&user)
	c.JSON(http.StatusOK, user)
}

