package api

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httputil"
	"net/url"
	"strings"
	"time"

	"backend/model"
	"backend/repository"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"gorm.io/gorm"
)

// HandleProxy 拦截并转发所有发往 Meilisearch 的请求
func HandleProxy(c *gin.Context) {
	// NOTE: 核心隔离修复——通过 App-Key 路由到对应实例，而非沿用第一个实例
	instance, currentApp := resolveInstanceByAppKey(c)

	proxyPath := c.Param("proxyPath")
	parts := strings.Split(strings.TrimPrefix(proxyPath, "/"), "/")

	// 尝试解析可选的系统管理员 JWT 以便让管理员无视锁限制
	isAdmin := false
	authHdr := c.GetHeader("Authorization")
	if strings.HasPrefix(authHdr, "Bearer ") {
		tokenStr := strings.TrimPrefix(authHdr, "Bearer ")
		token, _ := jwt.Parse(tokenStr, func(token *jwt.Token) (interface{}, error) { return JwtSecret, nil })
		if token != nil && token.Valid {
			if claims, ok := token.Claims.(jwt.MapClaims); ok {
				if role, _ := claims["role"].(string); role == "admin" {
					isAdmin = true
				}
			}
		}
	}

	// 安全鉴权与只读(Read-Only)限制
	if !isAdmin {
		// 阻断普通的写入/删除请求 (只允许 GET/OPTIONS 或者 POST查询)
		method := c.Request.Method
		pathSuffix := c.Request.URL.Path
		isSearchPost := method == "POST" && (strings.HasSuffix(pathSuffix, "/search") || strings.HasSuffix(pathSuffix, "/multi-search"))
		if method != "GET" && method != "OPTIONS" && !isSearchPost {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "只读模式 (Read-Only Mode): 缺乏系统管理员权限，禁止写入或修改索引配置。"})
			return
		}

		if len(parts) >= 2 && parts[0] == "indexes" {
			requestedIndex := parts[1]

			// 普通访客，检查该 Index 是否在前台隐藏
			var indexConf model.IndexConfig
			isVisible := true
			isLocked := false
			if err := repository.DB.Where("uid = ?", requestedIndex).First(&indexConf).Error; err == nil {
				isVisible = indexConf.IsVisible
				isLocked = indexConf.IsLocked
			}

			if !isVisible {
				c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "该索引已被前台隐藏"})
				return
			}

			if isLocked {
				// 如果被上锁，需要验证 App-Token 是否拥有权限
				userToken := c.GetHeader("App-Token")
				hasAccess := false

				if userToken != "" {
					var tok model.AccessToken
					if err := repository.DB.Where("token = ?", userToken).First(&tok).Error; err == nil {
						if tok.ExpiresAt != nil && tok.ExpiresAt.Before(time.Now()) {
							// NOTE: Token 已过期，执行软删除（model 已配置 DeletedAt）
							repository.DB.Delete(&tok)
						} else {
							// 防跨应用盗用 Token: 如果 Token 已绑定 App，则限制其只能在归属 App 下使用
							if currentApp != nil && tok.AppID != 0 && tok.AppID != currentApp.ID {
								c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "凭证不属于该应用，拒绝跨应用访问"})
								return
							}
							var allowedArr []string
							json.Unmarshal([]byte(tok.AllowIndexes), &allowedArr)
							for _, a := range allowedArr {
								if a == requestedIndex || a == "*" {
									hasAccess = true
									break
								}
							}
						}
					}
				}

				if !hasAccess {
					c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "该库已被锁定，请输入正确的访问凭证(Token)"})
					return
				}
			}
		}

		// 运行配额判定与用量递增
		isSearchReq := strings.HasSuffix(pathSuffix, "/search") || strings.HasSuffix(pathSuffix, "/multi-search")
		isImportReq := (strings.HasSuffix(pathSuffix, "/documents") || strings.Contains(pathSuffix, "/documents?")) &&
			(method == http.MethodPost || method == http.MethodPut)

		userToken := c.GetHeader("App-Token")
		appKey := c.GetHeader("App-Key")

		if userToken != "" {
			if isSearchReq {
				if allowed, errMsg := checkAndIncrementTokenQuota(userToken, true, 1); !allowed {
					c.AbortWithStatusJSON(http.StatusPaymentRequired, gin.H{"error": errMsg})
					return
				}
			} else if isImportReq {
				if count := estimateImportCountFromBody(c); count > 0 {
					if allowed, errMsg := checkAndIncrementTokenQuota(userToken, false, count); !allowed {
						c.AbortWithStatusJSON(http.StatusPaymentRequired, gin.H{"error": errMsg})
						return
					}
				}
			}
		}

		if appKey != "" {
			if isSearchReq {
				if allowed, errMsg := checkAndIncrementAppQuota(appKey, true, 1); !allowed {
					c.AbortWithStatusJSON(http.StatusPaymentRequired, gin.H{"error": errMsg})
					return
				}
			} else if isImportReq {
				if count := estimateImportCountFromBody(c); count > 0 {
					if allowed, errMsg := checkAndIncrementAppQuota(appKey, false, count); !allowed {
						c.AbortWithStatusJSON(http.StatusPaymentRequired, gin.H{"error": errMsg})
						return
					}
				}
			}
		}

		// 执行应用级索引白名单限制
		// NOTE: 如果 Application 配置了 AllowIndexes，则只允许访问白名单内的索引，实现应用级数据隔离
		if currentApp != nil && len(parts) >= 2 && parts[0] == "indexes" {
			var appAllowed []string
			json.Unmarshal([]byte(currentApp.AllowIndexes), &appAllowed)
			if len(appAllowed) > 0 {
				requestedIdx := parts[1]
				appHasAccess := false
				for _, a := range appAllowed {
					if a == requestedIdx || a == "*" {
						appHasAccess = true
						break
					}
				}
				if !appHasAccess {
					c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "该应用没有权限访问该索引，应用级白名单限制"})
					return
				}
			}
		}
	}

	method := c.Request.Method
	pathSuffix := c.Request.URL.Path
	indexUID := extractIndexUIDFromProxyPath(proxyPath)
	isSearchReq := strings.HasSuffix(pathSuffix, "/search") || strings.HasSuffix(pathSuffix, "/multi-search")
	if isSearchReq && (method == http.MethodGet || method == http.MethodPost) {
		increaseQueryMetric(1, indexUID)
	}

	if (strings.HasSuffix(pathSuffix, "/documents") || strings.Contains(pathSuffix, "/documents?")) &&
		(method == http.MethodPost || method == http.MethodPut) {
		if n := estimateImportCountFromBody(c); n > 0 {
			increaseImportMetric(n, indexUID)
			refreshTodayStorageMetric()
		}
	}

	target, _ := url.Parse(instance.Host)
	proxy := httputil.NewSingleHostReverseProxy(target)

	originalDirector := proxy.Director
	proxy.Director = func(req *http.Request) {
		originalDirector(req)
		req.Host = target.Host
		req.URL.Path = proxyPath
		req.Header.Set("Authorization", "Bearer "+instance.APIKey)
		req.Header.Set("X-Meili-API-Key", instance.APIKey)
		// 清除客户端传入的 App-Key / App-Token，不应将内部凭证泄漏给 Meilisearch
		req.Header.Del("App-Key")
		req.Header.Del("App-Token")
	}

	proxy.ServeHTTP(c.Writer, c.Request)
}

// resolveInstanceByAppKey 根据请求中的 App-Key 头查找对应的 MeiliInstance
// NOTE: 这是实现多租户实例隔离的核心 — 每个 App 可以连接到不同的 Meilisearch 节点
func resolveInstanceByAppKey(c *gin.Context) (model.MeiliInstance, *model.Application) {
	var instance model.MeiliInstance
	appKey := c.GetHeader("App-Key")
	if appKey == "" {
		appKey = "default-app-key"
	}

	var app model.Application
	if err := repository.DB.Where("app_key = ?", appKey).First(&app).Error; err == nil {
		// 找到对应的 Application，尝试加载其绑定的 MeiliInstance
		if err2 := repository.DB.First(&instance, app.InstanceID).Error; err2 == nil {
			return instance, &app
		}
	}

	// HACK: 找不到匹配的 App 或 Instance，回退到第一个实例
	repository.DB.First(&instance)
	return instance, nil
}

func estimateImportCountFromBody(c *gin.Context) int64 {
	if c.Request.Body == nil {
		return 0
	}
	body, err := io.ReadAll(c.Request.Body)
	if err != nil {
		return 0
	}
	c.Request.Body = io.NopCloser(bytes.NewReader(body))
	if len(bytes.TrimSpace(body)) == 0 {
		return 0
	}

	var arr []map[string]any
	if err := json.Unmarshal(body, &arr); err == nil {
		return int64(len(arr))
	}
	var single map[string]any
	if err := json.Unmarshal(body, &single); err == nil && len(single) > 0 {
		return 1
	}
	return 0
}

func extractIndexUIDFromProxyPath(proxyPath string) string {
	parts := strings.Split(strings.Trim(strings.TrimPrefix(proxyPath, "/"), "/"), "/")
	if len(parts) >= 2 && parts[0] == "indexes" && parts[1] != "" {
		return parts[1]
	}
	return ""
}

// checkAndIncrementTokenQuota 检查并递增 Token 的每日搜索/导入配额
func checkAndIncrementTokenQuota(tokenStr string, isQuery bool, count int64) (bool, string) {
	if tokenStr == "" {
		return true, ""
	}
	var tok model.AccessToken
	if err := repository.DB.Where("token = ?", tokenStr).First(&tok).Error; err != nil {
		// Token 不存在或失效，由后续鉴权流程处理，这里不拦截配额
		return true, ""
	}

	limit := tok.MaxQueriesPerDay
	if !isQuery {
		limit = tok.MaxImportsPerDay
	}

	// <= 0 表示不限制
	if limit <= 0 {
		return true, ""
	}

	date := time.Now().Format("2006-01-02")
	var allowed = true
	var errMsg = ""

	err := repository.DB.Transaction(func(tx *gorm.DB) error {
		var metric model.TokenUsageMetric
		if err := tx.Where("date = ? AND token = ?", date, tokenStr).FirstOrCreate(&metric, model.TokenUsageMetric{
			Date:  date,
			Token: tokenStr,
		}).Error; err != nil {
			return err
		}

		currentUsage := metric.QueryCount
		if !isQuery {
			currentUsage = metric.ImportCount
		}

		if currentUsage+count > limit {
			allowed = false
			if isQuery {
				errMsg = fmt.Sprintf("Token 每日查询限额已超出限制（限制: %d 次/天，今日已使用: %d 次）", limit, currentUsage)
			} else {
				errMsg = fmt.Sprintf("Token 每日导入限额已超出限制（限制: %d 条/天，今日已使用: %d 条）", limit, currentUsage)
			}
			return fmt.Errorf("quota exceeded")
		}

		if isQuery {
			if err := tx.Model(&metric).UpdateColumn("query_count", gorm.Expr("query_count + ?", count)).Error; err != nil {
				return err
			}
		} else {
			if err := tx.Model(&metric).UpdateColumn("import_count", gorm.Expr("import_count + ?", count)).Error; err != nil {
				return err
			}
		}
		return nil
	})

	if err != nil && !allowed {
		return false, errMsg
	}
	return true, ""
}

// checkAndIncrementAppQuota 检查并递增 App 的每日搜索/导入配额
func checkAndIncrementAppQuota(appKeyStr string, isQuery bool, count int64) (bool, string) {
	if appKeyStr == "" {
		return true, ""
	}
	var app model.Application
	if err := repository.DB.Where("app_key = ?", appKeyStr).First(&app).Error; err != nil {
		// App 不存在或失效，由后续路由配置拦截，这里不拦截配额
		return true, ""
	}

	limit := app.MaxQueriesPerDay
	if !isQuery {
		limit = app.MaxImportsPerDay
	}

	// <= 0 表示不限制
	if limit <= 0 {
		return true, ""
	}

	date := time.Now().Format("2006-01-02")
	var allowed = true
	var errMsg = ""

	err := repository.DB.Transaction(func(tx *gorm.DB) error {
		var metric model.AppUsageMetric
		if err := tx.Where("date = ? AND app_key = ?", date, appKeyStr).FirstOrCreate(&metric, model.AppUsageMetric{
			Date:   date,
			AppKey: appKeyStr,
		}).Error; err != nil {
			return err
		}

		currentUsage := metric.QueryCount
		if !isQuery {
			currentUsage = metric.ImportCount
		}

		if currentUsage+count > limit {
			allowed = false
			if isQuery {
				errMsg = fmt.Sprintf("App 每日查询限额已超出限制（限制: %d 次/天，今日已使用: %d 次）", limit, currentUsage)
			} else {
				errMsg = fmt.Sprintf("App 每日导入限额已超出限制（限制: %d 条/天，今日已使用: %d 条）", limit, currentUsage)
			}
			return fmt.Errorf("quota exceeded")
		}

		if isQuery {
			if err := tx.Model(&metric).UpdateColumn("query_count", gorm.Expr("query_count + ?", count)).Error; err != nil {
				return err
			}
		} else {
			if err := tx.Model(&metric).UpdateColumn("import_count", gorm.Expr("import_count + ?", count)).Error; err != nil {
				return err
			}
		}
		return nil
	})

	if err != nil && !allowed {
		return false, errMsg
	}
	return true, ""
}
