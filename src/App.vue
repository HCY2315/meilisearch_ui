<template>
  <router-view />
  <ToastContainer />
</template>

<script setup lang="ts">
import ToastContainer from '@/components/ToastContainer.vue'
</script>

<style>
/* 如果有全局必须放在 App.vue 的样式可以保留在这里 */
</style>
