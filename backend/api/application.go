package api

import (
	"encoding/json"
	"fmt"
	"math/rand"
	"net/http"
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

	setting := getSystemSetting()
	if !setting.AllowSelfApplication {
		c.JSON(http.StatusForbidden, gin.H{"error": "系统当前未开放自助申请，请联系管理员"})
		return
	}

	// 频率检查：N 天内只能申请一次（N 可在后台配置）
	intervalDays := setting.ApplicationIntervalDays
	if intervalDays <= 0 {
		intervalDays = 30
	}
	var lastApp model.TokenApplication
	if err := repository.DB.Unscoped().Where("email = ?", req.Email).Order("created_at desc").First(&lastApp).Error; err == nil {
		nextAvailableTime := lastApp.CreatedAt.Add(time.Duration(intervalDays) * 24 * time.Hour)
		if time.Now().Before(nextAvailableTime) {
			remainingDays := int(time.Until(nextAvailableTime).Hours()/24) + 1
			c.JSON(http.StatusForbidden, gin.H{"error": fmt.Sprintf("该邮箱 %d 天内只能申请一次，请在 %d 天后再次尝试", intervalDays, remainingDays)})
			return
		}
	}

	// 生成 6 位验证码
	code := fmt.Sprintf("%06d", rand.Intn(1000000))
	expiresAt := time.Now().Add(time.Duration(setting.VerificationTTLMinutes) * time.Minute)

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
	setting := getSystemSetting()
	subject := "验证码 - " + setting.SystemTitle + " Token 申请"
	body := fmt.Sprintf("您的验证码是：<b style='color:#4f46e5;font-size:24px;'>%s</b><br><br>有效期为 %d 分钟。请勿告诉他人。", code, setting.VerificationTTLMinutes)
	return sendSystemMail(to, subject, body)
}

// sendSystemMail 使用后台配置的 SMTP 发送邮件（未配置时回退环境变量）
func sendSystemMail(to, subject, body string) error {
	cfg, ok := getSmtpConfig()
	if !ok {
		return fmt.Errorf("SMTP 邮件服务未配置，请在后台「邮件配置」中设置")
	}
	return sendMailHTML(cfg, to, subject, body)
}

// HandleSubmitApplication 提交申请
func HandleSubmitApplication(c *gin.Context) {
	var req schema.TokenApplicationSubmitRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "参数校验失败"})
		return
	}

	setting := getSystemSetting()
	if !setting.AllowSelfApplication {
		c.JSON(http.StatusForbidden, gin.H{"error": "系统当前未开放自助申请，请联系管理员"})
		return
	}

	// 频率检查：N 天内只能申请一次 (二次校验)
	intervalDays := setting.ApplicationIntervalDays
	if intervalDays <= 0 {
		intervalDays = 30
	}
	var lastApp model.TokenApplication
	if err := repository.DB.Unscoped().Where("email = ?", req.Email).Order("created_at desc").First(&lastApp).Error; err == nil {
		if time.Since(lastApp.CreatedAt) < time.Duration(intervalDays)*24*time.Hour {
			c.JSON(http.StatusForbidden, gin.H{"error": fmt.Sprintf("申请过于频繁，请 %d 天后再试", intervalDays)})
			return
		}
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

	// 通知管理员（若配置了通知邮箱）
	if adminEmail := setting.AdminNotifyEmail; adminEmail != "" {
		body := fmt.Sprintf(
			"收到新的 Token 申请：<br><br>邮箱：%s<br>用途：%s<br>申请索引：%s",
			req.Email, req.Purpose, string(indexesJSON),
		)
		go func() {
			if err := sendSystemMail(adminEmail, setting.SystemTitle+" 新的 Token 申请", body); err != nil {
				fmt.Printf("[Notify] 管理员通知邮件发送失败: %v\n", err)
			}
		}()
	}

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

	var req schema.ApproveApplicationRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "审批参数不合法"})
		return
	}

	// 1. 更新状态
	app.Status = 1 // 通过
	repository.DB.Save(&app)

	// 2. 创建 Token 记录（审批时配置）
	newToken := req.Token
	if newToken == "" {
		newToken = fmt.Sprintf("tk_%d%03d", time.Now().Unix(), rand.Intn(1000))
	}

	allowIndexes := req.AllowIndexes
	if len(allowIndexes) == 0 {
		_ = json.Unmarshal([]byte(app.AllowIndexes), &allowIndexes)
	}
	allowIndexesJSON, _ := json.Marshal(allowIndexes)

	validDays := getSystemSetting().TokenValidDays
	if req.ValidDays != nil && *req.ValidDays > 0 {
		validDays = *req.ValidDays
	}
	expiresAt := time.Now().Add(time.Duration(validDays) * 24 * time.Hour)

	description := req.Description
	if description == "" {
		description = fmt.Sprintf("申请人邮箱: %s 用途: %s", app.Email, app.Purpose)
	}

	accessToken := model.AccessToken{
		Token:            newToken,
		AllowIndexes:     string(allowIndexesJSON),
		Description:      description,
		ExpiresAt:        &expiresAt,
		MaxQueriesPerDay: req.MaxQueriesPerDay,
		MaxImportsPerDay: req.MaxImportsPerDay,
	}
	if err := repository.DB.Create(&accessToken).Error; err != nil {
		app.Status = 0
		repository.DB.Save(&app)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "生成凭证失败: " + err.Error()})
		return
	}

	expireDate := expiresAt.Format("2006-01-02 15:04:05")
	allowIndexesText := string(allowIndexesJSON)
	mailBody := fmt.Sprintf(
		"您的申请已通过。<br><br>分发凭证(Token)：<b style='color:#16a34a;'>%s</b><br>过期时间：<b>%s</b><br>授权索引：<code>%s</code>",
		newToken,
		expireDate,
		allowIndexesText,
	)
	if err := sendSystemMail(app.Email, getSystemSetting().SystemTitle+" 申请审批通过通知", mailBody); err != nil {
		c.JSON(http.StatusOK, gin.H{
			"message": "申请已通过并生成凭证，但邮件发送失败",
			"token":   newToken,
			"warning": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "申请已通过，凭证已分发并发送邮件通知", "token": newToken, "expiresAt": expiresAt})
}

// HandleRejectApplication (Admin) 驳回申请
func HandleRejectApplication(c *gin.Context) {
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

	var req schema.RejectApplicationRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "驳回参数不合法"})
		return
	}

	app.Status = 2 // 驳回
	repository.DB.Save(&app)
	mailBody := "很抱歉，您的 Token 申请未通过审核。<br>如需继续申请，请完善申请用途后再次提交。"
	if req.RejectMessage != "" {
		mailBody = req.RejectMessage
	}
	if err := sendSystemMail(app.Email, getSystemSetting().SystemTitle+" 申请审批驳回通知", mailBody); err != nil {
		c.JSON(http.StatusOK, gin.H{
			"message": "已驳回该申请，但邮件发送失败",
			"warning": err.Error(),
		})
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "已驳回该申请，并发送邮件通知"})
}
