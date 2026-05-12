<template>
  <div class="results-wrapper">
    <!-- 空状态 -->
    <div v-if="!hasResults" class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
      <p>输入搜索关键词开始查询</p>
    </div>
    <div v-else class="results-content">
      <!-- 视图模式 -->
      <div v-if="store.viewMode !== 'table' && store.activeViewConfig() && store.activeViewConfig()!.columns.length > 0" class="custom-results">
        <div v-for="hit in sortedHits" :key="getId(hit)" class="custom-row">
          <div class="custom-row-columns" :style="gridStyle">
            <div v-for="(colFields, ci) in store.activeViewConfig()!.columns" :key="ci" class="custom-col">
              <template v-for="field in colFields" :key="field">
                <div v-if="!store.hiddenColumns.includes(field)" class="custom-field">
                  <span class="custom-label" :style="getLabelWidthStyle(ci)">{{ store.fieldLabels[field] || field }}</span>
                  <span
                    class="field-width-resizer"
                    @mousedown.prevent.stop="onViewFieldResizeStart($event, ci)"
                  ></span>
                  <!-- NOTE: 嵌套字段在自定义视图中同样渲染徽章+查看按钮 -->
                  <span class="custom-value">
                    <template v-if="isNested(hit, field)">
                      <span class="nested-badge">{{ getNestedBadge(hit, field) }}</span>
                      <button class="btn-nested-view" @click="openDrawer(field, hit[field])">🔍 查看</button>
                    </template>
                    <span v-else v-html="getCellHtml(hit, field)" :title="getCellTitle(hit, field)"></span>
                  </span>
                </div>
              </template>
              <span
                v-if="ci < store.activeViewConfig()!.columns.length - 1"
                class="view-col-resizer"
                @mousedown.prevent.stop="onViewColResizeStart($event, ci)"
              ></span>
            </div>
          </div>
          <div class="custom-row-actions">
            <div v-if="hit._rankingScore" style="margin-bottom: 8px; text-align: center;">
              <small class="rank-score" style="display: block; opacity: 0.8;">相关度评分</small>
              <span class="rank-score" style="font-weight: bold; color: var(--primary-color);">{{ ((hit._rankingScore as number) * 100).toFixed(1) }}%</span>
            </div>
            <button class="btn btn-secondary" @click="store.openResultModalByHit(hit)">查看</button>
          </div>
        </div>
      </div>

      <!-- 表格模式 -->
      <div v-else class="table-wrap table-responsive">
        <table class="results-table">
          <colgroup>
            <col v-for="col in allCols" :key="col" :style="{ width: store.columnWidths[col] ? store.columnWidths[col] + 'px' : 'auto' }" />
          </colgroup>
          <thead>
            <tr>
              <th
                v-for="col in store.visibleColumns"
                :key="col"
                :class="thClass(col)"
                draggable="true"
                @dragstart="onDragStart($event, col)"
                @dragover.prevent="onDragOver($event, col)"
                @drop.prevent="onDrop(col)"
                @dragend="onDragEnd"
                @click="store.toggleTableSort(col)"
              >
                <span class="th-label">
                  <span v-if="store.primaryKeyField === col">🔑 </span>
                  {{ store.fieldLabels[col] || col }}
                  <span v-if="store.tableSortField === col">{{ store.tableSortDir === 'asc' ? '▲' : '▼' }}</span>
                </span>
                <span class="column-resizer" @mousedown.prevent.stop="startResize($event, col)"></span>
              </th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="hit in sortedHits" :key="getId(hit)">
              <td v-for="col in store.visibleColumns" :key="col">
                <!-- 行内编辑模式 -->
                <template v-if="isEditable(col, hit)">
                  <input
                    class="form-control cell-input"
                    :value="getEditValue(hit, col)"
                    @input="onCellInput(hit, col, ($event.target as HTMLInputElement).value)"
                  />
                </template>
                <!-- 已修改的单元格 -->
                <template v-else-if="getEditedValue(hit, col) !== undefined">
                  <span class="edited-cell" :title="getEditedValue(hit, col)">{{ getEditedValue(hit, col) }}</span>
                </template>
                <!-- 正常单元格（嵌套类型走查看按钮，原始类型正常渲染） -->
                <template v-else-if="isNested(hit, col)">
                  <span class="nested-badge">{{ getNestedBadge(hit, col) }}</span>
                  <button class="btn-nested-view" @click="openDrawer(col, hit[col])">🔍 查看</button>
                </template>
                <template v-else>
                  <span v-html="getCellHtml(hit, col)" :title="getCellTitle(hit, col)"></span>
                </template>
              </td>
              <td>
                <div v-if="hit._rankingScore" style="margin-bottom: 4px;">
                  <small class="rank-score">评分: {{ ((hit._rankingScore as number) * 100).toFixed(1) }}%</small>
                </div>
                <button class="btn btn-secondary btn-sm" @click="store.openResultModalByHit(hit)">查看</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- 分页 -->
      <div v-if="store.totalPages > 1" class="pagination">
        <button class="btn btn-secondary btn-sm" :disabled="store.currentPage === 1" @click="store.goToPage(store.currentPage - 1)">上一页</button>
        <button
          v-for="p in pageRange"
          :key="p"
          :class="['btn btn-sm', p === store.currentPage ? 'btn-primary' : 'btn-secondary']"
          @click="store.goToPage(p)"
        >{{ p }}</button>
        <button class="btn btn-secondary btn-sm" :disabled="store.currentPage >= store.totalPages" @click="store.goToPage(store.currentPage + 1)">下一页</button>
      </div>
    </div>
  </div>

  <!-- 嵌套数据侧抽屉 -->
  <NestedDataDrawer
    v-model="drawerOpen"
    :initial-data="drawerData"
    :initial-label="drawerLabel"
    :is-admin="isAdmin"
  />
</template>

<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import { useConnectionStore } from '@/composables/useConnectionStore'
import { useSearchStore } from '@/composables/useSearchStore'
import { useUIStore } from '@/composables/useUIStore'
import { useAppStore } from '@/composables/useApp'
import { sortHits, getIdString, getDocKey, valueToStringForEdit, valueToString, getCellValue, isNestedValue, getNestedBadgeText } from '@/utils'
import type { SearchHit } from '@/types'
import NestedDataDrawer from '@/components/NestedDataDrawer.vue'

const connectionStore = useConnectionStore()
const searchStore = useSearchStore()
const uiStore = useUIStore()

const appStore = useAppStore()
const store: any = new Proxy({}, {
  get(_target, prop: string) {
    const p = prop as keyof typeof store
    if (p in appStore) return (appStore as any)[p]
    if (p in searchStore) return (searchStore as any)[p]
    if (p in connectionStore) return (connectionStore as any)[p]
    if (p in uiStore) return (uiStore as any)[p]
    return undefined
  },
  set(_target, prop: string, value: any) {
    const p = prop as keyof typeof store
    if (p in appStore) { (appStore as any)[p] = value; return true }
    if (p in searchStore) {
      const propVal = (searchStore as any)[p]
      if (propVal && typeof propVal === 'object' && 'value' in propVal) (propVal as any).value = value
      else (searchStore as any)[p] = value
      return true
    }
    if (p in connectionStore) {
      const propVal = (connectionStore as any)[p]
      if (propVal && typeof propVal === 'object' && 'value' in propVal) (propVal as any).value = value
      else (connectionStore as any)[p] = value
      return true
    }
    if (p in uiStore) {
      const propVal = (uiStore as any)[p]
      if (propVal && typeof propVal === 'object' && 'value' in propVal) (propVal as any).value = value
      else (uiStore as any)[p] = value
      return true
    }
    return false
  }
})

onMounted(() => {
  console.log('[ResultsTable mounted] appStore.lastHits:', appStore.lastHits)
})

// Helper to get lastHits array properly
const lastHitsData = computed(() => {
  const hitsRef = (appStore as any).lastHits
  return (hitsRef && typeof hitsRef === 'object' && 'value' in hitsRef) ? hitsRef.value : hitsRef
})

// Helper to get lastResults and check if it has hits
const hasResults = computed(() => {
  const res = (appStore as any).lastResults
  const raw = (res && typeof res === 'object' && 'value' in res) ? res.value : res
  return raw && raw.hits && raw.hits.length > 0
})

const sortedHits = computed(() => {
  const arr = Array.isArray(lastHitsData.value) ? lastHitsData.value : []
  if (!store.tableSortField || !store.visibleColumns.includes(store.tableSortField)) return arr
  return sortHits([...arr], store.tableSortField, store.tableSortDir)
})

// 从 localStorage 获取用户角色
const isAdmin = computed(() => {
  const authUserStr = localStorage.getItem('authUser')
  if (!authUserStr) return false
  try {
    const authUser = JSON.parse(authUserStr)
    return authUser.role === 'admin'
  } catch {
    return false
  }
})

// NOTE: 嵌套数据抽屉的开关状态与当前打开的数据源
const drawerOpen = ref(false)
const drawerData = ref<unknown>(null)
const drawerLabel = ref('')

const allCols = computed(() => [...store.visibleColumns, '__action__'])

const getLabelWidthStyle = (ci: number): string => {
  const widths = store.viewLabelWidthsWorking
  const w = widths[ci] ?? 140
  return `min-width: ${w}px;`
}

function onViewColResizeStart(e: MouseEvent, colIdx: number) {
  const el = (e.target as HTMLElement).closest('.custom-row-columns') as HTMLElement
  if (!el) return
  const rect = el.getBoundingClientRect()
  const containerWidthPx = rect.width
  const currentPercent = store.activeViewConfig()?.widths[colIdx] ?? 0
  const startWidthPx = (containerWidthPx * currentPercent) / 100
  store.startViewColumnResize(colIdx, e.clientX, startWidthPx, containerWidthPx)
}

function onViewFieldResizeStart(e: MouseEvent, colIdx: number) {
  const el = (e.target as HTMLElement).closest('.custom-col') as HTMLElement
  if (!el) return
  const rect = el.getBoundingClientRect()
  const containerWidthPx = rect.width
  const currentWidth = store.viewLabelWidthsWorking[colIdx] ?? 140
  store.startViewFieldResize(colIdx, e.clientX, currentWidth, containerWidthPx)
}

const gridStyle = computed(() => {
  const activeCfg = store.activeViewConfig()
  if (!activeCfg) return ''
  const colCount = activeCfg.columns.length
  if (colCount === 0) return ''

  const widths = store.viewWidthsWorking.length > 0 ? store.viewWidthsWorking : (activeCfg.widths ?? [])
  
  // 如果有明确配置的有效宽度，使用百分比
  if (widths.length === colCount && widths.some((w: number) => w > 0)) {
    return `grid-template-columns: ${widths.map((w: number) => `${w}%`).join(' ')};`
  }
  
  // 否则均分宽度 (1fr)
  return `grid-template-columns: repeat(${colCount}, 1fr);`
})

const pageRange = computed(() => {
  const total = store.totalPages
  const cur = store.currentPage
  const start = Math.max(1, cur - 2)
  const end = Math.min(total, cur + 2)
  const range: number[] = []
  for (let i = start; i <= end; i++) range.push(i)
  return range
})

function getId(hit: SearchHit): string {
  return getIdString(hit)
}

function getFieldValue(hit: SearchHit, field: string): string {
  const val = hit[field]
  if (val === null || val === undefined) return ''
  return valueToString(val)
}

// NOTE: 检查原始字段值（非 _formatted 高亮版）是否为嵌套类型
function isNested(hit: SearchHit, col: string): boolean {
  return isNestedValue(hit[col])
}

// 获取嵌套字段的徽章文本
function getNestedBadge(hit: SearchHit, col: string): string {
  return getNestedBadgeText(hit[col])
}

// 打开嵌套数据抽屉
function openDrawer(label: string, data: unknown) {
  drawerLabel.value = label
  drawerData.value = data
  drawerOpen.value = true
}

function isEditable(col: string, hit: SearchHit): boolean {
  if (store.editLocked) return false
  const docId = getDocKey(hit, store.primaryKeyField)
  if (!docId) return false
  // 主键不可编辑
  if (store.primaryKeyField && col === store.primaryKeyField) return false
  return true
}

function getEditValue(hit: SearchHit, col: string): string {
  const docId = getDocKey(hit, store.primaryKeyField)
  const pending = store.pendingEdits[docId]
  const val = pending?.[col] ?? hit[col]
  return valueToStringForEdit(val)
}

function getEditedValue(hit: SearchHit, col: string): string | undefined {
  const docId = getDocKey(hit, store.primaryKeyField)
  const pending = store.pendingEdits[docId]
  if (pending && pending[col] !== undefined) {
    return valueToStringForEdit(pending[col])
  }
  return undefined
}

function onCellInput(hit: SearchHit, col: string, value: string) {
  const docId = getDocKey(hit, store.primaryKeyField)
  if (!docId) return
  store.updateCellEdit(docId, col, value)
}

function getCellHtml(hit: SearchHit, col: string): string {
  const cell = getCellValue(hit, col, store.highlightEnabled, store.imagePreviewEnabled, store.imagePreviewLinksOnly, store.imagePreviewSize)
  return cell?.html ?? ''
}

function getCellTitle(hit: SearchHit, col: string): string {
  const cell = getCellValue(hit, col, store.highlightEnabled, store.imagePreviewEnabled, store.imagePreviewLinksOnly, store.imagePreviewSize)
  return cell?.title ?? ''
}

function thClass(col: string): string {
  let cls = 'sortable-th'
  if (store.dragOverCol === col) cls += ' drag-over'
  if (store.primaryKeyField === col) cls += ' pk-col'
  return cls
}

function onDragStart(e: DragEvent, col: string) {
  store.draggingCol = col
  e.dataTransfer?.setData('text/plain', col)
}
function onDragOver(_e: DragEvent, col: string) {
  store.dragOverCol = col
}
function onDrop(col: string) {
  if (store.draggingCol && store.draggingCol !== col) {
    store.moveColumnOrder(store.draggingCol, col)
  }
  store.draggingCol = null
  store.dragOverCol = null
}
function onDragEnd() {
  store.draggingCol = null
  store.dragOverCol = null
}

function startResize(e: MouseEvent, col: string) {
  const resizeStartX = e.clientX
  const th = (e.target as HTMLElement).closest('th')
  const resizeStartWidth = th?.offsetWidth ?? 120
  store.isResizingColumns = true

  const onMove = (ev: MouseEvent) => {
    const delta = ev.clientX - resizeStartX
    const w = Math.max(80, resizeStartWidth + delta)
    store.setColumnWidth(col, w)
  }
  const onUp = () => {
    store.isResizingColumns = false
    store.saveColumnWidthPrefs()
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
  }
  window.addEventListener('mousemove', onMove)
  window.addEventListener('mouseup', onUp)
}
</script>

<style scoped>

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-muted);
  gap: 12px;
}
.empty-state svg { width: 48px; height: 48px; }

.results-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
  table-layout: fixed;
}
.results-table th {
  background: var(--bg-secondary);
  padding: 8px 10px;
  text-align: left;
  border-bottom: 2px solid var(--border);
  position: relative;
  user-select: none;
  white-space: nowrap;
  cursor: pointer;
}
.results-table td { padding: 6px 10px; border-bottom: 1px solid var(--border); vertical-align: top; word-break: break-word; }
.sortable-th.drag-over { background: rgba(var(--primary-color-rgb), 0.1); }
.pk-col { color: var(--primary-color); }
.th-label { display: flex; align-items: center; gap: 4px; }
.column-resizer {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 10px;
  cursor: col-resize;
  z-index: 10;
  transition: background 0.2s;
}
.column-resizer:hover, .column-resizer.active {
  background: rgba(var(--primary-color-rgb), 0.2);
}
.column-resizer::after {
  content: "";
  position: absolute;
  right: 4px;
  top: 20%;
  bottom: 20%;
  width: 2px;
  background: rgba(255, 255, 255, 0.3);
  border-radius: 1px;
  opacity: 0;
  transition: opacity 0.2s;
}
.column-resizer:hover::after {
  opacity: 1;
}
.cell-input { padding: 4px 6px; font-size: 12px; width: 100%; min-width: 60px; }
.edited-cell { background: rgba(var(--warning-color-rgb), 0.15); border-radius: 3px; padding: 2px 4px; font-style: italic; }
.rank-score { color: var(--text-muted); font-size: 11px; }
.custom-results { display: flex; flex-direction: column; gap: 12px; }
.custom-row { background: var(--surface); border-radius: var(--radius); padding: 12px; border: 1px solid var(--border); }
.custom-row-columns { display: grid; gap: 12px; }
.custom-col { display: flex; flex-direction: column; gap: 4px; position: relative; }
.custom-field { display: flex; gap: 8px; }
.custom-label { font-weight: 600; color: var(--text-secondary); flex-shrink: 0; }
.custom-value { color: var(--text-primary); word-break: break-word; }
.custom-row-actions { margin-top: 8px; }
.view-col-resizer {
  position: absolute;
  right: -6px;
  top: 0;
  bottom: 0;
  width: 12px;
  cursor: col-resize;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: center;
}
.view-col-resizer::after {
  content: '';
  width: 3px;
  height: 30px;
  background: var(--border);
  border-radius: 2px;
  transition: background 0.2s;
}
.view-col-resizer:hover::after { background: var(--primary-color); }
.field-width-resizer {
  position: absolute;
  right: -6px;
  top: 0;
  height: 12px;
  width: 12px;
  cursor: row-resize;
  z-index: 3;
}
.field-width-resizer::after {
  content: '';
  position: absolute;
  right: 3px;
  top: 0;
  width: 30px;
  height: 3px;
  background: var(--border);
  border-radius: 2px;
  transition: background 0.2s;
}
.field-width-resizer:hover::after { background: var(--primary-color); }
.pagination { display: flex; gap: 4px; justify-content: center; margin-top: 16px; flex-wrap: wrap; }

/* ─── 嵌套字段徽章 & 查看按钮 ─────────────────────────────────────────────── */
.nested-badge {
  display: inline-flex;
  align-items: center;
  padding: 1px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
  background: rgba(99,179,237,0.12);
  color: #63b3ed;
  vertical-align: middle;
  white-space: nowrap;
  margin-right: 4px;
}

.btn-nested-view {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 1px 8px;
  border-radius: 4px;
  border: 1px solid rgba(72, 187, 120, 0.3);
  background: rgba(72, 187, 120, 0.08);
  color: #48bb78;
  font-size: 11.5px;
  cursor: pointer;
  white-space: nowrap;
  vertical-align: middle;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}
.btn-nested-view:hover {
  background: rgba(72, 187, 120, 0.2);
  color: #38a169;
  border-color: rgba(72, 187, 120, 0.5);
}

/* ============ Mobile Responsiveness ============ */
@media (max-width: 768px) {
  .custom-row-columns {
    grid-template-columns: 1fr !important;
  }
}
</style>
