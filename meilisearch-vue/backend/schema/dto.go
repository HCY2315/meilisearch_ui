package schema

import "time"

// LoginRequest 表示超级后台登录请求
type LoginRequest struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

// AppUpdateRequest UI修改
type AppUpdateRequest struct {
	UIConfig string `json:"uiConfig"`
}

type IndexConfigRequest struct {
	Uid                string `json:"uid" binding:"required"`
	Alias              string `json:"alias"`
	Description        string `json:"description"`
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
	Token        string     `json:"token" binding:"required"`
	AllowIndexes string     `json:"allowIndexes" binding:"required"` // '["docs", "finance"]'
	Description  string     `json:"description"`
	ExpiresAt    *time.Time `json:"expiresAt"`
}

type AccessTokenUpdateRequest struct {
	ID           uint       `json:"id" binding:"required"`
	Token        string     `json:"token"`
	AllowIndexes string     `json:"allowIndexes"`
	Description  string     `json:"description"`
	ExpiresAt    *time.Time `json:"expiresAt"`
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
