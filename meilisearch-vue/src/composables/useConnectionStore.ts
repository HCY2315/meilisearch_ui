import { useAppStore } from './useApp'

export function useConnectionStore() {
  const app = useAppStore()
  
  return {
    hostInput: app.hostInput,
    apiKeyInput: app.apiKeyInput,
    indexes: app.indexes,
    currentIndex: app.currentIndex,
    connect: () => app.connect(),
    selectIndex: (uid: string) => app.selectIndex(uid),
    loadIndexData: (uid: string) => app.selectIndex(uid),
  }
}

export type ConnectionStoreState = ReturnType<typeof useConnectionStore>
