package api

import (
	"crypto/rand"
	"encoding/hex"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"sync"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
)

var (
	jwtSecretOnce sync.Once
	jwtSecretVal  []byte
)

// JwtSecret 返回用于签发/校验的密钥（惰性加载，避免泄露与测试副作用）
// 优先级：环境变量 JWT_SECRET > 密钥文件 > 随机生成并持久化
func JwtSecret() []byte {
	jwtSecretOnce.Do(func() {
		jwtSecretVal = loadJWTSecret()
	})
	return jwtSecretVal
}

func loadJWTSecret() []byte {
	if s := strings.TrimSpace(os.Getenv("JWT_SECRET")); s != "" {
		log.Println("[JWT] 使用环境变量 JWT_SECRET")
		return []byte(s)
	}

	path := os.Getenv("JWT_SECRET_FILE")
	if path == "" {
		path = filepath.Join("data", "jwt_secret")
	}

	if data, err := os.ReadFile(path); err == nil && len(strings.TrimSpace(string(data))) > 0 {
		log.Printf("[JWT] 从密钥文件加载: %s", path)
		return []byte(strings.TrimSpace(string(data)))
	}

	buf := make([]byte, 32)
	if _, err := rand.Read(buf); err != nil {
		log.Fatalf("[JWT] 生成密钥失败: %v", err)
	}
	secret := []byte(hex.EncodeToString(buf))

	if dir := filepath.Dir(path); dir != "." {
		_ = os.MkdirAll(dir, 0755)
	}
	if err := os.WriteFile(path, secret, 0600); err != nil {
		log.Printf("[JWT] 警告: 密钥持久化失败(%v)，重启后会变化，请设置环境变量 JWT_SECRET", err)
	} else {
		log.Printf("[JWT] 已生成并持久化新密钥: %s（建议改用环境变量 JWT_SECRET）", path)
	}
	return secret
}

// CORSMiddleware 处理跨域问题
func CORSMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
		c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, GET, OPTIONS, PUT, DELETE")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization, App-Key, App-Token")
		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(204)
			return
		}
		c.Next()
	}
}

// AuthMiddleware 校验 JWT token
func AuthMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		tokenStr := c.GetHeader("Authorization")
		if tokenStr == "" || !strings.HasPrefix(tokenStr, "Bearer ") {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "Missing or invalid token"})
			return
		}
		tokenStr = strings.TrimPrefix(tokenStr, "Bearer ")

		token, err := jwt.Parse(tokenStr, func(token *jwt.Token) (interface{}, error) {
			return JwtSecret(), nil
		})

		if err != nil || !token.Valid {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "Invalid token"})
			return
		}

		claims, _ := token.Claims.(jwt.MapClaims)
		role, ok := claims["role"].(string)

		// 网关校验：只有后台路由必须限制 admin，这里判断一下是否是 admin 组路由
		if strings.HasPrefix(c.Request.URL.Path, "/api/v1/admin") && (!ok || role != "admin") {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "Admin access required"})
			return
		}

		c.Set("userId", claims["userId"])
		c.Set("role", claims["role"])
		c.Next()
	}
}
