# 本地开发快速指南

本地开发目标是快速迭代 UI，直接对接 MeiliSearch 的 HTTP API。前端使用 Rust + WASM（Yew）通过 trunk 构建运行在浏览器中。

1. 依赖环境
- Rust toolchain（推荐使用 rustup 安装）
- Trunk 构建工具
- MeiliSearch 服务（本地或远端均可），并确保 CORS 配置允许前端域访问
- 可选：nginx 用于静态资源托管（如需在现有站点下部署）

2. 安装与准备
- 安装 Trunk
  ```bash
  cargo install trunk
  ```
- 启动 MeiliSearch（示例命令，按你的部署方式调整）
  # 如需开启 CORS，请参考下方 CORS 配置要点
- 本地浏览器环境：确保前端页面能访问 MeiliSearch 的 API（CORS 已正确配置）

3. 快速启动本地开发
- 运行前端
  ```bash
  trunk serve
  ```
  默认访问地址： http://localhost:8080

- MeiliSearch CORS 配置要点
  - 为前端域开启 CORS。常见方式是在 MeiliSearch 服务器端启用 CORS，并允许来自前端的 origin。
  - 常用环境变量（依据版本可能略有差异）：
    * MEILI_CORS_ALLOW_ORIGIN="http://localhost:8080"
    * MEILI_CORS_ENABLE=true
  - 如果使用容器部署，请在容器运行命令中传入上述变量，或在部署配置中设置。
  - 测试跨域是否生效：在浏览器中发起对 MeiliSearch 的请求并观察响应头中的 Access-Control-Allow-Origin。

4. nginx 部署静态资源的示例
若你将前端静态资源放在 nginx 的子路径下（例如 /meilisearch/），请参照如下简易配置：
```nginx
server {
  listen 80;
  server_name your-domain.com;

  root /home/chaoyue/code/contoiner/nginx/html/meilisearch;
  index index.html;
  location / {
    try_files $uri $uri/ /index.html;
  }
}
```
说明：dist 输出路径要与 nginx 的 root 路径一致，Trunk.toml 中的 dist、public_url 配置需对应 nginx 的部署结构（public_url = "/meilisearch/"）。如果需要将 API 请求代理到 MeiliSearch，请在 nginx 增加 proxy_pass 配置，并确保 CORS 设置生效。

5. 验证步骤
- 访问 http://localhost:8080/meilisearch/ （前提是你将 public_url 设置为 /meilisearch/，并将 dist 放在 nginx 指定目录）
- 在 UI 中配置 MeiliSearch 主机地址与 API Key，执行一次搜索，检查返回结果与 UI 展示是否正常。

6. 常见问题排查
- Trunk 未安装：确保 cargo 已安装，并执行 cargo install trunk。
- CORS 未生效：检查 MeiliSearch 的启动参数/环境变量，确保前端域在允许列表中；必要时在 nginx/反向代理层添加 CX 相关头部。
- 静态资源加载路径错误：确认 Trunk 的 dist 输出路径与 nginx 静态资源目录匹配，public_url 配置与 nginx 路径一致。
