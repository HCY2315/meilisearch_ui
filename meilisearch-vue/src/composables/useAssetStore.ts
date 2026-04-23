import { useAppStore } from './useApp'

export function useAssetStore() {
  const app = useAppStore()
  
  return {
    assetForm: app.assetForm,
    assetList: app.assetList,
    assetModalOpen: app.assetModalOpen,
    assetDetail: app.assetDetail,
    assetsLoading: app.assetsLoading,
    uploadModalOpen: app.uploadModalOpen,
    saveAsset: () => app.saveAsset(),
    deleteAsset: (id: string) => app.deleteAsset(id),
    setCurrentTab: (tab: string) => app.setCurrentTab(tab as any),
  }
}

export type AssetStoreState = ReturnType<typeof useAssetStore>
