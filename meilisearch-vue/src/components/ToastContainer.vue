<template>
  <div class="toast-container">
    <TransitionGroup name="toast">
      <div v-for="toast in store.toasts" :key="toast.id" :class="['toast', toast.kind]">
        <span class="toast-icon">{{ icons[toast.kind as ToastType] }}</span>
        <span>{{ toast.message }}</span>
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
  top: 20px;
  right: 20px;
  z-index: 10000;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.toast {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  border-radius: 8px;
  font-size: 14px;
  min-width: 280px;
  max-width: 400px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
.toast.success { background: #065f46; color: #6ee7b7; }
.toast.error { background: #7f1d1d; color: #fca5a5; }
.toast.warning { background: #78350f; color: #fcd34d; }
.toast.info { background: #1e3a5f; color: #93c5fd; }
.toast-enter-active, .toast-leave-active { transition: all 0.3s ease; }
.toast-enter-from { opacity: 0; transform: translateX(100%); }
.toast-leave-to { opacity: 0; transform: translateX(100%); }
</style>
