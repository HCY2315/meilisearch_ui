<template>
  <div v-if="store.columnConfigOpen" class="modal-overlay" @click.self="store.columnConfigOpen = false">
    <div class="modal">
      <div class="modal-header">
        <h3>列配置</h3>
        <button class="modal-close" @click="store.columnConfigOpen = false">×</button>
      </div>
      <div class="modal-body">
        <div v-if="!isAdmin" class="modal-hint-warning">仅后台管理员可修改列配置与排序</div>
        <div
          v-for="col in orderedColumns"
          :key="col"
          class="column-config-item"
          :class="{ 'is-dragging': draggingColumn === col, 'is-drop-target': dragOverColumn === col }"
          :draggable="isAdmin"
          @dragstart="onDragStartColumn(col)"
          @dragover.prevent="onDragOverColumn(col)"
          @drop.prevent="onDropColumn(col)"
          @dragend="onDragEndColumn"
        >
          <label class="column-config-label">
            <span v-if="isAdmin" class="drag-handle" title="拖拽排序">⋮⋮</span>
            <input type="checkbox" v-model="hidden[col]" :disabled="!isAdmin" />
            <span class="column-name-original">{{ col }}</span>
            <input class="form-control" style="width:120px;font-size:12px" v-model="labels[col]" :placeholder="col" :disabled="!isAdmin" />
            <div class="column-order-actions">
              <button class="btn btn-secondary btn-sm" :disabled="!isAdmin || isFirstColumn(col)" @click.prevent="moveColumnUp(col)">↑</button>
              <button class="btn btn-secondary btn-sm" :disabled="!isAdmin || isLastColumn(col)" @click.prevent="moveColumnDown(col)">↓</button>
            </div>
          </label>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="store.columnConfigOpen = false">取消</button>
        <button v-if="isAdmin" class="btn btn-primary" @click="onSave">保存</button>
      </div>
    </div>
  </div>

  <div v-if="store.fieldConfigOpen" class="modal-overlay" @click.self="store.fieldConfigOpen = false">
    <div class="modal modal-lg">
      <div class="modal-header">
        <h3>字段配置</h3>
        <button class="modal-close" @click="store.fieldConfigOpen = false">×</button>
      </div>
      <div class="modal-body" style="max-height:70vh;overflow-y:auto">
        <div v-for="field in store.availableFields" :key="field" class="field-config">
          <div class="field-header">
            <h4>{{ field }}</h4>
            <label><input type="checkbox" v-model="fieldItems[field].searchable" /> 可搜索</label>
          </div>
          <div class="field-controls">
            <div class="form-group">
              <label>权重</label>
              <input type="number" class="form-control" v-model.number="fieldItems[field].weight" min="0" step="0.1" />
            </div>
            <div class="form-group">
              <label>列名</label>
              <input type="text" class="form-control" v-model="fieldItems[field].label" />
            </div>
            <div class="form-group">
              <label>高亮 <input type="checkbox" v-model="fieldItems[field].highlight" /></label>
            </div>
            <div class="form-group">
              <label>显示 <input type="checkbox" v-model="fieldItems[field].display" /></label>
            </div>
          </div>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="store.fieldConfigOpen = false">取消</button>
        <button class="btn btn-primary" @click="onSaveField">保存</button>
      </div>
    </div>
  </div>

  <div v-if="store.viewModalOpen" class="modal-overlay" @click.self="store.viewModalOpen = false">
    <div class="modal modal-lg">
      <div class="modal-header">
        <h3>视图配置</h3>
        <button class="modal-close" @click="store.viewModalOpen = false">×</button>
      </div>
      <div class="modal-body" style="max-height:70vh;overflow-y:auto">
        <div class="form-group">
          <label>视图名称</label>
          <input class="form-control" v-model="store.viewNameInput" placeholder="视图名称" />
        </div>
        <div class="form-group">
          <label>视图模式</label>
          <select class="form-control" v-model="store.viewMode">
            <option value="table">表格</option>
            <option v-for="cfg in store.viewConfigs" :key="cfg.name" :value="cfg.name">{{ cfg.name }}</option>
          </select>
        </div>
        <div class="view-columns">
          <div v-for="(col, ci) in store.viewLayoutWorking" :key="ci" class="view-column">
            <div class="view-title-row">
              <span>列 {{ ci + 1 }}</span>
              <button v-if="store.viewLayoutWorking.length > 1" class="btn btn-icon btn-sm" @click="removeColumn(ci)">删除</button>
            </div>
            <div
              class="view-column-body"
              @dragover.prevent
              @drop.prevent="onDropField(ci)"
            >
              <div
                v-for="field in col"
                :key="field"
                class="view-field-item view-field-selected"
                draggable="true"
                @dragstart="store.viewDragField = field; store.viewDragFromColumn = ci"
              >
                <span>{{ store.fieldLabels[field] || field }}</span>
              </div>
              <div v-if="!col.length" class="drop-hint">拖拽字段到这里</div>
            </div>
          </div>
          <button class="btn btn-secondary btn-sm" @click="addColumn">+ 添加列</button>
        </div>
        <div class="field-pool">
          <h4>可用字段</h4>
          <div class="pool-items">
            <div
              v-for="field in unusedFields"
              :key="field"
              class="view-field-item"
              draggable="true"
              @dragstart="store.viewDragField = field; store.viewDragFromColumn = null"
            >
              {{ store.fieldLabels[field] || field }}
            </div>
          </div>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="store.viewModalOpen = false">取消</button>
        <button class="btn btn-primary" @click="store.saveViewConfig()">保存</button>
      </div>
    </div>
  </div>

  <div v-if="store.advancedSettingsOpen" class="modal-overlay" @click.self="store.advancedSettingsOpen = false">
    <div class="modal">
      <div class="modal-header">
        <h3>⚙️ 高级搜索设置</h3>
        <button class="modal-close" @click="store.advancedSettingsOpen = false">×</button>
      </div>
      <div class="modal-body">
        <div class="adv-setting-group">
          <h4>显示控制</h4>
          <div class="adv-setting-item">
            <label class="inline-check">
              <input type="checkbox" v-model="store.highlightEnabled" @change="store.performSearch()" />
              启用高亮显示
            </label>
          </div>
          <div class="adv-setting-item">
            <label class="inline-check">
              <input type="checkbox" v-model="store.showRankingScore" @change="store.performSearch()" />
              显示评分
            </label>
          </div>
        </div>

        <div class="adv-setting-group">
          <h4>图片预览</h4>
          <div class="adv-setting-item">
            <label class="inline-check">
              <input type="checkbox" v-model="store.imagePreviewEnabled" @change="store.performSearch()" />
              启用图片预览
            </label>
          </div>
          <template v-if="store.imagePreviewEnabled">
            <div class="adv-setting-item">
              <label class="inline-check">
                <input type="checkbox" v-model="store.imagePreviewLinksOnly" @change="store.performSearch()" />
                缩略图模式
              </label>
            </div>
            <div v-if="store.imagePreviewLinksOnly" class="adv-setting-item">
              <div class="size-slider">
                <span>图片大小:</span>
                <input type="range" min="40" max="400" v-model.number="store.imagePreviewSize" @input="store.performSearch()" />
                <span>{{ store.imagePreviewSize }}px</span>
              </div>
            </div>
          </template>
        </div>

        <div class="adv-setting-group">
          <h4>内容显示</h4>
          <div class="adv-setting-item">
            <label class="inline-check">
              <span>内容截断长度:</span>
              <input type="number" class="form-control" style="width: 80px; padding: 6px 10px; margin-left: 8px;" v-model.number="store.cropLength" min="0" @change="store.performSearch()" />
            </label>
          </div>
          <div class="adv-setting-item">
            <label class="inline-check">
              <span>每页条数:</span>
              <select class="form-control" style="width: 100px; padding: 6px 10px; margin-left: 8px;" v-model.number="store.pageSize" @change="store.currentPage = 1; store.performSearch()">
                <option :value="10">10</option>
                <option :value="20">20</option>
                <option :value="50">50</option>
                <option :value="100">100</option>
                <option :value="500">500</option>
              </select>
            </label>
          </div>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-primary" @click="store.advancedSettingsOpen = false">确定</button>
      </div>
    </div>
  </div>

  <div v-if="store.resultModalOpen" class="modal-overlay" @click.self="store.resultModalOpen = false">
    <div class="modal modal-lg">
      <div class="modal-header">
        <h3>详情</h3>
        <button class="modal-close" @click="store.resultModalOpen = false">×</button>
      </div>
      <div class="modal-body">
        <div v-if="store.resultDetail" class="result-json-wrap">
          <div class="result-json-actions">
            <button class="btn btn-secondary btn-sm" @click="copyResultJson">复制 JSON</button>
          </div>
          <pre class="result-json">{{ fullResultJson }}</pre>
        </div>
        <div v-else class="result-detail">
          <div class="detail-row">
            <span>暂无详情</span>
          </div>
        </div>
      </div>
    </div>
  </div>

  <div v-if="store.tokenApplicationOpen" class="modal-overlay" @click.self="store.tokenApplicationOpen = false">
    <div class="modal">
      <div class="modal-header">
        <h3>申请访问凭证 (Token)</h3>
        <button class="modal-close" @click="store.tokenApplicationOpen = false">×</button>
      </div>
      <div class="modal-body" style="max-height: 75vh; overflow-y: auto;">
        <div class="form-group">
          <label>电子邮箱 (163 邮箱)</label>
          <div style="display:flex;gap:8px">
            <input type="email" class="form-control" v-model="store.applicationForm.email" placeholder="example@163.com" />
            <button class="btn btn-secondary btn-sm" :disabled="store.codeSending || store.codeCountdown > 0" @click="store.sendVerificationCode(store.applicationForm.email)">
              {{ store.codeSending ? '发送中...' : (store.codeCountdown > 0 ? `${store.codeCountdown}s` : (store.codeSent ? '重新发送' : '获取验证码')) }}
            </button>
          </div>
        </div>
        <div class="form-group">
          <label>验证码 (180分钟内有效)</label>
          <input type="text" class="form-control" v-model="store.applicationForm.code" placeholder="输入 6 位验证码" maxlength="6" />
        </div>
        <div class="form-group">
          <label>姓名</label>
          <input type="text" class="form-control" v-model="store.applicationForm.name" placeholder="您的真实姓名" />
        </div>
        <div class="form-group">
          <label>出生日期</label>
          <input type="date" class="form-control" v-model="store.applicationForm.birthday" />
        </div>
        <div class="form-group">
          <label>性别</label>
          <select class="form-control" v-model="store.applicationForm.gender">
            <option value="男">男</option>
            <option value="女">女</option>
            <option value="保密">保密</option>
          </select>
        </div>
        <div class="form-group">
          <label>申请用途</label>
          <textarea class="form-control" v-model="store.applicationForm.purpose" rows="3" placeholder="请简述申请 Token 的用途"></textarea>
        </div>
        <div class="form-group">
          <label>申请开通的索引 (多选)</label>
          <div class="index-checkboxes" style="display:grid; grid-template-columns: repeat(2, 1fr); gap: 8px; margin-top:8px">
            <label v-for="idx in store.indexes" :key="idx.uid" class="inline-check">
              <input type="checkbox" :value="idx.uid" v-model="store.applicationForm.allowIndexes" />
              {{ idx.displayName || idx.uid }}
            </label>
          </div>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="store.tokenApplicationOpen = false">取消</button>
        <button class="btn btn-primary" :disabled="store.applicationLoading" @click="store.submitApplication">
          {{ store.applicationLoading ? '提交中...' : '提交申请' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, watch, ref } from 'vue'
import { useAppStore } from '@/composables/useApp'
import type { FieldConfigItem } from '@/types'

const store = useAppStore()
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

const hidden = reactive<Record<string, boolean>>({})
const labels = reactive<Record<string, string>>({})
const columnOrderDraft = ref<string[]>([])
const draggingColumn = ref<string | null>(null)
const dragOverColumn = ref<string | null>(null)

watch(() => store.columnConfigOpen, (open) => {
  if (open) {
    const baseColumns = [...store.lastBaseColumns]
    const ordered: string[] = []
    if (store.columnOrder.length) {
      for (const c of store.columnOrder) {
        if (baseColumns.includes(c)) ordered.push(c)
      }
    }
    for (const c of baseColumns) {
      if (!ordered.includes(c)) ordered.push(c)
    }
    columnOrderDraft.value = ordered
    for (const c of ordered) {
      hidden[c] = !store.hiddenColumns.includes(c)
      labels[c] = store.fieldLabels[c] || c
    }
  }
})

const orderedColumns = computed(() => {
  return columnOrderDraft.value
})

function isFirstColumn(col: string): boolean {
  return orderedColumns.value.indexOf(col) === 0
}

function isLastColumn(col: string): boolean {
  return orderedColumns.value.indexOf(col) === orderedColumns.value.length - 1
}

function moveColumnUp(col: string) {
  const idx = orderedColumns.value.indexOf(col)
  if (idx <= 0) return
  const next = [...orderedColumns.value]
  ;[next[idx - 1], next[idx]] = [next[idx], next[idx - 1]]
  columnOrderDraft.value = next
}

function moveColumnDown(col: string) {
  const idx = orderedColumns.value.indexOf(col)
  if (idx < 0 || idx >= orderedColumns.value.length - 1) return
  const next = [...orderedColumns.value]
  ;[next[idx + 1], next[idx]] = [next[idx], next[idx + 1]]
  columnOrderDraft.value = next
}

function onDragStartColumn(col: string) {
  if (!isAdmin.value) return
  draggingColumn.value = col
  dragOverColumn.value = col
}

function onDragOverColumn(col: string) {
  if (!isAdmin.value || !draggingColumn.value) return
  dragOverColumn.value = col
}

function onDropColumn(targetCol: string) {
  if (!isAdmin.value || !draggingColumn.value) return
  const fromIdx = orderedColumns.value.indexOf(draggingColumn.value)
  const toIdx = orderedColumns.value.indexOf(targetCol)
  if (fromIdx < 0 || toIdx < 0 || fromIdx === toIdx) {
    onDragEndColumn()
    return
  }
  const next = [...orderedColumns.value]
  const [moved] = next.splice(fromIdx, 1)
  next.splice(toIdx, 0, moved)
  columnOrderDraft.value = next
  onDragEndColumn()
}

function onDragEndColumn() {
  draggingColumn.value = null
  dragOverColumn.value = null
}

async function onSave() {
  if (!isAdmin.value) return
  store.columnOrder = [...orderedColumns.value]
  store.hiddenColumns = store.lastBaseColumns.filter((c: string) => !hidden[c])
  for (const [k, v] of Object.entries(labels)) {
    if (v.trim() && v !== k) store.fieldLabels[k] = v.trim()
  }
  store.saveFieldLabels()
  store.saveColumnPrefs()
  await store.saveTableConfigsToBackend()
  store.columnConfigOpen = false
}

// Field config
const fieldItems = reactive<Record<string, FieldConfigItem>>({})

watch(() => store.fieldConfigOpen, (open) => {
  if (open) {
    for (const f of store.availableFields) {
      fieldItems[f] = {
        field: f,
        searchable: store.searchFields.includes(f) || !store.searchFields.length,
        weight: store.searchFieldWeights[f] ?? 1,
        label: store.fieldLabels[f] || f,
        highlight: store.highlightFields.includes(f) || !store.highlightFields.length,
        display: store.displayFields.includes(f) || !store.displayFields.length,
      }
    }
  }
})

function onSaveField() {
  store.saveFieldConfig(Object.values(fieldItems))
  store.fieldConfigOpen = false
}

// View config
const usedFields = computed(() => new Set(store.viewLayoutWorking.flat()))

const unusedFields = computed(() =>
  store.availableFields.filter((f: string) => !usedFields.value.has(f))
)

function addColumn() {
  store.viewLayoutWorking.push([])
  store.viewWidthsWorking.push(0)
  store.viewLabelWidthsWorking.push(140)
}

function removeColumn(ci: number) {
  if (store.viewLayoutWorking.length <= 1) return
  store.viewLayoutWorking.splice(ci, 1)
  store.viewWidthsWorking.splice(ci, 1)
  store.viewLabelWidthsWorking.splice(ci, 1)
}

function onDropField(targetCol: number) {
  const field = store.viewDragField
  if (!field) return
  // Remove from all columns
  for (const col of store.viewLayoutWorking) {
    const idx = col.indexOf(field)
    if (idx >= 0) col.splice(idx, 1)
  }
  // Add to target
  const insertAt = store.viewDragOverIndex ?? store.viewLayoutWorking[targetCol].length
  store.viewLayoutWorking[targetCol].splice(insertAt, 0, field)
  store.viewDragField = null
  store.viewDragFromColumn = null
  store.viewDragOverIndex = null
}

const fullResultJson = computed(() => {
  if (!store.resultDetail) return ''
  try {
    return JSON.stringify(store.resultDetail, null, 2)
  } catch {
    return String(store.resultDetail)
  }
})

async function copyResultJson() {
  if (!fullResultJson.value) return
  try {
    await navigator.clipboard.writeText(fullResultJson.value)
    store.pushToast('完整 JSON 已复制', 'success')
  } catch {
    store.pushToast('复制失败，请手动复制', 'warning')
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(3, 7, 18, 0.85);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(8px);
}
.modal {
  background: var(--glass-bg);
  border: 1px solid var(--border);
  border-radius: 20px;
  width: 520px;
  max-width: 90vw;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-lg);
  animation: modalIn 0.3s cubic-bezier(0.23, 1, 0.32, 1);
}

@keyframes modalIn {
  from { opacity: 0; transform: scale(0.95) translateY(20px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}
.modal-lg { width: 700px; }
.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}
.modal-body { flex: 1; overflow-y: auto; padding: 20px; }
.modal-footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
}
.column-config-item {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
  padding: 8px 10px;
  margin-bottom: 8px;
  transition: border-color 0.15s ease, background 0.15s ease, opacity 0.15s ease;
}
.column-config-item.is-drop-target {
  border-color: var(--primary-color);
  background: rgba(var(--primary-color-rgb), 0.08);
}
.column-config-item.is-dragging {
  opacity: 0.6;
}
.column-config-item label { display: flex; align-items: center; gap: 8px; margin-bottom: 0; font-size: 13px; }
.column-config-label { justify-content: space-between; }
.column-order-actions { display: flex; gap: 6px; margin-left: auto; }
.drag-handle {
  font-size: 14px;
  color: var(--text-muted);
  cursor: grab;
  user-select: none;
}
.modal-hint-warning {
  margin-bottom: 12px;
  padding: 8px 10px;
  border: 1px solid rgba(var(--warning-color-rgb, 245, 158, 11), 0.25);
  background: rgba(var(--warning-color-rgb, 245, 158, 11), 0.08);
  color: var(--text-secondary);
  border-radius: 8px;
  font-size: 12px;
}
.column-name-original { color: var(--text-secondary); min-width: 100px; }
.field-config {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 12px;
}
.field-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.field-header h4 { margin: 0; font-size: 14px; }
.field-controls { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
.view-columns { display: flex; gap: 12px; flex-wrap: wrap; margin: 16px 0; }
.view-column { flex: 1; min-width: 160px; background: var(--surface); border-radius: 8px; padding: 8px; }
.view-title-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; font-weight: 600; font-size: 13px; }
.view-column-body { min-height: 80px; display: flex; flex-direction: column; gap: 4px; }
.view-field-item {
  padding: 4px 8px;
  background: var(--bg-secondary);
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  border: 1px solid var(--border);
}
.view-field-selected { background: rgba(var(--primary-color-rgb), 0.1); border-color: var(--primary-color); }
.drop-hint { text-align: center; color: var(--text-muted); font-size: 12px; padding: 8px; }
.field-pool { margin-top: 16px; }
.field-pool h4 { font-size: 13px; margin-bottom: 8px; }
.pool-items { display: flex; flex-wrap: wrap; gap: 4px; }
.result-detail { line-height: 1.8; }
.detail-row { display: flex; gap: 8px; margin-bottom: 8px; font-size: 13px; }
.detail-row strong { min-width: 120px; color: var(--text-primary); }
.detail-row span { color: var(--text-secondary); word-break: break-all; }
.result-json-wrap { display: flex; flex-direction: column; gap: 10px; }
.result-json-actions { display: flex; justify-content: flex-end; }
.result-json {
  margin: 0;
  padding: 12px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: rgba(0, 0, 0, 0.25);
  color: var(--text-primary);
  max-height: 60vh;
  overflow: auto;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre;
}

.adv-setting-group {
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border);
}
.adv-setting-group:last-child {
  margin-bottom: 0;
  padding-bottom: 0;
  border-bottom: none;
}
.adv-setting-group h4 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 12px 0;
}
.adv-setting-item {
  margin-bottom: 10px;
}
.adv-setting-item:last-child { margin-bottom: 0; }
</style>
