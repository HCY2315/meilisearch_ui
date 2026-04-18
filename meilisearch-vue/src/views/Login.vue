<template>
  <div class="login-container">
    <div class="login-box">
      <h2>后台登录</h2>
      <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>
      <div class="form-group">
        <label>用户名</label>
        <input class="form-control" v-model="username" type="text" placeholder="请输入用户名" @keyup.enter="handleLogin" />
      </div>
      <div class="form-group">
        <label>密 码</label>
        <input class="form-control" v-model="password" type="password" placeholder="请输入密码" @keyup.enter="handleLogin" />
      </div>
      <button class="btn btn-primary" @click="handleLogin" :disabled="loading" style="width: 100%; margin-top: 10px;">
        {{ loading ? '登录中...' : '登 录' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const username = ref('')
const password = ref('')
const loading = ref(false)
const errorMsg = ref('')

async function handleLogin() {
  if (!username.value || !password.value) {
    errorMsg.value = '请输入用户名和密码'
    return
  }
  loading.value = true
  errorMsg.value = ''
  try {
    const res = await fetch('http://localhost:8080/api/v1/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: username.value, password: password.value })
    })
    const data = await res.json()
    if (!res.ok) {
      throw new Error(data.error || '登录失败')
    }
    localStorage.setItem('authToken', data.token)
    localStorage.setItem('authUser', JSON.stringify(data.user))
    router.push('/')
  } catch (err: any) {
    errorMsg.value = err.message
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
  background-color: var(--background, #1e1e1e);
  color: var(--text, #ececec);
}
.login-box {
  background: var(--surface, #2c2c2c);
  padding: 30px;
  border-radius: var(--radius, 8px);
  width: 350px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.15);
}
.login-box h2 {
  text-align: center;
  margin-bottom: 20px;
  color: var(--text, #ececec);
}
.form-group {
  margin-bottom: 15px;
}
.form-group label {
  display: block;
  margin-bottom: 5px;
  font-size: 14px;
}
.form-control {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border-color, #444);
  border-radius: var(--radius, 4px);
  background: var(--input-bg, #1a1a1a);
  color: var(--text, #fff);
}
.error-msg {
  color: #ff4a4a;
  margin-bottom: 15px;
  font-size: 14px;
  text-align: center;
}
</style>
