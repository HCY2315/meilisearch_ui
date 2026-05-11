```
src/
├── composables/
│   ├── index.ts                    # 统一导出
│   ├── useConnectionStore.ts       # 连接
│   ├── useSearchStore.ts       # 搜索
│   ├── useUIStore.ts       # UI
│   ├── useImportStore.ts # 导入/导出
│   └── useAssetStore.ts   # 资产
├── services/
│   └── api.ts              # 统一 API 封装
├── components/
│   ├── SearchSection.vue    # 已迁移
│   ├── AdminPanel.vue    # 已迁移
│   └── ...
├── views/
│   ├── SearchMain.vue   # 已迁移
│   └── Login.vue      # 已迁移
└── main.ts             # 全局错误拦截
```