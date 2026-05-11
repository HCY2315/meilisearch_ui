# 🚀 MeiliSearch UI 商业化改造方案

## 📋 核心优势分析

### 你现有系统的独特卖点：
```
✅ GUID-based 多租户架构  → 天然支持 SaaS 模式
✅ 索引级别的私有化控制   → 满足数据隐私合规需求
✅ 后端代理 + Meilisearch  → 可控成本的高性能搜索
✅ 前后端完整实现          → 可直接商业部署
✅ 配置级权限管理          → 细粒度的功能订阅
```

---

## 🎯 Phase 1: SaaS 基础设施升级（0-2个月）

### 1.1 多租户完全隔离

**当前状态：** Bearer Token 认证 + Index 级别隔离

**需要实现：**

```typescript
// src/middleware/guid-auth.ts（新文件）
interface TenantContext {
  guid: string                    // 租户唯一标识
  tier: 'free' | 'pro' | 'enterprise'
  quotas: {
    maxQueriesPerMonth: number
    maxIndexes: number
    maxDocumentsPerIndex: number
    maxStorageGB: number
    apiRateLimit: number          // QPS
  }
  features: {
    customization: boolean         // 字段配置权限
    export: boolean               // CSV导出
    aiSearch: boolean             // AI搜索
    webhooks: boolean             // Webhook集成
    apiKeys: boolean              // 多API密钥
  }
}

// 后端验证逻辑（示例 - Rust/Go）
pub async fn verify_tenant_access(
    guid: &str,
    index_uid: &str,
) -> Result<TenantContext, AuthError> {
    // 1. 验证 GUID 有效性
    let tenant = db.get_tenant(guid)?;
    
    // 2. 验证索引所有权（index_uid 必须属于该 GUID）
    let index = db.get_index(index_uid)?;
    if index.owner_guid != guid {
        return Err(AuthError::Unauthorized);
    }
    
    // 3. 检查配额和功能
    let context = build_tenant_context(&tenant);
    Ok(context)
}
```

**前端修改：**

```typescript
// src/composables/useApp.ts
export const useAppStore = defineStore('app', () => {
  const currentTenantGuid = ref(localStorage.getItem('tenantGuid') || '')
  const tenantTier = ref<'free' | 'pro' | 'enterprise'>('free')
  const tenantQuotas = ref({ /* ... */ })
  const tenantFeatures = ref({ /* ... */ })
  
  // 在 connect() 之前调用
  async function loadTenantContext() {
    try {
      const res = await fetch('/api/v1/tenant/context', {
        headers: { 'X-Tenant-GUID': currentTenantGuid.value }
      })
      const data = await res.json()
      tenantTier.value = data.tier
      tenantQuotas.value = data.quotas
      tenantFeatures.value = data.features
    } catch (e) {
      pushToast('无法加载租户信息', 'error')
    }
  }
  
  // 功能开关检查
  const canExportCsv = computed(() => tenantFeatures.value.export)
  const canUseAiSearch = computed(() => tenantFeatures.value.aiSearch)
  
  return { /* ... */, loadTenantContext, tenantTier, canExportCsv, canUseAiSearch }
})
```

---

### 1.2 用量统计和配额管理

**数据库表设计：**

```sql
-- 租户信息表
CREATE TABLE tenants (
  guid VARCHAR(36) PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL,
  tier ENUM('free', 'pro', 'enterprise') DEFAULT 'free',
  created_at TIMESTAMP,
  updated_at TIMESTAMP,
  status ENUM('active', 'suspended', 'cancelled') DEFAULT 'active'
);

-- 计费指标表
CREATE TABLE usage_metrics (
  id BIGINT PRIMARY KEY AUTO_INCREMENT,
  guid VARCHAR(36) NOT NULL,
  metric_date DATE NOT NULL,
  queries_count INT DEFAULT 0,
  documents_imported INT DEFAULT 0,
  storage_bytes BIGINT DEFAULT 0,
  exports_count INT DEFAULT 0,
  UNIQUE(guid, metric_date),
  FOREIGN KEY(guid) REFERENCES tenants(guid)
);

-- 索引权限表（替代现有简单的 index.owner_guid）
CREATE TABLE index_access (
  index_uid VARCHAR(255) NOT NULL,
  guid VARCHAR(36) NOT NULL,
  permission ENUM('read', 'write', 'admin') DEFAULT 'read',
  created_at TIMESTAMP,
  PRIMARY KEY(index_uid, guid),
  FOREIGN KEY(guid) REFERENCES tenants(guid)
);
```

**后端实现（Rust 示例）：**

```rust
// src/quota_manager.rs
pub struct QuotaManager {
    db: Database,
}

impl QuotaManager {
    pub async fn check_query_quota(
        &self,
        guid: &str,
        tier: &str,
    ) -> Result<(), QuotaExceeded> {
        let today = chrono::today();
        let usage = self.db.get_usage(guid, today).await?;
        let limit = self.get_query_limit(tier);
        
        if usage.queries_count >= limit {
            return Err(QuotaExceeded::MonthlyQuota);
        }
        Ok(())
    }
    
    pub async fn record_query(guid: &str, timestamp: DateTime) -> Result<()> {
        self.db.increment_usage_counter(guid, "queries_count", timestamp).await
    }
    
    fn get_query_limit(&self, tier: &str) -> i32 {
        match tier {
            "free" => 10_000,      // 10k/月
            "pro" => 1_000_000,    // 100万/月
            "enterprise" => i32::MAX,
        }
    }
}
```

---

### 1.3 动态定价和订阅管理

**前端新增页面：** `src/views/BillingDashboard.vue`

```vue
<template>
  <div class="billing-dashboard">
    <!-- 当前计划卡片 -->
    <div class="plan-card">
      <h3>当前计划: {{ tenantTier }}</h3>
      <p class="price">{{ planPrice }}元/月</p>
      <button @click="showUpgradeModal = true">升级计划</button>
    </div>
    
    <!-- 用量总览 -->
    <div class="usage-cards">
      <div class="usage-item">
        <label>本月查询数</label>
        <p class="value">{{ usageMetrics.queries }} / {{ quotas.maxQueriesPerMonth }}</p>
        <div class="progress-bar">
          <div class="progress" :style="{ width: usagePercent + '%' }"></div>
        </div>
      </div>
      
      <div class="usage-item">
        <label>索引数</label>
        <p class="value">{{ indexCount }} / {{ quotas.maxIndexes }}</p>
      </div>
      
      <div class="usage-item">
        <label>存储空间</label>
        <p class="value">{{ storageGB }}GB / {{ quotas.maxStorageGB }}GB</p>
      </div>
    </div>
    
    <!-- 价格对比表 -->
    <PricingTable 
      :current-tier="tenantTier"
      @upgrade="handleUpgrade"
    />
    
    <!-- 账单历史 -->
    <div class="billing-history">
      <h3>账单历史</h3>
      <table>
        <thead>
          <tr>
            <th>日期</th>
            <th>金额</th>
            <th>状态</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="bill in bills" :key="bill.id">
            <td>{{ bill.date }}</td>
            <td>¥{{ bill.amount }}</td>
            <td>{{ bill.status }}</td>
            <td><a href="#" @click="downloadInvoice(bill.id)">发票</a></td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAppStore } from '@/composables/useApp'

const app = useAppStore()
const showUpgradeModal = ref(false)
const usageMetrics = ref({ queries: 0, documents: 0, storage: 0 })
const bills = ref([])

const planPrice = computed(() => {
  return app.tenantTier === 'free' ? '0' : 
         app.tenantTier === 'pro' ? '99' : '599'
})

const usagePercent = computed(() => {
  return (usageMetrics.value.queries / app.tenantQuotas.maxQueriesPerMonth) * 100
})

async function loadUsageMetrics() {
  const res = await fetch('/api/v1/tenant/usage', {
    headers: { 'X-Tenant-GUID': app.currentTenantGuid }
  })
  usageMetrics.value = await res.json()
}

async function handleUpgrade(newTier: string) {
  // 调用支付网关或直接更新
  const res = await fetch('/api/v1/tenant/upgrade', {
    method: 'POST',
    headers: { 'X-Tenant-GUID': app.currentTenantGuid },
    body: JSON.stringify({ newTier })
  })
  if (res.ok) {
    app.loadTenantContext()
    showUpgradeModal.value = false
  }
}

onMounted(() => {
  loadUsageMetrics()
})
</script>
```

---

## 🎯 Phase 2: SaaS 核心功能（2-3个月）

### 2.1 API Keys 管理（多密钥支持）

```typescript
// src/views/AdminPanel.vue 中新增 ApiKeysTab

interface ApiKey {
  id: string
  name: string
  key: string                    // 只显示前4位和后4位
  permissions: string[]
  createdAt: string
  lastUsedAt?: string
  expiresAt?: string
  quotaLimit?: {
    qpm: number                  // Queries Per Month
    rps: number                  // Requests Per Second
  }
}

// 生成 API Key（后端）
pub async fn create_api_key(
    guid: &str,
    req: CreateApiKeyRequest,
) -> Result<ApiKey> {
    let key = generate_random_key(32);  // 类似 sk_live_xxxxx
    let hashed = hash_sha256(&key);
    
    db.save_api_key(ApiKeyRecord {
        guid: guid.to_string(),
        name: req.name,
        key_hash: hashed,
        permissions: req.permissions,
        created_at: now(),
        quota_qpm: req.quota_limit.qpm,
    }).await?;
    
    Ok(ApiKey {
        id: generate_id(),
        key: format!("sk_live_{}", &key[..28]),  // 隐藏大部分
        // ...
    })
}
```

### 2.2 Webhook 集成

```typescript
// src/views/WebhooksTab.vue

interface Webhook {
  id: string
  url: string
  events: string[]              // ['index.created', 'search.performed', 'error.occurred']
  secret: string                // 用于签名验证
  isActive: boolean
  retryPolicy: {
    maxAttempts: number
    backoffMultiplier: number
  }
}

// 后端发送 Webhook 事件
pub async fn trigger_webhook(
    event: &str,
    tenant_guid: &str,
    payload: serde_json::Value,
) -> Result<()> {
    let webhooks = db.get_webhooks(tenant_guid, event).await?;
    
    for webhook in webhooks {
        let signature = generate_signature(&webhook.secret, &payload);
        
        let client = reqwest::Client::new();
        client.post(&webhook.url)
            .header("X-Webhook-Signature", signature)
            .json(&payload)
            .send()
            .await?;
    }
    Ok(())
}
```

### 2.3 自定义域名绑定

```rust
// src/custom_domain.rs
pub struct CustomDomainManager {
    db: Database,
    dns_provider: CloudflareAPI,  // 或其他 DNS 服务
}

impl CustomDomainManager {
    pub async fn add_custom_domain(
        &self,
        guid: &str,
        domain: &str,
    ) -> Result<CustomDomain> {
        // 1. 验证域名所有权（CNAME 或 TXT 记录）
        self.verify_domain_ownership(domain).await?;
        
        // 2. 生成 SSL 证书（Let's Encrypt）
        let cert = self.generate_ssl_certificate(domain).await?;
        
        // 3. 配置反向代理
        self.db.save_custom_domain(CustomDomainRecord {
            guid: guid.to_string(),
            domain: domain.to_string(),
            ssl_cert: cert,
            created_at: now(),
        }).await?;
        
        Ok(CustomDomain { domain: domain.to_string() })
    }
}
```

---

## 🎯 Phase 3: 行业垂直解决方案（3-6个月）

### 方案A：电商搜索（SearchBox Pro）

**目标用户：** 中小型电商企业、SaaS 电商平台

**关键特性：**

```typescript
// src/plugins/ecommerce/ProductSearch.vue
interface ProductSearchConfig {
  // 商品字段映射
  fieldMappings: {
    productId: string           // SKU
    name: string               // 商品名
    category: string           // 分类
    price: string              // 价格
    stock: string              // 库存
    images: string[]           // 图片URL
    rating: string             // 评分
  }
  
  // 搜索行为配置
  facets: {
    category: true             // 分类多选
    priceRange: {              // 价格区间
      enabled: true
      ranges: [[0, 100], [100, 500], [500, 5000]]
    }
    rating: true               // 评分筛选
    inStock: true              // 库存状态
  }
  
  // 排序规则
  sortingStrategies: [
    'relevance',               // 相关性
    'price_asc', 'price_desc',
    'newest', 'bestseller',
    'rating_desc'
  ]
  
  // 推荐配置
  recommendations: {
    enableSimilarProducts: true
    enableBuyTogetherSuggestions: true
  }
}

// 集成示例：Shopify, WooCommerce, 自建系统
export async function syncProductsFromShopify(
  shopId: string,
  accessToken: string
): Promise<void> {
  const shopify = new ShopifyAPI(shopId, accessToken)
  const products = await shopify.getAllProducts()
  
  // 转换为 Meilisearch 文档
  const docs = products.map(p => ({
    id: p.id,
    name: p.title,
    description: p.body_html,
    category: p.product_type,
    price: p.variants[0].price,
    stock: p.variants[0].inventory_quantity,
    images: p.images.map(img => img.src),
    rating: p.rating?.average ?? 0,
    url: `https://${shopId}.myshopify.com/products/${p.handle}`
  }))
  
  await meilisearch.importDocuments(docs)
}
```

**定价示例：**
```
SearchBox Pro 套餐
├─ Starter: ¥199/月
│  ├─ 最多 50,000 个 SKU
│  ├─ 100 万次查询/月
│  └─ 基础分析
├─ Professional: ¥499/月
│  ├─ 最多 500,000 个 SKU
│  ├─ 1000 万次查询/月
│  ├─ 高级推荐
│  └─ API 集成
└─ Enterprise: 自定义
   ├─ 无限 SKU
   ├─ 多品牌管理
   └─ 专属客服
```

### 方案B：文档全文检索（DocSearch）

**目标用户：** SaaS 企业、知识库、内部文档管理

```typescript
// src/plugins/doc-search/DocumentIndexing.ts
interface DocumentConfig {
  sources: {
    confluence: { apiToken: string; spaces: string[] }
    notion: { apiKey: string; databaseIds: string[] }
    sharepoint: { tenantId: string; sites: string[] }
    s3: { bucket: string; prefix: string }
  }
  
  indexingStrategy: {
    fullText: boolean
    ocr: boolean                // 扫描 PDF 中的文字
    semanticEmbedding: boolean  // 向量化用于语义搜索
  }
  
  metadataExtraction: {
    author: true
    createdDate: true
    modifiedDate: true
    department: true
    classification: 'public' | 'internal' | 'confidential'
  }
  
  permissions: {
    inheritFromSource: boolean  // 继承原文档权限
    customRules: AccessRule[]
  }
}

// 自动同步
export async function syncConfluenceSpace(
  spaceKey: string,
  config: DocumentConfig
) {
  const confluence = new ConfluenceAPI(config.sources.confluence.apiToken)
  const pages = await confluence.getPages(spaceKey)
  
  const docs = pages.map(page => ({
    id: `conf_${page.id}`,
    title: page.title,
    content: page.body,
    author: page.author.name,
    createdAt: page.created,
    url: page.links.webui,
    metadata: {
      source: 'confluence',
      spaceKey,
      type: page.type
    }
  }))
  
  await meilisearch.importDocuments(docs)
}
```

---

## 💰 定价策略

### 分层定价模型

```
Free 计划（永久免费）
├─ 用户数: 1
├─ 索引数: 2
├─ 文档数: 100k/索引
├─ 查询数: 10k/月
├─ 功能: 基础搜索、字段配置
└─ 支持: 社区论坛

Pro 计划（¥99/月）
├─ 用户数: 5
├─ 索引数: 20
├─ 文档数: 5M/索引
├─ 查询数: 100万/月
├─ 功能: + API Keys + CSV 导出 + AI搜索
└─ 支持: 邮件支持 (24h)

Business 计划（¥299/月）
├─ 用户数: 不限
├─ 索引数: 不限
├─ 文档数: 不限
├─ 查询数: 1000万/月
├─ 功能: + Webhooks + 自定义域名 + 优先队列
└─ 支持: 电话 + 邮件（4h）

Enterprise（自定义）
├─ 独立部署
├─ SLA 保证 (99.9% uptime)
├─ 专属技术支持
└─ 定制化功能开发
```

### 超额计费

```
Free → Pro 升级后：
- 额外查询: ¥0.01 per 1000 queries
- 额外存储: ¥0.1 per GB/月
- 额外用户: ¥5 per user/月

按需付费示例：
- 1000万 QPM + 10GB 存储 = 基础费 + ¥50 + ¥1 = ¥150
```

---

## 🔧 技术改造清单

### 必须做的（Critical）
- [ ] 完整的 GUID 隔离（数据库级别）
- [ ] 配额检查中间件
- [ ] 用量统计和计费系统
- [ ] 多 API Key 支持
- [ ] 租户信息管理 API
- [ ] 支付网关集成（Stripe/支付宝）

### 应该做的（Important）
- [ ] Webhook 系统
- [ ] 白标/自定义域名
- [ ] 高级监控仪表板
- [ ] 审计日志
- [ ] 两因素认证 (2FA)
- [ ] 数据导出 API

### 可以做的（Nice to have）
- [ ] 垂直行业模板
- [ ] GraphQL API
- [ ] 离线搜索 SDK
- [ ] 浏览器扩展
- [ ] 移动应用

---

## 📊 上线检查清单

```javascript
{
  security: [
    '✅ GUID 隔离验证',
    '✅ API Key 密钥管理',
    '✅ HTTPS 强制',
    '✅ CORS 策略',
    '✅ 速率限制',
    '✅ DDoS 防护',
    '✅ 数据加密 (at rest + in transit)'
  ],
  
  reliability: [
    '✅ 数据库备份 (日 + 周 + 月)',
    '✅ 灾难恢复计划 (RTO < 1h)',
    '✅ 监控告警系统',
    '✅ 健康检查 (5min interval)',
    '✅ 自动扩展配置'
  ],
  
  usability: [
    '✅ 完整 API 文档 (Swagger/OpenAPI)',
    '✅ SDK (Python, JS, Go, Rust)',
    '✅ 快速入门指南',
    '✅ 代码示例库',
    '✅ 视频教程'
  ],
  
  compliance: [
    '✅ 用户协议和隐私政策',
    '✅ GDPR 合规 (数据删除)',
    '✅ CCPA 合规',
    '✅ 数据处理协议 (DPA)',
    '✅ SOC 2 Type II (后期)'
  ]
}
```

---

## 🎯 第一阶段（1-3个月）快速启动方案

如果你想快速进入市场，建议这样做：

### Week 1-2：基础设施
- 部署到云端（AWS/阿里云/腾讯云）
- 设置自动备份
- 配置 CDN

### Week 3-4：多租户改造
- 添加 GUID 隔离检查
- 实现配额管理
- 创建用量仪表板

### Week 5-8：商业功能
- 支付网关（Stripe 或 local payment）
- API Keys 管理
- 用户鉴权系统

### Week 9-12：行业定制
- 选择一个垂直市场（电商/文档搜索）
- 创建模板和示例
- 发布到 Product Hunt

---

## 💬 问题和建议

1. **当前用户认证方式是什么？** (JWT token? Session?)
   - 需要清楚了解，以便无缝迁移到多租户

2. **后端使用的数据库是？** (PostgreSQL? MySQL? MongoDB?)
   - 影响配额和用量统计的实现

3. **已有用户群体吗？**
   - 可以从他们收集付费意愿和需求优先级

4. **部署方式？** (Self-hosted? Cloud? 混合?)
   - 影响商业模式定位

---

**下一步行动：** 
- 优先实现 Phase 1 (多租户 + 配额)
- 选择你要聚焦的垂直市场
- 联系 3-5 个早期客户进行试用和反馈收集

需要我帮你进一步细化某个具体部分吗？比如支付网关集成、Webhook 实现细节或特定的垂直方案？
