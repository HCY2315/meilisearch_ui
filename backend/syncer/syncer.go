// Package syncer 通过 MySQL binlog 实时把「主表 -> 联表查询结果」同步到 Meilisearch。
//
// 规则：
//   - 监听 task.TableName 的 INSERT/UPDATE：按主键回查 task.JoinSQL(:ids 占位符) 取整条文档，upsert 到 Meilisearch
//   - 监听 DELETE：按主键在 Meilisearch 中批量删除
//   - 按事务/时间攒批写入；位置(GTID)在每次成功写入后持久化，支持断点续传
package syncer

import (
	"bytes"
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"

	"backend/model"
	"backend/repository"

	"github.com/go-mysql-org/go-mysql/canal"
	"github.com/go-mysql-org/go-mysql/mysql"
	"github.com/go-mysql-org/go-mysql/replication"
	_ "github.com/go-sql-driver/mysql"
)

// Manager 管理所有同步任务的运行实例
type Manager struct {
	mu      sync.Mutex
	runners map[uint]*Runner
}

// Default 全局管理器
var Default = &Manager{runners: map[uint]*Runner{}}

// StartEnabled 启动所有已启用且未运行的任务（服务启动时调用）
func (m *Manager) StartEnabled() {
	var tasks []model.SyncTask
	repository.DB.Where("enabled = ?", true).Find(&tasks)
	for _, t := range tasks {
		if err := m.Start(t.ID); err != nil {
			repository.DB.Model(&model.SyncTask{}).Where("id = ?", t.ID).
				Updates(map[string]any{"running": false, "last_error": "自动启动失败: " + err.Error()})
		}
	}
}

// IsRunning 判断任务是否在运行
func (m *Manager) IsRunning(taskID uint) bool {
	m.mu.Lock()
	defer m.mu.Unlock()
	_, ok := m.runners[taskID]
	return ok
}

// Start 启动一个同步任务
func (m *Manager) Start(taskID uint) error {
	m.mu.Lock()
	if _, ok := m.runners[taskID]; ok {
		m.mu.Unlock()
		return fmt.Errorf("任务已在运行")
	}
	m.mu.Unlock()

	task, source, instance, err := loadConfig(taskID)
	if err != nil {
		return err
	}

	r := &Runner{task: task, source: source, instance: instance}
	if err := r.start(); err != nil {
		repository.DB.Model(&model.SyncTask{}).Where("id = ?", task.ID).
			Updates(map[string]any{"running": false, "last_error": err.Error()})
		return err
	}

	m.mu.Lock()
	m.runners[taskID] = r
	m.mu.Unlock()
	repository.DB.Model(&model.SyncTask{}).Where("id = ?", task.ID).
		Updates(map[string]any{"running": true, "last_error": ""})
	return nil
}

// Stop 停止一个同步任务
func (m *Manager) Stop(taskID uint) error {
	m.mu.Lock()
	r, ok := m.runners[taskID]
	if ok {
		delete(m.runners, taskID)
	}
	m.mu.Unlock()
	if !ok {
		return fmt.Errorf("任务未在运行")
	}
	r.stop()
	repository.DB.Model(&model.SyncTask{}).Where("id = ?", taskID).
		Updates(map[string]any{"running": false})
	return nil
}

// StopAll 停止所有任务
func (m *Manager) StopAll() {
	m.mu.Lock()
	ids := make([]uint, 0, len(m.runners))
	for id := range m.runners {
		ids = append(ids, id)
	}
	m.mu.Unlock()
	for _, id := range ids {
		_ = m.Stop(id)
	}
}

// ---- 死信 ----

// DeadLetters 查询死信（taskID=0 返回全部，按时间倒序）
func (m *Manager) DeadLetters(taskID uint, limit int) []model.SyncDeadLetter {
	if limit <= 0 {
		limit = 200
	}
	var list []model.SyncDeadLetter
	q := repository.DB.Order("id desc").Limit(limit)
	if taskID > 0 {
		q = q.Where("task_id = ?", taskID)
	}
	q.Find(&list)
	return list
}

// DeadLetterCounts 各任务的死信数量
func DeadLetterCounts() map[uint]int64 {
	type row struct {
		TaskID uint
		Cnt    int64
	}
	var rows []row
	repository.DB.Model(&model.SyncDeadLetter{}).
		Select("task_id, count(*) as cnt").Group("task_id").Scan(&rows)
	out := map[uint]int64{}
	for _, r := range rows {
		out[r.TaskID] = r.Cnt
	}
	return out
}

// ReplayDeadLetter 重放一条死信（成功则删除该死信记录）
func (m *Manager) ReplayDeadLetter(id uint) error {
	var dl model.SyncDeadLetter
	if err := repository.DB.First(&dl, id).Error; err != nil {
		return fmt.Errorf("死信不存在")
	}
	var payload struct {
		Upserts []string `json:"upserts"`
		Deletes []string `json:"deletes"`
	}
	if err := json.Unmarshal([]byte(dl.Payload), &payload); err != nil {
		return fmt.Errorf("死信 payload 解析失败: %w", err)
	}

	task, source, instance, err := loadConfig(dl.TaskID)
	if err != nil {
		return err
	}
	db, err := openMySQL(source)
	if err != nil {
		return fmt.Errorf("连接 MySQL 失败: %w", err)
	}
	defer db.Close()

	r := &Runner{task: task, source: source, instance: instance, db: db}
	if err := r.writeBatch(&pendingBatch{upserts: payload.Upserts, deletes: payload.Deletes}); err != nil {
		repository.DB.Model(&model.SyncDeadLetter{}).Where("id = ?", id).Update("error", err.Error())
		return err
	}
	repository.DB.Delete(&model.SyncDeadLetter{}, id)
	return nil
}

// DeleteDeadLetter 删除一条死信
func (m *Manager) DeleteDeadLetter(id uint) {
	repository.DB.Delete(&model.SyncDeadLetter{}, id)
}

func loadConfig(taskID uint) (model.SyncTask, model.SyncSource, model.MeiliInstance, error) {
	var task model.SyncTask
	if err := repository.DB.First(&task, taskID).Error; err != nil {
		return task, model.SyncSource{}, model.MeiliInstance{}, fmt.Errorf("同步任务不存在")
	}
	var source model.SyncSource
	if err := repository.DB.First(&source, task.SourceID).Error; err != nil {
		return task, source, model.MeiliInstance{}, fmt.Errorf("MySQL 数据源不存在")
	}
	var instance model.MeiliInstance
	if err := repository.DB.First(&instance, task.InstanceID).Error; err != nil {
		return task, source, instance, fmt.Errorf("目标 Meilisearch 实例不存在")
	}
	return task, source, instance, nil
}

// Runner 单个同步任务的运行实例
type Runner struct {
	task     model.SyncTask
	source   model.SyncSource
	instance model.MeiliInstance

	db    *sql.DB
	canal *canal.Canal

	mu        sync.Mutex
	upsertIDs []string
	deleteIDs []string
	lastGTID  string

	// 待写入队列（失败重试用），按顺序处理
	pending       []*pendingBatch
	committedGTID string

	stopCh chan struct{}
	done   chan struct{}
}

// pendingBatch 一批待写入 Meilisearch 的变更
type pendingBatch struct {
	upserts  []string
	deletes  []string
	gtid     string // 该批次对应的 binlog 位点
	attempts int
	nextAt   time.Time
}

const maxFlushAttempts = 8

func backoffDelay(attempts int) time.Duration {
	d := time.Second * time.Duration(1<<uint(attempts-1)) // 1s,2s,4s,...
	if d > 30*time.Second {
		d = 30 * time.Second
	}
	return d
}

func (r *Runner) start() error {
	db, err := openMySQL(r.source)
	if err != nil {
		return fmt.Errorf("连接 MySQL 失败: %w", err)
	}
	r.db = db

	var format string
	if err := db.QueryRow("SELECT @@binlog_format").Scan(&format); err != nil {
		db.Close()
		return fmt.Errorf("读取 binlog_format 失败: %w", err)
	}
	if !strings.EqualFold(format, "ROW") {
		db.Close()
		return fmt.Errorf("MySQL binlog_format=%s，必须为 ROW", format)
	}

	startGTID := r.task.Position
	if startGTID == "" {
		g, err := masterGTID(db)
		if err != nil {
			db.Close()
			return fmt.Errorf("读取主库 GTID 失败: %w", err)
		}
		startGTID = g
	}

	cfg := canal.NewDefaultConfig()
	cfg.Addr = net.JoinHostPort(r.source.Host, strconv.Itoa(r.source.Port))
	cfg.User = r.source.User
	cfg.Password = r.source.Password
	cfg.Flavor = "mysql"
	cfg.ServerID = uint32(3000000 + r.task.ID) // 需与其它副本不同
	cfg.Dump = canal.DumpConfig{}

	c, err := canal.NewCanal(cfg)
	if err != nil {
		db.Close()
		return fmt.Errorf("初始化 binlog 客户端失败: %w", err)
	}
	r.canal = c
	c.SetEventHandler(&handler{r: r})
	if startGTID != "" {
		if set, err := mysql.ParseGTIDSet("mysql", startGTID); err == nil && set != nil {
			_ = c.StartFromGTID(set)
		}
	}

	r.stopCh = make(chan struct{})
	r.done = make(chan struct{})
	go r.flushLoop()
	go func() {
		defer close(r.done)
		if err := c.Run(); err != nil {
			r.recordError(fmt.Errorf("binlog 监听结束: %w", err))
		}
	}()
	return nil
}

func (r *Runner) stop() {
	select {
	case <-r.stopCh:
	default:
		close(r.stopCh)
	}
	if r.canal != nil {
		r.canal.Close()
	}
	select {
	case <-r.done:
	case <-time.After(5 * time.Second):
	}
	if r.db != nil {
		r.db.Close()
	}
}

func (r *Runner) recordError(err error) {
	if err == nil {
		return
	}
	repository.DB.Model(&model.SyncTask{}).Where("id = ?", r.task.ID).
		Updates(map[string]any{"last_error": err.Error()})
}

// ---- 缓冲与写入 ----

func (r *Runner) addUpsert(id string) {
	if id == "" {
		return
	}
	r.mu.Lock()
	r.upsertIDs = append(r.upsertIDs, id)
	r.mu.Unlock()
}

func (r *Runner) addDelete(id string) {
	if id == "" {
		return
	}
	r.mu.Lock()
	r.deleteIDs = append(r.deleteIDs, id)
	r.mu.Unlock()
}

func (r *Runner) setGTID(g string) {
	r.mu.Lock()
	r.lastGTID = g
	r.mu.Unlock()
}

func (r *Runner) flushLoop() {
	interval := time.Duration(r.task.FlushIntervalMs) * time.Millisecond
	if interval <= 0 {
		interval = 300 * time.Millisecond
	}
	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for {
		select {
		case <-r.stopCh:
			_ = r.flush()
			return
		case <-ticker.C:
			_ = r.flush()
		}
	}
}

func (r *Runner) flush() error {
	r.mu.Lock()
	upserts := r.upsertIDs
	deletes := r.deleteIDs
	gtid := r.lastGTID
	r.upsertIDs = nil
	r.deleteIDs = nil
	if len(upserts) > 0 || len(deletes) > 0 {
		r.pending = append(r.pending, &pendingBatch{
			upserts: unique(upserts),
			deletes: unique(deletes),
			gtid:    gtid,
		})
	}
	r.mu.Unlock()

	return r.processQueue()
}

// processQueue 按顺序写入队列：成功则推进位点；失败按退避重试；
// 可重试错误(网络/5xx)无限重试不丢数据，永久错误(4xx)重试上限后进死信。
func (r *Runner) processQueue() error {
	for {
		r.mu.Lock()
		if len(r.pending) == 0 {
			r.mu.Unlock()
			return nil
		}
		b := r.pending[0]
		if time.Now().Before(b.nextAt) {
			r.mu.Unlock()
			return nil // 等待退避时间
		}
		r.mu.Unlock()

		err := r.writeBatch(b)
		if err == nil {
			r.markCommitted(b)
			continue
		}

		b.attempts++
		if !isTransient(err) && b.attempts >= maxFlushAttempts {
			r.deadLetter(b, err)
			r.markCommitted(b)
			continue
		}
		b.nextAt = time.Now().Add(backoffDelay(b.attempts))
		r.recordError(fmt.Errorf("写入失败(第 %d 次，%s)：%w", b.attempts, backoffDelay(b.attempts), err))
		return err
	}
}

func (r *Runner) writeBatch(b *pendingBatch) error {
	if len(b.deletes) > 0 {
		if err := r.meiliDelete(b.deletes); err != nil {
			return err
		}
	}
	if len(b.upserts) > 0 {
		if err := r.upsertByIDs(b.upserts); err != nil {
			return err
		}
	}
	return nil
}

// markCommitted 批次写入成功（或已进死信）后，推进已提交位点并持久化
func (r *Runner) markCommitted(b *pendingBatch) {
	r.mu.Lock()
	if len(r.pending) > 0 && r.pending[0] == b {
		r.pending = r.pending[1:]
	}
	if b.gtid != "" {
		r.committedGTID = b.gtid
	}
	gtid := r.committedGTID
	r.mu.Unlock()

	if gtid != "" {
		r.task.Position = gtid
	}
	now := time.Now()
	repository.DB.Model(&model.SyncTask{}).Where("id = ?", r.task.ID).
		Updates(map[string]any{"position": gtid, "last_sync_at": &now, "last_error": ""})
}

func (r *Runner) deadLetter(b *pendingBatch, err error) {
	payload, _ := json.Marshal(map[string]any{"upserts": b.upserts, "deletes": b.deletes})
	repository.DB.Create(&model.SyncDeadLetter{
		TaskID:  r.task.ID,
		Payload: string(payload),
		Error:   err.Error(),
	})
	r.recordError(fmt.Errorf("批次重试 %d 次仍失败，已进死信: %w", maxFlushAttempts, err))
}

func (r *Runner) upsertByIDs(ids []string) error {
	docs, err := r.queryDocs(ids)
	if err != nil {
		return fmt.Errorf("联表查询失败: %w", err)
	}
	if len(docs) == 0 {
		return nil
	}

	// 软删除规则：命中则删除文档，否则 upsert
	var upserts []map[string]any
	var delIDs []string
	for _, d := range docs {
		if r.softDeleted(d) {
			if v, ok := d[r.pkField()]; ok {
				delIDs = append(delIDs, valueToString(v))
			}
			continue
		}
		upserts = append(upserts, d)
	}

	if len(delIDs) > 0 {
		if err := r.meiliDelete(unique(delIDs)); err != nil {
			return err
		}
	}
	if len(upserts) > 0 {
		return r.meiliPost("/indexes/"+r.task.IndexUID+"/documents", upserts)
	}
	return nil
}

func (r *Runner) pkField() string {
	if r.task.PrimaryKey != "" {
		return r.task.PrimaryKey
	}
	return "id"
}

func (r *Runner) softDeleted(doc map[string]any) bool {
	if r.task.SoftDeleteField == "" {
		return false
	}
	v, ok := doc[r.task.SoftDeleteField]
	if !ok {
		return false
	}
	return valueToString(v) == r.task.SoftDeleteValue
}

func (r *Runner) queryDocs(ids []string) ([]map[string]any, error) {
	if !strings.Contains(r.task.JoinSQL, ":ids") {
		return nil, fmt.Errorf("JoinSQL 缺少 :ids 占位符")
	}
	placeholders := strings.TrimSuffix(strings.Repeat("?,", len(ids)), ",")
	query := strings.Replace(r.task.JoinSQL, ":ids", placeholders, 1)
	args := make([]any, len(ids))
	for i, id := range ids {
		args[i] = id
	}

	rows, err := r.db.Query(query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	cols, err := rows.Columns()
	if err != nil {
		return nil, err
	}
	var out []map[string]any
	for rows.Next() {
		vals := make([]any, len(cols))
		ptrs := make([]any, len(cols))
		for i := range vals {
			ptrs[i] = &vals[i]
		}
		if err := rows.Scan(ptrs...); err != nil {
			return nil, err
		}
		m := make(map[string]any, len(cols))
		for i, c := range cols {
			m[c] = normalizeSQLValue(vals[i])
		}
		out = append(out, m)
	}
	return out, rows.Err()
}

func (r *Runner) meiliDelete(ids []string) error {
	arr := make([]any, len(ids))
	for i, id := range ids {
		arr[i] = idValue(id)
	}
	return r.meiliPost("/indexes/"+r.task.IndexUID+"/documents/delete-batch", arr)
}

func (r *Runner) meiliPost(path string, payload any) error {
	body, err := json.Marshal(payload)
	if err != nil {
		return err
	}
	url := strings.TrimRight(r.instance.Host, "/") + path
	req, err := http.NewRequest(http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+r.instance.APIKey)
	req.Header.Set("X-Meili-API-Key", r.instance.APIKey)

	client := &http.Client{Timeout: 30 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		// 网络层错误 -> 可重试
		return &meiliRequestError{status: 0, message: err.Error(), transient: true}
	}
	defer resp.Body.Close()
	if resp.StatusCode >= 300 {
		b, _ := io.ReadAll(resp.Body)
		return &meiliRequestError{
			status:    resp.StatusCode,
			message:   strings.TrimSpace(string(b)),
			transient: resp.StatusCode >= 500 || resp.StatusCode == 429, // 5xx/限流可重试
		}
	}
	return nil
}

type meiliRequestError struct {
	status    int
	message   string
	transient bool
}

func (e *meiliRequestError) Error() string {
	return fmt.Sprintf("Meilisearch 返回 %d: %s", e.status, e.message)
}

// isTransient 判断错误是否可重试（网络/5xx 视为可重试并无限重试，避免丢数据）
func isTransient(err error) bool {
	var me *meiliRequestError
	if errors.As(err, &me) {
		return me.transient
	}
	return true
}

// FullSync 全量初始化：先记录 GTID 起点，再按主键分页把整表（经联表）灌入 Meilisearch
func (m *Manager) FullSync(taskID uint) (int, error) {
	task, source, instance, err := loadConfig(taskID)
	if err != nil {
		return 0, err
	}
	if m.IsRunning(taskID) {
		return 0, fmt.Errorf("请先停止任务再执行全量初始化")
	}

	db, err := openMySQL(source)
	if err != nil {
		return 0, fmt.Errorf("连接 MySQL 失败: %w", err)
	}
	defer db.Close()

	g0, err := masterGTID(db)
	if err != nil {
		return 0, fmt.Errorf("读取主库 GTID 失败: %w", err)
	}

	r := &Runner{task: task, source: source, instance: instance, db: db}
	batch := task.BatchSize
	if batch <= 0 {
		batch = 500
	}
	tbl := "`" + task.DBName + "`.`" + task.TableName + "`"
	pk := task.PrimaryKey
	if pk == "" {
		pk = "id"
	}

	var after any
	count := 0
	for {
		q := "SELECT `" + pk + "` FROM " + tbl
		args := []any{}
		if after != nil {
			q += " WHERE `" + pk + "` > ?"
			args = append(args, after)
		}
		q += " ORDER BY `" + pk + "` ASC LIMIT " + strconv.Itoa(batch)

		rows, err := db.Query(q, args...)
		if err != nil {
			return count, fmt.Errorf("读取主表失败: %w", err)
		}
		var ids []string
		for rows.Next() {
			var v any
			if err := rows.Scan(&v); err != nil {
				rows.Close()
				return count, err
			}
			ids = append(ids, valueToString(v))
			after = v
		}
		rows.Close()
		if len(ids) == 0 {
			break
		}
		if err := r.upsertByIDs(ids); err != nil {
			return count, err
		}
		count += len(ids)
		if len(ids) < batch {
			break
		}
	}

	now := time.Now()
	repository.DB.Model(&model.SyncTask{}).Where("id = ?", task.ID).
		Updates(map[string]any{"position": g0, "last_sync_at": &now, "last_error": ""})
	return count, nil
}

// ---- handler ----

type handler struct {
	canal.DummyEventHandler
	r *Runner
}

func (h *handler) String() string { return "meili-syncer" }

func (h *handler) OnRow(e *canal.RowsEvent) error {
	if e.Table == nil {
		return nil
	}
	if e.Table.Name != h.r.task.TableName {
		return nil
	}
	if h.r.task.DBName != "" && e.Table.Schema != h.r.task.DBName {
		return nil
	}

	pk := h.r.task.PrimaryKey
	if pk == "" {
		pk = "id"
	}
	idx := -1
	for i, c := range e.Table.Columns {
		if c.Name == pk {
			idx = i
			break
		}
	}
	if idx < 0 {
		return nil
	}

	switch e.Action {
	case canal.InsertAction:
		for _, row := range e.Rows {
			h.r.addUpsert(pkString(row, idx))
		}
	case canal.DeleteAction:
		for _, row := range e.Rows {
			h.r.addDelete(pkString(row, idx))
		}
	case canal.UpdateAction:
		// [before, after, before, after, ...] 取 after
		for i := 1; i < len(e.Rows); i += 2 {
			h.r.addUpsert(pkString(e.Rows[i], idx))
		}
	}
	return nil
}

func (h *handler) OnXID(_ *replication.EventHeader, _ mysql.Position) error {
	return h.r.flush() // 事务结束，立即落库
}

func (h *handler) OnPosSynced(_ *replication.EventHeader, _ mysql.Position, set mysql.GTIDSet, _ bool) error {
	if set != nil {
		h.r.setGTID(set.String())
	}
	return nil
}

// ---- 工具函数 ----

func openMySQL(src model.SyncSource) (*sql.DB, error) {
	port := src.Port
	if port == 0 {
		port = 3306
	}
	dsn := fmt.Sprintf("%s:%s@tcp(%s)/?charset=utf8mb4&parseTime=true&loc=Local",
		src.User, src.Password, net.JoinHostPort(src.Host, strconv.Itoa(port)))
	db, err := sql.Open("mysql", dsn)
	if err != nil {
		return nil, err
	}
	db.SetConnMaxLifetime(time.Minute * 3)
	db.SetMaxOpenConns(4)
	db.SetMaxIdleConns(4)
	if err := db.Ping(); err != nil {
		db.Close()
		return nil, err
	}
	return db, nil
}

func masterGTID(db *sql.DB) (string, error) {
	rows, err := db.Query("SHOW MASTER STATUS")
	if err != nil {
		return "", err
	}
	defer rows.Close()
	cols, _ := rows.Columns()
	if !rows.Next() {
		return "", fmt.Errorf("SHOW MASTER STATUS 无结果（binlog 未开启？）")
	}
	vals := make([]any, len(cols))
	ptrs := make([]any, len(cols))
	for i := range vals {
		ptrs[i] = &vals[i]
	}
	if err := rows.Scan(ptrs...); err != nil {
		return "", err
	}
	for i, c := range cols {
		if strings.EqualFold(c, "Executed_Gtid_Set") {
			return valueToString(vals[i]), nil
		}
	}
	return "", nil
}

func pkString(row []any, idx int) string {
	if idx < 0 || idx >= len(row) {
		return ""
	}
	return valueToString(row[idx])
}

func valueToString(v any) string {
	switch t := v.(type) {
	case nil:
		return ""
	case []byte:
		return string(t)
	case string:
		return t
	default:
		return fmt.Sprintf("%v", t)
	}
}

func normalizeSQLValue(v any) any {
	switch t := v.(type) {
	case []byte:
		return string(t)
	case time.Time:
		return t.Format("2006-01-02 15:04:05")
	default:
		return v
	}
}

func idValue(id string) any {
	if n, err := strconv.ParseInt(id, 10, 64); err == nil {
		return n
	}
	return id
}

func unique(ids []string) []string {
	seen := make(map[string]struct{}, len(ids))
	out := make([]string, 0, len(ids))
	for _, id := range ids {
		if id == "" {
			continue
		}
		if _, ok := seen[id]; ok {
			continue
		}
		seen[id] = struct{}{}
		out = append(out, id)
	}
	return out
}
