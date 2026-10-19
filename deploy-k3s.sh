#!/usr/bin/env bash
set -euo pipefail

NAMESPACE="${NAMESPACE:-meili}"
IMAGE="${IMAGE:-meilisearch-ui:latest}"
MANIFESTS_DIR="${MANIFESTS_DIR:-/home/chaoyue/code/contoiner/k3s/meili}"
SKIP_BUILD=0
SKIP_APPLY=0

usage() {
  cat <<'EOF'
Usage: ./deploy-k3s.sh [options]

把 meilisearch-ui 构建并部署到本地 k3s（构建镜像 -> 导入 k3s containerd -> 应用清单 -> 滚动重启）。

Options:
  --skip-build   跳过 docker build，复用本地已有镜像
  --skip-apply   跳过 kubectl apply，仅重建镜像并重启
  -h, --help     显示帮助

环境变量覆盖:
  NAMESPACE       k8s 命名空间        (默认: meili)
  IMAGE           docker 镜像标签     (默认: meilisearch-ui:latest)
  MANIFESTS_DIR   k8s 清单目录        (默认: /home/chaoyue/code/contoiner/k3s/meili)
EOF
}

for arg in "$@"; do
  case "$arg" in
    --skip-build) SKIP_BUILD=1 ;;
    --skip-apply) SKIP_APPLY=1 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "未知参数: $arg" >&2; usage; exit 1 ;;
  esac
done

command -v docker >/dev/null 2>&1 || { echo "未找到 docker" >&2; exit 1; }
command -v kubectl >/dev/null 2>&1 || { echo "未找到 kubectl" >&2; exit 1; }
command -v k3s >/dev/null 2>&1 || { echo "未找到 k3s" >&2; exit 1; }

if ! kubectl get nodes >/dev/null 2>&1; then
  echo "无法连接 k3s 集群，请检查 KUBECONFIG（当前: ${KUBECONFIG:-未设置}）" >&2
  exit 1
fi

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  APP_VERSION="$(git rev-parse --short HEAD)"
  APP_COMMIT_MSG="$(git log -1 --pretty=%s)"
else
  APP_VERSION="dev"
  APP_COMMIT_MSG=""
fi

echo "==> 部署 ${IMAGE} 到 k3s 命名空间 '${NAMESPACE}'"
echo "==> 版本: ${APP_VERSION} (${APP_COMMIT_MSG})"

if [ "$SKIP_BUILD" -eq 0 ]; then
  echo "==> [1/4] docker build"
  docker build \
    --build-arg APP_VERSION="${APP_VERSION}" \
    --build-arg APP_COMMIT_MSG="${APP_COMMIT_MSG}" \
    -t "${IMAGE}" .
else
  echo "==> [1/4] 跳过 build (--skip-build)"
fi

echo "==> [2/4] 导入镜像到 k3s containerd"
TMP_TAR="$(mktemp /tmp/meilisearch-ui-XXXXXX.tar)"
trap 'rm -f "${TMP_TAR}"' EXIT
docker save -o "${TMP_TAR}" "${IMAGE}"
sudo k3s ctr images import "${TMP_TAR}"

if [ "$SKIP_APPLY" -eq 0 ]; then
  echo "==> [3/4] 应用清单: ${MANIFESTS_DIR}"
  kubectl apply -f "${MANIFESTS_DIR}"
else
  echo "==> [3/4] 跳过 apply (--skip-apply)"
fi

echo "==> [4/4] 滚动重启 meilisearch-ui"
kubectl -n "${NAMESPACE}" rollout restart deploy/meilisearch-ui
kubectl -n "${NAMESPACE}" rollout status deploy/meilisearch-ui --timeout=180s

NODE_IP="$(hostname -I 2>/dev/null | awk '{print $1}')"
echo "==> 完成。访问: http://${NODE_IP:-<node-ip>}:30701"
