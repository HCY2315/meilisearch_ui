import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'
import { execSync } from 'node:child_process'

function gitCmd(command: string): string {
  try {
    return execSync(command, { stdio: ['ignore', 'pipe', 'ignore'] }).toString().trim()
  } catch {
    return ''
  }
}

// 版本号优先取构建环境变量（Docker build arg），否则回退到本地 git，最后回退 dev
const appVersion = process.env.APP_VERSION || gitCmd('git rev-parse --short HEAD') || 'dev'
const appCommitMsg = process.env.APP_COMMIT_MSG || gitCmd('git log -1 --pretty=%s') || ''

export default defineConfig({
  plugins: [vue()],
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
    __APP_COMMIT_MSG__: JSON.stringify(appCommitMsg),
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  server: {
    port: 5173,
    host: true
  }
})
