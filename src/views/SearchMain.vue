<template>
  <div class="main-layout" :data-theme="theme">
    <!-- 高级背景装饰 -->
    <div class="bg-blur-1"></div>
    <div class="bg-blur-2"></div>

    <header class="app-header animate-fade-in">
      <div class="header-content">
        <div class="header-left">
          <div class="brand-logo">🔍</div>
          <div class="brand-text">
            <h1>{{ uiConfig.title || 'Meilisearch Pro' }}</h1>
            <p>高性能多维数据检索引擎</p>
          </div>
        </div>

        <div class="header-actions">
          <button class="glass-btn theme-toggle" @click="toggleTheme" :title="theme === 'dark' ? '切换日间' : '切换夜间'">
            {{ theme === 'dark' ? '🌙' : '☀️' }}
          </button>
          
          <div v-if="isLoggedIn" class="user-pill">
            <span class="role-badge">{{ userRole === 'admin' ? '管理员' : '访客' }}</span>
            <button class="logout-btn" @click="logout" title="退出登录">退出</button>© 2026 Meilisearch Pro UI · Powered by Antigravity
          </div>
          <button v-else class="btn btn-primary btn-sm" @click="router.push('/admin')">登录后台</button>
        </div>
      </div>

      <nav class="app-nav">
        <div class="nav-pill-group">
          <button
            :class="['nav-pill', store.currentTab === 'search' && 'active']"
            @click="store.setCurrentTab('search')"
          >
            <span class="icon">🔍</span> 搜索
          </button>
          <button
            v-if="userRole === 'admin'"
            :class="['nav-pill', store.currentTab === 'assets' && 'active']"
            @click="store.setCurrentTab('assets')"
          >
            <span class="icon">📦</span> 资产导入
          </button>
          <button
            v-if="userRole === 'admin'"
            :class="['nav-pill', store.currentTab === 'admin' && 'active']"
            @click="store.setCurrentTab('admin')"
          >
            <span class="icon">⚙️</span> 系统配置
          </button>
        </div>

        <div class="nav-extra" v-if="store.currentTab === 'search'">
          <div class="token-unlock-pill">
            <span class="icon">🔑</span>
            <input v-model="lockToken" placeholder="输入访问密钥解锁私有库" @keyup.enter="applyLockToken">
            <button @click="applyLockToken">解锁</button>
          </div>
        </div>
      </nav>
    </header>

    <main class="app-container">
      <div v-if="initLoading" class="loading-state">
        <div class="premium-loader"></div>
        <p>正在初始化搜索引擎配置...</p>
      </div>
      
      <div v-else class="content-wrapper animate-fade-in">
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
      </div>
    </main>

    <footer class="app-footer">
      <div class="footer-divider"></div>
      <p>© 2026 多维搜索引擎 - 数据集散中心 · Powered by 冰城拓</p>
    </footer>

    <!-- 全局挂载组件 -->
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
import { getAppConfig } from '@/services/api'

const store = useAppStore()
const router = useRouter()
const theme = ref<'dark' | 'light'>('dark')
const uiConfig = ref<any>({})
const userRole = ref('user')
const isLoggedIn = ref(false)
const initLoading = ref(true)
const lockToken = ref(localStorage.getItem('App-Token') || '')

function applyLockToken() {
  localStorage.setItem('App-Token', lockToken.value)
  window.location.reload()
}

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

  try {
    const data = await getAppConfig()
    if (data) {
      if (data.uiConfig) {
        try {
          uiConfig.value = typeof data.uiConfig === 'string' ? JSON.parse(data.uiConfig) : data.uiConfig
          if (uiConfig.value.theme) {
            theme.value = uiConfig.value.theme
            document.documentElement.setAttribute('data-theme', theme.value)
          }
        } catch {}
      }
      
      if (data.meili && (data.meili as any).host) {
        store.hostInput = (data.meili as any).host
        store.apiKeyInput = lockToken.value || localStorage.getItem('authToken') || ''
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

<style scoped>
.main-layout {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow-x: hidden;
}

/* 背景模糊效果 */
.bg-blur-1 { position: absolute; top: -100px; left: -100px; width: 600px; height: 600px; background: rgba(99, 102, 241, 0.05); filter: blur(150px); border-radius: 50%; z-index: -1; }
.bg-blur-2 { position: absolute; bottom: -100px; right: -100px; width: 500px; height: 500px; background: rgba(6, 182, 212, 0.05); filter: blur(150px); border-radius: 50%; z-index: -1; }

/* 页眉 */
.app-header {
  padding: 20px 40px 18px;
  background: rgba(15, 23, 42, 0.6);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  position: sticky;
  top: 0;
  z-index: 100;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.header-left { display: flex; align-items: center; gap: 16px; }
.brand-logo { font-size: 32px; filter: drop-shadow(0 0 10px rgba(99,102,241,0.4)); }
.brand-text h1 { font-family: 'Outfit'; font-size: 24px; font-weight: 700; color: white; margin-bottom: 2px; }
.brand-text p { font-size: 13px; color: #64748b; }

.header-actions { display: flex; align-items: center; gap: 16px; }

.glass-btn {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 8px 12px;
  border-radius: 12px;
  color: white;
  cursor: pointer;
  transition: all 0.3s ease;
}
.glass-btn:hover { background: rgba(255, 255, 255, 0.1); }

.user-pill {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(0, 0, 0, 0.2);
  padding: 4px 4px 4px 12px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}
.role-badge { font-size: 11px; font-weight: 700; color: var(--primary); text-transform: uppercase; }
.logout-btn {
  background: rgba(239, 68, 68, 0.1);
  color: #f87171;
  border: none;
  padding: 6px 12px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

/* 导航 */
.app-nav { display: flex; justify-content: space-between; align-items: center; }

.nav-pill-group {
  display: flex;
  background: rgba(0, 0, 0, 0.2);
  padding: 4px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}
.nav-pill {
  padding: 8px 18px;
  border-radius: 10px;
  border: none;
  background: transparent;
  color: #94a3b8;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
}
.nav-pill:hover { color: white; }
.nav-pill.active { background: var(--primary); color: white; box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3); }

.token-unlock-pill {
  display: flex;
  align-items: center;
  background: rgba(15, 23, 42, 0.8);
  border: 1px solid var(--primary);
  padding: 4px 4px 4px 12px;
  border-radius: 12px;
  box-shadow: 0 0 15px rgba(99, 102, 241, 0.1);
}
.token-unlock-pill input {
  background: transparent;
  border: none;
  color: white;
  padding: 4px 8px;
  font-size: 13px;
  width: 200px;
  outline: none;
}
.token-unlock-pill button {
  background: var(--primary);
  color: white;
  border: none;
  padding: 6px 14px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

/* 主容器 */
.app-container { flex: 1; padding: 24px 40px 30px; }

.content-wrapper {
  width: 100%;
  max-width: 1460px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 400px;
  color: #64748b;
}
.premium-loader {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(99, 102, 241, 0.1);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 20px;
}

.app-footer {
  padding: 32px 48px;
  text-align: center;
  color: #475569;
  font-size: 13px;
}
.footer-divider {
  width: 100%;
  height: 1px;
  background: linear-gradient(to right, transparent, rgba(255,255,255,0.05), transparent);
  margin-bottom: 24px;
}

@keyframes spin { to { transform: rotate(360deg); } }

/* ============ Mobile Layout Responsiveness ============ */
@media (max-width: 768px) {
  .app-header {
    padding: 16px 20px;
  }
  
  .header-content {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }
  
  .header-actions {
    width: 100%;
    justify-content: space-between;
  }
  
  .app-nav {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }
  
  .nav-pill-group {
    justify-content: space-between;
    width: 100%;
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }
  
  .nav-pill {
    padding: 8px 12px;
    font-size: 13px;
    flex-shrink: 0;
  }
  
  .nav-extra {
    width: 100%;
  }
  
  .token-unlock-pill {
    width: 100%;
    justify-content: space-between;
  }
  
  .token-unlock-pill input {
    width: auto;
    flex: 1;
  }
  
  .app-container {
    padding: 16px 20px;
  }
  
  .app-footer {
    padding: 20px;
  }
}
</style>
