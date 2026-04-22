import { useAppStore } from './useApp'

export function useImportStore() {
  const app = useAppStore()
  
  return {
    uploadData: app.uploadData,
    uploadFileName: app.uploadFileName,
    uploadPreviewData: app.uploadPreviewData,
    uploadPreviewPage: app.uploadPreviewPage,
    uploadPreviewPageSize: app.uploadPreviewPageSize,
    uploadProgress: app.uploadProgress,
    uploadLoading: app.uploadLoading,
    importMode: app.importMode,
    jsonTextInput: app.jsonTextInput,
    exportDownloading: app.exportDownloading,
    exportProgress: app.exportProgress,
    exportTotal: app.exportTotal,
    batchImport: () => app.batchImport(),
    exportCsv: () => app.exportCsv(),
    parseUploadData: (data: string) => app.parseUploadData(data),
  }
}

export type ImportStoreState = ReturnType<typeof useImportStore>
