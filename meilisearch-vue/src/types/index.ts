export interface QueryRow {
  id: number
  field: string
  operator: string
  value: string
  logic: string
}

export interface HistoryItem {
  query: string
  timestamp: string
}

export interface DeviceAsset {
  id: string
  name: string
  brand: string
  model: string
  purchase_date?: string
  warranty_end?: string
  serial_number?: string
  firmware_version?: string
  protocol?: string
  location?: string
  photo_url?: string
  price?: number
  purchase_link?: string
  invoice_image?: string
  notes?: string
}

export interface IndexListResponse {
  results: IndexItem[]
}

export interface IndexItem {
  uid: string
  isLocked?: boolean
  isUnlocked?: boolean
  displayName?: string
  fieldConfigs?: string
  viewConfigs?: string
  tableConfigs?: string
  nestedFieldConfigs?: string
  canEdit?: boolean
  primaryKey?: string
}

export interface IndexStats {
  numberOfDocuments?: number
}

export interface IndexSettings {
  searchableAttributes?: string[]
  displayedAttributes?: string[]
  filterableAttributes?: string[]
  sortableAttributes?: string[]
  embedding?: Record<string, unknown>
}

export interface DocumentsResponse {
  results: Record<string, unknown>[]
}

export interface SearchHit {
  id?: unknown
  _formatted?: Record<string, unknown>
  _rankingScore?: number
  [key: string]: unknown
}

export type FacetDistribution = Record<string, Record<string, number>>

export interface SearchResponse {
  hits: SearchHit[]
  estimatedTotalHits?: number
  processingTimeMs?: number
  facetDistribution?: FacetDistribution
}

export interface IndexInfo {
  uid: string
  count?: number
  isLocked?: boolean
  isUnlocked?: boolean
  displayName?: string
  fieldConfigs?: string
  viewConfigs?: string
  tableConfigs?: string
  nestedFieldConfigs?: string
  canEdit?: boolean
  drawerFieldOrder?: string
  primaryKey?: string
}

export interface PopularItem {
  value: string
  count?: number
}

export interface PopularSearchData {
  items: PopularItem[]
}

export type ToastType = 'success' | 'error' | 'warning' | 'info'

export interface Toast {
  id: number
  message: string
  kind: ToastType
}

export interface AiConfig {
  aiWeight: number
  aiEnabled: boolean
}

export interface ResizeState {
  col: string
  startX: number
  startWidth: number
}

export interface ViewResizeState {
  colIdx: number
  startX: number
  startWidthPx: number
  containerWidthPx: number
  baseWidths: number[]
}

export interface ViewFieldResizeState {
  colIdx: number
  startX: number
  startWidthPx: number
  containerWidthPx: number
  baseWidths: number[]
}

export interface FieldConfigItem {
  field: string
  searchable: boolean
  weight: number
  label: string
  highlight: boolean
  display: boolean
}

export interface NestedFieldConfigItem {
  visible: boolean
  alias: string
}

export type NestedFieldConfigsMap = Record<string, Record<string, NestedFieldConfigItem>>

export interface ViewConfig {
  name: string
  columns: string[][]
  widths: number[]
  labelWidths: number[]
}

export interface ConnectData {
  indexes: IndexInfo[]
}

export interface IndexData {
  availableFields: string[]
  searchFields: string[]
  highlightFields: string[]
  displayFields: string[]
  filterableFields: string[]
  sortableAttributes: string[]
  embedder: string | null
}

export interface CellValue {
  title: string
  html: string
}

export type Theme = 'dark' | 'light'

export type CurrentTab = 'search' | 'assets' | 'admin'
