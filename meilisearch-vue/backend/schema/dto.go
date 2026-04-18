package schema

// LoginRequest 表示超级后台登录请求
type LoginRequest struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

// AppUpdateRequest UI修改
type AppUpdateRequest struct {
	UIConfig string `json:"uiConfig"`
}

// IndexConfigRequest 索引加密配置
type IndexConfigRequest struct {
	Uid         string `json:"uid" binding:"required"`
	Alias       string `json:"alias"`
	Description string `json:"description"`
	IsLocked    bool   `json:"isLocked"`
}

// AccessTokenRequest 新建数据访问Token
type AccessTokenRequest struct {
	Token        string `json:"token" binding:"required"`
	AllowIndexes string `json:"allowIndexes" binding:"required"` // '["docs", "finance"]'
	Description  string `json:"description"`
}
