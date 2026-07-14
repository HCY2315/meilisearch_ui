import axios, { AxiosInstance, AxiosRequestConfig, AxiosError } from 'axios'
import type {
  ConnectData,
  IndexData,
  IndexListResponse,
  IndexInfo,
  IndexStats,
  IndexSettings,
  DocumentsResponse,
  SearchResponse,
  SearchHit,
  PopularSearchData,
  PopularItem,
} from '@/types'

let errorHandler: ((err: Error) => void) | null = null

export function setGlobalErrorHandler(handler: (err: Error) => void) {
  errorHandler = handler
}

function createClient(host: string, apiKey: string): AxiosInstance {
  const trimmed = host.trim().replace(/\/$/, '')
  const adminToken = localStorage.getItem('authToken')
  const instance = axios.create({
    baseURL: trimmed,
    headers: {
      'Content-Type': 'application/json',
      ...(apiKey.trim() ? { 'App-Token': apiKey.trim() } : {}),
      ...(adminToken ? { 'Authorization': `Bearer ${adminToken}` } : {}),
    },
  })
  
  instance.interceptors.response.use(
    response => response,
    (error: AxiosError) => {
      if (errorHandler && error instanceof Error) {
        errorHandler(error)
      }
      return Promise.reject(error)
    }
  )
  
  return instance
}

export async function connectIndexes(host: string, apiKey: string): Promise<ConnectData> {
  const client = createClient(host, apiKey)
  const isProxy = host.includes('/api/v1/proxy')
  // 如果是代理模式，直接请求后端的 public 接口，注意这里不使用 client.get 以免 baseURL 导致 URL 嵌套错误
  const fetchUrl = isProxy ? '/api/v1/public/indexes' : `${host}/indexes`
  const resp = await axios.get<IndexListResponse>(fetchUrl, {
    headers: {
      'App-Token': apiKey,
      ...(localStorage.getItem('authToken') ? { 'Authorization': `Bearer ${localStorage.getItem('authToken')}` } : {})
    }
  })
  const results: IndexInfo[] = []
  for (const idx of resp.data.results as any[]) {
    try {
      const statsResp = await client.get<IndexStats>(`/indexes/${idx.uid}/stats`)
      results.push({ 
        uid: idx.uid, 
        count: statsResp.data.numberOfDocuments,
        isVisible: idx.isVisible,
        isLocked: idx.isLocked,
        isUnlocked: idx.isUnlocked,
        displayName: idx.displayName,
        fieldConfigs: idx.fieldConfigs,
        viewConfigs: idx.viewConfigs,
        tableConfigs: idx.tableConfigs,
        nestedFieldConfigs: idx.nestedFieldConfigs,
        canEdit: idx.canEdit,
        drawerFieldOrder: idx.drawerFieldOrder,
        primaryKey: idx.primaryKey
      })
    } catch {
      results.push({ 
        uid: idx.uid,
        isVisible: idx.isVisible,
        isLocked: idx.isLocked,
        isUnlocked: idx.isUnlocked,
        displayName: idx.displayName,
        fieldConfigs: idx.fieldConfigs,
        viewConfigs: idx.viewConfigs,
        tableConfigs: idx.tableConfigs,
        nestedFieldConfigs: idx.nestedFieldConfigs,
        canEdit: idx.canEdit,
        drawerFieldOrder: idx.drawerFieldOrder,
        primaryKey: idx.primaryKey
      })
    }
  }
  return { indexes: results }
}

export async function loadIndexData(host: string, apiKey: string, uid: string): Promise<IndexData> {
  const client = createClient(host, apiKey)
  const [settingsResp, docsResp] = await Promise.all([
    client.get<IndexSettings>(`/indexes/${uid}/settings`),
    client.get<DocumentsResponse>(`/indexes/${uid}/documents?limit=1`),
  ])

  const settings = settingsResp.data
  let searchable = settings.searchableAttributes ?? []
  let displayed = settings.displayedAttributes ?? []
  let filterable = settings.filterableAttributes ?? []
  let sortable = settings.sortableAttributes ?? []

  searchable = searchable.filter(f => f !== '*')
  displayed = displayed.filter(f => f !== '*')
  filterable = filterable.filter(f => f !== '*')
  sortable = sortable.filter(f => f !== '*')

  let embedder: string | null = null
  if (settings.embedding) {
    const emb = settings.embedding as Record<string, unknown>
    const keys = Object.keys(emb)
    if (keys.length > 0) {
      embedder = keys[0]
    }
  }

  const availableSet = new Set<string>()
  const available: string[] = []

  for (const f of [...searchable, ...displayed]) {
    if (!availableSet.has(f)) {
      availableSet.add(f)
      available.push(f)
    }
  }

  const sampleDoc = docsResp.data.results[0]
  if (sampleDoc) {
    for (const key of Object.keys(sampleDoc)) {
      if (key === '*') continue
      if (!availableSet.has(key)) {
        availableSet.add(key)
        available.push(key)
      }
    }
  }

  return {
    availableFields: available,
    searchFields: searchable,
    highlightFields: displayed,
    displayFields: displayed,
    filterableFields: filterable,
    sortableAttributes: sortable,
    embedder,
  }
}

export async function performSearch(
  host: string,
  apiKey: string,
  index: string,
  query: string,
  params: Record<string, unknown>
): Promise<SearchResponse> {
  const client = createClient(host, apiKey)
  const body = { ...params, q: query }
  const resp = await client.post<SearchResponse>(`/indexes/${index}/search`, body)
  return resp.data
}

export async function updateIndexSettings(
  host: string,
  apiKey: string,
  index: string,
  searchable: string[],
  filterable: string[]
): Promise<void> {
  const client = createClient(host, apiKey)
  await client.patch(`/indexes/${index}/settings`, {
    searchableAttributes: searchable,
    filterableAttributes: filterable,
  })
}

export async function loadPopularSearches(
  host: string,
  apiKey: string,
  index: string,
  field: string
): Promise<PopularSearchData> {
  const client = createClient(host, apiKey)
  const resp = await client.post<SearchResponse>(`/indexes/${index}/search`, {
    q: '',
    limit: 0,
    facets: [field],
  })
  const items: PopularItem[] = []
  const dist = resp.data.facetDistribution
  if (dist && dist[field]) {
    const entries = Object.entries(dist[field])
    entries.sort((a, b) => b[1] - a[1])
    for (const [val, count] of entries.slice(0, 10)) {
      items.push({ value: val, count })
    }
  }
  return { items }
}

export async function fetchById(
  host: string,
  apiKey: string,
  index: string,
  id: string
): Promise<SearchHit> {
  const client = createClient(host, apiKey)
  const safeId = !isNaN(Number(id)) ? id : `"${id.replace(/"/g, '\\"')}"`
  const resp = await client.post<SearchResponse>(`/indexes/${index}/search`, {
    q: '',
    filter: [`id = ${safeId}`],
  })
  if (!resp.data.hits.length) throw new Error('not found')
  return resp.data.hits[0]
}

export async function updateDocuments(
  host: string,
  apiKey: string,
  index: string,
  primaryKey: string,
  hits: SearchHit[],
  edits: Record<string, Record<string, unknown>>
): Promise<void> {
  const client = createClient(host, apiKey)
  const docs: Record<string, unknown>[] = []

  for (const hit of hits) {
    const docId = String(hit[primaryKey] ?? hit.id ?? '')
    if (!docId) continue
    const fields = edits[docId]
    if (!fields) continue

    const obj: Record<string, unknown> = { ...hit }
    if (primaryKey && hit[primaryKey]) obj[primaryKey] = hit[primaryKey]

    for (const [field, value] of Object.entries(fields)) {
      obj[field] = parseEditValue(value)
    }
    docs.push(obj)
  }

  if (!docs.length) return
  await client.post(`/indexes/${index}/documents`, docs)
}

function parseEditValue(value: unknown): unknown {
  if (typeof value !== 'string') return value
  const s = value.trim()
  if (s === 'null') return null
  if (s === 'true') return true
  if (s === 'false') return false
  const num = Number(s)
  if (!isNaN(num) && s !== '') return num
  return value
}

export async function createIndex(
  host: string,
  apiKey: string,
  uid: string,
  primaryKey?: string
): Promise<void> {
  const client = createClient(host, apiKey)
  await client.post('/indexes', primaryKey ? { uid, primaryKey } : { uid })
}

export async function setIndexPrimaryKey(
  host: string,
  apiKey: string,
  uid: string,
  primaryKey: string
): Promise<void> {
  const client = createClient(host, apiKey)
  await client.put(`/indexes/${uid}/primaryKey`, { primaryKey })
}

export async function batchImportDocuments(
  host: string,
  apiKey: string,
  index: string,
  jsonData: string
): Promise<string> {
  const client = createClient(host, apiKey)
  const docs = JSON.parse(jsonData)
  if (!Array.isArray(docs) || !docs.length) throw new Error('JSON数组为空')
  await client.post(`/indexes/${index}/documents`, docs)
  return `成功导入 ${docs.length} 条数据到索引 ${index}`
}

export async function exportAllCsv(
  host: string,
  apiKey: string,
  index: string,
  params: Record<string, unknown>,
  onProgress: (progress: number, total: number) => void
): Promise<string> {
  const client = createClient(host, apiKey)
  const limit = 1000
  const allHits: Record<string, unknown>[] = []
  let offset = 0

  while (true) {
    const body = { ...params, q: params.q ?? '', limit, offset }
    const resp = await client.post<SearchResponse>(`/indexes/${index}/search`, body)
    const hits = resp.data.hits
    if (!hits.length) break

    for (const hit of hits) {
      const flat: Record<string, unknown> = {}
      for (const [k, v] of Object.entries(hit)) {
        if (k === '_formatted' || k === '_rankingScore') continue
        flat[k] = v
      }
      allHits.push(flat)
    }

    const total = resp.data.estimatedTotalHits ?? offset + hits.length
    onProgress(allHits.length, total)
    offset += limit
    if (allHits.length >= total) break
  }

  const filename = `export_${index}_${Date.now()}.csv`
  const csv = buildCsv(allHits)
  downloadBlob(csv, filename, 'text/csv')
  return filename
}

function buildCsv(data: Record<string, unknown>[]): string {
  if (!data.length) return ''
  const headers = Object.keys(data[0])
  const escape = (v: unknown): string => {
    const s = String(v ?? '')
    if (s.includes(',') || s.includes('"') || s.includes('\n')) {
      return `"${s.replace(/"/g, '""')}"`
    }
    return s
  }
  const lines = [
    headers.join(','),
    ...data.map(row => headers.map(h => escape(row[h])).join(',')),
  ]
  return '\ufeff' + lines.join('\n')
}

function downloadBlob(content: string, filename: string, type: string): void {
  const blob = new Blob([content], { type })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

function getAuthHeaders(): Record<string, string> {
  const token = localStorage.getItem('authToken')
  return token ? { 'Authorization': `Bearer ${token}` } : {}
}

export async function saveIndexConfig(config: Record<string, unknown>): Promise<{ ok: boolean; status: number }> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch('/api/v1/admin/index_configs', {
    method: 'POST',
    headers,
    body: JSON.stringify(config)
  })
  return { ok: res.ok, status: res.status }
}

export async function getIndexConfig(uid: string): Promise<Record<string, unknown> | null> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/admin/index_configs/${uid}`, { headers })
  if (!res.ok) return null
  return res.json()
}

export async function getAppConfig(): Promise<Record<string, unknown> | null> {
  const headers = getAuthHeaders()
  const res = await fetch('/api/v1/app/config', { headers })
  if (!res.ok) return null
  return res.json()
}

export async function login(username: string, password: string): Promise<{ token: string; user: Record<string, unknown> }> {
  const res = await fetch('/api/v1/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password })
  })
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: '登录失败' }))
    throw new Error(err.error || '登录失败')
  }
  return res.json()
}

export async function getAdminIndexConfigs(): Promise<unknown[]> {
  const headers = getAuthHeaders()
  const res = await fetch('/api/v1/admin/index_configs', { headers })
  if (!res.ok) return []
  return res.json()
}

export async function getAdminAccessTokens(): Promise<unknown[]> {
  const headers = getAuthHeaders()
  const res = await fetch('/api/v1/admin/access_tokens', { headers })
  if (!res.ok) return []
  return res.json()
}

export async function getAdminApps(): Promise<unknown[]> {
  const headers = getAuthHeaders()
  const res = await fetch('/api/v1/admin/apps', { headers })
  if (!res.ok) return []
  return res.json()
}

export async function getProxyIndexes(): Promise<{ results: { uid: string }[] }> {
  const headers = getAuthHeaders()
  const res = await fetch('/api/v1/proxy/indexes', { headers })
  if (!res.ok) return { results: [] }
  return res.json()
}

export async function getProxyIndexStats(uid: string): Promise<IndexStats | null> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/proxy/indexes/${encodeURIComponent(uid)}/stats`, { headers })
  if (!res.ok) return null
  return res.json()
}

export async function getAdminInstances(): Promise<unknown[]> {
  const headers = getAuthHeaders()
  const res = await fetch('/api/v1/admin/instances', { headers })
  if (!res.ok) return []
  return res.json()
}

export async function createAdminInstance(data: Record<string, unknown>): Promise<boolean> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch('/api/v1/admin/instances', {
    method: 'POST',
    headers,
    body: JSON.stringify(data)
  })
  return res.ok
}

export async function updateAdminInstance(data: Record<string, unknown>): Promise<boolean> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch('/api/v1/admin/instances', {
    method: 'PUT',
    headers,
    body: JSON.stringify(data)
  })
  return res.ok
}

export async function deleteAdminInstance(id: number): Promise<boolean> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/admin/instances/${id}`, {
    method: 'DELETE',
    headers
  })
  return res.ok
}

export async function saveAdminIndexConfig(data: Record<string, unknown>): Promise<boolean> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch('/api/v1/admin/index_configs', {
    method: 'POST',
    headers,
    body: JSON.stringify(data)
  })
  return res.ok
}

export async function deleteAdminIndexConfig(uid: string): Promise<boolean> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/admin/index_configs/${uid}`, {
    method: 'DELETE',
    headers
  })
  return res.ok
}

export async function createAccessToken(data: Record<string, unknown>): Promise<boolean> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch('/api/v1/admin/access_tokens', {
    method: 'POST',
    headers,
    body: JSON.stringify(data)
  })
  return res.ok
}

export async function updateAccessToken(data: Record<string, unknown>): Promise<boolean> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch('/api/v1/admin/access_tokens', {
    method: 'PUT',
    headers,
    body: JSON.stringify(data)
  })
  return res.ok
}

export async function deleteAccessToken(id: number): Promise<boolean> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/admin/access_tokens/${id}`, {
    method: 'DELETE',
    headers
  })
  return res.ok
}

export async function updateApp(id: number, data: Record<string, unknown>): Promise<boolean> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch(`/api/v1/admin/apps/${id}`, {
    method: 'PUT',
    headers,
    body: JSON.stringify(data)
  })
  return res.ok
}

export async function createApp(data: Record<string, unknown>): Promise<{ ok: boolean; data?: any }> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch('/api/v1/admin/apps', {
    method: 'POST',
    headers,
    body: JSON.stringify(data)
  })
  const payload = await res.json().catch(() => ({}))
  return { ok: res.ok, data: payload }
}

export async function deleteApp(id: number): Promise<boolean> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/admin/apps/${id}`, {
    method: 'DELETE',
    headers
  })
  return res.ok
}

// ---- 租户管理 API ----

export async function getAdminTenants(): Promise<any[]> {
  const headers = getAuthHeaders()
  const res = await fetch('/api/v1/admin/tenants', { headers })
  if (!res.ok) return []
  return res.json()
}

export async function createTenant(data: Record<string, unknown>): Promise<{ ok: boolean; data?: any; error?: string }> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch('/api/v1/admin/tenants', {
    method: 'POST',
    headers,
    body: JSON.stringify(data)
  })
  const payload = await res.json().catch(() => ({}))
  if (!res.ok) return { ok: false, error: payload.error || '创建失败' }
  return { ok: true, data: payload }
}

export async function updateTenant(id: number, data: Record<string, unknown>): Promise<boolean> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch(`/api/v1/admin/tenants/${id}`, {
    method: 'PUT',
    headers,
    body: JSON.stringify(data)
  })
  return res.ok
}

export async function deleteTenant(id: number): Promise<boolean> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/admin/tenants/${id}`, {
    method: 'DELETE',
    headers
  })
  return res.ok
}

export async function getTenantApps(tenantId: number): Promise<any[]> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/admin/tenants/${tenantId}/apps`, { headers })
  if (!res.ok) return []
  return res.json()
}


export async function updateAdminPassword(newPassword: string): Promise<boolean> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch('/api/v1/admin/password', {
    method: 'PUT',
    headers,
    body: JSON.stringify({ newPassword })
  })
  return res.ok
}

// Meilisearch 设置
export async function getMeiliIndexSettings(uid: string): Promise<any> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/admin/meilisearch/settings/${uid}`, { headers })
  if (!res.ok) return null
  return res.json()
}

export async function updateMeiliIndexSettings(
  uid: string,
  settings: {
    searchableAttributes?: string[]
    filterableAttributes?: string[]
    embedders?: Record<string, unknown>
  }
): Promise<boolean> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch(`/api/v1/admin/meilisearch/settings/${uid}`, {
    method: 'PUT',
    headers,
    body: JSON.stringify({ uid, ...settings })
  })
  return res.ok
}

export async function validateMeiliEmbedder(uid: string, embedder: Record<string, unknown>): Promise<{ ok: boolean; message?: string }> {
  const headers = { ...getAuthHeaders(), 'Content-Type': 'application/json' }
  const res = await fetch(`/api/v1/admin/meilisearch/settings/${uid}/validate-embedder`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ uid, embedder })
  })
  const payload = await res.json().catch(() => ({}))
  if (!res.ok) return { ok: false, message: payload.error || '模型校验失败' }
  return { ok: true }
}

export async function getMeiliTasks(host: string, apiKey: string): Promise<any> {
  const headers = getAuthHeaders()
  const res = await fetch('/api/v1/proxy/tasks?limit=50', { headers })
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: '请求任务失败' }))
    throw new Error(err.error || '获取任务失败')
  }
  return res.json()
}

export async function cancelMeiliTask(uid: number | string): Promise<boolean> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/proxy/tasks/cancel?uids=${uid}`, {
    method: 'POST',
    headers
  })
  return res.ok
}

// Token 申请相关
export async function sendVerificationCode(email: string): Promise<void> {
  const res = await fetch('/api/v1/application/send-code', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email })
  })
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: '发送失败' }))
    throw new Error(err.error || '发送失败')
  }
}

export async function submitApplication(data: Record<string, unknown>): Promise<void> {
  const res = await fetch('/api/v1/application/submit', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data)
  })
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: '提交失败' }))
    throw new Error(err.error || '提交失败')
  }
}

export async function getApplications(): Promise<any[]> {
  const headers = getAuthHeaders()
  const res = await fetch('/api/v1/admin/applications', { headers })
  if (!res.ok) return []
  return res.json()
}

export async function approveApplication(id: number, data: Record<string, unknown>): Promise<{ ok: boolean; message: string }> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/admin/applications/${id}/approve`, {
    method: 'POST',
    headers: { ...headers, 'Content-Type': 'application/json' },
    body: JSON.stringify(data)
  })
  const payload = await res.json().catch(() => ({}))
  if (!res.ok) return { ok: false, message: payload.error || '审批失败' }
  return { ok: true, message: payload.message || '审批成功' }
}

export async function rejectApplication(id: number, data?: Record<string, unknown>): Promise<{ ok: boolean; message: string }> {
  const headers = getAuthHeaders()
  const res = await fetch(`/api/v1/admin/applications/${id}/reject`, {
    method: 'POST',
    headers: { ...headers, 'Content-Type': 'application/json' },
    body: JSON.stringify(data || {})
  })
  const payload = await res.json().catch(() => ({}))
  if (!res.ok) return { ok: false, message: payload.error || '驳回失败' }
  return { ok: true, message: payload.message || '驳回成功' }
}

export async function getAdminUsageMetrics(): Promise<{ results: any[]; indexResults: any[] }> {
  const headers = getAuthHeaders()
  const res = await fetch('/api/v1/admin/usage_metrics', { headers })
  if (!res.ok) return { results: [], indexResults: [] }
  const payload = await res.json().catch(() => ({ results: [], indexResults: [] }))
  return {
    results: payload.results || [],
    indexResults: payload.indexResults || [],
  }
}
