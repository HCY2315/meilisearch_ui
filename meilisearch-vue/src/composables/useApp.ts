import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import type {
  QueryRow,
  HistoryItem,
  IndexInfo,
  SearchHit,
  SearchResponse,
  SearchResponse as SearchResponseType,
  FacetDistribution,
  AiConfig,
  Toast,
  ToastType,
  ToastType as ToastTypeEnum,
  DeviceAsset,
  ViewConfig,
  FieldConfigItem,
  CurrentTab,
  NestedFieldConfigsMap,
  NestedFieldConfigItem,
} from '@/types'
import * as api from '@/services/api'
import * as storage from '@/services/storage'
import {
  buildFilterExpression,
  sortHits,
  getIdString,
  getDocKey,
  valueToStringForEdit,
  getCellValue,
  hitHasId,
  normalizeViewWidths,
  normalizeLabelWidths,
  formatNumber,
  generateUUID,
} from '@/utils'

let nextQueryId = 1
let nextToastId = 1

function createQueryRow(): QueryRow {
  return { id: nextQueryId++, field: '', operator: '=', value: '', logic: 'AND' }
}

export const useAppStore = defineStore('app', () => {
  const hostInput = ref('http://localhost:7700')
  const apiKeyInput = ref('123456')
  const indexes = ref<IndexInfo[]>([])
  const currentIndex = ref(storage.storageGet('currentIndex') || '')
  const searchInput = ref('')
  const queryRows = ref<QueryRow[]>([createQueryRow()])

  const searchFields = ref<string[]>([])
  const searchFieldWeights = ref<Record<string, number>>({})
  const filterableFields = ref<string[]>([])
  const availableFields = ref<string[]>([])
  const highlightFields = ref<string[]>([])
  const displayFields = ref<string[]>([])
  const facets = ref<Record<string, string[]>>({})
  const searchHistory = ref<HistoryItem[]>(storage.loadSearchHistory())
  const fieldLabels = ref<Record<string, string>>({})
  const popularSearches = ref<{ value: string; count?: number }[]>([])
  const popularSearchField = ref('')

  const currentPage = ref(1)
  const pageSize = ref(10)
  const maxResultsPerPage = ref(1000)
  const resultsCount = ref(0)
  const processingTimeMs = ref<number | null>(null)
  const facetDistribution = ref<FacetDistribution | null>(null)
  const sortableAttributes = ref<string[]>([])
  const lastHits = ref<SearchHit[]>([])
  const lastResults = ref<SearchResponseType | null>(null)
  const lastBaseColumns = ref<string[]>([])

  const tableSortField = ref('')
  const tableSortDir = ref('asc')
  const columnOrder = ref<string[]>([])
  const hiddenColumns = ref<string[]>([])
  const columnWidths = ref<Record<string, number>>({})
  const drawerFieldOrder = ref<Record<string, string[]>>({})

  const aiConfig = ref<AiConfig>(storage.loadAiConfig())
  const aiDropdownOpen = ref(false)
  const aiEmbedder = ref<string | null>(null)
  const highlightEnabled = ref(true)
  const showRankingScore = ref(true)
  const cropLength = ref(1000)
  const sortValue = ref('')

  const filtersDrawerOpen = ref(false)
  const columnConfigOpen = ref(false)
  const fieldConfigOpen = ref(false)
  const viewModalOpen = ref(false)
  const advancedSettingsOpen = ref(false)
  const viewMode = ref('table')
  const viewNameInput = ref('')
  const viewConfigs = ref<ViewConfig[]>([])

  const exportDownloading = ref(false)
  const exportProgress = ref(0)
  const exportTotal = ref(0)

  const loading = ref(false)
  const toasts = ref<Toast[]>([])
  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  const resultModalOpen = ref(false)
  const resultDetail = ref<SearchHit | null>(null)
  const editLocked = ref(true)
  const primaryKeyField = ref('id')
  const pendingEdits = ref<Record<string, Record<string, unknown>>>({})

  const imagePreviewEnabled = ref(storage.loadImagePreviewEnabled())
  const imagePreviewLinksOnly = ref(storage.loadImagePreviewLinksOnly())
  const imagePreviewSize = ref(storage.loadImagePreviewSize())

  const currentTab = ref<CurrentTab>('search')
  const assetForm = ref<DeviceAsset>({ id: generateUUID(), name: '', brand: '', model: '' })
  const assetList = ref<DeviceAsset[]>([])
  const assetModalOpen = ref(false)
  const assetDetail = ref<DeviceAsset | null>(null)
  const assetsLoading = ref(false)
  const uploadModalOpen = ref(false)
  const uploadData = ref('')
  const uploadFileName = ref('')
  const uploadPreviewData = ref<Record<string, unknown>[]>([])
  const uploadPreviewPage = ref(1)
  const uploadPreviewPageSize = ref(10)
  const uploadProgress = ref(0)
  const uploadLoading = ref(false)
  // NOTE: 导入方式切换：'file' 为文件上传，'json' 为 JSON 文本直接输入
  const importMode = ref<'file' | 'json'>('file')
  // JSON 文本输入框的内容（数组格式）
  const jsonTextInput = ref('')

  const newIndexUid = ref('')
  const newIndexPk = ref('')

  const draggingCol = ref<string | null>(null)
  const dragOverCol = ref<string | null>(null)
  const isResizingColumns = ref(false)

  const viewLayoutWorking = ref<string[][]>([[], []])
  const viewWidthsWorking = ref<number[]>([0, 0])
  const viewLabelWidthsWorking = ref<number[]>([140, 140])
  const viewDragField = ref<string | null>(null)
  const viewDragFromColumn = ref<number | null>(null)
  const viewDragColumn = ref<number | null>(null)
  const viewDragOverIndex = ref<number | null>(null)

  const viewResizeState = ref<{ colIdx: number; startX: number; startWidthPx: number; containerWidthPx: number; baseWidths: number[] } | null>(null)

  const viewFieldResizeState = ref<{ colIdx: number; startX: number; startWidthPx: number; containerWidthPx: number; baseWidths: number[] } | null>(null)
  const nestedFieldConfigs = ref<NestedFieldConfigsMap>({})
  let nestedConfigSaveTimer: ReturnType<typeof setTimeout> | null = null

  const visibleColumns = computed(() => {
    let cols = [...lastBaseColumns.value]
    if (columnOrder.value.length) {
      const ordered: string[] = []
      for (const c of columnOrder.value) {
        if (cols.includes(c)) ordered.push(c)
      }
      for (const c of cols) {
        if (!ordered.includes(c)) ordered.push(c)
      }
      cols = ordered
    }
    return cols.filter(c => !hiddenColumns.value.includes(c))
  })

  const visibleAvailableFields = computed(() => {
    return availableFields.value.filter(f => !hiddenColumns.value.includes(f))
  })

  const visibleFilterableFields = computed(() => {
    return filterableFields.value.filter(f => !hiddenColumns.value.includes(f))
  })

  const visibleFacetDistribution = computed(() => {
    if (!facetDistribution.value) return null
    const result: FacetDistribution = {}
    for (const [key, val] of Object.entries(facetDistribution.value)) {
      if (!hiddenColumns.value.includes(key)) {
        result[key] = val
      }
    }
    return result
  })

  const totalPages = computed(() => Math.ceil(resultsCount.value / pageSize.value))

  const getHost = () => hostInput.value.trim()
  const getApiKey = () => apiKeyInput.value.trim()

  function normalizeNestedPath(pathLabel: string): string {
    return pathLabel.replace(/\[\d+\]/g, '[*]')
  }

  function getNestedFieldConfig(pathLabel: string): Record<string, NestedFieldConfigItem> {
    return nestedFieldConfigs.value[normalizeNestedPath(pathLabel)] || {}
  }

  function setNestedFieldConfig(pathLabel: string, fieldConfig: Record<string, NestedFieldConfigItem>) {
    const normalizedPath = normalizeNestedPath(pathLabel)
    nestedFieldConfigs.value = {
      ...nestedFieldConfigs.value,
      [normalizedPath]: fieldConfig,
    }
  }

  function scheduleSaveNestedFieldConfigs() {
    if (nestedConfigSaveTimer) clearTimeout(nestedConfigSaveTimer)
    nestedConfigSaveTimer = setTimeout(() => {
      saveNestedFieldConfigs()
    }, 300)
  }

  async function saveNestedFieldConfigs() {
    const token = localStorage.getItem('authToken')
    if (!token) return
    if (!currentIndex.value) return
    try {
      const res = await fetch('/api/v1/admin/nested_field_configs', {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          uid: currentIndex.value,
          nestedFieldConfigs: JSON.stringify(nestedFieldConfigs.value),
        }),
      })
      if (!res.ok) {
        pushToast(`保存嵌套字段配置失败: ${res.status}`, 'error')
      }
    } catch {
      pushToast('保存嵌套字段配置失败: 网络错误', 'error')
    }
  }

  async function saveDrawerFieldOrder() {
    const token = localStorage.getItem('authToken')
    if (!token) return
    if (!currentIndex.value) return
    try {
      const res = await fetch('/api/v1/admin/drawer_field_order', {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          uid: currentIndex.value,
          drawerFieldOrder: JSON.stringify(drawerFieldOrder.value),
        }),
      })
      if (!res.ok) {
        pushToast(`保存抽屉字段顺序失败: ${res.status}`, 'error')
      }
    } catch {
      pushToast('保存抽屉字段顺序失败: 网络错误', 'error')
    }
  }

  function pushToast(message: string, kind: ToastTypeEnum) {
    toasts.value.push({ id: nextToastId++, message, kind })
    setTimeout(() => {
      toasts.value.shift()
    }, 3000)
  }

  async function connect() {
    if (!hostInput.value.trim()) {
      pushToast('请输入服务器地址', 'error')
      return
    }
    loading.value = true
    try {
      const data = await api.connectIndexes(getHost(), getApiKey())
      indexes.value = data.indexes
      if (currentIndex.value && indexes.value.some(i => i.uid === currentIndex.value)) {
        await selectIndex(currentIndex.value)
      } else if (currentIndex.value) {
        // indexes 加载后如果 currentIndex 被设置但不在列表中，需要重新选择
        currentIndex.value = ''
      }
      pushToast('连接成功！', 'success')
    } catch (e) {
      pushToast(`连接失败: ${e}`, 'error')
    } finally {
      loading.value = false
    }
  }

  // 监听 currentIndex 或 indexes 变化，自动加载索引数据
  watch([currentIndex, indexes], async ([newIndex, idxList]) => {
    if (!newIndex || !idxList.length) return
    const exists = idxList.some(i => i.uid === newIndex)
    if (exists) {
      await selectIndex(newIndex)
    } else {
      currentIndex.value = ''
    }
  })

  async function selectIndex(uid: string) {
    currentIndex.value = uid
    if (uid) storage.storageSet('currentIndex', uid)
    resetIndexState()
    if (!uid) {
      popularSearches.value = []
      return
    }

    const idxMeta = indexes.value.find(i => i.uid === uid)

    loading.value = true
    try {
      const data = await api.loadIndexData(getHost(), getApiKey(), uid)
      availableFields.value = data.availableFields
      searchFields.value = data.searchFields
      highlightFields.value = data.highlightFields
      displayFields.value = data.displayFields
      filterableFields.value = data.filterableFields
      sortableAttributes.value = data.sortableAttributes
      aiEmbedder.value = data.embedder
      sortValue.value = ''
      
      // ---- 重要：应用后端同步过来的持久化配置 ----
      if (idxMeta) {
        // 1. 字段与权限设置
        if (idxMeta.fieldConfigs) {
          try {
            const configs: FieldConfigItem[] = JSON.parse(idxMeta.fieldConfigs)
            const searchable: string[] = []
            const highlight: string[] = []
            const display: string[] = []
            const weights: Record<string, number> = {}
            const labels: Record<string, string> = {}
            for (const item of configs) {
               if (item.searchable) searchable.push(item.field)
               if (item.highlight) highlight.push(item.field)
               if (item.display) display.push(item.field)
               weights[item.field] = item.weight
               labels[item.field] = item.label
            }
            searchFields.value = searchable
            highlightFields.value = highlight
            displayFields.value = display
            searchFieldWeights.value = weights
            fieldLabels.value = labels
          } catch (e) { console.error('Parse fieldConfigs failed', e) }
        }
        // 2. 视图设置
        if (idxMeta.viewConfigs) {
          try { 
            const parsed = JSON.parse(idxMeta.viewConfigs)
            if (Array.isArray(parsed)) {
              for (const cfg of parsed) {
                cfg.widths = normalizeViewWidths(cfg.widths, Math.max(cfg.columns.length, 1))
                cfg.labelWidths = normalizeLabelWidths(cfg.labelWidths, Math.max(cfg.columns.length, 1))
              }
              viewConfigs.value = parsed
            }
          } catch(e) { console.error('Parse viewConfigs failed', e) }
        }
        // 3. 表格布局设置 (顺序与隐藏)
        if (idxMeta.tableConfigs) {
          try {
            const tConf = JSON.parse(idxMeta.tableConfigs)
            if (tConf.order) columnOrder.value = tConf.order
            if (tConf.hidden) hiddenColumns.value = tConf.hidden
          } catch (e) { console.error('Parse tableConfigs failed', e) }
        }
        // 4. 嵌套抽屉字段配置（按路径）
        if (idxMeta.nestedFieldConfigs) {
          try {
            const parsedNested = JSON.parse(idxMeta.nestedFieldConfigs)
            nestedFieldConfigs.value = parsedNested && typeof parsedNested === 'object' ? parsedNested : {}
          } catch (e) {
            console.error('Parse nestedFieldConfigs failed', e)
            nestedFieldConfigs.value = {}
          }
        } else {
          nestedFieldConfigs.value = {}
        }
        // 4. 编辑权限
        if (idxMeta.canEdit !== undefined) {
          editLocked.value = !idxMeta.canEdit
        }
        // 5. 抽屉字段顺序（每层独立）
        console.log('idxMeta.drawerFieldOrder:', idxMeta.drawerFieldOrder)
        console.log('idxMeta keys:', Object.keys(idxMeta))
        if (idxMeta.drawerFieldOrder) {
          try {
            const dfo = JSON.parse(idxMeta.drawerFieldOrder)
            console.log('parsed drawerFieldOrder:', dfo)
            drawerFieldOrder.value = (dfo && typeof dfo === 'object') ? dfo : {}
            console.log('drawerFieldOrder.value after set:', drawerFieldOrder.value)
          } catch (e) {
            console.error('Parse drawerFieldOrder failed', e)
            drawerFieldOrder.value = {}
          }
        } else {
          drawerFieldOrder.value = {}
          console.log('no drawerFieldOrder in idxMeta, set to {}')
        }
      }

      updatePopularField()
      // 如果有后端配置，则不再从本地加载
      if (!idxMeta?.tableConfigs) {
        loadColumnPrefs()
      }
      loadColumnWidthPrefs()
      // 如果没有后端配置，再尝试加载本地(兼容逻辑)
      if (!idxMeta?.fieldConfigs) loadFieldLabels()
      if (!idxMeta?.viewConfigs) loadViewConfigs()
      else {
        // 后端有配置时，尝试恢复该 index 下上次选中的 viewMode
        const modeKey = `viewMode:${currentIndex.value || 'default'}`
        viewMode.value = localStorage.getItem(modeKey) || 'table'
      }

      if (filterableFields.value.length > 0) {
        popularSearchField.value = filterableFields.value[0]
        await loadPopularSearches()
      }
      pushToast(`已选择索引: ${uid}`, 'success')
      await performSearch()
    } catch (e) {
      const errMsg = String(e)
      // 资源受限时静默处理，不弹出错误 toast
      if (!errMsg.includes('403') && !errMsg.includes('locked') && !errMsg.includes('Unauthorized')) {
        pushToast(`加载索引失败: ${e}`, 'error')
      }
    } finally {
      loading.value = false
    }
  }

  function currentFieldConfigsForSync(): FieldConfigItem[] {
    return availableFields.value.map(f => ({
      field: f,
      searchable: searchFields.value.includes(f),
      highlight: highlightFields.value.includes(f),
      display: displayFields.value.includes(f),
      weight: searchFieldWeights.value[f] || 0,
      label: fieldLabels.value[f] || f
    }))
  }

  function resetIndexState() {
    availableFields.value = []
    searchFields.value = []
    highlightFields.value = []
    displayFields.value = []
    filterableFields.value = []
    facets.value = {}
    lastHits.value = []
    lastResults.value = null
    lastBaseColumns.value = []
    columnOrder.value = []
    hiddenColumns.value = []
    columnWidths.value = {}
    searchFieldWeights.value = {}
    popularSearches.value = []
    popularSearchField.value = ''
    currentPage.value = 1
    primaryKeyField.value = 'id'
    pendingEdits.value = {}
    viewMode.value = 'table'
    viewConfigs.value = []
    resultsCount.value = 0
    processingTimeMs.value = null
    facetDistribution.value = null
    sortableAttributes.value = []
    nestedFieldConfigs.value = {}
    if (nestedConfigSaveTimer) {
      clearTimeout(nestedConfigSaveTimer)
      nestedConfigSaveTimer = null
    }
  }

  function scheduleDebouncedSearch() {
    if (debounceTimer) clearTimeout(debounceTimer)
    debounceTimer = setTimeout(() => performSearch(), 300)
  }

  async function performSearch() {
    if (!currentIndex.value) return
    loading.value = true
    try {
      const params = buildSearchParams()
      let res = await api.performSearch(getHost(), getApiKey(), currentIndex.value, searchInput.value.trim(), params)
      if (searchInput.value.trim()) addToHistory(searchInput.value.trim())
      lastHits.value = res.hits ?? []
      lastResults.value = { ...res, hits: res.hits ?? [] }
      applyResultsColumns(res)
      updateStats(res)
      updateFacetsFromResponse(res)
    } catch (e) {
      const errMsg = String(e)
      if (aiConfig.value.aiEnabled && aiConfig.value.aiWeight > 0 && (errMsg.includes('embedder') || errMsg.includes('image') || errMsg.includes('model'))) {
        aiConfig.value.aiEnabled = false
        storage.saveAiConfig(aiConfig.value)
        try {
          const params = buildSearchParams()
          const res = await api.performSearch(getHost(), getApiKey(), currentIndex.value, searchInput.value.trim(), params)
          if (searchInput.value.trim()) addToHistory(searchInput.value.trim())
          lastHits.value = res.hits ?? []
          lastResults.value = { ...res, hits: res.hits ?? [] }
          applyResultsColumns(res)
          updateStats(res)
          updateFacetsFromResponse(res)
          pushToast('AI搜索不可用，已自动关闭', 'warning')
        } catch (e2) {
          pushToast(`搜索失败: ${e2}`, 'error')
        }
      } else {
        pushToast(`搜索失败: ${e}`, 'error')
      }
    } finally {
      loading.value = false
    }
  }

  function buildSearchParams(): Record<string, unknown> {
    const limit = Math.min(pageSize.value, maxResultsPerPage.value)
    const params: Record<string, unknown> = {
      limit,
      offset: Math.max(currentPage.value - 1, 0) * limit,
      matchingStrategy: 'last',
    }

    if (highlightEnabled.value) {
      const targets = highlightFields.value.length ? highlightFields.value : searchFields.value.length ? searchFields.value : ['*']
      params.attributesToHighlight = targets
      params.highlightPreTag = '<em class="highlight">'
      params.highlightPostTag = '</em>'
    }

    if (cropLength.value > 0 && highlightEnabled.value) {
      const targets = highlightFields.value.length ? highlightFields.value : searchFields.value.length ? searchFields.value : ['*']
      params.attributesToCrop = targets
      params.cropLength = cropLength.value
    }

    if (showRankingScore.value) {
      params.showRankingScore = true
      params.showRankingScoreDetails = true
    }

    if (aiConfig.value.aiEnabled && aiConfig.value.aiWeight > 0) {
      const ratio = Math.min(aiConfig.value.aiWeight, 100) / 100
      const hybrid: Record<string, unknown> = { semanticRatio: ratio }
      if (aiEmbedder.value) hybrid.embedder = aiEmbedder.value
      params.hybrid = hybrid
    }

    if (sortValue.value) params.sort = [sortValue.value]

    const filterParts: string[] = []
    const expr = buildFilterExpression(queryRows.value)
    if (expr) filterParts.push(`(${expr})`)

    const facetFilters = buildFacetFilters()
    if (facetFilters.length) filterParts.push(`(${facetFilters.join(') AND (')})`)

    if (filterParts.length) params.filter = filterParts.join(' AND ')

    if (displayFields.value.length) {
      const attrs = [...displayFields.value]
      if (!attrs.includes('id')) attrs.push('id')
      if (primaryKeyField.value && !attrs.includes(primaryKeyField.value)) attrs.push(primaryKeyField.value)
      params.attributesToRetrieve = attrs
    } else {
      params.attributesToRetrieve = ['*']
    }

    if (filterableFields.value.length) params.facets = filterableFields.value

    return params
  }

  function buildFacetFilters(): string[] {
    const filters: string[] = []
    for (const [facet, values] of Object.entries(facets.value)) {
      if (!values.length) continue
      if (values.length === 1) {
        filters.push(`${facet} = "${values[0]}"`)
      } else {
        filters.push(`${facet} IN [${values.map(v => `"${v}"`).join(', ')}]`)
      }
    }
    return filters
  }

  function applyResultsColumns(res: SearchResponseType) {
    const hits = res.hits
    if (!hits.length) { lastBaseColumns.value = []; return }
    let cols: string[]
    if (displayFields.value.length) {
      cols = [...displayFields.value]
    } else {
      cols = Object.keys(hits[0]).filter(k => k !== 'id' && k !== '_formatted' && k !== '_rankingScore')
    }
    if (!cols.length) cols = Object.keys(hits[0]).filter(k => k !== '_formatted' && k !== '_rankingScore')
    if (!cols.length) cols = ['id']
    if (hitHasId(hits[0]) && !cols.includes('id')) cols.unshift('id')
    lastBaseColumns.value = cols
  }

  function updateStats(res: SearchResponseType) {
    resultsCount.value = res.estimatedTotalHits ?? 0
    processingTimeMs.value = res.processingTimeMs ?? null
  }

  function updateFacetsFromResponse(res: SearchResponseType) {
    facetDistribution.value = res.facetDistribution ?? null
  }

  function addQueryRow() {
    queryRows.value.push(createQueryRow())
  }

  function removeQueryRow(id: number) {
    queryRows.value = queryRows.value.filter(r => r.id !== id)
    if (!queryRows.value.length) addQueryRow()
  }

  function clearQuery() {
    searchInput.value = ''
    queryRows.value = [createQueryRow()]
    currentPage.value = 1
    performSearch()
  }

  function addToHistory(query: string) {
    if (!query || searchHistory.value.some(h => h.query === query)) return
    const item: HistoryItem = { query, timestamp: new Date().toISOString() }
    searchHistory.value.unshift(item)
    if (searchHistory.value.length > 20) searchHistory.value.length = 20
    storage.saveSearchHistory(searchHistory.value)
  }

  function clearHistory() {
    searchHistory.value = []
    storage.saveSearchHistory([])
  }

  async function loadPopularSearches() {
    if (!currentIndex.value || !popularSearchField.value) {
      popularSearches.value = []
      return
    }
    try {
      const data = await api.loadPopularSearches(getHost(), getApiKey(), currentIndex.value, popularSearchField.value)
      popularSearches.value = data.items
    } catch {
      popularSearches.value = []
    }
  }

  function updatePopularField() {
    const saved = storage.loadPopularField(currentIndex.value || null)
    if (saved && filterableFields.value.includes(saved)) {
      popularSearchField.value = saved
    } else if (filterableFields.value.length) {
      popularSearchField.value = filterableFields.value[0]
      storage.savePopularField(popularSearchField.value, currentIndex.value || null)
    } else {
      popularSearchField.value = ''
    }
  }

  function toggleFacet(facet: string, value: string, checked: boolean) {
    if (!facets.value[facet]) facets.value[facet] = []
    if (checked) {
      if (!facets.value[facet].includes(value)) facets.value[facet].push(value)
    } else {
      facets.value[facet] = facets.value[facet].filter(v => v !== value)
    }
    currentPage.value = 1
    performSearch()
  }

  function toggleSearchField(field: string, checked: boolean) {
    if (checked) {
      if (!searchFields.value.includes(field)) searchFields.value.push(field)
    } else {
      searchFields.value = searchFields.value.filter(f => f !== field)
    }
    performSearch()
  }

  function toggleTableSort(field: string) {
    if (isResizingColumns.value || draggingCol.value) return
    if (tableSortField.value === field) {
      if (tableSortDir.value === 'asc') tableSortDir.value = 'desc'
      else { tableSortField.value = ''; tableSortDir.value = 'asc' }
    } else {
      tableSortField.value = field
      tableSortDir.value = 'asc'
    }
    performSearch()
  }

  function goToPage(page: number) {
    if (page === currentPage.value || page < 1) return
    currentPage.value = page
    performSearch()
  }

  function moveColumnOrder(from: string, to: string) {
    let cols = columnOrder.value.length ? columnOrder.value.filter(c => lastBaseColumns.value.includes(c)) : [...lastBaseColumns.value]
    const fi = cols.indexOf(from)
    const ti = cols.indexOf(to)
    if (fi < 0 || ti < 0) return
    const item = cols.splice(fi, 1)[0]
    cols.splice(ti, 0, item)
    columnOrder.value = cols
    saveColumnPrefs()
  }

  function setColumnWidth(col: string, width: number) {
    columnWidths.value[col] = width
  }

  function saveColumnWidthPrefs() {
    const key = `columnWidthPrefs:${currentIndex.value || 'default'}`
    localStorage.setItem(key, JSON.stringify(columnWidths.value))
  }

  function loadColumnWidthPrefs() {
    const key = `columnWidthPrefs:${currentIndex.value || 'default'}`
    const raw = localStorage.getItem(key)
    if (raw) {
      try { columnWidths.value = JSON.parse(raw) } catch { columnWidths.value = {} }
    } else {
      columnWidths.value = {}
    }
  }

  function loadColumnPrefs() {
    const key = `columnPrefs:${currentIndex.value || 'default'}`
    const raw = localStorage.getItem(key)
    if (!raw) return
    try {
      const data = JSON.parse(raw)
      if (data.order) columnOrder.value = data.order
      if (data.hidden) hiddenColumns.value = data.hidden
    } catch { /* ignore */ }
  }

  function saveColumnPrefs() {
    const key = `columnPrefs:${currentIndex.value || 'default'}`
    localStorage.setItem(key, JSON.stringify({ order: columnOrder.value, hidden: hiddenColumns.value }))
  }

  function loadFieldLabels() {
    fieldLabels.value = storage.loadFieldLabels(currentIndex.value || null)
  }

  function saveFieldLabels() {
    storage.saveFieldLabels(fieldLabels.value, currentIndex.value || null)
  }

  function loadViewConfigs() {
    const key = `viewConfigs:${currentIndex.value || 'default'}`
    const raw = localStorage.getItem(key)
    viewConfigs.value = raw ? JSON.parse(raw) : []
    for (const cfg of viewConfigs.value) {
      cfg.widths = normalizeViewWidths(cfg.widths, Math.max(cfg.columns.length, 1))
      cfg.labelWidths = normalizeLabelWidths(cfg.labelWidths, Math.max(cfg.columns.length, 1))
    }
    const modeKey = `viewMode:${currentIndex.value || 'default'}`
    viewMode.value = localStorage.getItem(modeKey) || 'table'
  }

  function saveViewConfigs() {
    const key = `viewConfigs:${currentIndex.value || 'default'}`
    localStorage.setItem(key, JSON.stringify(viewConfigs.value))
  }

  function saveViewMode() {
    const key = `viewMode:${currentIndex.value || 'default'}`
    localStorage.setItem(key, viewMode.value)
  }

  function activeViewConfig(): ViewConfig | undefined {
    return viewConfigs.value.find(c => c.name === viewMode.value)
  }

  async function saveFieldConfig(items: FieldConfigItem[]) {
    const searchable: string[] = []
    const highlight: string[] = []
    const display: string[] = []
    const weights: Record<string, number> = {}
    const labels: Record<string, string> = {}

    for (const item of items) {
      if (item.searchable) searchable.push(item.field)
      if (item.highlight) highlight.push(item.field)
      if (item.display) display.push(item.field)
      weights[item.field] = item.weight
      labels[item.field] = item.label
    }

    searchFields.value = searchable
    highlightFields.value = highlight
    displayFields.value = display
    searchFieldWeights.value = weights
    fieldLabels.value = labels
    saveFieldLabels()
    updatePopularField()

    loading.value = true
    try {
      const nextFilterable = [...filterableFields.value]
      for (const f of searchable) {
        if (!nextFilterable.includes(f)) nextFilterable.push(f)
      }
      filterableFields.value = nextFilterable
      await api.updateIndexSettings(getHost(), getApiKey(), currentIndex.value, searchable, nextFilterable)
      pushToast('字段配置已保存', 'success')
      await loadPopularSearches()
      performSearch()
    } catch (e) {
      pushToast(`保存字段配置但无法更新索引设置: ${e}`, 'warning')
    } finally {
      loading.value = false
    }
  }

  function toggleEditLock() {
    editLocked.value = !editLocked.value
    if (!editLocked.value) {
      if (!lastBaseColumns.value.length) pushToast('暂无结果列，请先搜索', 'warning')
      else if (!primaryKeyField.value || !lastBaseColumns.value.includes(primaryKeyField.value)) {
        primaryKeyField.value = lastBaseColumns.value.includes('id') ? 'id' : lastBaseColumns.value[0]
      }
      if (!primaryKeyField.value) pushToast('请选择主键字段', 'warning')
    }
  }

  function updateCellEdit(docId: string, field: string, value: unknown) {
    if (!docId) return
    if (!pendingEdits.value[docId]) pendingEdits.value[docId] = {}
    pendingEdits.value[docId][field] = value
  }

  async function saveEdits() {
    if (editLocked.value) { pushToast('当前处于锁定状态', 'warning'); return }
    if (!Object.keys(pendingEdits.value).length) { pushToast('没有需要保存的修改', 'warning'); return }
    loading.value = true
    try {
      await api.updateDocuments(getHost(), getApiKey(), currentIndex.value, primaryKeyField.value, lastHits.value, pendingEdits.value)
      pendingEdits.value = {}
      pushToast('修改已提交保存', 'success')
      await performSearch()
    } catch (e) {
      pushToast(`保存失败: ${e}`, 'error')
    } finally {
      loading.value = false
    }
  }

  async function openResultModal(id: string | null) {
    if (!id) { resultModalOpen.value = false; resultDetail.value = null; return }
    const hit = lastHits.value.find(h => getIdString(h) === id)
    if (hit) {
      resultDetail.value = hit
      resultModalOpen.value = true
      return
    }
    if (!currentIndex.value) return
    loading.value = true
    try {
      resultDetail.value = await api.fetchById(getHost(), getApiKey(), currentIndex.value, id)
      resultModalOpen.value = true
    } catch {
      pushToast('未找到该记录', 'warning')
    } finally {
      loading.value = false
    }
  }

  function openResultModalByHit(hit: SearchHit | null) {
    if (!hit) {
      resultModalOpen.value = false
      resultDetail.value = null
      return
    }
    resultDetail.value = hit
    resultModalOpen.value = true
  }

  async function createIndex() {
    if (!newIndexUid.value.trim()) { pushToast('请提供索引 UID', 'error'); return }
    loading.value = true
    try {
      await api.createIndex(getHost(), getApiKey(), newIndexUid.value.trim(), newIndexPk.value.trim() || undefined)
      pushToast('索引创建成功', 'success')
      if (newIndexPk.value.trim()) primaryKeyField.value = newIndexPk.value.trim()
      newIndexUid.value = ''
      newIndexPk.value = ''
      await connect()
    } catch (e) {
      pushToast(`创建索引失败: ${e}`, 'error')
    } finally {
      loading.value = false
    }
  }

  function parseUploadData(data: string) {
    try {
      const items = JSON.parse(data)
      if (Array.isArray(items)) {
        uploadPreviewData.value = items
        uploadPreviewPage.value = 1
      }
    } catch (e) {
      pushToast(`JSON解析失败: ${e}`, 'error')
    }
  }

  async function batchImport() {
    if (!currentIndex.value) { pushToast('请先选择一个索引', 'error'); return }

    // NOTE: 根据导入模式决定数据来源
    let rawJson: string
    if (importMode.value === 'json') {
      if (!jsonTextInput.value.trim()) { pushToast('请先输入 JSON 数据', 'error'); return }
      rawJson = jsonTextInput.value.trim()
    } else {
      if (!uploadData.value.trim()) { pushToast('请先上传 JSON 文件', 'error'); return }
      rawJson = uploadData.value
    }

    loading.value = true
    uploadProgress.value = 40
    try {
      const msg = await api.batchImportDocuments(getHost(), getApiKey(), currentIndex.value, rawJson)
      uploadProgress.value = 100
      pushToast(msg, 'success')
      // 清理对应模式的数据
      if (importMode.value === 'json') {
        jsonTextInput.value = ''
        uploadPreviewData.value = []
        uploadPreviewPage.value = 1
      } else {
        uploadData.value = ''
        uploadFileName.value = ''
      }
      await performSearch()
    } catch (e) {
      pushToast(`批量导入失败: ${e}`, 'error')
    } finally {
      loading.value = false
      uploadProgress.value = 0
    }
  }

  async function exportCsv() {
    if (exportDownloading.value) return
    exportDownloading.value = true
    exportProgress.value = 0
    exportTotal.value = 0
    pushToast('开始导出 CSV...', 'warning')
    try {
      const params = buildSearchParams()
      const filename = await api.exportAllCsv(getHost(), getApiKey(), currentIndex.value, params, (p, t) => {
        exportProgress.value = p
        exportTotal.value = t
      })
      pushToast(`CSV 文件已导出: ${filename}`, 'success')
    } catch (e) {
      pushToast(`导出失败: ${e}`, 'error')
    } finally {
      exportDownloading.value = false
      exportProgress.value = 0
      exportTotal.value = 0
    }
  }

  function saveAsset() {
    let assets = storage.loadDeviceAssets()
    const idx = assets.findIndex(a => a.id === assetForm.value.id)
    if (idx >= 0) assets[idx] = { ...assetForm.value }
    else assets.push({ ...assetForm.value })
    storage.saveDeviceAssets(assets)
    assetList.value = assets
    assetModalOpen.value = false
    pushToast('设备保存成功', 'success')
  }

  function deleteAsset(id: string) {
    let assets = storage.loadDeviceAssets()
    assets = assets.filter(a => a.id !== id)
    storage.saveDeviceAssets(assets)
    assetList.value = assets
    pushToast('设备已删除', 'success')
  }

  function applySearchHistory(query: string) {
    searchInput.value = query
    performSearch()
  }

  function applyPopularSearch(value: string) {
    if (!popularSearchField.value) {
      searchInput.value = value
      performSearch()
      return
    }
    searchInput.value = ''
    queryRows.value = [createQueryRow()]
    queryRows.value[0].field = popularSearchField.value
    queryRows.value[0].operator = '='
    queryRows.value[0].value = value
    performSearch()
  }

  function saveViewConfig() {
    const name = viewNameInput.value.trim()
    if (!name) { pushToast('请输入视图名称', 'warning'); return }
    const columns = viewLayoutWorking.value.map(col => col.filter(f => f.trim()))
    const widths = normalizeViewWidths(viewWidthsWorking.value, columns.length)
    const labelWidths = normalizeLabelWidths(viewLabelWidthsWorking.value, columns.length)
    const cfg: ViewConfig = { name, columns, widths, labelWidths }
    const existing = viewConfigs.value.findIndex(c => c.name === name)
    if (existing >= 0) viewConfigs.value[existing] = cfg
    else viewConfigs.value.push(cfg)
    saveViewConfigs()
    viewMode.value = name
    saveViewMode()
    viewModalOpen.value = false
  }

  function openViewConfig() {
    viewModalOpen.value = true
    const cfg = activeViewConfig()
    if (cfg) {
      viewNameInput.value = cfg.name
      viewLayoutWorking.value = cfg.columns.map(c => [...c])
      viewWidthsWorking.value = [...cfg.widths]
      viewLabelWidthsWorking.value = [...cfg.labelWidths]
    } else {
      viewNameInput.value = ''
      viewLayoutWorking.value = [[], []]
      viewWidthsWorking.value = [0, 0]
      viewLabelWidthsWorking.value = [140, 140]
    }
  }

  function setAiEnabled(enabled: boolean) {
    aiConfig.value.aiEnabled = enabled
    storage.saveAiConfig(aiConfig.value)
    performSearch()
  }

  function setAiWeight(weight: number) {
    aiConfig.value.aiWeight = weight
    storage.saveAiConfig(aiConfig.value)
    performSearch()
  }

  function setCurrentTab(tab: CurrentTab) {
    currentTab.value = tab
    if (tab === 'assets') assetList.value = storage.loadDeviceAssets()
  }

  function startViewColumnResize(colIdx: number, startX: number, startWidthPx: number, containerWidthPx: number) {
    const widths = viewWidthsWorking.value.length ? [...viewWidthsWorking.value] : [0]
    viewResizeState.value = { colIdx, startX, startWidthPx, containerWidthPx, baseWidths: widths }

    const onMove = (e: MouseEvent) => {
      viewColumnResizeMove(e.clientX)
    }
    const onUp = () => {
      endViewColumnResize()
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }

  function viewColumnResizeMove(clientX: number) {
    const state = viewResizeState.value
    if (!state || state.containerWidthPx <= 0) return
    const delta = clientX - state.startX
    let newPx = state.startWidthPx + delta
    const minPx = 100
    newPx = Math.max(minPx, Math.min(state.containerWidthPx, newPx))
    let newPercent = Math.round((newPx / state.containerWidthPx) * 100)
    newPercent = Math.max(5, Math.min(95, newPercent))

    const widths = [...state.baseWidths]
    if (state.colIdx >= widths.length || widths.length <= 1) return

    const oldCurrent = widths[state.colIdx]
    const sumOthers = widths.reduce((a, b, i) => a + (i !== state.colIdx ? b : 0), 0)
    const remaining = 100 - newPercent

    if (sumOthers === 0) {
      const even = Math.floor(remaining / (widths.length - 1))
      let remain = remaining - even * (widths.length - 1)
      for (let i = 0; i < widths.length; i++) {
        if (i === state.colIdx) {
          widths[i] = newPercent
        } else {
          widths[i] = even + (remain > 0 ? 1 : 0)
          remain--
        }
      }
    } else {
      widths[state.colIdx] = newPercent
      for (let i = 0; i < widths.length; i++) {
        if (i === state.colIdx) continue
        widths[i] = Math.floor((widths[i] / sumOthers) * remaining)
      }
      let used = widths.reduce((a, b, i) => a + (i !== state.colIdx ? b : 0), 0)
      let remain = 100 - used
      let idx = 0
      while (remain > 0 && widths.length > 1) {
        if (idx !== state.colIdx) {
          widths[idx]++
          remain--
        }
        idx = (idx + 1) % widths.length
      }
    }
    viewWidthsWorking.value = widths
  }

  function endViewColumnResize() {
    viewResizeState.value = null
  }

  function startViewFieldResize(colIdx: number, startX: number, startWidthPx: number, containerWidthPx: number) {
    const widths = viewLabelWidthsWorking.value.length ? [...viewLabelWidthsWorking.value] : [140]
    viewFieldResizeState.value = { colIdx, startX, startWidthPx, containerWidthPx, baseWidths: widths }

    const onMove = (e: MouseEvent) => {
      viewFieldResizeMove(e.clientX)
    }
    const onUp = () => {
      endViewFieldResize()
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }

  function viewFieldResizeMove(clientX: number) {
    const state = viewFieldResizeState.value
    if (!state || state.containerWidthPx <= 0) return
    const delta = clientX - state.startX
    let newPx = state.startWidthPx + delta
    newPx = Math.max(60, Math.min(state.containerWidthPx, newPx))
    const widths = [...state.baseWidths]
    widths[state.colIdx] = Math.round(newPx)
    viewLabelWidthsWorking.value = widths
  }

  function endViewFieldResize() {
    viewFieldResizeState.value = null
  }

  return {
    hostInput, apiKeyInput, indexes, currentIndex, searchInput, queryRows,
    searchFields, searchFieldWeights, filterableFields, availableFields,
    highlightFields, displayFields, facets, searchHistory, fieldLabels,
    popularSearches, popularSearchField, currentPage, pageSize, maxResultsPerPage,
    resultsCount, processingTimeMs, facetDistribution, sortableAttributes,
    lastHits, lastResults, lastBaseColumns, tableSortField, tableSortDir,
    columnOrder, hiddenColumns, columnWidths, drawerFieldOrder, aiConfig, aiDropdownOpen,
    highlightEnabled, showRankingScore, cropLength, sortValue,
    filtersDrawerOpen, columnConfigOpen, fieldConfigOpen, viewModalOpen, advancedSettingsOpen,
    viewMode, viewNameInput, viewConfigs, exportDownloading, exportProgress,
    exportTotal, loading, toasts, resultModalOpen, resultDetail, editLocked,
    primaryKeyField, pendingEdits, imagePreviewEnabled, imagePreviewLinksOnly,
    imagePreviewSize, currentTab, assetForm, assetList, assetModalOpen,
    assetDetail, assetsLoading, uploadModalOpen, uploadData, uploadFileName,
    uploadPreviewData, uploadPreviewPage, uploadPreviewPageSize, uploadProgress,
    uploadLoading, importMode, jsonTextInput,
    newIndexUid, newIndexPk, visibleColumns, visibleAvailableFields, visibleFilterableFields, visibleFacetDistribution, totalPages,
    draggingCol, dragOverCol, isResizingColumns,
    viewLayoutWorking, viewWidthsWorking, viewLabelWidthsWorking,
    nestedFieldConfigs,
    startViewColumnResize, viewColumnResizeMove, endViewColumnResize,
    startViewFieldResize, viewFieldResizeMove, endViewFieldResize,
    viewDragField, viewDragFromColumn, viewDragColumn, viewDragOverIndex,
    pushToast, connect, selectIndex, resetIndexState,
    scheduleDebouncedSearch, performSearch, addQueryRow, removeQueryRow,
    clearQuery, addToHistory, clearHistory, loadPopularSearches, toggleFacet,
    toggleSearchField, toggleTableSort, goToPage, moveColumnOrder, setColumnWidth,
    saveColumnWidthPrefs, loadColumnWidthPrefs, loadColumnPrefs, saveColumnPrefs,
    loadFieldLabels, saveFieldLabels, loadViewConfigs, saveViewConfigs, saveViewMode,
    activeViewConfig, saveFieldConfig, toggleEditLock, updateCellEdit, saveEdits,
    openResultModal, openResultModalByHit, createIndex, parseUploadData, batchImport, exportCsv,
    saveAsset, deleteAsset, applySearchHistory, applyPopularSearch,
    saveViewConfig, openViewConfig, setAiEnabled, setAiWeight, setCurrentTab,
    currentFieldConfigsForSync,
    getNestedFieldConfig, setNestedFieldConfig, saveNestedFieldConfigs, scheduleSaveNestedFieldConfigs, saveDrawerFieldOrder,
  }
})
