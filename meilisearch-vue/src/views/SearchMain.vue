<template>
  <div class="container" :data-theme="theme">
    <header class="header">
      <div class="header-top">
        <!-- 页面标题从后端配置读取，这里做个默认值保底 -->
        <h1>{{ uiConfig.title || '🔍 多维查询系统' }}</h1>
        <button class="theme-toggle" @click="toggleTheme">
          {{ theme === 'dark' ? '🌙 夜间模式' : '☀️ 日间模式' }}
        </button>
        <button v-if="isLoggedIn" class="btn btn-secondary" @click="logout" style="margin-left: 10px;">
          退出登录
        </button>
      </div>
      <p>强大的多维度搜索与过滤功能，快速定位您需要的内容</p>
    </header>

    <nav class="tab-nav">
      <button
        :class="['tab-btn', store.currentTab === 'search' && 'active']"
        @click="store.setCurrentTab('search')"
      >🔍 搜索</button>
      <!-- 根据权限控制显示 -->
      <button
        v-if="userRole === 'admin'"
        :class="['tab-btn', store.currentTab === 'assets' && 'active']"
        @click="store.setCurrentTab('assets')"
      >📦 批量新增</button>
      <button
        v-if="userRole === 'admin'"
        :class="['tab-btn', store.currentTab === 'admin' && 'active']"
        @click="store.setCurrentTab('admin')"
      >⚙️ 系统管理</button>
    </nav>

    <div v-if="initLoading" style="padding: 20px; text-align: center;">
      <p>加载配置中...</p>
    </div>
    <template v-else>
      <template v-if="store.currentTab === 'search'">
        <ConnectionPanel />
        <SearchSection />
        <ResultsTable />
      </template>
      <template v-else-if="store.currentTab === 'assets'">
        <AssetManagement />
      </template>
      <template v-else-if="store.currentTab === 'admin'">
        <AdminPanel />
      </template>
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
import { useRouter } from 'vue-router'
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
import AdminPanel from '@/components/AdminPanel.vue'

const store = useAppStore()
const router = useRouter()
const theme = ref<'dark' | 'light'>('dark')
const uiConfig = ref<any>({})
const userRole = ref('user')
const isLoggedIn = ref(false)
const initLoading = ref(true)

function toggleTheme() {
  theme.value = theme.value === 'dark' ? 'light' : 'dark'
  storage.setTheme(theme.value)
  document.documentElement.setAttribute('data-theme', theme.value)
}

function logout() {
  localStorage.removeItem('authToken')
  localStorage.removeItem('authUser')
  isLoggedIn.value = false
  userRole.value = 'user'
  // 退出后保持在首页或者刷新即可，不再强制跳到独立登录页
  window.location.reload()
}

onMounted(async () => {
  const saved = storage.getTheme()
  theme.value = saved
  document.documentElement.setAttribute('data-theme', saved)

  const authUserStr = localStorage.getItem('authUser')
  if (authUserStr) {
    try {
      const authUser = JSON.parse(authUserStr)
      userRole.value = authUser.role || 'user'
      isLoggedIn.value = true
    } catch {}
  }

  // 从后端获取配置
  const token = localStorage.getItem('authToken')
  try {
    const res = await fetch('http://localhost:8080/api/v1/app/config', {
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })
    if (res.ok) {
      const data = await res.json()
      if (data.uiConfig) {
        try {
          uiConfig.value = JSON.parse(data.uiConfig)
          if (uiConfig.value.theme) {
            theme.value = uiConfig.value.theme
            document.documentElement.setAttribute('data-theme', theme.value)
          }
        } catch {}
      }
      
      // 这里的 meili 配置自动注入 store 并执行连接
      if (data.meili && data.meili.host && data.meili.searchToken) {
        store.hostInput = data.meili.host
        store.apiKeyInput = data.meili.searchToken
        await store.connect()
      }
    }
  } catch (err) {
    console.error("加载配置失败:", err)
  } finally {
    initLoading.value = false
  }
})
</script>
