package api

import (
	"backend/model"
	"backend/repository"
	"backend/schema"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/meilisearch/meilisearch-go"
	"golang.org/x/crypto/bcrypt"
)

func HandleDeleteIndex(c *gin.Context) {
	uid := c.Param("uid")
	if uid == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "UID is required"})
		return
	}

	// 1. 从数据库中删除配置
	repository.DB.Where("uid = ?", uid).Delete(&model.IndexConfig{})

	// 2. 尝试从 Meilisearch 中删除 (使用第一个实例)
	var instance model.MeiliInstance
	if err := repository.DB.First(&instance).Error; err == nil {
		client := meilisearch.New(instance.Host, meilisearch.WithAPIKey(instance.APIKey))
		client.DeleteIndex(uid)
	}

	c.JSON(http.StatusOK, gin.H{"message": "Index deleted successfully from DB and Meilisearch"})
}

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
			Uid:                req.Uid,
			Alias:              req.Alias,
			Description:        req.Description,
			IsVisible:          req.IsVisible,
			IsLocked:           req.IsLocked,
			FieldConfigs:       req.FieldConfigs,
			ViewConfigs:        req.ViewConfigs,
			TableConfigs:       req.TableConfigs,
			CanEdit:            req.CanEdit,
			NestedFieldConfigs: req.NestedFieldConfigs,
		}
		repository.DB.Create(&config)
	} else {
		// Update
		config.Alias = req.Alias
		config.Description = req.Description
		config.IsVisible = req.IsVisible
		config.IsLocked = req.IsLocked
		config.FieldConfigs = req.FieldConfigs
		config.ViewConfigs = req.ViewConfigs
		config.TableConfigs = req.TableConfigs
		config.CanEdit = req.CanEdit
		config.NestedFieldConfigs = req.NestedFieldConfigs
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
		ExpiresAt:    req.ExpiresAt,
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
	tok.ExpiresAt = req.ExpiresAt
	repository.DB.Save(&tok)
	c.JSON(http.StatusOK, tok)
}

func HandleDeleteAccessToken(c *gin.Context) {
	id := c.Param("id")
	repository.DB.Delete(&model.AccessToken{}, id)
	c.JSON(http.StatusOK, gin.H{"success": true})
}

// HandleUpdateAdminPassword 修改管理员密码
func HandleUpdateAdminPassword(c *gin.Context) {
	var req schema.PasswordUpdateRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的参数"})
		return
	}

	var user model.User
	// 默认修改 admin 账户
	if err := repository.DB.Where("username = ?", "admin").First(&user).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "管理员用户未找到"})
		return
	}

	hash, err := bcrypt.GenerateFromPassword([]byte(req.NewPassword), bcrypt.DefaultCost)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "加密失败"})
		return
	}

	user.PasswordHash = string(hash)
	if err := repository.DB.Save(&user).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "保存失败"})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "密码修改成功"})
}

// HandleSaveNestedFieldConfigs 单独更新嵌套查看器的字段配置，不影响其他索引设置
func HandleSaveNestedFieldConfigs(c *gin.Context) {
	var req schema.NestedFieldConfigsUpdateRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	var config model.IndexConfig
	if err := repository.DB.Where("uid = ?", req.Uid).First(&config).Error; err != nil {
		// NOTE: 该索引尚无完整 IndexConfig 记录，创建一条只含 uid 和嵌套配置的记录
		config = model.IndexConfig{
			Uid:                req.Uid,
			NestedFieldConfigs: req.NestedFieldConfigs,
		}
		repository.DB.Create(&config)
	} else {
		// 仅更新 NestedFieldConfigs 字段，避免覆盖其他配置
		repository.DB.Model(&config).Update("nested_field_configs", req.NestedFieldConfigs)
	}

	c.JSON(http.StatusOK, gin.H{"uid": req.Uid, "nestedFieldConfigs": req.NestedFieldConfigs})
}

// HandleSaveTableConfigs 单独更新表格列配置（顺序/隐藏），不覆盖其他索引配置
func HandleSaveTableConfigs(c *gin.Context) {
	var req schema.TableConfigsUpdateRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	var config model.IndexConfig
	if err := repository.DB.Where("uid = ?", req.Uid).First(&config).Error; err != nil {
		config = model.IndexConfig{
			Uid:          req.Uid,
			TableConfigs: req.TableConfigs,
		}
		repository.DB.Create(&config)
	} else {
		repository.DB.Model(&config).Update("table_configs", req.TableConfigs)
	}

	c.JSON(http.StatusOK, gin.H{"uid": req.Uid, "tableConfigs": req.TableConfigs})
}

type DrawerFieldOrderRequest struct {
	Uid              string `json:"uid" binding:"required"`
	DrawerFieldOrder string `json:"drawerFieldOrder" binding:"required"`
}

func HandleSaveDrawerFieldOrder(c *gin.Context) {
	var req DrawerFieldOrderRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	var config model.IndexConfig
	if err := repository.DB.Where("uid = ?", req.Uid).First(&config).Error; err != nil {
		config = model.IndexConfig{
			Uid:              req.Uid,
			DrawerFieldOrder: req.DrawerFieldOrder,
		}
		repository.DB.Create(&config)
	} else {
		repository.DB.Model(&config).Update("drawer_field_order", req.DrawerFieldOrder)
	}

	c.JSON(http.StatusOK, gin.H{"uid": req.Uid, "drawerFieldOrder": req.DrawerFieldOrder})
}

// HandleGetIndexSettings 获取索引的 Meilisearch 设置
func HandleGetIndexSettings(c *gin.Context) {
	uid := c.Param("uid")
	if uid == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "UID is required"})
		return
	}

	var instance model.MeiliInstance
	if err := repository.DB.First(&instance).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Meilisearch instance not found"})
		return
	}

	client := meilisearch.New(instance.Host, meilisearch.WithAPIKey(instance.APIKey))
	index := client.Index(uid)
	settings, err := index.GetSettings()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, settings)
}

// HandleUpdateIndexSettings 更新索引的 Meilisearch 设置
func HandleUpdateIndexSettings(c *gin.Context) {
	var req schema.IndexSettingsRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	var instance model.MeiliInstance
	if err := repository.DB.First(&instance).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Meilisearch instance not found"})
		return
	}

	client := meilisearch.New(instance.Host, meilisearch.WithAPIKey(instance.APIKey))
	index := client.Index(req.Uid)

	settings := &meilisearch.Settings{}
	if req.SearchableAttributes != nil {
		settings.SearchableAttributes = *req.SearchableAttributes
	}
	if req.FilterableAttributes != nil {
		settings.FilterableAttributes = *req.FilterableAttributes
	}
	if req.Embedders != nil {
		raw, err := json.Marshal(req.Embedders)
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid embedders payload"})
			return
		}
		embedders := make(map[string]meilisearch.Embedder)
		if err := json.Unmarshal(raw, &embedders); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid embedders payload"})
			return
		}
		settings.Embedders = embedders
	}
	task, err := index.UpdateSettings(settings)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, task)
}

func HandleValidateEmbedder(c *gin.Context) {
	uid := c.Param("uid")
	var req schema.ValidateEmbedderRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}
	if req.Uid != uid {
		c.JSON(http.StatusBadRequest, gin.H{"error": "UID mismatch"})
		return
	}

	raw, err := json.Marshal(req.Embedder)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid embedder payload"})
		return
	}
	var emb meilisearch.Embedder
	if err := json.Unmarshal(raw, &emb); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid embedder payload"})
		return
	}

	if err := validateEmbedderConfig(emb); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := probeEmbedderAvailability(emb); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"ok": true})
}

func validateEmbedderConfig(emb meilisearch.Embedder) error {
	source := strings.TrimSpace(string(emb.Source))
	if source == "" {
		return fmt.Errorf("source is required")
	}
	switch source {
	case "openAi":
		if strings.TrimSpace(emb.Model) == "" {
			return fmt.Errorf("openAi source requires model")
		}
		if strings.TrimSpace(emb.APIKey) == "" {
			return fmt.Errorf("openAi source requires apiKey")
		}
	case "huggingFace":
		if strings.TrimSpace(emb.Model) == "" {
			return fmt.Errorf("huggingFace source requires model")
		}
	case "ollama":
		if strings.TrimSpace(emb.Model) == "" {
			return fmt.Errorf("ollama source requires model")
		}
	case "rest":
		if strings.TrimSpace(emb.URL) == "" {
			return fmt.Errorf("rest source requires url")
		}
	case "userProvided":
		if emb.Dimensions <= 0 {
			return fmt.Errorf("userProvided source requires dimensions")
		}
	default:
		return fmt.Errorf("unsupported source: %s", source)
	}
	return nil
}

func probeEmbedderAvailability(emb meilisearch.Embedder) error {
	source := strings.TrimSpace(string(emb.Source))
	switch source {
	case "openAi":
		targetURL := strings.TrimSpace(emb.URL)
		if targetURL == "" {
			targetURL = "https://api.openai.com/v1/embeddings"
		}
		payload := map[string]any{"model": emb.Model, "input": "ping"}
		return doJSONProbe(targetURL, emb.APIKey, payload)
	case "ollama":
		targetURL := strings.TrimSpace(emb.URL)
		if targetURL == "" {
			targetURL = "http://localhost:11434/api/embeddings"
		}
		payload := map[string]any{"model": emb.Model, "prompt": "ping"}
		return doJSONProbe(targetURL, emb.APIKey, payload)
	case "huggingFace":
		targetURL := "https://api-inference.huggingface.co/models/" + strings.TrimSpace(emb.Model)
		payload := map[string]any{"inputs": "ping"}
		return doJSONProbe(targetURL, emb.APIKey, payload)
	case "rest":
		// 对于自定义 REST，只验证 URL 可访问和返回 2xx
		payload := map[string]any{"input": "ping"}
		return doJSONProbe(strings.TrimSpace(emb.URL), emb.APIKey, payload)
	case "userProvided":
		// userProvided 由上游提供向量，无法在服务端主动探测模型可用性
		return nil
	default:
		return fmt.Errorf("unsupported source: %s", source)
	}
}

func doJSONProbe(targetURL string, apiKey string, payload map[string]any) error {
	body, _ := json.Marshal(payload)
	req, err := http.NewRequest(http.MethodPost, targetURL, bytes.NewReader(body))
	if err != nil {
		return fmt.Errorf("probe request build failed: %v", err)
	}
	req.Header.Set("Content-Type", "application/json")
	if strings.TrimSpace(apiKey) != "" {
		req.Header.Set("Authorization", "Bearer "+strings.TrimSpace(apiKey))
	}

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return fmt.Errorf("probe failed: %v", err)
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		respBody, _ := io.ReadAll(io.LimitReader(resp.Body, 512))
		return fmt.Errorf("probe failed with status %d: %s", resp.StatusCode, strings.TrimSpace(string(respBody)))
	}
	return nil
}
