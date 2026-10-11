#!/usr/bin/env bash
set -e

# 版本号取当前 git commit 短哈希与提交信息（编译进前端，显示在后台侧边栏）
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  export APP_VERSION="$(git rev-parse --short HEAD)"
  export APP_COMMIT_MSG="$(git log -1 --pretty=%s)"
else
  export APP_VERSION="dev"
  export APP_COMMIT_MSG=""
fi

echo "==> Deploy version: ${APP_VERSION} (${APP_COMMIT_MSG})"

sudo docker image prune -f
sudo docker compose up -d --build
