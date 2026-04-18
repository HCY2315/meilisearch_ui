<template>
  <Transition name="drawer">
    <div v-if="store.filtersDrawerOpen" class="drawer-overlay" @click.self="store.filtersDrawerOpen = false">
      <div class="drawer">
        <div class="drawer-header">
          <h3>筛选面板</h3>
          <button class="modal-close" @click="store.filtersDrawerOpen = false">×</button>
        </div>
        <div class="drawer-body">
          <!-- Facet 筛选 -->
          <div v-if="store.visibleFacetDistribution">
            <div v-for="(values, facet) in store.visibleFacetDistribution" :key="facet" class="filter-section">
              <h4>{{ store.fieldLabels[facet] || facet }}</h4>
              <div class="filter-list">
                <div
                  v-for="[val, count] in sortedFacets(values)"
                  :key="val"
                  class="filter-item"
                >
                  <label>
                    <input
                      type="checkbox"
                      :checked="isFacetChecked(facet, val)"
                      @change="store.toggleFacet(facet, val, ($event.target as HTMLInputElement).checked)"
                    />
                    <span>{{ val }}</span>
                    <span class="filter-count">{{ count }}</span>
                  </label>
                </div>
              </div>
            </div>
          </div>

          <!-- 搜索字段选择 -->
          <div class="filter-section">
            <h4>搜索字段</h4>
            <div class="filter-list">
              <div v-for="field in store.visibleAvailableFields" :key="field" class="filter-item">
                <label>
                  <input
                    type="checkbox"
                    :checked="store.searchFields.includes(field)"
                    @change="store.toggleSearchField(field, ($event.target as HTMLInputElement).checked)"
                  />
                  <span>{{ store.fieldLabels[field] || field }}</span>
                </label>
              </div>
            </div>
          </div>

          <!-- 搜索历史 -->
          <div v-if="store.searchHistory.length" class="filter-section">
            <div class="section-header">
              <h4>搜索历史</h4>
              <button class="btn btn-link" @click="store.clearHistory()">清空</button>
            </div>
            <div class="filter-list">
              <div v-for="item in store.searchHistory" :key="item.timestamp" class="history-item" @click="store.applySearchHistory(item.query)">
                <span>{{ item.query }}</span>
                <span class="history-time">{{ formatTime(item.timestamp) }}</span>
              </div>
            </div>
          </div>

          <!-- 热门搜索 -->
          <div v-if="store.popularSearches.length" class="filter-section">
            <div class="section-header">
              <h4>热门搜索</h4>
              <select v-if="store.visibleFilterableFields.length" class="form-control" v-model="store.popularSearchField" @change="store.loadPopularSearches()">
                <option v-for="f in store.visibleFilterableFields" :key="f" :value="f">{{ store.fieldLabels[f] || f }}</option>
              </select>
            </div>
            <div class="filter-list">
              <div v-for="item in store.popularSearches" :key="item.value" class="suggestion-item" @click="store.applyPopularSearch(item.value)">
                <span>{{ item.value }}</span>
                <span v-if="item.count" class="filter-count">{{ item.count }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { useAppStore } from '@/composables/useApp'
import { formatTime } from '@/utils'

const store = useAppStore()

function sortedFacets(values: Record<string, number>): [string, number][] {
  return Object.entries(values).sort((a, b) => b[1] - a[1])
}

function isFacetChecked(facet: string, value: string): boolean {
  return store.facets[facet]?.includes(value) ?? false
}
</script>

<style scoped>
.drawer-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
}
.drawer {
  width: 340px;
  max-width: 90vw;
  background: var(--bg-secondary);
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}
.drawer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid var(--border);
}
.drawer-body { flex: 1; overflow-y: auto; padding: 16px; }
.filter-section { margin-bottom: 24px; }
.filter-section h4 { font-size: 14px; color: var(--text-secondary); margin-bottom: 8px; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.section-header h4 { margin-bottom: 0; }
.section-header select { width: 120px; font-size: 12px; padding: 4px; }
.filter-list { display: flex; flex-direction: column; gap: 4px; }
.filter-item label { display: flex; align-items: center; gap: 8px; cursor: pointer; padding: 4px 0; font-size: 13px; }
.filter-count { color: var(--text-muted); font-size: 12px; margin-left: auto; }
.history-item, .suggestion-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.history-item:hover, .suggestion-item:hover { background: var(--surface); }
.history-time { color: var(--text-muted); font-size: 11px; }
.drawer-enter-active, .drawer-leave-active { transition: transform 0.3s ease; }
.drawer-enter-from, .drawer-leave-to { transform: translateX(100%); }
</style>
