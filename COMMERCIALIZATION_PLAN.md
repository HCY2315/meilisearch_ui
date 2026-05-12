# 🚀 MeiliSearch UI 商业化改造方案 (Go 后端版)

## 📋 核心优势分析

### 你现有系统的独特卖点：
```
✅ GUID-based 多租户架构  → 天然支持 SaaS 模式
✅ 索引级别的私有化控制   → 满足数据隐私合规需求
✅ Go 后端 + Meilisearch   → 高性能、低成本、易维护
✅ 前后端完整实现          → 可直接商业部署
✅ 配置级权限管理          → 细粒度的功能订阅
```

---

## 🎯 Phase 1: SaaS 基础设施升级（0-2个月）

### 1.1 多租户完全隔离

**当前状态：** Bearer Token 认证 + Index 级别隔离

**Go 后端改造方案：**

#### 步骤1：定义租户上下文

```go
// internal/models/tenant.go
package models

import "time"

type TenantTier string

const (
    TierFree       TenantTier = "free"
    TierPro        TenantTier = "pro"
    TierBusiness   TenantTier = "business"
    TierEnterprise TenantTier = "enterprise"
)

// Tenant 租户基本信息
type Tenant struct {
    GUID      string    `gorm:"primaryKey" json:"guid"`
    Name      string    `json:"name"`
    Email     string    `json:"email"`
    Tier      TenantTier `json:"tier"`
    CreatedAt time.Time `json:"created_at"`
    UpdatedAt time.Time `json:"updated_at"`
    Status    string    `json:"status"` // active, suspended, cancelled
}

// TenantQuotas 租户配额限制
type TenantQuotas struct {
    TenantGUID         string `gorm:"primaryKey"`
    MaxQueriesPerMonth int
    MaxIndexes         int
    MaxDocumentsPerIndex int64
    MaxStorageGB       int
    APIRateLimit       int    // QPS
    CreatedAt          time.Time
    UpdatedAt          time.Time
}

// TenantFeatures 租户功能开关
type TenantFeatures struct {
    TenantGUID       string `gorm:"primaryKey"`
    Customization    bool   // 字段配置权限
    Export           bool   // CSV导出
    AISearch         bool   // AI搜索
    Webhooks         bool   // Webhook集成
    APIKeys          bool   // 多API密钥
    CustomDomain     bool   // 自定义域名
    CreatedAt        time.Time
    UpdatedAt        time.Time
}

// TenantContext 请求上下文中的租户信息
type TenantContext struct {
    GUID     string
    Tier     TenantTier
    Quotas   *TenantQuotas
    Features *TenantFeatures
}
```

#### 步骤2：创建租户管理服务

```go
// internal/service/tenant_service.go
package service

import (
    "context"
    "errors"
    "fmt"
    "time"
    "github.com/google/uuid"
    "gorm.io/gorm"
    "your-app/internal/models"
)

type TenantService struct {
    db *gorm.DB
}

// NewTenantService 创建租户服务
func NewTenantService(db *gorm.DB) *TenantService {
    return &TenantService{db: db}
}

// CreateTenant 创建新租户
func (s *TenantService) CreateTenant(ctx context.Context, req CreateTenantRequest) (*models.Tenant, error) {
    tenant := &models.Tenant{
        GUID:      uuid.New().String(),
        Name:      req.Name,
        Email:     req.Email,
        Tier:      models.TierFree,
        Status:    "active",
        CreatedAt: time.Now(),
        UpdatedAt: time.Now(),
    }

    if err := s.db.WithContext(ctx).Create(tenant).Error; err != nil {
        return nil, fmt.Errorf("failed to create tenant: %w", err)
    }

    // 创建默认配额
    quotas := &models.TenantQuotas{
        TenantGUID:           tenant.GUID,
        MaxQueriesPerMonth:   10000,     // Free 限额
        MaxIndexes:           2,
        MaxDocumentsPerIndex: 100000,
        MaxStorageGB:         1,
        APIRateLimit:         100,       // 100 QPS
        CreatedAt:            time.Now(),
        UpdatedAt:            time.Now(),
    }
    if err := s.db.WithContext(ctx).Create(quotas).Error; err != nil {
        return nil, fmt.Errorf("failed to create quotas: %w", err)
    }

    // 创建默认功能配置
    features := &models.TenantFeatures{
        TenantGUID:    tenant.GUID,
        Customization: true,
        Export:        false,         // Free 不支持导出
        AISearch:      false,
        Webhooks:      false,
        APIKeys:       false,
        CustomDomain:  false,
        CreatedAt:     time.Now(),
        UpdatedAt:     time.Now(),
    }
    if err := s.db.WithContext(ctx).Create(features).Error; err != nil {
        return nil, fmt.Errorf("failed to create features: %w", err)
    }

    return tenant, nil
}

// GetTenantContext 获取完整的租户上下文
func (s *TenantService) GetTenantContext(ctx context.Context, guid string) (*models.TenantContext, error) {
    tenant := &models.Tenant{}
    if err := s.db.WithContext(ctx).First(tenant, "guid = ?", guid).Error; err != nil {
        return nil, fmt.Errorf("tenant not found: %w", err)
    }

    if tenant.Status != "active" {
        return nil, errors.New("tenant is not active")
    }

    quotas := &models.TenantQuotas{}
    if err := s.db.WithContext(ctx).First(quotas, "tenant_guid = ?", guid).Error; err != nil {
        return nil, fmt.Errorf("quotas not found: %w", err)
    }

    features := &models.TenantFeatures{}
    if err := s.db.WithContext(ctx).First(features, "tenant_guid = ?", guid).Error; err != nil {
        return nil, fmt.Errorf("features not found: %w", err)
    }

    return &models.TenantContext{
        GUID:     guid,
        Tier:     tenant.Tier,
        Quotas:   quotas,
        Features: features,
    }, nil
}

// UpgradeTenant 升级租户套餐
func (s *TenantService) UpgradeTenant(ctx context.Context, guid string, newTier models.TenantTier) error {
    tenant := &models.Tenant{}
    if err := s.db.WithContext(ctx).First(tenant, "guid = ?", guid).Error; err != nil {
        return fmt.Errorf("tenant not found: %w", err)
    }

    tenant.Tier = newTier
    tenant.UpdatedAt = time.Now()

    // 更新配额
    quotas := &models.TenantQuotas{}
    if err := s.db.WithContext(ctx).First(quotas, "tenant_guid = ?", guid).Error; err != nil {
        return fmt.Errorf("quotas not found: %w", err)
    }

    switch newTier {
    case models.TierFree:
        quotas.MaxQueriesPerMonth = 10000
        quotas.MaxIndexes = 2
        quotas.APIRateLimit = 100
    case models.TierPro:
        quotas.MaxQueriesPerMonth = 1000000
        quotas.MaxIndexes = 20
        quotas.APIRateLimit = 1000
    case models.TierBusiness:
        quotas.MaxQueriesPerMonth = 10000000
        quotas.MaxIndexes = 100
        quotas.APIRateLimit = 10000
    case models.TierEnterprise:
        quotas.MaxQueriesPerMonth = 999999999
        quotas.MaxIndexes = 999999
        quotas.APIRateLimit = 999999
    }
    quotas.UpdatedAt = time.Now()

    // 更新功能开关
    features := &models.TenantFeatures{}
    if err := s.db.WithContext(ctx).First(features, "tenant_guid = ?", guid).Error; err != nil {
        return fmt.Errorf("features not found: %w", err)
    }

    switch newTier {
    case models.TierFree:
        features.Export = false
        features.AISearch = false
        features.Webhooks = false
        features.APIKeys = false
    case models.TierPro:
        features.Export = true
        features.AISearch = true
        features.Webhooks = false
        features.APIKeys = true
    case models.TierBusiness, models.TierEnterprise:
        features.Export = true
        features.AISearch = true
        features.Webhooks = true
        features.APIKeys = true
        features.CustomDomain = true
    }
    features.UpdatedAt = time.Now()

    return s.db.WithContext(ctx).Transaction(func(tx *gorm.DB) error {
        if err := tx.Save(tenant).Error; err != nil {
            return err
        }
        if err := tx.Save(quotas).Error; err != nil {
            return err
        }
        if err := tx.Save(features).Error; err != nil {
            return err
        }
        return nil
    })
}

type CreateTenantRequest struct {
    Name  string `json:"name"`
    Email string `json:"email"`
}
```

#### 步骤3：创建中间件验证 GUID

```go
// internal/middleware/tenant_middleware.go
package middleware

import (
    "context"
    "net/http"
    "github.com/gin-gonic/gin"
    "your-app/internal/models"
    "your-app/internal/service"
)

type TenantMiddleware struct {
    tenantService *service.TenantService
}

// NewTenantMiddleware 创建租户中间件
func NewTenantMiddleware(ts *service.TenantService) *TenantMiddleware {
    return &TenantMiddleware{tenantService: ts}
}

// AuthTenant 验证请求中的租户 GUID
func (m *TenantMiddleware) AuthTenant() gin.HandlerFunc {
    return func(c *gin.Context) {
        // 从请求头获取 GUID
        guid := c.GetHeader("X-Tenant-GUID")
        if guid == "" {
            c.JSON(http.StatusUnauthorized, gin.H{"error": "missing X-Tenant-GUID"})
            c.Abort()
            return
        }

        // 获取租户上下文
        ctx := context.Background()
        tenantCtx, err := m.tenantService.GetTenantContext(ctx, guid)
        if err != nil {
            c.JSON(http.StatusUnauthorized, gin.H{"error": "invalid tenant"})
            c.Abort()
            return
        }

        // 将租户信息存储在上下文中
        c.Set("tenant", tenantCtx)
        c.Set("tenant_guid", guid)

        c.Next()
    }
}

// CheckFeature 检查租户是否具有某个功能
func (m *TenantMiddleware) CheckFeature(featureName string) gin.HandlerFunc {
    return func(c *gin.Context) {
        tenantCtx, exists := c.Get("tenant")
        if !exists {
            c.JSON(http.StatusUnauthorized, gin.H{"error": "tenant context not found"})
            c.Abort()
            return
        }

        tenant := tenantCtx.(*models.TenantContext)

        var hasFeature bool
        switch featureName {
        case "export":
            hasFeature = tenant.Features.Export
        case "ai_search":
            hasFeature = tenant.Features.AISearch
        case "webhooks":
            hasFeature = tenant.Features.Webhooks
        case "api_keys":
            hasFeature = tenant.Features.APIKeys
        case "custom_domain":
            hasFeature = tenant.Features.CustomDomain
        default:
            hasFeature = false
        }

        if !hasFeature {
            c.JSON(http.StatusForbidden, gin.H{
                "error": "feature not available in your plan",
                "feature": featureName,
            })
            c.Abort()
            return
        }

        c.Next()
    }
}
```

---

### 1.2 用量统计和配额管理

#### 数据库表设计

```go
// internal/models/usage.go
package models

import "time"

// UsageMetric 使用量统计
type UsageMetric struct {
    ID              uint      `gorm:"primaryKey"`
    TenantGUID      string    `gorm:"index"`
    MetricDate      time.Time `gorm:"index"`
    QueriesCount    int       `json:"queries_count"`
    DocumentsImported int     `json:"documents_imported"`
    StorageBytes    int64     `json:"storage_bytes"`
    ExportsCount    int       `json:"exports_count"`
    APICallsCount   int       `json:"api_calls_count"`
    CreatedAt       time.Time
    UpdatedAt       time.Time
}

// IndexAccess 索引访问权限
type IndexAccess struct {
    IndexUID  string `gorm:"primaryKey"`
    TenantGUID string `gorm:"primaryKey"`
    Permission string // read, write, admin
    CreatedAt  time.Time
    UpdatedAt  time.Time
}

// APIKey API 密钥（多密钥支持）
type APIKey struct {
    ID         string    `gorm:"primaryKey"`
    TenantGUID string    `gorm:"index"`
    Name       string
    KeyHash    string    // SHA256 hash
    Permissions []string // JSON array
    ExpiresAt  *time.Time
    CreatedAt  time.Time
    UpdatedAt  time.Time
}
```

#### 配额检查服务

```go
// internal/service/quota_service.go
package service

import (
    "context"
    "fmt"
    "time"
    "gorm.io/gorm"
    "your-app/internal/models"
)

type QuotaService struct {
    db *gorm.DB
}

// NewQuotaService 创建配额服务
func NewQuotaService(db *gorm.DB) *QuotaService {
    return &QuotaService{db: db}
}

// CheckQueryQuota 检查查询配额
func (s *QuotaService) CheckQueryQuota(ctx context.Context, guid string, tier models.TenantTier) error {
    today := time.Now().Format("2006-01-02")
    
    metric := &models.UsageMetric{}
    err := s.db.WithContext(ctx).Where("tenant_guid = ? AND DATE(metric_date) = ?", guid, today).
        First(metric).Error

    if err != nil && err != gorm.ErrRecordNotFound {
        return fmt.Errorf("failed to check quota: %w", err)
    }

    var limit int
    switch tier {
    case models.TierFree:
        limit = 10000
    case models.TierPro:
        limit = 1000000
    case models.TierBusiness:
        limit = 10000000
    case models.TierEnterprise:
        limit = 999999999
    default:
        limit = 0
    }

    if metric.QueriesCount >= limit {
        return fmt.Errorf("query quota exceeded: %d/%d", metric.QueriesCount, limit)
    }

    return nil
}

// RecordQuery 记录查询
func (s *QuotaService) RecordQuery(ctx context.Context, guid string) error {
    today := time.Now()
    todayDate := today.Format("2006-01-02")

    metric := &models.UsageMetric{}
    result := s.db.WithContext(ctx).
        Where("tenant_guid = ? AND DATE(metric_date) = ?", guid, todayDate).
        First(metric)

    if result.Error == gorm.ErrRecordNotFound {
        // 创建新记录
        metric = &models.UsageMetric{
            TenantGUID:   guid,
            MetricDate:   today,
            QueriesCount: 1,
            CreatedAt:    time.Now(),
            UpdatedAt:    time.Now(),
        }
        return s.db.WithContext(ctx).Create(metric).Error
    } else if result.Error != nil {
        return fmt.Errorf("failed to record query: %w", result.Error)
    }

    // 增加计数
    metric.QueriesCount++
    metric.UpdatedAt = time.Now()
    return s.db.WithContext(ctx).Save(metric).Error
}

// GetTodayUsage 获取今天的使用情况
func (s *QuotaService) GetTodayUsage(ctx context.Context, guid string) (*models.UsageMetric, error) {
    today := time.Now().Format("2006-01-02")
    
    metric := &models.UsageMetric{}
    err := s.db.WithContext(ctx).
        Where("tenant_guid = ? AND DATE(metric_date) = ?", guid, today).
        First(metric).Error

    if err == gorm.ErrRecordNotFound {
        return &models.UsageMetric{
            TenantGUID: guid,
            MetricDate: time.Now(),
        }, nil
    }

    return metric, err
}

// GetMonthlyUsage 获取本月的使用情况
func (s *QuotaService) GetMonthlyUsage(ctx context.Context, guid string) (*models.UsageMetric, error) {
    now := time.Now()
    startOfMonth := time.Date(now.Year(), now.Month(), 1, 0, 0, 0, 0, now.Location())

    metric := &models.UsageMetric{}
    err := s.db.WithContext(ctx).
        Where("tenant_guid = ? AND metric_date >= ?", guid, startOfMonth).
        First(metric).Error

    if err == gorm.ErrRecordNotFound {
        return &models.UsageMetric{
            TenantGUID: guid,
            MetricDate: now,
        }, nil
    }

    return metric, err
}
```

#### 在搜索中使用配额检查

```go
// internal/handler/search_handler.go
package handler

import (
    "net/http"
    "github.com/gin-gonic/gin"
    "your-app/internal/models"
    "your-app/internal/service"
)

type SearchHandler struct {
    quotaService *service.QuotaService
    tenantService *service.TenantService
}

// Search 执行搜索查询
func (h *SearchHandler) Search(c *gin.Context) {
    // 从上下文获取租户信息
    tenantCtx, _ := c.Get("tenant")
    tenant := tenantCtx.(*models.TenantContext)

    // ✅ 第1步：检查配额
    err := h.quotaService.CheckQueryQuota(c.Request.Context(), tenant.GUID, tenant.Tier)
    if err != nil {
        c.JSON(http.StatusPaymentRequired, gin.H{
            "error": "quota exceeded",
            "detail": err.Error(),
        })
        return
    }

    // ✅ 第2步：记录使用量
    h.quotaService.RecordQuery(c.Request.Context(), tenant.GUID)

    // 第3步：执行搜索（原有逻辑）
    query := c.Query("q")
    // ... 调用 Meilisearch 进行搜索
    
    c.JSON(http.StatusOK, gin.H{"results": "..."})
}
```

---

### 1.3 动态定价和订阅管理

#### API 端点设计

```go
// internal/handler/billing_handler.go
package handler

import (
    "net/http"
    "github.com/gin-gonic/gin"
    "your-app/internal/models"
    "your-app/internal/service"
)

type BillingHandler struct {
    tenantService *service.TenantService
    quotaService  *service.QuotaService
}

// GetBillingInfo 获取计费信息
func (h *BillingHandler) GetBillingInfo(c *gin.Context) {
    guid := c.GetString("tenant_guid")
    tenantCtx, _ := c.Get("tenant")
    tenant := tenantCtx.(*models.TenantContext)

    // 获取本月用量
    monthlyUsage, err := h.quotaService.GetMonthlyUsage(c.Request.Context(), guid)
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to get usage"})
        return
    }

    // 计算账单
    price := getPriceByTier(tenant.Tier)
    overageCharge := calculateOverage(tenant.Quotas, monthlyUsage)

    c.JSON(http.StatusOK, gin.H{
        "tier": tenant.Tier,
        "base_price": price,
        "usage": gin.H{
            "queries": monthlyUsage.QueriesCount,
            "storage": monthlyUsage.StorageBytes,
            "exports": monthlyUsage.ExportsCount,
        },
        "quotas": gin.H{
            "max_queries": tenant.Quotas.MaxQueriesPerMonth,
            "max_storage": tenant.Quotas.MaxStorageGB,
        },
        "overage_charge": overageCharge,
        "total_charge": price + overageCharge,
    })
}

// UpgradePlan 升级套餐
func (h *BillingHandler) UpgradePlan(c *gin.Context) {
    guid := c.GetString("tenant_guid")

    var req struct {
        NewTier string `json:"new_tier"`
    }
    if err := c.BindJSON(&req); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": "invalid request"})
        return
    }

    // 验证套餐有效性
    newTier := models.TenantTier(req.NewTier)
    if newTier != models.TierFree && newTier != models.TierPro && 
       newTier != models.TierBusiness && newTier != models.TierEnterprise {
        c.JSON(http.StatusBadRequest, gin.H{"error": "invalid tier"})
        return
    }

    // 更新租户
    if err := h.tenantService.UpgradeTenant(c.Request.Context(), guid, newTier); err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "upgrade failed"})
        return
    }

    c.JSON(http.StatusOK, gin.H{"message": "upgrade successful", "tier": newTier})
}

// 定价函数
func getPriceByTier(tier models.TenantTier) int {
    switch tier {
    case models.TierFree:
        return 0
    case models.TierPro:
        return 9900    // ¥99/月
    case models.TierBusiness:
        return 29900   // ¥299/月
    case models.TierEnterprise:
        return 99900   // ¥999/月
    default:
        return 0
    }
}

// 计算超额费用
func calculateOverage(quotas *models.TenantQuotas, usage *models.UsageMetric) int {
    var charge int

    // 查询超额：¥0.01 per 1000 queries
    if usage.QueriesCount > quotas.MaxQueriesPerMonth {
        excess := usage.QueriesCount - quotas.MaxQueriesPerMonth
        charge += excess / 1000  // 单位：分
    }

    // 存储超额：¥0.1 per GB
    usageGB := usage.StorageBytes / (1024 * 1024 * 1024)
    if int(usageGB) > quotas.MaxStorageGB {
        excess := int(usageGB) - quotas.MaxStorageGB
        charge += excess * 10   // 10分 per GB
    }

    return charge
}
```

---

## 🎯 Phase 2: SaaS 核心功能（2-3个月）

### 2.1 API Keys 管理（多密钥支持）

```go
// internal/service/api_key_service.go
package service

import (
    "context"
    "crypto/sha256"
    "encoding/hex"
    "fmt"
    "time"
    "github.com/google/uuid"
    "gorm.io/gorm"
    "your-app/internal/models"
)

type APIKeyService struct {
    db *gorm.DB
}

// NewAPIKeyService 创建 API Key 服务
func NewAPIKeyService(db *gorm.DB) *APIKeyService {
    return &APIKeyService{db: db}
}

// CreateAPIKey 创建新的 API Key
func (s *APIKeyService) CreateAPIKey(ctx context.Context, guid string, name string) (string, error) {
    // 生成随机密钥
    randomPart := uuid.New().String()
    apiKey := fmt.Sprintf("sk_live_%s_%s", guid[:8], randomPart[:16])

    // 计算 SHA256 hash（只存储 hash）
    hash := sha256.Sum256([]byte(apiKey))
    keyHash := hex.EncodeToString(hash[:])

    record := &models.APIKey{
        ID:         uuid.New().String(),
        TenantGUID: guid,
        Name:       name,
        KeyHash:    keyHash,
        Permissions: []string{"search", "read"},
        CreatedAt:  time.Now(),
        UpdatedAt:  time.Now(),
    }

    if err := s.db.WithContext(ctx).Create(record).Error; err != nil {
        return "", fmt.Errorf("failed to create API key: %w", err)
    }

    // 返回完整的密钥（只能看一次）
    return apiKey, nil
}

// ValidateAPIKey 验证 API Key
func (s *APIKeyService) ValidateAPIKey(ctx context.Context, apiKey string) (string, error) {
    hash := sha256.Sum256([]byte(apiKey))
    keyHash := hex.EncodeToString(hash[:])

    record := &models.APIKey{}
    err := s.db.WithContext(ctx).Where("key_hash = ?", keyHash).First(record).Error

    if err != nil {
        return "", fmt.Errorf("invalid API key")
    }

    // 检查是否过期
    if record.ExpiresAt != nil && time.Now().After(*record.ExpiresAt) {
        return "", fmt.Errorf("API key has expired")
    }

    return record.TenantGUID, nil
}

// ListAPIKeys 列出租户的所有 API Key
func (s *APIKeyService) ListAPIKeys(ctx context.Context, guid string) ([]*models.APIKey, error) {
    var keys []*models.APIKey
    err := s.db.WithContext(ctx).Where("tenant_guid = ?", guid).Find(&keys).Error
    return keys, err
}

// RevokeAPIKey 撤销 API Key
func (s *APIKeyService) RevokeAPIKey(ctx context.Context, guid string, keyID string) error {
    return s.db.WithContext(ctx).Where("id = ? AND tenant_guid = ?", keyID, guid).
        Delete(&models.APIKey{}).Error
}
```

#### API Key 中间件

```go
// internal/middleware/api_key_middleware.go
package middleware

import (
    "net/http"
    "strings"
    "github.com/gin-gonic/gin"
    "your-app/internal/service"
)

type APIKeyMiddleware struct {
    apiKeyService *service.APIKeyService
}

// NewAPIKeyMiddleware 创建 API Key 中间件
func NewAPIKeyMiddleware(apiKeyService *service.APIKeyService) *APIKeyMiddleware {
    return &APIKeyMiddleware{apiKeyService: apiKeyService}
}

// AuthAPIKey 验证 API Key
func (m *APIKeyMiddleware) AuthAPIKey() gin.HandlerFunc {
    return func(c *gin.Context) {
        // 从 Authorization 头获取 API Key
        authHeader := c.GetHeader("Authorization")
        if authHeader == "" {
            c.JSON(http.StatusUnauthorized, gin.H{"error": "missing authorization"})
            c.Abort()
            return
        }

        // 格式: Authorization: Bearer sk_live_xxx
        parts := strings.Split(authHeader, " ")
        if len(parts) != 2 || parts[0] != "Bearer" {
            c.JSON(http.StatusUnauthorized, gin.H{"error": "invalid authorization format"})
            c.Abort()
            return
        }

        apiKey := parts[1]

        // 验证 API Key
        guid, err := m.apiKeyService.ValidateAPIKey(c.Request.Context(), apiKey)
        if err != nil {
            c.JSON(http.StatusUnauthorized, gin.H{"error": err.Error()})
            c.Abort()
            return
        }

        // 设置租户信息
        c.Set("tenant_guid", guid)
        c.Next()
    }
}
```

---

### 2.2 Webhook 系统

```go
// internal/models/webhook.go
package models

import "time"

type Webhook struct {
    ID         string    `gorm:"primaryKey"`
    TenantGUID string    `gorm:"index"`
    URL        string
    Events     string    // JSON array: ["index.created", "search.performed"]
    Secret     string    // 用于签名
    IsActive   bool
    MaxRetries int
    CreatedAt  time.Time
    UpdatedAt  time.Time
}

type WebhookEvent struct {
    ID         string    `gorm:"primaryKey"`
    TenantGUID string    `gorm:"index"`
    WebhookID  string
    Event      string
    Payload    string    // JSON
    Status     string    // pending, success, failed
    Attempts   int
    CreatedAt  time.Time
    UpdatedAt  time.Time
}

type WebhookDelivery struct {
    ID         string    `gorm:"primaryKey"`
    EventID    string
    StatusCode int
    Response   string
    Error      string
    CreatedAt  time.Time
}
```

```go
// internal/service/webhook_service.go
package service

import (
    "bytes"
    "context"
    "crypto/hmac"
    "crypto/sha256"
    "encoding/hex"
    "encoding/json"
    "fmt"
    "io/ioutil"
    "net/http"
    "time"
    "github.com/google/uuid"
    "gorm.io/gorm"
    "your-app/internal/models"
)

type WebhookService struct {
    db     *gorm.DB
    client *http.Client
}

// NewWebhookService 创建 Webhook 服务
func NewWebhookService(db *gorm.DB) *WebhookService {
    return &WebhookService{
        db: db,
        client: &http.Client{
            Timeout: 10 * time.Second,
        },
    }
}

// TriggerWebhook 触发 Webhook 事件
func (s *WebhookService) TriggerWebhook(ctx context.Context, guid string, eventType string, payload interface{}) error {
    // 获取该租户订阅此事件的所有 Webhook
    webhooks, err := s.getWebhooksForEvent(ctx, guid, eventType)
    if err != nil {
        return err
    }

    payloadJSON, err := json.Marshal(payload)
    if err != nil {
        return err
    }

    // 为每个 Webhook 创建事件记录
    for _, webhook := range webhooks {
        event := &models.WebhookEvent{
            ID:         uuid.New().String(),
            TenantGUID: guid,
            WebhookID:  webhook.ID,
            Event:      eventType,
            Payload:    string(payloadJSON),
            Status:     "pending",
            Attempts:   0,
            CreatedAt:  time.Now(),
            UpdatedAt:  time.Now(),
        }

        if err := s.db.WithContext(ctx).Create(event).Error; err != nil {
            return fmt.Errorf("failed to create webhook event: %w", err)
        }

        // 异步发送（使用 goroutine）
        go s.deliverWebhook(context.Background(), webhook, event)
    }

    return nil
}

// deliverWebhook 发送 Webhook 到目标 URL
func (s *WebhookService) deliverWebhook(ctx context.Context, webhook *models.Webhook, event *models.WebhookEvent) {
    var lastErr error

    for attempt := 1; attempt <= webhook.MaxRetries; attempt++ {
        // 生成签名
        signature := s.generateSignature(webhook.Secret, event.Payload)

        // 创建请求
        req, err := http.NewRequestWithContext(ctx, "POST", webhook.URL, bytes.NewBufferString(event.Payload))
        if err != nil {
            lastErr = err
            continue
        }

        req.Header.Set("Content-Type", "application/json")
        req.Header.Set("X-Webhook-Signature", signature)
        req.Header.Set("X-Event-Type", event.Event)
        req.Header.Set("X-Event-ID", event.ID)

        // 发送请求
        resp, err := s.client.Do(req)
        if err != nil {
            lastErr = err
            time.Sleep(time.Duration(attempt*2) * time.Second) // 指数退避
            continue
        }

        body, _ := ioutil.ReadAll(resp.Body)
        resp.Body.Close()

        // 记录交付结果
        delivery := &models.WebhookDelivery{
            ID:         uuid.New().String(),
            EventID:    event.ID,
            StatusCode: resp.StatusCode,
            Response:   string(body),
            CreatedAt:  time.Now(),
        }
        s.db.Create(delivery)

        // 2xx 状态码视为成功
        if resp.StatusCode >= 200 && resp.StatusCode < 300 {
            event.Status = "success"
            event.Attempts = attempt
            event.UpdatedAt = time.Now()
            s.db.Save(event)
            return
        }

        lastErr = fmt.Errorf("webhook returned %d", resp.StatusCode)
        time.Sleep(time.Duration(attempt*2) * time.Second)
    }

    // 全部重试失败
    event.Status = "failed"
    event.Attempts = webhook.MaxRetries
    event.UpdatedAt = time.Now()
    s.db.Save(event)
}

// generateSignature 生成 HMAC-SHA256 签名
func (s *WebhookService) generateSignature(secret string, payload string) string {
    hash := hmac.New(sha256.New, []byte(secret))
    hash.Write([]byte(payload))
    return hex.EncodeToString(hash.Sum(nil))
}

// getWebhooksForEvent 获取订阅特定事件的所有 Webhook
func (s *WebhookService) getWebhooksForEvent(ctx context.Context, guid string, eventType string) ([]*models.Webhook, error) {
    var webhooks []*models.Webhook
    
    err := s.db.WithContext(ctx).
        Where("tenant_guid = ? AND is_active = true", guid).
        Find(&webhooks).Error

    return webhooks, err
}
```

---

## 💰 定价策略

```go
// internal/models/pricing.go
package models

type PricingTier struct {
    Name                 string
    MonthlyPrice         int    // 单位：分
    MaxIndexes           int
    MaxDocumentsPerIndex int64
    MaxQueriesPerMonth   int
    MaxStorageGB         int
    Features             map[string]bool
}

var PricingTiers = map[TenantTier]*PricingTier{
    TierFree: {
        Name:                 "Free",
        MonthlyPrice:         0,
        MaxIndexes:           2,
        MaxDocumentsPerIndex: 100000,
        MaxQueriesPerMonth:   10000,
        MaxStorageGB:         1,
        Features: map[string]bool{
            "export":       false,
            "ai_search":    false,
            "webhooks":     false,
            "api_keys":     false,
            "custom_domain": false,
        },
    },
    TierPro: {
        Name:                 "Pro",
        MonthlyPrice:         9900,  // ¥99
        MaxIndexes:           20,
        MaxDocumentsPerIndex: 5000000,
        MaxQueriesPerMonth:   1000000,
        MaxStorageGB:         100,
        Features: map[string]bool{
            "export":       true,
            "ai_search":    true,
            "webhooks":     false,
            "api_keys":     true,
            "custom_domain": false,
        },
    },
    TierBusiness: {
        Name:                 "Business",
        MonthlyPrice:         29900, // ¥299
        MaxIndexes:           100,
        MaxDocumentsPerIndex: 999999999,
        MaxQueriesPerMonth:   10000000,
        MaxStorageGB:         1000,
        Features: map[string]bool{
            "export":       true,
            "ai_search":    true,
            "webhooks":     true,
            "api_keys":     true,
            "custom_domain": true,
        },
    },
    TierEnterprise: {
        Name:                 "Enterprise",
        MonthlyPrice:         99900, // ¥999+ 自定义
        MaxIndexes:           999999,
        MaxDocumentsPerIndex: 999999999,
        MaxQueriesPerMonth:   999999999,
        MaxStorageGB:         999999,
        Features: map[string]bool{
            "export":       true,
            "ai_search":    true,
            "webhooks":     true,
            "api_keys":     true,
            "custom_domain": true,
        },
    },
}
```

---

## 🔧 主要路由注册

```go
// internal/router/router.go
package router

import (
    "github.com/gin-gonic/gin"
    "your-app/internal/handler"
    "your-app/internal/middleware"
    "your-app/internal/service"
)

func SetupRoutes(r *gin.Engine, services *service.ServiceContainer) {
    tenantMiddleware := middleware.NewTenantMiddleware(services.TenantService)
    apiKeyMiddleware := middleware.NewAPIKeyMiddleware(services.APIKeyService)

    // ===== 公开路由 =====
    public := r.Group("/api/v1/public")
    {
        // 用户注册、登录
        authHandler := &handler.AuthHandler{TenantService: services.TenantService}
        public.POST("/register", authHandler.Register)
        public.POST("/login", authHandler.Login)
    }

    // ===== 受保护的路由（需要 X-Tenant-GUID） =====
    protected := r.Group("/api/v1/tenant")
    protected.Use(tenantMiddleware.AuthTenant())
    {
        // 搜索
        searchHandler := &handler.SearchHandler{
            QuotaService:  services.QuotaService,
            TenantService: services.TenantService,
        }
        protected.POST("/search", searchHandler.Search)

        // 计费
        billingHandler := &handler.BillingHandler{
            TenantService: services.TenantService,
            QuotaService:  services.QuotaService,
        }
        protected.GET("/billing", billingHandler.GetBillingInfo)
        protected.POST("/upgrade", billingHandler.UpgradePlan)

        // API Keys
        apiKeyHandler := &handler.APIKeyHandler{
            APIKeyService: services.APIKeyService,
        }
        protected.POST("/api-keys", apiKeyHandler.CreateAPIKey)
        protected.GET("/api-keys", apiKeyHandler.ListAPIKeys)
        protected.DELETE("/api-keys/:id", apiKeyHandler.RevokeAPIKey)

        // Webhooks
        webhookHandler := &handler.WebhookHandler{
            WebhookService: services.WebhookService,
        }
        protected.POST("/webhooks", webhookHandler.CreateWebhook)
        protected.GET("/webhooks", webhookHandler.ListWebhooks)
    }

    // ===== API Key 认证路由 =====
    apiAuth := r.Group("/api/v1")
    apiAuth.Use(apiKeyMiddleware.AuthAPIKey())
    {
        searchHandler := &handler.SearchHandler{
            QuotaService:  services.QuotaService,
            TenantService: services.TenantService,
        }
        apiAuth.POST("/search", searchHandler.Search)
    }
}
```

---

## 📊 数据库迁移脚本

```sql
-- 租户表
CREATE TABLE tenants (
    guid VARCHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    tier ENUM('free', 'pro', 'business', 'enterprise') DEFAULT 'free',
    status ENUM('active', 'suspended', 'cancelled') DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_email (email),
    INDEX idx_tier (tier),
    INDEX idx_status (status)
);

-- 租户配额
CREATE TABLE tenant_quotas (
    tenant_guid VARCHAR(36) PRIMARY KEY,
    max_queries_per_month INT DEFAULT 10000,
    max_indexes INT DEFAULT 2,
    max_documents_per_index BIGINT DEFAULT 100000,
    max_storage_gb INT DEFAULT 1,
    api_rate_limit INT DEFAULT 100,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_guid) REFERENCES tenants(guid) ON DELETE CASCADE
);

-- 租户功能
CREATE TABLE tenant_features (
    tenant_guid VARCHAR(36) PRIMARY KEY,
    customization BOOLEAN DEFAULT true,
    export BOOLEAN DEFAULT false,
    ai_search BOOLEAN DEFAULT false,
    webhooks BOOLEAN DEFAULT false,
    api_keys BOOLEAN DEFAULT false,
    custom_domain BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_guid) REFERENCES tenants(guid) ON DELETE CASCADE
);

-- 使用量统计
CREATE TABLE usage_metrics (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    tenant_guid VARCHAR(36) NOT NULL,
    metric_date DATE NOT NULL,
    queries_count INT DEFAULT 0,
    documents_imported INT DEFAULT 0,
    storage_bytes BIGINT DEFAULT 0,
    exports_count INT DEFAULT 0,
    api_calls_count INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY unique_tenant_date (tenant_guid, metric_date),
    FOREIGN KEY (tenant_guid) REFERENCES tenants(guid) ON DELETE CASCADE,
    INDEX idx_date (metric_date)
);

-- API Keys
CREATE TABLE api_keys (
    id VARCHAR(36) PRIMARY KEY,
    tenant_guid VARCHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(64) NOT NULL UNIQUE,
    permissions JSON,
    expires_at TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_guid) REFERENCES tenants(guid) ON DELETE CASCADE,
    INDEX idx_key_hash (key_hash),
    INDEX idx_tenant (tenant_guid)
);

-- Webhooks
CREATE TABLE webhooks (
    id VARCHAR(36) PRIMARY KEY,
    tenant_guid VARCHAR(36) NOT NULL,
    url VARCHAR(2048) NOT NULL,
    events JSON NOT NULL,
    secret VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    max_retries INT DEFAULT 3,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_guid) REFERENCES tenants(guid) ON DELETE CASCADE,
    INDEX idx_tenant (tenant_guid),
    INDEX idx_active (is_active)
);

-- Webhook 事件
CREATE TABLE webhook_events (
    id VARCHAR(36) PRIMARY KEY,
    tenant_guid VARCHAR(36) NOT NULL,
    webhook_id VARCHAR(36) NOT NULL,
    event VARCHAR(100) NOT NULL,
    payload LONGTEXT NOT NULL,
    status ENUM('pending', 'success', 'failed') DEFAULT 'pending',
    attempts INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_guid) REFERENCES tenants(guid) ON DELETE CASCADE,
    FOREIGN KEY (webhook_id) REFERENCES webhooks(id) ON DELETE CASCADE,
    INDEX idx_status (status),
    INDEX idx_created (created_at)
);

-- 索引访问权限
CREATE TABLE index_access (
    index_uid VARCHAR(255) NOT NULL,
    tenant_guid VARCHAR(36) NOT NULL,
    permission ENUM('read', 'write', 'admin') DEFAULT 'read',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (index_uid, tenant_guid),
    FOREIGN KEY (tenant_guid) REFERENCES tenants(guid) ON DELETE CASCADE,
    INDEX idx_tenant (tenant_guid)
);
```

---

## 🚀 实施时间表

### Week 1-2: 基础设施
- [ ] 创建租户模型和数据库表
- [ ] 实现 TenantService
- [ ] 创建租户中间件

### Week 3-4: 配额管理
- [ ] 实现 QuotaService
- [ ] 添加配额检查中间件
- [ ] 创建计费 API 端点

### Week 5-6: API Keys
- [ ] 实现 APIKeyService
- [ ] 创建 API Key 管理界面
- [ ] 实现 API Key 认证

### Week 7-8: Webhook 系统
- [ ] 实现 WebhookService
- [ ] 创建 Webhook 管理 UI
- [ ] 添加事件触发逻辑

### Week 9-12: 支付 + 上线
- [ ] 集成支付网关（Stripe/支付宝）
- [ ] 测试完整流程
- [ ] 部署到生产环境
- [ ] 发布到 Product Hunt

---

## 💬 前端修改提示

更新 `src/composables/useApp.ts`：

```typescript
// 添加租户信息
const currentTenantGuid = ref(localStorage.getItem('tenantGuid') || '')
const tenantTier = ref<'free' | 'pro' | 'business' | 'enterprise'>('free')
const tenantFeatures = ref({
  export: false,
  aiSearch: false,
  webhooks: false,
  apiKeys: false,
})

// 在每个请求中添加 X-Tenant-GUID 头
const createClient = (host: string, apiKey: string): AxiosInstance => {
  const adminToken = localStorage.getItem('authToken')
  const instance = axios.create({
    baseURL: host.trim().replace(/\/$/, ''),
    headers: {
      'Content-Type': 'application/json',
      'X-Tenant-GUID': currentTenantGuid.value,  // ⭐ 新增
      ...(apiKey.trim() ? { 'App-Token': apiKey.trim() } : {}),
      ...(adminToken ? { 'Authorization': `Bearer ${adminToken}` } : {}),
    },
  })
  return instance
}

// 功能开关
const canExport = computed(() => tenantFeatures.value.export)
const canUseAI = computed(() => tenantFeatures.value.aiSearch)
```

---

## ✅ 上线检查清单

```
安全性
├─ [ ] GUID 隔离验证
├─ [ ] API Key 密钥管理
├─ [ ] HTTPS 强制
├─ [ ] CORS 策略配置
├─ [ ] 速率限制实现
└─ [ ] SQL 注入防护

可靠性
├─ [ ] 数据库备份配置
├─ [ ] 监控告警设置
├─ [ ] 错误日志记录
├─ [ ] 性能监控
└─ [ ] 灾难恢复计划

文档
├─ [ ] API 文档（Swagger）
├─ [ ] SDK 文档
├─ [ ] 快速入门指南
├─ [ ] 定价页面
└─ [ ] 常见问题

法律
├─ [ ] 用户协议
├─ [ ] 隐私政策
├─ [ ] 服务条款
└─ [ ] GDPR 合规
```

---

**需要我继续补充前端界面代码或支付网关集成的详细实现吗？**
