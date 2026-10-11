package api

import (
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync/atomic"
	"testing"

	"backend/model"
	"backend/repository"

	"github.com/gin-gonic/gin"
)

// newFakeMeili 返回一个模拟 Meilisearch 的测试服务器：
//   - GET  /indexes/{uid}        返回 primaryKey
//   - POST /indexes/{uid}/search 返回预设 hits
func newFakeMeili(t *testing.T, hits []map[string]any) *httptest.Server {
	t.Helper()
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		switch {
		case r.Method == http.MethodGet && strings.Contains(r.URL.Path, "/indexes/"):
			_ = json.NewEncoder(w).Encode(map[string]any{"primaryKey": "id"})
		case r.Method == http.MethodPost && strings.HasSuffix(r.URL.Path, "/search"):
			_ = json.NewEncoder(w).Encode(map[string]any{
				"hits":               hits,
				"estimatedTotalHits": len(hits),
				"processingTimeMs":   3,
			})
		default:
			w.WriteHeader(http.StatusNotFound)
		}
	}))
}

func setupAggInstances(t *testing.T) (model.MeiliInstance, model.MeiliInstance, func()) {
	t.Helper()
	serverA := newFakeMeili(t, []map[string]any{{"id": "1"}, {"id": "2"}})
	serverB := newFakeMeili(t, []map[string]any{{"id": "2"}, {"id": "3"}})

	instA := model.MeiliInstance{Name: "e2e-A", Host: serverA.URL, APIKey: "k"}
	instB := model.MeiliInstance{Name: "e2e-B", Host: serverB.URL, APIKey: "k"}
	if err := repository.DB.Create(&instA).Error; err != nil {
		t.Fatal(err)
	}
	if err := repository.DB.Create(&instB).Error; err != nil {
		t.Fatal(err)
	}

	cleanup := func() {
		repository.DB.Delete(&model.MeiliInstance{}, instA.ID)
		repository.DB.Delete(&model.MeiliInstance{}, instB.ID)
		serverA.Close()
		serverB.Close()
	}
	return instA, instB, cleanup
}

func TestHandleAggregatedSearchEndToEnd(t *testing.T) {
	gin.SetMode(gin.TestMode)
	uid := "e2e_hotels_single"
	instA, instB, cleanup := setupAggInstances(t)
	defer cleanup()

	clearIndexRouteCache(uid)
	pkCacheMu.Lock()
	delete(pkCache, uid)
	pkCacheMu.Unlock()

	app := model.Application{
		Name:                 "e2e-app",
		AppKey:               "e2e-key-single",
		InstanceID:           instA.ID,
		AggregateInstanceIDs: fmt.Sprintf("[%d]", instB.ID),
	}

	w := httptest.NewRecorder()
	c, _ := gin.CreateTestContext(w)
	c.Request = httptest.NewRequest(http.MethodPost, "/api/v1/proxy/indexes/"+uid+"/search", strings.NewReader(`{"q":"","limit":10}`))
	c.Request.Header.Set("Content-Type", "application/json")

	if !handleAggregatedSearch(c, &app, "/indexes/"+uid+"/search") {
		t.Fatal("期望聚合搜索被接管")
	}
	if w.Code != http.StatusOK {
		t.Fatalf("状态码应为 200，实际 %d", w.Code)
	}

	var resp struct {
		Hits []map[string]any `json:"hits"`
	}
	if err := json.Unmarshal(w.Body.Bytes(), &resp); err != nil {
		t.Fatal(err)
	}
	if len(resp.Hits) != 3 {
		t.Fatalf("两个实例去重后应有 3 条，实际 %d", len(resp.Hits))
	}
}

// newFakeMeiliCounting 模拟 Meilisearch，并统计写入文档的调用次数
func newFakeMeiliCounting(t *testing.T, counter *int32) *httptest.Server {
	t.Helper()
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		switch {
		case r.Method == http.MethodGet && strings.Contains(r.URL.Path, "/indexes/"):
			_ = json.NewEncoder(w).Encode(map[string]any{"primaryKey": "id"})
		case r.Method == http.MethodPost && strings.HasSuffix(r.URL.Path, "/documents"):
			atomic.AddInt32(counter, 1)
			_ = json.NewEncoder(w).Encode(map[string]any{"taskUid": 1, "status": "enqueued"})
		default:
			w.WriteHeader(http.StatusNotFound)
		}
	}))
}

func TestHandleBroadcastWriteEndToEnd(t *testing.T) {
	gin.SetMode(gin.TestMode)
	uid := "bw_hotels_e2e"

	var countA, countB int32
	serverA := newFakeMeiliCounting(t, &countA)
	serverB := newFakeMeiliCounting(t, &countB)
	defer serverA.Close()
	defer serverB.Close()

	instA := model.MeiliInstance{Name: "bw-A", Host: serverA.URL, APIKey: "k"}
	instB := model.MeiliInstance{Name: "bw-B", Host: serverB.URL, APIKey: "k"}
	if err := repository.DB.Create(&instA).Error; err != nil {
		t.Fatal(err)
	}
	if err := repository.DB.Create(&instB).Error; err != nil {
		t.Fatal(err)
	}
	defer func() {
		repository.DB.Delete(&model.MeiliInstance{}, instA.ID)
		repository.DB.Delete(&model.MeiliInstance{}, instB.ID)
	}()

	app := model.Application{
		Name:                 "bw-app",
		AppKey:               "bw-key-e2e",
		InstanceID:           instA.ID,
		AggregateInstanceIDs: fmt.Sprintf("[%d]", instB.ID),
		BroadcastWrites:      true,
	}

	w := httptest.NewRecorder()
	c, _ := gin.CreateTestContext(w)
	c.Request = httptest.NewRequest(http.MethodPost, "/api/v1/proxy/indexes/"+uid+"/documents", strings.NewReader(`[{"id":"1"}]`))
	c.Request.Header.Set("Content-Type", "application/json")

	if !handleBroadcastWrite(c, &app, "/indexes/"+uid+"/documents", http.MethodPost) {
		t.Fatal("期望广播写入被接管")
	}
	if w.Code != http.StatusOK {
		t.Fatalf("状态码应为 200，实际 %d", w.Code)
	}
	if atomic.LoadInt32(&countA) != 1 || atomic.LoadInt32(&countB) != 1 {
		t.Fatalf("两个实例都应各收到一次写入，实际 A=%d B=%d", countA, countB)
	}

	// 关闭开关后不应再广播
	app.BroadcastWrites = false
	if handleBroadcastWrite(c, &app, "/indexes/"+uid+"/documents", http.MethodPost) {
		t.Fatal("开关关闭时不应广播")
	}
}

func TestHandleAggregatedMultiSearchEndToEnd(t *testing.T) {
	gin.SetMode(gin.TestMode)
	uid := "e2e_hotels_multi"
	instA, instB, cleanup := setupAggInstances(t)
	defer cleanup()

	pkCacheMu.Lock()
	delete(pkCache, uid)
	pkCacheMu.Unlock()

	app := model.Application{
		Name:                 "e2e-app-multi",
		AppKey:               "e2e-key-multi",
		InstanceID:           instA.ID,
		AggregateInstanceIDs: fmt.Sprintf("[%d]", instB.ID),
	}

	body := fmt.Sprintf(`{"queries":[{"indexUid":"%s","q":"","limit":10}]}`, uid)
	w := httptest.NewRecorder()
	c, _ := gin.CreateTestContext(w)
	c.Request = httptest.NewRequest(http.MethodPost, "/api/v1/proxy/multi-search", strings.NewReader(body))
	c.Request.Header.Set("Content-Type", "application/json")

	if !handleAggregatedMultiSearch(c, &app, "/multi-search") {
		t.Fatal("期望聚合 multi-search 被接管")
	}
	if w.Code != http.StatusOK {
		t.Fatalf("状态码应为 200，实际 %d", w.Code)
	}

	var resp struct {
		Results []struct {
			IndexUID string           `json:"indexUid"`
			Hits     []map[string]any `json:"hits"`
		} `json:"results"`
	}
	if err := json.Unmarshal(w.Body.Bytes(), &resp); err != nil {
		t.Fatal(err)
	}
	if len(resp.Results) != 1 {
		t.Fatalf("应返回 1 个子查询结果，实际 %d", len(resp.Results))
	}
	if resp.Results[0].IndexUID != uid {
		t.Errorf("indexUid 应透传，实际 %q", resp.Results[0].IndexUID)
	}
	if len(resp.Results[0].Hits) != 3 {
		t.Fatalf("子查询去重后应有 3 条，实际 %d", len(resp.Results[0].Hits))
	}
}
