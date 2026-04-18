package api

import (
	"github.com/gin-gonic/gin"
)

// InitRouter setup the Gin router and returns
func InitRouter() *gin.Engine {
	r := gin.Default()
	r.Use(CORSMiddleware())

	v1 := r.Group("/api/v1")
	{
		v1.POST("/auth/login", HandleLogin)
		v1.GET("/app/config", HandleAppConfig)

		adminGroup := v1.Group("/admin")
		adminGroup.Use(AuthMiddleware())
		{
			adminGroup.GET("/instances", HandleGetInstances)
			adminGroup.GET("/apps", HandleGetApps)
			adminGroup.PUT("/apps/:id", HandleUpdateApp)

			adminGroup.GET("/users", HandleGetUsers)
			adminGroup.POST("/users", HandleCreateUser)
			adminGroup.PUT("/users/:id/permissions", HandleUpdateUserPermissions)
		}

		proxyGroup := v1.Group("/proxy")
		proxyGroup.Use(AuthMiddleware())
		proxyGroup.Any("/*proxyPath", HandleProxy)
	}

	return r
}
