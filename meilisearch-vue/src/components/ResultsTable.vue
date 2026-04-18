<template>
  <div class="results-wrapper">
    <!-- 空状态 -->
    <div v-if="!store.lastResults" class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
      <p>输入搜索关键词开始查询</p>
    </div>
    <div v-else-if="!(store.lastResults.hits && store.lastResults.hits.length)" class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
      <p>未找到匹配结果</p>
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
                  <span class="custom-value">{{ getFieldValue(hit, field) }}</span>
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
            <button class="btn btn-secondary" @click="store.openResultModal(getId(hit))">查看</button>
          </div>
        </div>
      </div>

      <!-- 表格模式 -->
      <div v-else class="table-wrap">
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
                <!-- 正常单元格 -->
                <template v-else>
                  <span v-html="getCellHtml(hit, col)" :title="getCellTitle(hit, col)"></span>
                </template>
                <!-- 评分显示（仅第一列） -->
                <template v-if="col === store.visibleColumns[0] && hit._rankingScore">
                  <br /><small class="rank-score">评分: {{ ((hit._rankingScore as number) * 100).toFixed(1) }}%</small>
                </template>
              </td>
              <td>
                <button class="btn btn-secondary btn-sm" @click="store.openResultModal(getId(hit))">查看</button>
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
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/composables/useApp'
import { sortHits, getIdString, getDocKey, valueToStringForEdit, valueToString, getCellValue } from '@/utils'
import type { SearchHit } from '@/types'

const store = useAppStore()

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

const sortedHits = computed(() => {
  const hits = store.lastHits ?? []
  if (!store.tableSortField || !store.visibleColumns.includes(store.tableSortField)) return hits
  return sortHits([...hits], store.tableSortField, store.tableSortDir)
})

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
.results-wrapper { }
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
.table-wrap { }
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
.edited-cell { background: rgba(var(--warning-color), 0.15); border-radius: 3px; padding: 2px 4px; font-style: italic; }
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
</style>
