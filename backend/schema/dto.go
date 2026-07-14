package schema

import "time"

// LoginRequest 表示超级后台登录请求
type LoginRequest struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}



type IndexConfigRequest struct {
	Uid                string `json:"uid" binding:"required"`
	Alias              string `json:"alias"`
	Description        string `json:"description"`
	IsVisible          bool   `json:"isVisible"`
	IsLocked           bool   `json:"isLocked"`
	FieldConfigs       string `json:"fieldConfigs"`
	ViewConfigs        string `json:"viewConfigs"`
	TableConfigs       string `json:"tableConfigs"`
	CanEdit            bool   `json:"canEdit"`
	NestedFieldConfigs string `json:"nestedFieldConfigs"`
	DrawerFieldOrder   string `json:"drawerFieldOrder"`
}

// NestedFieldConfigsUpdateRequest 单独更新嵌套字段配置（不需传其他字段）
type NestedFieldConfigsUpdateRequest struct {
	Uid                string `json:"uid" binding:"required"`
	NestedFieldConfigs string `json:"nestedFieldConfigs" binding:"required"` // 全量 JSON
}

// TableConfigsUpdateRequest 单独更新列表列配置（顺序/隐藏）
type TableConfigsUpdateRequest struct {
	Uid          string `json:"uid" binding:"required"`
	TableConfigs string `json:"tableConfigs" binding:"required"`
}

// AccessTokenRequest 新建数据访问Token
type AccessTokenRequest struct {
	Token            string     `json:"token" binding:"required"`
	AllowIndexes     string     `json:"allowIndexes" binding:"required"` // '["docs", "finance"]'
	Description      string     `json:"description"`
	ExpiresAt        *time.Time `json:"expiresAt"`
	MaxQueriesPerDay int64      `json:"maxQueriesPerDay"` // 每日最大查询次数，0表示不限制
	MaxImportsPerDay int64      `json:"maxImportsPerDay"` // 每日最大导入次数，0表示不限制
}

type AccessTokenUpdateRequest struct {
	ID               uint       `json:"id" binding:"required"`
	Token            string     `json:"token"`
	AllowIndexes     string     `json:"allowIndexes"`
	Description      string     `json:"description"`
	ExpiresAt        *time.Time `json:"expiresAt"`
	MaxQueriesPerDay *int64     `json:"maxQueriesPerDay"` // 指针类型以便于零值更新区分
	MaxImportsPerDay *int64     `json:"maxImportsPerDay"`
}

// MeiliInstanceRequest 新建实例
type MeiliInstanceRequest struct {
	Name   string `json:"name" binding:"required"`
	Host   string `json:"host" binding:"required"`
	APIKey string `json:"apiKey"`
}

type MeiliInstanceUpdateRequest struct {
	ID     uint   `json:"id" binding:"required"`
	Name   string `json:"name"`
	Host   string `json:"host"`
	APIKey string `json:"apiKey"`
}

type PasswordUpdateRequest struct {
	NewPassword string `json:"newPassword" binding:"required"`
}

// IndexSettingsRequest 更新索引的可搜索和可过滤字段配置
type IndexSettingsRequest struct {
	Uid                  string    `json:"uid" binding:"required"`
	SearchableAttributes *[]string `json:"searchableAttributes"`
	FilterableAttributes *[]string `json:"filterableAttributes"`
	Embedders            any       `json:"embedders"`
}

type ValidateEmbedderRequest struct {
	Uid      string `json:"uid" binding:"required"`
	Embedder any    `json:"embedder" binding:"required"`
}

// SendCodeRequest 发送验证码请求
type SendCodeRequest struct {
	Email string `json:"email" binding:"required,email"`
}

// TokenApplicationSubmitRequest 提交申请请求
type TokenApplicationSubmitRequest struct {
	Email        string   `json:"email" binding:"required,email"`
	Code         string   `json:"code" binding:"required"`
	Name         string   `json:"name" binding:"required"`
	Birthday     string   `json:"birthday"`
	Gender       string   `json:"gender"`
	Purpose      string   `json:"purpose"`
	AllowIndexes []string `json:"allowIndexes"`
}

type ApproveApplicationRequest struct {
	Token            string   `json:"token"`
	AllowIndexes     []string `json:"allowIndexes"`
	Description      string   `json:"description"`
	ValidDays        *int     `json:"validDays"`
	MaxQueriesPerDay int64    `json:"maxQueriesPerDay"` // 每日最大查询次数
	MaxImportsPerDay int64    `json:"maxImportsPerDay"` // 每日最大导入次数
}

type RejectApplicationRequest struct {
	RejectMessage string `json:"rejectMessage"`
}

// AppCreateRequest 创建新应用请求
type AppCreateRequest struct {
	Name         string `json:"name" binding:"required"`
	AppKey       string `json:"appKey" binding:"required"` // 前台调用凭证
	InstanceID   uint   `json:"instanceId" binding:"required"`
	TenantID     uint   `json:"tenantId"`                  // 绑定租户，0 表示无租户
	UIConfig     string `json:"uiConfig"`
	AllowIndexes string `json:"allowIndexes"` // JSON 数组
}

// AppUpdateRequest 更新应用请求
type AppUpdateRequest struct {
	UIConfig         string  `json:"uiConfig"`
	Name             string  `json:"name"`
	AllowIndexes     string  `json:"allowIndexes"`          // JSON 数组
	InstanceID       uint    `json:"instanceId"`
	TenantID         *uint   `json:"tenantId"`              // 指针类型，允许设置为 0
	MaxQueriesPerDay *int64  `json:"maxQueriesPerDay"`
	MaxImportsPerDay *int64  `json:"maxImportsPerDay"`
}

// TenantRequest 创建租户请求
type TenantRequest struct {
	Name         string `json:"name" binding:"required"`
	Slug         string `json:"slug" binding:"required"`   // URL 安全标识符
	Plan         string `json:"plan"`                      // free / pro / enterprise
	MaxApps      int    `json:"maxApps"`
	MaxIndexes   int    `json:"maxIndexes"`
	ContactEmail string `json:"contactEmail"`
}

// TenantUpdateRequest 更新租户请求
type TenantUpdateRequest struct {
	Name         string `json:"name"`
	Plan         string `json:"plan"`
	Status       *int   `json:"status"`    // 指针类型，允许设置为 0（停用）
	MaxApps      *int   `json:"maxApps"`
	MaxIndexes   *int   `json:"maxIndexes"`
	ContactEmail string `json:"contactEmail"`
}
