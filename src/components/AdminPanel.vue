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
              <div class="modal-title">
                <h3>⚡ 任务详情 - {{ currentTaskInstance?.name }}</h3>
                <span v-if="taskLastUpdatedAt" class="task-refresh-time">
                  更新于 {{ taskLastUpdatedAt.toLocaleTimeString() }}
                </span>
              </div>
              <div class="modal-actions">
                <button
                  class="btn btn-secondary btn-sm"
                  :disabled="isLoadingTasks || isRefreshingTasks"
                  @click="refreshTasks"
                >
                  {{ isRefreshingTasks ? '刷新中...' : '刷新' }}
                </button>
                <button class="close-btn" @click="showTasksModal = false">×</button>
              </div>
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
                      <th>详情 / 操作</th>
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
                          <div style="display: flex; gap: 8px;">
                            <button class="btn btn-secondary btn-sm" @click="task._showJson = !task._showJson">
                              {{ task._showJson ? '收起' : '查看' }}
                            </button>
                            <button 
                              v-if="['enqueued', 'processing'].includes(task.status)"
                              class="btn btn-danger btn-sm" 
                              @click="handleCancelTask(task)"
                            >终止</button>
                          </div>
                        </td>
                      </tr>
                      <tr v-if="task._showJson" class="json-expanded-row">
                        <td colspan="7" style="border-bottom: none; padding-top: 0;">
                          <div class="json-preview-container">
                            <pre class="task-json-preview">{{ JSON.stringify(task, (k, v) => k === '_showJson' ? undefined : v, 2) }}</pre>
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
            <div class="input-group">
              <label>前台显示</label>
              <div class="toggle-group">
                <input type="checkbox" v-model="newIndex.isVisible" id="visible-toggle">
                <label for="visible-toggle">在前台索引列表中显示</label>
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
              <th>文档数</th>
              <th>别名 / 备注</th>
              <th>对外状态</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cfg in indexConfigs" :key="cfg.id">
              <td><strong>{{ cfg.uid }}</strong></td>
              <td>
                <span class="doc-count">
                  {{ typeof cfg.documentCount === 'number' ? cfg.documentCount.toLocaleString() : '-' }}
                </span>
              </td>
              <td>
                <div class="alias-info">
                  <span class="alias">{{ cfg.alias || '-' }}</span>
                  <span class="desc">{{ cfg.description }}</span>
                </div>
              </td>
              <td>
                <span :class="['status-badge', cfg.isVisible === false ? 'status-err' : 'status-ok']">
                  {{ cfg.isVisible === false ? '🙈 前台隐藏' : '👁️ 前台显示' }}
                </span>
                <span :class="['status-badge', cfg.isLocked ? 'status-err' : 'status-ok']" style="margin-left: 8px;">
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
            <div class="input-group">
              <label>日查询限制 (次/天)</label>
              <input type="number" v-model="newToken.maxQueriesPerDay" placeholder="0或留空不限制" class="form-control">
            </div>
            <div class="input-group">
              <label>日导入限制 (条/天)</label>
              <input type="number" v-model="newToken.maxImportsPerDay" placeholder="0或留空不限制" class="form-control">
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
              <div class="tok-meta" style="margin-top: 4px;">
                <span>🔍 查询限额: {{ tok.maxQueriesPerDay > 0 ? tok.maxQueriesPerDay + ' 次/天' : '无限制' }}</span>
                <span>📥 导入限额: {{ tok.maxImportsPerDay > 0 ? tok.maxImportsPerDay + ' 条/天' : '无限制' }}</span>
              </div>
            </div>
            <div class="token-ops">
              <button class="btn btn-secondary btn-sm" @click="editToken(tok)">编辑</button>
              <button class="btn btn-danger btn-sm" @click="deleteToken(tok.id)">撤销</button>
            </div>
          </div>
        </div>
      </section>

      <!-- 申请列表管理 -->
      <section v-if="activeTab === 'applications'" class="content-section">
        <header class="section-header">
          <h1>Token 申请管理 <span>Applications</span></h1>
          <p>审核来自前台用户的 Token 申请请求。</p>
        </header>

        <div class="applications-table">
          <table class="data-table">
            <thead>
              <tr>
                <th>申请人</th>
                <th>联系方式</th>
                <th>详细信息</th>
                <th>申请索引</th>
                <th>状态</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="app in tokenApplications" :key="app.id">
                <td>
                  <div class="user-info">
                    <strong>{{ app.name }}</strong>
                    <span class="meta">{{ app.gender }} | {{ app.birthday }}</span>
                  </div>
                </td>
                <td><code>{{ app.email }}</code></td>
                <td>
                  <div class="purpose-info" :title="app.purpose">
                    {{ app.purpose || '无说明' }}
                  </div>
                </td>
                <td>
                  <div class="index-tags">
                    <span v-for="idx in parseAllowIndexes(app.allowIndexes)" :key="idx" class="tag">{{ idx }}</span>
                  </div>
                </td>
                <td>
                  <span :class="['status-badge', app.status === 1 ? 'status-ok' : (app.status === 2 ? 'status-err' : 'status-warn')]">
                    {{ app.status === 1 ? '已通过' : (app.status === 2 ? '已驳回' : '待审批') }}
                  </span>
                </td>
                <td>
                  <div v-if="app.status === 0" style="display:flex; gap:8px">
                    <button class="btn btn-primary btn-sm" @click="openApproveModal(app)">通过</button>
                    <button class="btn btn-danger btn-sm" @click="openRejectModal(app)">驳回</button>
                  </div>
                  <span v-else class="text-muted">已处理</span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="applications-mobile">
          <div v-if="tokenApplications.length === 0" class="empty-state">暂无申请记录</div>
          <div v-for="app in tokenApplications" :key="`mobile-${app.id}`" class="application-card">
            <div class="application-card-header">
              <div class="user-info">
                <strong>{{ app.name }}</strong>
                <span class="meta">{{ app.gender }} | {{ app.birthday }}</span>
              </div>
              <span :class="['status-badge', app.status === 1 ? 'status-ok' : (app.status === 2 ? 'status-err' : 'status-warn')]">
                {{ app.status === 1 ? '已通过' : (app.status === 2 ? '已驳回' : '待审批') }}
              </span>
            </div>
            <div class="application-field">
              <label>联系方式</label>
              <code>{{ app.email }}</code>
            </div>
            <div class="application-field">
              <label>用途说明</label>
              <div class="application-purpose">{{ app.purpose || '无说明' }}</div>
            </div>
            <div class="application-field">
              <label>申请索引</label>
              <div class="index-tags">
                <span v-for="idx in parseAllowIndexes(app.allowIndexes)" :key="`mobile-idx-${app.id}-${idx}`" class="tag">{{ idx }}</span>
              </div>
            </div>
            <div class="application-actions">
              <template v-if="app.status === 0">
                <button class="btn btn-primary btn-sm" @click="openApproveModal(app)">通过</button>
                <button class="btn btn-danger btn-sm" @click="openRejectModal(app)">驳回</button>
              </template>
              <span v-else class="text-muted">已处理</span>
            </div>
          </div>
        </div>
      </section>

      <section v-if="activeTab === 'metrics'" class="content-section">
        <header class="section-header">
          <h1>用量监控 <span>Usage Metrics</span></h1>
          <p>按天统计查询请求、文档导入量和存储空间占用（仅保留最近六个月）。</p>
        </header>

        <div class="data-grid metrics-grid">
          <div class="data-card">
            <h4>总查询量</h4>
            <strong class="metric-value">{{ totalQueryCount.toLocaleString() }}</strong>
          </div>
          <div class="data-card">
            <h4>总导入量</h4>
            <strong class="metric-value">{{ totalImportCount.toLocaleString() }}</strong>
          </div>
          <div class="data-card">
            <h4>当前存储占用</h4>
            <strong class="metric-value">{{ formatBytes(latestDatabaseSize) }}</strong>
          </div>
        </div>

        <div class="glass-editor chart-panel">
          <h3>按天趋势（查询量 / 导入量）</h3>
          <div v-if="chartPointsQuery.length" class="line-chart-wrap">
            <div class="line-chart-stage" @mouseleave="clearChartHover">
              <svg
                :viewBox="`0 0 ${chartSvgWidth} ${chartSvgHeight}`"
                preserveAspectRatio="none"
                class="line-chart"
                @mousemove="handleChartHover"
                @mouseenter="handleChartHover"
              >
                <g class="chart-grid">
                  <line
                    v-for="(tick, idx) in chartYAxisTicks"
                    :key="`grid-${idx}-${tick.value}`"
                    :x1="chartPlotLeft"
                    :x2="chartPlotLeft + chartPlotWidth"
                    :y1="tick.y"
                    :y2="tick.y"
                    class="chart-grid-line"
                  />
                </g>
                <line :x1="chartPlotLeft" :y1="chartPlotBottom" :x2="chartPlotLeft + chartPlotWidth" :y2="chartPlotBottom" class="axis-line" />
                <line :x1="chartPlotLeft" :y1="chartPlotTop" :x2="chartPlotLeft" :y2="chartPlotBottom" class="axis-line" />
                <g class="chart-axis-labels">
                  <text
                    v-for="(tick, idx) in chartYAxisTicks"
                    :key="`y-${idx}-${tick.value}`"
                    :x="chartPlotLeft - 10"
                    :y="tick.y + 4"
                    class="chart-axis-label y-label"
                    text-anchor="end"
                  >
                    {{ tick.label }}
                  </text>
                  <text
                    v-for="label in chartXAxisLabels"
                    :key="`x-${label.value}-${label.x}`"
                    :x="label.x"
                    :y="chartPlotBottom + 22"
                    class="chart-axis-label x-label"
                    text-anchor="middle"
                  >
                    {{ label.label }}
                  </text>
                </g>
                <line
                  v-if="chartHoverPosition"
                  :x1="chartHoverPosition.x"
                  :x2="chartHoverPosition.x"
                  :y1="chartPlotTop"
                  :y2="chartPlotBottom"
                  class="chart-crosshair"
                />
                <line
                  v-if="chartHoverPosition"
                  :x1="chartPlotLeft"
                  :x2="chartPlotLeft + chartPlotWidth"
                  :y1="chartHoverPosition.y"
                  :y2="chartHoverPosition.y"
                  class="chart-crosshair"
                />
                <polyline :points="chartPointsQuery" class="line-query" />
                <polyline :points="chartPointsImport" class="line-import" />
                <g class="chart-point-layer">
                  <circle
                    v-for="point in chartDataPoints"
                    :key="`query-point-${point.index}`"
                    :cx="point.x"
                    :cy="point.queryY"
                    :r="hoveredChartPoint?.index === point.index ? 6 : 3.5"
                    :class="['chart-point', 'query', { active: hoveredChartPoint?.index === point.index }]"
                  />
                  <circle
                    v-for="point in chartDataPoints"
                    :key="`import-point-${point.index}`"
                    :cx="point.x"
                    :cy="point.importY"
                    :r="hoveredChartPoint?.index === point.index ? 6 : 3.5"
                    :class="['chart-point', 'import', { active: hoveredChartPoint?.index === point.index }]"
                  />
                </g>
              </svg>
              <div v-if="hoveredChartPoint" class="chart-tooltip" :style="chartTooltipStyle">
                <div class="chart-tooltip-date">{{ hoveredChartPoint.date }}</div>
                <div class="chart-tooltip-row">
                  <i class="legend-dot query"></i>
                  查询量：{{ hoveredChartPoint.queryCount.toLocaleString() }}
                </div>
                <div class="chart-tooltip-row">
                  <i class="legend-dot import"></i>
                  导入量：{{ hoveredChartPoint.importCount.toLocaleString() }}
                </div>
              </div>
            </div>
            <div class="chart-legend">
              <span class="legend-item"><i class="legend-dot query"></i>查询量</span>
              <span class="legend-item"><i class="legend-dot import"></i>导入量</span>
            </div>
          </div>
          <div v-else class="text-muted">暂无趋势数据</div>
        </div>

        <div class="glass-editor metric-detail-panel">
          <div class="metric-detail-header">
            <div>
              <h3>每日明细</h3>
              <p>共 {{ usageMetrics.length }} 条记录，每页显示 {{ DAILY_METRIC_PAGE_SIZE }} 条。</p>
            </div>
            <div v-if="totalPages > 1" class="metrics-pagination">
              <button 
                class="btn btn-secondary btn-sm" 
                :disabled="currentPage === 1"
                @click="currentPage--"
              >
                ◀ 上一页
              </button>
              <span class="pagination-info">{{ currentPage }} / {{ totalPages }} 页</span>
              <button 
                class="btn btn-secondary btn-sm" 
                :disabled="currentPage === totalPages"
                @click="currentPage++"
              >
                下一页 ▶
              </button>
            </div>
          </div>
          <table class="data-table compact-metrics-table">
            <thead>
              <tr>
                <th>日期</th>
                <th>查询量</th>
                <th>导入量</th>
                <th>存储占用</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in displayedUsageMetrics" :key="item.date">
                <td>{{ item.date }}</td>
                <td>{{ Number(item.queryCount || 0).toLocaleString() }}</td>
                <td>{{ Number(item.importCount || 0).toLocaleString() }}</td>
                <td>{{ formatBytes(Number(item.databaseSize || 0)) }}</td>
              </tr>
              <tr v-if="usageMetrics.length === 0">
                <td colspan="4" class="text-muted">暂无统计数据</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="glass-editor metric-detail-panel">
          <div class="metric-detail-header">
            <div>
              <h3>按索引维度统计</h3>
              <p>六个月汇总，默认显示前 {{ INDEX_METRIC_PREVIEW_LIMIT }} 个索引。</p>
            </div>
            <button
              v-if="indexUsageMetrics.length > INDEX_METRIC_PREVIEW_LIMIT"
              class="btn btn-secondary btn-sm"
              @click="showAllIndexMetrics = !showAllIndexMetrics"
            >
              {{ showAllIndexMetrics ? '收起' : '展开全部' }}
            </button>
          </div>
          <table class="data-table compact-metrics-table">
            <thead>
              <tr>
                <th>索引</th>
                <th>查询量</th>
                <th>导入量</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in displayedIndexUsageMetrics" :key="item.indexUid">
                <td>{{ item.indexUid }}</td>
                <td>{{ Number(item.queryCount || 0).toLocaleString() }}</td>
                <td>{{ Number(item.importCount || 0).toLocaleString() }}</td>
              </tr>
              <tr v-if="indexUsageMetrics.length === 0">
                <td colspan="3" class="text-muted">暂无索引维度数据</td>
              </tr>
            </tbody>
          </table>
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
              <button :class="['sub-tab', { active: subTab === 'embedding' }]" @click="subTab = 'embedding'">🧠 Embedding 模型</button>
            </div>

            <div v-if="subTab === 'searchable'" class="settings-pane animate-fade-in">
              <div class="input-group">
                <label>配置搜索权重优先级 (影响主搜索框)</label>
                <p class="helper-text" style="color: var(--text-muted); font-size: 13px; margin-bottom: 16px;">
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
              <div class="editor-actions" style="margin-top: 24px; border-top: 1px solid var(--border); padding-top: 20px;">
                <button class="btn btn-primary" @click="saveSearchFieldSettings" :disabled="isSavingSearchFields">
                  {{ isSavingSearchFields ? '正在应用配置...' : '保存搜索/过滤配置' }}
                </button>
                <button class="btn btn-secondary" @click="resetSearchFieldSettings">恢复搜索/过滤默认</button>
              </div>
            </div>

            <div v-if="subTab === 'filterable'" class="settings-pane animate-fade-in">
              <div class="input-group">
                <label>配置可过滤字段 (影响查询条件构建器)</label>
                <p class="helper-text" style="color: var(--text-muted); font-size: 13px; margin-bottom: 16px;">
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
              <div class="editor-actions" style="margin-top: 24px; border-top: 1px solid var(--border); padding-top: 20px;">
                <button class="btn btn-primary" @click="saveSearchFieldSettings" :disabled="isSavingSearchFields">
                  {{ isSavingSearchFields ? '正在应用配置...' : '保存搜索/过滤配置' }}
                </button>
                <button class="btn btn-secondary" @click="resetSearchFieldSettings">恢复搜索/过滤默认</button>
              </div>
            </div>

            <div v-if="subTab === 'embedding'" class="settings-pane animate-fade-in">
              <div class="input-group">
                <label>配置索引 Embedding 模型</label>
                <p class="helper-text" style="color: var(--text-muted); font-size: 13px; margin-bottom: 16px;">
                  这里会更新 Meilisearch 的 embedders 设置。建议先配置一套默认模型，再按需扩展高级参数。
                </p>
                <label class="checkbox-container" style="margin-bottom: 16px;">
                  <input type="checkbox" v-model="embeddingEnabled">
                  <span class="checkmark"></span>
                  <span class="field-name">启用 Embedding（关闭时将清空当前索引的 embedders）</span>
                </label>

                <template v-if="embeddingEnabled">
                  <div class="grid-inputs">
                    <div class="input-group">
                      <label>Embedder 名称</label>
                      <input v-model="currentEmbedderName" class="form-control" placeholder="default">
                    </div>
                    <div class="input-group">
                      <label>来源 (Source)</label>
                      <select v-model="currentEmbedder.source" class="form-control">
                        <option value="openAi">openAi</option>
                        <option value="huggingFace">huggingFace</option>
                        <option value="ollama">ollama</option>
                        <option value="rest">rest</option>
                        <option value="userProvided">userProvided</option>
                      </select>
                    </div>
                    <div class="input-group" v-if="currentEmbedder.source !== 'rest'">
                      <label>模型名 (Model)</label>
                      <input v-model="currentEmbedder.model" class="form-control" placeholder="text-embedding-3-small / BAAI/bge-base-en-v1.5">
                    </div>
                    <div class="input-group">
                      <label>向量维度 (可选)</label>
                      <input type="number" min="1" v-model.number="currentEmbedder.dimensions" class="form-control" placeholder="1536">
                    </div>
                    <div class="input-group">
                      <label>API 地址 / URL (可选)</label>
                      <input v-model="currentEmbedder.url" class="form-control" placeholder="https://api.openai.com/v1/embeddings">
                    </div>
                    <div class="input-group">
                      <label>API Key (可选)</label>
                      <input type="password" v-model="currentEmbedder.apiKey" class="form-control" placeholder="sk-...">
                    </div>
                    <div class="input-group" style="grid-column: 1 / -1;" v-if="currentEmbedder.source === 'rest'">
                      <label>REST 请求体 (Request JSON)</label>
                      <textarea
                        v-model="currentEmbedder.request"
                        class="form-control json-editor"
                        rows="5"
                        placeholder='{ "model": "bge-large-zh-v1.5", "input": ["{{text}}"] }'
                      />
                    </div>
                    <div class="input-group" style="grid-column: 1 / -1;" v-if="currentEmbedder.source === 'rest'">
                      <label>REST 响应解析 (Response JSON)</label>
                      <textarea
                        v-model="currentEmbedder.response"
                        class="form-control json-editor"
                        rows="3"
                        placeholder='{ "embedding": "$.data[0].embedding" }'
                      />
                    </div>
                    <div class="input-group" style="grid-column: 1 / -1;">
                      <label>Document Template (可选，推荐)</label>
                      <textarea
                        v-model="currentEmbedder.documentTemplate"
                        class="form-control"
                        rows="4"
                        placeholder="{% for field in fields %}{% if field.is_searchable and not field.value == nil %}{{ field.name }}: {{ field.value }} {% endif %}{% endfor %}"
                      />
                    </div>
                    <div class="input-group">
                      <label>Template 最大字节数 (可选)</label>
                      <input type="number" min="1" v-model.number="currentEmbedder.documentTemplateMaxBytes" class="form-control" placeholder="400">
                    </div>
                  </div>
                </template>
              </div>
              <div class="editor-actions" style="margin-top: 24px; border-top: 1px solid var(--border); padding-top: 20px;">
                <button class="btn btn-primary" @click="saveEmbeddingSettings" :disabled="isSavingEmbedding">
                  {{ isSavingEmbedding ? '正在应用配置...' : '保存 Embedding 配置' }}
                </button>
                <button class="btn btn-secondary" @click="resetEmbeddingSettings">恢复 Embedding 默认</button>
              </div>
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

    <div v-if="approveModalOpen" class="tasks-modal-overlay animate-fade-in" @click.self="closeApproveModal">
      <div class="tasks-modal approve-modal">
        <div class="modal-header">
          <h3>审批通过并分发凭证</h3>
          <button class="close-btn" @click="closeApproveModal">×</button>
        </div>
        <div class="modal-body">
          <div class="input-group">
            <label>申请人</label>
            <input class="form-control" :value="pendingApproveApp?.name || '-'" disabled>
          </div>
          <div class="input-group" style="margin-top: 12px;">
            <label>邮箱</label>
            <input class="form-control" :value="pendingApproveApp?.email || '-'" disabled>
          </div>
          <div class="token-generator" style="margin-top: 16px;">
            <input v-model="approveForm.token" class="form-control token-input" readonly>
            <button class="btn btn-secondary" @click="approveForm.token = generateUUID()">重新生成</button>
          </div>
          <div class="input-group" style="margin-top: 16px;">
            <label>授权范围</label>
            <div class="index-chips">
              <label v-for="uid in availableIndexes" :key="uid" :class="['chip', { selected: approveForm.allowIndexes.includes(uid) }]">
                <input type="checkbox" :value="uid" v-model="approveForm.allowIndexes"> {{ uid }}
              </label>
              <label :class="['chip all', { selected: approveForm.allowIndexes.includes('*') }]">
                <input type="checkbox" value="*" :checked="approveForm.allowIndexes.includes('*')" @change="toggleApproveAllIndexes"> [ 全部索引 * ]
              </label>
            </div>
          </div>
          <div class="grid-inputs" style="margin-top: 16px;">
            <div class="input-group">
              <label>备注</label>
              <input v-model="approveForm.description" class="form-control" placeholder="审批分发备注">
            </div>
            <div class="input-group">
              <label>有效期 (天，默认30天)</label>
              <input type="number" v-model="approveForm.validDays" min="1" class="form-control">
            </div>
            <div class="input-group">
              <label>日查询限制 (次/天)</label>
              <input type="number" v-model="approveForm.maxQueriesPerDay" placeholder="0或留空不限制" class="form-control">
            </div>
            <div class="input-group">
              <label>日导入限制 (条/天)</label>
              <input type="number" v-model="approveForm.maxImportsPerDay" placeholder="0或留空不限制" class="form-control">
            </div>
          </div>
          <div class="editor-actions">
            <button class="btn btn-primary" @click="submitApprove">保存并通过</button>
            <button class="btn btn-secondary" @click="closeApproveModal">取消</button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="rejectModalOpen" class="tasks-modal-overlay animate-fade-in" @click.self="closeRejectModal">
      <div class="tasks-modal approve-modal">
        <div class="modal-header">
          <h3>驳回申请并通知用户</h3>
          <button class="close-btn" @click="closeRejectModal">×</button>
        </div>
        <div class="modal-body">
          <div class="input-group">
            <label>申请人</label>
            <input class="form-control" :value="pendingRejectApp?.name || '-'" disabled>
          </div>
          <div class="input-group" style="margin-top: 12px;">
            <label>邮箱</label>
            <input class="form-control" :value="pendingRejectApp?.email || '-'" disabled>
          </div>
          <div class="input-group" style="margin-top: 16px;">
            <label>驳回消息（留空使用默认消息）</label>
            <textarea
              v-model="rejectForm.rejectMessage"
              class="form-control"
              rows="4"
              placeholder="很抱歉，您的 Token 申请未通过审核。"
            />
          </div>
          <div class="editor-actions">
            <button class="btn btn-danger" @click="submitReject">确认驳回并发送邮件</button>
            <button class="btn btn-secondary" @click="closeRejectModal">取消</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { generateUUID } from '@/utils'
import {
  getAdminIndexConfigs,
  getAdminAccessTokens,
  getAdminApps,
  getProxyIndexes,
  getProxyIndexStats,
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
  validateMeiliEmbedder,
  loadIndexData,
  getMeiliTasks,
  cancelMeiliTask,
  getApplications,
  approveApplication,
  rejectApplication,
  getAdminUsageMetrics
} from '@/services/api'

function formatDuration(isoDuration: string | undefined): string {
  if (!isoDuration) return '-'
  const regex = /PT(?:(\d+)H)?(?:(\d+)M)?(?:([\d.]+)S)?/
  const match = isoDuration.match(regex)
  if (!match) return isoDuration
  
  let h = parseInt(match[1] || '0', 10)
  let m = parseInt(match[2] || '0', 10)
  let s = parseFloat(match[3] || '0')
  
  if (s >= 60) {
    m += Math.floor(s / 60)
    s = s % 60
  }
  
  if (m >= 60) {
    h += Math.floor(m / 60)
    m = m % 60
  }
  
  if (h === 0 && m === 0 && s < 1) {
    return `${(s * 1000).toFixed(2)} ms`
  }
  
  let result = ''
  if (h > 0) result += `${h}时`
  if (m > 0) result += `${m}分`
  
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
  { id: 'applications', label: '申请管理', icon: '📝' },
  { id: 'metrics', label: '用量监控', icon: '📊' },
  { id: 'search', label: '搜索配置', icon: '🔍' },
  { id: 'settings', label: '应用设置', icon: '⚙️' },
  { id: 'password', label: '账户安全', icon: '🔐' },
]

const indexConfigs = ref<any[]>([])
const accessTokens = ref<any[]>([])
const apps = ref<any[]>([])
const instances = ref<any[]>([])
const availableIndexes = ref<string[]>([])
const tokenApplications = ref<any[]>([])
const usageMetrics = ref<any[]>([])
const indexUsageMetrics = ref<any[]>([])

// NOTE: 用量监控折叠及分页控制
const DAILY_METRIC_PAGE_SIZE = 6
const INDEX_METRIC_PREVIEW_LIMIT = 5

const currentPage = ref(1)
const showAllIndexMetrics = ref(false)

const totalPages = computed(() => {
  return Math.ceil(usageMetrics.value.length / DAILY_METRIC_PAGE_SIZE) || 1
})

// 计算每日明细显示数据（按日期降序，且每页显示 DAILY_METRIC_PAGE_SIZE 条记录）
const displayedUsageMetrics = computed(() => {
  const sorted = [...usageMetrics.value].sort((a, b) => {
    return new Date(b.date).getTime() - new Date(a.date).getTime()
  })
  const start = (currentPage.value - 1) * DAILY_METRIC_PAGE_SIZE
  return sorted.slice(start, start + DAILY_METRIC_PAGE_SIZE)
})

// 计算索引维度显示数据（默认仅显示前 INDEX_METRIC_PREVIEW_LIMIT 个索引）
const displayedIndexUsageMetrics = computed(() => {
  if (showAllIndexMetrics.value) {
    return indexUsageMetrics.value
  }
  return indexUsageMetrics.value.slice(0, INDEX_METRIC_PREVIEW_LIMIT)
})

const approveModalOpen = ref(false)
const pendingApproveApp = ref<any | null>(null)
const approveForm = ref({ token: '', allowIndexes: [] as string[], description: '', validDays: 30, maxQueriesPerDay: 0, maxImportsPerDay: 0 })
const rejectModalOpen = ref(false)
const pendingRejectApp = ref<any | null>(null)
const rejectForm = ref({ rejectMessage: '' })

function parseAllowIndexes(raw: string | null | undefined): string[] {
  if (!raw) return []
  try {
    const parsed = JSON.parse(raw)
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
}

function openApproveModal(app: any) {
  pendingApproveApp.value = app
  let defaultAllowIndexes: string[] = []
  try {
    defaultAllowIndexes = JSON.parse(app.allowIndexes || '[]')
  } catch {
    defaultAllowIndexes = []
  }
  approveForm.value = {
    token: generateUUID(),
    allowIndexes: defaultAllowIndexes,
    description: `申请人: ${app.name} (${app.email}) 用途: ${app.purpose || '无'}`,
    validDays: 30,
    maxQueriesPerDay: 0,
    maxImportsPerDay: 0
  }
  approveModalOpen.value = true
}

function closeApproveModal() {
  approveModalOpen.value = false
  pendingApproveApp.value = null
}

function toggleApproveAllIndexes(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  if (checked) {
    approveForm.value.allowIndexes = ['*']
  } else {
    approveForm.value.allowIndexes = []
  }
}

async function submitApprove() {
  const app = pendingApproveApp.value
  if (!app) return
  if (!approveForm.value.token) return alert('请先生成凭证')
  const validDays = Number(approveForm.value.validDays || 30)
  const result = await approveApplication(app.id, {
    token: approveForm.value.token,
    allowIndexes: approveForm.value.allowIndexes,
    description: approveForm.value.description,
    validDays: validDays > 0 ? validDays : 30,
    maxQueriesPerDay: Number(approveForm.value.maxQueriesPerDay || 0),
    maxImportsPerDay: Number(approveForm.value.maxImportsPerDay || 0)
  })
  if (!result.ok) return alert(result.message)
  alert(result.message)
  closeApproveModal()
  loadAdminData()
}

function openRejectModal(app: any) {
  pendingRejectApp.value = app
  rejectForm.value = { rejectMessage: '' }
  rejectModalOpen.value = true
}

function closeRejectModal() {
  rejectModalOpen.value = false
  pendingRejectApp.value = null
}

async function submitReject() {
  const app = pendingRejectApp.value
  if (!app) return
  const result = await rejectApplication(app.id, { rejectMessage: rejectForm.value.rejectMessage.trim() })
  if (!result.ok) return alert(result.message)
  alert(result.message)
  closeRejectModal()
  loadAdminData()
}

const showAddInstance = ref(false)
const editingInstanceId = ref<number | null>(null)
const newInstance = ref({ name: '', host: '', apiKey: '' })

const showTasksModal = ref(false)
const currentTaskInstance = ref<any>(null)
const instanceTasks = ref<any[]>([])
const isLoadingTasks = ref(false)
const isRefreshingTasks = ref(false)
const taskLastUpdatedAt = ref<Date | null>(null)

async function loadTasks(ins: any, options: { clear?: boolean } = {}) {
  if (!ins) return
  const shouldClear = options.clear === true
  if (shouldClear) {
    isLoadingTasks.value = true
    instanceTasks.value = []
    taskLastUpdatedAt.value = null
  } else {
    isRefreshingTasks.value = true
  }

  try {
    const data = await getMeiliTasks(ins.host, ins.apiKey || '')
    instanceTasks.value = data.results || []
    taskLastUpdatedAt.value = new Date()
  } catch (e) {
    alert('获取任务失败: ' + (e as Error).message)
  } finally {
    if (shouldClear) {
      isLoadingTasks.value = false
    } else {
      isRefreshingTasks.value = false
    }
  }
}

async function viewTasks(ins: any) {
  currentTaskInstance.value = ins
  showTasksModal.value = true
  await loadTasks(ins, { clear: true })
}

async function refreshTasks() {
  if (isLoadingTasks.value || isRefreshingTasks.value) return
  await loadTasks(currentTaskInstance.value)
}

async function handleCancelTask(task: any) {
  if (!confirm(`确定要终止任务 ${task.uid} 吗？`)) return
  
  try {
    const ok = await cancelMeiliTask(task.uid)
    if (ok) {
      alert('任务终止请求已发送')
      // 延迟刷新列表
      setTimeout(() => {
        if (currentTaskInstance.value) {
          refreshTasks()
        }
      }, 500)
    } else {
      alert('终止任务失败')
    }
  } catch (e) {
    alert('请求异常: ' + (e as Error).message)
  }
}

const showAddIndexConf = ref(false)
const editingIndexId = ref<number | null>(null)
const newIndex = ref({ uid: '', alias: '', description: '', isVisible: true, isLocked: false, fieldConfigs: '', viewConfigs: '', tableConfigs: '', canEdit: false })

const showAddToken = ref(false)
const editingTokenId = ref<number | null>(null)
const newToken = ref({ token: '', allowIndexes: [] as string[], description: '', validDays: null as number | null, maxQueriesPerDay: 0, maxImportsPerDay: 0 })

const passwordForm = ref({ newPassword: '', confirmPassword: '' })

// 搜索配置相关
const selectedSearchIndex = ref('')
const subTab = ref('searchable')
const currentSearchableAttributes = ref<string[]>([])
const currentFilterableAttributes = ref<string[]>([])
const allFieldsForIndex = ref<string[]>([])
const embeddingEnabled = ref(false)
const currentEmbedderName = ref('default')
const currentEmbedder = ref({
  source: 'openAi',
  model: '',
  url: '',
  apiKey: '',
  dimensions: undefined as number | undefined,
  documentTemplate: '',
  documentTemplateMaxBytes: undefined as number | undefined,
  request: '',
  response: ''
})
const isSavingSearchFields = ref(false)
const isSavingEmbedding = ref(false)
const draggedIndex = ref<number | null>(null)

const totalQueryCount = computed(() => usageMetrics.value.reduce((sum, item) => sum + Number(item.queryCount || 0), 0))
const totalImportCount = computed(() => usageMetrics.value.reduce((sum, item) => sum + Number(item.importCount || 0), 0))
const latestDatabaseSize = computed(() => {
  if (!usageMetrics.value.length) return 0
  const last = usageMetrics.value[usageMetrics.value.length - 1]
  return Number(last.databaseSize || 0)
})
const maxChartValue = computed(() => {
  const maxInQuery = usageMetrics.value.reduce((m, item) => Math.max(m, Number(item.queryCount || 0)), 0)
  const maxInImport = usageMetrics.value.reduce((m, item) => Math.max(m, Number(item.importCount || 0)), 0)
  return Math.max(1, maxInQuery, maxInImport)
})
const chartSvgWidth = 1000
const chartSvgHeight = 260
const chartPlotLeft = 40
const chartPlotRight = 20
const chartPlotTop = 20
const chartPlotBottom = 220
const chartPlotWidth = chartSvgWidth - chartPlotLeft - chartPlotRight
const chartPlotHeight = chartPlotBottom - chartPlotTop
type ChartPoint = {
  index: number
  date: string
  queryCount: number
  importCount: number
  x: number
  queryY: number
  importY: number
  tooltipY: number
}
type ChartAxisTick = {
  value: number
  label: string
  y: number
}
type ChartAxisLabel = {
  value: string
  label: string
  x: number
}
const chartDataPoints = computed<ChartPoint[]>(() => buildChartDataPoints())
const chartYAxisTicks = computed<ChartAxisTick[]>(() => buildChartYAxisTicks())
const chartXAxisLabels = computed<ChartAxisLabel[]>(() => buildChartXAxisLabels())
const chartPointsQuery = computed(() => buildLinePoints('queryCount'))
const chartPointsImport = computed(() => buildLinePoints('importCount'))
const hoveredChartIndex = ref<number | null>(null)
const chartHoverPosition = ref<{ x: number; y: number } | null>(null)
const hoveredChartPoint = computed(() => {
  if (hoveredChartIndex.value === null) return null
  return chartDataPoints.value[hoveredChartIndex.value] || null
})
const chartTooltipStyle = computed(() => {
  const point = hoveredChartPoint.value
  if (!point) return {}
  const left = Math.min(Math.max((point.x / chartSvgWidth) * 100, 10), 90)
  const top = Math.max(8, (point.tooltipY / chartSvgHeight) * 100 - 12)
  const placeBelow = top < 18
  return {
    left: `${left}%`,
    top: `${placeBelow ? top + 8 : top}%`,
    transform: `translate(-50%, ${placeBelow ? '0' : '-100%'})`
  }
})

function formatBytes(bytes: number): string {
  if (!bytes || bytes <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let n = bytes
  let i = 0
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024
    i++
  }
  return `${n.toFixed(i === 0 ? 0 : 2)} ${units[i]}`
}

function buildChartDataPoints(): ChartPoint[] {
  const rows = usageMetrics.value
  const step = rows.length > 1 ? chartPlotWidth / (rows.length - 1) : 0
  return rows.map((item, idx) => {
    const queryCount = Number(item.queryCount || 0)
    const importCount = Number(item.importCount || 0)
    const x = rows.length === 1 ? chartPlotLeft + chartPlotWidth / 2 : chartPlotLeft + step * idx
    return {
      index: idx,
      date: item.date,
      queryCount,
      importCount,
      x,
      queryY: chartPlotBottom - ((queryCount / maxChartValue.value) * chartPlotHeight),
      importY: chartPlotBottom - ((importCount / maxChartValue.value) * chartPlotHeight),
      tooltipY: Math.min(
        chartPlotBottom - ((queryCount / maxChartValue.value) * chartPlotHeight),
        chartPlotBottom - ((importCount / maxChartValue.value) * chartPlotHeight),
      ),
    }
  })
}

function buildLinePoints(field: 'queryCount' | 'importCount'): string {
  const points = chartDataPoints.value
  if (!points.length) return ''
  if (points.length === 1) {
    const single = points[0]
    const y = field === 'queryCount' ? single.queryY : single.importY
    return `${chartPlotLeft},${y.toFixed(2)} ${chartPlotLeft + chartPlotWidth},${y.toFixed(2)}`
  }
  return points.map(point => {
    const y = field === 'queryCount' ? point.queryY : point.importY
    return `${point.x.toFixed(2)},${y.toFixed(2)}`
  }).join(' ')
}

function buildChartYAxisTicks(): ChartAxisTick[] {
  const max = maxChartValue.value
  const tickCount = 4
  return Array.from({ length: tickCount + 1 }, (_, idx) => {
    const ratio = idx / tickCount
    const value = Math.round(max * (1 - ratio))
    return {
      value,
      label: value.toLocaleString(),
      y: chartPlotTop + chartPlotHeight * ratio,
    }
  })
}

function formatChartLabel(date: string): string {
  const normalized = (date || '').trim()
  if (!normalized) return '-'
  const match = normalized.match(/^(\d{4})-(\d{2})-(\d{2})/)
  if (match) return `${match[2]}/${match[3]}`
  return normalized.replace(/-/g, '/')
}

function buildChartXAxisLabels(): ChartAxisLabel[] {
  const points = chartDataPoints.value
  if (!points.length) return []
  const step = Math.max(1, Math.ceil(points.length / 6))
  return points
    .filter((point, idx) => idx === 0 || idx === points.length - 1 || idx % step === 0)
    .map(point => ({
      value: point.date,
      label: formatChartLabel(point.date),
      x: point.x,
    }))
}

function handleChartHover(event: MouseEvent) {
  if (!chartDataPoints.value.length) return
  const svg = event.currentTarget as SVGSVGElement | null
  if (!svg) return
  const rect = svg.getBoundingClientRect()
  if (!rect.width) return
  const x = ((event.clientX - rect.left) / rect.width) * chartSvgWidth
  const y = ((event.clientY - rect.top) / rect.height) * chartSvgHeight
  chartHoverPosition.value = {
    x: Math.min(Math.max(x, chartPlotLeft), chartPlotLeft + chartPlotWidth),
    y: Math.min(Math.max(y, chartPlotTop), chartPlotBottom),
  }
  const nearest = chartDataPoints.value.reduce((best, point) => {
    if (!best) return point
    return Math.abs(point.x - x) < Math.abs(best.x - x) ? point : best
  }, null as ChartPoint | null)
  hoveredChartIndex.value = nearest ? nearest.index : null
}

function clearChartHover() {
  hoveredChartIndex.value = null
  chartHoverPosition.value = null
}


function openAddToken() {
  showAddToken.value = true
  editingTokenId.value = null
  newToken.value = { token: generateUUID(), allowIndexes: [], description: '', validDays: null, maxQueriesPerDay: 0, maxImportsPerDay: 0 }
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
    const [idxData, tokData, appsData, proxyData, insData, applicationData, metricsData] = await Promise.all([
      getAdminIndexConfigs(),
      getAdminAccessTokens(),
      getAdminApps(),
      getProxyIndexes(),
      getAdminInstances(),
      getApplications(),
      getAdminUsageMetrics()
    ])

    const docCountEntries = await Promise.all(
      (idxData as any[]).map(async cfg => {
        const stats = await getProxyIndexStats(cfg.uid).catch(() => null)
        return [cfg.uid, stats?.numberOfDocuments] as const
      })
    )
    const docCountMap = new Map(docCountEntries)

    indexConfigs.value = (idxData as any[]).map(cfg => ({
      ...cfg,
      documentCount: docCountMap.get(cfg.uid)
    }))
    accessTokens.value = tokData as any[]
    apps.value = appsData as any[]
    instances.value = insData as any[]
    tokenApplications.value = applicationData as any[]
    usageMetrics.value = (metricsData as any).results || []
    indexUsageMetrics.value = (metricsData as any).indexResults || []
    currentPage.value = 1
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
    isVisible: cfg.isVisible !== false,
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
  newIndex.value = { uid: '', alias: '', description: '', isVisible: true, isLocked: false, fieldConfigs: '', viewConfigs: '', tableConfigs: '', canEdit: false }
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
  const ok = await createAccessToken({
    ...newToken.value,
    allowIndexes: JSON.stringify(newToken.value.allowIndexes),
    expiresAt,
    maxQueriesPerDay: Number(newToken.value.maxQueriesPerDay || 0),
    maxImportsPerDay: Number(newToken.value.maxImportsPerDay || 0)
  })
  if (ok) {
      cancelTokenEdit()
      loadAdminData()
  }
}

async function updateToken() {
  const expiresAt = newToken.value.validDays 
    ? new Date(Date.now() + newToken.value.validDays * 24 * 60 * 60 * 1000).toISOString()
    : null
  const ok = await updateAccessToken({
    id: editingTokenId.value,
    ...newToken.value,
    allowIndexes: JSON.stringify(newToken.value.allowIndexes),
    expiresAt,
    maxQueriesPerDay: Number(newToken.value.maxQueriesPerDay || 0),
    maxImportsPerDay: Number(newToken.value.maxImportsPerDay || 0)
  })
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
    validDays: tok.expiresAt ? Math.round((new Date(tok.expiresAt).getTime() - Date.now()) / (24 * 60 * 60 * 1000)) : null,
    maxQueriesPerDay: tok.maxQueriesPerDay || 0,
    maxImportsPerDay: tok.maxImportsPerDay || 0
  }
  showAddToken.value = true
}

function cancelTokenEdit() {
  showAddToken.value = false
  editingTokenId.value = null
  newToken.value = { token: '', allowIndexes: [], description: '', validDays: null, maxQueriesPerDay: 0, maxImportsPerDay: 0 }
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
      const embedders = settings.embedders || settings.embedding || {}
      const names = Object.keys(embedders || {})
      if (names.length > 0) {
        const name = names[0]
        const cfg = embedders[name] || {}
        embeddingEnabled.value = true
        currentEmbedderName.value = name
        currentEmbedder.value = {
          source: cfg.source || 'openAi',
          model: cfg.model || '',
          url: cfg.url || '',
          apiKey: cfg.apiKey || '',
          dimensions: typeof cfg.dimensions === 'number' ? cfg.dimensions : undefined,
          documentTemplate: cfg.documentTemplate || '',
          documentTemplateMaxBytes: typeof cfg.documentTemplateMaxBytes === 'number' ? cfg.documentTemplateMaxBytes : undefined,
          request: cfg.request ? JSON.stringify(cfg.request, null, 2) : '',
          response: cfg.response ? JSON.stringify(cfg.response, null, 2) : ''
        }
      } else {
        embeddingEnabled.value = false
        currentEmbedderName.value = 'default'
        currentEmbedder.value = {
          source: 'openAi',
          model: '',
          url: '',
          apiKey: '',
          dimensions: undefined,
          documentTemplate: '',
          documentTemplateMaxBytes: undefined,
          request: '',
          response: ''
        }
      }
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

async function saveSearchFieldSettings() {
  if (!selectedSearchIndex.value) return
  isSavingSearchFields.value = true
  try {
    const payload = {
      searchableAttributes: currentSearchableAttributes.value,
      filterableAttributes: currentFilterableAttributes.value
    }
    const ok = await updateMeiliIndexSettings(selectedSearchIndex.value, payload)
    if (ok) {
      alert('搜索/过滤配置更新成功！Meilisearch 正在异步处理更新任务。')
    } else {
      alert('更新失败，请检查后端日志')
    }
  } catch (e) {
    alert('请求失败: ' + (e as Error).message)
  } finally {
    isSavingSearchFields.value = false
  }
}

function resetSearchFieldSettings() {
  currentSearchableAttributes.value = [...allFieldsForIndex.value]
  currentFilterableAttributes.value = [...allFieldsForIndex.value]
}

async function saveEmbeddingSettings() {
  if (!selectedSearchIndex.value) return
  isSavingEmbedding.value = true
  try {
    let embedders: Record<string, unknown> = {}
    if (embeddingEnabled.value) {
      const name = (currentEmbedderName.value || 'default').trim() || 'default'
      const source = currentEmbedder.value.source
      const model = currentEmbedder.value.model.trim()
      const url = currentEmbedder.value.url.trim()
      const apiKey = currentEmbedder.value.apiKey.trim()

      if (source === 'openAi') {
        if (!model) {
          alert('openAi 模式下必须填写 model')
          isSavingEmbedding.value = false
          return
        }
        if (!apiKey) {
          alert('openAi 模式下必须填写 apiKey')
          isSavingEmbedding.value = false
          return
        }
      }
      if (source === 'huggingFace' && !model) {
        alert('huggingFace 模式下必须填写 model')
        isSavingEmbedding.value = false
        return
      }
      if (source === 'ollama' && !model) {
        alert('ollama 模式下必须填写 model')
        isSavingEmbedding.value = false
        return
      }
      if (source === 'rest' && !url) {
        alert('rest 模式下必须填写 url')
        isSavingEmbedding.value = false
        return
      }
      if (source === 'userProvided' && !(typeof currentEmbedder.value.dimensions === 'number' && currentEmbedder.value.dimensions > 0)) {
        alert('userProvided 模式下必须填写 dimensions')
        isSavingEmbedding.value = false
        return
      }
      const embedderConfig: Record<string, unknown> = {
        source
      }
      if (source !== 'rest' && model) embedderConfig.model = model
      if (url) embedderConfig.url = url
      if (apiKey) embedderConfig.apiKey = apiKey

      if (source === 'rest') {
        try {
          if (currentEmbedder.value.request) {
            embedderConfig.request = JSON.parse(currentEmbedder.value.request)
          }
          if (currentEmbedder.value.response) {
            embedderConfig.response = JSON.parse(currentEmbedder.value.response)
          }
        } catch (e) {
          alert('REST 请求体或响应解析 JSON 格式错误')
          isSavingEmbedding.value = false
          return
        }
      }
      if (typeof currentEmbedder.value.dimensions === 'number' && currentEmbedder.value.dimensions > 0) {
        embedderConfig.dimensions = currentEmbedder.value.dimensions
      }
      if (currentEmbedder.value.source !== 'userProvided') {
        const documentTemplate = currentEmbedder.value.documentTemplate.trim()
        if (documentTemplate) embedderConfig.documentTemplate = documentTemplate
        if (typeof currentEmbedder.value.documentTemplateMaxBytes === 'number' && currentEmbedder.value.documentTemplateMaxBytes > 0) {
          embedderConfig.documentTemplateMaxBytes = currentEmbedder.value.documentTemplateMaxBytes
        }
      }
      embedders = { [name]: embedderConfig }

      const validation = await validateMeiliEmbedder(selectedSearchIndex.value, embedderConfig)
      if (!validation.ok) {
        alert(`Embedding 模型校验失败: ${validation.message || '未知错误'}`)
        isSavingEmbedding.value = false
        return
      }
    }

    const ok = await updateMeiliIndexSettings(selectedSearchIndex.value, { embedders })
    if (ok) {
      alert('Embedding 配置更新成功！Meilisearch 正在异步处理更新任务。')
    } else {
      alert('更新失败，请检查后端日志')
    }
  } catch (e) {
    alert('请求失败: ' + (e as Error).message)
  } finally {
    isSavingEmbedding.value = false
  }
}

function resetEmbeddingSettings() {
  embeddingEnabled.value = false
  currentEmbedderName.value = 'default'
  currentEmbedder.value = {
    source: 'openAi',
    model: '',
    url: '',
    apiKey: '',
    dimensions: undefined,
    documentTemplate: '',
    documentTemplateMaxBytes: undefined,
    request: '',
    response: ''
  }
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
  background: transparent;
  color: var(--text-main);
}

/* 侧边栏 */
.admin-sidebar {
  width: 260px;
  background: var(--surface-glass);
  backdrop-filter: var(--glass-blur);
  border-right: 1px solid var(--border);
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
.sidebar-header h2 { font-family: 'Outfit'; font-size: 20px; font-weight: 700; color: var(--text-main); }

.sidebar-nav { flex: 1; padding: 0 16px; display: flex; flex-direction: column; gap: 8px; }
.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 12px;
  border: none;
  background: transparent;
  color: var(--text-sub);
  cursor: pointer;
  transition: all 0.3s ease;
  font-weight: 500;
}
.nav-item:hover { background: var(--bg-card-hover); color: var(--text-main); }
.nav-item.active { background: rgba(var(--primary-color-rgb), 0.12); color: var(--primary); }
.nav-icon { font-size: 18px; }

.metrics-pagination {
  display: flex;
  align-items: center;
  gap: 12px;
}
.pagination-info {
  font-size: 13px;
  color: var(--text-sub);
  min-width: 60px;
  text-align: center;
}

.sidebar-footer { padding: 0 32px; font-size: 11px; color: var(--text-muted); }

/* 主内容区 */
.admin-main { flex: 1; margin-left: 260px; padding: 48px 64px; }

.content-section { max-width: 1000px; }
.metric-value { display: inline-block; margin-top: 8px; font-size: 24px; font-weight: 700; }
.metrics-grid { margin-bottom: 20px; }
.chart-panel { margin-bottom: 20px; }
.line-chart-wrap { width: 100%; }
.line-chart-stage { position: relative; }
.line-chart { width: 100%; height: 260px; display: block; border: 1px solid var(--border); border-radius: 8px; background: rgba(var(--surface-rgb), 0.35); }
.axis-line { stroke: var(--border); stroke-width: 1; }
.chart-grid-line { stroke: var(--border); stroke-width: 1; opacity: 0.35; }
.line-query { fill: none; stroke: #3b82f6; stroke-width: 2.5; }
.line-import { fill: none; stroke: #22c55e; stroke-width: 2.5; }
.chart-point-layer,
.chart-hover-points { pointer-events: none; }
.chart-point { stroke: rgba(255, 255, 255, 0.95); stroke-width: 1.5; opacity: 0.9; }
.chart-point.query { fill: #3b82f6; }
.chart-point.import { fill: #22c55e; }
.chart-point.active { stroke-width: 3; opacity: 1; }
.chart-crosshair {
  stroke: rgba(var(--primary-color-rgb), 0.28);
  stroke-width: 1.2;
  stroke-dasharray: 6 6;
  pointer-events: none;
}
.chart-axis-label {
  fill: var(--text-muted);
  font-size: 11px;
  user-select: none;
  pointer-events: none;
}
.chart-axis-label.y-label { letter-spacing: 0.01em; }
.chart-axis-label.x-label { dominant-baseline: hanging; }
.chart-tooltip {
  position: absolute;
  min-width: 150px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--border-active);
  background: rgba(var(--surface-rgb), 0.96);
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.18);
  color: var(--text-main);
  font-size: 12px;
  line-height: 1.5;
  pointer-events: none;
  z-index: 2;
}
.chart-tooltip-date { margin-bottom: 4px; font-weight: 700; color: var(--text-main); }
.chart-tooltip-row { display: flex; align-items: center; gap: 8px; white-space: nowrap; }
.chart-legend { display: flex; gap: 20px; margin-top: 10px; color: var(--text-sub); font-size: 13px; }
.legend-item { display: inline-flex; align-items: center; gap: 8px; }
.legend-dot { width: 10px; height: 10px; border-radius: 50%; display: inline-block; }
.legend-dot.query { background: #3b82f6; }
.legend-dot.import { background: #22c55e; }

.user-info { display: flex; flex-direction: column; gap: 2px; }
.user-info strong { font-size: 14px; color: var(--text-primary); }
.user-info .meta { font-size: 11px; color: var(--text-muted); }
.purpose-info { 
  max-width: 200px; 
  font-size: 13px; 
  color: var(--text-secondary); 
  white-space: nowrap; 
  overflow: hidden; 
  text-overflow: ellipsis; 
}
.index-tags { display: flex; flex-wrap: wrap; gap: 4px; }
.index-tags .tag { 
  background: rgba(var(--primary-color-rgb), 0.1); 
  color: var(--primary-color); 
  padding: 2px 6px; 
  border-radius: 4px; 
  font-size: 11px; 
}
.text-muted { color: var(--text-muted); font-size: 12px; }

.applications-mobile {
  display: none;
}

.application-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.application-card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.application-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.application-field label {
  font-size: 12px;
  color: var(--text-muted);
}

.application-purpose {
  color: var(--text-main);
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.application-actions {
  display: flex;
  gap: 8px;
  padding-top: 6px;
}

.section-header { margin-bottom: 40px; display: flex; flex-direction: column; gap: 8px; position: relative; }
.section-header h1 { font-family: 'Outfit'; font-size: 32px; font-weight: 700; color: var(--text-main); }
.section-header h1 span { font-weight: 300; opacity: 0.3; margin-left: 8px; font-size: 0.6em; }
.section-header p { color: var(--text-sub); font-size: 15px; }
.section-header .btn { position: absolute; right: 0; top: 0; }

/* 编辑器容器 */
.glass-editor {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 20px;
  padding: 32px;
  margin-bottom: 32px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
}
.glass-editor h3 { margin-bottom: 24px; font-size: 18px; color: var(--text-main); }
.grid-inputs { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; }
.input-group label { display: block; font-size: 13px; color: var(--text-muted); margin-bottom: 8px; }
.editor-actions { margin-top: 32px; display: flex; gap: 12px; justify-content: flex-end; }

/* 数据卡片 */
.data-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 24px; }
.data-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  transition: all 0.3s ease;
}
.data-card:hover { transform: translateY(-4px); border-color: var(--border-active); }
.card-info { display: flex; align-items: center; gap: 16px; }
.ins-avatar { width: 48px; height: 48px; border-radius: 12px; background: var(--primary); color: white; font-weight: 700; font-size: 20px; display: flex; align-items: center; justify-content: center; }
.card-info h4 { font-size: 16px; color: var(--text-main); margin-bottom: 4px; }
.card-info code { font-size: 12px; color: var(--primary); }
.card-ops { display: flex; gap: 10px; border-top: 1px solid var(--border); pt: 16px; padding-top: 16px; }

/* Token 特殊样式 */
.token-list { display: flex; flex-direction: column; gap: 16px; }
.token-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.tok-tag { font-size: 9px; font-weight: 800; color: var(--primary); letter-spacing: 0.1em; background: rgba(var(--primary-color-rgb), 0.1); padding: 2px 6px; border-radius: 4px; margin-right: 8px; }
.tok-val { font-size: 16px; color: #fbbf24; font-family: monospace; display: block; margin: 12px 0; }
.tok-desc { color: var(--text-main); font-size: 14px; margin-bottom: 8px; }
.tok-meta { display: flex; gap: 24px; font-size: 12px; color: var(--text-muted); }

/* 设置 */
.settings-card { display: flex; flex-direction: column; gap: 32px; }

.settings-row { display: flex; gap: 48px; align-items: flex-start; }

.row-label { width: 240px; }
.row-label h4 { font-size: 16px; color: var(--text-main); margin-bottom: 4px; }
.row-label p { font-size: 13px; color: var(--text-muted); }
.row-val { flex: 1; }

.app-identity { display: flex; align-items: center; gap: 12px; font-size: 20px; }
.json-preview { background: rgba(0, 0, 0, 0.2); padding: 16px; border-radius: 8px; font-size: 12px; color: #10b981; max-height: 200px; overflow: auto; margin-bottom: 12px; }
.json-editor { font-family: monospace; font-size: 13px; line-height: 1.4; color: #10b981; background: rgba(0, 0, 0, 0.3) !important; }

/* 通用列表项 */
.alias-info { display: flex; flex-direction: column; gap: 4px; }
.alias { color: var(--text-main); font-weight: 600; }
.desc { font-size: 12px; color: var(--text-muted); }
.doc-count {
  color: var(--text-main);
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

/* 芯片多选 */
.index-chips { display: flex; flex-wrap: wrap; gap: 8px; }
.chip { padding: 6px 12px; background: var(--bg-card); border-radius: 8px; font-size: 13px; color: var(--text-sub); cursor: pointer; border: 1px solid transparent; }
.chip:hover { background: var(--bg-card-hover); }
.chip.selected { background: rgba(var(--primary-color-rgb), 0.15); border-color: var(--primary); color: var(--text-main); }
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
  color: var(--text-main);
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
  color: var(--text-muted);
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
  color: var(--text-sub);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.sub-tab.active {
  background: var(--primary);
  color: white;
  box-shadow: 0 4px 12px rgba(var(--primary-color-rgb), 0.3);
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
  color: var(--text-muted);
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
  color: var(--text-main);
}

.remove-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
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
  color: var(--text-muted);
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
  border-bottom: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.modal-header h3 {
  margin: 0;
  color: var(--text-main);
  font-size: 18px;
}
.modal-title {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.task-refresh-time {
  color: var(--text-muted);
  font-size: 12px;
}
.modal-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}
.modal-actions .btn:disabled {
  cursor: not-allowed;
  opacity: 0.6;
  transform: none;
}
.close-btn {
  background: transparent;
  border: none;
  color: var(--text-sub);
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
  color: var(--text-sub);
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
.task-json-preview {
  margin: 0;
  font-family: monospace;
  font-size: 13px;
  color: #000000;
}

.approve-modal {
  width: 760px;
}

/* ============ Mobile Responsiveness ============ */
@media (max-width: 768px) {
  .admin-layout {
    flex-direction: column;
  }
  
  .admin-sidebar {
    position: static;
    width: 100%;
    height: auto;
    padding: 16px 0;
    border-right: none;
    border-bottom: 1px solid var(--border);
  }
  
  .sidebar-header {
    margin-bottom: 16px;
    padding: 0 16px;
  }
  
  .sidebar-nav {
    flex-direction: row;
    overflow-x: auto;
    padding: 0 16px;
    -webkit-overflow-scrolling: touch;
  }
  
  .nav-item {
    white-space: nowrap;
    flex-shrink: 0;
  }
  
  .sidebar-footer {
    display: none;
  }
  
  .admin-main {
    margin-left: 0;
    padding: 16px;
  }
  
  .section-header h1 {
    font-size: 24px;
  }
  
  .section-header .btn {
    position: static;
    margin-top: 16px;
    display: block;
    width: 100%;
    text-align: center;
  }
  
  .settings-row {
    flex-direction: column;
    gap: 16px;
  }
  
  .row-label {
    width: 100%;
  }
  
  .data-grid {
    grid-template-columns: 1fr;
  }

  .applications-table {
    display: none;
  }

  .applications-mobile {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .application-actions .btn {
    flex: 1;
  }

  .application-card .index-tags {
    gap: 6px;
  }
  
  .tasks-modal {
    width: 100%;
    max-width: 100vw;
    height: 100vh;
    max-height: 100vh;
    border-radius: 0;
  }
  
  .glass-editor {
    padding: 16px;
  }
  
  .field-selector {
    grid-template-columns: 1fr;
  }
}
</style>
