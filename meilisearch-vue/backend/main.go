package main

import (
	"log"

	"backend/api"
	"backend/repository"
)

func main() {
	// 初始化数据库 (将加载 gorm 与 glebarez/sqlite 并建立表与初始种子数据)
	repository.InitDB()

	// 初始化 Gin 路由
	r := api.InitRouter()

	log.Println("Server mapping to port :7701")
	if err := r.Run(":7701"); err != nil {
		log.Fatalf("Server forced to shutdown: %v", err)
	}
}
