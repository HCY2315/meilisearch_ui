package model

import "time"

// SyncSource MySQL 数据源连接配置
type SyncSource struct {
	ID        uint      `gorm:"primarykey" json:"id"`
	Name      string    `gorm:"size:128" json:"name"`
	Host      string    `gorm:"size:128" json:"host"`
	Port      int       `gorm:"default:3306" json:"port"`
	User      string    `gorm:"size:128" json:"user"`
	Password  string    `gorm:"size:256" json:"-"` // 不对外暴露
	CreatedAt time.Time `json:"createdAt"`
	UpdatedAt time.Time `json:"updatedAt"`
}

// SyncDeadLetter 重试多次仍失败的批次（保存原始 payload 以便人工重放，保证不静默丢数据）
type SyncDeadLetter struct {
	ID        uint      `gorm:"primarykey" json:"id"`
	TaskID    uint      `gorm:"index" json:"taskId"`
	Payload   string    `gorm:"type:text" json:"payload"` // JSON: {"upserts":[...],"deletes":[...]}
	Error     string    `gorm:"size:512" json:"error"`
	CreatedAt time.Time `json:"createdAt"`
}

// SyncTask 一条「MySQL 表 -> Meilisearch 索引」的实时同步任务
// NOTE: 监听 TableName 的 binlog，INSERT/UPDATE 时按 PrimaryKey 回查 JoinSQL 取整条文档 upsert；
//
//	DELETE 时按 PrimaryKey 在 Meilisearch 中删除对应文档。
type SyncTask struct {
	ID              uint   `gorm:"primarykey" json:"id"`
	Name            string `gorm:"size:128" json:"name"`
	SourceID        uint   `json:"sourceId"`
	DBName          string `gorm:"size:128" json:"dbName"`                 // 数据库名（binlog 过滤 + JOIN SQL 默认 schema）
	TableName       string `gorm:"size:128" json:"tableName"`              // 主表（binlog 监听对象）
	PrimaryKey      string `gorm:"size:64;default:'id'" json:"primaryKey"` // 主表主键列名
	JoinSQL         string `gorm:"type:text" json:"joinSql"`               // 含 :ids 占位符的查询语句，返回文档
	InstanceID      uint   `json:"instanceId"`                             // 目标 Meilisearch 实例
	IndexUID        string `gorm:"size:128" json:"indexUid"`               // 目标索引
	BatchSize       int    `gorm:"default:500" json:"batchSize"`           // 批量条数
	FlushIntervalMs int    `gorm:"default:300" json:"flushIntervalMs"`     // 攒批时间(ms)
	Enabled         bool   `json:"enabled"`                                // 是否随服务启动自动运行
	SoftDeleteField string `gorm:"size:64" json:"softDeleteField"`         // 软删除判定字段（联表结果中的列名）
	SoftDeleteValue string `gorm:"size:128" json:"softDeleteValue"`        // 当该字段等于此值时，从索引删除文档

	// 运行时统计（不入库）
	DeadLetterCount int64 `gorm:"-" json:"deadLetterCount"`

	// 运行状态
	Running    bool       `gorm:"default:false" json:"running"`
	LastError  string     `gorm:"size:512" json:"lastError"`
	LastSyncAt *time.Time `json:"lastSyncAt"`
	Position   string     `gorm:"type:text" json:"position"` // GTID set，用于断点续传

	CreatedAt time.Time `json:"createdAt"`
	UpdatedAt time.Time `json:"updatedAt"`
}
