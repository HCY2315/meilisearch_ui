package main

import (
	"log"
	"time"

	"backend/api"
	"backend/model"
	"backend/repository"
)

func main() {
	// 初始化数据库 (将加载 gorm 与 glebarez/sqlite 并建立表与初始种子数据)
	repository.InitDB()

	// 启动定期清理过期 Token 的任务 (每小时清理一次)
	go func() {
		ticker := time.NewTicker(time.Hour)
		for range ticker.C {
			log.Println("[Cleanup] Cleaning up expired access tokens...")
			repository.DB.Where("expires_at IS NOT NULL AND expires_at < ?", time.Now()).Delete(&model.AccessToken{})
		}
	}()

	// 初始化 Gin 路由
	r := api.InitRouter()

	log.Println("Server mapping to port :80")
	if err := r.Run(":8080"); err != nil {
		log.Fatalf("Server forced to shutdown: %v", err)
	}
}
