package api

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"backend/model"
	"backend/repository"

	"github.com/gin-gonic/gin"
)

func TestSmtpConfigPersistence(t *testing.T) {
	gin.SetMode(gin.TestMode)
	// 隔离环境变量回退
	t.Setenv("SMTP_USER", "")
	t.Setenv("SMTP_PASS", "")
	repository.DB.Where("1 = 1").Delete(&model.SmtpConfig{})
	t.Cleanup(func() { repository.DB.Where("1 = 1").Delete(&model.SmtpConfig{}) })

	if _, ok := getSmtpConfig(); ok {
		t.Fatal("初始应未配置邮件服务")
	}

	// 保存配置
	w := httptest.NewRecorder()
	c, _ := gin.CreateTestContext(w)
	c.Request = httptest.NewRequest(http.MethodPut, "/", strings.NewReader(
		`{"host":"smtp.test.com","port":587,"username":"u@test.com","password":"secret","encryption":"starttls","fromName":"T"}`))
	c.Request.Header.Set("Content-Type", "application/json")
	HandleUpdateSmtpConfig(c)
	if w.Code != http.StatusOK {
		t.Fatalf("保存失败: %d %s", w.Code, w.Body.String())
	}

	cfg, ok := getSmtpConfig()
	if !ok || cfg.Host != "smtp.test.com" || cfg.Password != "secret" || cfg.Encryption != "starttls" || cfg.Port != 587 {
		t.Fatalf("配置读取错误: %+v ok=%v", cfg, ok)
	}

	// GET 必须脱敏，不返回密码明文
	w2 := httptest.NewRecorder()
	c2, _ := gin.CreateTestContext(w2)
	c2.Request = httptest.NewRequest(http.MethodGet, "/", nil)
	HandleGetSmtpConfig(c2)
	body := w2.Body.String()
	if strings.Contains(body, "secret") {
		t.Errorf("GET 不应返回密码明文: %s", body)
	}
	if !strings.Contains(body, `"hasPassword":true`) {
		t.Errorf("应标记 hasPassword=true: %s", body)
	}
	if !strings.Contains(body, `"source":"db"`) {
		t.Errorf("来源应为 db: %s", body)
	}

	// 密码留空表示不修改
	w3 := httptest.NewRecorder()
	c3, _ := gin.CreateTestContext(w3)
	c3.Request = httptest.NewRequest(http.MethodPut, "/", strings.NewReader(
		`{"host":"smtp.test.com","port":465,"username":"u@test.com","password":"","encryption":"ssl"}`))
	c3.Request.Header.Set("Content-Type", "application/json")
	HandleUpdateSmtpConfig(c3)
	cfg2, _ := getSmtpConfig()
	if cfg2.Password != "secret" {
		t.Errorf("密码留空应保留原值，实际 %q", cfg2.Password)
	}
	if cfg2.Encryption != "ssl" || cfg2.Port != 465 {
		t.Errorf("其它字段应更新: %+v", cfg2)
	}

	// 环境变量回退：清空 DB 后应回退到 env
	repository.DB.Where("1 = 1").Delete(&model.SmtpConfig{})
	t.Setenv("SMTP_USER", "env@test.com")
	t.Setenv("SMTP_PASS", "envpass")
	cfg3, ok := getSmtpConfig()
	if !ok || cfg3.Username != "env@test.com" || cfg3.Host != "smtp.163.com" {
		t.Fatalf("环境变量回退失败: %+v ok=%v", cfg3, ok)
	}
}

func TestSystemSettingPersistence(t *testing.T) {
	gin.SetMode(gin.TestMode)
	repository.DB.Where("1 = 1").Delete(&model.SystemSetting{})
	t.Cleanup(func() { repository.DB.Where("1 = 1").Delete(&model.SystemSetting{}) })

	// 默认值
	d := getSystemSetting()
	if d.TokenValidDays != 30 || d.DefaultPageSize != 10 || !d.AllowSelfApplication {
		t.Fatalf("默认值错误: %+v", d)
	}

	// 保存
	w := httptest.NewRecorder()
	c, _ := gin.CreateTestContext(w)
	c.Request = httptest.NewRequest(http.MethodPut, "/", strings.NewReader(
		`{"systemTitle":"我的系统","defaultSearchTitle":"标题A","frontendSubtitle":"副标题","copyright":"© X","tokenValidDays":7,"applicationIntervalDays":15,"verificationTtlMinutes":60,"defaultPageSize":20,"maxResultsPerPage":500,"allowSelfApplication":false,"adminNotifyEmail":"a@b.com"}`))
	c.Request.Header.Set("Content-Type", "application/json")
	HandleUpdateSystemSetting(c)
	if w.Code != http.StatusOK {
		t.Fatalf("保存失败: %d %s", w.Code, w.Body.String())
	}

	got := getSystemSetting()
	if got.SystemTitle != "我的系统" || got.TokenValidDays != 7 || got.DefaultPageSize != 20 || got.AllowSelfApplication {
		t.Fatalf("保存后读取错误: %+v", got)
	}

	// 非法值应回退默认
	w2 := httptest.NewRecorder()
	c2, _ := gin.CreateTestContext(w2)
	c2.Request = httptest.NewRequest(http.MethodPut, "/", strings.NewReader(
		`{"systemTitle":"","tokenValidDays":0,"defaultPageSize":0,"maxResultsPerPage":0}`))
	c2.Request.Header.Set("Content-Type", "application/json")
	HandleUpdateSystemSetting(c2)
	got2 := getSystemSetting()
	if got2.TokenValidDays != 30 || got2.DefaultPageSize != 10 {
		t.Fatalf("非法值应回退默认: %+v", got2)
	}
}

func TestTokenApplicationInsertWithoutLegacyFields(t *testing.T) {
	repository.DB.Where("1 = 1").Delete(&model.TokenApplication{})
	t.Cleanup(func() { repository.DB.Where("1 = 1").Delete(&model.TokenApplication{}) })

	// 不再收集 Name/Birthday/Gender，插入仍需成功（旧列 NOT NULL 由空串满足）
	app := model.TokenApplication{Email: "a@b.com", Purpose: "test", AllowIndexes: "[]", Status: 0}
	if err := repository.DB.Create(&app).Error; err != nil {
		t.Fatalf("插入申请记录失败: %v", err)
	}

	var got model.TokenApplication
	if err := repository.DB.First(&got, app.ID).Error; err != nil {
		t.Fatal(err)
	}
	if got.Email != "a@b.com" || got.Purpose != "test" {
		t.Fatalf("读取错误: %+v", got)
	}
}
