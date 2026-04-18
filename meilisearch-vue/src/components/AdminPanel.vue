<template>
  <div class="admin-panel">

    <div class="card">
      <div class="card-header">
        <h3>用户与权限管理 (Users)</h3>
      </div>
      <div class="card-body">
        <button class="btn btn-primary" style="margin-bottom: 12px;" @click="showAddUser = true">新增用户</button>
        <div v-if="showAddUser" class="edit-box" style="margin-bottom: 15px;">
           <input v-model="newUser.username" placeholder="登录用户名" class="form-control" style="width: 150px; display: inline-block; margin-right: 8px;">
           <input v-model="newUser.password" type="password" placeholder="密码" class="form-control" style="width: 150px; display: inline-block; margin-right: 8px;">
           <select v-model="newUser.role" class="form-control" style="width: 100px; display: inline-block; margin-right: 8px;">
             <option value="user">普通用户</option>
             <option value="admin">管理员</option>
           </select>
           <input v-model="newUser.allowIndexes" placeholder='允许的索引(如: ["movies", "books"])' class="form-control" style="width: 250px; display: inline-block; margin-right: 8px;">
           <button class="btn btn-secondary btn-sm" @click="createUser">确认新建</button>
           <button class="btn btn-secondary btn-sm" @click="showAddUser = false" style="margin-left: 8px;">取消</button>
        </div>

        <table class="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>用户名</th>
              <th>角色</th>
              <th>数据源(Index)权限范围</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="u in users" :key="u.id">
              <td>{{ u.id }}</td>
              <td>{{ u.username }}</td>
              <td>
                <span :class="['status-badge', u.role === 'admin' ? 'status-ok' : '']">
                  {{ u.role === 'admin' ? '超级管理员' : '普通用户' }}
                </span>
              </td>
              <td><code>{{ u.allowIndexes || '[]' }}</code></td>
              <td>
                <button class="btn btn-primary btn-sm" @click="editUserPerms(u)" :disabled="u.username === 'admin'">修改权限</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="card" style="margin-top: 20px;">
      <div class="card-header">
        <h3>平台实例管理 (Instances)</h3>
      </div>
      <div class="card-body">
        <table class="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>名称</th>
              <th>主机地址</th>
              <th>状态</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="inst in instances" :key="inst.id">
              <td>{{ inst.id }}</td>
              <td>{{ inst.name }}</td>
              <td>{{ inst.host }}</td>
              <td>
                <span :class="['status-badge', inst.status === 1 ? 'status-ok' : 'status-err']">
                  {{ inst.status === 1 ? '正常' : '已停用' }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="card" style="margin-top: 20px;">
      <div class="card-header">
        <h3>前台应用配置 (Applications)</h3>
      </div>
      <div class="card-body">
        <table class="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>应用名称</th>
              <th>App Key (调用凭证)</th>
              <th>UI配置</th>
              <th>全局允许的Index</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="app in apps" :key="app.id">
              <td>{{ app.id }}</td>
              <td>{{ app.name }}</td>
              <td><code>{{ app.appKey }}</code></td>
              <td class="code-cell" :title="app.uiConfig">{{ app.uiConfig }}</td>
              <td class="code-cell">{{ app.allowIndexes }}</td>
              <td>
                <button class="btn btn-primary btn-sm" @click="editApp(app)">更新UI配置</button>
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

const instances = ref<any[]>([])
const apps = ref<any[]>([])
const users = ref<any[]>([])

const showAddUser = ref(false)
const newUser = ref({ username: '', password: '', role: 'user', allowIndexes: '[]' })

async function loadAdminData() {
  const token = localStorage.getItem('authToken')
  if (!token) return

  try {
    const headers = { 'Authorization': `Bearer ${token}` }
    const [resInst, resApps, resUsers] = await Promise.all([
      fetch('http://localhost:8080/api/v1/admin/instances', { headers }),
      fetch('http://localhost:8080/api/v1/admin/apps', { headers }),
      fetch('http://localhost:8080/api/v1/admin/users', { headers })
    ])

    if (resInst.ok) instances.value = await resInst.json()
    if (resApps.ok) apps.value = await resApps.json()
    if (resUsers.ok) users.value = await resUsers.json()

  } catch (e) {
    console.error('加载管理数据失败', e)
  }
}

async function createUser() {
  if (!newUser.value.username || !newUser.value.password) {
      alert("请填写完整信息")
      return
  }
  const token = localStorage.getItem('authToken')
  try {
      const res = await fetch(`http://localhost:8080/api/v1/admin/users`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
        body: JSON.stringify(newUser.value)
      })
      if (res.ok) {
          alert('用户新建成功！')
          showAddUser.value = false
          newUser.value = { username: '', password: '', role: 'user', allowIndexes: '[]' }
          loadAdminData()
      } else {
          alert('新建失败，可能是用户名重复')
      }
  } catch (e) { alert('网络错误') }
}

function editUserPerms(u: any) {
  const newPerms = prompt(`修改用户 ${u.username} 允许访问的索引(JSON Array):`, u.allowIndexes || '[]')
  if (newPerms !== null) {
      updateUserPerms(u.id, u.role, newPerms)
  }
}

async function updateUserPerms(id: number, role: string, allowIndexes: string) {
  const token = localStorage.getItem('authToken')
  try {
     const res = await fetch(`http://localhost:8080/api/v1/admin/users/${id}/permissions`, {
        method: 'PUT',
        headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
        body: JSON.stringify({ role, allowIndexes })
     })
     if (res.ok) { loadAdminData(); alert('修改成功') }
  } catch (e) {}
}

function editApp(app: any) {
  const newConfig = prompt('编辑 UI 配置 (JSON 格式):', app.uiConfig)
  if (newConfig !== null) {
      updateApp(app.id, newConfig)
  }
}

async function updateApp(id: number, uiConfig: string) {
  const token = localStorage.getItem('authToken')
  try {
     const res = await fetch(`http://localhost:8080/api/v1/admin/apps/${id}`, {
        method: 'PUT',
        headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
        body: JSON.stringify({ uiConfig })
     })
     if (res.ok) {
         alert('配置更新成功，刷新页面生效！')
         loadAdminData()
     } else {
         alert('更新失败！')
     }
  } catch (e) { alert('网络错误') }
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
  max-width: 200px;
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
.form-control {
  padding: 8px 12px;
  border: 1px solid var(--border-color, #444);
  border-radius: var(--radius, 4px);
  background: var(--input-bg, #1a1a1a);
  color: var(--text, #fff);
}
</style>
