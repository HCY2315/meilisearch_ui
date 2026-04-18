<template>
  <div class="admin-panel">

    <div class="card">
      <div class="card-header">
        <h3>1. 索引锁库配置 (Index Security)</h3>
        <p style="font-size: 12px; color: #888; margin-top: 4px;">配置哪些索引是公开可见的，哪些是被加锁隐藏的。</p>
      </div>
      <div class="card-body">
        <button class="btn btn-primary" style="margin-bottom: 12px;" @click="showAddIndexConf = true">配置指定索引加密</button>
        <div v-if="showAddIndexConf" class="edit-box" style="margin-bottom: 15px;">
           <select v-model="newIndex.uid" class="form-control" style="width: 180px; display: inline-block; margin-right: 8px;">
              <option disabled value="">-- 选择要加密的索引 --</option>
              <option v-for="uid in availableIndexes" :key="uid" :value="uid">{{ uid }}</option>
           </select>
           <input v-model="newIndex.alias" placeholder="别名备注" class="form-control" style="width: 150px; display: inline-block; margin-right: 8px;">
           <label style="margin-right: 12px; font-size: 14px;">
               <input type="checkbox" v-model="newIndex.isLocked"> 设置为私有锁定
           </label>
           <button class="btn btn-secondary btn-sm" @click="saveIndexConfig">保存配置</button>
           <button class="btn btn-secondary btn-sm" @click="showAddIndexConf = false" style="margin-left: 8px;">取消</button>
        </div>

        <table class="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>索引标识 (UID)</th>
              <th>别名/备注</th>
              <th>对外状态</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cfg in indexConfigs" :key="cfg.id">
              <td>{{ cfg.id }}</td>
              <td><b>{{ cfg.uid }}</b></td>
              <td>{{ cfg.alias || '-' }}</td>
              <td>
                <span :class="['status-badge', cfg.isLocked ? 'status-err' : 'status-ok']">
                  {{ cfg.isLocked ? '🔒 已加锁 (凭证可见)' : '🌐 完全公开' }}
                </span>
              </td>
              <td>
                <button class="btn btn-primary btn-sm" @click="toggleIndexLock(cfg)">切换锁定状态</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="card" style="margin-top: 20px;">
      <div class="card-header">
        <h3>2. 访问凭证分发 (Access Tokens)</h3>
        <p style="font-size: 12px; color: #888; margin-top: 4px;">为“已加锁”的私有库派发解锁令牌。访问者在前台输入令牌即可跨越屏障。</p>
      </div>
      <div class="card-body">
        <button class="btn btn-primary" style="margin-bottom: 12px;" @click="showAddToken = true">派发新 Token</button>
        <div v-if="showAddToken" class="edit-box" style="margin-bottom: 15px;">
           <input v-model="newToken.token" placeholder="自定义 Token 字符串" class="form-control" style="width: 200px; display: inline-block; margin-right: 8px;">
           <input v-model="newToken.allowIndexes" placeholder='解锁目标 (例如: ["movies", "books"])' class="form-control" style="width: 250px; display: inline-block; margin-right: 8px;">
           <input v-model="newToken.description" placeholder="拥有者备注" class="form-control" style="width: 150px; display: inline-block; margin-right: 8px;">
           <button class="btn btn-secondary btn-sm" @click="createToken">生成</button>
           <button class="btn btn-secondary btn-sm" @click="showAddToken = false" style="margin-left: 8px;">取消</button>
        </div>

        <table class="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>凭证口令 (Token)</th>
              <th>解锁的私密库 (UIDs)</th>
              <th>备注下发对象</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="tok in accessTokens" :key="tok.id">
              <td>{{ tok.id }}</td>
              <td><code style="color: #ffb86c">{{ tok.token }}</code></td>
              <td><code>{{ tok.allowIndexes }}</code></td>
              <td>{{ tok.description }}</td>
              <td>
                <button class="btn btn-danger btn-sm" @click="deleteToken(tok.id)">吊销</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="card" style="margin-top: 20px;">
      <div class="card-header">
        <h3>3. 基础页面设置 (App Config)</h3>
      </div>
      <div class="card-body">
        <table class="data-table">
          <thead>
            <tr>
              <th>应用名称</th>
              <th>全局 UI 配置</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="app in apps" :key="app.id">
              <td>{{ app.name }}</td>
              <td class="code-cell" :title="app.uiConfig">{{ app.uiConfig }}</td>
              <td>
                <button class="btn btn-primary btn-sm" @click="editApp(app)">更新皮肤/名称</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

const indexConfigs = ref<any[]>([])
const accessTokens = ref<any[]>([])
const apps = ref<any[]>([])
const availableIndexes = ref<string[]>([])

const showAddIndexConf = ref(false)
const newIndex = ref({ uid: '', alias: '', isLocked: false })

const showAddToken = ref(false)
const newToken = ref({ token: '', allowIndexes: '[]', description: '' })

async function loadAdminData() {
  const token = localStorage.getItem('authToken')
  if (!token) return

  try {
    const headers = { 'Authorization': `Bearer ${token}` }
    const [resIdx, resTok, resApps, resActual] = await Promise.all([
      fetch('/api/v1/admin/index_configs', { headers }),
      fetch('/api/v1/admin/access_tokens', { headers }),
      fetch('/api/v1/admin/apps', { headers }),
      fetch('/api/v1/proxy/indexes', { headers })
    ])

    if (resIdx.ok) indexConfigs.value = await resIdx.json()
    if (resTok.ok) accessTokens.value = await resTok.json()
    if (resApps.ok) apps.value = await resApps.json()
    if (resActual.ok) {
       const body = await resActual.json()
       if(body && body.results) {
         availableIndexes.value = body.results.map((r: any) => r.uid)
       }
    } else {
       console.error('Fetch actual indexes failed:', resActual.status)
    }
  } catch (e) {
    console.error('Admin Data Load Error:', e)
  }
}

async function saveIndexConfig() {
    if (!newIndex.value.uid) return alert('请先从下拉列表中选择一个索引')
    submitIndexConfig(newIndex.value)
    showAddIndexConf.value = false
}

function toggleIndexLock(cfg: any) {
    submitIndexConfig({ uid: cfg.uid, alias: cfg.alias, isLocked: !cfg.isLocked })
}

async function submitIndexConfig(payload: any) {
    const token = localStorage.getItem('authToken')
    await fetch(`/api/v1/admin/index_configs`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
    })
    loadAdminData()
}

async function createToken() {
  if (!newToken.value.token) return alert('请填入Token字符串')
  const token = localStorage.getItem('authToken')
  const res = await fetch(`/api/v1/admin/access_tokens`, {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
      body: JSON.stringify(newToken.value)
  })
  if (res.ok) {
      alert('令牌下发成功')
      showAddToken.value = false
      newToken.value = { token: '', allowIndexes: '[]', description: '' }
      loadAdminData()
  }
}

async function deleteToken(id: number) {
   if(!confirm('确定吊销该令牌？前台正在使用该令牌的用户将立即失去访问权。')) return
   const token = localStorage.getItem('authToken')
   await fetch(`/api/v1/admin/access_tokens/${id}`, {
      method: 'DELETE',
      headers: { 'Authorization': `Bearer ${token}` }
   })
   loadAdminData()
}

function editApp(app: any) {
  const newConfig = prompt('编辑 UI 配置 (JSON 格式):', app.uiConfig)
  if (newConfig !== null) {
      updateApp(app.id, newConfig)
  }
}

async function updateApp(id: number, uiConfig: string) {
  const token = localStorage.getItem('authToken')
  await fetch(`/api/v1/admin/apps/${id}`, {
      method: 'PUT',
      headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
      body: JSON.stringify({ uiConfig })
  })
  loadAdminData()
}

onMounted(() => {
  loadAdminData()
})
</script>

<style scoped>
.admin-panel {
  padding: 16px;
  max-width: 1200px;
  margin: 0 auto;
}
.card {
  background: var(--surface);
  border-radius: var(--radius, 8px);
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  overflow: hidden;
}
.card-header {
  padding: 16px;
  background: rgba(0, 0, 0, 0.05);
  border-bottom: 1px solid var(--border-color, #444);
}
.card-header h3 {
  margin: 0;
  font-size: 16px;
  color: var(--text);
}
.card-body {
  padding: 16px;
}
.data-table {
  width: 100%;
  border-collapse: collapse;
}
.data-table th, .data-table td {
  padding: 12px;
  text-align: left;
  border-bottom: 1px solid var(--border-color, #444);
  color: var(--text);
}
.data-table th {
  font-weight: 600;
  color: var(--text-muted, #aaa);
}
.status-badge {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: bold;
  background: rgba(255,255,255,0.1);
}
.status-ok {
  background: #28c84022;
  color: #28c840;
}
.status-err {
  background: #ff4a4a22;
  color: #ff4a4a;
}
.code-cell {
  max-width: 300px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: monospace;
}
.edit-box {
  background: rgba(0, 0, 0, 0.2);
  padding: 12px;
  border-radius: 8px;
}
.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}
.btn-danger {
  background: #ff4a4a;
  color: white;
  border: none;
  cursor: pointer;
}
.form-control {
  padding: 8px 12px;
  border: 1px solid var(--border-color, #444);
  border-radius: var(--radius, 4px);
  background: var(--input-bg, #2a2a2e);
  color: var(--text, #fff);
  outline: none;
  transition: border-color 0.2s;
}
.form-control:focus {
  border-color: var(--primary, #6366f1);
}
select.form-control {
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='white'%3E%3Cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'%3E%3C/path%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  background-size: 16px;
  padding-right: 32px;
}
option {
  background: #2a2a2e;
  color: #fff;
}
</style>
