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
	_ = repository.DB.Where("date < ?", sixMonthsAgoDate()).Delete(&model.UsageMetricIndex{}).Error
}

func increaseQueryMetric(count int64, indexUID string) {
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
		if indexUID != "" {
			if err := tx.Where("date = ? AND index_uid = ?", date, indexUID).
				FirstOrCreate(&model.UsageMetricIndex{Date: date, IndexUID: indexUID}).Error; err != nil {
				return err
			}
			if err := tx.Model(&model.UsageMetricIndex{}).Where("date = ? AND index_uid = ?", date, indexUID).
				UpdateColumn("query_count", gorm.Expr("query_count + ?", count)).Error; err != nil {
				return err
			}
		}
		if err := tx.Where("date < ?", sixMonthsAgoDate()).Delete(&model.UsageMetricIndex{}).Error; err != nil {
			return err
		}
		return tx.Where("date < ?", sixMonthsAgoDate()).Delete(&model.UsageMetric{}).Error
	})
}

func increaseImportMetric(count int64, indexUID string) {
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
		if indexUID != "" {
			if err := tx.Where("date = ? AND index_uid = ?", date, indexUID).
				FirstOrCreate(&model.UsageMetricIndex{Date: date, IndexUID: indexUID}).Error; err != nil {
				return err
			}
			if err := tx.Model(&model.UsageMetricIndex{}).Where("date = ? AND index_uid = ?", date, indexUID).
				UpdateColumn("import_count", gorm.Expr("import_count + ?", count)).Error; err != nil {
				return err
			}
		}
		if err := tx.Where("date < ?", sixMonthsAgoDate()).Delete(&model.UsageMetric{}).Error; err != nil {
			return err
		}
		return tx.Where("date < ?", sixMonthsAgoDate()).Delete(&model.UsageMetricIndex{}).Error
	})
}

// refreshTodayStorageMetric 汇总所有实例的存储占用写入当日用量
func refreshTodayStorageMetric() {
	instances := allInstances()
	if len(instances) == 0 {
		return
	}

	var totalSize int64
	hasAny := false
	for _, instance := range instances {
		if size, ok := fetchInstanceDatabaseSize(instance); ok {
			totalSize += size
			hasAny = true
		}
	}
	if !hasAny {
		return
	}

	date := metricDate()
	_ = repository.DB.Transaction(func(tx *gorm.DB) error {
		if err := tx.Where("date = ?", date).FirstOrCreate(&model.UsageMetric{Date: date}).Error; err != nil {
			return err
		}
		if err := tx.Model(&model.UsageMetric{}).Where("date = ?", date).Update("database_size", totalSize).Error; err != nil {
			return err
		}
		return tx.Where("date < ?", sixMonthsAgoDate()).Delete(&model.UsageMetric{}).Error
	})
}

// fetchInstanceDatabaseSize 读取单个实例的 databaseSize
func fetchInstanceDatabaseSize(instance model.MeiliInstance) (int64, bool) {
	statsURL, err := url.JoinPath(instance.Host, "stats")
	if err != nil {
		return 0, false
	}
	req, err := http.NewRequest(http.MethodGet, statsURL, nil)
	if err != nil {
		return 0, false
	}
	req.Header.Set("Authorization", "Bearer "+instance.APIKey)
	req.Header.Set("X-Meili-API-Key", instance.APIKey)

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return 0, false
	}
	defer resp.Body.Close()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return 0, false
	}

	var payload map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&payload); err != nil {
		return 0, false
	}

	rawSize, ok := payload["databaseSize"]
	if !ok {
		return 0, false
	}
	switch v := rawSize.(type) {
	case float64:
		return int64(v), true
	case string:
		if n, convErr := strconv.ParseInt(v, 10, 64); convErr == nil {
			return n, true
		}
	}
	return 0, false
}

func HandleGetUsageMetrics(c *gin.Context) {
	cleanupUsageMetrics()
	refreshTodayStorageMetric()

	var metrics []model.UsageMetric
	if err := repository.DB.Order("date asc").Find(&metrics).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to load usage metrics"})
		return
	}

	type indexAgg struct {
		IndexUID    string `json:"indexUid"`
		QueryCount  int64  `json:"queryCount"`
		ImportCount int64  `json:"importCount"`
	}
	var indexMetrics []indexAgg
	if err := repository.DB.Model(&model.UsageMetricIndex{}).
		Select("index_uid, SUM(query_count) as query_count, SUM(import_count) as import_count").
		Group("index_uid").
		Order("query_count desc, import_count desc").
		Scan(&indexMetrics).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to load index usage metrics"})
		return
	}

	c.JSON(http.StatusOK, gin.H{"results": metrics, "indexResults": indexMetrics})
}
