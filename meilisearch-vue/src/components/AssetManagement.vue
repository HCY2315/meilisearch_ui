<template>
  <div class="asset-management">
    <div class="asset-header">
      <h2>📦 批量新增</h2>
    </div>

    <!-- 创建新索引 -->
    <div class="section-card">
      <h3>创建新索引</h3>
      <div class="row-flex">
        <div class="form-group">
          <label>索引 UID</label>
          <input class="form-control" v-model="store.newIndexUid" placeholder="my_index" />
        </div>
        <div class="form-group">
          <label>主键字段</label>
          <input class="form-control" v-model="store.newIndexPk" placeholder="id" />
        </div>
        <button class="btn btn-primary" @click="store.createIndex()" style="align-self:flex-end">
          创建索引
        </button>
      </div>
    </div>

    <!-- 无索引提示 -->
    <div v-if="!store.currentIndex" class="warning-box">
      <strong>⚠️ 请选择或创建索引后进行批量导入</strong>
    </div>

    <template v-else>
      <!-- 当前索引信息 -->
      <div class="info-box">
        <span>当前索引: <strong>{{ store.currentIndex }}</strong></span>
        <span>主键字段: <strong>{{ store.primaryKeyField }}</strong></span>
      </div>

      <!-- JSON 文件上传 -->
      <div class="section-card">
        <h3>选择 JSON 文件</h3>
        <input type="file" accept=".json" @change="onFileChange" />
        <p class="hint">支持 JSON 数组格式</p>

        <div v-if="store.uploadLoading" class="loading-text">正在解析文件...</div>

        <div v-if="store.uploadFileName" class="file-info">
          ✓ 已选择: {{ store.uploadFileName }} ({{ store.uploadPreviewData.length }} 条数据)
        </div>
      </div>

      <!-- 导入进度 -->
      <div v-if="store.uploadProgress > 0" class="progress-bar">
        <div class="progress-fill" :style="{ width: store.uploadProgress + '%' }"></div>
        <p>导入进度: {{ store.uploadProgress.toFixed(0) }}%</p>
      </div>

      <!-- 数据预览 -->
      <div v-if="store.uploadPreviewData.length" class="section-card">
        <h3>数据预览 ({{ previewStart }} - {{ previewEnd }}，共 {{ store.uploadPreviewData.length }} 条)</h3>
        <div class="preview-list">
          <div v-for="(item, i) in previewItems" :key="i" class="preview-item">
            <div class="preview-num">#{{ previewStart + i }}</div>
            <pre>{{ JSON.stringify(item, null, 2) }}</pre>
          </div>
        </div>

        <!-- 预览分页 -->
        <div v-if="previewTotalPages > 1" class="preview-pagination">
          <button class="btn btn-secondary btn-sm" :disabled="store.uploadPreviewPage === 1" @click="store.uploadPreviewPage--">‹</button>
          <span>{{ store.uploadPreviewPage }} / {{ previewTotalPages }}</span>
          <button class="btn btn-secondary btn-sm" :disabled="store.uploadPreviewPage >= previewTotalPages" @click="store.uploadPreviewPage++">›</button>
        </div>
      </div>

      <!-- 确认导入 -->
      <div v-if="store.uploadPreviewData.length && !store.loading" class="import-action">
        <button class="btn btn-primary btn-lg" @click="store.batchImport()">
          📤 确认导入 ({{ store.uploadPreviewData.length }} 条)
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/composables/useApp'

const store = useAppStore()

const previewStart = computed(() => (store.uploadPreviewPage - 1) * store.uploadPreviewPageSize + 1)
const previewEnd = computed(() => Math.min(store.uploadPreviewPage * store.uploadPreviewPageSize, store.uploadPreviewData.length))
const previewTotalPages = computed(() => Math.ceil(store.uploadPreviewData.length / store.uploadPreviewPageSize))

const previewItems = computed(() =>
  store.uploadPreviewData.slice(previewStart.value - 1, previewEnd.value)
)

async function onFileChange(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  const text = await file.text()
  store.uploadFileName = file.name
  store.parseUploadData(text)
}
</script>

<style scoped>
.asset-management { display: flex; flex-direction: column; gap: 16px; }
.asset-header h2 { margin: 0; font-size: 18px; }
.section-card {
  background: var(--surface);
  border-radius: var(--radius);
  padding: 16px;
}
.section-card h3 { margin: 0 0 12px 0; font-size: 15px; }
.row-flex { display: flex; gap: 12px; align-items: flex-end; flex-wrap: wrap; }
.row-flex .form-group { flex: 1; min-width: 200px; margin: 0; }
.warning-box {
  background: rgba(245,158,11,0.1);
  padding: 16px;
  border-radius: var(--radius);
  border-left: 3px solid var(--warning-color);
  color: var(--warning-color);
}
.info-box {
  background: var(--surface);
  padding: 12px 16px;
  border-radius: var(--radius);
  display: flex;
  gap: 24px;
  font-size: 14px;
}
.hint { margin: 8px 0 0; font-size: 12px; color: var(--text-muted); }
.loading-text { margin-top: 12px; color: var(--text-muted); }
.file-info { margin-top: 12px; padding: 8px 12px; background: var(--bg-secondary); border-radius: 4px; color: var(--success-color); }
.progress-bar { margin: 8px 0; }
.progress-fill {
  height: 8px;
  background: var(--primary-color);
  border-radius: 4px;
  transition: width 0.3s;
}
.progress-bar p { margin: 4px 0 0; font-size: 12px; color: var(--text-muted); }
.preview-list { max-height: 400px; overflow-y: auto; display: flex; flex-direction: column; gap: 8px; }
.preview-item { background: var(--bg-secondary); padding: 8px; border-radius: 4px; font-size: 12px; }
.preview-num { color: var(--text-muted); margin-bottom: 4px; }
.preview-item pre { margin: 0; white-space: pre-wrap; word-break: break-all; max-height: 120px; overflow: auto; }
.preview-pagination { display: flex; gap: 8px; align-items: center; justify-content: center; margin-top: 12px; }
.import-action { text-align: center; }
.btn-lg { padding: 12px 24px; font-size: 14px; }
</style>
