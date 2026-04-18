package api

import (
	"net/http"

	"backend/model"
	"backend/repository"
	"backend/schema"

	"github.com/gin-gonic/gin"
)

func HandleGetInstances(c *gin.Context) {
	var instances []model.MeiliInstance
	repository.DB.Find(&instances)
	c.JSON(http.StatusOK, instances)
}

func HandleCreateInstance(c *gin.Context) {
	var req schema.MeiliInstanceRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid data"})
		return
	}
	ins := model.MeiliInstance{
		Name:   req.Name,
		Host:   req.Host,
		APIKey: req.APIKey,
	}
	if err := repository.DB.Create(&ins).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create instance"})
		return
	}
	c.JSON(http.StatusOK, ins)
}

func HandleUpdateInstance(c *gin.Context) {
	var req schema.MeiliInstanceUpdateRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid data"})
		return
	}
	var ins model.MeiliInstance
	if err := repository.DB.First(&ins, req.ID).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "Instance not found"})
		return
	}
	if req.Name != "" {
		ins.Name = req.Name
	}
	if req.Host != "" {
		ins.Host = req.Host
	}
	if req.APIKey != "" {
		ins.APIKey = req.APIKey
	}
	repository.DB.Save(&ins)
	c.JSON(http.StatusOK, ins)
}

func HandleDeleteInstance(c *gin.Context) {
	id := c.Param("id")
	repository.DB.Delete(&model.MeiliInstance{}, id)
	c.JSON(http.StatusOK, gin.H{"success": true})
}

func HandleGetApps(c *gin.Context) {
	var apps []model.Application
	repository.DB.Find(&apps)
	c.JSON(http.StatusOK, apps)
}

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

// ---- Index Settings ----
func HandleGetIndexConfigs(c *gin.Context) {
	var configs []model.IndexConfig
	repository.DB.Find(&configs)
	c.JSON(http.StatusOK, configs)
}

func HandleSaveIndexConfig(c *gin.Context) {
	var req schema.IndexConfigRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}
	var config model.IndexConfig
	if err := repository.DB.Where("uid = ?", req.Uid).First(&config).Error; err != nil {
		// Create new
		config = model.IndexConfig{
			Uid:          req.Uid,
			Alias:        req.Alias,
			Description:  req.Description,
			IsLocked:     req.IsLocked,
			FieldConfigs: req.FieldConfigs,
			ViewConfigs:  req.ViewConfigs,
			TableConfigs: req.TableConfigs,
			CanEdit:      req.CanEdit,
		}
		repository.DB.Create(&config)
	} else {
		// Update
		config.Alias = req.Alias
		config.Description = req.Description
		config.IsLocked = req.IsLocked
		config.FieldConfigs = req.FieldConfigs
		config.ViewConfigs = req.ViewConfigs
		config.TableConfigs = req.TableConfigs
		config.CanEdit = req.CanEdit
		repository.DB.Save(&config)
	}
	c.JSON(http.StatusOK, config)
}

// ---- Access Tokens ----
func HandleGetAccessTokens(c *gin.Context) {
	var tokens []model.AccessToken
	repository.DB.Find(&tokens)
	c.JSON(http.StatusOK, tokens)
}

func HandleCreateAccessToken(c *gin.Context) {
	var req schema.AccessTokenRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid data"})
		return
	}
	tok := model.AccessToken{
		Token:        req.Token,
		AllowIndexes: req.AllowIndexes,
		Description:  req.Description,
	}
	if err := repository.DB.Create(&tok).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Token already exists"})
		return
	}
	c.JSON(http.StatusOK, tok)
}

func HandleUpdateAccessToken(c *gin.Context) {
	var req schema.AccessTokenUpdateRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid data"})
		return
	}
	var tok model.AccessToken
	if err := repository.DB.First(&tok, req.ID).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "Token not found"})
		return
	}
	tok.Token = req.Token
	tok.AllowIndexes = req.AllowIndexes
	tok.Description = req.Description
	repository.DB.Save(&tok)
	c.JSON(http.StatusOK, tok)
}

func HandleDeleteAccessToken(c *gin.Context) {
	id := c.Param("id")
	repository.DB.Delete(&model.AccessToken{}, id)
	c.JSON(http.StatusOK, gin.H{"success": true})
}
