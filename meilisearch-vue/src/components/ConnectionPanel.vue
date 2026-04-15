<template>
  <div class="connection-panel">
    <div class="connection-row">
      <div class="form-group" style="flex:1">
        <label>服务器地址</label>
        <input class="form-control" v-model="store.hostInput" placeholder="http://localhost:7700" @keyup.enter="store.connect()" />
      </div>
      <div class="form-group" style="flex:0 0 220px">
        <label>API Key</label>
        <input class="form-control" v-model="store.apiKeyInput" type="password" placeholder="API Key" @keyup.enter="store.connect()" />
      </div>
      <button class="btn btn-primary" @click="store.connect()" :disabled="store.loading" style="align-self:flex-end">
        {{ store.loading ? '连接中...' : '连接' }}
      </button>
    </div>

    <div v-if="store.indexes.length > 0" class="index-section">
      <div class="form-group" style="flex:1">
        <label>选择索引</label>
        <select class="form-control" v-model="selectedIndex" @change="onIndexChange">
          <option value="">选择索引</option>
          <option v-for="idx in store.indexes" :key="idx.uid" :value="idx.uid">
            {{ idx.uid }}{{ idx.count !== undefined ? ` (${formatNumber(idx.count)})` : '' }}
          </option>
        </select>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useAppStore } from '@/composables/useApp'
import { formatNumber } from '@/utils'

const store = useAppStore()
const selectedIndex = ref('')

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
.connection-row {
  display: flex;
  gap: 12px;
  align-items: flex-end;
}
.index-section {
  margin-top: 12px;
  display: flex;
  gap: 12px;
}
</style>
