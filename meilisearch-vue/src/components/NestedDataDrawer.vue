<template>
  <Teleport to="body">
    <!-- 半透明遮罩 -->
    <Transition name="drawer-backdrop">
      <div v-if="modelValue" class="nd-backdrop" @click.self="handleClose" />
    </Transition>

    <!-- 抽屉主体 -->
    <Transition name="drawer-slide">
      <div v-if="modelValue" class="nd-drawer" role="dialog" aria-modal="true">

        <!-- ── 头部：面包屑 + 操作按钮 ── -->
        <div class="nd-header">
          <nav class="nd-breadcrumb" aria-label="嵌套路径">
            <button class="bc-root" @click="handleClose" title="关闭，返回主表格">
              ← 返回
            </button>
            <template v-for="(frame, i) in stack" :key="i">
              <span class="bc-sep" aria-hidden="true">›</span>
              <button
                class="bc-item"
                :class="{ 'bc-current': i === stack.length - 1 }"
                :disabled="i === stack.length - 1"
                @click="jumpTo(i)"
                :title="frame.label"
              >{{ frame.label }}</button>
            </template>
          </nav>
          <div class="nd-header-actions">
            <!-- 字段配置切换按钮（纯数组无字段配置） -->
            <button
              v-if="!isPureArray && isAdmin"
              class="btn-config-toggle"
              :class="{ active: configOpen }"
              @click="configOpen = !configOpen"
              title="配置字段可见性和别名"
            >⚙ 配置字段</button>
            <button class="nd-close-btn" @click="handleClose" aria-label="关闭">✕</button>
          </div>
        </div>

        <!-- ── 子头部：类型徽章 + 隐藏字段提示 ── -->
        <div class="nd-subheader">
          <div class="subheader-left">
            <span v-if="isArrayOfObjects" class="type-badge badge-array">
              🗂 对象数组 · {{ (currentData as unknown[]).length }} 条
            </span>
            <span v-else-if="isPureArray" class="type-badge badge-prim-array">
              📋 基础数组 · {{ (currentData as unknown[]).length }} 条
            </span>
            <span v-else class="type-badge badge-object">
              📦 对象 · {{ objectEntries.length }} 个字段
            </span>
            <span v-if="hiddenCount > 0" class="hidden-hint">（已隐藏 {{ hiddenCount }} 个字段）</span>
          </div>
          <div class="subheader-right">
            <span class="path-hint" :title="currentPathLabel">{{ currentPathLabel }}</span>
          </div>
        </div>

        <!-- ── 字段配置面板（可折叠） ── -->
        <Transition name="config-panel">
          <div v-if="configOpen && allConfigFields.length > 0" class="config-panel">
            <div class="config-panel-hd">
              <div class="config-panel-title-wrap">
                <span class="config-panel-title">字段配置</span>
                <span class="config-panel-desc">显示/隐藏、别名、拖拽排序</span>
              </div>
              <button
                v-if="isAdmin"
                class="btn-save-config"
                :disabled="!hasPendingSave && !hasOrderChange"
                @click="saveAllConfig"
              >
                保存配置
              </button>
            </div>
            <div class="config-list">
              <div
                v-for="field in sortedConfigFields"
                :key="field"
                class="config-row"
                :class="{ 'is-hidden-row': !isFieldVisible(field) }"
                draggable="true"
                @dragstart="onDragStartConfig(sortedConfigFields.indexOf(field))"
                @dragover="onDragOverConfig($event, sortedConfigFields.indexOf(field))"
                @dragend="onDragEndConfig"
              >
                <!-- 拖拽手柄 -->
                <span class="drag-handle">⋮⋮</span>
                <!-- 可见性切换 -->
                <button
                  class="vis-btn"
                  :class="{ 'vis-off': !isFieldVisible(field) }"
                  @click="toggleVisible(field)"
                  :title="isFieldVisible(field) ? '点击隐藏此字段' : '点击显示此字段'"
                >
                  <span v-if="isFieldVisible(field)">👁</span>
                  <span v-else>🙈</span>
                </button>
                <!-- 字段名 -->
                <span class="config-field-name" :title="field">{{ field }}</span>
                <span class="config-arrow">→</span>
                <!-- 别名输入框 -->
                <input
                  class="alias-input"
                  :value="getFieldAlias(field)"
                  :placeholder="field"
                  :disabled="!isFieldVisible(field)"
                  @input="setAlias(field, ($event.target as HTMLInputElement).value)"
                />
              </div>
            </div>
          </div>
        </Transition>

        <!-- ── 主体内容区 ── -->
        <div class="nd-body">

          <!-- ① 对象数组 → 迷你表格 -->
          <div v-if="isArrayOfObjects" class="mini-table-wrap">
            <div v-if="visibleArrayColumns.length === 0" class="all-hidden-tip">
              所有字段已隐藏，请在「配置字段」中开启至少一个字段
            </div>
            <table v-else class="mini-table">
              <thead>
                <tr>
                  <th class="row-num-th">#</th>
                  <th v-for="col in visibleArrayColumns" :key="col" :title="col">
                    {{ getFieldAlias(col) || col }}
                    <span v-if="getFieldAlias(col)" class="alias-tag" :title="`原始字段：${col}`">{{ col }}</span>
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(row, ri) in (currentData as Record<string, unknown>[])" :key="ri">
                  <td class="row-num">{{ ri + 1 }}</td>
                  <td v-for="col in visibleArrayColumns" :key="col">
                    <div class="cell-with-preview">
                      <template v-if="isNestedValue(row[col])">
                        <span class="nested-badge-sm">{{ getNestedBadgeText(row[col]) }}</span>
                        <span class="nested-text">
                          {{ primitiveToStr(row[col]) }}
                        </span>
                        <button
                          class="btn-drill"
                          @click="push(`${currentPathLabel}[${ri}].${col}`, row[col])"
                        >🔍</button>
                      </template>
                      <template v-else>
                        <!-- 对象/数组：只显示放大镜，悬停显示预览 -->
                        <span 
                          v-if="isObjectOrArray(row[col])"
                          class="btn-preview" 
                          @mouseenter="showPreview($event, col, row[col])" 
                          @mouseleave="hidePreview"
                        >🔎</span>
                        <!-- 原始类型：显示值（支持缩略图） -->
                        <span 
                          v-else
                          class="cell-text"
                          :title="primitiveToStr(row[col])"
                          v-html="renderCell(row, col)"
                        ></span>
                      </template>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- ② 基础数组（元素全为原始类型） -->
          <div v-else-if="isPureArray" class="prim-array-list">
            <div
              v-for="(item, i) in (currentData as unknown[])"
              :key="i"
              class="prim-array-item"
            >
              <span class="prim-idx">#{{ i + 1 }}</span>
              <span class="prim-val">{{ primitiveToStr(item) }}</span>
            </div>
          </div>

          <!-- ③ 单个对象 → 键值对列表 -->
          <div v-else class="kv-list">
            <div v-if="visibleObjectEntries.length === 0" class="all-hidden-tip">
              所有字段已隐藏，请在「配置字段」中开启至少一个字段
            </div>
            <div
              v-for="[key, val] in visibleObjectEntries"
              :key="key"
              class="kv-row"
            >
              <span class="kv-key" :title="key">
                {{ getFieldAlias(key) || key }}
                <span v-if="getFieldAlias(key)" class="alias-raw-tag" :title="`原始字段：${key}`">{{ key }}</span>
              </span>
              <span class="kv-val">
                <template v-if="isNestedValue(val)">
                  <span class="nested-badge-sm">{{ getNestedBadgeText(val) }}</span>
                  <button
                    class="btn-drill"
                    @click="push(`${currentPathLabel}.${key}`, val)"
                  >🔍 查看</button>
                </template>
                <template v-else>
                  <!-- 对象/数组：只显示放大镜，悬停显示预览 -->
                  <span v-if="isObjectOrArray(val)" class="btn-preview" @mouseenter="showPreview($event, key, val)" @mouseleave="hidePreview">🔎</span>
                  <!-- 原始类型：显示值 -->
                  <span v-else class="val-text" :title="primitiveToStr(val)">{{ primitiveToStr(val) }}</span>
                </template>
              </span>
            </div>
          </div>

        </div>
      </div>
    </Transition>
    
    <!-- 字段预览弹窗 -->
    <div v-if="previewData" class="preview-popup" :style="{ left: previewData.x + 10 + 'px', top: previewData.y + 10 + 'px' }">
      <div class="preview-header">{{ previewData.field }}</div>
      <pre class="preview-content">{{ formatPreviewValue(previewData.value) }}</pre>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useAppStore } from '@/composables/useApp'
import type { NestedFieldConfigItem } from '@/types'
import { isNestedValue, getNestedBadgeText, getCellValue } from '@/utils'

const appStore = useAppStore()

// 高级设置状态（从 appStore 获取）
const imagePreviewEnabled = computed(() => appStore.imagePreviewEnabled)
const imagePreviewLinksOnly = computed(() => appStore.imagePreviewLinksOnly)
const imagePreviewSize = computed(() => appStore.imagePreviewSize)

// ─── 辅助函数 ───────────────────────────────────────────────────────────────
function primitiveToStr(val: unknown): string {
  if (val === null) return 'null'
  if (val === undefined) return ''
  if (typeof val === 'string') return val
  if (typeof val === 'number' || typeof val === 'boolean') return String(val)
  return JSON.stringify(val)
}

function isObjectOrArray(val: unknown): boolean {
  if (val !== null && typeof val === 'object') return true
  return false
}

// 渲染单元格（支持缩略图模式）
function renderCell(row: Record<string, unknown>, col: string) {
  if (imagePreviewEnabled.value) {
    // 构建临时的 hit 对象用于 getCellValue
    const fakeHit = { ...row } as any
    const cell = getCellValue(
      fakeHit,
      col,
      false,  // highlightEnabled
      imagePreviewEnabled.value,
      imagePreviewLinksOnly.value,
      imagePreviewSize.value
    )
    if (cell?.html) return cell.html
  }
  // 默认渲染
  return primitiveToStr(row[col])
}

// ─── 类型定义 ─────────────────────────────────────────────────────────────────

interface NestedFrame {
  label: string    // 面包屑显示名
  data: unknown    // 该层数据
}

// ─── Props & Emits ───────────────────────────────────────────────────────────

const props = defineProps<{
  modelValue: boolean
  initialData: unknown
  initialLabel: string
  isAdmin: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', val: boolean): void
}>()
const store = useAppStore()

// ─── 导航栈 ──────────────────────────────────────────────────────────────────

const stack = ref<NestedFrame[]>([])
const configOpen = ref(false)
const hasPendingSave = ref(false)
  const draftConfigs = ref<Record<string, Record<string, NestedFieldConfigItem>>>({})

  // 抽屉字段顺序
  const fieldOrder = ref<string[]>([])
  const hasOrderChange = ref(false)
  const draggedIndex = ref<number | null>(null)
  
  // 配置面板中的字段顺序（与 fieldOrder 同步）
  const configOrder = ref<string[]>([])
  const configDragIndex = ref<number | null>(null)

  // 监听配置面板打开时，同步 fieldOrder 到 configOrder
  watch(() => configOpen.value, (open) => {
    if (open && fieldOrder.value.length > 0) {
      configOrder.value = [...fieldOrder.value]
    }
  })

// 字段预览
const previewData = ref<{ x: number, y: number, field: string, value: unknown } | null>(null)

// 当抽屉打开时，重置导航栈到初始层并加载对应的字段配置
watch(
  () => [props.modelValue, props.initialData, props.initialLabel] as const,
  ([open]) => {
    if (open) {
      stack.value = [{ label: props.initialLabel, data: props.initialData }]
      draftConfigs.value = {}
      hasPendingSave.value = false
      loadConfig()
    }
  },
  { immediate: true }
)

// ─── 字段配置面板 ─────────────────────────────────────────────────────────────

// 当前层的字段配置（可见性 + 别名），会随导航路径自动切换
const config = ref<Record<string, NestedFieldConfigItem>>({})

// 当前层的路径标签（面包屑最后一段）
const currentPathLabel = computed<string>(() => {
  if (!stack.value.length) return ''
  return stack.value[stack.value.length - 1].label
})

function loadConfig() {
  const pathKey = normalizePath(currentPathLabel.value)
  const draft = draftConfigs.value[pathKey]
  if (draft) {
    config.value = { ...draft }
    return
  }
  const serverConfig = store.getNestedFieldConfig(pathKey)
  config.value = { ...serverConfig }
}

function normalizePath(label: string): string {
  // 不替换数组索引，保持原始 key 匹配数据库配置
  return label
}

function markDirtyAndSaveDraft(nextConfig: Record<string, NestedFieldConfigItem>) {
  const pathKey = normalizePath(currentPathLabel.value)
  config.value = nextConfig
  draftConfigs.value = {
    ...draftConfigs.value,
    [pathKey]: { ...nextConfig },
  }
  hasPendingSave.value = true
}

async function saveConfig() {
  if (!hasPendingSave.value) return
  for (const [pathKey, cfg] of Object.entries(draftConfigs.value)) {
    store.setNestedFieldConfig(pathKey, cfg)
  }
  await store.saveNestedFieldConfigs()
  draftConfigs.value = {}
  hasPendingSave.value = false
  store.pushToast('嵌套字段配置已保存', 'success')
}

async function saveAllConfig() {
  // 保存字段配置（可见性+别名）
  if (hasPendingSave.value) {
    for (const [pathKey, cfg] of Object.entries(draftConfigs.value)) {
      store.setNestedFieldConfig(pathKey, cfg)
    }
    await store.saveNestedFieldConfigs()
    draftConfigs.value = {}
    hasPendingSave.value = false
  }
  // 保存字段排序
  if (hasOrderChange.value && props.isAdmin) {
    await saveFieldOrder()
  } else if (hasOrderChange.value && !props.isAdmin) {
    saveFieldOrderLocal()
  }
  store.pushToast('字段配置已保存', 'success')
}

// 导航路径变化时自动切换配置
watch(currentPathLabel, () => {
  loadConfig()
  loadFieldOrder()
})

function loadFieldOrder() {
  const fields = allConfigFields.value
  if (!fields || fields.length === 0) {
    fieldOrder.value = []
    return
  }
  const pathKey = normalizePath(currentPathLabel.value)
  // 直接从 appStore 获取原始值，避免 Proxy 干扰
  const appStore = useAppStore()
  const dfo = appStore.drawerFieldOrder
  const dfoObj = dfo && typeof dfo === 'object' ? dfo : {}

  // 优先从数据库配置加载，否则使用字段默认顺序
  const saved = dfoObj[pathKey] as string[] | undefined
  if (Array.isArray(saved) && saved.length > 0) {
    fieldOrder.value = [...saved]
  } else if (!props.isAdmin) {
    const localKey = `drawerFieldOrder:${store.currentIndex}:${pathKey}`
    const localStored = localStorage.getItem(localKey)
    if (localStored) {
      try {
        fieldOrder.value = JSON.parse(localStored)
      } catch {
        fieldOrder.value = [...fields]
      }
    } else {
      fieldOrder.value = [...fields]
    }
  } else {
    fieldOrder.value = [...fields]
  }
}

function saveFieldOrderLocal() {
  if (!props.isAdmin) {
    const pathKey = normalizePath(currentPathLabel.value)
    const localKey = `drawerFieldOrder:${store.currentIndex}:${pathKey}`
    localStorage.setItem(localKey, JSON.stringify(fieldOrder.value))
  }
}

function onDragStart(index: number) {
  draggedIndex.value = index
}

function onDragOver(e: DragEvent, index: number) {
  e.preventDefault()
  if (draggedIndex.value === null || draggedIndex.value === index) return
  const items = [...fieldOrder.value]
  const item = items[draggedIndex.value]
  items.splice(draggedIndex.value, 1)
  items.splice(index, 0, item)
  fieldOrder.value = items
  draggedIndex.value = index
  hasOrderChange.value = true
}

function onDragEnd() {
  if (!props.isAdmin && hasOrderChange.value) {
    saveFieldOrderLocal()
  }
  draggedIndex.value = null
}

// 配置面板拖拽排序
function onDragStartConfig(index: number) {
  configDragIndex.value = index
}

function onDragOverConfig(e: DragEvent, index: number) {
  e.preventDefault()
  if (configDragIndex.value === null || configDragIndex.value === index) return
  const items = [...configOrder.value]
  const item = items[configDragIndex.value]
  items.splice(configDragIndex.value, 1)
  items.splice(index, 0, item)
  configOrder.value = items
  configDragIndex.value = index
  hasOrderChange.value = true
}

function onDragEndConfig() {
  // 更新 fieldOrder
  fieldOrder.value = [...configOrder.value]
  if (!props.isAdmin && hasOrderChange.value) {
    saveFieldOrderLocal()
  }
  configDragIndex.value = null
}

let hideTimer: ReturnType<typeof setTimeout> | null = null

function showPreview(e: MouseEvent, field: string, value: unknown) {
  if (hideTimer) clearTimeout(hideTimer)
  previewData.value = {
    x: e.clientX,
    y: e.clientY,
    field: getFieldAlias(field) || field,
    value
  }
}

function hidePreview() {
  hideTimer = setTimeout(() => {
    previewData.value = null
  }, 200)
}

function formatPreviewValue(val: unknown): string {
  if (val === null || val === undefined) return 'null'
  if (typeof val === 'string') return val
  if (typeof val === 'number' || typeof val === 'boolean') return String(val)
  if (Array.isArray(val)) return JSON.stringify(val, null, 2)
  if (typeof val === 'object') return JSON.stringify(val, null, 2)
  return String(val)
}

async function saveFieldOrder() {
  const pathKey = normalizePath(currentPathLabel.value)
  const appStore = useAppStore()
  const dfo = appStore.drawerFieldOrder
  const dfoObj = dfo && typeof dfo === 'object' ? dfo : {}
  dfoObj[pathKey] = [...fieldOrder.value]
  appStore.drawerFieldOrder = { ...dfoObj }
  await appStore.saveDrawerFieldOrder()
  hasOrderChange.value = false
  store.pushToast('抽屉字段顺序已保存', 'success')
}

// ─── 字段配置 API ─────────────────────────────────────────────────────────────

function isFieldVisible(field: string): boolean {
  // 默认可见，只有明确设置 visible: false 时才隐藏
  return config.value[field]?.visible !== false
}

function getFieldAlias(field: string): string {
  return config.value[field]?.alias || ''
}

function toggleVisible(field: string) {
  const current = isFieldVisible(field)
  const nextConfig = {
    ...config.value,
    [field]: { visible: !current, alias: config.value[field]?.alias || '' }
  }
  markDirtyAndSaveDraft(nextConfig)
}

function setAlias(field: string, alias: string) {
  const nextConfig = {
    ...config.value,
    [field]: { visible: isFieldVisible(field), alias }
  }
  markDirtyAndSaveDraft(nextConfig)
}

// ─── 当前层数据 ──────────────────────────────────────────────────────────────

const currentData = computed<unknown>(() => {
  if (!stack.value.length) return null
  return stack.value[stack.value.length - 1].data
})

const isArrayOfObjects = computed(() => {
  const d = currentData.value
  return Array.isArray(d) && d.length > 0 && d[0] !== null && typeof d[0] === 'object'
})

const isPureArray = computed(() => {
  const d = currentData.value
  return Array.isArray(d) && !isArrayOfObjects.value
})

// 对象数组的所有列名（含在各行中分布不均匀的列，取并集）
const arrayColumns = computed<string[]>(() => {
  if (!isArrayOfObjects.value) return []
  const keySet = new Set<string>()
  for (const item of (currentData.value as Record<string, unknown>[])) {
    if (item && typeof item === 'object') {
      Object.keys(item).forEach(k => keySet.add(k))
    }
  }
  return Array.from(keySet)
})

// 对象模式的所有键值对
const objectEntries = computed<[string, unknown][]>(() => {
  const d = currentData.value
  if (d === null || Array.isArray(d) || typeof d !== 'object') return []
  return Object.entries(d as object)
})

// ── 配置面板展示的全量字段（含已隐藏），用于开关和别名设置 ──
const allConfigFields = computed<string[]>(() => {
  if (isArrayOfObjects.value) return arrayColumns.value
  if (isPureArray.value) return []  // 纯数组无字段维度
  return objectEntries.value.map(([k]) => k)
})

// 配置面板中排序后的字段列表
const sortedConfigFields = computed<string[]>(() => {
  const order = configOpen.value && configOrder.value.length > 0 
    ? configOrder.value 
    : fieldOrder.value.length > 0 
      ? fieldOrder.value 
      : null
  
  if (!order) return allConfigFields.value
  
  const fields = [...allConfigFields.value]
  return fields.sort((a, b) => {
    const idxA = order.indexOf(a)
    const idxB = order.indexOf(b)
    if (idxA === -1) return 1
    if (idxB === -1) return -1
    return idxA - idxB
  })
})

// 过滤掉被隐藏字段后的可见列（用于实际渲染），按 fieldOrder 排序
const visibleArrayColumns = computed(() => {
  const ordered = fieldOrder.value.length > 0 ? fieldOrder.value : arrayColumns.value
  return ordered.filter(col => isFieldVisible(col))
})

const visibleObjectEntries = computed(() => {
  const orderedFields = fieldOrder.value.length > 0 ? fieldOrder.value : objectEntries.value.map(([k]) => k)
  const filtered = objectEntries.value.filter(([k]) => isFieldVisible(k))
  return filtered.sort((a, b) => {
    const idxA = orderedFields.indexOf(a[0])
    const idxB = orderedFields.indexOf(b[0])
    if (idxA === -1 && idxB === -1) return 0
    if (idxA === -1) return 1
    if (idxB === -1) return -1
    return idxA - idxB
  })
})

// 隐藏字段数量，用于提示
const hiddenCount = computed(() =>
  allConfigFields.value.filter(f => !isFieldVisible(f)).length
)

// ─── 导航操作 ─────────────────────────────────────────────────────────────────

function push(label: string, data: unknown) {
  stack.value = [...stack.value, { label, data }]
  // watch(currentPathLabel) 会自动触发 loadConfig
}

function jumpTo(index: number) {
  if (index < 0 || index >= stack.value.length - 1) return
  stack.value = stack.value.slice(0, index + 1)
}

function handleClose() {
  emit('update:modelValue', false)
}
</script>

<style scoped>
/* ─── 背景遮罩 ──────────────────────────────────────────────────────────────── */
.nd-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 1000;
}

/* ─── 抽屉主体 ──────────────────────────────────────────────────────────────── */
.nd-drawer {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: 80%;
  min-width: 400px;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary, #0f1117);
  border-left: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: -12px 0 48px rgba(0, 0, 0, 0.55);
  z-index: 1001;
  overflow: hidden;
}

/* ─── 头部 ──────────────────────────────────────────────────────────────────── */
.nd-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(255, 255, 255, 0.025);
  flex-shrink: 0;
}

.nd-breadcrumb {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
  overflow: hidden;
  min-width: 0;
}

.bc-root {
  background: transparent;
  border: none;
  color: var(--primary-color);
  font-size: 13px;
  cursor: pointer;
  padding: 3px 8px;
  border-radius: 4px;
  flex-shrink: 0;
  transition: background 0.15s;
  white-space: nowrap;
}
.bc-root:hover { background: rgba(255,255,255,0.06); }

.bc-sep {
  color: rgba(255, 255, 255, 0.25);
  font-size: 14px;
  flex-shrink: 0;
}

.bc-item {
  background: transparent;
  border: none;
  color: rgba(255,255,255,0.45);
  font-size: 12.5px;
  cursor: pointer;
  padding: 3px 8px;
  border-radius: 4px;
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: background 0.15s, color 0.15s;
}
.bc-item:not(:disabled):hover {
  background: rgba(255,255,255,0.06);
  color: #fff;
}
.bc-current {
  color: var(--text-primary, #fff) !important;
  font-weight: 600;
  cursor: default;
}

.nd-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

/* 字段配置切换按钮 */
.btn-config-toggle {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  border-radius: 6px;
  border: 1px solid rgba(255,255,255,0.1);
  background: transparent;
  color: rgba(255,255,255,0.55);
  font-size: 12.5px;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}
.btn-config-toggle:hover {
  background: rgba(255,255,255,0.06);
  color: #fff;
  border-color: rgba(255,255,255,0.2);
}
.btn-config-toggle.active {
  background: rgba(var(--primary-color-rgb, 99,179,237), 0.15);
  color: #63b3ed;
  border-color: rgba(99,179,237,0.3);
}

.nd-close-btn {
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.35);
  font-size: 16px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  line-height: 1;
  transition: color 0.15s, background 0.15s;
}
.nd-close-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.08);
}

/* ─── 子头部（类型 + 路径） ─────────────────────────────────────────────────── */
.nd-subheader {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 18px;
  border-bottom: 1px solid rgba(255,255,255,0.05);
  flex-shrink: 0;
  gap: 12px;
}

.subheader-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.subheader-right {
  flex-shrink: 0;
}

.path-hint {
  font-size: 11.5px;
  color: rgba(255,255,255,0.25);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  max-width: 280px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: block;
}

.type-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  flex-shrink: 0;
}
.badge-array      { background: rgba(99,179,237,0.12);  color: #63b3ed; }
.badge-prim-array { background: rgba(154,230,180,0.1);  color: #9ae6b4; }
.badge-object     { background: rgba(214,188,250,0.12); color: #d6bcfa; }

.hidden-hint {
  font-size: 11.5px;
  color: rgba(255,200,100,0.7);
}

/* ─── 字段配置面板 ──────────────────────────────────────────────────────────── */
.config-panel {
  border-bottom: 1px solid rgba(255,255,255,0.07);
  background: rgba(255,255,255,0.02);
  flex-shrink: 0;
  max-height: 280px;
  overflow-y: auto;
}

.config-panel-hd {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 18px 6px;
  position: sticky;
  top: 0;
  background: var(--bg-primary, #0f1117);
  border-bottom: 1px solid rgba(255,255,255,0.05);
  z-index: 1;
}

.config-panel-title-wrap {
  display: flex;
  align-items: baseline;
  gap: 12px;
  min-width: 0;
}

.config-panel-title {
  font-size: 13px;
  font-weight: 600;
  color: rgba(255,255,255,0.8);
}

.config-panel-desc {
  font-size: 11.5px;
  color: rgba(255,255,255,0.3);
}

.btn-save-config {
  border: 1px solid rgba(99, 179, 237, 0.35);
  background: rgba(99, 179, 237, 0.16);
  color: #9fd3ff;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  flex-shrink: 0;
}
.btn-save-config:hover:not(:disabled) {
  background: rgba(99, 179, 237, 0.24);
}
.btn-save-config:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.config-list {
  padding: 6px 12px 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.config-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 6px;
  border-radius: 6px;
  transition: background 0.1s;
  cursor: grab;
}
.config-row:hover { background: rgba(255,255,255,0.03); }
.config-row:active { cursor: grabbing; }

.config-row .drag-handle {
  color: var(--text-muted);
  font-size: 14px;
  cursor: grab;
}

.config-row.is-hidden-row {
  opacity: 0.45;
}

/* 可见性切换按钮 */
.vis-btn {
  background: transparent;
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 5px;
  cursor: pointer;
  width: 28px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  flex-shrink: 0;
  transition: border-color 0.15s, background 0.15s;
}
.vis-btn:hover { background: rgba(255,255,255,0.06); }
.vis-btn.vis-off {
  border-color: rgba(255,100,100,0.2);
  background: rgba(255,100,100,0.05);
}

.config-field-name {
  font-size: 12.5px;
  color: rgba(255,255,255,0.6);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  min-width: 100px;
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.config-arrow {
  color: rgba(255,255,255,0.2);
  font-size: 12px;
  flex-shrink: 0;
}

/* 别名输入框 */
.alias-input {
  flex: 1;
  min-width: 80px;
  padding: 4px 9px;
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 5px;
  color: var(--text-primary, #e2e8f0);
  font-size: 12.5px;
  outline: none;
  transition: border-color 0.15s;
}
.alias-input:focus {
  border-color: var(--primary-color, #63b3ed);
  background: rgba(255,255,255,0.07);
}
.alias-input:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
.alias-input::placeholder {
  color: rgba(255,255,255,0.2);
  font-style: italic;
}

/* ─── 主体内容区 ─────────────────────────────────────────────────────────────── */
.nd-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 18px;
}

.all-hidden-tip {
  padding: 32px 20px;
  text-align: center;
  color: rgba(255,200,100,0.6);
  font-size: 13px;
}

/* ─── 迷你表格 ──────────────────────────────────────────────────────────────── */
.mini-table-wrap {
  overflow-x: auto;
}

.mini-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
  min-width: 300px;
}

.mini-table th {
  background: rgba(255,255,255,0.04);
  padding: 8px 12px;
  text-align: left;
  border-bottom: 1px solid rgba(255,255,255,0.08);
  color: var(--text-main, #111827);
  font-weight: 600;
  white-space: nowrap;
}

.mini-table th .alias-tag {
  margin-left: 5px;
  font-size: 10.5px;
  color: rgba(255,255,255,0.3);
  font-weight: 400;
  font-family: 'JetBrains Mono', monospace;
}

.mini-table td {
  padding: 7px 12px;
  border-bottom: 1px solid rgba(255,255,255,0.04);
  vertical-align: top;
}

.mini-table tr:hover td {
  background: rgba(255,255,255,0.02);
}

.row-num-th { color: rgba(255,255,255,0.25); font-weight: 400; width: 36px; }
.row-num    { color: rgba(255,255,255,0.2);  font-size: 11px; }

.cell-text {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 260px;
  color: var(--text-primary, #e2e8f0);
}

/* ─── 键值对列表 ─────────────────────────────────────────────────────────────── */
.kv-list {
  display: flex;
  flex-direction: column;
}

.kv-row {
  display: grid;
  grid-template-columns: 200px 1fr;
  gap: 16px;
  padding: 9px 12px;
  border-bottom: 1px solid rgba(255,255,255,0.04);
  align-items: flex-start;
  border-radius: 6px;
  transition: background 0.1s;
}
.kv-row:hover { background: rgba(255,255,255,0.025); }

.kv-key {
  color: rgba(255,255,255,0.55);
  font-size: 13px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding-top: 1px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.alias-raw-tag {
  font-size: 10.5px;
  color: rgba(255,255,255,0.25);
  font-weight: 400;
  font-family: 'JetBrains Mono', monospace;
  overflow: hidden;
  text-overflow: ellipsis;
}

.kv-val {
  color: var(--text-primary, #e2e8f0);
  font-size: 13px;
  word-break: break-word;
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

/* ─── 基础数组列表 ───────────────────────────────────────────────────────────── */
.prim-array-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.prim-array-item {
  display: flex;
  gap: 12px;
  align-items: baseline;
  padding: 7px 12px;
  border-radius: 6px;
  font-size: 13px;
  border-bottom: 1px solid rgba(255,255,255,0.04);
}

.prim-idx { color: rgba(255,255,255,0.25); font-size: 11px; min-width: 30px; flex-shrink: 0; }
.prim-val { color: var(--text-primary, #e2e8f0); word-break: break-word; }

/* ─── 嵌套徽章 & 下钻按钮 ───────────────────────────────────────────────────── */
.nested-badge-sm {
  display: inline-flex;
  align-items: center;
  padding: 1px 7px;
  border-radius: 10px;
  font-size: 11px;
  background: rgba(99,179,237,0.1);
  color: #63b3ed;
  white-space: nowrap;
}

.btn-drill {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid rgba(255,255,255,0.12);
  background: rgba(255,255,255,0.04);
  color: rgba(255,255,255,0.7);
  font-size: 11.5px;
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}
.btn-drill:hover {
  background: rgba(99,179,237,0.15);
  color: #63b3ed;
  border-color: rgba(99,179,237,0.3);
}

/* ─── 动画 ──────────────────────────────────────────────────────────────────── */
.drawer-backdrop-enter-active,
.drawer-backdrop-leave-active { transition: opacity 0.25s ease; }
.drawer-backdrop-enter-from,
.drawer-backdrop-leave-to    { opacity: 0; }

.drawer-slide-enter-active { transition: transform 0.28s cubic-bezier(0.22, 1, 0.36, 1); }
.drawer-slide-leave-active { transition: transform 0.22s cubic-bezier(0.22, 1, 0.36, 1); }
.drawer-slide-enter-from,
.drawer-slide-leave-to     { transform: translateX(100%); }

/* 配置面板折叠动画 */
.config-panel-enter-active,
.config-panel-leave-active { transition: max-height 0.25s ease, opacity 0.2s ease; overflow: hidden; }
.config-panel-enter-from,
.config-panel-leave-to    { max-height: 0; opacity: 0; }
.config-panel-enter-to,
.config-panel-leave-from  { max-height: 280px; opacity: 1; }

/* ─── 配置面板 Tabs ─────────────────────────────────────────────────────────────── */
.config-panel-tabs { display: flex; gap: 4px; padding: 8px 12px; border-bottom: 1px solid var(--border); }
.config-panel-tabs button {
  padding: 6px 12px; border: none; background: transparent; color: var(--text-secondary);
  font-size: 13px; cursor: pointer; border-radius: 4px; transition: all 0.15s;
}
.config-panel-tabs button.active {
  background: var(--primary-color); color: #fff;
}

/* ─── 字段排序面板 ───────────────────────────────────────────────────────────── */
.order-panel { padding: 12px; }
.order-panel-hd {
  display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;
}
.order-hint { font-size: 12px; color: var(--text-muted); }
.btn-save-order {
  padding: 6px 12px; background: var(--primary-color); color: #fff; border: none;
  border-radius: 4px; font-size: 12px; cursor: pointer;
}
.btn-save-order:disabled { opacity: 0.5; cursor: not-allowed; }
.order-list { max-height: 200px; overflow-y: auto; }
.order-row {
  display: flex; align-items: center; gap: 8px; padding: 8px 10px; margin-bottom: 4px;
  background: var(--surface); border-radius: 4px; cursor: grab; transition: background 0.15s;
}
.order-row:hover { background: var(--bg-secondary); }
.order-row:active { cursor: grabbing; }
.drag-handle { color: var(--text-muted); font-size: 14px; }
.order-field-name { font-size: 13px; color: var(--text-primary); }

/* ─── 字段预览弹窗 ───────────────────────────────────────────────────────── */
.cell-with-preview { display: flex; align-items: center; gap: 4px; }
.cell-text { cursor: default; }
.kv-val { display: flex; align-items: center; gap: 4px; }
.btn-preview {
  cursor: pointer; font-size: 12px; opacity: 1; transition: opacity 0.2s;
}

.preview-popup {
  position: fixed; z-index: 10000; max-width: 400px; max-height: 300px;
  background: var(--surface); border: 1px solid var(--border); border-radius: 6px;
  box-shadow: var(--shadow-lg); overflow: hidden;
}
.preview-header {
  padding: 8px 12px; font-size: 12px; font-weight: 600;
  background: var(--bg-secondary); border-bottom: 1px solid var(--border);
  color: var(--text-primary);
}
.preview-content {
  padding: 10px; margin: 0; font-size: 12px; font-family: monospace;
  white-space: pre-wrap; word-break: break-all; overflow-y: auto;
  color: var(--text-primary); max-height: 250px;
}

/* 浅色主题：Clean Gray 抽屉配色 */
:global(html[data-theme="light"]) .nd-drawer {
  --nd-bg: #ffffff;
  --nd-subtle-bg: #f5f7fa;
  --nd-hover-bg: #f9fafb;
  --nd-border: #e5e7eb;
  --nd-text: #111827;
  --nd-text-secondary: #6b7280;
  --nd-accent: #2563eb;
  background: var(--nd-bg);
  border-left: 1px solid var(--nd-border);
  box-shadow: -12px 0 36px rgba(15, 23, 42, 0.12);
}

:global(html[data-theme="light"]) .nd-header,
:global(html[data-theme="light"]) .nd-subheader {
  background: var(--nd-subtle-bg);
  border-bottom-color: var(--nd-border);
}

:global(html[data-theme="light"]) .path-hint,
:global(html[data-theme="light"]) .hidden-hint,
:global(html[data-theme="light"]) .bc-sep,
:global(html[data-theme="light"]) .bc-item,
:global(html[data-theme="light"]) .config-panel-desc,
:global(html[data-theme="light"]) .config-arrow,
:global(html[data-theme="light"]) .row-num-th,
:global(html[data-theme="light"]) .row-num {
  color: var(--nd-text-secondary);
}

:global(html[data-theme="light"]) .bc-item:not(:disabled):hover,
:global(html[data-theme="light"]) .bc-current,
:global(html[data-theme="light"]) .bc-root {
  color: var(--nd-accent);
}

:global(html[data-theme="light"]) .bc-item:not(:disabled):hover,
:global(html[data-theme="light"]) .bc-root:hover {
  background: #eef2ff;
}

:global(html[data-theme="light"]) .btn-config-toggle {
  border-color: var(--nd-border);
  color: var(--nd-text-secondary);
  background: #fff;
}

:global(html[data-theme="light"]) .btn-config-toggle:hover {
  border-color: #cbd5e1;
  color: var(--nd-text);
  background: var(--nd-hover-bg);
}

:global(html[data-theme="light"]) .btn-config-toggle.active {
  background: #eff6ff;
  color: var(--nd-accent);
  border-color: #bfdbfe;
}

:global(html[data-theme="light"]) .nd-close-btn {
  color: var(--nd-text-secondary);
}

:global(html[data-theme="light"]) .nd-close-btn:hover {
  color: var(--nd-text);
  background: #f3f4f6;
}

:global(html[data-theme="light"]) .config-panel {
  background: #fcfcfd;
  border-bottom-color: var(--nd-border);
}

:global(html[data-theme="light"]) .config-panel-hd {
  background: #f8fafc;
  border-bottom-color: var(--nd-border);
}

:global(html[data-theme="light"]) .config-panel-title,
:global(html[data-theme="light"]) .config-field-name,
:global(html[data-theme="light"]) .kv-key,
:global(html[data-theme="light"]) .kv-val,
:global(html[data-theme="light"]) .prim-val,
:global(html[data-theme="light"]) .cell-text,
:global(html[data-theme="light"]) .mini-table td {
  color: var(--nd-text);
}

:global(html[data-theme="light"]) .config-row:hover,
:global(html[data-theme="light"]) .kv-row:hover,
:global(html[data-theme="light"]) .mini-table tr:hover td {
  background: var(--nd-hover-bg);
}

:global(html[data-theme="light"]) .alias-input {
  background: #fff;
  border-color: #d1d5db;
  color: var(--nd-text);
}

:global(html[data-theme="light"]) .alias-input:focus {
  border-color: var(--nd-accent);
  background: #fff;
}

:global(html[data-theme="light"]) .alias-input::placeholder,
:global(html[data-theme="light"]) .mini-table th .alias-tag,
:global(html[data-theme="light"]) .alias-raw-tag {
  color: var(--nd-text-secondary);
}

:global(html[data-theme="light"]) .mini-table th {
  position: sticky;
  top: 0;
  z-index: 2;
  background: var(--nd-subtle-bg);
  border-bottom: 1px solid var(--nd-border);
  color: var(--nd-text);
}

:global(html[data-theme="light"]) .mini-table td,
:global(html[data-theme="light"]) .kv-row,
:global(html[data-theme="light"]) .prim-array-item {
  border-bottom-color: var(--nd-border);
}
</style>
