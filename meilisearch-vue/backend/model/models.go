package model

import (
	"time"
)

// User 系统用户表
type User struct {
	ID           uint      `gorm:"primarykey" json:"id"`
	Username     string    `gorm:"uniqueIndex;size:128;not null" json:"username"`
	PasswordHash string    `gorm:"not null" json:"-"` // 不返回密码哈希
	Role         string    `gorm:"size:32;default:'user'" json:"role"` // admin / user
	AllowIndexes string    `gorm:"type:text" json:"allowIndexes"`      // JSON 数组，记录该用户允许访问的 index
	CreatedAt    time.Time `json:"createdAt"`
	UpdatedAt    time.Time `json:"updatedAt"`
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
