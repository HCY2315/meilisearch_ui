package syncer

import (
	"net/http"
	"net/http/httptest"
	"os"
	"sync/atomic"
	"testing"
	"time"

	"backend/model"
	"backend/repository"

	"github.com/glebarez/sqlite"
	"gorm.io/gorm"
)

func TestMain(m *testing.M) {
	var err error
	repository.DB, err = gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	if err != nil {
		panic(err)
	}
	_ = repository.DB.AutoMigrate(&model.SyncTask{}, &model.SyncDeadLetter{}, &model.MeiliInstance{})
	os.Exit(m.Run())
}

// 临时性错误(5xx)只重试、不进死信、不丢批次
func TestProcessQueueTransientRetry(t *testing.T) {
	var calls int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if atomic.AddInt32(&calls, 1) <= 2 {
			w.WriteHeader(http.StatusInternalServerError) // 前两次失败
			return
		}
		w.WriteHeader(http.StatusAccepted)
	}))
	defer srv.Close()

	r := &Runner{
		instance: model.MeiliInstance{Host: srv.URL, APIKey: "k"},
		task:     model.SyncTask{ID: 9001, IndexUID: "x"},
	}
	r.pending = []*pendingBatch{{deletes: []string{"1"}, gtid: "g1"}}

	// 第 1 次失败
	_ = r.processQueue()
	if len(r.pending) != 1 {
		t.Fatalf("失败后批次应仍在队列，实际 %d", len(r.pending))
	}
	// 第 2 次失败
	r.pending[0].nextAt = time.Now().Add(-time.Second)
	_ = r.processQueue()
	if len(r.pending) != 1 {
		t.Fatalf("第2次失败后批次应仍在队列")
	}
	// 第 3 次成功
	r.pending[0].nextAt = time.Now().Add(-time.Second)
	if err := r.processQueue(); err != nil {
		t.Fatalf("第3次应成功，实际 %v", err)
	}
	if len(r.pending) != 0 {
		t.Fatalf("成功后队列应清空")
	}
	if r.committedGTID != "g1" {
		t.Fatalf("成功后位点应推进为 g1，实际 %q", r.committedGTID)
	}
}

// 永久性错误(4xx)重试上限后进死信
func TestProcessQueuePermanentDeadLetter(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusBadRequest) // 永久失败
	}))
	defer srv.Close()

	repository.DB.Where("task_id = ?", 9002).Delete(&model.SyncDeadLetter{})

	r := &Runner{
		instance: model.MeiliInstance{Host: srv.URL, APIKey: "k"},
		task:     model.SyncTask{ID: 9002, IndexUID: "x"},
	}
	r.pending = []*pendingBatch{{deletes: []string{"1"}, gtid: "g2"}}

	for i := 0; i < maxFlushAttempts+2 && len(r.pending) > 0; i++ {
		r.pending[0].nextAt = time.Now().Add(-time.Second)
		_ = r.processQueue()
	}

	if len(r.pending) != 0 {
		t.Fatalf("超过重试上限后应移出队列，实际 %d", len(r.pending))
	}
	var cnt int64
	repository.DB.Model(&model.SyncDeadLetter{}).Where("task_id = ?", 9002).Count(&cnt)
	if cnt != 1 {
		t.Fatalf("应写入 1 条死信，实际 %d", cnt)
	}
}

func TestSoftDeleted(t *testing.T) {
	r := &Runner{task: model.SyncTask{PrimaryKey: "id", SoftDeleteField: "status", SoftDeleteValue: "2"}}
	if !r.softDeleted(map[string]any{"id": int64(1), "status": int64(2)}) {
		t.Error("status=2 应判定为软删除")
	}
	if r.softDeleted(map[string]any{"id": int64(1), "status": int64(1)}) {
		t.Error("status=1 不应判定为软删除")
	}
	if r.softDeleted(map[string]any{"id": int64(1)}) {
		t.Error("缺字段不应判定为软删除")
	}
	r2 := &Runner{task: model.SyncTask{}}
	if r2.softDeleted(map[string]any{"status": int64(2)}) {
		t.Error("未配置规则不应判定为软删除")
	}
}
