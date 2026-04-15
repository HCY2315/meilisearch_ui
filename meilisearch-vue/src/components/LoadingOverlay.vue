<template>
  <div v-if="store.loading" class="loading-overlay">
    <div class="spinner"></div>
    <p>加载中...</p>
  </div>
</template>

<script setup lang="ts">
import { useAppStore } from '@/composables/useApp'
const store = useAppStore()
</script>

<style scoped>
.loading-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  z-index: 9998;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
}
.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255,255,255,0.1);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
