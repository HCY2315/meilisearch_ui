import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import { setGlobalErrorHandler } from './services/api'
import { useAppStore } from './composables/useApp'
import App from './App.vue'
import './style.css'

const app = createApp(App)
app.use(createPinia())
app.use(router)

setGlobalErrorHandler((err) => {
  // 401/403 属权限/锁定类，通常由调用方给出更友好的提示，这里不再重复弹英文错误
  const status = (err as { response?: { status?: number } })?.response?.status
  if (status === 401 || status === 403) return
  const store = useAppStore()
  store.pushToast(err.message, 'error')
})

app.mount('#app')
