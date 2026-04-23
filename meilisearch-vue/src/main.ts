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
  const store = useAppStore()
  store.pushToast(err.message, 'error')
})

app.mount('#app')
