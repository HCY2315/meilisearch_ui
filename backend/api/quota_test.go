package api

import (
	"backend/model"
	"backend/repository"
	"fmt"
	"os"
	"testing"

	"github.com/glebarez/sqlite"
	"gorm.io/gorm"
)

func TestMain(m *testing.M) {
	// 在内存中初始化临时 SQLite 数据库用于测试
	var err error
	repository.DB, err = gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	if err != nil {
		panic("failed to connect database: " + err.Error())
	}

	err = repository.DB.AutoMigrate(
		&model.AccessToken{},
		&model.Application{},
		&model.TokenUsageMetric{},
		&model.AppUsageMetric{},
	)
	if err != nil {
		panic("failed to migrate database: " + err.Error())
	}

	os.Exit(m.Run())
}

func TestCheckAndIncrementTokenQuota(t *testing.T) {
	token := "test-token-123"
	// 1. 无限额的 Token
	tokFree := model.AccessToken{
		Token:            token,
		MaxQueriesPerDay: 0,
		MaxImportsPerDay: 0,
	}
	repository.DB.Create(&tokFree)

	// 应始终允许
	for i := 0; i < 10; i++ {
		allowed, _ := checkAndIncrementTokenQuota(token, true, 1)
		if !allowed {
			t.Errorf("expected allowed for unlimited token, got forbidden")
		}
	}

	// 2. 限额为 5 的 Token
	tokenLimited := "test-token-limited"
	tokLimited := model.AccessToken{
		Token:            tokenLimited,
		MaxQueriesPerDay: 5,
		MaxImportsPerDay: 2,
	}
	repository.DB.Create(&tokLimited)

	// 连续 5 次查询应成功
	for i := 0; i < 5; i++ {
		allowed, errStr := checkAndIncrementTokenQuota(tokenLimited, true, 1)
		if !allowed {
			t.Errorf("expected query %d to be allowed, got forbidden: %s", i+1, errStr)
		}
	}

	// 第 6 次应该被拦截
	allowed, errStr := checkAndIncrementTokenQuota(tokenLimited, true, 1)
	if allowed {
		t.Errorf("expected 6th query to be blocked, but it passed")
	} else {
		fmt.Println("Blocked successfully as expected:", errStr)
	}

	// 导入限额测试
	// 第一次导入 1 个文档
	allowed, _ = checkAndIncrementTokenQuota(tokenLimited, false, 1)
	if !allowed {
		t.Errorf("expected first import of 1 doc to pass")
	}
	// 第二次导入 2 个文档，由于 1+2 = 3 > 2，应该被拦截
	allowed, errStr = checkAndIncrementTokenQuota(tokenLimited, false, 2)
	if allowed {
		t.Errorf("expected import of 2 docs to be blocked (total 3 > 2)")
	} else {
		fmt.Println("Blocked successfully as expected:", errStr)
	}
}

func TestCheckAndIncrementAppQuota(t *testing.T) {
	appKey := "test-app-key-123"
	// 限额为 3 的 App
	app := model.Application{
		AppKey:           appKey,
		Name:             "Test App",
		MaxQueriesPerDay: 3,
		MaxImportsPerDay: 1,
	}
	repository.DB.Create(&app)

	// 连续 3 次查询应成功
	for i := 0; i < 3; i++ {
		allowed, _ := checkAndIncrementAppQuota(appKey, true, 1)
		if !allowed {
			t.Errorf("expected query %d to be allowed", i+1)
		}
	}

	// 第 4 次应该被拦截
	allowed, errStr := checkAndIncrementAppQuota(appKey, true, 1)
	if allowed {
		t.Errorf("expected 4th query to be blocked")
	} else {
		fmt.Println("Blocked app successfully as expected:", errStr)
	}
}
