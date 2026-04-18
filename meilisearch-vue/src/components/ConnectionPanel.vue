<template>
  <div class="connection-panel">
    <div class="connection-row status-bar">
      <div class="status-indicator">
        <span class="dot" :class="{ 'connected': store.indexes.length > 0 }"></span>
        {{ store.indexes.length > 0 ? '已连接到搜索引擎节点' : '等待连接...' }}
      </div>
    </div>

    <div v-if="store.indexes.length > 0" class="index-section">
      <div class="form-group" style="flex:1">
        <label>选择查询资源 (Index)</label>
        <select class="form-control" v-model="selectedIndex" @change="onIndexChange">
          <option value="">-- 请选择 --</option>
          
          <!-- 公开资源 -->
          <option v-for="idx in publicIndexes" :key="idx.uid + 'pub'" :value="idx.uid">
            {{ idx.displayName || idx.uid }} {{ idx.count !== undefined ? `(${formatNumber(idx.count)})` : '' }} 🌐
          </option>

          <!-- 已解锁私有 -->
          <option v-for="idx in unlockedIndexes" :key="idx.uid + 'unl'" :value="idx.uid">
            {{ idx.displayName || idx.uid }} {{ idx.count !== undefined ? `(${formatNumber(idx.count)})` : '' }} 🔓
          </option>

          <!-- 锁定资源 -->
          <option v-for="idx in lockedIndexes" :key="idx.uid + 'loc'" :value="idx.uid" disabled>
            {{ idx.displayName || idx.uid }} 🔒 (受限)
          </option>
        </select>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useAppStore } from '@/composables/useApp'
import { formatNumber } from '@/utils'
import type { IndexInfo } from '@/types'

const store = useAppStore()
const selectedIndex = ref('')

// 分组逻辑
const publicIndexes = computed(() => store.indexes.filter((idx: IndexInfo) => !idx.isLocked))
const unlockedIndexes = computed(() => store.indexes.filter((idx: IndexInfo) => idx.isLocked && idx.isUnlocked))
const lockedIndexes = computed(() => store.indexes.filter((idx: IndexInfo) => idx.isLocked && !idx.isUnlocked))

// 当后端推送了初始连接后，可能已有 currentIndex
watch(() => store.currentIndex, (newVal) => {
  if (newVal && newVal !== selectedIndex.value) {
    selectedIndex.value = newVal
  }
}, { immediate: true })

function onIndexChange() {
  store.selectIndex(selectedIndex.value)
}
</script>

<style scoped>
.connection-panel {
  background: var(--surface);
  border-radius: var(--radius);
  padding: 16px;
  margin-bottom: 16px;
}
.status-bar {
  display: flex;
  align-items: center;
  font-size: 14px;
}
.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-muted, #888);
}
.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #ff4a4a;
}
.dot.connected {
  background: #28c840;
}
.index-section {
  margin-top: 16px;
  display: flex;
  gap: 12px;
}
</style>
