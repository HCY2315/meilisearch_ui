<template>
  <div class="container" :data-theme="theme">
    <header class="header">
      <div class="header-top">
        <h1>🔍 多维查询系统</h1>
        <button class="theme-toggle" @click="toggleTheme">
          {{ theme === 'dark' ? '🌙 夜间模式' : '☀️ 日间模式' }}
        </button>
      </div>
      <p>强大的多维度搜索与过滤功能，快速定位您需要的内容</p>
    </header>

    <nav class="tab-nav">
      <button
        :class="['tab-btn', store.currentTab === 'search' && 'active']"
        @click="store.setCurrentTab('search')"
      >🔍 搜索</button>
      <button
        :class="['tab-btn', store.currentTab === 'assets' && 'active']"
        @click="store.setCurrentTab('assets')"
      >📦 批量新增</button>
    </nav>

    <template v-if="store.currentTab === 'search'">
      <ConnectionPanel />
      <SearchSection />
      <ResultsTable />
    </template>
    <template v-else>
      <AssetManagement />
    </template>

    <footer class="footer">
      <p>作者: 冰城拓 Copyright © 2025. 保留所有权利.</p>
    </footer>

    <ToastContainer />
    <LoadingOverlay />
    <Modals />
    <FilterDrawer />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAppStore } from '@/composables/useApp'
import * as storage from '@/services/storage'
import ConnectionPanel from '@/components/ConnectionPanel.vue'
import SearchSection from '@/components/SearchSection.vue'
import ResultsTable from '@/components/ResultsTable.vue'
import ToastContainer from '@/components/ToastContainer.vue'
import LoadingOverlay from '@/components/LoadingOverlay.vue'
import Modals from '@/components/Modals.vue'
import FilterDrawer from '@/components/FilterDrawer.vue'
import AssetManagement from '@/components/AssetManagement.vue'

const store = useAppStore()
const theme = ref<'dark' | 'light'>('dark')

function toggleTheme() {
  theme.value = theme.value === 'dark' ? 'light' : 'dark'
  storage.setTheme(theme.value)
  document.documentElement.setAttribute('data-theme', theme.value)
}

onMounted(() => {
  const saved = storage.getTheme()
  theme.value = saved
  document.documentElement.setAttribute('data-theme', saved)
})
</script>
