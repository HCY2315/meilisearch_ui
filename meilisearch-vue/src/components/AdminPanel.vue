<template>
  <div class="admin-layout animate-fade-in">
    <!-- 侧边导航 -->
    <aside class="admin-sidebar">
      <div class="sidebar-header">
        <div class="sidebar-logo">🛡️</div>
        <h2>控制台</h2>
      </div>
      <nav class="sidebar-nav">
        <button 
          v-for="tab in tabs" 
          :key="tab.id" 
          :class="['nav-item', { active: activeTab === tab.id }]"
          @click="activeTab = tab.id"
        >
          <span class="nav-icon">{{ tab.icon }}</span>
          <span class="nav-label">{{ tab.label }}</span>
        </button>
      </nav>
      <div class="sidebar-footer">
        v1.2.0-pro
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="admin-main">
      <!-- 0. 搜索引擎实例配置 -->
      <section v-if="activeTab === 'instances'" class="content-section">
        <header class="section-header">
          <h1>搜索引擎实例 <span>Instances</span></h1>
          <p>管理多节点连接，支持分布式部署配置。</p>
          <button class="btn btn-primary" @click="showAddInstance = true">+ 添加新实例</button>
        </header>

        <div v-if="showAddInstance" class="glass-editor">
          <h3>{{ editingInstanceId ? '📝 编辑实例' : '✨ 新建实例' }}</h3>
          <div class="grid-inputs">
            <div class="input-group">
              <label>实例名称</label>
              <input v-model="newInstance.name" placeholder="生产环境 / 测试集群" class="form-control">
            </div>
            <div class="input-group">
              <label>Host 地址</label>
              <input v-model="newInstance.host" placeholder="http://77.0.0.1:7700" class="form-control">
            </div>
            <div class="input-group">
              <label>API Key</label>
              <input v-model="newInstance.apiKey" placeholder="Master Key (可选)" class="form-control">
            </div>
          </div>
          <div class="editor-actions">
            <button class="btn btn-primary" @click="handleSaveInstance">{{ editingInstanceId ? '保存更改' : '立即创建' }}</button>
            <button class="btn btn-secondary" @click="cancelInstanceEdit">取消</button>
          </div>
        </div>

        <div class="data-grid">
          <div v-for="ins in instances" :key="ins.id" class="data-card">
            <div class="card-info">
              <div class="ins-avatar">{{ ins.name.charAt(0) }}</div>
              <div>
                <h4>{{ ins.name }}</h4>
                <code>{{ ins.host }}</code>
              </div>
            </div>
            <div class="card-ops">
              <button class="btn btn-primary btn-sm" @click="viewTasks(ins)">任务详情</button>
              <button class="btn btn-secondary btn-sm" @click="editInstance(ins)">编辑</button>
              <button class="btn btn-danger btn-sm" @click="deleteInstance(ins.id)">移除</button>
            </div>
          </div>
        </div>

        <!-- 任务详情 Modal -->
        <div v-if="showTasksModal" class="tasks-modal-overlay animate-fade-in" @click.self="showTasksModal = false">
          <div class="tasks-modal">
            <div class="modal-header">
              <h3>⚡ 任务详情 - {{ currentTaskInstance?.name }}</h3>
              <button class="close-btn" @click="showTasksModal = false">×</button>
            </div>
            <div class="modal-body">
              <div v-if="isLoadingTasks" class="loading-state">
                加载中...
              </div>
              <div v-else-if="instanceTasks.length > 0" class="table-container">
                <table class="data-table">
                  <thead>
                    <tr>
                      <th>任务 UID</th>
                      <th>目标索引</th>
                      <th>类型</th>
                      <th>状态</th>
                      <th>耗时</th>
                      <th>排队时间</th>
                      <th>详情</th>
                    </tr>
                  </thead>
                  <tbody>
                    <template v-for="task in instanceTasks" :key="task.uid">
                      <tr>
                        <td><code>{{ task.uid }}</code></td>
                        <td>{{ task.indexUid || '-' }}</td>
                        <td>{{ task.type }}</td>
                        <td>
                          <span :class="['status-badge', task.status === 'succeeded' ? 'status-ok' : (task.status === 'failed' ? 'status-err' : 'status-warn')]">
                            {{ task.status }}
                          </span>
                        </td>
                        <td>{{ formatDuration(task.duration) }}</td>
                        <td>{{ new Date(task.enqueuedAt).toLocaleString() }}</td>
                        <td>
                          <button class="btn btn-secondary btn-sm" @click="task._showJson = !task._showJson">
                            {{ task._showJson ? '收起' : '查看' }}
                          </button>
                        </td>
                      </tr>
                      <tr v-if="task._showJson" class="json-expanded-row">
                        <td colspan="7" style="border-bottom: none; padding-top: 0;">
                          <div class="json-preview-container">
                            <pre class="json-preview">{{ JSON.stringify(task, (k, v) => k === '_showJson' ? undefined : v, 2) }}</pre>
                          </div>
                        </td>
                      </tr>
                    </template>
                  </tbody>
                </table>
              </div>
              <div v-else class="empty-state">
                暂无任务记录
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- 1. 索引锁库配置 -->
      <section v-if="activeTab === 'security'" class="content-section">
        <header class="section-header">
          <h1>索引权限配置 <span>Index Security</span></h1>
          <p>控制索引的可见性，为敏感数据设置访问屏障。</p>
          <button class="btn btn-primary" @click="showAddIndexConf = true">🔒 配置加密索引</button>
        </header>

        <div v-if="showAddIndexConf" class="glass-editor">
          <h3>{{ editingIndexId ? '📝 编辑权限' : '🔒 新建加密策略' }}</h3>
          <div class="grid-inputs">
            <div class="input-group">
              <label>目标索引 (UID)</label>
              <select v-model="newIndex.uid" class="form-control">
                <option disabled value="">-- 选择可用索引 --</option>
                <option v-for="uid in availableIndexes" :key="uid" :value="uid">{{ uid }}</option>
              </select>
            </div>
            <div class="input-group">
              <label>显示别名</label>
              <input v-model="newIndex.alias" placeholder="如：秘密文档库" class="form-control">
            </div>
            <div class="input-group">
              <label>私有状态</label>
              <div class="toggle-group">
                <input type="checkbox" v-model="newIndex.isLocked" id="lock-toggle">
                <label for="lock-toggle">启用锁定 (需要 Token 访问)</label>
              </div>
            </div>
          </div>
          <div class="editor-actions">
            <button class="btn btn-primary" @click="saveIndexConfig">保存策略</button>
            <button class="btn btn-secondary" @click="cancelIndexEdit">取消</button>
          </div>
        </div>

        <table class="data-table">
          <thead>
            <tr>
              <th>索引标识</th>
              <th>别名 / 备注</th>
              <th>对外状态</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cfg in indexConfigs" :key="cfg.id">
              <td><strong>{{ cfg.uid }}</strong></td>
              <td>
                <div class="alias-info">
                  <span class="alias">{{ cfg.alias || '-' }}</span>
                  <span class="desc">{{ cfg.description }}</span>
                </div>
              </td>
              <td>
                <span :class="['status-badge', cfg.isLocked ? 'status-err' : 'status-ok']">
                  {{ cfg.isLocked ? '🔒 私有锁定' : '🌐 公开访问' }}
                </span>
              </td>
              <td>
                <button class="btn btn-secondary btn-sm" @click="editIndex(cfg)">编辑</button>
                <button class="btn btn-danger btn-sm" @click="deleteIndex(cfg.uid)">彻底删除</button>
              </td>
            </tr>
          </tbody>
        </table>
      </section>

      <!-- 2. 访问凭证分发 -->
      <section v-if="activeTab === 'tokens'" class="content-section">
        <header class="section-header">
          <h1>访问凭证管理 <span>Access Tokens</span></h1>
          <p>派发和管理访问私有索引的通行证。</p>
          <button class="btn btn-primary" @click="openAddToken">🎫 派发新 Token</button>
        </header>

        <div v-if="showAddToken" class="glass-editor">
          <h3>Token 配置</h3>
          <div class="token-generator">
            <input v-model="newToken.token" class="form-control token-input" readonly>
            <button class="btn btn-secondary" @click="newToken.token = generateUUID()">重新生成</button>
          </div>
          <div class="input-group" style="margin-top:20px">
            <label>授权范围 (允许访问的库)</label>
            <div class="index-chips">
              <label v-for="uid in availableIndexes" :key="uid" :class="['chip', { selected: newToken.allowIndexes.includes(uid) }]">
                <input type="checkbox" :value="uid" v-model="newToken.allowIndexes"> {{ uid }}
              </label>
              <label :class="['chip all', { selected: newToken.allowIndexes.includes('*') }]">
                <input type="checkbox" value="*" :checked="newToken.allowIndexes.includes('*')" @change="toggleAllIndexes"> [ 全部索引 * ]
              </label>
            </div>
          </div>
          <div class="grid-inputs" style="margin-top:20px">
            <div class="input-group">
              <label>拥有者备注</label>
              <input v-model="newToken.description" placeholder="如：外部合作伙伴" class="form-control">
            </div>
            <div class="input-group">
              <label>有效期 (天)</label>
              <input type="number" v-model="newToken.validDays" placeholder="留空永不过期" class="form-control">
            </div>
          </div>
          <div class="editor-actions">
            <button class="btn btn-primary" @click="handleSaveToken">确认派发</button>
            <button class="btn btn-secondary" @click="cancelTokenEdit">取消</button>
          </div>
        </div>

        <div class="token-list">
          <div v-for="tok in accessTokens" :key="tok.id" class="token-card">
            <div class="token-info">
              <div class="tok-header">
                <span class="tok-tag">ACTIVE TOKEN</span>
                <span :class="['status-badge', isExpired(tok.expiresAt) ? 'status-err' : 'status-ok']">
                  {{ isExpired(tok.expiresAt) ? '已过期' : '正常' }}
                </span>
              </div>
              <code class="tok-val">{{ tok.token }}</code>
              <p class="tok-desc">👤 {{ tok.description || '未命名持有者' }}</p>
              <div class="tok-meta">
                <span>📂 授权: {{ tok.allowIndexes }}</span>
                <span>⏳ 截止: {{ tok.expiresAt ? new Date(tok.expiresAt).toLocaleDateString() : '永久' }}</span>
              </div>
            </div>
            <div class="token-ops">
              <button class="btn btn-secondary btn-sm" @click="editToken(tok)">编辑</button>
              <button class="btn btn-danger btn-sm" @click="deleteToken(tok.id)">撤销</button>
            </div>
          </div>
        </div>
      </section>

      <!-- 3. Meilisearch 索引设置 -->
      <section v-if="activeTab === 'search'" class="content-section">
        <header class="section-header">
          <h1>索引核心配置 <span>Meilisearch Settings</span></h1>
          <p>精确控制字段的可搜索性、权重排序以及在构建器中的过滤权限。</p>
        </header>

        <div class="glass-editor">
          <div class="input-group" style="margin-bottom: 32px;">
            <label>选择目标索引</label>
            <select v-model="selectedSearchIndex" class="form-control" @change="fetchSearchSettings">
              <option disabled value="">-- 选择要配置的索引 --</option>
              <option v-for="uid in availableIndexes" :key="uid" :value="uid">{{ uid }}</option>
            </select>
          </div>

          <div v-if="selectedSearchIndex" class="search-settings-area animate-fade-in">
            <!-- 子标签切换 -->
            <div class="sub-tabs">
              <button :class="['sub-tab', { active: subTab === 'searchable' }]" @click="subTab = 'searchable'">🔍 搜索权重 (Searchable)</button>
              <button :class="['sub-tab', { active: subTab === 'filterable' }]" @click="subTab = 'filterable'">📋 过滤构建 (Filterable)</button>
            </div>

            <div v-if="subTab === 'searchable'" class="settings-pane animate-fade-in">
              <div class="input-group">
                <label>配置搜索权重优先级 (影响主搜索框)</label>
                <p class="helper-text" style="color: #64748b; font-size: 13px; margin-bottom: 16px;">
                  勾选字段加入搜索范围，并拖拽排序（排在前面的权重得分越高）。
                </p>
                
                <div class="field-selector mb-4">
                  <div v-for="field in allFieldsForIndex" :key="field" class="field-item">
                    <label class="checkbox-container">
                      <input type="checkbox" :value="field" v-model="currentSearchableAttributes">
                      <span class="checkmark"></span>
                      <span class="field-name">{{ field }}</span>
                    </label>
                  </div>
                </div>

                <div class="priority-list">
                  <div 
                    v-for="(field, index) in currentSearchableAttributes" 
                    :key="field"
                    class="priority-item"
                    draggable="true"
                    @dragstart="handleDragStart(index)"
                    @dragover.prevent
                    @drop="handleDrop(index)"
                  >
                    <span class="drag-handle">⠿</span>
                    <span class="priority-rank">{{ index + 1 }}</span>
                    <span class="field-name">{{ field }}</span>
                  </div>
                  <div v-if="currentSearchableAttributes.length === 0" class="empty-priority">
                    未选择任何搜索字段
                  </div>
                </div>
              </div>
            </div>

            <div v-if="subTab === 'filterable'" class="settings-pane animate-fade-in">
              <div class="input-group">
                <label>配置可过滤字段 (影响查询条件构建器)</label>
                <p class="helper-text" style="color: #64748b; font-size: 13px; margin-bottom: 16px;">
                  只有被勾选为「可过滤」的字段，才会出现在搜索页面的查询条件构建器中。
                </p>
                <div class="field-selector">
                  <div v-for="field in allFieldsForIndex" :key="field" class="field-item">
                    <label class="checkbox-container">
                      <input type="checkbox" :value="field" v-model="currentFilterableAttributes">
                      <span class="checkmark"></span>
                      <span class="field-name">{{ field }}</span>
                    </label>
                  </div>
                </div>
              </div>
            </div>

            <div class="editor-actions" style="margin-top: 40px; border-top: 1px solid rgba(255,255,255,0.1); padding-top: 24px;">
              <button class="btn btn-primary" @click="saveSearchSettings" :disabled="isSavingSearch">
                {{ isSavingSearch ? '正在应用配置...' : '保存当前索引配置' }}
              </button>
              <button class="btn btn-secondary" @click="resetSearchSettings">恢复全部字段</button>
            </div>
          </div>
          <div v-else class="empty-state">
            <div class="empty-icon">⚙️</div>
            <p>请先从上方选择一个索引进行配置</p>
          </div>
        </div>
      </section>

      <!-- 4. 应用全局设置 -->
      <section v-if="activeTab === 'settings'" class="content-section">
        <header class="section-header">
          <h1>全局应用设置 <span>App Configuration</span></h1>
          <p>配置应用名称、外观皮肤及核心 UI 参数。</p>
        </header>

        <div v-for="app in apps" :key="app.id" class="card settings-card">
          <div class="settings-row">
            <div class="row-label">
              <h4>应用基本信息</h4>
              <p>名称、版本及全局标识。</p>
            </div>
            <div class="row-val">
              <div class="app-identity">
                <span class="app-icon">🚀</span>
                <strong>{{ app.name }}</strong>
              </div>
            </div>
          </div>
          <div class="settings-row">
            <div class="row-label">
              <h4>UI 配置矩阵</h4>
              <p>自定义界面的 JSON 配置参数。</p>
            </div>
            <div class="row-val">
              <pre class="json-preview">{{ app.uiConfig }}</pre>
              <button class="btn btn-primary btn-sm" @click="editApp(app)">更新配置</button>
            </div>
          </div>
        </div>
      </section>

      <!-- 4. 账户安全设置 -->
      <section v-if="activeTab === 'password'" class="content-section">
        <header class="section-header">
          <h1>账户安全设置 <span>Account Security</span></h1>
          <p>定期更换密码可显著提高系统安全性。</p>
        </header>

        <div class="glass-editor">
          <h3>🔐 修改管理员密码</h3>
          <div class="grid-inputs" style="max-width: 400px;">
            <div class="input-group">
              <label>新密码</label>
              <input type="password" v-model="passwordForm.newPassword" placeholder="请输入新密码" class="form-control">
            </div>
            <div class="input-group">
              <label>确认新密码</label>
              <input type="password" v-model="passwordForm.confirmPassword" placeholder="请再次输入新密码" class="form-control">
            </div>
          </div>
          <div class="editor-actions">
            <button class="btn btn-primary" @click="handleUpdatePassword">保存并重新登录</button>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { generateUUID } from '@/utils'
import {
  getAdminIndexConfigs,
  getAdminAccessTokens,
  getAdminApps,
  getProxyIndexes,
  getAdminInstances,
  createAdminInstance,
  updateAdminInstance,
  deleteAdminInstance,
  saveAdminIndexConfig,
  deleteAdminIndexConfig,
  createAccessToken,
  updateAccessToken,
  deleteAccessToken,
  updateApp,
  updateAdminPassword,
  getMeiliIndexSettings,
  updateMeiliIndexSettings,
  loadIndexData,
  getMeiliTasks
} from '@/services/api'

function formatDuration(isoDuration: string | undefined): string {
  if (!isoDuration) return '-'
  const regex = /PT(?:(\d+)H)?(?:(\d+)M)?(?:([\d.]+)S)?/
  const match = isoDuration.match(regex)
  if (!match) return isoDuration
  
  const h = parseInt(match[1] || '0', 10)
  const m = parseInt(match[2] || '0', 10)
  const s = parseFloat(match[3] || '0')
  
  if (h === 0 && m === 0 && s < 1) {
    return `${(s * 1000).toFixed(2)} ms`
  }
  
  let result = ''
  if (h > 0) result += `${h}时`
  if (m > 0 || h > 0) result += `${m}分`
  
  if (s > 0 || (h === 0 && m === 0)) {
     result += `${s.toFixed(2)}秒`
  }
  
  return result || '0秒'
}

const activeTab = ref('instances')
const tabs = [
  { id: 'instances', label: '节点管理', icon: '☁️' },
  { id: 'security', label: '安全锁库', icon: '🛡️' },
  { id: 'tokens', label: '凭证分发', icon: '🎫' },
  { id: 'search', label: '搜索配置', icon: '🔍' },
  { id: 'settings', label: '应用设置', icon: '⚙️' },
  { id: 'password', label: '账户安全', icon: '🔐' },
]

const indexConfigs = ref<any[]>([])
const accessTokens = ref<any[]>([])
const apps = ref<any[]>([])
const instances = ref<any[]>([])
const availableIndexes = ref<string[]>([])

const showAddInstance = ref(false)
const editingInstanceId = ref<number | null>(null)
const newInstance = ref({ name: '', host: '', apiKey: '' })

const showTasksModal = ref(false)
const currentTaskInstance = ref<any>(null)
const instanceTasks = ref<any[]>([])
const isLoadingTasks = ref(false)

async function viewTasks(ins: any) {
  currentTaskInstance.value = ins
  showTasksModal.value = true
  isLoadingTasks.value = true
  instanceTasks.value = []
  try {
    const data = await getMeiliTasks(ins.host, ins.apiKey || '')
    instanceTasks.value = data.results || []
  } catch (e) {
    alert('获取任务失败: ' + (e as Error).message)
  } finally {
    isLoadingTasks.value = false
  }
}

const showAddIndexConf = ref(false)
const editingIndexId = ref<number | null>(null)
const newIndex = ref({ uid: '', alias: '', description: '', isLocked: false, fieldConfigs: '', viewConfigs: '', tableConfigs: '', canEdit: false })

const showAddToken = ref(false)
const editingTokenId = ref<number | null>(null)
const newToken = ref({ token: '', allowIndexes: [] as string[], description: '', validDays: null as number | null })

const passwordForm = ref({ newPassword: '', confirmPassword: '' })

// 搜索配置相关
const selectedSearchIndex = ref('')
const subTab = ref('searchable')
const currentSearchableAttributes = ref<string[]>([])
const currentFilterableAttributes = ref<string[]>([])
const allFieldsForIndex = ref<string[]>([])
const isSavingSearch = ref(false)
const draggedIndex = ref<number | null>(null)


function openAddToken() {
  showAddToken.value = true
  editingTokenId.value = null
  newToken.value = { token: generateUUID(), allowIndexes: [], description: '', validDays: null }
}

function toggleAllIndexes(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  if (checked) {
    newToken.value.allowIndexes = ['*']
  } else {
    newToken.value.allowIndexes = []
  }
}

function isExpired(date: string | null) {
  if (!date) return false
  return new Date(date).getTime() < Date.now()
}

async function loadAdminData() {
  try {
    const [idxData, tokData, appsData, proxyData, insData] = await Promise.all([
      getAdminIndexConfigs(),
      getAdminAccessTokens(),
      getAdminApps(),
      getProxyIndexes(),
      getAdminInstances()
    ])

    indexConfigs.value = idxData as any[]
    accessTokens.value = tokData as any[]
    apps.value = appsData as any[]
    instances.value = insData as any[]
    if (proxyData.results) {
      availableIndexes.value = proxyData.results.map(r => r.uid)
    }
  } catch (e) {
    console.error('Admin Data Load Error:', e)
  }
}

async function handleSaveInstance() {
  if (editingInstanceId.value) {
    updateInstance()
  } else {
    createInstance()
  }
}

async function createInstance() {
  if (!newInstance.value.name || !newInstance.value.host) return alert('请填入名称和地址')
  const ok = await createAdminInstance(newInstance.value)
  if (ok) {
      cancelInstanceEdit()
      loadAdminData()
  }
}

async function updateInstance() {
  const ok = await updateAdminInstance({ id: editingInstanceId.value, ...newInstance.value })
  if (ok) {
      cancelInstanceEdit()
      loadAdminData()
  }
}

function editInstance(ins: any) {
  editingInstanceId.value = ins.id
  newInstance.value = { name: ins.name, host: ins.host, apiKey: ins.apiKey || '' }
  showAddInstance.value = true
}

function cancelInstanceEdit() {
  showAddInstance.value = false
  editingInstanceId.value = null
  newInstance.value = { name: '', host: '', apiKey: '' }
}

async function deleteInstance(id: number) {
   if(!confirm('确定删除该实例配置？')) return
   await deleteAdminInstance(id)
   loadAdminData()
}

async function saveIndexConfig() {
    if (!newIndex.value.uid) return alert('请先选择一个索引')
    await submitIndexConfig(newIndex.value)
    cancelIndexEdit()
}

function editIndex(cfg: any) {
  editingIndexId.value = cfg.id
  newIndex.value = {
    uid: cfg.uid,
    alias: cfg.alias,
    description: cfg.description,
    isLocked: cfg.isLocked,
    fieldConfigs: cfg.fieldConfigs || '',
    viewConfigs: cfg.viewConfigs || '',
    tableConfigs: cfg.tableConfigs || '',
    canEdit: cfg.canEdit ?? false
  }
  showAddIndexConf.value = true
}

function cancelIndexEdit() {
  showAddIndexConf.value = false
  editingIndexId.value = null
  newIndex.value = { uid: '', alias: '', description: '', isLocked: false, fieldConfigs: '', viewConfigs: '', tableConfigs: '', canEdit: false }
}

async function submitIndexConfig(payload: any) {
    await saveAdminIndexConfig(payload)
    loadAdminData()
}

async function handleSaveToken() {
  if (editingTokenId.value) {
    updateToken()
  } else {
    createToken()
  }
}

async function createToken() {
  if (!newToken.value.token) return alert('请填入Token字符串')
  const expiresAt = newToken.value.validDays 
    ? new Date(Date.now() + newToken.value.validDays * 24 * 60 * 60 * 1000).toISOString()
    : null
  const ok = await createAccessToken({ ...newToken.value, allowIndexes: JSON.stringify(newToken.value.allowIndexes), expiresAt })
  if (ok) {
      cancelTokenEdit()
      loadAdminData()
  }
}

async function updateToken() {
  const expiresAt = newToken.value.validDays 
    ? new Date(Date.now() + newToken.value.validDays * 24 * 60 * 60 * 1000).toISOString()
    : null
  const ok = await updateAccessToken({ id: editingTokenId.value, ...newToken.value, allowIndexes: JSON.stringify(newToken.value.allowIndexes), expiresAt })
  if (ok) {
      cancelTokenEdit()
      loadAdminData()
  }
}

function editToken(tok: any) {
  editingTokenId.value = tok.id
  let allowed = []
  try { allowed = JSON.parse(tok.allowIndexes || '[]') } catch { allowed = [] }

  newToken.value = { 
    token: tok.token, 
    allowIndexes: allowed, 
    description: tok.description,
    validDays: tok.expiresAt ? Math.round((new Date(tok.expiresAt).getTime() - Date.now()) / (24 * 60 * 60 * 1000)) : null
  }
  showAddToken.value = true
}

function cancelTokenEdit() {
  showAddToken.value = false
  editingTokenId.value = null
  newToken.value = { token: '', allowIndexes: [], description: '', validDays: null }
}

async function deleteIndex(uid: string) {
  if (!confirm(`确定要彻底删除索引 [${uid}] 吗？此操作将同时删除本地配置及 Meilisearch 中的原始数据，不可恢复！`)) return
  await deleteAdminIndexConfig(uid)
  loadAdminData()
}

async function deleteToken(id: number) {
   if(!confirm('确定吊销该令牌？')) return
   await deleteAccessToken(id)
   loadAdminData()
}

function editApp(app: any) {
  const newConfig = prompt('编辑 UI 配置 (JSON 格式):', app.uiConfig)
  if (newConfig !== null) {
      updateAppConfig(app.id, newConfig)
  }
}

async function updateAppConfig(id: number, uiConfig: string) {
  await updateApp(id, { uiConfig })
  loadAdminData()
}

async function handleUpdatePassword() {
  if (!passwordForm.value.newPassword) return alert('请输入新密码')
  if (passwordForm.value.newPassword !== passwordForm.value.confirmPassword) return alert('两次输入的密码不一致')
  if (passwordForm.value.newPassword.length < 6) return alert('密码长度至少为 6 位')

  try {
    const ok = await updateAdminPassword(passwordForm.value.newPassword)

    if (ok) {
      alert('密码修改成功，请使用新密码重新登录')
      localStorage.removeItem('authToken')
      window.location.reload()
    } else {
      alert('修改失败')
    }
  } catch (e) {
    alert('请求网络异常')
  }
}

async function fetchSearchSettings() {
  if (!selectedSearchIndex.value) return
  
  try {
    // 1. 获取当前 Meilisearch 设置
    const settings = await getMeiliIndexSettings(selectedSearchIndex.value)
    if (settings) {
      currentSearchableAttributes.value = settings.searchableAttributes || []
      currentFilterableAttributes.value = settings.filterableAttributes || []
    }

    // 2. 获取所有可用字段 (通过采样数据和现有设置)
    const data = await loadIndexData('/api/v1/proxy', '', selectedSearchIndex.value)
    allFieldsForIndex.value = data.availableFields
    
    // 如果 searchable 为空，意味着 Meilisearch 默认搜索所有字段
    if (currentSearchableAttributes.value.length === 0 || (currentSearchableAttributes.value.length === 1 && currentSearchableAttributes.value[0] === '*')) {
      currentSearchableAttributes.value = [...allFieldsForIndex.value]
    }
    // Filterable 同理
    if (currentFilterableAttributes.value.length === 0) {
      currentFilterableAttributes.value = [...allFieldsForIndex.value]
    }
  } catch (e) {
    console.error('Fetch search settings error:', e)
    alert('获取配置失败')
  }
}

async function saveSearchSettings() {
  if (!selectedSearchIndex.value) return
  isSavingSearch.value = true
  try {
    const payload = {
      searchableAttributes: currentSearchableAttributes.value,
      filterableAttributes: currentFilterableAttributes.value
    }
    const ok = await updateMeiliIndexSettings(selectedSearchIndex.value, payload)
    if (ok) {
      alert('索引设置更新成功！Meilisearch 正在异步处理更新任务。')
    } else {
      alert('更新失败，请检查后端日志')
    }
  } catch (e) {
    alert('请求失败: ' + (e as Error).message)
  } finally {
    isSavingSearch.value = false
  }
}

function resetSearchSettings() {
  currentSearchableAttributes.value = [...allFieldsForIndex.value]
  currentFilterableAttributes.value = [...allFieldsForIndex.value]
}

function handleDragStart(index: number) {
  draggedIndex.value = index
}

function handleDrop(index: number) {
  if (draggedIndex.value === null) return
  const list = [...currentSearchableAttributes.value]
  const [movedItem] = list.splice(draggedIndex.value, 1)
  list.splice(index, 0, movedItem)
  currentSearchableAttributes.value = list
  draggedIndex.value = null
}

function toggleField(field: string) {
  const index = currentSearchableAttributes.value.indexOf(field)
  if (index > -1) {
    currentSearchableAttributes.value.splice(index, 1)
  } else {
    currentSearchableAttributes.value.push(field)
  }
}

onMounted(() => {
  loadAdminData()
})
</script>

<style scoped>
.admin-layout {
  display: flex;
  min-height: 100vh;
  background: #030712;
}

/* 侧边栏 */
.admin-sidebar {
  width: 260px;
  background: rgba(15, 23, 42, 0.8);
  backdrop-filter: blur(20px);
  border-right: 1px solid rgba(255, 255, 255, 0.05);
  display: flex;
  flex-direction: column;
  padding: 32px 0;
  position: fixed;
  height: 100vh;
}

.sidebar-header {
  padding: 0 32px;
  margin-bottom: 48px;
  display: flex;
  align-items: center;
  gap: 12px;
}
.sidebar-logo { font-size: 32px; }
.sidebar-header h2 { font-family: 'Outfit'; font-size: 20px; font-weight: 700; color: white; }

.sidebar-nav { flex: 1; padding: 0 16px; display: flex; flex-direction: column; gap: 8px; }
.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 12px;
  border: none;
  background: transparent;
  color: #94a3b8;
  cursor: pointer;
  transition: all 0.3s ease;
  font-weight: 500;
}
.nav-item:hover { background: rgba(255, 255, 255, 0.05); color: white; }
.nav-item.active { background: rgba(99, 102, 241, 0.1); color: var(--primary); }
.nav-icon { font-size: 18px; }

.sidebar-footer { padding: 0 32px; font-size: 11px; color: #475569; }

/* 主内容区 */
.admin-main { flex: 1; margin-left: 260px; padding: 48px 64px; }

.content-section { max-width: 1000px; }

.section-header { margin-bottom: 40px; display: flex; flex-direction: column; gap: 8px; position: relative; }
.section-header h1 { font-family: 'Outfit'; font-size: 32px; font-weight: 700; color: white; }
.section-header h1 span { font-weight: 300; opacity: 0.3; margin-left: 8px; font-size: 0.6em; }
.section-header p { color: #94a3b8; font-size: 15px; }
.section-header .btn { position: absolute; right: 0; top: 0; }

/* 编辑器容器 */
.glass-editor {
  background: rgba(30, 41, 59, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 20px;
  padding: 32px;
  margin-bottom: 32px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
}
.glass-editor h3 { margin-bottom: 24px; font-size: 18px; color: white; }
.grid-inputs { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; }
.input-group label { display: block; font-size: 13px; color: #64748b; margin-bottom: 8px; }
.editor-actions { margin-top: 32px; display: flex; gap: 12px; justify-content: flex-end; }

/* 数据卡片 */
.data-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 24px; }
.data-card {
  background: rgba(15, 23, 42, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 16px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  transition: all 0.3s ease;
}
.data-card:hover { transform: translateY(-4px); border-color: rgba(99, 102, 241, 0.3); }
.card-info { display: flex; align-items: center; gap: 16px; }
.ins-avatar { width: 48px; height: 48px; border-radius: 12px; background: var(--primary); color: white; font-weight: 700; font-size: 20px; display: flex; align-items: center; justify-content: center; }
.card-info h4 { font-size: 16px; color: white; margin-bottom: 4px; }
.card-info code { font-size: 12px; color: var(--primary); }
.card-ops { display: flex; gap: 10px; border-top: 1px solid rgba(255, 255, 255, 0.05); pt: 16px; padding-top: 16px; }

/* Token 特殊样式 */
.token-list { display: flex; flex-direction: column; gap: 16px; }
.token-card {
  background: rgba(30, 41, 59, 0.4);
  border-radius: 16px;
  padding: 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.tok-tag { font-size: 9px; font-weight: 800; color: var(--primary); letter-spacing: 0.1em; background: rgba(99, 102, 241, 0.1); padding: 2px 6px; border-radius: 4px; margin-right: 8px; }
.tok-val { font-size: 16px; color: #fbbf24; font-family: monospace; display: block; margin: 12px 0; }
.tok-desc { color: white; font-size: 14px; margin-bottom: 8px; }
.tok-meta { display: flex; gap: 24px; font-size: 12px; color: #64748b; }

/* 设置 */
.settings-card { display: flex; flex-direction: column; gap: 32px; }
.settings-row { display: flex; gap: 48px; align-items: flex-start; }
.row-label { width: 240px; }
.row-label h4 { font-size: 16px; color: white; margin-bottom: 4px; }
.row-label p { font-size: 13px; color: #64748b; }
.row-val { flex: 1; }
.app-identity { display: flex; align-items: center; gap: 12px; font-size: 20px; }
.json-preview { background: #000; padding: 16px; border-radius: 8px; font-size: 12px; color: #10b981; max-height: 200px; overflow: auto; margin-bottom: 12px; }

/* 通用列表项 */
.alias-info { display: flex; flex-direction: column; gap: 4px; }
.alias { color: white; font-weight: 600; }
.desc { font-size: 12px; color: #64748b; }

/* 芯片多选 */
.index-chips { display: flex; flex-wrap: wrap; gap: 8px; }
.chip { padding: 6px 12px; background: rgba(255, 255, 255, 0.05); border-radius: 8px; font-size: 13px; color: #94a3b8; cursor: pointer; border: 1px solid transparent; }
.chip:hover { background: rgba(255, 255, 255, 0.1); }
.chip.selected { background: rgba(99, 102, 241, 0.15); border-color: var(--primary); color: white; }
.chip input { display: none; }

/* 搜索配置专用样式 */
.field-selector {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
  background: rgba(0, 0, 0, 0.2);
  padding: 20px;
  border-radius: 12px;
  max-height: 400px;
  overflow-y: auto;
}

.field-item {
  display: flex;
  align-items: center;
}

.checkbox-container {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  font-size: 14px;
  color: #e2e8f0;
  user-select: none;
}

.checkbox-container input {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.field-name {
  word-break: break-all;
}

.empty-state {
  text-align: center;
  padding: 60px 0;
  color: #64748b;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.2;
}

.animate-fade-in {
  animation: fadeIn 0.4s ease-out;
}

/* 子标签样式 */
.sub-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
  background: rgba(0, 0, 0, 0.2);
  padding: 6px;
  border-radius: 12px;
  width: fit-content;
}

.sub-tab {
  padding: 8px 16px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: #94a3b8;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.sub-tab.active {
  background: var(--primary);
  color: white;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
}

.mb-4 { margin-bottom: 24px; }

/* 优先级列表样式 */
.priority-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: rgba(0, 0, 0, 0.3);
  padding: 16px;
  border-radius: 12px;
  border: 1px dashed rgba(255, 255, 255, 0.1);
}

.priority-item {
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(30, 41, 59, 0.6);
  padding: 10px 16px;
  border-radius: 8px;
  cursor: grab;
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.priority-item:hover {
  background: rgba(51, 65, 85, 0.8);
  border-color: var(--primary);
  transform: translateX(4px);
}

.priority-item:active {
  cursor: grabbing;
}

.drag-handle {
  color: #475569;
  font-size: 18px;
  user-select: none;
}

.priority-rank {
  font-family: 'Outfit';
  font-weight: 700;
  color: var(--primary);
  min-width: 20px;
}

.priority-item .field-name {
  flex: 1;
  font-family: monospace;
  font-size: 14px;
  color: white;
}

.remove-btn {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s;
}

.remove-btn:hover {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}

.empty-priority {
  text-align: center;
  padding: 32px;
  color: #475569;
  font-size: 14px;
  font-style: italic;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.tasks-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.tasks-modal {
  background: var(--bg-card);
  backdrop-filter: var(--glass-blur);
  border: 1px solid var(--border);
  color: var(--text-main);
  border-radius: 16px;
  width: 1200px;
  max-width: 95vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
}
.modal-header {
  padding: 20px 24px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.modal-header h3 {
  margin: 0;
  color: var(--text-main);
  font-size: 18px;
}
.close-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 24px;
  cursor: pointer;
}
.close-btn:hover { color: var(--primary); }
.modal-body {
  padding: 24px;
  overflow-y: auto;
}
.table-container {
  overflow-x: auto;
}
.loading-state {
  text-align: center;
  color: #94a3b8;
  padding: 40px;
}
.json-expanded-row {
  background: transparent !important;
}
.json-preview-container {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 12px;
  overflow-x: auto;
}
.json-preview {
  margin: 0;
  font-family: monospace;
  font-size: 13px;
  color: #10b981;
}
</style>
