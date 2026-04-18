package api

import (
	"net/http"

	"backend/model"
	"backend/repository"
	"backend/schema"

	"github.com/gin-gonic/gin"
	"golang.org/x/crypto/bcrypt"
)

// HandleGetInstances 获取所有实例信息
func HandleGetInstances(c *gin.Context) {
	var instances []model.MeiliInstance
	repository.DB.Find(&instances)
	c.JSON(http.StatusOK, instances)
}

// HandleGetApps 获取所有前台项目
func HandleGetApps(c *gin.Context) {
	var apps []model.Application
	repository.DB.Find(&apps)
	c.JSON(http.StatusOK, apps)
}

// HandleUpdateApp 更新应用UI配置
func HandleUpdateApp(c *gin.Context) {
	id := c.Param("id")
	var req schema.AppUpdateRequest
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

// HandleGetUsers 获取系统用户
func HandleGetUsers(c *gin.Context) {
	var users []model.User
	repository.DB.Find(&users)
	c.JSON(http.StatusOK, users)
}

// HandleCreateUser 创建普通用户
func HandleCreateUser(c *gin.Context) {
	var req schema.UserCreateRequest
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

// HandleUpdateUserPermissions 更新用户权限
func HandleUpdateUserPermissions(c *gin.Context) {
	id := c.Param("id")
	var req schema.UserPermRequest
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
