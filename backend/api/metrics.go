package api

import (
	"backend/model"
	"backend/repository"
	"encoding/json"
	"net/http"
	"net/url"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

func metricDate() string {
	return time.Now().Format("2006-01-02")
}

func sixMonthsAgoDate() string {
	return time.Now().AddDate(0, -6, 0).Format("2006-01-02")
}

func cleanupUsageMetrics() {
	_ = repository.DB.Where("date < ?", sixMonthsAgoDate()).Delete(&model.UsageMetric{}).Error
}

func increaseQueryMetric(count int64) {
	if count <= 0 {
		return
	}
	date := metricDate()
	_ = repository.DB.Transaction(func(tx *gorm.DB) error {
		if err := tx.Where("date = ?", date).FirstOrCreate(&model.UsageMetric{Date: date}).Error; err != nil {
			return err
		}
		if err := tx.Model(&model.UsageMetric{}).Where("date = ?", date).
			UpdateColumn("query_count", gorm.Expr("query_count + ?", count)).Error; err != nil {
			return err
		}
		return tx.Where("date < ?", sixMonthsAgoDate()).Delete(&model.UsageMetric{}).Error
	})
}

func increaseImportMetric(count int64) {
	if count <= 0 {
		return
	}
	date := metricDate()
	_ = repository.DB.Transaction(func(tx *gorm.DB) error {
		if err := tx.Where("date = ?", date).FirstOrCreate(&model.UsageMetric{Date: date}).Error; err != nil {
			return err
		}
		if err := tx.Model(&model.UsageMetric{}).Where("date = ?", date).
			UpdateColumn("import_count", gorm.Expr("import_count + ?", count)).Error; err != nil {
			return err
		}
		if err := tx.Where("date < ?", sixMonthsAgoDate()).Delete(&model.UsageMetric{}).Error; err != nil {
			return err
		}
		return nil
	})
}

func refreshTodayStorageMetric() {
	var instance model.MeiliInstance
	if err := repository.DB.First(&instance).Error; err != nil {
		return
	}

	statsURL, err := url.JoinPath(instance.Host, "stats")
	if err != nil {
		return
	}
	req, err := http.NewRequest(http.MethodGet, statsURL, nil)
	if err != nil {
		return
	}
	req.Header.Set("Authorization", "Bearer "+instance.APIKey)
	req.Header.Set("X-Meili-API-Key", instance.APIKey)

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return
	}

	var payload map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&payload); err != nil {
		return
	}

	rawSize, ok := payload["databaseSize"]
	if !ok {
		return
	}
	var dbSize int64
	switch v := rawSize.(type) {
	case float64:
		dbSize = int64(v)
	case string:
		n, convErr := strconv.ParseInt(v, 10, 64)
		if convErr != nil {
			return
		}
		dbSize = n
	default:
		return
	}

	date := metricDate()
	_ = repository.DB.Transaction(func(tx *gorm.DB) error {
		if err := tx.Where("date = ?", date).FirstOrCreate(&model.UsageMetric{Date: date}).Error; err != nil {
			return err
		}
		if err := tx.Model(&model.UsageMetric{}).Where("date = ?", date).Update("database_size", dbSize).Error; err != nil {
			return err
		}
		return tx.Where("date < ?", sixMonthsAgoDate()).Delete(&model.UsageMetric{}).Error
	})
}

func HandleGetUsageMetrics(c *gin.Context) {
	cleanupUsageMetrics()
	refreshTodayStorageMetric()

	var metrics []model.UsageMetric
	if err := repository.DB.Order("date asc").Find(&metrics).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to load usage metrics"})
		return
	}
	c.JSON(http.StatusOK, gin.H{"results": metrics})
}
