package repository

import (
	"log"
	"os"

	"backend/model"

	"github.com/glebarez/sqlite" // 替换为纯 Go 实现的 sqlite 驱动，解决 cgo 报错
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

var DB *gorm.DB

// InitDB 初始化本地 SQLite 数据库并进行字段迁移
func InitDB() {
	var err error
	DB, err = gorm.Open(sqlite.Open("meili_admin.db"), &gorm.Config{})
	if err != nil {
		log.Fatalf("failed to connect database: %v", err)
	}

	// 自动迁移模式
	err = DB.AutoMigrate(
		&model.User{},
		&model.MeiliInstance{},
		&model.Application{},
		&model.IndexConfig{},
		&model.AccessToken{},
	)
	if err != nil {
		log.Fatalf("failed to migrate database: %v", err)
	}

	seedData()
}

// seedData 生成默认管理员账号
func seedData() {
	var count int64
	DB.Model(&model.User{}).Count(&count)
	if count == 0 {
		hash, _ := bcrypt.GenerateFromPassword([]byte("admin123"), bcrypt.DefaultCost)
		admin := model.User{
			Username:     "admin",
			PasswordHash: string(hash),
			Role:         "admin",
		}
		DB.Create(&admin)

		meiliName := os.Getenv("MEILI_NAME")
		if meiliName == "" {
			meiliName = "Default Local"
		}
		meiliHost := os.Getenv("MEILI_HOST")
		if meiliHost == "" {
			meiliHost = "http://localhost:7700"
		}
		meiliKey := os.Getenv("MEILI_API_KEY")
		if meiliKey == "" {
			meiliKey = "123456"
		}

		testInstance := model.MeiliInstance{
			Name:   meiliName,
			Host:   meiliHost,
			APIKey: meiliKey,
		}
		DB.Create(&testInstance)

		testApp := model.Application{
			Name: "默认前台",
			AppKey: "default-app-key",
			InstanceID: testInstance.ID,
			UIConfig: `{"theme":"dark", "title":"🔍 多维查询系统"}`,
			AllowIndexes: `[]`,
		}
		DB.Create(&testApp)

		log.Println("Seeded default admin user (admin / admin123) and sample app config")
	}
}
