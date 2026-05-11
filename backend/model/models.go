package model

import (
	"time"
)

// User 系统管理员表
type User struct {
	ID           uint      `gorm:"primarykey" json:"id"`
	Username     string    `gorm:"uniqueIndex;size:128;not null" json:"username"`
	PasswordHash string    `gorm:"not null" json:"-"`
	Role         string    `gorm:"size:32;default:'admin'" json:"role"` // 仅用于区分超管
	CreatedAt    time.Time `json:"createdAt"`
	UpdatedAt    time.Time `json:"updatedAt"`
}

type IndexConfig struct {
	ID                 uint   `gorm:"primarykey" json:"id"`
	Uid                string `gorm:"uniqueIndex;not null" json:"uid"`
	Alias              string `gorm:"size:255" json:"alias"`
	Description        string `gorm:"size:512" json:"description"`
	IsLocked           bool   `gorm:"default:false" json:"isLocked"`
	FieldConfigs       string `gorm:"type:text" json:"fieldConfigs"`       // JSON: Field settings
	ViewConfigs        string `gorm:"type:text" json:"viewConfigs"`        // JSON: Custom views
	TableConfigs       string `gorm:"type:text" json:"tableConfigs"`       // JSON: Table order/hidden
	CanEdit            bool   `gorm:"default:false" json:"canEdit"`        // Whether front-end can edit docs
	NestedFieldConfigs string `gorm:"type:text" json:"nestedFieldConfigs"` // JSON: Nested viewer field configs
	// 抽屉字段显示顺序，JSON 数组：["field1", "field2", ...]
	DrawerFieldOrder string `gorm:"type:text" json:"drawerFieldOrder"`
}

// AccessToken 前台解锁用的专属凭证
type AccessToken struct {
	ID           uint      `gorm:"primarykey" json:"id"`
	Token        string    `gorm:"uniqueIndex;not null" json:"token"`
	AllowIndexes string    `gorm:"type:text" json:"allowIndexes"` // JSON数组，例如 ["docs", "finance"]
	Description  string    `gorm:"size:255" json:"description"`
	ExpiresAt    *time.Time `json:"expiresAt"` // 过期时间，nil 表示永不过期
	CreatedAt    time.Time `json:"createdAt"`
}

// MeiliInstance Meilisearch 搜索引擎节点实例表
type MeiliInstance struct {
	ID        uint      `gorm:"primarykey" json:"id"`
	Name      string    `gorm:"size:128;not null" json:"name"`
	Host      string    `gorm:"size:256;not null" json:"host"`
	APIKey    string    `gorm:"size:256;not null" json:"-"` // 管理员 Key，不对外暴露
	Status    int       `gorm:"default:1" json:"status"` // 1: 正常, 0: 停用
	CreatedAt time.Time `json:"createdAt"`
	UpdatedAt time.Time `json:"updatedAt"`
}

// Application 前台应用配置表
type Application struct {
	ID           uint      `gorm:"primarykey" json:"id"`
	Name         string    `gorm:"uniqueIndex;size:128;not null" json:"name"`
	AppKey       string    `gorm:"uniqueIndex;size:64;not null" json:"appKey"` // 前台调用凭证
	InstanceID   uint      `gorm:"not null" json:"instanceId"`
	UIConfig     string    `gorm:"type:text" json:"uiConfig"`      // 存储前台 UI 相关的 JSON 配置
	AllowIndexes string    `gorm:"type:text" json:"allowIndexes"`  // JSON 数组，记录允许访问的 index 列表
	CreatedAt    time.Time `json:"createdAt"`
	UpdatedAt    time.Time `json:"updatedAt"`
}
