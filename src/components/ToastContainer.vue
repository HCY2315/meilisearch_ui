<template>
  <div class="toast-container">
    <TransitionGroup name="toast">
      <div v-for="toast in store.toasts" :key="toast.id" :class="['toast', toast.kind]">
        <span class="toast-icon">{{ icons[toast.kind as ToastType] }}</span>
        <span class="toast-msg">{{ toast.message }}</span>
        <button class="toast-close" title="关闭" @click="store.removeToast(toast.id)">×</button>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import { useAppStore } from '@/composables/useApp'
import type { ToastType } from '@/types'

const store = useAppStore()
const icons: Record<ToastType, string> = {
  success: '✅',
  error: '❌',
  warning: '⚠️',
  info: 'ℹ️',
}
</script>

<style scoped>
.toast-container {
  position: fixed;
  right: 20px;
  bottom: 24px;
  z-index: 10000;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  /* 容器本身不拦截点击，避免遮挡页面按钮 */
  pointer-events: none;
}
.toast {
  pointer-events: auto;
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 36px 12px 20px;
  border-radius: 8px;
  font-size: 14px;
  min-width: 280px;
  max-width: 400px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
.toast-msg { flex: 1; word-break: break-word; }
.toast-icon { flex-shrink: 0; }
.toast-close {
  position: absolute;
  top: 6px;
  right: 8px;
  width: 20px;
  height: 20px;
  line-height: 18px;
  text-align: center;
  border: none;
  background: transparent;
  color: inherit;
  opacity: 0.6;
  font-size: 16px;
  cursor: pointer;
  border-radius: 4px;
}
.toast-close:hover { opacity: 1; background: rgba(255,255,255,0.15); }
.toast.success { background: #065f46; color: #6ee7b7; }
.toast.error { background: #7f1d1d; color: #fca5a5; }
.toast.warning { background: #78350f; color: #fcd34d; }
.toast.info { background: #1e3a5f; color: #93c5fd; }
.toast-enter-active, .toast-leave-active { transition: all 0.3s ease; }
.toast-enter-from { opacity: 0; transform: translateX(100%); }
.toast-leave-to { opacity: 0; transform: translateX(100%); }
</style>
