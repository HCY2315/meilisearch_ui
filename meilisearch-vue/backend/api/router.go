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

		// 前台拉取允许展示的 Index 列表
		v1.GET("/public/indexes", HandleGetVisibleIndexes)

		adminGroup := v1.Group("/admin")
		adminGroup.Use(AuthMiddleware())
		{
			adminGroup.GET("/instances", HandleGetInstances)
			adminGroup.POST("/instances", HandleCreateInstance)
			adminGroup.PUT("/instances", HandleUpdateInstance)
			adminGroup.DELETE("/instances/:id", HandleDeleteInstance)
			adminGroup.GET("/apps", HandleGetApps)
			adminGroup.PUT("/apps/:id", HandleUpdateApp)

			adminGroup.GET("/index_configs", HandleGetIndexConfigs)
			adminGroup.POST("/index_configs", HandleSaveIndexConfig)
			adminGroup.DELETE("/index_configs/:uid", HandleDeleteIndex)

			adminGroup.GET("/access_tokens", HandleGetAccessTokens)
			adminGroup.POST("/access_tokens", HandleCreateAccessToken)
			adminGroup.PUT("/access_tokens", HandleUpdateAccessToken)
			adminGroup.DELETE("/access_tokens/:id", HandleDeleteAccessToken)
		}

		proxyGroup := v1.Group("/proxy")
		proxyGroup.Any("/*proxyPath", HandleProxy)
	}

	r.Static("/assets", "../dist/assets")
	r.StaticFile("/favicon.ico", "../dist/favicon.ico")

	r.NoRoute(func(c *gin.Context) {
		if c.Request.URL.Path == "/" || c.Request.URL.Path == "/admin" {
			c.File("../dist/index.html")
			return
		}
		c.AbortWithStatusJSON(404, gin.H{"error": "route not found"})
	})

	return r
}
