package api

import (
	"fmt"
	"testing"

	"backend/model"
	"backend/repository"
)

func idOf(hit map[string]any) string {
	return fmt.Sprintf("%v", hit["id"])
}

func TestMergeSearchResponsesDedupeRoundRobin(t *testing.T) {
	r1 := map[string]any{
		"hits": []any{
			map[string]any{"id": "1", "name": "A"},
			map[string]any{"id": "2", "name": "B"},
		},
		"estimatedTotalHits": float64(2),
		"processingTimeMs":   float64(5),
		"facetDistribution":  map[string]any{"city": map[string]any{"北京": float64(2)}},
	}
	r2 := map[string]any{
		"hits": []any{
			map[string]any{"id": "2", "name": "B"}, // 与 r1 重复，应被去除
			map[string]any{"id": "3", "name": "C"},
		},
		"estimatedTotalHits": float64(2),
		"processingTimeMs":   float64(9),
		"facetDistribution":  map[string]any{"city": map[string]any{"北京": float64(1), "上海": float64(1)}},
	}

	merged := mergeSearchResponses([]map[string]any{r1, r2}, 0, 10, "id")
	hits := merged["hits"].([]map[string]any)
	if len(hits) != 3 {
		t.Fatalf("去重后应返回 3 条，实际 %d", len(hits))
	}
	// 轮询交错：先各取第 1 条 (1, 3)，再取 r1 的第 2 条 (2)
	if idOf(hits[0]) != "1" || idOf(hits[1]) != "3" || idOf(hits[2]) != "2" {
		t.Fatalf("轮询顺序错误: %v %v %v", idOf(hits[0]), idOf(hits[1]), idOf(hits[2]))
	}
	if merged["estimatedTotalHits"].(int) != 4 {
		t.Errorf("estimatedTotalHits 应为各实例之和 4，实际 %v", merged["estimatedTotalHits"])
	}
	if merged["processingTimeMs"].(int) != 9 {
		t.Errorf("processingTimeMs 应取最大值 9，实际 %v", merged["processingTimeMs"])
	}
	fd, ok := merged["facetDistribution"].(map[string]map[string]int)
	if !ok {
		t.Fatalf("facetDistribution 类型错误: %T", merged["facetDistribution"])
	}
	if fd["city"]["北京"] != 3 || fd["city"]["上海"] != 1 {
		t.Errorf("facet 合并错误: %+v", fd)
	}
}

func TestMergeSearchResponsesScoreOrdering(t *testing.T) {
	r1 := map[string]any{
		"hits": []any{
			map[string]any{"id": "1", "_rankingScore": float64(0.5)},
			map[string]any{"id": "2", "_rankingScore": float64(0.9)},
		},
		"estimatedTotalHits": float64(2),
	}
	r2 := map[string]any{
		"hits": []any{
			map[string]any{"id": "3", "_rankingScore": float64(0.7)},
		},
		"estimatedTotalHits": float64(1),
	}

	merged := mergeSearchResponses([]map[string]any{r1, r2}, 0, 10, "id")
	hits := merged["hits"].([]map[string]any)
	if len(hits) != 3 {
		t.Fatalf("应返回 3 条，实际 %d", len(hits))
	}
	// 全部带分数的场景应按分数降序
	if idOf(hits[0]) != "2" || idOf(hits[1]) != "3" || idOf(hits[2]) != "1" {
		t.Fatalf("分数排序错误: %v %v %v", idOf(hits[0]), idOf(hits[1]), idOf(hits[2]))
	}
}

func TestMergeSearchResponsesPagination(t *testing.T) {
	r1 := map[string]any{
		"hits": []any{
			map[string]any{"id": "1"},
			map[string]any{"id": "2"},
			map[string]any{"id": "3"},
		},
		"estimatedTotalHits": float64(3),
	}
	merged := mergeSearchResponses([]map[string]any{r1}, 1, 1, "id")
	hits := merged["hits"].([]map[string]any)
	if len(hits) != 1 || idOf(hits[0]) != "2" {
		t.Fatalf("分页切片错误: %+v", hits)
	}
	if merged["offset"].(int) != 1 || merged["limit"].(int) != 1 {
		t.Errorf("offset/limit 回写错误: %v/%v", merged["offset"], merged["limit"])
	}
}

func TestDedupeKey(t *testing.T) {
	if got := dedupeKey(map[string]any{"hotel_id": "H1"}, "hotel_id"); got != "H1" {
		t.Errorf("应使用主键去重，got %q", got)
	}
	if got := dedupeKey(map[string]any{"id": 42}, "hotel_id"); got != "42" {
		t.Errorf("主键缺失时应回退到 id，got %q", got)
	}
	if got := dedupeKey(map[string]any{"name": "x"}, "hotel_id"); got != "" {
		t.Errorf("无可用去重键时应返回空串，got %q", got)
	}
}

func TestResolveInstancesForApp(t *testing.T) {
	instA := model.MeiliInstance{Name: "A", Host: "http://a:7700", APIKey: "k"}
	instB := model.MeiliInstance{Name: "B", Host: "http://b:7700", APIKey: "k"}
	repository.DB.Create(&instA)
	repository.DB.Create(&instB)
	defer func() {
		repository.DB.Delete(&model.MeiliInstance{}, instA.ID)
		repository.DB.Delete(&model.MeiliInstance{}, instB.ID)
	}()

	app := model.Application{
		Name:                 "agg-app",
		AppKey:               "agg-app-key-test",
		InstanceID:           instA.ID,
		AggregateInstanceIDs: fmt.Sprintf("[%d,%d]", instB.ID, instA.ID), // 含重复主实例
	}

	got := resolveInstancesForApp(&app)
	if len(got) != 2 {
		t.Fatalf("应解析出 2 个实例（去重），实际 %d", len(got))
	}
	if got[0].ID != instA.ID || got[1].ID != instB.ID {
		t.Fatalf("实例顺序/内容错误: %d, %d", got[0].ID, got[1].ID)
	}

	// 未配置聚合时只返回主实例
	app2 := model.Application{Name: "single", AppKey: "single-key-test", InstanceID: instA.ID}
	if got2 := resolveInstancesForApp(&app2); len(got2) != 1 || got2[0].ID != instA.ID {
		t.Fatalf("单实例解析错误: %+v", got2)
	}
}
