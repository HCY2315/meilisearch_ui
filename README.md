# meilisearch_ui

Yew + WASM 前端，直连 MeiliSearch HTTP API。

## 开发

1. 安装 Trunk

```bash
cargo install trunk
```

2. 启动

```bash
trunk serve
```

默认地址：`http://localhost:8080`

## 说明

- 纯前端 WASM，需确保 MeiliSearch 允许 CORS。
- `Trunk.toml` 提供了构建与本地服务配置。
- 
- 快速本地开发文档：请参阅 docs/LOCAL_DEV.md 获取完整步骤、MeiliSearch CORS 配置要点以及 nginx 部署示例。
