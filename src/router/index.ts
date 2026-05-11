/// <reference types="vite/client" />
import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import Login from '../views/Login.vue'
import SearchMain from '../views/SearchMain.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/admin',
      name: 'admin-login',
      component: Login
    },
    {
      path: '/',
      name: 'home',
      component: SearchMain
    }
  ]
})

router.beforeEach((to, _from, next) => {
  const token = localStorage.getItem('authToken')
  // 如果已经登录过，还想访问登录页，就直接回到首页
  if (to.path === '/admin' && token) {
    next('/')
  } else {
    next()
  }
})

export default router
