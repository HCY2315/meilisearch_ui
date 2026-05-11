<template>
  <div class="search-section">
    <!-- 搜索框 -->
    <div class="search-bar">
      <div class="search-box-wrap" ref="searchBoxWrapRef">
        <span class="search-icon">🔍</span>
<input
          class="search-input"
          v-model="store.searchInput"
          placeholder="输入关键词搜索..."
          @input="store.scheduleDebouncedSearch()"
          @keyup.enter="store.performSearch()"
        />
        <span
          ref="aiBadgeRef"
          class="ai-badge"
          :class="{ active: store.aiConfig.aiEnabled && store.aiConfig.aiWeight > 0 }"
          @click="onAiBadgeClick"
        >AI</span>
        <div
          v-if="store.aiDropdownOpen"
          ref="aiDropdownRef"
          class="ai-dropdown"
          :style="dropdownStyle"
        >
          <div class="dropdown-title">AI 权重</div>
          <label class="dropdown-row" style="margin-bottom: 10px; cursor: pointer;">
            <span>启用 AI</span>
            <input type="checkbox" v-model="store.aiConfig.aiEnabled" @change="onAiToggle" />
          </label>
          <div class="dropdown-row">
            <span>基础</span>
            <span class="ai-weight-value">{{ baseWeight }}%</span>
          </div>
          <div class="dropdown-row">
            <span>AI</span>
            <span class="ai-weight-value">{{ store.aiConfig.aiWeight }}%</span>
          </div>
          <input
            type="range"
            min="0"
            max="100"
            v-model.number="store.aiConfig.aiWeight"
            @input="onAiWeightChange"
          />
          <div class="ai-weight-hint">基础 {{ baseWeight }}% · AI {{ store.aiConfig.aiWeight }}%</div>
        </div>
      </div>
      <button class="btn btn-primary" @click="store.performSearch()">搜索</button>
      <button class="btn btn-secondary" @click="handleClearQuery">清空</button>
    </div>

    <!-- 查询条件构建器 -->
    <div v-if="store.currentIndex" class="query-editor">
      <h3>📋 查询条件构建器</h3>
      <div class="query-rows">
        <div v-for="(row, idx) in store.queryRows" :key="row.id" class="query-row">
          <select v-if="idx > 0" class="form-control logic-select" v-model="row.logic" @change="store.scheduleDebouncedSearch()">
            <option value="AND">且</option>
            <option value="OR">或</option>
          </select>
          <span v-else class="logic-label"></span>
          <select class="form-control field-select" v-model="row.field" @change="store.scheduleDebouncedSearch()">
            <option value="">选择字段</option>
            <option v-for="f in store.visibleAvailableFields" :key="f" :value="f">{{ store.fieldLabels[f] || f }}</option>
          </select>
          <select class="form-control op-select" v-model="row.operator" @change="store.scheduleDebouncedSearch()">
            <option value="=">=</option>
            <option value="!=">≠</option>
            <option value=">">&gt;</option>
            <option value=">=">≥</option>
            <option value="<">&lt;</option>
            <option value="<=">≤</option>
            <option value="contains">包含</option>
            <option value="exists">存在</option>
          </select>
          <input class="form-control value-input" v-model="row.value" placeholder="值" @input="store.scheduleDebouncedSearch()" />
          <button class="btn btn-icon" @click="handleRemoveQueryRow(row.id)" title="删除条件">×</button>
        </div>
      </div>
      <div class="query-actions">
        <button class="btn btn-secondary btn-sm" @click="store.addQueryRow()">➕ 添加查询条件</button>
        <button class="btn btn-primary btn-sm" @click="store.performSearch()">✅ 应用查询</button>
        <button class="btn btn-secondary btn-sm" @click="handleClearQuery">🗑️ 清空查询</button>
      </div>
      <div class="filter-preview">当前过滤: {{ filterPreviewText }}</div>
    </div>

<!-- 结果面板 -->
    <div v-if="store.lastResults" class="results-panel">
      <div class="results-stats">
        <span class="results-count">找到 <strong>{{ formatNumber(store.resultsCount) }}</strong> 条结果</span>
        <div class="results-actions">
          <span>{{ store.processingTimeMs ? `耗时 ${store.processingTimeMs}ms` : '' }}</span>
          <button class="btn btn-secondary btn-sm" @click="openAdvancedSettings">⚙️ 高级设置</button>
          <button v-if="isAdmin" class="btn btn-secondary btn-sm" @click="openColumnConfig">列设置</button>
          <button class="btn btn-secondary btn-sm" @click="store.openViewConfig()">视图设置</button>
          <button class="btn btn-secondary btn-sm" @click="store.exportCsv()" :disabled="store.exportDownloading">
            {{ store.exportDownloading ? `📥 导出 ${store.exportProgress}/${store.exportTotal}` : '📥 导出 CSV' }}
          </button>
          <select class="form-control" style="min-width: 120px; padding: 6px 10px;" v-model="store.viewMode">
            <option value="table">表格</option>
            <option v-for="cfg in store.viewConfigs" :key="cfg.name" :value="cfg.name">{{ cfg.name }}</option>
          </select>
          <template v-if="isAdmin">
            <button
              class="btn btn-sm"
              :class="store.editLocked ? 'btn-secondary' : 'btn-warning'"
              @click="store.toggleEditLock()"
            >
              {{ store.editLocked ? '🔒 已锁定' : '🔓 可编辑' }}
            </button>
            <template v-if="!store.editLocked">
              <select
                class="form-control"
                style="min-width: 120px; padding: 6px 10px;"
                v-model="store.primaryKeyField"
              >
                <option value="">选择主键</option>
                <option v-for="col in store.lastBaseColumns" :key="col" :value="col">{{ col }}</option>
              </select>
            </template>
            <button
              class="btn btn-primary btn-sm"
              :disabled="store.editLocked || !hasPendingEdits"
              @click="store.saveEdits()"
            >
              💾 保存修改
            </button>
          </template>
          <button v-if="isAdmin" class="btn btn-primary btn-sm" @click="saveAllUISettingsToBackend">推送同步配置</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
import { useConnectionStore } from '@/composables/useConnectionStore'
import { useSearchStore } from '@/composables/useSearchStore'
import { useUIStore } from '@/composables/useUIStore'
import { useAppStore } from '@/composables/useApp'
import { formatNumber, buildFilterExpression } from '@/utils'
import * as storage from '@/services/storage'
import { saveIndexConfig } from '@/services/api'
import type { IndexInfo } from '@/types'

const connectionStore = useConnectionStore()
const searchStore = useSearchStore()
const uiStore = useUIStore()
const appStore = useAppStore()

// 用于模板的统一访问（只读，set 通过具体 store 处理）
const store = appStore

const aiBadgeRef = ref<HTMLElement | null>(null)
const aiDropdownRef = ref<HTMLElement | null>(null)
const searchBoxWrapRef = ref<HTMLElement | null>(null)

const baseWeight = computed(() => 100 - store.aiConfig.aiWeight)

const dropdownStyle = computed(() => {
  const wrap = searchBoxWrapRef.value
  if (!wrap) return {}
  const rect = wrap.getBoundingClientRect()
  return {
    top: `${rect.height + 4}px`,
    left: '0px',
    right: '0px',
  }
})

const userRole = ref('user')
const isAdmin = ref(false)

onMounted(() => {
  const authUserStr = localStorage.getItem('authUser')
  if (authUserStr) {
    try {
      const authUser = JSON.parse(authUserStr)
      userRole.value = authUser.role || 'user'
      isAdmin.value = authUser.role === 'admin'
    } catch {}
  }
  document.addEventListener('click', onDocumentClick)
})

function openAdvancedSettings() {
  console.log('[高级设置] 点击了')
  appStore.advancedSettingsOpen = true
}

async function saveAllUISettingsToBackend() {
  const token = localStorage.getItem('authToken')
  if (!token) return alert('未获得登录凭证')

  const currentIdx = store.indexes.find((i: IndexInfo) => i.uid === store.currentIndex)

  const payload = {
    uid: store.currentIndex,
    alias: currentIdx?.displayName || store.currentIndex,
    isLocked: currentIdx?.isLocked || false,
    fieldConfigs: JSON.stringify(store.currentFieldConfigsForSync()),
    viewConfigs: JSON.stringify(store.viewConfigs),
    tableConfigs: JSON.stringify({
       hidden: store.hiddenColumns,
       order: store.columnOrder
    }),
    canEdit: !store.editLocked,
    nestedFieldConfigs: JSON.stringify(store.nestedFieldConfigs)
  }

  try {
     const res = await saveIndexConfig(payload)
     if(res.ok) alert('前台视图/列配置已同步至后端，永久保存成功！')
     else alert('同步失败：' + res.status)
  } catch {
     alert('请求发生错误')
  }
}

const filterPreviewText = computed(() => {
  const rows = store.queryRows
  const expr = buildFilterExpression(rows)
  return expr || '无'
})

const hasPendingEdits = computed(() => Object.keys(store.pendingEdits).length > 0)

function onAiBadgeClick() {
  store.aiDropdownOpen = !store.aiDropdownOpen
}

function openColumnConfig() {
  if (!isAdmin.value) {
    appStore.pushToast('仅管理员可配置列设置', 'warning')
    return
  }
  appStore.columnConfigOpen = !appStore.columnConfigOpen
  console.log('appStore.columnConfigOpen:', appStore.columnConfigOpen)
}

function onAiToggle() {
  storage.saveAiConfig(store.aiConfig)
  store.performSearch()
}

function onAiWeightChange() {
  storage.saveAiConfig(store.aiConfig)
  store.performSearch()
}

function handleClearQuery() {
  appStore.clearQuery()
}

function handleRemoveQueryRow(id: number) {
  appStore.removeQueryRow(id)
}

function onDocumentClick(e: MouseEvent) {
  if (!store.aiDropdownOpen) return
  const badge = aiBadgeRef.value
  const dropdown = aiDropdownRef.value
  if (badge && badge.contains(e.target as Node)) return
  if (dropdown && dropdown.contains(e.target as Node)) return
  store.aiDropdownOpen = false
}

onBeforeUnmount(() => {
  document.removeEventListener('click', onDocumentClick)
})
</script>

<style scoped>
.search-section { display: flex; flex-direction: column; gap: 14px; }
.search-bar { display: flex; gap: 10px; align-items: center; margin-bottom: 12px; position: relative; }
.search-box-wrap { position: relative; flex: 1; z-index: 10; }
.search-input {
  width: 100%;
  padding: 15px 70px 15px 50px;
  border: 2px solid var(--border);
  border-radius: var(--radius-lg);
  font-size: 1rem;
  transition: var(--transition);
  background: var(--surface);
  color: var(--text-primary);
  font-family: inherit;
}
.search-input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 5px rgba(99,102,241,0.1), 0 8px 32px rgba(99,102,241,0.12);
  transform: translateY(-3px);
}
.search-input::placeholder { color: var(--text-muted); }
.search-icon {
  position: absolute;
  left: 14px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-secondary);
  font-size: 1.2rem;
  pointer-events: none;
  z-index: 2;
}
.query-editor { background: var(--surface-glass); backdrop-filter: blur(20px); border-radius: var(--radius-lg); padding: 18px 22px; border: 1px solid var(--border-light); }
.query-editor h3 { margin: 0 0 12px; font-size: 14px; font-weight: 600; }
.query-rows { display: flex; flex-direction: column; gap: 8px; }
.query-row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.logic-label { width: 48px; flex-shrink: 0; }
.logic-select { width: 60px; flex-shrink: 0; }
.field-select { flex: 1; min-width: 120px; }
.op-select { width: 90px; flex-shrink: 0; }
.value-input { flex: 2; min-width: 120px; }
.btn-icon { background: transparent; border: 1.5px solid var(--border); color: var(--text-muted); border-radius: 6px; width: 32px; height: 32px; padding: 0; cursor: pointer; font-size: 16px; transition: var(--transition); }
.btn-icon:hover { border-color: var(--error-color); color: var(--error-color); }
.query-actions { display: flex; gap: 8px; margin-top: 12px; flex-wrap: wrap; }
.filter-preview { margin-top: 8px; font-size: 0.8rem; color: var(--text-muted); padding: 4px 8px; background: var(--bg-secondary); border-radius: 4px; display: inline-block; }
.results-panel { background: var(--surface-glass); backdrop-filter: blur(20px); border-radius: var(--radius-lg); padding: 18px 22px; border: 1px solid var(--border-light); }
.results-stats { display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px; }
.results-count { color: var(--text-secondary); font-size: 0.9rem; }
.results-count strong { color: var(--primary-color); font-size: 1.05rem; }
.results-actions { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }

.ai-badge {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.5px;
  background: rgba(99,102,241,0.1);
  border: 1.5px solid var(--border);
  color: var(--text-muted);
  transition: all 0.3s ease;
  cursor: pointer;
  user-select: none;
  z-index: 5;
}
.ai-badge.active {
  background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
  color: white;
  border-color: transparent;
  box-shadow: 0 4px 15px rgba(99,102,241,0.4);
}
.ai-badge:hover { transform: translateY(-50%) scale(1.05); }
.ai-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: var(--shadow-lg);
  padding: 16px;
  z-index: 100;
}
.dropdown-title { font-size: 0.9rem; font-weight: 600; margin-bottom: 12px; }
.dropdown-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; font-size: 0.85rem; color: var(--text-secondary); }
.ai-weight-value { font-weight: 700; color: var(--primary-color); }
.ai-dropdown input[type="range"] { width: 100%; height: 6px; border-radius: 3px; background: var(--border); appearance: none; cursor: pointer; margin: 8px 0; }
.ai-dropdown input[type="range"]::-webkit-slider-thumb { appearance: none; width: 18px; height: 18px; border-radius: 50%; background: linear-gradient(135deg, var(--primary-color), var(--secondary-color)); box-shadow: 0 2px 8px rgba(99,102,241,0.4); cursor: pointer; }
.ai-weight-hint { text-align: center; font-size: 0.75rem; color: var(--text-muted); margin-top: 4px; }

/* ============ Mobile Responsiveness ============ */
@media (max-width: 768px) {
  .search-bar {
    flex-direction: column;
    align-items: stretch;
  }
  
  .query-row {
    flex-direction: column;
    align-items: stretch;
    background: rgba(0, 0, 0, 0.02);
    padding: 10px;
    border-radius: 8px;
  }
  
  .logic-label {
    display: none;
  }
  
  .results-stats {
    flex-direction: column;
    align-items: flex-start;
  }
  
  .results-actions {
    width: 100%;
    margin-top: 12px;
  }
  
  .query-actions {
    flex-direction: column;
  }
  
  .query-actions button {
    width: 100%;
  }
}
</style>
