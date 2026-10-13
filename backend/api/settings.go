package api

import (
	"crypto/tls"
	"fmt"
	"net"
	"net/http"
	"net/smtp"
	"os"
	"strconv"
	"time"

	"backend/model"
	"backend/repository"
	"backend/schema"
	"backend/util"

	"github.com/gin-gonic/gin"
)

// defaultSystemSetting 系统设置的默认值（与历史硬编码行为保持一致）
func defaultSystemSetting() model.SystemSetting {
	return model.SystemSetting{
		SystemTitle:             util.SysTitle,
		Copyright:               "© 2026 多维搜索引擎 - 数据集散中心 · Powered by 冰城拓",
		FrontendSubtitle:        "高性能多维数据检索引擎",
		DefaultSearchTitle:      "搜索引擎 Pro",
		TokenValidDays:          30,
		ApplicationIntervalDays: 30,
		VerificationTTLMinutes:  180,
		DefaultPageSize:         10,
		MaxResultsPerPage:       1000,
		AllowSelfApplication:    true,
	}
}

// getSystemSetting 读取系统设置：数据库有则用数据库，否则返回默认值
func getSystemSetting() model.SystemSetting {
	var row model.SystemSetting
	repository.DB.Limit(1).Find(&row)
	if row.ID == 0 {
		return defaultSystemSetting()
	}
	return row
}

// HandleGetSystemSetting 返回系统设置（含默认值）
func HandleGetSystemSetting(c *gin.Context) {
	c.JSON(http.StatusOK, getSystemSetting())
}

// HandleUpdateSystemSetting 保存系统设置
func HandleUpdateSystemSetting(c *gin.Context) {
	var req model.SystemSetting
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "参数不合法"})
		return
	}

	d := defaultSystemSetting()
	if req.SystemTitle == "" {
		req.SystemTitle = d.SystemTitle
	}
	if req.DefaultSearchTitle == "" {
		req.DefaultSearchTitle = d.DefaultSearchTitle
	}
	if req.TokenValidDays <= 0 {
		req.TokenValidDays = d.TokenValidDays
	}
	if req.ApplicationIntervalDays < 0 {
		req.ApplicationIntervalDays = d.ApplicationIntervalDays
	}
	if req.VerificationTTLMinutes <= 0 {
		req.VerificationTTLMinutes = d.VerificationTTLMinutes
	}
	if req.DefaultPageSize <= 0 {
		req.DefaultPageSize = d.DefaultPageSize
	}
	if req.MaxResultsPerPage <= 0 {
		req.MaxResultsPerPage = d.MaxResultsPerPage
	}
	if req.MaxResultsPerPage < req.DefaultPageSize {
		req.MaxResultsPerPage = req.DefaultPageSize
	}

	existing := getSystemSetting()
	req.ID = existing.ID
	req.UpdatedAt = time.Now()

	if err := repository.DB.Save(&req).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "保存失败: " + err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "系统设置已保存"})
}

// loadSmtpConfigRow 读取数据库中的邮件配置行（不存在时返回零值）
func loadSmtpConfigRow() model.SmtpConfig {
	var cfg model.SmtpConfig
	repository.DB.Limit(1).Find(&cfg)
	return cfg
}

// getSmtpConfig 读取邮件配置：优先数据库，回退环境变量 SMTP_USER / SMTP_PASS
func getSmtpConfig() (model.SmtpConfig, bool) {
	if cfg := loadSmtpConfigRow(); cfg.Host != "" {
		return cfg, true
	}

	user := os.Getenv("SMTP_USER")
	pass := os.Getenv("SMTP_PASS")
	if user != "" && pass != "" {
		return model.SmtpConfig{
			Host:       "smtp.163.com",
			Port:       465,
			Username:   user,
			Password:   pass,
			Encryption: "ssl",
		}, true
	}
	return model.SmtpConfig{}, false
}

// HandleGetSmtpConfig 返回邮件配置（密码脱敏，仅返回是否已设置）
func HandleGetSmtpConfig(c *gin.Context) {
	dbHas := loadSmtpConfigRow().Host != ""

	cfg, ok := getSmtpConfig()
	if !ok {
		c.JSON(http.StatusOK, gin.H{
			"configured":  false,
			"source":      "",
			"host":        "",
			"port":        465,
			"username":    "",
			"encryption":  "ssl",
			"fromName":    "",
			"hasPassword": false,
		})
		return
	}

	source := "env"
	if dbHas {
		source = "db"
	}
	c.JSON(http.StatusOK, gin.H{
		"configured":  true,
		"source":      source,
		"host":        cfg.Host,
		"port":        cfg.Port,
		"username":    cfg.Username,
		"encryption":  cfg.Encryption,
		"fromName":    cfg.FromName,
		"hasPassword": cfg.Password != "",
	})
}

// HandleUpdateSmtpConfig 保存邮件配置到数据库
func HandleUpdateSmtpConfig(c *gin.Context) {
	var req schema.SmtpConfigRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "参数不合法"})
		return
	}
	if req.Host == "" || req.Username == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "SMTP 服务器地址与账号不能为空"})
		return
	}

	port := req.Port
	if port == 0 {
		port = 465
	}
	enc := req.Encryption
	if enc == "" {
		enc = "ssl"
	}

	cfg := loadSmtpConfigRow()
	if cfg.ID == 0 {
		cfg = model.SmtpConfig{
			Host:       req.Host,
			Port:       port,
			Username:   req.Username,
			Password:   req.Password,
			Encryption: enc,
			FromName:   req.FromName,
		}
		if err := repository.DB.Create(&cfg).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "保存失败: " + err.Error()})
			return
		}
	} else {
		cfg.Host = req.Host
		cfg.Port = port
		cfg.Username = req.Username
		if req.Password != "" { // 留空表示保留原密码
			cfg.Password = req.Password
		}
		cfg.Encryption = enc
		cfg.FromName = req.FromName
		if err := repository.DB.Save(&cfg).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "保存失败: " + err.Error()})
			return
		}
	}

	c.JSON(http.StatusOK, gin.H{"message": "邮件配置已保存"})
}

// HandleTestSmtp 使用当前配置发送一封测试邮件
func HandleTestSmtp(c *gin.Context) {
	var req schema.SmtpTestRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "请输入有效的收件邮箱"})
		return
	}
	cfg, ok := getSmtpConfig()
	if !ok {
		c.JSON(http.StatusBadRequest, gin.H{"error": "尚未配置 SMTP 邮件服务"})
		return
	}
	body := "<p>这是一封来自系统的测试邮件，收到即表示邮件配置可用。</p>"
	if err := sendMailHTML(cfg, req.To, "邮件配置测试", body); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "发送失败: " + err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "测试邮件已发送，请查收"})
}

// sendMailHTML 按配置发送 HTML 邮件，支持 ssl / starttls / none 三种加密方式
func sendMailHTML(cfg model.SmtpConfig, to, subject, body string) error {
	host := cfg.Host
	port := cfg.Port
	if port == 0 {
		port = 465
	}
	enc := cfg.Encryption
	if enc == "" {
		enc = "ssl"
	}
	from := cfg.Username

	header := map[string]string{
		"From":         formatFrom(cfg),
		"To":           to,
		"Subject":      subject,
		"Content-Type": "text/html; charset=UTF-8",
	}
	message := ""
	for k, v := range header {
		message += fmt.Sprintf("%s: %s\r\n", k, v)
	}
	message += "\r\n" + body

	addr := net.JoinHostPort(host, strconv.Itoa(port))
	auth := smtp.PlainAuth("", cfg.Username, cfg.Password, host)

	switch enc {
	case "none":
		return smtp.SendMail(addr, auth, from, []string{to}, []byte(message))
	case "starttls":
		conn, err := smtp.Dial(addr)
		if err != nil {
			return err
		}
		defer conn.Close()
		if err := conn.StartTLS(&tls.Config{ServerName: host}); err != nil {
			return err
		}
		if err := conn.Auth(auth); err != nil {
			return err
		}
		return writeMail(conn, from, to, message)
	default: // ssl（隐式 TLS，如 163 的 465 端口）
		tlsConn, err := tls.Dial("tcp", addr, &tls.Config{ServerName: host})
		if err != nil {
			return err
		}
		client, err := smtp.NewClient(tlsConn, host)
		if err != nil {
			return err
		}
		defer client.Close()
		if err := client.Auth(auth); err != nil {
			return err
		}
		return writeMail(client, from, to, message)
	}
}

func writeMail(client *smtp.Client, from, to, message string) error {
	if err := client.Mail(from); err != nil {
		return err
	}
	if err := client.Rcpt(to); err != nil {
		return err
	}
	w, err := client.Data()
	if err != nil {
		return err
	}
	if _, err := w.Write([]byte(message)); err != nil {
		return err
	}
	if err := w.Close(); err != nil {
		return err
	}
	return client.Quit()
}

func formatFrom(cfg model.SmtpConfig) string {
	if cfg.FromName != "" {
		return fmt.Sprintf("%s <%s>", cfg.FromName, cfg.Username)
	}
	return cfg.Username
}
