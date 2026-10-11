package api

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"

	"backend/model"
	"backend/repository"

	"github.com/gin-gonic/gin"
)

// 索引主键缓存：uid -> primaryKey，避免每次聚合搜索都去探测主键
var (
	pkCacheMu sync.RWMutex
	pkCache   = map[string]string{}
)

// 索引归属缓存：uid -> 拥有该索引的实例ID，用于把非搜索请求路由到正确实例
var (
	routeCacheMu sync.RWMutex
	routeCache   = map[string]indexRouteEntry{}
)

const routeCacheTTL = 60 * time.Second

type indexRouteEntry struct {
	instanceID uint
	expiresAt  time.Time
}

// allInstances 返回全部 Meilisearch 实例
func allInstances() []model.MeiliInstance {
	var list []model.MeiliInstance
	repository.DB.Find(&list)
	return list
}

// indexExistsOn 判断某实例上是否存在指定索引
func indexExistsOn(inst model.MeiliInstance, uid string) bool {
	target, err := url.JoinPath(inst.Host, "indexes", uid)
	if err != nil {
		return false
	}
	req, err := http.NewRequest(http.MethodGet, target, nil)
	if err != nil {
		return false
	}
	req.Header.Set("Authorization", "Bearer "+inst.APIKey)
	req.Header.Set("X-Meili-API-Key", inst.APIKey)
	client := &http.Client{Timeout: 8 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return false
	}
	defer resp.Body.Close()
	io.Copy(io.Discard, resp.Body)
	return resp.StatusCode >= 200 && resp.StatusCode < 300
}

// clearIndexRouteCache 清理某索引的归属缓存（索引创建/删除后调用）
func clearIndexRouteCache(uid string) {
	if uid == "" {
		return
	}
	routeCacheMu.Lock()
	delete(routeCache, uid)
	routeCacheMu.Unlock()
}

// resolveInstanceForIndex 在候选实例中找出实际拥有该索引的实例（带 60s 缓存）
func resolveInstanceForIndex(candidates []model.MeiliInstance, uid string) (model.MeiliInstance, bool) {
	if uid == "" || len(candidates) == 0 {
		return model.MeiliInstance{}, false
	}

	routeCacheMu.RLock()
	entry, ok := routeCache[uid]
	routeCacheMu.RUnlock()
	if ok && time.Now().Before(entry.expiresAt) {
		for _, inst := range candidates {
			if inst.ID == entry.instanceID {
				return inst, true
			}
		}
	}

	for _, inst := range candidates {
		if indexExistsOn(inst, uid) {
			routeCacheMu.Lock()
			routeCache[uid] = indexRouteEntry{instanceID: inst.ID, expiresAt: time.Now().Add(routeCacheTTL)}
			routeCacheMu.Unlock()
			return inst, true
		}
	}
	return model.MeiliInstance{}, false
}

// resolveInstancesForApp 解析某个 Application 参与搜索的实例列表
// NOTE: 若配置了 AggregateInstanceIDs，则把主实例与其一起扇出合并；否则只返回主实例
func resolveInstancesForApp(app *model.Application) []model.MeiliInstance {
	if app == nil {
		var inst model.MeiliInstance
		if err := repository.DB.First(&inst).Error; err == nil {
			return []model.MeiliInstance{inst}
		}
		return nil
	}

	ids := []uint{app.InstanceID}
	if app.AggregateInstanceIDs != "" {
		var extra []uint
		if err := json.Unmarshal([]byte(app.AggregateInstanceIDs), &extra); err == nil {
			ids = append(ids, extra...)
		}
	}

	seen := map[uint]bool{}
	var result []model.MeiliInstance
	for _, id := range ids {
		if id == 0 || seen[id] {
			continue
		}
		seen[id] = true
		var inst model.MeiliInstance
		if err := repository.DB.First(&inst, id).Error; err == nil {
			result = append(result, inst)
		}
	}
	return result
}

// handleAggregatedSearch 把一次搜索请求扇出到多个实例并合并结果
// 返回 true 表示已接管并写回响应
func handleAggregatedSearch(c *gin.Context, app *model.Application, proxyPath string) bool {
	instances := resolveInstancesForApp(app)
	if len(instances) <= 1 {
		return false
	}

	uid := extractIndexUIDFromProxyPath(proxyPath)
	if uid == "" {
		return false
	}

	bodyBytes, err := io.ReadAll(c.Request.Body)
	if err != nil {
		return false
	}
	c.Request.Body = io.NopCloser(bytes.NewReader(bodyBytes))

	var params map[string]any
	if err := json.Unmarshal(bodyBytes, &params); err != nil {
		return false
	}

	limit := jsonInt(params["limit"], 20)
	offset := jsonInt(params["offset"], 0)
	if limit <= 0 {
		limit = 20
	}
	// 每个实例都取 offset+limit 条，合并去重后再做分页切片
	params["limit"] = offset + limit
	params["offset"] = 0

	type fanoutResult struct {
		resp map[string]any
		err  error
	}
	ch := make(chan fanoutResult, len(instances))
	for _, inst := range instances {
		go func(inst model.MeiliInstance) {
			resp, err := doInstanceSearch(inst, proxyPath, params)
			ch <- fanoutResult{resp: resp, err: err}
		}(inst)
	}

	var responses []map[string]any
	for range instances {
		r := <-ch
		if r.err != nil {
			fmt.Printf("[AggregateSearch] instance query failed: %v\n", r.err)
			continue
		}
		if r.resp != nil {
			responses = append(responses, r.resp)
		}
	}

	if len(responses) == 0 {
		c.JSON(http.StatusBadGateway, gin.H{"error": "所有聚合实例查询失败"})
		return true
	}

	pk := getIndexPrimaryKey(instances, uid)
	merged := mergeSearchResponses(responses, offset, limit, pk)
	c.JSON(http.StatusOK, merged)
	return true
}

// handleAggregatedMultiSearch 把 multi-search 的每个子查询分别扇出到多个实例并合并
// 返回 true 表示已接管并写回响应
func handleAggregatedMultiSearch(c *gin.Context, app *model.Application, proxyPath string) bool {
	instances := resolveInstancesForApp(app)
	if len(instances) <= 1 {
		return false
	}

	bodyBytes, err := io.ReadAll(c.Request.Body)
	if err != nil {
		return false
	}
	c.Request.Body = io.NopCloser(bytes.NewReader(bodyBytes))

	var payload struct {
		Queries []map[string]any `json:"queries"`
	}
	if err := json.Unmarshal(bodyBytes, &payload); err != nil || len(payload.Queries) == 0 {
		return false
	}

	results := make([]gin.H, len(payload.Queries))
	var wg sync.WaitGroup
	for i, q := range payload.Queries {
		wg.Add(1)
		go func(i int, q map[string]any) {
			defer wg.Done()
			results[i] = aggregateOneQuery(instances, q)
		}(i, q)
	}
	wg.Wait()

	c.JSON(http.StatusOK, gin.H{"results": results})
	return true
}

// aggregateOneQuery 对单个子查询执行多实例扇出合并
func aggregateOneQuery(instances []model.MeiliInstance, query map[string]any) gin.H {
	uid, _ := query["indexUid"].(string)
	result := gin.H{"indexUid": uid}
	if uid == "" {
		return result
	}

	limit := jsonInt(query["limit"], 20)
	offset := jsonInt(query["offset"], 0)
	if limit <= 0 {
		limit = 20
	}

	params := make(map[string]any, len(query)+2)
	for k, v := range query {
		if k == "indexUid" {
			continue
		}
		params[k] = v
	}
	params["limit"] = offset + limit
	params["offset"] = 0

	path := "/indexes/" + url.PathEscape(uid) + "/search"

	type fanoutResult struct {
		resp map[string]any
		err  error
	}
	ch := make(chan fanoutResult, len(instances))
	for _, inst := range instances {
		go func(inst model.MeiliInstance) {
			resp, err := doInstanceSearch(inst, path, params)
			ch <- fanoutResult{resp: resp, err: err}
		}(inst)
	}

	var responses []map[string]any
	for range instances {
		r := <-ch
		if r.err == nil && r.resp != nil {
			responses = append(responses, r.resp)
		}
	}
	if len(responses) == 0 {
		return result
	}

	pk := getIndexPrimaryKey(instances, uid)
	merged := mergeSearchResponses(responses, offset, limit, pk)
	for k, v := range merged {
		result[k] = v
	}
	return result
}

// resolveRequestTargetInstance 解析请求头 X-Target-Instance，手动指定目标实例
// 仅允许该应用有权访问的实例，防止越权跨租户
func resolveRequestTargetInstance(c *gin.Context, app *model.Application) (model.MeiliInstance, bool) {
	raw := strings.TrimSpace(c.GetHeader("X-Target-Instance"))
	if raw == "" {
		return model.MeiliInstance{}, false
	}
	id, err := strconv.ParseUint(raw, 10, 64)
	if err != nil || id == 0 {
		return model.MeiliInstance{}, false
	}
	for _, inst := range resolveInstancesForApp(app) {
		if inst.ID == uint(id) {
			return inst, true
		}
	}
	return model.MeiliInstance{}, false
}

// isWriteMethod 判断是否为写操作
func isWriteMethod(method string) bool {
	switch method {
	case http.MethodPost, http.MethodPut, http.MethodPatch, http.MethodDelete:
		return true
	}
	return false
}

// resolveAllInstanceOwners 并发探测哪些候选实例拥有该索引
func resolveAllInstanceOwners(candidates []model.MeiliInstance, uid string) []model.MeiliInstance {
	type probe struct {
		inst   model.MeiliInstance
		exists bool
	}
	ch := make(chan probe, len(candidates))
	for _, inst := range candidates {
		go func(inst model.MeiliInstance) {
			ch <- probe{inst: inst, exists: indexExistsOn(inst, uid)}
		}(inst)
	}
	var owners []model.MeiliInstance
	for range candidates {
		p := <-ch
		if p.exists {
			owners = append(owners, p.inst)
		}
	}
	return owners
}

// doInstanceRawRequest 向单个实例发起任意方法请求，返回状态码与响应体
func doInstanceRawRequest(inst model.MeiliInstance, method, proxyPath, rawQuery string, body []byte, contentType string) (int, []byte, error) {
	base := strings.TrimRight(inst.Host, "/")
	target := base + "/" + strings.TrimPrefix(proxyPath, "/")
	if rawQuery != "" {
		target += "?" + rawQuery
	}

	var reader io.Reader
	if body != nil {
		reader = bytes.NewReader(body)
	}
	req, err := http.NewRequest(method, target, reader)
	if err != nil {
		return 0, nil, err
	}
	if contentType != "" {
		req.Header.Set("Content-Type", contentType)
	}
	req.Header.Set("Authorization", "Bearer "+inst.APIKey)
	req.Header.Set("X-Meili-API-Key", inst.APIKey)

	client := &http.Client{Timeout: 30 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return 0, nil, err
	}
	defer resp.Body.Close()
	data, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, data, nil
}

// handleBroadcastWrite 把写操作广播到所有拥有该索引的实例
// 返回 true 表示已接管并写回响应
func handleBroadcastWrite(c *gin.Context, app *model.Application, proxyPath, method string) bool {
	if app == nil || !app.BroadcastWrites || !isWriteMethod(method) {
		return false
	}
	uid := extractIndexUIDFromProxyPath(proxyPath)
	if uid == "" {
		return false
	}

	instances := resolveInstancesForApp(app)
	if len(instances) <= 1 {
		return false
	}

	owners := resolveAllInstanceOwners(instances, uid)
	if len(owners) <= 1 {
		// 只有单个实例拥有该索引，交回常规单实例路由处理
		return false
	}

	bodyBytes, err := io.ReadAll(c.Request.Body)
	if err != nil {
		return false
	}
	c.Request.Body = io.NopCloser(bytes.NewReader(bodyBytes))
	contentType := c.Request.Header.Get("Content-Type")
	rawQuery := c.Request.URL.RawQuery

	type writeResult struct {
		status int
		body   []byte
		err    error
	}
	ch := make(chan writeResult, len(owners))
	for _, inst := range owners {
		go func(inst model.MeiliInstance) {
			status, data, err := doInstanceRawRequest(inst, method, proxyPath, rawQuery, bodyBytes, contentType)
			ch <- writeResult{status: status, body: data, err: err}
		}(inst)
	}

	var firstOK *writeResult
	var lastErr *writeResult
	for range owners {
		r := <-ch
		if r.err == nil && r.status >= 200 && r.status < 300 {
			if firstOK == nil {
				rr := r
				firstOK = &rr
			}
			continue
		}
		rr := r
		lastErr = &rr
	}

	if method == http.MethodDelete {
		clearIndexRouteCache(uid)
	}

	if firstOK != nil {
		c.Data(firstOK.status, "application/json; charset=utf-8", firstOK.body)
		return true
	}
	if lastErr != nil {
		if lastErr.err != nil || lastErr.body == nil {
			c.JSON(http.StatusBadGateway, gin.H{"error": "广播写入失败，所有实例均未成功"})
		} else {
			c.Data(lastErr.status, "application/json; charset=utf-8", lastErr.body)
		}
		return true
	}
	return false
}

// doInstanceSearch 向单个 Meilisearch 实例发起搜索并解析响应
func doInstanceSearch(inst model.MeiliInstance, proxyPath string, body map[string]any) (map[string]any, error) {
	payload, err := json.Marshal(body)
	if err != nil {
		return nil, err
	}

	target, err := url.JoinPath(inst.Host, strings.TrimPrefix(proxyPath, "/"))
	if err != nil {
		return nil, err
	}

	req, err := http.NewRequest(http.MethodPost, target, bytes.NewReader(payload))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+inst.APIKey)
	req.Header.Set("X-Meili-API-Key", inst.APIKey)

	client := &http.Client{Timeout: 20 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return nil, fmt.Errorf("instance %s returned status %d", inst.Name, resp.StatusCode)
	}

	var out map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&out); err != nil {
		return nil, err
	}
	return out, nil
}

// getIndexPrimaryKey 获取索引主键（带缓存），用于跨实例去重
func getIndexPrimaryKey(instances []model.MeiliInstance, uid string) string {
	pkCacheMu.RLock()
	if pk, ok := pkCache[uid]; ok {
		pkCacheMu.RUnlock()
		return pk
	}
	pkCacheMu.RUnlock()

	pk := ""
	if len(instances) > 0 {
		inst := instances[0]
		target, err := url.JoinPath(inst.Host, "indexes", uid)
		if err == nil {
			req, err := http.NewRequest(http.MethodGet, target, nil)
			if err == nil {
				req.Header.Set("Authorization", "Bearer "+inst.APIKey)
				req.Header.Set("X-Meili-API-Key", inst.APIKey)
				client := &http.Client{Timeout: 10 * time.Second}
				if resp, err := client.Do(req); err == nil {
					defer resp.Body.Close()
					if resp.StatusCode >= 200 && resp.StatusCode < 300 {
						var meta struct {
							PrimaryKey string `json:"primaryKey"`
						}
						if json.NewDecoder(resp.Body).Decode(&meta) == nil {
							pk = meta.PrimaryKey
						}
					}
				}
			}
		}
	}
	if pk == "" {
		pk = "id"
	}

	pkCacheMu.Lock()
	pkCache[uid] = pk
	pkCacheMu.Unlock()
	return pk
}

type aggregatedHit struct {
	hit      map[string]any
	score    float64
	hasScore bool
}

// mergeSearchResponses 合并多个实例的搜索结果：去重 + 排序/轮询 + 分页 + facet 合并
func mergeSearchResponses(responses []map[string]any, offset, limit int, pk string) gin.H {
	perSrc := make([][]aggregatedHit, len(responses))
	seen := map[string]bool{}
	total := 0
	maxMs := 0
	facets := map[string]map[string]int{}
	var first map[string]any
	allScored := true
	hitCount := 0

	for si, resp := range responses {
		if first == nil {
			first = resp
		}
		if v, ok := resp["estimatedTotalHits"].(float64); ok {
			total += int(v)
		}
		if v, ok := resp["processingTimeMs"].(float64); ok && int(v) > maxMs {
			maxMs = int(v)
		}
		if fd, ok := resp["facetDistribution"].(map[string]any); ok {
			for facet, distAny := range fd {
				dist, ok := distAny.(map[string]any)
				if !ok {
					continue
				}
				if facets[facet] == nil {
					facets[facet] = map[string]int{}
				}
				for val, cAny := range dist {
					if c, ok := cAny.(float64); ok {
						facets[facet][val] += int(c)
					}
				}
			}
		}

		hitsAny, _ := resp["hits"].([]any)
		for _, h := range hitsAny {
			hm, ok := h.(map[string]any)
			if !ok {
				continue
			}
			key := dedupeKey(hm, pk)
			if key != "" {
				if seen[key] {
					continue
				}
				seen[key] = true
			}
			e := aggregatedHit{hit: hm}
			if rs, ok := hm["_rankingScore"].(float64); ok {
				e.score = rs
				e.hasScore = true
			} else {
				allScored = false
			}
			perSrc[si] = append(perSrc[si], e)
			hitCount++
		}
	}

	var ordered []aggregatedHit
	if hitCount > 0 && allScored {
		for _, s := range perSrc {
			ordered = append(ordered, s...)
		}
		sort.SliceStable(ordered, func(i, j int) bool { return ordered[i].score > ordered[j].score })
	} else {
		// 轮询交错，尽量保留各来源内部的相关度顺序
		maxLen := 0
		for _, s := range perSrc {
			if len(s) > maxLen {
				maxLen = len(s)
			}
		}
		for i := 0; i < maxLen; i++ {
			for _, s := range perSrc {
				if i < len(s) {
					ordered = append(ordered, s[i])
				}
			}
		}
	}

	start := offset
	if start > len(ordered) {
		start = len(ordered)
	}
	end := start + limit
	if end > len(ordered) {
		end = len(ordered)
	}
	page := ordered[start:end]

	hits := make([]map[string]any, len(page))
	for i, e := range page {
		hits[i] = e.hit
	}

	result := gin.H{
		"hits":               hits,
		"offset":             offset,
		"limit":              limit,
		"estimatedTotalHits": total,
		"processingTimeMs":   maxMs,
	}
	if len(facets) > 0 {
		result["facetDistribution"] = facets
	}
	// 透传其它 Meilisearch 响应字段（如 query、semanticHitCount 等）
	for k, v := range first {
		switch k {
		case "hits", "estimatedTotalHits", "processingTimeMs", "facetDistribution", "offset", "limit":
			continue
		}
		result[k] = v
	}
	return result
}

// dedupeKey 计算文档去重键
func dedupeKey(hit map[string]any, pk string) string {
	if pk != "" {
		if v, ok := hit[pk]; ok && v != nil {
			return fmt.Sprintf("%v", v)
		}
	}
	if v, ok := hit["id"]; ok && v != nil {
		return fmt.Sprintf("%v", v)
	}
	return ""
}

func jsonInt(v any, def int) int {
	switch n := v.(type) {
	case float64:
		return int(n)
	case int:
		return n
	case int64:
		return int(n)
	case json.Number:
		if i, err := n.Int64(); err == nil {
			return int(i)
		}
	}
	return def
}
