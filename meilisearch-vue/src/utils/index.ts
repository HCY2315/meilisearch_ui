import type { SearchHit, CellValue, ToastType, Toast, ViewConfig } from '@/types'

export function formatNumber(n: number): string {
  return n.toLocaleString('zh-CN')
}

/**
 * 生成 UUID (v4)
 * 处理在非安全上下文（HTTP）下 crypto.randomUUID 不可用的情况
 */
export function generateUUID(): string {
  // 1. 优先尝试原生 randomUUID
  if (typeof window !== 'undefined' && window.crypto && typeof window.crypto.randomUUID === 'function') {
    return window.crypto.randomUUID();
  }

  // 2. 降级：使用 getRandomValues (浏览器兼容性更好，但在非安全上下文也可能受限)
  const cryptoObj = typeof window !== 'undefined' ? (window.crypto || (window as any).msCrypto) : null;
  if (cryptoObj && cryptoObj.getRandomValues) {
    try {
      return (([1e7] as any) + -1e3 + -4e3 + -8e3 + -1e11).replace(/[018]/g, (c: any) =>
        (c ^ cryptoObj.getRandomValues(new Uint8Array(1))[0] & 15 >> c / 4).toString(16)
      );
    } catch (e) {
      console.warn('Crypto getRandomValues fallback failed, using Math.random', e);
    }
  }

  // 3. 最终降级：使用 Math.random (非加密安全，但能保证在任何环境下不报错)
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

export function formatTime(isoString: string): string {
  try {
    const d = new Date(isoString)
    const pad = (v: number) => String(v).padStart(2, '0')
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
  } catch {
    return isoString
  }
}

export function getIdString(hit: SearchHit): string {
  return String(hit.id ?? hit['_id'] ?? hit['Id'] ?? '')
}

export function getDocKey(hit: SearchHit, primaryKey: string): string {
  if (primaryKey && hit[primaryKey] !== undefined) {
    return String(hit[primaryKey])
  }
  if (hit.id !== undefined) return String(hit.id)
  return ''
}

export function valueToString(val: unknown): string {
  if (val === null || val === undefined) return ''
  if (typeof val === 'string') return val
  if (typeof val === 'number') return String(val)
  if (typeof val === 'boolean') return String(val)
  if (Array.isArray(val)) {
    return val.map(v => valueToString(v)).join(', ')
  }
  return JSON.stringify(val)
}

export function valueToStringForEdit(val: unknown): string {
  if (val === null) return 'null'
  if (val === undefined) return ''
  if (typeof val === 'string') return val
  return JSON.stringify(val)
}

export function hitEntries(hit: SearchHit): [string, unknown][] {
  return Object.entries(hit).filter(([k]) => k !== 'id' && k !== '_formatted' && k !== '_rankingScore')
}

export function hitHasId(hit: SearchHit): boolean {
  return hit.id !== undefined || hit['_id'] !== undefined || hit['Id'] !== undefined
}

function isImageUrl(url: string): boolean {
  return /\.(jpg|jpeg|png|gif|webp|svg|bmp)(\?.*)?$/i.test(url)
}

function isHttpUrl(s: string): boolean {
  return s.startsWith('http://') || s.startsWith('https://')
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}

function stripHtml(s: string): string {
  return s.replace(/<[^>]+>/g, '')
}

export function getCellValue(
  hit: SearchHit,
  field: string,
  highlightEnabled: boolean,
  imagePreviewEnabled: boolean,
  imagePreviewLinksOnly: boolean,
  imagePreviewSize: number
): CellValue | null {
  const formatted = hit._formatted
  let raw: unknown

  if (formatted && formatted[field] !== undefined) {
    raw = formatted[field]
  } else {
    raw = hit[field]
  }

  if (raw === null || raw === undefined) {
    return null
  }

  // 图片预览逻辑（与原始 Rust 代码行为一致）
  if (imagePreviewEnabled) {
    if (imagePreviewLinksOnly) {
      // 缩略图模式：渲染图片缩略图
      const result = tryRenderImageCell(raw, imagePreviewSize)
      if (result) return result
    } else {
      // 链接模式：渲染可点击链接
      const result = renderImageLinksCell(raw)
      if (result) return result
    }
  } else {
    // 未启用预览：仅当看起来像图片 URL 时渲染
    if (typeof raw === 'string' && isImageUrl(raw)) {
      const esc = escapeHtml(raw)
      return {
        title: esc,
        html: `<img src="${esc}" class="cell-thumbnail" alt="image" loading="lazy" />`,
      }
    }
  }

  // 普通渲染
  let displayHtml: string
  let titleText: string

  if (Array.isArray(raw)) {
    const items: string[] = []
    for (const v of raw) {
      const s = valueToString(v)
      if (s) items.push(s)
    }
    displayHtml = items.join(', ')
    titleText = displayHtml
  } else if (typeof raw === 'object') {
    const json = JSON.stringify(raw)
    displayHtml = escapeHtml(json)
    titleText = json
  } else {
    const text = typeof raw === 'string' ? raw : String(raw)
    if (highlightEnabled && formatted && formatted[field] !== undefined) {
      displayHtml = text
      titleText = stripHtml(text)
    } else {
      displayHtml = escapeHtml(text)
      titleText = text
    }
  }

  return { title: titleText, html: displayHtml }
}

// 渲染图片缩略图（缩略图模式）
function tryRenderImageCell(raw: unknown, size: number): CellValue | null {
  const sizePx = `${size}px`

  function renderOne(url: string) {
    const esc = escapeHtml(url)
    const display = url.split('/').pop() ?? url
    return `
      <div class="thumb-container" style="display:inline-flex; flex-direction:column; align-items:center; margin-right:8px; vertical-align:top;">
        <img class="cell-thumbnail" src="${esc}" alt="thumbnail" loading="lazy" style="width:${sizePx};height:${sizePx}; object-fit:cover; border-radius:4px; margin-bottom:4px;" />
        <a href="${esc}" target="_blank" rel="noopener noreferrer" style="font-size:11px; max-width:${sizePx}; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color:var(--primary-color); text-decoration:none;">${escapeHtml(display)}</a>
      </div>
    `
  }

  if (Array.isArray(raw)) {
    const urls: string[] = []
    for (const v of raw) {
      if (typeof v === 'string' && v.trim() && isHttpUrl(v)) urls.push(v)
    }
    if (!urls.length) return null

    const imgs = urls.slice(0, 5).map(url => renderOne(url)).join('')
    return { title: urls.join(', '), html: `<div class="cell-thumbnail-scroll" style="display:flex; overflow-x:auto; padding-bottom:4px;">${imgs}</div>` }
  }

  if (typeof raw === 'string') {
    if (!raw.trim() || !isHttpUrl(raw)) return null
    return {
      title: raw,
      html: renderOne(raw),
    }
  }

  return null
}

// 渲染可点击链接（链接模式）
function renderImageLinksCell(raw: unknown): CellValue | null {
  function extractFilename(url: string): string | null {
    if (url.startsWith('http://') || url.startsWith('https://')) {
      return url.split('/').pop() ?? url
    }
    return null
  }

  if (Array.isArray(raw)) {
    const urls: string[] = []
    for (const v of raw) {
      if (typeof v === 'string' && v.trim()) urls.push(v)
    }
    if (!urls.length) return null

    const links = urls.slice(0, 3).map(url => {
      const display = extractFilename(url)
      const esc = escapeHtml(url)
      if (display) return `<a href="${esc}" target="_blank" rel="noopener noreferrer">${escapeHtml(display)}</a>`
      return `<span>${esc}</span>`
    }).join('')
    return { title: urls.join(', '), html: `<div class="cell-thumbnail-scroll">${links}</div>` }
  }

  if (typeof raw === 'string') {
    if (!raw.trim()) return null
    const display = extractFilename(raw)
    const esc = escapeHtml(raw)
    if (display) {
      return { title: esc, html: `<a href="${esc}" target="_blank" rel="noopener noreferrer">${escapeHtml(display)}</a>` }
    }
    return { title: esc, html: `<span>${esc}</span>` }
  }

  return null
}

export function sortHits(hits: SearchHit[], field: string, dir: string): SearchHit[] {
  return [...hits].sort((a, b) => {
    const va = a[field]
    const vb = b[field]
    if (va === undefined || va === null) return 1
    if (vb === undefined || vb === null) return -1
    let cmp = 0
    if (typeof va === 'number' && typeof vb === 'number') {
      cmp = va - vb
    } else {
      cmp = String(va).localeCompare(String(vb))
    }
    return dir === 'desc' ? -cmp : cmp
  })
}

export function buildFilterExpression(rows: { field: string; operator: string; value: string; logic: string }[]): string | null {
  const parts: string[] = []
  for (const row of rows) {
    if (!row.field || !row.value.trim()) continue
    const val = row.value.trim()
    const escaped = val.replace(/"/g, '\\"')
    switch (row.operator) {
      case '=': parts.push(`${row.field} = "${escaped}"`); break
      case '!=': parts.push(`${row.field} != "${escaped}"`); break
      case '>': parts.push(`${row.field} > ${val}`); break
      case '>=': parts.push(`${row.field} >= ${val}`); break
      case '<': parts.push(`${row.field} < ${val}`); break
      case '<=': parts.push(`${row.field} <= ${val}`); break
      case 'contains': parts.push(`${row.field} CONTAINS "${escaped}"`); break
      case 'exists': parts.push(`${row.field} EXISTS`); break
      default: parts.push(`${row.field} = "${escaped}"`)
    }
  }

  if (!parts.length) return null

  let result = parts[0]
  for (let i = 1; i < parts.length; i++) {
    const logic = rows[i]?.logic ?? 'AND'
    result += ` ${logic} ${parts[i]}`
  }
  return result
}

export function makeToast(message: string, kind: ToastType, seq: { value: number }): Toast {
  const id = seq.value++
  return { id, message, kind }
}

export function normalizeViewWidths(widths: number[], len: number): number[] {
  const arr = [...widths]
  while (arr.length < len) arr.push(0)
  if (arr.length > len) arr.length = len

  if (arr.every(w => w === 0)) {
    const even = Math.floor(100 / len)
    const result = Array(len).fill(even)
    let remain = 100 - even * len
    let i = 0
    while (remain > 0) {
      result[i++]++
      if (i >= len) i = 0
      remain--
    }
    return result
  }

  const sum = arr.reduce((a, b) => a + b, 0)
  if (sum === 0) return arr

  const normalized: number[] = []
  let used = 0
  for (let i = 0; i < arr.length; i++) {
    const v = Math.round((arr[i] / sum) * 100)
    normalized.push(v)
    used += v
  }
  let remain = 100 - used
  let idx = 0
  while (remain > 0) {
    normalized[idx]++
    remain--
    idx = (idx + 1) % normalized.length
  }
  return normalized
}

export function normalizeLabelWidths(widths: number[], len: number): number[] {
  const arr = [...widths]
  while (arr.length < len) arr.push(140)
  if (arr.length > len) arr.length = len
  return arr.map(w => Math.max(w, 60))
}

export function collectColumnLabels(): Record<string, string> {
  const labels: Record<string, string> = {}
  const inputs = document.querySelectorAll<HTMLInputElement>('.column-label-input[data-col]')
  inputs.forEach(input => {
    const col = input.getAttribute('data-col')
    if (col) labels[col] = input.value
  })
  return labels
}

export function setAllCheckboxes(selector: string, checked: boolean): void {
  document.querySelectorAll<HTMLInputElement>(selector).forEach(el => {
    el.checked = checked
  })
}

export function parseSortValue(value: string): { field: string; dir: string } {
  const parts = value.split(':')
  return { field: parts[0], dir: parts[1] ?? 'asc' }
}

export function debounce<T extends (...args: unknown[]) => void>(fn: T, ms: number): (...args: Parameters<T>) => void {
  let timer: ReturnType<typeof setTimeout> | null = null
  return (...args: Parameters<T>) => {
    if (timer) clearTimeout(timer)
    timer = setTimeout(() => fn(...args), ms)
  }
}
