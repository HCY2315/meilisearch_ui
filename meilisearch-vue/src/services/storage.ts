import type { HistoryItem, AiConfig, DeviceAsset, ToastType, Theme } from '@/types'

function storageGet(key: string): string | null {
  return localStorage.getItem(key)
}

function storageSet(key: string, value: string): void {
  localStorage.setItem(key, value)
}

function storageRemove(key: string): void {
  localStorage.removeItem(key)
}

export function loadSearchHistory(): HistoryItem[] {
  const raw = storageGet('searchHistory')
  if (!raw) return []
  try {
    return JSON.parse(raw)
  } catch {
    return []
  }
}

export function saveSearchHistory(history: HistoryItem[]): void {
  storageSet('searchHistory', JSON.stringify(history))
}

export function loadFieldLabels(indexName?: string | null): Record<string, string> {
  const key = `fieldLabels:${indexName ?? 'default'}`
  const raw = storageGet(key)
  if (!raw) return {}
  try {
    return JSON.parse(raw)
  } catch {
    return {}
  }
}

export function saveFieldLabels(labels: Record<string, string>, indexName?: string | null): void {
  const key = `fieldLabels:${indexName ?? 'default'}`
  storageSet(key, JSON.stringify(labels))
}

export function loadAiConfig(): AiConfig {
  const raw = storageGet('aiConfig')
  if (!raw) return { aiWeight: 80, aiEnabled: false }
  try {
    return JSON.parse(raw)
  } catch {
    return { aiWeight: 80, aiEnabled: false }
  }
}

export function saveAiConfig(config: AiConfig): void {
  storageSet('aiConfig', JSON.stringify(config))
}

export function loadDeviceAssets(): DeviceAsset[] {
  const raw = storageGet('deviceAssets')
  if (!raw) return []
  try {
    return JSON.parse(raw)
  } catch {
    return []
  }
}

export function saveDeviceAssets(assets: DeviceAsset[]): void {
  storageSet('deviceAssets', JSON.stringify(assets))
}

export function loadPopularField(indexName?: string | null): string {
  const key = `popularField:${indexName ?? 'default'}`
  return storageGet(key) ?? ''
}

export function savePopularField(field: string, indexName?: string | null): void {
  const key = `popularField:${indexName ?? 'default'}`
  storageSet(key, field)
}

export function getTheme(): Theme {
  return (storageGet('theme') as Theme) || 'dark'
}

export function setTheme(theme: Theme): void {
  storageSet('theme', theme)
  document.documentElement.setAttribute('data-theme', theme)
}

export function loadImagePreviewEnabled(): boolean {
  return storageGet('imagePreviewEnabled') === 'true'
}

export function saveImagePreviewEnabled(enabled: boolean): void {
  storageSet('imagePreviewEnabled', String(enabled))
}

export function loadImagePreviewLinksOnly(): boolean {
  return storageGet('imagePreviewLinksOnly') === 'true'
}

export function saveImagePreviewLinksOnly(linksOnly: boolean): void {
  storageSet('imagePreviewLinksOnly', String(linksOnly))
}

export function loadImagePreviewSize(): number {
  const raw = storageGet('imagePreviewSize')
  const size = Number(raw)
  return isNaN(size) ? 80 : size
}

export function saveImagePreviewSize(size: number): void {
  storageSet('imagePreviewSize', String(size))
}
