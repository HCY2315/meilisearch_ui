<template>
  <div class="admin-panel">
    <div class="card">
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
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="app in apps" :key="app.id">
              <td>{{ app.id }}</td>
              <td>{{ app.name }}</td>
              <td><code>{{ app.appKey }}</code></td>
              <td class="code-cell" :title="app.uiConfig">{{ app.uiConfig }}</td>
              <td>
                <button class="btn btn-primary btn-sm" @click="editApp(app)">快速配置</button>
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

async function loadAdminData() {
  const token = localStorage.getItem('authToken')
  if (!token) return

  try {
    const resInst = await fetch('http://localhost:8080/api/v1/admin/instances', {
      headers: { 'Authorization': `Bearer ${token}` }
    })
    if (resInst.ok) {
      instances.value = await resInst.json()
    }
    
    const resApps = await fetch('http://localhost:8080/api/v1/admin/apps', {
      headers: { 'Authorization': `Bearer ${token}` }
    })
    if (resApps.ok) {
      apps.value = await resApps.json()
    }
  } catch (e) {
    console.error('加载管理数据失败', e)
  }
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
        headers: { 
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ uiConfig })
     })
     if (res.ok) {
         alert('配置更新成功，刷新页面生效！')
         loadAdminData()
     } else {
         alert('更新失败！')
     }
  } catch (e) {
      alert('网络错误')
  }
}

onMounted(() => {
  loadAdminData()
})
</script>

<style scoped>
.admin-panel {
  padding: 16px;
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
.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}
</style>
