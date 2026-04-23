import { useAppStore } from './useApp'

export function useSearchStore() {
  const app = useAppStore()
  
  return {
    searchInput: app.searchInput,
    queryRows: app.queryRows,
    facets: app.facets,
    lastHits: app.lastHits,
    lastResults: app.lastResults,
    searchHistory: app.searchHistory,
    fieldLabels: app.fieldLabels,
    popularSearches: app.popularSearches,
    popularSearchField: app.popularSearchField,
    currentPage: app.currentPage,
    pageSize: app.pageSize,
    resultsCount: app.resultsCount,
    processingTimeMs: app.processingTimeMs,
    facetDistribution: app.facetDistribution,
    sortableAttributes: app.sortableAttributes,
    visibleAvailableFields: app.visibleAvailableFields,
    visibleFilterableFields: app.visibleFilterableFields,
    visibleFacetDistribution: app.visibleFacetDistribution,
    totalPages: app.totalPages,
    performSearch: () => app.performSearch(),
    buildSearchParams: () => {
      const limit = Math.min(app.pageSize, app.maxResultsPerPage)
      const params: Record<string, unknown> = {
        limit,
        offset: Math.max(app.currentPage - 1, 0) * limit,
        matchingStrategy: 'last',
      }
      if (app.highlightEnabled) {
        const targets = app.highlightFields.length ? app.highlightFields : app.searchFields.length ? app.searchFields : ['*']
        params.attributesToHighlight = targets
        params.highlightPreTag = '<em class="highlight">'
        params.highlightPostTag = '</em>'
      }
      if (app.cropLength > 0 && app.highlightEnabled) {
        const targets = app.highlightFields.length ? app.highlightFields : app.searchFields.length ? app.searchFields : ['*']
        params.attributesToCrop = targets
        params.cropLength = app.cropLength
      }
      if (app.showRankingScore) {
        params.showRankingScore = true
        params.showRankingScoreDetails = true
      }
      if (app.aiConfig.aiEnabled && app.aiConfig.aiWeight > 0) {
        const ratio = Math.min(app.aiConfig.aiWeight, 100) / 100
        const hybrid: Record<string, unknown> = { semanticRatio: ratio }
        params.hybrid = hybrid
      }
      if (app.sortValue) params.sort = [app.sortValue]
      
      const filterParts: string[] = []
      for (const row of app.queryRows) {
        if (!row.field || !row.value.trim()) continue
        const val = row.value.trim().replace(/"/g, '\\"')
        let part = ''
        switch (row.operator) {
          case '=': part = `${row.field} = "${val}"`; break
          case '!=': part = `${row.field} != "${val}"`; break
          case '>': part = `${row.field} > ${row.value.trim()}`; break
          case '>=': part = `${row.field} >= ${row.value.trim()}`; break
          case '<': part = `${row.field} < ${row.value.trim()}`; break
          case '<=': part = `${row.field} <= ${row.value.trim()}`; break
          case 'contains': part = `${row.field} CONTAINS "${val}"`; break
          case 'exists': part = `${row.field} EXISTS`; break
          default: part = `${row.field} = "${val}"`
        }
        filterParts.push(part)
      }
      if (filterParts.length) params.filter = filterParts.join(' AND ')
      
      if (app.displayFields.length) {
        const attrs = [...app.displayFields]
        if (!attrs.includes('id')) attrs.push('id')
        if (app.primaryKeyField && !attrs.includes(app.primaryKeyField)) attrs.push(app.primaryKeyField)
        params.attributesToRetrieve = attrs
      } else {
        params.attributesToRetrieve = ['*']
      }
      if (app.filterableFields.length) params.facets = app.filterableFields
      
      return params
    },
    addToHistory: (query: string) => app.addToHistory(query),
    clearHistory: () => app.clearHistory(),
    loadPopularSearches: () => app.loadPopularSearches(),
    scheduleDebouncedSearch: () => app.scheduleDebouncedSearch(),
    addQueryRow: () => app.addQueryRow(),
    removeQueryRow: (id: number) => app.removeQueryRow(id),
    clearQuery: () => app.clearQuery(),
    toggleFacet: (field: string, value: string, checked: boolean) => app.toggleFacet(field, value, checked),
    goToPage: (page: number) => app.goToPage(page),
  }
}

export type SearchStoreState = ReturnType<typeof useSearchStore>
