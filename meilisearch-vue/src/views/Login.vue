<template>
  <div class="login-wrapper">
    <!-- 动态背景装饰 -->
    <div class="mesh-gradient"></div>
    <div class="blob-1"></div>
    <div class="blob-2"></div>

    <div class="login-card animate-fade-in">
      <div class="login-header">
        <div class="logo-icon">🔑</div>
        <h1>系统管理后台</h1>
        <p>请输入管理员凭据以继续</p>
      </div>

      <div v-if="errorMsg" class="error-banner">
        <span class="icon">⚠️</span> {{ errorMsg }}
      </div>

      <div class="form">
        <div class="input-group">
          <label>用户名</label>
          <div class="input-wrapper">
            <span class="input-icon">👤</span>
            <input v-model="username" type="text" placeholder="Admin Username" @keyup.enter="handleLogin" />
          </div>
        </div>

        <div class="input-group">
          <label>密码</label>
          <div class="input-wrapper">
            <span class="input-icon">🔒</span>
            <input v-model="password" type="password" placeholder="••••••••" @keyup.enter="handleLogin" />
          </div>
        </div>

        <button class="login-btn" @click="handleLogin" :disabled="loading">
          <span v-if="loading" class="loader"></span>
          {{ loading ? '验证中...' : '立即登录' }}
        </button>
      </div>

      <div class="login-footer">
        © 2026 Meilisearch Pro UI · Powered by Antigravity
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { login } from '@/services/api'

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
    const data = await login(username.value, password.value)
    localStorage.setItem('authToken', data.token)
    localStorage.setItem('authUser', JSON.stringify(data.user))
    router.push('/')
  } catch (err: unknown) {
    errorMsg.value = err instanceof Error ? err.message : '登录失败'
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-wrapper {
  position: relative;
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
  width: 100vw;
  background: #030712;
  overflow: hidden;
}

/* 动态背景 */
.mesh-gradient {
  position: absolute;
  inset: 0;
  background: 
    radial-gradient(circle at 20% 20%, rgba(99, 102, 241, 0.15) 0%, transparent 40%),
    radial-gradient(circle at 80% 80%, rgba(6, 182, 212, 0.15) 0%, transparent 40%);
  filter: blur(80px);
}

.blob-1, .blob-2 {
  position: absolute;
  width: 500px;
  height: 500px;
  background: var(--primary);
  opacity: 0.1;
  filter: blur(100px);
  border-radius: 50%;
  animation: float 20s infinite alternate;
}
.blob-1 { top: -100px; left: -100px; }
.blob-2 { bottom: -100px; right: -100px; animation-delay: -10s; }

@keyframes float {
  from { transform: translate(0, 0) scale(1); }
  to { transform: translate(100px, 50px) scale(1.1); }
}

/* 登录卡片 */
.login-card {
  position: relative;
  z-index: 10;
  width: min(420px, 90%);
  background: rgba(15, 23, 42, 0.6);
  backdrop-filter: blur(24px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 24px;
  padding: 40px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
}

.login-header {
  text-align: center;
  margin-bottom: 32px;
}
.logo-icon {
  font-size: 48px;
  margin-bottom: 16px;
  filter: drop-shadow(0 0 10px var(--primary-glow));
}
.login-header h1 {
  font-family: 'Outfit', sans-serif;
  font-size: 28px;
  font-weight: 700;
  background: linear-gradient(to right, #fff, #94a3b8);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  margin-bottom: 8px;
}
.login-header p {
  color: #94a3b8;
  font-size: 14px;
}

.error-banner {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #f87171;
  padding: 12px;
  border-radius: 12px;
  margin-bottom: 24px;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.input-group label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: #94a3b8;
  margin-bottom: 8px;
  padding-left: 4px;
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}
.input-icon {
  position: absolute;
  left: 14px;
  font-size: 16px;
  opacity: 0.5;
}
.input-wrapper input {
  width: 100%;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.08);
  padding: 14px 14px 14px 42px;
  border-radius: 14px;
  color: white;
  font-size: 15px;
  transition: all 0.3s ease;
}
.input-wrapper input:focus {
  border-color: var(--primary);
  background: rgba(0, 0, 0, 0.4);
  box-shadow: 0 0 0 4px rgba(99, 102, 241, 0.1);
  outline: none;
}

.login-btn {
  background: linear-gradient(135deg, var(--primary), #4f46e5);
  color: white;
  padding: 16px;
  border-radius: 14px;
  border: none;
  font-weight: 700;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
  margin-top: 10px;
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.3);
}
.login-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 20px 25px -5px rgba(99, 102, 241, 0.4);
}
.login-btn:active {
  transform: translateY(0);
}

.login-footer {
  text-align: center;
  margin-top: 32px;
  font-size: 11px;
  color: #475569;
  letter-spacing: 0.05em;
}

.loader {
  width: 18px;
  height: 18px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  border-top-color: #fff;
  animation: spin 0.8s linear infinite;
  display: inline-block;
  margin-right: 8px;
  vertical-align: middle;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}

/* ============ Mobile Responsiveness ============ */
@media (max-width: 480px) {
  .login-card {
    padding: 24px;
    width: 95%;
  }
  
  .login-header h1 {
    font-size: 24px;
  }
  
  .input-wrapper input {
    padding: 12px 12px 12px 36px;
    font-size: 14px;
  }
  
  .login-btn {
    padding: 14px;
    font-size: 15px;
  }
}
</style>
