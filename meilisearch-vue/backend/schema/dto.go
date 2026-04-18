package schema

// LoginRequest 表示登录请求
type LoginRequest struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

// UserCreateRequest 用户创建
type UserCreateRequest struct {
	Username     string `json:"username" binding:"required"`
	Password     string `json:"password" binding:"required"`
	Role         string `json:"role"`
	AllowIndexes string `json:"allowIndexes"`
}

// UserPermRequest 权限更新
type UserPermRequest struct {
	Role         string `json:"role"`
	AllowIndexes string `json:"allowIndexes"`
}

// AppUpdateRequest UI修改
type AppUpdateRequest struct {
	UIConfig string `json:"uiConfig"`
}
