export { useConnectionStore } from './useConnectionStore'
export { useSearchStore } from './useSearchStore'
export { useUIStore } from './useUIStore'
export { useImportStore } from './useImportStore'
export { useAssetStore } from './useAssetStore'

export { useAppStore } from './useApp'

import { useAppStore as _useAppStore } from './useApp'
import { useConnectionStore as _useConnectionStore } from './useConnectionStore'
import { useSearchStore as _useSearchStore } from './useSearchStore'
import { useUIStore as _useUIStore } from './useUIStore'
import { useImportStore as _useImportStore } from './useImportStore'
import { useAssetStore as _useAssetStore } from './useAssetStore'

export function useStores() {
  return {
    app: _useAppStore(),
    connection: _useConnectionStore(),
    search: _useSearchStore(),
    ui: _useUIStore(),
    importStore: _useImportStore(),
    asset: _useAssetStore(),
  }
}
