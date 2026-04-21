<template>
  <Teleport to="body">
    <Transition name="drawer-backdrop">
      <div
        v-if="modelValue"
        class="nd-backdrop"
        @click.self="handleClose"
      />
    </Transition>
    <Transition name="drawer-slide">
      <div
        v-if="modelValue"
        class="nd-drawer"
        role="dialog"
        aria-modal="true"
        @keydown.esc="handleClose"
      >
        <!-- 头部：面包屑 + 关闭按钮 -->
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
              >
                {{ frame.label }}
              </button>
            </template>
          </nav>
          <button class="nd-close-btn" @click="handleClose" aria-label="关闭">✕</button>
        </div>

        <!-- 类型徽章 + 当前层说明 -->
        <div class="nd-subheader">
          <span v-if="isArrayOfObjects" class="type-badge badge-array">
            🗂 对象数组 · {{ (currentData as unknown[]).length }} 条
          </span>
          <span v-else-if="isPureArray" class="type-badge badge-prim-array">
            📋 基础数组 · {{ (currentData as unknown[]).length }} 条
          </span>
          <span v-else class="type-badge badge-object">
            📦 对象 · {{ objectEntries.length }} 个字段
          </span>
        </div>

        <!-- 主内容区 -->
        <div class="nd-body">

          <!-- ① 对象数组 → 迷你表格 -->
          <div v-if="isArrayOfObjects" class="mini-table-wrap">
            <table class="mini-table">
              <thead>
                <tr>
                  <th class="row-num-th">#</th>
                  <th v-for="col in arrayColumns" :key="col" :title="col">
                    {{ col }}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(row, ri) in (currentData as Record<string, unknown>[])" :key="ri">
                  <td class="row-num">{{ ri + 1 }}</td>
                  <td v-for="col in arrayColumns" :key="col">
                    <template v-if="isNestedValue(row[col])">
                      <span class="nested-badge-sm">{{ getNestedBadgeText(row[col]) }}</span>
                      <button
                        class="btn-drill"
                        @click="push(`${stack[stack.length - 1].label}[${ri}].${col}`, row[col])"
                      >🔍</button>
                    </template>
                    <span v-else class="cell-text" :title="primitiveToStr(row[col])">
                      {{ primitiveToStr(row[col]) }}
                    </span>
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
            <div
              v-for="[key, val] in objectEntries"
              :key="key"
              class="kv-row"
            >
              <span class="kv-key" :title="key">{{ key }}</span>
              <span class="kv-val">
                <template v-if="isNestedValue(val)">
                  <span class="nested-badge-sm">{{ getNestedBadgeText(val) }}</span>
                  <button
                    class="btn-drill"
                    @click="push(`${stack[stack.length - 1].label}.${key}`, val)"
                  >🔍 查看</button>
                </template>
                <span v-else :title="primitiveToStr(val)">{{ primitiveToStr(val) }}</span>
              </span>
            </div>
          </div>

        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { isNestedValue, getNestedBadgeText } from '@/utils'

// ─── Props & Emits ───────────────────────────────────────────────────────────
const props = defineProps<{
  modelValue: boolean       // 控制抽屉开关
  initialData: unknown      // 初始层数据（Object 或 Object[]）
  initialLabel: string      // 初始字段名（面包屑第一节）
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', val: boolean): void
}>()

// ─── 导航栈 ──────────────────────────────────────────────────────────────────
interface NestedFrame {
  label: string    // 面包屑显示名
  data: unknown    // 该层数据
}

const stack = ref<NestedFrame[]>([])

// 当抽屉打开时（或打开时传入的数据变化），重置导航栈到初始层
watch(
  () => [props.modelValue, props.initialData, props.initialLabel] as const,
  ([open]) => {
    if (open) {
      stack.value = [{ label: props.initialLabel, data: props.initialData }]
    }
  },
  { immediate: true }
)

// ─── 当前层 ──────────────────────────────────────────────────────────────────
const currentData = computed<unknown>(() => {
  if (!stack.value.length) return null
  return stack.value[stack.value.length - 1].data
})

// 是否为"对象数组"（每个元素都是对象）
const isArrayOfObjects = computed(() => {
  const d = currentData.value
  return Array.isArray(d) && d.length > 0 && d[0] !== null && typeof d[0] === 'object'
})

// 是否为基础类型数组（元素为 string/number/boolean）
const isPureArray = computed(() => {
  const d = currentData.value
  return Array.isArray(d) && !isArrayOfObjects.value
})

// 对象数组的列名：取所有行 key 的并集，确保无遗漏
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

// 单个对象的键值对列表
const objectEntries = computed<[string, unknown][]>(() => {
  const d = currentData.value
  if (d === null || Array.isArray(d) || typeof d !== 'object') return []
  return Object.entries(d as object)
})

// ─── 导航操作 ─────────────────────────────────────────────────────────────────
// 向下钻取一层（压栈）
function push(label: string, data: unknown) {
  stack.value = [...stack.value, { label, data }]
}

// 点击面包屑跳回指定层（弹栈到该索引）
function jumpTo(index: number) {
  if (index < 0 || index >= stack.value.length - 1) return
  stack.value = stack.value.slice(0, index + 1)
}

// 关闭抽屉
function handleClose() {
  emit('update:modelValue', false)
}

// ─── 原始值渲染 ───────────────────────────────────────────────────────────────
/**
 * 将原始类型值转为可读字符串，用于单元格/键值对的文字展示。
 * 对象/数组类型不会走到这里（会走嵌套按钮分支），但保留兜底处理。
 */
function primitiveToStr(val: unknown): string {
  if (val === null) return 'null'
  if (val === undefined) return ''
  if (typeof val === 'string') return val
  if (typeof val === 'number' || typeof val === 'boolean') return String(val)
  // NOTE: 兜底：理论上不会出现（嵌套类型会被 isNestedValue 拦截），
  //       但保留以防边界情况
  return JSON.stringify(val)
}
</script>

<style scoped>
/* ─── 背景遮罩 ──────────────────────────────────────────────────────────────── */
.nd-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 1000;
}

/* ─── 抽屉主体 ──────────────────────────────────────────────────────────────── */
.nd-drawer {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: min(65vw, 960px);
  min-width: 360px;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary, #0f1117);
  border-left: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: -8px 0 40px rgba(0, 0, 0, 0.5);
  z-index: 1001;
  overflow: hidden;
}

/* ─── 头部 ──────────────────────────────────────────────────────────────────── */
.nd-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
  flex-shrink: 0;
}

/* ─── 面包屑 ─────────────────────────────────────────────────────────────────── */
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
  padding: 2px 6px;
  border-radius: 4px;
  flex-shrink: 0;
  transition: background 0.15s;
}
.bc-root:hover { background: rgba(255,255,255,0.06); }

.bc-sep {
  color: rgba(255, 255, 255, 0.3);
  font-size: 13px;
  flex-shrink: 0;
}

.bc-item {
  background: transparent;
  border: none;
  color: var(--text-muted, rgba(255,255,255,0.5));
  font-size: 12.5px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: background 0.15s, color 0.15s;
}
.bc-item:not(:disabled):hover {
  background: rgba(255, 255, 255, 0.06);
  color: #fff;
}
.bc-current {
  color: var(--text-primary, #fff) !important;
  font-weight: 600;
  cursor: default;
}

.nd-close-btn {
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.4);
  font-size: 16px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  line-height: 1;
  transition: color 0.15s, background 0.15s;
  flex-shrink: 0;
}
.nd-close-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.08);
}

/* ─── 子头部（类型徽章） ────────────────────────────────────────────────────── */
.nd-subheader {
  padding: 10px 18px;
  border-bottom: 1px solid rgba(255,255,255,0.05);
  flex-shrink: 0;
}

.type-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
}
.badge-array   { background: rgba(99,179,237,0.12); color: #63b3ed; }
.badge-prim-array { background: rgba(154,230,180,0.1); color: #9ae6b4; }
.badge-object  { background: rgba(214,188,250,0.12); color: #d6bcfa; }

/* ─── 主体内容区 ─────────────────────────────────────────────────────────────── */
.nd-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 18px;
}

/* ─── 迷你表格 ──────────────────────────────────────────────────────────────── */
.mini-table-wrap {
  overflow-x: auto;
}

.mini-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12.5px;
  min-width: 300px;
}

.mini-table th {
  background: rgba(255,255,255,0.04);
  padding: 8px 10px;
  text-align: left;
  border-bottom: 1px solid rgba(255,255,255,0.08);
  color: rgba(255,255,255,0.65);
  font-weight: 600;
  white-space: nowrap;
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mini-table td {
  padding: 7px 10px;
  border-bottom: 1px solid rgba(255,255,255,0.04);
  vertical-align: top;
  max-width: 220px;
}

.mini-table tr:hover td {
  background: rgba(255,255,255,0.025);
}

.row-num-th { color: rgba(255,255,255,0.3); font-weight: 400; width: 36px; }
.row-num    { color: rgba(255,255,255,0.25); font-size: 11px; width: 36px; }

.cell-text {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
  color: var(--text-primary, #e2e8f0);
}

/* ─── 键值对列表 ─────────────────────────────────────────────────────────────── */
.kv-list {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.kv-row {
  display: grid;
  grid-template-columns: 180px 1fr;
  gap: 12px;
  padding: 9px 12px;
  border-bottom: 1px solid rgba(255,255,255,0.04);
  align-items: flex-start;
  border-radius: 6px;
  transition: background 0.1s;
}
.kv-row:hover { background: rgba(255,255,255,0.025); }

.kv-key {
  color: rgba(255,255,255,0.55);
  font-size: 12.5px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding-top: 1px;
}

.kv-val {
  color: var(--text-primary, #e2e8f0);
  font-size: 12.5px;
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
  gap: 4px;
}

.prim-array-item {
  display: flex;
  gap: 10px;
  align-items: baseline;
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 13px;
  border-bottom: 1px solid rgba(255,255,255,0.04);
}

.prim-idx { color: rgba(255,255,255,0.3); font-size: 11px; min-width: 28px; }
.prim-val { color: var(--text-primary, #e2e8f0); word-break: break-word; }

/* ─── 嵌套徽章（行内小版本） ────────────────────────────────────────────────── */
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

/* ─── 下钻按钮 ──────────────────────────────────────────────────────────────── */
.btn-drill {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 1px 7px;
  border-radius: 4px;
  border: 1px solid rgba(255,255,255,0.12);
  background: rgba(255,255,255,0.04);
  color: rgba(255,255,255,0.75);
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
/* 背景遮罩淡入淡出 */
.drawer-backdrop-enter-active,
.drawer-backdrop-leave-active { transition: opacity 0.25s ease; }
.drawer-backdrop-enter-from,
.drawer-backdrop-leave-to    { opacity: 0; }

/* 抽屉从右侧滑入滑出 */
.drawer-slide-enter-active { transition: transform 0.28s cubic-bezier(0.32, 0.72, 0, 1); }
.drawer-slide-leave-active { transition: transform 0.22s cubic-bezier(0.32, 0.72, 0, 1); }
.drawer-slide-enter-from,
.drawer-slide-leave-to     { transform: translateX(100%); }
</style>
