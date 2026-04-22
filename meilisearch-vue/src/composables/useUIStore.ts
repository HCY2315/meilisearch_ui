import { useAppStore } from './useApp'

export function useUIStore() {
  const app = useAppStore()
  
  return {
    // View state
    viewMode: app.viewMode,
    viewNameInput: app.viewNameInput,
    viewConfigs: app.viewConfigs,
    viewLayoutWorking: app.viewLayoutWorking,
    viewWidthsWorking: app.viewWidthsWorking,
    viewLabelWidthsWorking: app.viewLabelWidthsWorking,
    // Column state
    columnOrder: app.columnOrder,
    hiddenColumns: app.hiddenColumns,
    columnWidths: app.columnWidths,
    tableSortField: app.tableSortField,
    tableSortDir: app.tableSortDir,
    // UI state
    toasts: app.toasts,
    loading: app.loading,
    filtersDrawerOpen: app.filtersDrawerOpen,
    // Modal states
    viewModalOpen: app.viewModalOpen,
    advancedSettingsOpen: app.advancedSettingsOpen,
    columnConfigOpen: app.columnConfigOpen.value,
    fieldConfigOpen: app.fieldConfigOpen,
    resultModalOpen: app.resultModalOpen,
    assetModalOpen: app.assetModalOpen,
    uploadModalOpen: app.uploadModalOpen,
    newIndexUid: app.newIndexUid,
    newIndexPk: app.newIndexPk,
    // Actions
    pushToast: (message: string, kind: 'success' | 'error' | 'warning' | 'info') => app.pushToast(message, kind),
    setViewMode: (v: string) => { app.viewMode = v },
    saveViewConfig: () => app.saveViewConfig(),
    openViewConfig: () => app.openViewConfig(),
    setColumnWidth: (col: string, w: number) => app.setColumnWidth(col, w),
    moveColumnOrder: (from: string, to: string) => app.moveColumnOrder(from, to),
    saveColumnWidthPrefs: () => app.saveColumnWidthPrefs(),
    loadColumnWidthPrefs: () => app.loadColumnWidthPrefs(),
  }
}

export type UIStoreState = ReturnType<typeof useUIStore>
