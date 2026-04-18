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
          
          <optgroup v-if="publicIndexes.length" label="🌐 公开可见">
            <option v-for="idx in publicIndexes" :key="idx.uid" :value="idx.uid">
              {{ idx.displayName || idx.uid }} {{ idx.count !== undefined ? ` (${formatNumber(idx.count)})` : '' }}
            </option>
          </optgroup>

          <optgroup v-if="unlockedIndexes.length" label="🔓 已解锁 (私有)">
            <option v-for="idx in unlockedIndexes" :key="idx.uid" :value="idx.uid">
              {{ idx.displayName || idx.uid }} {{ idx.count !== undefined ? ` (${formatNumber(idx.count)})` : '' }}
            </option>
          </optgroup>

          <optgroup v-if="lockedIndexes.length" label="🔒 需凭证解锁">
            <option v-for="idx in lockedIndexes" :key="idx.uid" :value="idx.uid" disabled>
              {{ idx.displayName || idx.uid }} (受限访问)
            </option>
          </optgroup>
        </select>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useAppStore } from '@/composables/useApp'
import { formatNumber } from '@/utils'

const store = useAppStore()
const selectedIndex = ref('')

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
