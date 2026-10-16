package api

import (
	"net/http"
	"strconv"

	"backend/model"
	"backend/repository"
	"backend/syncer"

	"github.com/gin-gonic/gin"
)

// ---- MySQL 数据源 ----

func HandleGetSyncSources(c *gin.Context) {
	var list []model.SyncSource
	repository.DB.Order("id asc").Find(&list)
	out := make([]gin.H, 0, len(list))
	for _, s := range list {
		out = append(out, gin.H{
			"id":          s.ID,
			"name":        s.Name,
			"host":        s.Host,
			"port":        s.Port,
			"user":        s.User,
			"hasPassword": s.Password != "",
			"createdAt":   s.CreatedAt,
		})
	}
	c.JSON(http.StatusOK, out)
}

func HandleCreateSyncSource(c *gin.Context) {
	var req struct {
		Name     string `json:"name" binding:"required"`
		Host     string `json:"host" binding:"required"`
		Port     int    `json:"port"`
		User     string `json:"user" binding:"required"`
		Password string `json:"password"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "参数不合法"})
		return
	}
	if req.Port == 0 {
		req.Port = 3306
	}
	s := model.SyncSource{Name: req.Name, Host: req.Host, Port: req.Port, User: req.User, Password: req.Password}
	if err := repository.DB.Create(&s).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "创建失败: " + err.Error()})
		return
	}
	c.JSON(http.StatusOK, s)
}

func HandleUpdateSyncSource(c *gin.Context) {
	id := c.Param("id")
	var s model.SyncSource
	if err := repository.DB.First(&s, id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "数据源不存在"})
		return
	}
	var req struct {
		Name     string `json:"name"`
		Host     string `json:"host"`
		Port     int    `json:"port"`
		User     string `json:"user"`
		Password string `json:"password"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "参数不合法"})
		return
	}
	if req.Name != "" {
		s.Name = req.Name
	}
	if req.Host != "" {
		s.Host = req.Host
	}
	if req.Port > 0 {
		s.Port = req.Port
	}
	if req.User != "" {
		s.User = req.User
	}
	if req.Password != "" { // 留空保留
		s.Password = req.Password
	}
	if err := repository.DB.Save(&s).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "保存失败"})
		return
	}
	c.JSON(http.StatusOK, s)
}

func HandleDeleteSyncSource(c *gin.Context) {
	id := c.Param("id")
	repository.DB.Delete(&model.SyncSource{}, id)
	c.JSON(http.StatusOK, gin.H{"message": "已删除"})
}

// ---- 同步任务 ----

func HandleGetSyncTasks(c *gin.Context) {
	var list []model.SyncTask
	repository.DB.Order("id asc").Find(&list)
	counts := syncer.DeadLetterCounts()
	// 以运行时为准修正 running 状态
	for i := range list {
		list[i].Running = syncer.Default.IsRunning(list[i].ID)
		list[i].DeadLetterCount = counts[list[i].ID]
	}
	c.JSON(http.StatusOK, list)
}

// ---- 死信 ----

func HandleGetDeadLetters(c *gin.Context) {
	taskID, _ := strconv.Atoi(c.Query("taskId"))
	list := syncer.Default.DeadLetters(uint(taskID), 200)
	c.JSON(http.StatusOK, list)
}

func HandleReplayDeadLetter(c *gin.Context) {
	id, _ := strconv.Atoi(c.Param("id"))
	if err := syncer.Default.ReplayDeadLetter(uint(id)); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "重放成功，该死信已清除"})
}

func HandleDeleteDeadLetter(c *gin.Context) {
	id, _ := strconv.Atoi(c.Param("id"))
	syncer.Default.DeleteDeadLetter(uint(id))
	c.JSON(http.StatusOK, gin.H{"message": "已删除"})
}

func HandleCreateSyncTask(c *gin.Context) {
	var req model.SyncTask
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "参数不合法"})
		return
	}
	if req.TableName == "" || req.JoinSQL == "" || req.IndexUID == "" || req.SourceID == 0 || req.InstanceID == 0 {
		c.JSON(http.StatusBadRequest, gin.H{"error": "请完整填写数据源、库表、JoinSQL、目标实例与索引"})
		return
	}
	req.Running = false
	if err := repository.DB.Create(&req).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "创建失败: " + err.Error()})
		return
	}
	c.JSON(http.StatusOK, req)
}

func HandleUpdateSyncTask(c *gin.Context) {
	id := c.Param("id")
	var t model.SyncTask
	if err := repository.DB.First(&t, id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "任务不存在"})
		return
	}
	if syncer.Default.IsRunning(t.ID) {
		c.JSON(http.StatusBadRequest, gin.H{"error": "任务运行中，请先停止再修改"})
		return
	}
	var req model.SyncTask
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "参数不合法"})
		return
	}
	req.ID = t.ID
	req.Running = false
	req.Position = t.Position // 位置不允许手工覆盖
	if err := repository.DB.Save(&req).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "保存失败"})
		return
	}
	c.JSON(http.StatusOK, req)
}

func HandleDeleteSyncTask(c *gin.Context) {
	id := c.Param("id")
	var t model.SyncTask
	if err := repository.DB.First(&t, id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "任务不存在"})
		return
	}
	if syncer.Default.IsRunning(t.ID) {
		_ = syncer.Default.Stop(t.ID)
	}
	repository.DB.Delete(&model.SyncTask{}, id)
	c.JSON(http.StatusOK, gin.H{"message": "已删除"})
}

func HandleStartSyncTask(c *gin.Context) {
	id := c.Param("id")
	var t model.SyncTask
	if err := repository.DB.First(&t, id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "任务不存在"})
		return
	}
	if err := syncer.Default.Start(t.ID); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "已启动"})
}

func HandleStopSyncTask(c *gin.Context) {
	id := c.Param("id")
	var t model.SyncTask
	if err := repository.DB.First(&t, id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "任务不存在"})
		return
	}
	if err := syncer.Default.Stop(t.ID); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "已停止"})
}

func HandleFullSyncTask(c *gin.Context) {
	id := c.Param("id")
	var t model.SyncTask
	if err := repository.DB.First(&t, id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "任务不存在"})
		return
	}
	n, err := syncer.Default.FullSync(t.ID)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "全量初始化完成", "count": n})
}
