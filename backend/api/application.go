package api

import (
	"crypto/tls"
	"encoding/json"
	"fmt"
	"math/rand"
	"net/http"
	"net/smtp"
	"os"
	"time"

	"backend/model"
	"backend/repository"
	"backend/schema"

	"github.com/gin-gonic/gin"
)

// HandleSendCode 发送邮箱验证码
func HandleSendCode(c *gin.Context) {
	var req schema.SendCodeRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的邮箱地址"})
		return
	}

	// 生成 6 位验证码
	code := fmt.Sprintf("%06d", rand.Intn(1000000))
	expiresAt := time.Now().Add(180 * time.Minute)

	// 保存到数据库 (覆盖旧的)
	verification := model.EmailVerification{
		Email:     req.Email,
		Code:      code,
		ExpiresAt: expiresAt,
	}
	if err := repository.DB.Save(&verification).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "保存验证码失败"})
		return
	}

	// 发送邮件
	err := sendEmailVia163(req.Email, code)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "邮件发送失败: " + err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "验证码已发送至您的邮箱，请查收"})
}

func sendEmailVia163(to, code string) error {
	user := os.Getenv("SMTP_USER")
	pass := os.Getenv("SMTP_PASS") // 163 授权码
	host := "smtp.163.com"
	port := "465"

	if user == "" || pass == "" {
		return fmt.Errorf("SMTP 配置缺失 (请在环境变量中设置 SMTP_USER 和 SMTP_PASS)")
	}

	header := make(map[string]string)
	header["From"] = user
	header["To"] = to
	header["Subject"] = "验证码 - Meilisearch UI Token 申请"
	header["Content-Type"] = "text/html; charset=UTF-8"

	message := ""
	for k, v := range header {
		message += fmt.Sprintf("%s: %s\r\n", k, v)
	}
	message += "\r\n" + fmt.Sprintf("您的验证码是：<b style='color:#4f46e5;font-size:24px;'>%s</b><br><br>有效期为 180 分钟。请勿告诉他人。", code)

	auth := smtp.PlainAuth("", user, pass, host)

	tlsconfig := &tls.Config{
		InsecureSkipVerify: true,
		ServerName:         host,
	}

	conn, err := tls.Dial("tcp", host+":"+port, tlsconfig)
	if err != nil {
		return err
	}

	client, err := smtp.NewClient(conn, host)
	if err != nil {
		return err
	}

	if err = client.Auth(auth); err != nil {
		return err
	}

	if err = client.Mail(user); err != nil {
		return err
	}

	if err = client.Rcpt(to); err != nil {
		return err
	}

	w, err := client.Data()
	if err != nil {
		return err
	}

	_, err = w.Write([]byte(message))
	if err != nil {
		return err
	}

	err = w.Close()
	if err != nil {
		return err
	}

	client.Quit()
	return nil
}

// HandleSubmitApplication 提交申请
func HandleSubmitApplication(c *gin.Context) {
	var req schema.TokenApplicationSubmitRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "参数校验失败"})
		return
	}

	// 验证码校验
	var verification model.EmailVerification
	if err := repository.DB.Where("email = ? AND code = ?", req.Email, req.Code).First(&verification).Error; err != nil {
		c.JSON(http.StatusForbidden, gin.H{"error": "验证码错误或不存在"})
		return
	}

	if verification.ExpiresAt.Before(time.Now()) {
		c.JSON(http.StatusForbidden, gin.H{"error": "验证码已过期"})
		return
	}

	// 转换索引列表为 JSON
	indexesJSON, _ := json.Marshal(req.AllowIndexes)

	// 创建申请记录
	app := model.TokenApplication{
		Email:        req.Email,
		Name:         req.Name,
		Birthday:     req.Birthday,
		Gender:       req.Gender,
		Purpose:      req.Purpose,
		AllowIndexes: string(indexesJSON),
		Status:       0, // 待审批
	}

	if err := repository.DB.Create(&app).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "提交申请失败"})
		return
	}

	// 验证码用完即毁
	repository.DB.Delete(&verification)

	c.JSON(http.StatusOK, gin.H{"message": "申请已提交，请等待管理员审批"})
}

// HandleGetApplications (Admin) 获取申请列表
func HandleGetApplications(c *gin.Context) {
	var apps []model.TokenApplication
	repository.DB.Order("created_at desc").Find(&apps)
	c.JSON(http.StatusOK, apps)
}

// HandleApproveApplication (Admin) 通过申请
func HandleApproveApplication(c *gin.Context) {
	id := c.Param("id")
	var app model.TokenApplication
	if err := repository.DB.First(&app, id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "申请不存在"})
		return
	}

	if app.Status != 0 {
		c.JSON(http.StatusBadRequest, gin.H{"error": "申请已处理，请勿重复操作"})
		return
	}

	// 1. 更新状态
	app.Status = 1 // 通过
	repository.DB.Save(&app)

	// 2. 自动生成 Token 记录
	newToken := fmt.Sprintf("tk_%d%03d", time.Now().Unix(), rand.Intn(1000))
	
	accessToken := model.AccessToken{
		Token:        newToken,
		AllowIndexes: app.AllowIndexes,
		Description:  fmt.Sprintf("申请人: %s (%s) 用途: %s", app.Name, app.Email, app.Purpose),
	}
	repository.DB.Create(&accessToken)

	c.JSON(http.StatusOK, gin.H{"message": "申请已通过，并已生成对应 Token", "token": newToken})
}

// HandleRejectApplication (Admin) 驳回申请
func HandleRejectApplication(c *gin.Context) {
	id := c.Param("id")
	var app model.TokenApplication
	if err := repository.DB.First(&app, id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "申请不存在"})
		return
	}
	app.Status = 2 // 驳回
	repository.DB.Save(&app)
	c.JSON(http.StatusOK, gin.H{"message": "已驳回该申请"})
}
