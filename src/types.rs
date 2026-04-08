use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct QueryRow {
    pub id: u64,
    pub field: String,
    pub operator: String,
    pub value: String,
    pub logic: String,
}

impl QueryRow {
    pub fn new(id: u64) -> Self {
        QueryRow {
            id,
            field: String::new(),
            operator: "=".to_string(),
            value: String::new(),
            logic: "AND".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryItem {
    pub query: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceAsset {
    pub id: String,
    pub name: String,
    pub brand: String,
    pub model: String,
    pub purchase_date: Option<String>,
    pub warranty_end: Option<String>,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
    pub protocol: Option<String>,
    pub location: Option<String>,
    pub photo_url: Option<String>,
    pub price: Option<f64>,
    pub purchase_link: Option<String>,
    pub invoice_image: Option<String>,
    pub notes: Option<String>,
}

impl DeviceAsset {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            brand: String::new(),
            model: String::new(),
            purchase_date: None,
            warranty_end: None,
            serial_number: None,
            firmware_version: None,
            protocol: None,
            location: None,
            photo_url: None,
            price: None,
            purchase_link: None,
            invoice_image: None,
            notes: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct IndexListResponse {
    pub results: Vec<IndexItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct IndexItem {
    pub uid: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct IndexStats {
    #[serde(rename = "numberOfDocuments")]
    pub number_of_documents: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct IndexSettings {
    #[serde(rename = "searchableAttributes")]
    pub searchable_attributes: Option<Vec<String>>,
    #[serde(rename = "displayedAttributes")]
    pub displayed_attributes: Option<Vec<String>>,
    #[serde(rename = "filterableAttributes")]
    pub filterable_attributes: Option<Vec<String>>,
    #[serde(rename = "sortableAttributes")]
    pub sortable_attributes: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DocumentsResponse {
    pub results: Vec<Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SearchHit {
    #[serde(rename = "id")]
    pub id: Option<Value>,
    #[serde(rename = "_formatted")]
    pub formatted: Option<HashMap<String, Value>>,
    #[serde(rename = "_rankingScore")]
    pub ranking_score: Option<f64>,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

pub type FacetDistribution = HashMap<String, HashMap<String, u64>>;

#[derive(Clone, Debug, Deserialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    #[serde(rename = "estimatedTotalHits")]
    pub estimated_total_hits: Option<u64>,
    #[serde(rename = "processingTimeMs")]
    pub processing_time_ms: Option<u64>,
    #[serde(rename = "facetDistribution")]
    pub facet_distribution: Option<FacetDistribution>,
}

#[derive(Clone, Debug)]
pub struct IndexInfo {
    pub uid: String,
    pub count: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct PopularItem {
    pub value: String,
    pub count: Option<u64>,
}

#[derive(Clone, Debug)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub kind: ToastType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(rename = "aiWeight")]
    pub ai_weight: u32,
    #[serde(rename = "aiEnabled")]
    pub ai_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct ResizeState {
    pub col: String,
    pub start_x: i32,
    pub start_width: i32,
}

#[derive(Clone, Debug)]
pub struct ViewResizeState {
    pub col_idx: usize,
    pub start_x: i32,
    pub start_width_px: f32,
    pub container_width_px: f32,
    pub base_widths: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct ViewFieldResizeState {
    pub col_idx: usize,
    pub start_x: i32,
    pub start_width_px: f32,
    pub container_width_px: f32,
    pub base_widths: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct FieldConfigItem {
    pub field: String,
    pub searchable: bool,
    pub weight: f64,
    pub label: String,
    pub highlight: bool,
    pub display: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewConfig {
    pub name: String,
    pub columns: Vec<Vec<String>>,
    #[serde(default)]
    pub widths: Vec<u32>,
    #[serde(default)]
    pub label_widths: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct ConnectData {
    pub indexes: Vec<IndexInfo>,
}

#[derive(Clone, Debug)]
pub struct IndexData {
    pub available_fields: Vec<String>,
    pub search_fields: Vec<String>,
    pub highlight_fields: Vec<String>,
    pub display_fields: Vec<String>,
    pub filterable_fields: Vec<String>,
    pub sortable_attributes: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PopularSearchData {
    pub items: Vec<PopularItem>,
}

#[derive(Clone, Debug)]
pub enum Msg {
    SetHost(String),
    SetApiKey(String),
    Connect,
    ConnectFinished(Result<ConnectData, String>),
    SelectIndex(String),
    IndexLoaded(Result<IndexData, String>),
    SetSearchInput(String),
    DebouncedSearch,
    PerformSearch,
    SearchFinished(Result<SearchResponse, String>),
    AddQueryRow,
    RemoveQueryRow(u64),
    UpdateQueryField(u64, String),
    UpdateQueryOperator(u64, String),
    UpdateQueryValue(u64, String),
    UpdateQueryLogic(u64, String),
    ApplyQuery,
    ClearQuery,
    ToggleSearchField(String, bool),
    ToggleFacet(String, String, bool),
    ToggleHighlight(bool),
    ToggleRanking(bool),
    UpdateCropLength(String),
    UpdatePageSize(String),
    GoToPage(u32),
    SortSelect(String),
    OpenFilters(bool),
    ToggleTheme,
    ToggleAiDropdown,
    SetAiEnabled(bool),
    SetAiWeight(u32),
    DocumentClick,
    OpenColumnConfig(bool),
    SaveColumnConfig(Vec<String>),
    DragStart(String),
    DragOver(String),
    DropOn(String),
    DragEnd,
    ToggleTableSort(String),
    StartResize(String, i32, i32),
    ResizeMove(i32),
    EndResize,
    OpenFieldConfig(bool),
    SelectAllSearchable(bool),
    SelectAllHighlight(bool),
    SelectAllDisplay(bool),
    SaveFieldConfig(Vec<FieldConfigItem>),
    SaveFieldConfigResult(Result<(), String>),
    ToggleEditLock,
    SetPrimaryKey(String),
    UpdateCellEdit(String, String, String),
    SaveEdits,
    SaveEditsFinished(Result<(), String>),
    OpenViewConfig(bool),
    SetViewMode(String),
    UpdateViewName(String),
    StartViewFieldDrag(String),
    AddViewColumn,
    RemoveViewColumn,
    RemoveViewColumnAt(usize),
    StartViewColumnDrag(usize),
    DropViewColumnAt(usize),
    StartViewFieldDragInColumn(String, usize),
    ViewFieldDragOver(usize, usize),
    DropViewField(usize),
    DropViewFieldToPool,
    StartViewColumnResize(usize, i32, f32, f32),
    ViewColumnResizeMove(i32),
    EndViewColumnResize,
    StartViewFieldResize(usize, i32, f32, f32),
    ViewFieldResizeMove(i32),
    EndViewFieldResize,
    SaveViewConfig,
    OpenResultModal(Option<String>),
    ShowResultDetail(SearchHit),
    CloseResultModal,
    DismissToast(u64),
    PerformHistorySearch(String),
    ClearHistory,
    SelectPopularField(String),
    PopularSearchesLoaded(Result<PopularSearchData, String>),
    PerformPopularSearch(String),
    ExportCsv,
    ExportCsvProgress(u64, u64),
    ExportCsvFinished(Result<String, String>),
    UpdateMaxResults(String),
    SetNewIndexUid(String),
    SetNewIndexPk(String),
    CreateIndexLocal, // Local creation (UI only, no remote call)
    IndexCreated(Result<(), String>),
    SetImagePreview(bool),
    SetImagePreviewLinksOnly(bool),
    SetImagePreviewSize(u32),
    SetCurrentTab(String),
    OpenAssetModal(bool),
    SetAssetForm(DeviceAsset),
    SaveAsset,
    SaveAssetFinished(Result<DeviceAsset, String>),
    DeleteAsset(String),
    DeleteAssetFinished(Result<String, String>),
    OpenAssetDetail(DeviceAsset),
    CloseAssetDetail,
    ImportAssets(String),
    ExportAssets,
    ExportAssetsFinished(Result<String, String>),
    TriggerFileImport(Option<String>),
    FileImportTriggered(Option<String>),
    OpenUploadModal(bool),
    SetUploadData(String),
    FileInputChanged(String),
    SetUploadFile(String, String),
    SetUploadPreviewData(Vec<serde_json::Value>),
    SetUploadPage(usize),
    BatchImportToMeiliSearch,
    BatchImportFinished(Result<String, String>),
    BatchImportProgress(f32),
}
