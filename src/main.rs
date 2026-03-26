use gloo_events::EventListener;
use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, Element, HtmlElement, Window};
use yew::events::{DragEvent, MouseEvent};
use yew::{html, AttrValue, Callback, Component, Context, Html, TargetCast};

#[derive(Clone, Debug, Deserialize)]
struct IndexListResponse {
    results: Vec<IndexItem>,
}

#[derive(Clone, Debug, Deserialize)]
struct IndexItem {
    uid: String,
}

#[derive(Clone, Debug, Deserialize)]
struct IndexStats {
    #[serde(rename = "numberOfDocuments")]
    number_of_documents: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
struct IndexSettings {
    #[serde(rename = "searchableAttributes")]
    searchable_attributes: Option<Vec<String>>,
    #[serde(rename = "displayedAttributes")]
    displayed_attributes: Option<Vec<String>>,
    #[serde(rename = "filterableAttributes")]
    filterable_attributes: Option<Vec<String>>,
    #[serde(rename = "sortableAttributes")]
    sortable_attributes: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
struct DocumentsResponse {
    results: Vec<Value>,
}

#[derive(Clone, Debug, Deserialize)]
struct SearchHit {
    #[serde(rename = "id")]
    id: Option<Value>,
    #[serde(rename = "_formatted")]
    formatted: Option<HashMap<String, Value>>,
    #[serde(rename = "_rankingScore")]
    ranking_score: Option<f64>,
    #[serde(flatten)]
    fields: HashMap<String, Value>,
}

type FacetDistribution = HashMap<String, HashMap<String, u64>>;

#[derive(Clone, Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<SearchHit>,
    #[serde(rename = "estimatedTotalHits")]
    estimated_total_hits: Option<u64>,
    #[serde(rename = "processingTimeMs")]
    processing_time_ms: Option<u64>,
    #[serde(rename = "facetDistribution")]
    facet_distribution: Option<FacetDistribution>,
}

#[derive(Clone, Debug)]
struct IndexInfo {
    uid: String,
    count: Option<u64>,
}

#[derive(Clone, Debug)]
struct QueryRow {
    id: u64,
    field: String,
    operator: String,
    value: String,
    logic: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct HistoryItem {
    query: String,
    timestamp: String,
}

#[derive(Clone, Debug)]
struct PopularItem {
    value: String,
    count: Option<u64>,
}

#[derive(Clone, Debug)]
enum ToastType {
    Success,
    Error,
    Warning,
}

#[derive(Clone, Debug)]
struct Toast {
    id: u64,
    message: String,
    kind: ToastType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AiConfig {
    #[serde(rename = "aiWeight")]
    ai_weight: u32,
    #[serde(rename = "aiEnabled")]
    ai_enabled: bool,
}

#[derive(Clone, Debug)]
struct ResizeState {
    col: String,
    start_x: i32,
    start_width: i32,
}

#[derive(Clone, Debug)]
struct ViewResizeState {
    col_idx: usize,
    start_x: i32,
    start_width_px: f32,
    container_width_px: f32,
    base_widths: Vec<u32>,
}

#[derive(Clone, Debug)]
struct ViewFieldResizeState {
    col_idx: usize,
    start_x: i32,
    start_width_px: f32,
    container_width_px: f32,
    base_widths: Vec<u32>,
}

#[derive(Clone, Debug)]
struct FieldConfigItem {
    field: String,
    searchable: bool,
    weight: f64,
    label: String,
    highlight: bool,
    display: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ViewConfig {
    name: String,
    columns: Vec<Vec<String>>,
    #[serde(default)]
    widths: Vec<u32>,
    #[serde(default)]
    label_widths: Vec<u32>,
}

#[derive(Clone, Debug)]
struct ConnectData {
    indexes: Vec<IndexInfo>,
}

#[derive(Clone, Debug)]
struct IndexData {
    available_fields: Vec<String>,
    search_fields: Vec<String>,
    highlight_fields: Vec<String>,
    display_fields: Vec<String>,
    filterable_fields: Vec<String>,
    sortable_attributes: Vec<String>,
}

#[derive(Clone, Debug)]
struct PopularSearchData {
    items: Vec<PopularItem>,
}

enum Msg {
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
    DropViewField(usize),
    AddViewColumn,
    RemoveViewColumn,
    RemoveViewColumnAt(usize),
    StartViewColumnDrag(usize),
    DropViewColumnAt(usize),
    StartViewFieldDragInColumn(String, usize),
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
}

struct App {
    host_input: String,
    api_key_input: String,
    indexes: Vec<IndexInfo>,
    current_index: String,
    search_input: String,
    query_rows: Vec<QueryRow>,
    next_query_id: u64,
    search_fields: Vec<String>,
    search_field_weights: HashMap<String, f64>,
    filterable_fields: Vec<String>,
    available_fields: Vec<String>,
    highlight_fields: Vec<String>,
    display_fields: Vec<String>,
    facets: HashMap<String, Vec<String>>,
    search_history: Vec<HistoryItem>,
    field_labels: HashMap<String, String>,
    popular_searches: Vec<PopularItem>,
    popular_search_field: String,
    current_page: u32,
    page_size: u32,
    results_count: u64,
    processing_time_ms: Option<u64>,
    facet_distribution: Option<FacetDistribution>,
    sortable_attributes: Vec<String>,
    last_hits: Vec<SearchHit>,
    last_results: Option<SearchResponse>,
    last_base_columns: Vec<String>,
    table_sort_field: String,
    table_sort_dir: String,
    column_order: Vec<String>,
    hidden_columns: Vec<String>,
    column_widths: HashMap<String, u32>,
    ai_config: AiConfig,
    ai_dropdown_open: bool,
    highlight_enabled: bool,
    show_ranking_score: bool,
    crop_length: u32,
    sort_value: String,
    filters_drawer_open: bool,
    column_config_open: bool,
    field_config_open: bool,
    result_modal_open: bool,
    result_detail: Option<SearchHit>,
    edit_locked: bool,
    primary_key_field: String,
    pending_edits: HashMap<String, HashMap<String, Value>>,
    view_modal_open: bool,
    view_mode: String,
    view_name_input: String,
    view_configs: Vec<ViewConfig>,
    view_layout_working: Vec<Vec<String>>,
    view_widths_working: Vec<u32>,
    view_label_widths_working: Vec<u32>,
    view_drag_field: Option<String>,
    view_drag_from_column: Option<usize>,
    view_drag_column: Option<usize>,
    loading: bool,
    toasts: Vec<Toast>,
    toast_seq: u64,
    debounce: Option<Timeout>,
    _document_listener: Option<EventListener>,
    resize_move_listener: Option<EventListener>,
    resize_up_listener: Option<EventListener>,
    resize_state: Option<ResizeState>,
    dragging_col: Option<String>,
    drag_over_col: Option<String>,
    is_resizing_columns: bool,
    view_resize_state: Option<ViewResizeState>,
    view_resize_move_listener: Option<EventListener>,
    view_resize_up_listener: Option<EventListener>,
    view_field_resize_state: Option<ViewFieldResizeState>,
    view_field_resize_move_listener: Option<EventListener>,
    view_field_resize_up_listener: Option<EventListener>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let document_listener = {
            let link = ctx.link().clone();
            let window = web_window();
            let document = window.document().unwrap();
            Some(EventListener::new(&document, "click", move |_| {
                link.send_message(Msg::DocumentClick);
            }))
        };

        let mut app = Self {
            host_input: "http://192.168.2.27:7700".to_string(),
            api_key_input: "insur132".to_string(),
            indexes: vec![],
            current_index: "".to_string(),
            search_input: "".to_string(),
            query_rows: vec![],
            next_query_id: 1,
            search_fields: vec![],
            search_field_weights: HashMap::new(),
            filterable_fields: vec![],
            available_fields: vec![],
            highlight_fields: vec![],
            display_fields: vec![],
            facets: HashMap::new(),
            search_history: load_search_history(),
            field_labels: load_field_labels(None),
            popular_searches: vec![],
            popular_search_field: "".to_string(),
            current_page: 1,
            page_size: 50,
            results_count: 0,
            processing_time_ms: None,
            facet_distribution: None,
            sortable_attributes: vec![],
            last_hits: vec![],
            last_results: None,
            last_base_columns: vec![],
            table_sort_field: "".to_string(),
            table_sort_dir: "asc".to_string(),
            column_order: vec![],
            hidden_columns: vec![],
            column_widths: HashMap::new(),
            ai_config: load_ai_config(),
            ai_dropdown_open: false,
            highlight_enabled: true,
            show_ranking_score: true,
            crop_length: 1000,
            sort_value: "".to_string(),
            filters_drawer_open: false,
            column_config_open: false,
            field_config_open: false,
            result_modal_open: false,
            result_detail: None,
            edit_locked: true,
            primary_key_field: "id".to_string(),
            pending_edits: HashMap::new(),
            view_modal_open: false,
            view_mode: "table".to_string(),
            view_name_input: "".to_string(),
            view_configs: vec![],
            view_layout_working: vec![vec![], vec![]],
            view_widths_working: vec![0, 0],
            view_label_widths_working: vec![140, 140],
            view_drag_field: None,
            view_drag_from_column: None,
            view_drag_column: None,
            loading: false,
            toasts: vec![],
            toast_seq: 1,
            debounce: None,
            _document_listener: document_listener,
            resize_move_listener: None,
            resize_up_listener: None,
            resize_state: None,
            dragging_col: None,
            drag_over_col: None,
            is_resizing_columns: false,
            view_resize_state: None,
            view_resize_move_listener: None,
            view_resize_up_listener: None,
            view_field_resize_state: None,
            view_field_resize_move_listener: None,
            view_field_resize_up_listener: None,
        };

        app.add_query_row();
        app.initialize_theme();
        app
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetHost(value) => {
                self.host_input = value;
                true
            }
            Msg::SetApiKey(value) => {
                self.api_key_input = value;
                true
            }
            Msg::Connect => {
                let host = self.host_input.trim().to_string();
                if host.is_empty() {
                    self.push_toast("请输入服务器地址".to_string(), ToastType::Error, ctx);
                    return false;
                }
                self.loading = true;
                let api_key = self.api_key_input.trim().to_string();
                let link = ctx.link().clone();
                spawn_local(async move {
                    let res = connect_indexes(&host, &api_key).await;
                    link.send_message(Msg::ConnectFinished(res));
                });
                true
            }
            Msg::ConnectFinished(result) => {
                self.loading = false;
                match result {
                    Ok(data) => {
                        self.indexes = data.indexes;
                        self.push_toast("连接成功！".to_string(), ToastType::Success, ctx);
                        true
                    }
                    Err(err) => {
                        self.push_toast(format!("连接失败: {err}"), ToastType::Error, ctx);
                        true
                    }
                }
            }
            Msg::SelectIndex(value) => {
                self.current_index = value.clone();
                self.reset_index_state();
                if value.is_empty() {
                    self.popular_searches = vec![];
                    return true;
                }
                self.loading = true;
                let host = self.host_input.trim().to_string();
                let api_key = self.api_key_input.trim().to_string();
                let link = ctx.link().clone();
                spawn_local(async move {
                    let res = load_index_data(&host, &api_key, &value).await;
                    link.send_message(Msg::IndexLoaded(res));
                });
                true
            }
            Msg::IndexLoaded(result) => {
                self.loading = false;
                match result {
                    Ok(data) => {
                        self.available_fields = data.available_fields;
                        self.search_fields = data.search_fields;
                        self.highlight_fields = data.highlight_fields;
                        self.display_fields = data.display_fields;
                        self.filterable_fields = data.filterable_fields;
                        self.sortable_attributes = data.sortable_attributes;
                        self.sort_value = "".to_string();
                        self.update_popular_field();
                        self.load_column_prefs();
                        self.load_column_width_prefs();
                        self.load_field_labels();
                        self.load_view_configs();
                        self.refresh_query_rows();
                        self.push_toast(
                            format!("已选择索引: {}", self.current_index),
                            ToastType::Success,
                            ctx,
                        );
                        ctx.link().send_message(Msg::PopularSearchesLoaded(Ok(PopularSearchData { items: vec![] })));
                        ctx.link().send_message(Msg::PerformSearch);
                        true
                    }
                    Err(err) => {
                        self.push_toast(format!("加载索引失败: {err}"), ToastType::Error, ctx);
                        true
                    }
                }
            }
            Msg::SetSearchInput(value) => {
                self.search_input = value;
                self.schedule_debounced_search(ctx, 300);
                true
            }
            Msg::DebouncedSearch => {
                ctx.link().send_message(Msg::PerformSearch);
                false
            }
            Msg::PerformSearch => {
                if self.current_index.is_empty() {
                    return false;
                }
                self.loading = true;
                let host = self.host_input.trim().to_string();
                let api_key = self.api_key_input.trim().to_string();
                let index = self.current_index.clone();
                let query = self.search_input.trim().to_string();
                let params = self.build_search_params();
                let link = ctx.link().clone();
                spawn_local(async move {
                    let res = perform_search(&host, &api_key, &index, &query, params).await;
                    link.send_message(Msg::SearchFinished(res));
                });
                true
            }
            Msg::SearchFinished(result) => {
                self.loading = false;
                match result {
                    Ok(res) => {
                        if !self.search_input.trim().is_empty() {
                            self.add_to_history(self.search_input.trim().to_string());
                        }
                        self.last_hits = res.hits.clone();
                        self.last_results = Some(res.clone());
                        self.apply_results_columns(&res);
                        self.update_stats(&res);
                        self.update_facets_from_response(&res);
                        true
                    }
                    Err(err) => {
                        self.push_toast(format!("搜索失败: {err}"), ToastType::Error, ctx);
                        true
                    }
                }
            }
            Msg::AddQueryRow => {
                self.add_query_row();
                true
            }
            Msg::RemoveQueryRow(id) => {
                self.sync_query_rows_from_dom();
                self.query_rows.retain(|row| row.id != id);
                if self.query_rows.is_empty() {
                    self.add_query_row();
                }
                true
            }
            Msg::UpdateQueryField(id, value) => {
                if let Some(row) = self.query_rows.iter_mut().find(|row| row.id == id) {
                    row.field = value;
                }
                self.current_page = 1;
                self.schedule_debounced_search(ctx, 300);
                true
            }
            Msg::UpdateQueryOperator(id, value) => {
                if let Some(row) = self.query_rows.iter_mut().find(|row| row.id == id) {
                    row.operator = value;
                }
                self.current_page = 1;
                self.schedule_debounced_search(ctx, 300);
                true
            }
            Msg::UpdateQueryValue(id, value) => {
                if let Some(row) = self.query_rows.iter_mut().find(|row| row.id == id) {
                    row.value = value;
                }
                self.current_page = 1;
                self.schedule_debounced_search(ctx, 300);
                true
            }
            Msg::UpdateQueryLogic(id, value) => {
                if let Some(row) = self.query_rows.iter_mut().find(|row| row.id == id) {
                    row.logic = value;
                }
                self.current_page = 1;
                self.schedule_debounced_search(ctx, 300);
                true
            }
            Msg::ApplyQuery => {
                if self.current_index.is_empty() {
                    self.push_toast("请先选择索引".to_string(), ToastType::Error, ctx);
                    return false;
                }
                self.current_page = 1;
                if let Some(timeout) = self.debounce.take() {
                    timeout.cancel();
                }
                ctx.link().send_message(Msg::PerformSearch);
                false
            }
            Msg::ClearQuery => {
                self.query_rows.clear();
                self.add_query_row();
                self.push_toast("查询条件已清空".to_string(), ToastType::Success, ctx);
                self.current_page = 1;
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::ToggleSearchField(field, checked) => {
                if checked {
                    if !self.search_fields.contains(&field) {
                        self.search_fields.push(field);
                    }
                } else {
                    self.search_fields.retain(|f| f != &field);
                }
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::ToggleFacet(facet, value, checked) => {
                let entry = self.facets.entry(facet).or_default();
                if checked {
                    if !entry.contains(&value) {
                        entry.push(value);
                    }
                } else {
                    entry.retain(|v| v != &value);
                }
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::ToggleHighlight(checked) => {
                self.highlight_enabled = checked;
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::ToggleRanking(checked) => {
                self.show_ranking_score = checked;
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::UpdateCropLength(value) => {
                self.crop_length = value.parse::<u32>().unwrap_or(0);
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::UpdatePageSize(value) => {
                self.page_size = value.parse::<u32>().unwrap_or(12).max(1);
                self.current_page = 1;
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::GoToPage(page) => {
                if page == self.current_page {
                    return false;
                }
                self.current_page = page.max(1);
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::SortSelect(value) => {
                self.sort_value = value;
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::OpenFilters(open) => {
                self.filters_drawer_open = open;
                true
            }
            Msg::ToggleTheme => {
                let next = if get_theme() == "dark" { "light" } else { "dark" };
                set_theme(next);
                true
            }
            Msg::ToggleAiDropdown => {
                self.ai_dropdown_open = !self.ai_dropdown_open;
                true
            }
            Msg::SetAiEnabled(enabled) => {
                self.ai_config.ai_enabled = enabled;
                save_ai_config(&self.ai_config);
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::SetAiWeight(weight) => {
                self.ai_config.ai_weight = weight;
                save_ai_config(&self.ai_config);
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::DocumentClick => {
                if self.ai_dropdown_open {
                    self.ai_dropdown_open = false;
                    return true;
                }
                false
            }
            Msg::OpenColumnConfig(open) => {
                if open && self.last_base_columns.is_empty() {
                    self.push_toast("暂无结果列可配置，请先搜索".to_string(), ToastType::Warning, ctx);
                    return false;
                }
                self.column_config_open = open;
                true
            }
            Msg::SaveColumnConfig(hidden) => {
                self.hidden_columns = hidden;
                let labels = collect_column_labels();
                if !labels.is_empty() {
                    for (k, v) in labels.iter() {
                        if !v.trim().is_empty() {
                            self.field_labels.insert(k.clone(), v.clone());
                        }
                    }
                    save_field_labels(&self.field_labels, Some(&self.current_index));
                }
                self.save_column_prefs();
                self.column_config_open = false;
                self.refresh_results_table();
                true
            }
            Msg::DragStart(col) => {
                if self.is_resizing_columns {
                    return false;
                }
                self.dragging_col = Some(col);
                true
            }
            Msg::DragOver(col) => {
                self.drag_over_col = Some(col);
                true
            }
            Msg::DropOn(col) => {
                if let Some(from) = self.dragging_col.clone() {
                    if from != col {
                        self.move_column_order(&from, &col);
                        self.refresh_results_table();
                    }
                }
                self.dragging_col = None;
                self.drag_over_col = None;
                true
            }
            Msg::DragEnd => {
                self.dragging_col = None;
                self.drag_over_col = None;
                true
            }
            Msg::ToggleTableSort(field) => {
                if self.is_resizing_columns || self.dragging_col.is_some() {
                    return false;
                }
                if self.table_sort_field == field {
                    if self.table_sort_dir == "asc" {
                        self.table_sort_dir = "desc".to_string();
                    } else {
                        self.table_sort_field.clear();
                        self.table_sort_dir = "asc".to_string();
                    }
                } else {
                    self.table_sort_field = field;
                    self.table_sort_dir = "asc".to_string();
                }
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::StartResize(col, start_x, start_width) => {
                self.is_resizing_columns = true;
                self.resize_state = Some(ResizeState {
                    col,
                    start_x,
                    start_width,
                });
                self.bind_resize_listeners(ctx);
                true
            }
            Msg::ResizeMove(x) => {
                if let Some(state) = &self.resize_state {
                    let col = state.col.clone();
                    let start_x = state.start_x;
                    let start_width = state.start_width;
                    let delta = x - start_x;
                    let next_width = (start_width + delta).max(80) as u32;
                    self.set_column_width(&col, next_width);
                    return true;
                }
                false
            }
            Msg::EndResize => {
                self.is_resizing_columns = false;
                self.resize_state = None;
                self.resize_move_listener = None;
                self.resize_up_listener = None;
                self.save_column_width_prefs();
                true
            }
            Msg::OpenFieldConfig(open) => {
                if open && self.current_index.is_empty() {
                    self.push_toast("请先选择索引".to_string(), ToastType::Error, ctx);
                    return false;
                }
                self.field_config_open = open;
                true
            }
            Msg::SelectAllSearchable(checked) => {
                set_all_checkboxes(".searchable-check", checked);
                false
            }
            Msg::SelectAllHighlight(checked) => {
                set_all_checkboxes(".highlight-check", checked);
                false
            }
            Msg::SelectAllDisplay(checked) => {
                set_all_checkboxes(".display-check", checked);
                false
            }
            Msg::SaveFieldConfig(items) => {
                let mut searchable = vec![];
                let mut highlight = vec![];
                let mut display = vec![];
                let mut weights = HashMap::new();
                let mut labels = HashMap::new();

                for item in items {
                    if item.searchable {
                        searchable.push(item.field.clone());
                    }
                    if item.highlight {
                        highlight.push(item.field.clone());
                    }
                    if item.display {
                        display.push(item.field.clone());
                    }
                    weights.insert(item.field.clone(), item.weight);
                    labels.insert(item.field.clone(), item.label.clone());
                }

                self.search_fields = searchable;
                self.highlight_fields = highlight;
                self.display_fields = display;
                self.search_field_weights = weights;
                self.field_labels = labels.clone();
                save_field_labels(&labels, Some(&self.current_index));
                self.update_popular_field();
                self.refresh_query_rows();

                let host = self.host_input.trim().to_string();
                let api_key = self.api_key_input.trim().to_string();
                let index = self.current_index.clone();
                let searchable_fields = self.search_fields.clone();
                let mut next_filterable = self.filterable_fields.clone();
                for field in &searchable_fields {
                    if !next_filterable.contains(field) {
                        next_filterable.push(field.clone());
                    }
                }
                self.filterable_fields = next_filterable.clone();

                let link = ctx.link().clone();
                self.loading = true;
                spawn_local(async move {
                    let res = update_index_settings(&host, &api_key, &index, &searchable_fields, &next_filterable).await;
                    link.send_message(Msg::SaveFieldConfigResult(res));
                });
                true
            }
            Msg::SaveFieldConfigResult(result) => {
                self.loading = false;
                match result {
                    Ok(_) => {
                        self.push_toast(
                            "字段配置已保存并已更新索引设置(含筛选字段)".to_string(),
                            ToastType::Success,
                            ctx,
                        );
                    }
                    Err(err) => {
                        self.push_toast(
                            format!("保存字段配置但无法更新索引设置: {err}"),
                            ToastType::Warning,
                            ctx,
                        );
                    }
                }
                self.field_config_open = false;
                self.load_popular_searches(ctx);
                true
            }
            Msg::ToggleEditLock => {
                self.edit_locked = !self.edit_locked;
                if !self.edit_locked {
                    if self.last_base_columns.is_empty() {
                        self.push_toast("暂无结果列，请先搜索".to_string(), ToastType::Warning, ctx);
                    } else if self.primary_key_field.is_empty() || !self.last_base_columns.contains(&self.primary_key_field) {
                        if self.last_base_columns.contains(&"id".to_string()) {
                            self.primary_key_field = "id".to_string();
                        } else {
                            self.primary_key_field = self.last_base_columns[0].clone();
                        }
                    }
                    if self.primary_key_field.is_empty() {
                        self.push_toast("请选择主键字段".to_string(), ToastType::Warning, ctx);
                    }
                }
                true
            }
            Msg::SetPrimaryKey(field) => {
                self.primary_key_field = field;
                self.pending_edits.clear();
                true
            }
            Msg::UpdateCellEdit(doc_id, field, value) => {
                if doc_id.is_empty() {
                    return false;
                }
                let entry = self.pending_edits.entry(doc_id).or_insert_with(HashMap::new);
                entry.insert(field, Value::String(value));
                true
            }
            Msg::SaveEdits => {
                if self.edit_locked {
                    self.push_toast("当前处于锁定状态，无法保存修改".to_string(), ToastType::Warning, ctx);
                    return false;
                }
                if self.pending_edits.is_empty() {
                    self.push_toast("没有需要保存的修改".to_string(), ToastType::Warning, ctx);
                    return false;
                }
                if self.current_index.is_empty() {
                    self.push_toast("请先选择索引".to_string(), ToastType::Error, ctx);
                    return false;
                }
                let host = self.host_input.trim().to_string();
                let api_key = self.api_key_input.trim().to_string();
                let index = self.current_index.clone();
                let hits = self.last_hits.clone();
                let primary_key = self.primary_key_field.clone();
                let edits = self.pending_edits.clone();
                let link = ctx.link().clone();
                self.loading = true;
                spawn_local(async move {
                    let res = update_documents(&host, &api_key, &index, &primary_key, &hits, &edits).await;
                    link.send_message(Msg::SaveEditsFinished(res));
                });
                true
            }
            Msg::SaveEditsFinished(result) => {
                self.loading = false;
                match result {
                    Ok(_) => {
                        self.pending_edits.clear();
                        self.push_toast("修改已提交保存".to_string(), ToastType::Success, ctx);
                        ctx.link().send_message(Msg::PerformSearch);
                    }
                    Err(err) => {
                        self.push_toast(format!("保存失败: {err}"), ToastType::Error, ctx);
                    }
                }
                true
            }
            Msg::StartViewColumnResize(col_idx, start_x, start_width_px, container_width_px) => {
                self.view_resize_state = Some(ViewResizeState {
                    col_idx,
                    start_x,
                    start_width_px,
                    container_width_px,
                    base_widths: self.current_view_widths(),
                });
                let link = ctx.link().clone();
                let window = web_window();
                let move_listener = EventListener::new(&window, "mousemove", move |event| {
                    let event = event.dyn_ref::<web_sys::MouseEvent>();
                    if let Some(event) = event {
                        link.send_message(Msg::ViewColumnResizeMove(event.client_x()));
                    }
                });
                let link = ctx.link().clone();
                let up_listener = EventListener::new(&window, "mouseup", move |_| {
                    link.send_message(Msg::EndViewColumnResize);
                });
                self.view_resize_move_listener = Some(move_listener);
                self.view_resize_up_listener = Some(up_listener);
                true
            }
            Msg::ViewColumnResizeMove(client_x) => {
                let state = match self.view_resize_state.clone() {
                    Some(state) => state,
                    None => return false,
                };
                if state.container_width_px <= 0.0 {
                    return false;
                }
                let delta = (client_x - state.start_x) as f32;
                let mut new_px = state.start_width_px + delta;
                let min_px = 140.0_f32;
                new_px = new_px.max(min_px).min(state.container_width_px);
                let mut new_percent = (new_px / state.container_width_px * 100.0).round() as u32;
                new_percent = new_percent.clamp(5, 95);
                let widths = Self::normalize_view_widths(state.base_widths.clone(), state.base_widths.len());
                if state.col_idx >= widths.len() || widths.len() == 1 {
                    return false;
                }
                let old_current = widths[state.col_idx];
                let sum_others = widths.iter().sum::<u32>().saturating_sub(old_current);
                let remaining = 100u32.saturating_sub(new_percent);
                let mut next = vec![0u32; widths.len()];
                next[state.col_idx] = new_percent;
                if sum_others == 0 {
                    let even = remaining / (widths.len() as u32 - 1);
                    let mut remain = remaining.saturating_sub(even * (widths.len() as u32 - 1));
                    for (i, item) in next.iter_mut().enumerate() {
                        if i == state.col_idx {
                            continue;
                        }
                        *item = even;
                        if remain > 0 {
                            *item += 1;
                            remain -= 1;
                        }
                    }
                } else {
                    let mut used = new_percent;
                    for (i, w) in widths.iter().enumerate() {
                        if i == state.col_idx {
                            continue;
                        }
                        let value = ((*w as f32 / sum_others as f32) * remaining as f32).floor() as u32;
                        next[i] = value;
                        used += value;
                    }
                    let mut remain = 100u32.saturating_sub(used);
                    let mut idx = 0usize;
                    while remain > 0 && next.len() > 1 {
                        if idx != state.col_idx {
                            next[idx] += 1;
                            remain -= 1;
                        }
                        idx = (idx + 1) % next.len();
                    }
                }
                self.set_active_view_widths(next.clone());
                true
            }
            Msg::EndViewColumnResize => {
                self.view_resize_state = None;
                self.view_resize_move_listener = None;
                self.view_resize_up_listener = None;
                true
            }
            Msg::StartViewFieldResize(col_idx, start_x, start_width_px, container_width_px) => {
                self.view_field_resize_state = Some(ViewFieldResizeState {
                    col_idx,
                    start_x,
                    start_width_px,
                    container_width_px,
                    base_widths: self.current_view_label_widths(),
                });
                let link = ctx.link().clone();
                let window = web_window();
                let move_listener = EventListener::new(&window, "mousemove", move |event| {
                    let event = event.dyn_ref::<web_sys::MouseEvent>();
                    if let Some(event) = event {
                        link.send_message(Msg::ViewFieldResizeMove(event.client_x()));
                    }
                });
                let link = ctx.link().clone();
                let up_listener = EventListener::new(&window, "mouseup", move |_| {
                    link.send_message(Msg::EndViewFieldResize);
                });
                self.view_field_resize_move_listener = Some(move_listener);
                self.view_field_resize_up_listener = Some(up_listener);
                true
            }
            Msg::ViewFieldResizeMove(client_x) => {
                let state = match self.view_field_resize_state.clone() {
                    Some(state) => state,
                    None => return false,
                };
                if state.container_width_px <= 0.0 {
                    return false;
                }
                let delta = (client_x - state.start_x) as f32;
                let mut new_px = state.start_width_px + delta;
                let min_px = 80.0_f32;
                let max_px = (state.container_width_px - 80.0).max(min_px);
                new_px = new_px.max(min_px).min(max_px);
                let mut widths = Self::normalize_label_widths(state.base_widths.clone(), state.base_widths.len());
                if state.col_idx >= widths.len() {
                    return false;
                }
                widths[state.col_idx] = new_px.round() as u32;
                self.set_active_view_label_widths(widths);
                true
            }
            Msg::EndViewFieldResize => {
                self.view_field_resize_state = None;
                self.view_field_resize_move_listener = None;
                self.view_field_resize_up_listener = None;
                true
            }
            Msg::OpenViewConfig(open) => {
                self.view_modal_open = open;
                if open {
                    if let Some(cfg) = self.active_view_config() {
                        self.view_name_input = cfg.name.clone();
                        self.view_layout_working = cfg.columns.clone();
                        self.view_widths_working = Self::normalize_view_widths(cfg.widths.clone(), self.view_layout_working.len());
                        self.view_label_widths_working = Self::normalize_label_widths(cfg.label_widths.clone(), self.view_layout_working.len());
                    } else {
                        self.view_name_input = "".to_string();
                        self.view_layout_working = vec![vec![], vec![]];
                        self.view_widths_working = vec![0; self.view_layout_working.len()];
                        self.view_label_widths_working = vec![140; self.view_layout_working.len()];
                    }
                }
                true
            }
            Msg::SetViewMode(mode) => {
                self.view_mode = mode;
                self.save_view_mode();
                true
            }
            Msg::UpdateViewName(value) => {
                self.view_name_input = value;
                true
            }
            Msg::StartViewFieldDrag(field) => {
                self.view_drag_field = Some(field);
                self.view_drag_from_column = None;
                true
            }
            Msg::StartViewFieldDragInColumn(field, column_idx) => {
                self.view_drag_field = Some(field);
                self.view_drag_from_column = Some(column_idx);
                true
            }
            Msg::DropViewField(column_idx) => {
                if let Some(field) = self.view_drag_field.clone() {
                    self.view_drag_field = None;
                    self.view_drag_from_column = None;
                    self.view_layout_working.iter_mut().for_each(|col| col.retain(|f| f != &field));
                    if column_idx < self.view_layout_working.len() {
                        self.view_layout_working[column_idx].push(field);
                    }
                    return true;
                }
                false
            }
            Msg::DropViewFieldToPool => {
                if let Some(field) = self.view_drag_field.clone() {
                    self.view_drag_field = None;
                    self.view_drag_from_column = None;
                    self.view_layout_working.iter_mut().for_each(|col| col.retain(|f| f != &field));
                    return true;
                }
                false
            }
            Msg::AddViewColumn => {
                self.view_layout_working.push(vec![]);
                self.view_widths_working.push(0);
                self.view_label_widths_working.push(140);
                true
            }
            Msg::RemoveViewColumn => {
                if self.view_layout_working.len() > 1 {
                    self.view_layout_working.pop();
                    self.view_widths_working.pop();
                    self.view_label_widths_working.pop();
                    return true;
                }
                false
            }
            Msg::RemoveViewColumnAt(idx) => {
                if self.view_layout_working.len() > 1 && idx < self.view_layout_working.len() {
                    self.view_layout_working.remove(idx);
                    if idx < self.view_widths_working.len() {
                        self.view_widths_working.remove(idx);
                    }
                    if idx < self.view_label_widths_working.len() {
                        self.view_label_widths_working.remove(idx);
                    }
                    return true;
                }
                false
            }
            Msg::StartViewColumnDrag(idx) => {
                self.view_drag_field = None;
                self.view_drag_from_column = None;
                self.view_drag_column = Some(idx);
                true
            }
            Msg::DropViewColumnAt(target_idx) => {
                if let Some(field) = self.view_drag_field.clone() {
                    self.view_drag_field = None;
                    self.view_drag_from_column = None;
                    if target_idx < self.view_layout_working.len() {
                        self.view_layout_working.iter_mut().for_each(|col| col.retain(|f| f != &field));
                        self.view_layout_working[target_idx].push(field);
                        return true;
                    }
                    return false;
                }
                let Some(from_idx) = self.view_drag_column.take() else { return false };
                let len = self.view_layout_working.len();
                if from_idx >= len || target_idx >= len || from_idx == target_idx {
                    return false;
                }
                let col = self.view_layout_working.remove(from_idx);
                let width = if from_idx < self.view_widths_working.len() { Some(self.view_widths_working.remove(from_idx)) } else { None };
                let label_width = if from_idx < self.view_label_widths_working.len() { Some(self.view_label_widths_working.remove(from_idx)) } else { None };
                let insert_at = if target_idx > from_idx { target_idx - 1 } else { target_idx };
                self.view_layout_working.insert(insert_at, col);
                if let Some(w) = width {
                    self.view_widths_working.insert(insert_at, w);
                }
                if let Some(w) = label_width {
                    self.view_label_widths_working.insert(insert_at, w);
                }
                true
            }
            Msg::SaveViewConfig => {
                let name = self.view_name_input.trim().to_string();
                if name.is_empty() {
                    self.push_toast("请输入视图名称".to_string(), ToastType::Warning, ctx);
                    return false;
                }
                let mut columns = self.view_layout_working.clone();
                for col in columns.iter_mut() {
                    col.retain(|f| !f.trim().is_empty());
                }
                let widths = Self::normalize_view_widths(self.view_widths_working.clone(), columns.len());
                let label_widths = Self::normalize_label_widths(self.view_label_widths_working.clone(), columns.len());
                let cfg = ViewConfig { name: name.clone(), columns, widths, label_widths };
                if let Some(existing) = self.view_configs.iter_mut().find(|c| c.name == name) {
                    *existing = cfg;
                } else {
                    self.view_configs.push(cfg);
                }
                self.save_view_configs();
                self.view_mode = name;
                self.save_view_mode();
                self.view_modal_open = false;
                true
            }
            Msg::OpenResultModal(id) => {
                match id {
                    Some(value) => {
                        if let Some(hit) = self.last_hits.iter().find(|hit| value == get_id_string(hit)) {
                            self.result_detail = Some(hit.clone());
                            self.result_modal_open = true;
                            return true;
                        }
                        if !self.current_index.is_empty() {
                            let host = self.host_input.trim().to_string();
                            let api_key = self.api_key_input.trim().to_string();
                            let index = self.current_index.clone();
                            let link = ctx.link().clone();
                            spawn_local(async move {
                                let res = fetch_by_id(&host, &api_key, &index, &value).await;
                                match res {
                                    Ok(hit) => {
                                        link.send_message(Msg::ShowResultDetail(hit));
                                    }
                                    Err(_) => {
                                        link.send_message(Msg::OpenResultModal(None));
                                    }
                                }
                            });
                        }
                        false
                    }
                    None => {
                        self.result_modal_open = false;
                        self.result_detail = None;
                        true
                    }
                }
            }
            Msg::ShowResultDetail(hit) => {
                self.result_detail = Some(hit);
                self.result_modal_open = true;
                true
            }
            Msg::CloseResultModal => {
                self.result_modal_open = false;
                self.result_detail = None;
                true
            }
            Msg::DismissToast(id) => {
                self.toasts.retain(|toast| toast.id != id);
                true
            }
            Msg::PerformHistorySearch(query) => {
                self.search_input = query;
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::ClearHistory => {
                self.search_history.clear();
                save_search_history(&self.search_history);
                self.push_toast("搜索历史已清空".to_string(), ToastType::Success, ctx);
                true
            }
            Msg::SelectPopularField(field) => {
                self.popular_search_field = field.clone();
                save_popular_field(&field, Some(&self.current_index));
                self.load_popular_searches(ctx);
                true
            }
            Msg::PopularSearchesLoaded(result) => {
                match result {
                    Ok(data) => {
                        self.popular_searches = data.items;
                    }
                    Err(err) => {
                        self.popular_searches = vec![];
                        self.push_toast(format!("热门搜索加载失败: {err}"), ToastType::Warning, ctx);
                    }
                }
                true
            }
            Msg::PerformPopularSearch(value) => {
                if self.popular_search_field.is_empty() {
                    self.search_input = value.clone();
                    ctx.link().send_message(Msg::PerformSearch);
                    return true;
                }
                self.search_input = "".to_string();
                self.query_rows.clear();
                self.add_query_row();
                if let Some(row) = self.query_rows.first_mut() {
                    row.field = self.popular_search_field.clone();
                    row.operator = "=".to_string();
                    row.value = value;
                }
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ai_active = self.ai_config.ai_enabled && self.ai_config.ai_weight > 0;
        let ai_weight = self.ai_config.ai_weight.min(100);
        let base_weight = 100 - ai_weight;
        let theme_is_dark = get_theme() == "dark";
        let filters_drawer_class = if self.filters_drawer_open { "filters-drawer active" } else { "filters-drawer" };
        let column_modal_class = if self.column_config_open { "modal active" } else { "modal" };
        let field_modal_class = if self.field_config_open { "modal active" } else { "modal" };
        let result_modal_class = if self.result_modal_open { "modal active" } else { "modal" };
        let loading_class = if self.loading { "loading-overlay active" } else { "loading-overlay" };
        let ai_dropdown_class = if self.ai_dropdown_open { "ai-dropdown active" } else { "ai-dropdown" };
        let ai_badge_class = if ai_active { "ai-badge active" } else { "ai-badge" };

        html! {
            <div class="container">
                <header class="header">
                    <div class="header-top">
                        <h1>{ "🔍  多维查询系统" }</h1>
                        <button class="theme-toggle" type="button" onclick={ctx.link().callback(|_| Msg::ToggleTheme)}>
                            { if theme_is_dark { "🌙 夜间模式" } else { "☀️ 日间模式" } }
                        </button>
                    </div>
                    <p>{ "强大的多维度搜索与过滤功能，快速定位您需要的内容" }</p>
                </header>

                <section class="connection-panel">
                    <div class="panel-header">
                        <h2 class="panel-title">{ "🔌 服务器连接配置" }</h2>
                        <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::Connect)}>{ "连接服务器" }</button>
                    </div>
                    <div class="form-grid">
                        <div class="form-group">
                            <label>{ "服务器地址" }</label>
                            <input
                                class="form-control"
                                value={self.host_input.clone()}
                                oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::SetHost(input_value(e)))}
                                placeholder="http://192.168.2.27:7700"
                            />
                        </div>
                        <div class="form-group">
                            <label>{ "API 密钥" }</label>
                            <input
                                class="form-control"
                                type="password"
                                value={self.api_key_input.clone()}
                                oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::SetApiKey(input_value(e)))}
                                placeholder="请输入API密钥"
                            />
                        </div>
                        <div class="form-group">
                            <label>{ "选择索引" }</label>
                            <select
                                class="form-control"
                                onchange={ctx.link().callback(|e: yew::events::Event| Msg::SelectIndex(select_value(e)))}
                                value={self.current_index.clone()}
                            >
                                { self.render_index_options() }
                            </select>
                        </div>
                    </div>
                </section>

                <section class="search-section">
                    <div class="search-panel">
                        <div class="search-box">
                            <span class="search-icon">{ "🔍" }</span>
                            <input
                                class="search-input"
                                value={self.search_input.clone()}
                                placeholder="输入搜索关键词..."
                                oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::SetSearchInput(input_value(e)))}
                            />
                            <span
                                class={ai_badge_class}
                                onclick={ctx.link().callback(|e: MouseEvent| {
                                    e.stop_propagation();
                                    Msg::ToggleAiDropdown
                                })}
                            >{ "AI" }</span>
                            <div
                                class={ai_dropdown_class}
                                onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                            >
                                <div class="dropdown-title">{ "AI 权重" }</div>
                                <label class="dropdown-row" style="margin-bottom: 10px; cursor: pointer;">
                                    <span>{ "启用 AI" }</span>
                                    <input
                                        type="checkbox"
                                        checked={self.ai_config.ai_enabled}
                                        onchange={ctx.link().callback(|e: yew::events::Event| Msg::SetAiEnabled(checkbox_checked(e)))}
                                    />
                                </label>
                                <div class="dropdown-row">
                                    <span>{ "基础" }</span>
                                    <span class="ai-weight-value">{ format!("{}%", base_weight) }</span>
                                </div>
                                <div class="dropdown-row">
                                    <span>{ "AI" }</span>
                                    <span class="ai-weight-value">{ format!("{}%", ai_weight) }</span>
                                </div>
                                <input
                                    type="range"
                                    min="0"
                                    max="100"
                                    value={ai_weight.to_string()}
                                    oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::SetAiWeight(input_value(e).parse::<u32>().unwrap_or(0)))}
                                />
                                <div class="ai-weight-hint">{ format!("基础 {}% · AI {}%", base_weight, ai_weight) }</div>
                            </div>
                        </div>

                        <div class="query-builder">
                            <h3 style="margin-bottom: 16px; font-size: 1.1rem;">{ "📋 查询条件构建器" }</h3>
                            <div>
                                { for self.query_rows.iter().map(|row| self.render_query_row(ctx, row)) }
                            </div>
                <div class="query-actions" style="margin-top: 16px;">
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::AddQueryRow)}>{ "➕ 添加查询条件" }</button>
                    <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::ApplyQuery)}>{ "✅ 应用查询" }</button>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::ClearQuery)}>{ "🗑️ 清空查询" }</button>
                </div>
                <div style="margin-top: 10px; font-size: 0.85rem; color: var(--text-secondary);">
                    { "当前过滤: " }{ self.filter_preview_text() }
                </div>
            </div>

                        <div class="advanced-settings">
                            <h3 style="margin-bottom: 16px; font-size: 1.1rem;">{ "⚙️ 高级搜索设置" }</h3>
                            <div class="settings-grid">
                                <div class="checkbox-group">
                                    <input
                                        type="checkbox"
                                        checked={self.highlight_enabled}
                                        onchange={ctx.link().callback(|e: yew::events::Event| Msg::ToggleHighlight(checkbox_checked(e)))}
                                    />
                                    <label>{ "启用高亮显示" }</label>
                                </div>
                                <div class="checkbox-group">
                                    <input
                                        type="checkbox"
                                        checked={self.show_ranking_score}
                                        onchange={ctx.link().callback(|e: yew::events::Event| Msg::ToggleRanking(checkbox_checked(e)))}
                                    />
                                    <label>{ "显示评分" }</label>
                                </div>
                                <div class="checkbox-group">
                                    <input
                                        type="number"
                                        value={self.crop_length.to_string()}
                                        min="0"
                                        style="width: 80px;"
                                        oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::UpdateCropLength(input_value(e)))}
                                    />
                                    <label>{ "内容截断长度" }</label>
                                </div>
                                <div class="form-group" style="flex: 1;">
                                    <label>{ "每页结果数" }</label>
                                    <input
                                        type="number"
                                        class="form-control"
                                        value={self.page_size.to_string()}
                                        min="1"
                                        max="100"
                                        oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::UpdatePageSize(input_value(e)))}
                                    />
                                </div>
                            </div>
                        </div>

                        <div class="results-panel" style="margin-top: 24px;">
                            <div class="results-stats">
                                <span class="results-count">{ "找到 " }<strong>{ self.results_count_text() }</strong>{ " 条结果" }</span>
                        <div class="results-actions">
                            <span>{ self.search_time_text() }</span>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::OpenFilters(true))}>{ "筛选入口" }</button>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::OpenColumnConfig(true))}>{ "列设置" }</button>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::OpenViewConfig(true))}>{ "视图设置" }</button>
                            <select
                                class="form-control"
                                style="min-width: 140px;"
                                onchange={ctx.link().callback(|e: yew::events::Event| Msg::SetViewMode(select_value(e)))}
                                value={self.view_mode.clone()}
                            >
                                { self.render_view_options() }
                            </select>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::ToggleEditLock)}>
                                { if self.edit_locked { "🔒 已锁定" } else { "🔓 可编辑" } }
                            </button>
                            { if !self.edit_locked {
                                html! {
                                    <select
                                        class="form-control"
                                        style="min-width: 140px;"
                                        onchange={ctx.link().callback(|e: yew::events::Event| Msg::SetPrimaryKey(select_value(e)))}
                                        value={self.primary_key_field.clone()}
                                    >
                                        { self.render_primary_key_options() }
                                    </select>
                                }
                            } else { Html::default() } }
                            <button
                                class="btn btn-primary"
                                disabled={self.edit_locked || self.pending_edits.is_empty()}
                                onclick={ctx.link().callback(|_| Msg::SaveEdits)}
                            >
                                { "保存修改" }
                            </button>
                        </div>
                            </div>
                            <div id="resultsContainer" class={if self.view_mode == "table" { "results-grid" } else { "results-grid custom-grid" }}>
                                { self.render_results(ctx) }
                            </div>
                            <div id="pagination" class="pagination">
                                { self.render_pagination(ctx) }
                            </div>
                        </div>
                    </div>
                </section>

                <footer class="footer">
                    <p>{ "作者: 冰城拓 Copyright © 2025. 保留所有权利." }</p>
                </footer>

                <div class="toast-container">
                    { for self.toasts.iter().map(|toast| self.render_toast(toast)) }
                </div>

                <div class={loading_class}>
                    <div class="spinner"></div>
                </div>

                <div class={field_modal_class}>
                    <div class="modal-content">
                        <div class="modal-header">
                            <h2 class="modal-title">{ "配置搜索字段" }</h2>
                            <button class="modal-close" onclick={ctx.link().callback(|_| Msg::OpenFieldConfig(false))}>{ "×" }</button>
                        </div>
                        <div style="display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 12px;">
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllSearchable(true))}>{ "全选可搜索" }</button>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllSearchable(false))}>{ "取消可搜索" }</button>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllHighlight(true))}>{ "全选高亮" }</button>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllHighlight(false))}>{ "取消高亮" }</button>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllDisplay(true))}>{ "全选显示" }</button>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllDisplay(false))}>{ "取消显示" }</button>
                        </div>
                        <div>
                            { for self.render_field_config(ctx) }
                        </div>
                        <div style="margin-top: 20px; text-align: right;">
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::OpenFieldConfig(false))}>{ "取消" }</button>
                            <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::SaveFieldConfig(collect_field_config()))}>{ "保存配置" }</button>
                        </div>
                    </div>
                </div>

                <div class={column_modal_class}>
                    <div class="modal-content">
                        <div class="modal-header">
                            <h2 class="modal-title">{ "列设置" }</h2>
                            <button class="modal-close" onclick={ctx.link().callback(|_| Msg::OpenColumnConfig(false))}>{ "×" }</button>
                        </div>
                        <div>
                            <p style="color: var(--text-secondary); font-size: 0.9rem; margin-bottom: 8px;">{ "勾选控制显示/隐藏，列顺序可在表头拖拽调整。" }</p>
                            <div class="column-config-list">
                                { self.render_column_config(ctx) }
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::OpenColumnConfig(false))}>{ "取消" }</button>
                            <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::SaveColumnConfig(collect_hidden_columns()))}>{ "保存设置" }</button>
                        </div>
                    </div>
                </div>

                <div class={if self.view_modal_open { "modal active" } else { "modal" }}>
                    <div class="modal-content view-modal">
                        <div class="modal-header">
                            <h2 class="modal-title">{ "视图设置" }</h2>
                            <button class="modal-close" onclick={ctx.link().callback(|_| Msg::OpenViewConfig(false))}>{ "×" }</button>
                        </div>
                        <div style="margin-bottom: 16px; display: flex; gap: 12px; align-items: center;">
                            <input
                                class="form-control"
                                style="max-width: 240px;"
                                placeholder="视图名称"
                                value={self.view_name_input.clone()}
                                oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::UpdateViewName(input_value(e)))}
                            />
                            <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::SaveViewConfig)}>{ "保存视图" }</button>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::AddViewColumn)}>{ "增加列" }</button>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::RemoveViewColumn)}>{ "删除列" }</button>
                        </div>
                        <div class="view-workspace">
                            <div class="view-fields">
                                <div class="view-title">{ "可用字段" }</div>
                                <div
                                    class="view-field-list"
                                    ondragover={Callback::from(|e: DragEvent| e.prevent_default())}
                                    ondrop={ctx.link().callback(|e: DragEvent| { e.prevent_default(); Msg::DropViewFieldToPool })}
                                >
                                    { for self.render_view_field_pool(ctx) }
                                </div>
                            </div>
                            <div class="view-columns" style={format!("grid-template-columns: repeat({}, minmax(0, 1fr));", self.view_layout_working.len().max(1))}>
                                { for (0..self.view_layout_working.len()).map(|idx| self.render_view_column(ctx, idx)) }
                            </div>
                        </div>
                    </div>
                </div>

                <div class={result_modal_class}>
                    <div class="modal-content">
                        <div class="modal-header">
                            <h2 class="modal-title">{ "结果详情" }</h2>
                            <button class="modal-close" onclick={ctx.link().callback(|_| Msg::CloseResultModal)}>{ "×" }</button>
                        </div>
                        <div>
                            { self.render_result_detail() }
                        </div>
                    </div>
                </div>

                <div class={filters_drawer_class} onclick={ctx.link().callback(|e: MouseEvent| {
                    let target = e.target().and_then(|t| t.dyn_into::<Element>().ok());
                    if let Some(el) = target {
                        if el.class_list().contains("filters-drawer") {
                            return Msg::OpenFilters(false);
                        }
                    }
                    Msg::OpenFilters(true)
                })}>
                    <div class="filters-drawer-content" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                        <div class="filters-drawer-header">
                            <h3 style="font-size: 1.1rem;">{ "筛选与排序" }</h3>
                            <button class="btn btn-secondary" style="padding: 4px 12px;" onclick={ctx.link().callback(|_| Msg::OpenFilters(false))}>{ "关闭" }</button>
                        </div>
                        <div class="filters-drawer-body">
                            <aside class="filters-sidebar">
                                <div class="filter-section">
                                    <h3 class="filter-title">
                                        { "搜索字段" }
                                        <button class="btn btn-secondary" style="padding: 4px 12px; font-size: 0.85rem;" onclick={ctx.link().callback(|_| Msg::OpenFieldConfig(true))}>{ "配置" }</button>
                                    </h3>
                                    <ul class="filter-list">
                                        { self.render_search_fields(ctx) }
                                    </ul>
                                </div>

                                <div class="filter-section" style={if self.facets_available() { "" } else { "display:none;" }}>
                                    <h3 class="filter-title">{ "📊 分面筛选" }</h3>
                                    <div>
                                        { self.render_facets(ctx) }
                                    </div>
                                </div>

                                <div class="filter-section">
                                    <h3 class="filter-title">
                                        { "📝 搜索历史" }
                                        <span class="clear-history" onclick={ctx.link().callback(|_| Msg::ClearHistory)}>{ "清空" }</span>
                                    </h3>
                                    <ul class="history-list">
                                        { self.render_history(ctx) }
                                    </ul>
                                </div>

                                <div class="filter-section">
                                    <h3 class="filter-title">{ "🔥 热门搜索" }</h3>
                                    <div class="form-group" style="margin-bottom: 12px;">
                                        <label style="font-size: 0.85rem; color: var(--text-secondary);">{ "热门字段" }</label>
                                        <select class="form-control" onchange={ctx.link().callback(|e: yew::events::Event| Msg::SelectPopularField(select_value(e)))} value={self.popular_search_field.clone()}>
                                            { self.render_popular_field_options() }
                                        </select>
                                    </div>
                                    <ul class="suggestions-list">
                                        { self.render_popular_searches(ctx) }
                                    </ul>
                                </div>

                                <div class="filter-section">
                                    <h3 class="filter-title">{ "📈 排序方式" }</h3>
                                    <select class="form-control" onchange={ctx.link().callback(|e: yew::events::Event| Msg::SortSelect(select_value(e)))} value={self.sort_value.clone()}>
                                        { self.render_sort_options() }
                                    </select>
                                </div>
                            </aside>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

impl App {
    fn initialize_theme(&self) {
        let saved = storage_get("theme").unwrap_or_else(|| "dark".to_string());
        set_theme(&saved);
    }

    fn reset_index_state(&mut self) {
        self.available_fields.clear();
        self.search_fields.clear();
        self.highlight_fields.clear();
        self.display_fields.clear();
        self.filterable_fields.clear();
        self.facets.clear();
        self.last_hits.clear();
        self.last_results = None;
        self.last_base_columns.clear();
        self.column_order.clear();
        self.hidden_columns.clear();
        self.column_widths.clear();
        self.search_field_weights.clear();
        self.popular_searches.clear();
        self.popular_search_field.clear();
        self.current_page = 1;
        self.primary_key_field = "id".to_string();
        self.pending_edits.clear();
        self.view_mode = "table".to_string();
        self.view_configs.clear();
        self.view_layout_working = vec![vec![], vec![]];
        self.view_widths_working = vec![0; self.view_layout_working.len()];
        self.view_label_widths_working = vec![140; self.view_layout_working.len()];
        self.view_drag_field = None;
        self.view_drag_from_column = None;
        self.view_drag_column = None;
        self.results_count = 0;
        self.processing_time_ms = None;
        self.facet_distribution = None;
        self.sortable_attributes.clear();
    }

    fn add_query_row(&mut self) {
        let id = self.next_query_id;
        self.next_query_id += 1;
        self.query_rows.push(QueryRow {
            id,
            field: "".to_string(),
            operator: "=".to_string(),
            value: "".to_string(),
            logic: "AND".to_string(),
        });
    }

    fn render_index_options(&self) -> Html {
        if self.indexes.is_empty() {
            return html! { <option value="">{ "请先连接服务器" }</option> };
        }
        let mut options = vec![html! { <option value="">{ "选择索引" }</option> }];
        for idx in &self.indexes {
            let count = idx.count.map(|v| format!(" ({})", format_number(v)));
            let label = format!("{}{}", idx.uid, count.unwrap_or_default());
            options.push(html! { <option value={idx.uid.clone()}>{ label }</option> });
        }
        html! { for options }
    }

    fn render_query_row(&self, ctx: &Context<Self>, row: &QueryRow) -> Html {
        let id = row.id;
        let filter_fields = self.get_filter_fields_for_query();
        let operator_value = normalize_operator(&row.operator);
        let logic_value = if row.logic.trim().is_empty() { "AND".to_string() } else { row.logic.clone() };
        html! {
            <div class="query-row" key={row.id} data-row-id={row.id.to_string()}>
                <select
                    class="form-control query-field"
                    value={row.field.clone()}
                    onchange={ctx.link().callback(move |e: yew::events::Event| Msg::UpdateQueryField(id, select_value(e)))}
                >
                    <option value="">{ "选择字段" }</option>
                    { for filter_fields.iter().map(|field| {
                        let label = self.field_labels.get(field).cloned().unwrap_or_else(|| field.clone());
                        html! { <option value={field.clone()}>{ label }</option> }
                    }) }
                </select>
                <select
                    class="form-control query-operator"
                    value={operator_value}
                    onchange={ctx.link().callback(move |e: yew::events::Event| Msg::UpdateQueryOperator(id, select_value(e)))}
                >
                    <option value="=">{ "等于" }</option>
                    <option value="!=">{ "不等于" }</option>
                    <option value=">">{ "大于" }</option>
                    <option value="<">{ "小于" }</option>
                    <option value=">=">{ "大于等于" }</option>
                    <option value="<=">{ "小于等于" }</option>
                    <option value="IN">{ "包含于" }</option>
                    <option value="NOT IN">{ "不包含于" }</option>
                    <option value="EXISTS">{ "存在" }</option>
                    <option value="NOT EXISTS">{ "不存在" }</option>
                </select>
                <input
                    class="form-control query-value"
                    value={row.value.clone()}
                    placeholder="输入值"
                    oninput={ctx.link().callback(move |e: yew::events::InputEvent| Msg::UpdateQueryValue(id, input_value(e)))}
                />
                <select
                    class="form-control query-logic"
                    value={logic_value}
                    onchange={ctx.link().callback(move |e: yew::events::Event| Msg::UpdateQueryLogic(id, select_value(e)))}
                >
                    <option value="AND">{ "AND" }</option>
                    <option value="OR">{ "OR" }</option>
                </select>
                <button class="btn remove-btn" onclick={ctx.link().callback(move |_| Msg::RemoveQueryRow(id))}>{ "删除" }</button>
            </div>
        }
    }

    fn filter_preview_text(&self) -> String {
        let keyword = self.search_input.trim();
        let filter_expr = build_filter_expression_from_dom();
        let keyword_part = if keyword.is_empty() {
            None
        } else {
            Some(format!("关键词: \"{}\"", keyword))
        };
        match (keyword_part, filter_expr) {
            (Some(k), Some(f)) => format!("{} AND {}", k, f),
            (Some(k), None) => k,
            (None, Some(f)) => f,
            (None, None) => "（无）".to_string(),
        }
    }

    fn get_filter_fields_for_query(&self) -> Vec<String> {
        if !self.filterable_fields.is_empty() {
            self.filterable_fields.clone()
        } else {
            self.search_fields.clone()
        }
    }

    fn refresh_query_rows(&mut self) {
        let fields = self.get_filter_fields_for_query();
        for row in &mut self.query_rows {
            if !fields.contains(&row.field) {
                row.field = "".to_string();
            }
        }
    }

    fn sync_query_rows_from_dom(&mut self) {
        let Ok(rows) = web_document().query_selector_all(".query-row") else { return };
        for i in 0..rows.length() {
            let Some(node) = rows.item(i) else { continue };
            let Some(row_el) = node.dyn_ref::<Element>() else { continue };
            let id_attr = row_el.get_attribute("data-row-id").unwrap_or_default();
            let Ok(row_id) = id_attr.parse::<u64>() else { continue };
            let field = row_el
                .query_selector(".query-field")
                .ok()
                .flatten()
                .and_then(|e| e.dyn_into::<web_sys::HtmlSelectElement>().ok())
                .map(|s| s.value())
                .unwrap_or_default();
            let operator = row_el
                .query_selector(".query-operator")
                .ok()
                .flatten()
                .and_then(|e| e.dyn_into::<web_sys::HtmlSelectElement>().ok())
                .map(|s| s.value())
                .unwrap_or_else(|| "=".to_string());
            let value = row_el
                .query_selector(".query-value")
                .ok()
                .flatten()
                .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                .map(|s| s.value())
                .unwrap_or_default();
            let logic = row_el
                .query_selector(".query-logic")
                .ok()
                .flatten()
                .and_then(|e| e.dyn_into::<web_sys::HtmlSelectElement>().ok())
                .map(|s| s.value())
                .unwrap_or_else(|| "AND".to_string());
            if let Some(row) = self.query_rows.iter_mut().find(|r| r.id == row_id) {
                row.field = field;
                row.operator = operator;
                row.value = value;
                row.logic = logic;
            }
        }
    }

    fn build_search_params(&self) -> Value {
        let mut params = json!({
            "limit": self.page_size,
            "offset": (self.current_page.saturating_sub(1)) * self.page_size,
            "matchingStrategy": "last"
        });

        if self.highlight_enabled {
            let targets = if !self.highlight_fields.is_empty() {
                self.highlight_fields.clone()
            } else if !self.search_fields.is_empty() {
                self.search_fields.clone()
            } else {
                vec!["*".to_string()]
            };
            params["attributesToHighlight"] = json!(targets);
            params["highlightPreTag"] = json!("<em class=\"highlight\">");
            params["highlightPostTag"] = json!("</em>");
        }

        if self.crop_length > 0 && self.highlight_enabled {
            let targets = if !self.highlight_fields.is_empty() {
                self.highlight_fields.clone()
            } else if !self.search_fields.is_empty() {
                self.search_fields.clone()
            } else {
                vec!["*".to_string()]
            };
            params["attributesToCrop"] = json!(targets);
            params["cropLength"] = json!(self.crop_length);
        }

        if self.show_ranking_score {
            params["showRankingScore"] = json!(true);
            params["showRankingScoreDetails"] = json!(true);
        }

        if self.ai_config.ai_enabled && self.ai_config.ai_weight > 0 {
            let ratio = (self.ai_config.ai_weight.min(100) as f64) / 100.0;
            params["hybrid"] = json!({
                "semanticRatio": ratio,
                "embedder": "bgem3"
            });
        }

        if !self.sort_value.is_empty() {
            params["sort"] = json!([self.sort_value.clone()]);
        }

        let mut filter_expr = build_filter_expression_from_dom();
        let facet_filters = self.build_facet_filters();
        if !facet_filters.is_empty() {
            let facet_expr = facet_filters.join(" AND ");
            filter_expr = match filter_expr {
                Some(expr) => Some(format!("({}) AND ({})", expr, facet_expr)),
                None => Some(facet_expr),
            };
        }
        if let Some(expr) = filter_expr {
            params["filter"] = json!(expr);
        }

        if !self.display_fields.is_empty() {
            let mut attrs = self.display_fields.clone();
            if !attrs.contains(&"id".to_string()) {
                attrs.push("id".to_string());
            }
            if !self.primary_key_field.is_empty() && !attrs.contains(&self.primary_key_field) {
                attrs.push(self.primary_key_field.clone());
            }
            params["attributesToRetrieve"] = json!(attrs);
        } else {
            params["attributesToRetrieve"] = json!(["*"]);
        }

        if !self.filterable_fields.is_empty() {
            params["facets"] = json!(self.filterable_fields.clone());
        }

        params
    }


    fn build_facet_filters(&self) -> Vec<String> {
        let mut filters = vec![];
        for (facet, values) in &self.facets {
            if values.is_empty() {
                continue;
            }
            if values.len() == 1 {
                filters.push(format!("{} = \"{}\"", facet, values[0]));
            } else {
                let list = values
                    .iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(", ");
                filters.push(format!("{} IN [{}]", facet, list));
            }
        }
        filters
    }

    fn apply_results_columns(&mut self, res: &SearchResponse) {
        let hits = res.hits.clone();
        if hits.is_empty() {
            self.last_base_columns.clear();
            return;
        }
        let mut columns = if !self.display_fields.is_empty() {
            self.display_fields.clone()
        } else {
            hits[0].fields.keys().cloned().collect::<Vec<_>>()
        };

        if columns.is_empty() {
            columns = hits[0].fields.keys().cloned().collect();
        }

        if columns.is_empty() {
            columns = vec!["id".to_string()];
        }

        if hit_has_id(&hits[0]) && !columns.contains(&"id".to_string()) {
            columns.insert(0, "id".to_string());
        }

        self.last_base_columns = columns;
    }

    fn update_stats(&mut self, res: &SearchResponse) {
        self.results_count = res.estimated_total_hits.unwrap_or(0);
        self.processing_time_ms = res.processing_time_ms;
    }

    fn update_facets_from_response(&mut self, res: &SearchResponse) {
        self.facet_distribution = res.facet_distribution.clone();
    }

    fn render_results(&self, ctx: &Context<Self>) -> Html {
        if self.last_results.is_none() {
            return html! {
                <div class="empty-state">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="11" cy="11" r="8"></circle>
                        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                    </svg>
                    <p>{ "输入搜索关键词开始查询" }</p>
                </div>
            };
        }
        let Some(results) = &self.last_results else { return Html::default(); };
        if results.hits.is_empty() {
            return html! {
                <div class="empty-state" style="grid-column: 1 / -1;">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="11" cy="11" r="8"></circle>
                        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                    </svg>
                    <p>{ "未找到匹配结果" }</p>
                </div>
            };
        }

        let mut hits = results.hits.clone();
        if self.view_mode != "table" {
            if let Some(cfg) = self.active_view_config() {
                return self.render_custom_results(ctx, &hits, &cfg);
            }
        }
        let columns = self.apply_column_prefs(self.last_base_columns.clone());
        if columns.is_empty() {
            return html! {
                <div class="empty-state" style="grid-column: 1 / -1;">
                    <p>{ "当前列已全部隐藏，请在“列设置”中勾选显示。" }</p>
                </div>
            };
        }

        if !self.table_sort_field.is_empty() && columns.contains(&self.table_sort_field) {
            hits = sort_hits(hits, &self.table_sort_field, &self.table_sort_dir);
        }

        let action_col = "__action__".to_string();
        let mut all_cols = columns.clone();
        all_cols.push(action_col.clone());

        let colgroup = html! {
            <colgroup>
                { for all_cols.iter().map(|col| {
                    let width = self.column_widths.get(col).copied();
                    let style = width.map(|w| format!("width: {}px;", w));
                    html! { <col data-col={col.clone()} style={style.unwrap_or_default()} /> }
                }) }
            </colgroup>
        };

        let header = html! {
            <thead>
                <tr>
                    { for columns.iter().map(|col| {
                        let label = self.field_labels.get(col).cloned().unwrap_or_else(|| col.clone());
                        let is_active = self.table_sort_field == *col;
                        let dir_mark = if is_active { if self.table_sort_dir == "asc" { " ▲" } else { " ▼" } } else { "" };
                        let col_key = col.clone();
                        let drag_col = col.clone();
                        let over_col = col.clone();
                        let drop_col = col.clone();
                        let sort_col = col.clone();
                        let resize_col = col.clone();
                        let mut th_class = if self.drag_over_col.as_ref() == Some(col) { "sortable-th drag-over".to_string() } else { "sortable-th".to_string() };
                        if !self.primary_key_field.is_empty() && self.primary_key_field == *col {
                            th_class.push_str(" pk-col");
                        }
                        html! {
                            <th
                                class={th_class}
                                draggable="true"
                                data-col={col_key.clone()}
                                ondragstart={ctx.link().callback(move |_| Msg::DragStart(drag_col.clone()))}
                                ondragover={ctx.link().callback(move |e: DragEvent| { e.prevent_default(); Msg::DragOver(over_col.clone()) })}
                                ondrop={ctx.link().callback(move |e: DragEvent| { e.prevent_default(); Msg::DropOn(drop_col.clone()) })}
                                ondragend={ctx.link().callback(|_| Msg::DragEnd)}
                                onclick={ctx.link().callback(move |_| Msg::ToggleTableSort(sort_col.clone()))}
                            >
                                <span class="th-label">
                                    { if !self.primary_key_field.is_empty() && self.primary_key_field == *col { "🔑 " } else { "" } }
                                    { format!("{}{}", label, dir_mark) }
                                </span>
                                <span
                                    class="column-resizer"
                                    data-col={col_key.clone()}
                                    title="拖动调整列宽"
                                    onmousedown={ctx.link().callback(move |e: MouseEvent| {
                                        e.prevent_default();
                                        e.stop_propagation();
                                        let target = e.target().and_then(|t| t.dyn_into::<Element>().ok());
                                        let start_width = target
                                            .and_then(|el| el.closest("th").ok().flatten())
                                            .and_then(|th| th.dyn_into::<HtmlElement>().ok())
                                            .map(|el| el.get_bounding_client_rect().width() as i32)
                                            .unwrap_or(120);
                                        Msg::StartResize(resize_col.clone(), e.client_x(), start_width)
                                    })}
                                ></span>
                            </th>
                        }
                    }) }
                    <th data-col={action_col.clone()}>{ "操作" }
                        <span
                            class="column-resizer"
                            data-col={action_col.clone()}
                            title="拖动调整列宽"
                            onmousedown={ctx.link().callback(move |e: MouseEvent| {
                                e.prevent_default();
                                e.stop_propagation();
                                let target = e.target().and_then(|t| t.dyn_into::<Element>().ok());
                                let start_width = target
                                    .and_then(|el| el.closest("th").ok().flatten())
                                    .and_then(|th| th.dyn_into::<HtmlElement>().ok())
                                    .map(|el| el.get_bounding_client_rect().width() as i32)
                                    .unwrap_or(120);
                                Msg::StartResize(action_col.clone(), e.client_x(), start_width)
                            })}
                        ></span>
                    </th>
                </tr>
            </thead>
        };

        let body = html! {
            <tbody>
                { for hits.iter().map(|hit| {
                    let id = get_id_string(hit);
                let row_cells = columns.iter().map(|col| {
                    let doc_id = get_doc_key(hit, &self.primary_key_field);
                    let is_primary_key = !self.primary_key_field.is_empty() && self.primary_key_field == *col;
                    let is_editable = !self.edit_locked && !doc_id.is_empty() && !is_primary_key;
                    let edited_value = self
                        .pending_edits
                        .get(&doc_id)
                        .and_then(|m| m.get(col))
                        .cloned();

                    if is_editable {
                        let raw_value = edited_value.unwrap_or_else(|| hit.get(col).cloned().unwrap_or(Value::Null));
                        let display = value_to_string_for_edit(&raw_value);
                        let field = col.clone();
                        let doc_key = doc_id.clone();
                        return html! {
                            <td>
                                <input
                                    class="form-control"
                                    style="padding: 6px 8px; font-size: 0.9rem;"
                                    value={display}
                                    oninput={ctx.link().callback(move |e: yew::events::InputEvent| {
                                        Msg::UpdateCellEdit(doc_key.clone(), field.clone(), input_value(e))
                                    })}
                                />
                            </td>
                        };
                    }

                    if let Some(edit_override) = edited_value {
                        let display = value_to_string_for_edit(&edit_override);
                        let class = if is_primary_key { "pk-cell" } else { "" };
                        return html! { <td class={class} title={display.clone()}>{ display }</td> };
                    }

                    if let Some(cell) = get_cell_value(hit, col, self.highlight_enabled) {
                        let class = if is_primary_key { "pk-cell" } else { "" };
                        html! { <td class={class} title={cell.title}>{ cell.html }</td> }
                    } else {
                        let class = if is_primary_key { "pk-cell" } else { "" };
                        html! { <td class={class}></td> }
                    }
                });
                    let ranking = hit.ranking_score.map(|score| {
                        html! { <>
                            <br />
                            <small style="color: var(--text-secondary);">{ format!("评分: {:.1}%", score * 100.0) }</small>
                        </> }
                    });
                    html! {
                        <tr>
                            { for row_cells }
                            <td>
                                <button class="btn btn-secondary" onclick={ctx.link().callback(move |_| Msg::OpenResultModal(Some(id.clone())))}>{ "查看" }</button>
                                { ranking }
                            </td>
                        </tr>
                    }
                }) }
            </tbody>
        };

        html! {
            <div class="results-table-wrap">
                <table class="results-table">
                    { colgroup }
                    { header }
                    { body }
                </table>
            </div>
        }
    }

    fn render_pagination(&self, ctx: &Context<Self>) -> Html {
        let total_hits = self.results_count;
        let total_pages = (total_hits as f64 / self.page_size as f64).ceil() as u32;
        if total_pages <= 1 {
            return Html::default();
        }
        let mut items: Vec<Html> = vec![];
        let prev_disabled = self.current_page == 1;
        let prev_page = self.current_page.saturating_sub(1);
        items.push(html! {
            <button class="pagination-btn" disabled={prev_disabled} onclick={ctx.link().callback(move |_| Msg::GoToPage(prev_page))}>{ "上一页" }</button>
        });
        let start_page = self.current_page.saturating_sub(2).max(1);
        let end_page = (self.current_page + 2).min(total_pages);
        if start_page > 1 {
            items.push(html! { <button class="pagination-btn" onclick={ctx.link().callback(|_| Msg::GoToPage(1))}>{ "1" }</button> });
            if start_page > 2 {
                items.push(html! { <span>{ "..." }</span> });
            }
        }
        for i in start_page..=end_page {
            let class = if i == self.current_page { "pagination-btn active" } else { "pagination-btn" };
            items.push(html! { <button class={class} onclick={ctx.link().callback(move |_| Msg::GoToPage(i))}>{ i }</button> });
        }
        if end_page < total_pages {
            if end_page + 1 < total_pages {
                items.push(html! { <span>{ "..." }</span> });
            }
            let last_page = total_pages;
            items.push(html! { <button class="pagination-btn" onclick={ctx.link().callback(move |_| Msg::GoToPage(last_page))}>{ last_page }</button> });
        }
        let next_disabled = self.current_page >= total_pages;
        let next_page = self.current_page + 1;
        items.push(html! {
            <button class="pagination-btn" disabled={next_disabled} onclick={ctx.link().callback(move |_| Msg::GoToPage(next_page))}>{ "下一页" }</button>
        });
        html! { for items }
    }

    fn render_custom_results(&self, ctx: &Context<Self>, hits: &[SearchHit], cfg: &ViewConfig) -> Html {
        let columns = if cfg.columns.is_empty() { vec![vec![]] } else { cfg.columns.clone() };
        let widths = Self::normalize_view_widths(cfg.widths.clone(), columns.len());
        let label_widths = Self::normalize_label_widths(cfg.label_widths.clone(), columns.len());
        let gap_px = 12.0_f32;
        let gap_each = if columns.len() > 0 {
            gap_px * (columns.len() as f32 - 1.0) / columns.len() as f32
        } else {
            0.0
        };
        let grid_cols = widths
            .iter()
            .map(|w| format!("calc({}% - {:.2}px)", w, gap_each))
            .collect::<Vec<_>>()
            .join(" ");
        html! {
            <div class="custom-results">
                { for hits.iter().map(|hit| {
                    let id = get_id_string(hit);
                    html! {
                        <div class="custom-row">
                            <div class="custom-row-columns" style={format!("grid-template-columns: {};", grid_cols)}>
                                { for columns.iter().enumerate().map(|(col_idx, col_fields)| {
                                    let widths = widths.clone();
                                    let label_widths = label_widths.clone();
                                    let label_px = label_widths.get(col_idx).cloned().unwrap_or(140);
                                    html! {
                                        <div class="custom-col" style={format!("--label-width: {}px;", label_px)}>
                                            <span
                                                class="view-col-resizer"
                                                title="拖动调整列宽"
                                                onmousedown={ctx.link().callback(move |e: MouseEvent| {
                                                    e.prevent_default();
                                                    e.stop_propagation();
                                                    let target = e.target().and_then(|t| t.dyn_into::<Element>().ok());
                                                    let (start_width, container_width) = target
                                                        .and_then(|el| el.closest(".custom-row-columns").ok().flatten())
                                                        .and_then(|el| el.dyn_into::<HtmlElement>().ok())
                                                        .map(|el| {
                                                            let rect = el.get_bounding_client_rect();
                                                            (rect.width() as f32, rect.width() as f32)
                                                        })
                                                        .unwrap_or((0.0, 0.0));
                                                    let current_percent = widths.get(col_idx).cloned().unwrap_or(0) as f32;
                                                    let start_width_px = if container_width > 0.0 { container_width * current_percent / 100.0 } else { start_width };
                                                    Msg::StartViewColumnResize(col_idx, e.client_x(), start_width_px, container_width)
                                                })}
                                            ></span>
                                            <span
                                                class="field-width-resizer"
                                                title="拖动调整字段宽度"
                                                onmousedown={ctx.link().callback(move |e: MouseEvent| {
                                                    e.prevent_default();
                                                    e.stop_propagation();
                                                    let target = e.target().and_then(|t| t.dyn_into::<Element>().ok());
                                                    let container_width = target
                                                        .and_then(|el| el.closest(".custom-col").ok().flatten())
                                                        .and_then(|el| el.dyn_into::<HtmlElement>().ok())
                                                        .map(|el| el.get_bounding_client_rect().width() as f32)
                                                        .unwrap_or(0.0);
                                                    Msg::StartViewFieldResize(col_idx, e.client_x(), label_px as f32, container_width)
                                                })}
                                            ></span>
                                            { for col_fields.iter().map(|field| {
                                                let label = self.field_labels.get(field).cloned().unwrap_or_else(|| field.clone());
                                                let value = hit.get(field).map(value_to_string).unwrap_or_default();
                                                html! {
                                                    <div class="custom-field">
                                                        <span class="custom-label">{ label }</span>
                                                        <span class="custom-value">{ value }</span>
                                                    </div>
                                                }
                                            }) }
                                        </div>
                                    }
                                }) }
                            </div>
                            <div class="custom-row-actions">
                                <button class="btn btn-secondary" onclick={ctx.link().callback(move |_| Msg::OpenResultModal(Some(id.clone())))}>{ "查看" }</button>
                            </div>
                        </div>
                    }
                }) }
            </div>
        }
    }

    fn render_search_fields(&self, ctx: &Context<Self>) -> Html {
        if self.available_fields.is_empty() {
            return html! { <li style="padding: 12px; color: var(--text-secondary);">{ "暂无搜索字段" }</li> };
        }
        html! {
            for self.available_fields.iter().map(|field| {
                let checked = self.search_fields.contains(field);
                let f = field.clone();
                html! {
                    <li class="filter-item">
                        <input
                            type="checkbox"
                            checked={checked}
                            onchange={ctx.link().callback(move |e: yew::events::Event| Msg::ToggleSearchField(f.clone(), checkbox_checked(e)))}
                        />
                        <span class="filter-label">{ field.clone() }</span>
                    </li>
                }
            })
        }
    }

    fn render_history(&self, ctx: &Context<Self>) -> Html {
        if self.search_history.is_empty() {
            return html! { <li style="padding: 12px; color: var(--text-secondary);">{ "暂无搜索历史" }</li> };
        }
        html! {
            for self.search_history.iter().map(|item| {
                let query = item.query.clone();
                html! {
                    <li class="history-item" onclick={ctx.link().callback(move |_| Msg::PerformHistorySearch(query.clone()))}>
                        <span>{ item.query.clone() }</span>
                        <span class="history-time">{ format_time(&item.timestamp) }</span>
                    </li>
                }
            })
        }
    }

    fn add_to_history(&mut self, query: String) {
        if query.is_empty() || self.search_history.iter().any(|h| h.query == query) {
            return;
        }
        let timestamp = js_sys::Date::new_0()
            .to_iso_string()
            .as_string()
            .unwrap_or_default();
        let item = HistoryItem { query, timestamp };
        self.search_history.insert(0, item);
        if self.search_history.len() > 20 {
            self.search_history.truncate(20);
        }
        save_search_history(&self.search_history);
    }

    fn render_popular_field_options(&self) -> Html {
        if self.filterable_fields.is_empty() {
            return html! { <option value="">{ "选择字段" }</option> };
        }
        let mut items = vec![html! { <option value="">{ "选择字段" }</option> }];
        for field in &self.filterable_fields {
            items.push(html! { <option value={field.clone()}>{ field.clone() }</option> });
        }
        html! { for items }
    }

    fn render_popular_searches(&self, ctx: &Context<Self>) -> Html {
        if self.popular_searches.is_empty() {
            return html! { <li style="padding: 12px; color: var(--text-secondary);">{ "暂无热门搜索" }</li> };
        }
        html! {
            for self.popular_searches.iter().map(|item| {
                let value = item.value.clone();
                html! {
                    <li class="suggestion-item" onclick={ctx.link().callback(move |_| Msg::PerformPopularSearch(value.clone()))}>
                        { item.value.clone() }
                        { item.count.map(|c| html! { <span class="filter-count">{ c }</span> }).unwrap_or_default() }
                    </li>
                }
            })
        }
    }

    fn render_sort_options(&self) -> Html {
        let mut items = vec![html! { <option value="">{ "默认排序" }</option> }];
        for attr in self.sortable_attributes() {
            let asc = format!("{}:asc", attr);
            let desc = format!("{}:desc", attr);
            items.push(html! { <option value={asc.clone()}>{ format!("{} 升序", attr) }</option> });
            items.push(html! { <option value={desc.clone()}>{ format!("{} 降序", attr) }</option> });
        }
        html! { for items }
    }

    fn render_primary_key_options(&self) -> Html {
        if self.last_base_columns.is_empty() {
            return html! { <option value="">{ "选择主键" }</option> };
        }
        let mut items = vec![html! { <option value="">{ "选择主键" }</option> }];
        for col in &self.last_base_columns {
            items.push(html! { <option value={col.clone()}>{ col.clone() }</option> });
        }
        html! { for items }
    }

    fn render_view_options(&self) -> Html {
        let mut items = vec![html! { <option value="table">{ "表格" }</option> }];
        for cfg in &self.view_configs {
            items.push(html! { <option value={cfg.name.clone()}>{ cfg.name.clone() }</option> });
        }
        html! { for items }
    }

    fn render_view_field_pool(&self, ctx: &Context<Self>) -> Vec<Html> {
        let mut used = HashSet::new();
        for col in &self.view_layout_working {
            for f in col {
                used.insert(f.clone());
            }
        }
        let mut nodes = vec![];
        for field in &self.available_fields {
            if used.contains(field) {
                continue;
            }
            let label = self.field_labels.get(field).cloned().unwrap_or_else(|| field.clone());
            let field_name = field.clone();
            nodes.push(html! {
                <div
                    class="view-field-item"
                    draggable="true"
                    ondragstart={ctx.link().callback(move |e: DragEvent| {
                        if let Some(dt) = e.data_transfer() {
                            let _ = dt.set_data("text/plain", &field_name);
                        }
                        Msg::StartViewFieldDrag(field_name.clone())
                    })}
                >
                    <span>{ label }</span>
                </div>
            });
        }
        nodes
    }

    fn render_view_column(&self, ctx: &Context<Self>, idx: usize) -> Html {
        let title = format!("列 {}", idx + 1);
        let fields = self.view_layout_working.get(idx).cloned().unwrap_or_default();
        let can_remove = self.view_layout_working.len() > 1;
        html! {
            <div
                class="view-column"
            >
                <div
                    class="view-title-row"
                    draggable="true"
                    ondragstart={ctx.link().callback(move |_| Msg::StartViewColumnDrag(idx))}
                    ondragover={Callback::from(|e: DragEvent| e.prevent_default())}
                    ondrop={ctx.link().callback(move |e: DragEvent| { e.prevent_default(); Msg::DropViewColumnAt(idx) })}
                >
                    <div class="view-title">{ title }</div>
                    <button
                        class="btn btn-secondary"
                        style="padding: 2px 8px; font-size: 0.75rem;"
                        disabled={!can_remove}
                        onclick={ctx.link().callback(move |_| Msg::RemoveViewColumnAt(idx))}
                    >
                        { "删除" }
                    </button>
                </div>
                <div
                    class="view-column-body"
                    ondragover={Callback::from(|e: DragEvent| e.prevent_default())}
                    ondrop={ctx.link().callback(move |e: DragEvent| { e.prevent_default(); Msg::DropViewField(idx) })}
                >
                    { for fields.iter().map(|field| {
                        let label = self.field_labels.get(field).cloned().unwrap_or_else(|| field.clone());
                        let drag_field = field.clone();
                        html! {
                            <div
                                class="view-field-item view-field-selected"
                                draggable="true"
                                ondragstart={ctx.link().callback(move |e: DragEvent| {
                                    if let Some(dt) = e.data_transfer() {
                                        let _ = dt.set_data("text/plain", &drag_field);
                                    }
                                    Msg::StartViewFieldDragInColumn(drag_field.clone(), idx)
                                })}
                            >
                                <span>{ label }</span>
                            </div>
                        }
                    }) }
                </div>
            </div>
        }
    }

    fn render_facets(&self, ctx: &Context<Self>) -> Html {
        let mut nodes = vec![];
                if let Some(map) = &self.facet_distribution {
                    for (facet_name, values) in map {
                        let mut entries: Vec<(String, u64)> = values.iter().map(|(v, c)| (v.clone(), *c)).collect();
                        entries.sort_by(|a, b| b.1.cmp(&a.1));
                        let subset = entries.into_iter().take(10).collect::<Vec<_>>();
                let facet_title = facet_name.clone();
                nodes.push(html! {
                    <div class="filter-section" style="margin-bottom: 16px; padding-bottom: 16px;">
                        <h4 style="margin-bottom: 12px; font-size: 1rem;">{ facet_title }</h4>
                        <ul class="filter-list">
                            { for subset.iter().map(|(value, count)| {
                                let value_clone = value.clone();
                                let facet_name_clone = facet_name.clone();
                                let checked = self.facets.get(&facet_name_clone).map(|v| v.contains(value)).unwrap_or(false);
                                html! {
                                    <li class="filter-item">
                                        <input
                                            type="checkbox"
                                            checked={checked}
                                            onchange={ctx.link().callback(move |e: yew::events::Event| Msg::ToggleFacet(facet_name_clone.clone(), value_clone.clone(), checkbox_checked(e)))}
                                        />
                                        <span class="filter-label">{ value.clone() }</span>
                                        <span class="filter-count">{ count }</span>
                                    </li>
                                }
                            }) }
                        </ul>
                    </div>
                });
            }
        }
        html! { for nodes }
    }

    fn render_toast(&self, toast: &Toast) -> Html {
        let class = match toast.kind {
            ToastType::Success => "toast success",
            ToastType::Error => "toast error",
            ToastType::Warning => "toast warning",
        };
        let icon = match toast.kind {
            ToastType::Success => "✅",
            ToastType::Error => "❌",
            ToastType::Warning => "⚠️",
        };
        html! {
            <div class={class}>
                <span>{ icon }</span>
                <span>{ toast.message.clone() }</span>
            </div>
        }
    }

    fn render_column_config(&self, _ctx: &Context<Self>) -> Html {
        let base = if !self.last_base_columns.is_empty() {
            self.last_base_columns.clone()
        } else {
            vec![]
        };
        let ordered = self.order_columns(base);
        let hidden: HashSet<String> = self.hidden_columns.iter().cloned().collect();
        html! {
            for ordered.iter().map(|col| {
                let checked = !hidden.contains(col);
                let label_value = self.field_labels.get(col).cloned().unwrap_or_else(|| col.clone());
                html! {
                    <label class="column-config-item">
                        <input type="checkbox" data-col={col.clone()} checked={checked} />
                        <div class="column-name-stack">
                            <span class="column-name-original">{ col.clone() }</span>
                            <input
                                class="form-control column-label-input column-name-display"
                                data-col={col.clone()}
                                value={label_value}
                            />
                        </div>
                    </label>
                }
            })
        }
    }


    fn render_field_config(&self, _ctx: &Context<Self>) -> Vec<Html> {
        let mut nodes = vec![];
        let weights = &self.search_field_weights;
        let searchable_default = self.search_fields.is_empty();
        let highlight_default = self.highlight_fields.is_empty();
        let display_default = self.display_fields.is_empty();
        for field in &self.available_fields {
            let searchable = searchable_default || self.search_fields.contains(field);
            let highlight = highlight_default || self.highlight_fields.contains(field);
            let display = display_default || self.display_fields.contains(field);
            let weight = weights.get(field).cloned().unwrap_or(1.0);
            let label = self.field_labels.get(field).cloned().unwrap_or_else(|| field.clone());
            nodes.push(html! {
                <div class="field-config">
                    <div class="field-header">
                        <h4 style="margin: 0;">{ field.clone() }</h4>
                        <label>
                            <input type="checkbox" class="searchable-check" data-field={field.clone()} checked={searchable} />
                            { " 可搜索" }
                        </label>
                    </div>
                    <div class="field-controls">
                        <div class="form-group">
                            <label>{ "权重" }</label>
                            <input type="number" class="form-control field-weight" data-field={field.clone()} value={weight.to_string()} min="0" step="0.1" />
                        </div>
                        <div class="form-group">
                            <label>{ "列名" }</label>
                            <input type="text" class="form-control field-label" data-field={field.clone()} value={label} />
                        </div>
                        <div class="form-group">
                            <label>{ "高亮" }</label>
                            <label class="inline-check">
                                <input type="checkbox" class="highlight-check" data-field={field.clone()} checked={highlight} />
                                { "启用" }
                            </label>
                        </div>
                        <div class="form-group">
                            <label>{ "显示" }</label>
                            <label class="inline-check">
                                <input type="checkbox" class="display-check" data-field={field.clone()} checked={display} />
                                { "启用" }
                            </label>
                        </div>
                    </div>
                </div>
            });
        }
        nodes
    }


    fn render_result_detail(&self) -> Html {
        if let Some(item) = &self.result_detail {
            let title = item
                .get("title")
                .or_else(|| item.get("name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let description = item
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let mut rows = vec![];
            if let Some(id_value) = item.id.as_ref().or_else(|| item.fields.get("id")) {
                rows.push(html! {
                    <div style="margin-bottom: 12px;">
                        <strong style="color: var(--text-primary);">{ "id:" }</strong>
                        <span style="color: var(--text-secondary);">{ value_to_string(id_value) }</span>
                    </div>
                });
            }
            for (key, value) in hit_entries(item) {
                if ["id", "title", "name", "description"].contains(&key.as_str()) {
                    continue;
                }
                let display = if value.is_array() {
                    value.as_array().unwrap().iter().map(|v| value_to_string(v)).collect::<Vec<_>>().join(", ")
                } else if value.is_object() {
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| value_to_string(&value))
                } else {
                    value_to_string(&value)
                };
                rows.push(html! {
                    <div style="margin-bottom: 12px;">
                        <strong style="color: var(--text-primary);">{ format!("{}:", key) }</strong>
                        <span style="color: var(--text-secondary);">{ display }</span>
                    </div>
                });
            }
            html! {
                <div style="line-height: 1.8;">
                    { title.map(|t| html! { <h3 style="margin-bottom: 16px;">{ t }</h3> }).unwrap_or_default() }
                    { description.map(|d| html! { <p style="margin-bottom: 16px; color: var(--text-secondary);">{ d }</p> }).unwrap_or_default() }
                    { for rows }
                </div>
            }
        } else {
            html! {}
        }
    }

    fn results_count_text(&self) -> String {
        format_number(self.results_count)
    }

    fn search_time_text(&self) -> String {
        self.processing_time_ms
            .map(|ms| format!("耗时 {}ms", ms))
            .unwrap_or_default()
    }

    fn facets_available(&self) -> bool {
        self.facet_distribution.as_ref().map(|m| !m.is_empty()).unwrap_or(false)
    }

    fn sortable_attributes(&self) -> Vec<String> {
        self.sortable_attributes.clone()
    }

    fn update_popular_field(&mut self) {
        let current = load_popular_field(Some(&self.current_index));
        if !current.is_empty() && self.filterable_fields.contains(&current) {
            self.popular_search_field = current;
        } else if self.filterable_fields.is_empty() {
            self.popular_search_field.clear();
        } else {
            self.popular_search_field = self.filterable_fields[0].clone();
            save_popular_field(&self.popular_search_field, Some(&self.current_index));
        }
    }

    fn load_popular_searches(&self, ctx: &Context<Self>) {
        if self.current_index.is_empty() || self.popular_search_field.is_empty() {
            ctx.link().send_message(Msg::PopularSearchesLoaded(Ok(PopularSearchData { items: vec![] })));
            return;
        }
        let host = self.host_input.trim().to_string();
        let api_key = self.api_key_input.trim().to_string();
        let index = self.current_index.clone();
        let field = self.popular_search_field.clone();
        let link = ctx.link().clone();
        spawn_local(async move {
            let res = load_popular_searches(&host, &api_key, &index, &field).await;
            link.send_message(Msg::PopularSearchesLoaded(res));
        });
    }

    fn load_column_prefs(&mut self) {
        let key = format!("columnPrefs:{}", if self.current_index.is_empty() { "default" } else { &self.current_index });
        if let Some(value) = storage_get(&key) {
            if let Ok(data) = serde_json::from_str::<Value>(&value) {
                if let Some(order) = data.get("order").and_then(|v| v.as_array()) {
                    self.column_order = order.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(hidden) = data.get("hidden").and_then(|v| v.as_array()) {
                    self.hidden_columns = hidden.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }
            }
        }
    }

    fn save_column_prefs(&self) {
        let key = format!("columnPrefs:{}", if self.current_index.is_empty() { "default" } else { &self.current_index });
        let data = json!({
            "order": self.column_order,
            "hidden": self.hidden_columns,
        });
        storage_set(&key, &data.to_string());
    }

    fn load_column_width_prefs(&mut self) {
        let key = format!("columnWidthPrefs:{}", if self.current_index.is_empty() { "default" } else { &self.current_index });
        if let Some(value) = storage_get(&key) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, u32>>(&value) {
                self.column_widths = map;
            }
        }
    }

    fn load_view_configs(&mut self) {
        let key = format!("viewConfigs:{}", if self.current_index.is_empty() { "default" } else { &self.current_index });
        self.view_configs = storage_get(&key)
            .and_then(|value| serde_json::from_str::<Vec<ViewConfig>>(&value).ok())
            .unwrap_or_default();
        for cfg in self.view_configs.iter_mut() {
            let len = cfg.columns.len().max(1);
            cfg.widths = Self::normalize_view_widths(cfg.widths.clone(), len);
            cfg.label_widths = Self::normalize_label_widths(cfg.label_widths.clone(), len);
        }
        let mode_key = format!("viewMode:{}", if self.current_index.is_empty() { "default" } else { &self.current_index });
        let mode = storage_get(&mode_key).unwrap_or_else(|| "table".to_string());
        self.view_mode = if mode.is_empty() { "table".to_string() } else { mode };
    }

    fn save_view_configs(&self) {
        let key = format!("viewConfigs:{}", if self.current_index.is_empty() { "default" } else { &self.current_index });
        if let Ok(value) = serde_json::to_string(&self.view_configs) {
            storage_set(&key, &value);
        }
    }

    fn save_view_mode(&self) {
        let key = format!("viewMode:{}", if self.current_index.is_empty() { "default" } else { &self.current_index });
        storage_set(&key, &self.view_mode);
    }

    fn active_view_config(&self) -> Option<ViewConfig> {
        self.view_configs.iter().find(|c| c.name == self.view_mode).cloned()
    }

    fn normalize_view_widths(mut widths: Vec<u32>, len: usize) -> Vec<u32> {
        if widths.len() < len {
            widths.resize(len, 0);
        } else if widths.len() > len {
            widths.truncate(len);
        }
        if len == 0 {
            return widths;
        }
        if widths.iter().any(|w| *w > 100) {
            let even = 100 / len as u32;
            let mut result = vec![even; len];
            let mut remain = 100u32.saturating_sub(even * len as u32);
            let mut idx = 0usize;
            while remain > 0 {
                result[idx] += 1;
                remain -= 1;
                idx = (idx + 1) % len;
            }
            return result;
        }
        let sum: u32 = widths.iter().sum();
        if sum == 0 {
            let even = 100 / len as u32;
            let mut result = vec![even; len];
            let mut remain = 100u32.saturating_sub(even * len as u32);
            let mut idx = 0usize;
            while remain > 0 {
                result[idx] += 1;
                remain -= 1;
                idx = (idx + 1) % len;
            }
            return result;
        }
        let mut normalized = Vec::with_capacity(len);
        let mut used = 0u32;
        for w in widths.iter() {
            let value = (*w as u64 * 100u64 / sum as u64) as u32;
            normalized.push(value);
            used += value;
        }
        let mut remain = 100u32.saturating_sub(used);
        let mut idx = 0usize;
        while remain > 0 && !normalized.is_empty() {
            normalized[idx] += 1;
            remain -= 1;
            idx = (idx + 1) % normalized.len();
        }
        normalized
    }

    fn normalize_label_widths(mut widths: Vec<u32>, len: usize) -> Vec<u32> {
        let default_width = 140u32;
        if widths.len() < len {
            widths.resize(len, default_width);
        } else if widths.len() > len {
            widths.truncate(len);
        }
        for item in widths.iter_mut() {
            if *item < 60 {
                *item = 60;
            }
        }
        widths
    }

    fn current_view_widths(&self) -> Vec<u32> {
        if let Some(cfg) = self.active_view_config() {
            return Self::normalize_view_widths(cfg.widths.clone(), cfg.columns.len().max(1));
        }
        Self::normalize_view_widths(vec![0; self.view_layout_working.len().max(1)], self.view_layout_working.len().max(1))
    }

    fn set_active_view_widths(&mut self, widths: Vec<u32>) {
        if let Some(cfg) = self.view_configs.iter_mut().find(|c| c.name == self.view_mode) {
            cfg.widths = Self::normalize_view_widths(widths, cfg.columns.len().max(1));
            self.save_view_configs();
        }
    }

    fn current_view_label_widths(&self) -> Vec<u32> {
        if let Some(cfg) = self.active_view_config() {
            return Self::normalize_label_widths(cfg.label_widths.clone(), cfg.columns.len().max(1));
        }
        Self::normalize_label_widths(vec![140; self.view_layout_working.len().max(1)], self.view_layout_working.len().max(1))
    }

    fn set_active_view_label_widths(&mut self, widths: Vec<u32>) {
        if let Some(cfg) = self.view_configs.iter_mut().find(|c| c.name == self.view_mode) {
            cfg.label_widths = Self::normalize_label_widths(widths, cfg.columns.len().max(1));
            self.save_view_configs();
        }
    }

    fn save_column_width_prefs(&self) {
        let key = format!("columnWidthPrefs:{}", if self.current_index.is_empty() { "default" } else { &self.current_index });
        if let Ok(value) = serde_json::to_string(&self.column_widths) {
            storage_set(&key, &value);
        }
    }

    fn load_field_labels(&mut self) {
        self.field_labels = load_field_labels(Some(&self.current_index));
    }

    fn apply_column_prefs(&self, columns: Vec<String>) -> Vec<String> {
        let ordered = self.order_columns(columns);
        if self.hidden_columns.is_empty() {
            return ordered;
        }
        ordered
            .into_iter()
            .filter(|col| !self.hidden_columns.contains(col))
            .collect()
    }

    fn order_columns(&self, columns: Vec<String>) -> Vec<String> {
        if self.column_order.is_empty() {
            return columns;
        }
        let mut ordered = vec![];
        for col in &self.column_order {
            if columns.contains(col) {
                ordered.push(col.clone());
            }
        }
        for col in columns {
            if !ordered.contains(&col) {
                ordered.push(col);
            }
        }
        ordered
    }

    fn move_column_order(&mut self, from: &str, to: &str) {
        let base = if !self.last_base_columns.is_empty() {
            self.last_base_columns.clone()
        } else {
            vec![]
        };
        let mut order = if !self.column_order.is_empty() {
            self.column_order
                .iter()
                .cloned()
                .filter(|c| base.contains(c))
                .collect::<Vec<_>>()
        } else {
            base
        };
        let from_idx = order.iter().position(|c| c == from);
        let to_idx = order.iter().position(|c| c == to);
        if let (Some(from_idx), Some(to_idx)) = (from_idx, to_idx) {
            let item = order.remove(from_idx);
            order.insert(to_idx, item);
            self.column_order = order;
            self.save_column_prefs();
        }
    }

    fn refresh_results_table(&mut self) {
        if let Some(res) = self.last_results.clone() {
            self.apply_results_columns(&res);
        }
    }

    fn set_column_width(&mut self, col: &str, width: u32) {
        self.column_widths.insert(col.to_string(), width);
    }

    fn bind_resize_listeners(&mut self, ctx: &Context<Self>) {
        let window = web_window();
        let link = ctx.link().clone();
        let move_listener = EventListener::new(&window, "mousemove", move |event| {
            if let Some(event) = event.dyn_ref::<web_sys::MouseEvent>() {
                link.send_message(Msg::ResizeMove(event.client_x()));
            }
        });
        let link = ctx.link().clone();
        let up_listener = EventListener::new(&window, "mouseup", move |_| {
            link.send_message(Msg::EndResize);
        });
        self.resize_move_listener = Some(move_listener);
        self.resize_up_listener = Some(up_listener);
    }

    fn schedule_debounced_search(&mut self, ctx: &Context<Self>, delay_ms: u32) {
        if let Some(timeout) = self.debounce.take() {
            timeout.cancel();
        }
        let link = ctx.link().clone();
        self.debounce = Some(Timeout::new(delay_ms, move || {
            link.send_message(Msg::DebouncedSearch);
        }));
    }

    fn push_toast(&mut self, message: String, kind: ToastType, ctx: &Context<Self>) {
        let id = self.toast_seq;
        self.toast_seq += 1;
        self.toasts.push(Toast { id, message, kind });
        let link = ctx.link().clone();
        let timeout = Timeout::new(3000, move || {
            link.send_message(Msg::DismissToast(id));
        });
        timeout.forget();
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

fn web_window() -> Window {
    web_sys::window().expect("no window")
}

fn web_document() -> Document {
    web_window().document().expect("no document")
}

fn storage_get(key: &str) -> Option<String> {
    web_window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|storage| storage.get_item(key).ok().flatten())
}

fn storage_set(key: &str, value: &str) {
    if let Ok(Some(storage)) = web_window().local_storage() {
        let _ = storage.set_item(key, value);
    }
}

fn storage_remove(key: &str) {
    if let Ok(Some(storage)) = web_window().local_storage() {
        let _ = storage.remove_item(key);
    }
}

fn load_search_history() -> Vec<HistoryItem> {
    storage_get("searchHistory")
        .and_then(|value| serde_json::from_str::<Vec<HistoryItem>>(&value).ok())
        .unwrap_or_default()
}

fn save_search_history(history: &[HistoryItem]) {
    if let Ok(value) = serde_json::to_string(history) {
        storage_set("searchHistory", &value);
    }
}

fn load_field_labels(index: Option<&str>) -> HashMap<String, String> {
    let key = format!("fieldLabels:{}", index.unwrap_or("default"));
    storage_get(&key)
        .and_then(|value| serde_json::from_str::<HashMap<String, String>>(&value).ok())
        .unwrap_or_default()
}

fn save_field_labels(labels: &HashMap<String, String>, index: Option<&str>) {
    let key = format!("fieldLabels:{}", index.unwrap_or("default"));
    if let Ok(value) = serde_json::to_string(labels) {
        storage_set(&key, &value);
    }
}

fn load_popular_field(index: Option<&str>) -> String {
    let key = format!("popularSearchField:{}", index.unwrap_or("default"));
    storage_get(&key).unwrap_or_default()
}

fn save_popular_field(field: &str, index: Option<&str>) {
    let key = format!("popularSearchField:{}", index.unwrap_or("default"));
    if field.is_empty() {
        storage_remove(&key);
    } else {
        storage_set(&key, field);
    }
}

fn load_ai_config() -> AiConfig {
    let defaults = AiConfig {
        ai_weight: 40,
        ai_enabled: true,
    };
    if let Some(value) = storage_get("aiSearchConfig") {
        if let Ok(mut cfg) = serde_json::from_str::<AiConfig>(&value) {
            if cfg.ai_weight > 100 {
                cfg.ai_weight = 100;
            }
            return cfg;
        }
    }
    defaults
}

fn save_ai_config(config: &AiConfig) {
    if let Ok(value) = serde_json::to_string(config) {
        storage_set("aiSearchConfig", &value);
    }
}

fn get_theme() -> String {
    storage_get("theme").unwrap_or_else(|| "dark".to_string())
}

fn set_theme(theme: &str) {
    let next = if theme == "light" { "light" } else { "dark" };
    if let Some(el) = web_document().document_element() {
        let _ = el.set_attribute("data-theme", next);
    }
    storage_set("theme", next);
}

fn set_all_checkboxes(selector: &str, checked: bool) {
    if let Ok(nodes) = web_document().query_selector_all(selector) {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
                    input.set_checked(checked);
                }
            }
        }
    }
}

fn build_filter_expression_from_dom() -> Option<String> {
    let Ok(rows) = web_document().query_selector_all(".query-row") else { return None };
    let mut conditions: Vec<(String, String)> = vec![];
    for i in 0..rows.length() {
        let Some(node) = rows.item(i) else { continue };
        let Some(row) = node.dyn_ref::<Element>() else { continue };
        let field = row
            .query_selector(".query-field")
            .ok()
            .flatten()
            .and_then(|e| e.dyn_into::<web_sys::HtmlSelectElement>().ok())
            .map(|s| s.value())
            .unwrap_or_default();
        if field.trim().is_empty() {
            continue;
        }
        let operator = row
            .query_selector(".query-operator")
            .ok()
            .flatten()
            .and_then(|e| e.dyn_into::<web_sys::HtmlSelectElement>().ok())
            .map(|s| s.value())
            .unwrap_or_else(|| "=".to_string());
        let value = row
            .query_selector(".query-value")
            .ok()
            .flatten()
            .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|s| s.value())
            .unwrap_or_default();
        let logic = row
            .query_selector(".query-logic")
            .ok()
            .flatten()
            .and_then(|e| e.dyn_into::<web_sys::HtmlSelectElement>().ok())
            .map(|s| s.value())
            .unwrap_or_else(|| "AND".to_string());

        let operator = normalize_operator(&operator);
        let filter_condition = match operator.as_str() {
            "IN" => {
                let items: Vec<String> = value
                    .split(',')
                    .map(|v| v.trim())
                    .filter(|v| !v.is_empty())
                    .map(format_filter_value)
                    .collect();
                if items.is_empty() {
                    continue;
                }
                format!("{} IN [{}]", field, items.join(", "))
            }
            "NOT IN" => {
                let items: Vec<String> = value
                    .split(',')
                    .map(|v| v.trim())
                    .filter(|v| !v.is_empty())
                    .map(format_filter_value)
                    .collect();
                if items.is_empty() {
                    continue;
                }
                format!("{} NOT IN [{}]", field, items.join(", "))
            }
            "EXISTS" => format!("{} EXISTS", field),
            "NOT EXISTS" => format!("{} NOT EXISTS", field),
            _ => {
                if value.trim().is_empty() {
                    continue;
                }
                format!("{} {} {}", field, operator, format_filter_value(&value))
            }
        };

        conditions.push((filter_condition, logic));
    }

    build_filter_expression_from_conditions(&conditions)
}

fn build_filter_expression_from_conditions(conditions: &[(String, String)]) -> Option<String> {
    if conditions.is_empty() {
        return None;
    }
    let mut expr = conditions[0].0.clone();
    for i in 1..conditions.len() {
        let logic = conditions[i].1.to_uppercase();
        let joiner = if logic == "OR" { " OR " } else { " AND " };
        expr = format!("{}{}{}", expr, joiner, conditions[i].0);
    }
    Some(expr)
}

fn collect_hidden_columns() -> Vec<String> {
    let mut hidden = vec![];
    if let Ok(inputs) = web_document().query_selector_all(".column-config-item input[type=\"checkbox\"]") {
        for i in 0..inputs.length() {
            if let Some(node) = inputs.item(i) {
                if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
                    if !input.checked() {
                        if let Some(col) = input.get_attribute("data-col") {
                            hidden.push(col);
                        }
                    }
                }
            }
        }
    }
    hidden
}

fn collect_column_labels() -> HashMap<String, String> {
    let mut labels = HashMap::new();
    if let Ok(nodes) = web_document().query_selector_all(".column-label-input") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
                    if let Some(col) = input.get_attribute("data-col") {
                        labels.insert(col, input.value());
                    }
                }
            }
        }
    }
    labels
}

fn collect_field_config() -> Vec<FieldConfigItem> {
    let document = web_document();
    let mut items = vec![];
    let nodes = match document.query_selector_all(".field-config") {
        Ok(nodes) => nodes,
        Err(_) => return items,
    };
    for i in 0..nodes.length() {
        if let Some(node) = nodes.item(i) {
            let element = node.dyn_into::<Element>().ok();
            if let Some(el) = element {
                let field = el
                    .query_selector(".searchable-check")
                    .ok()
                    .flatten()
                    .and_then(|e| e.get_attribute("data-field"))
                    .unwrap_or_default();
                if field.is_empty() {
                    continue;
                }
                let searchable = el
                    .query_selector(".searchable-check")
                    .ok()
                    .flatten()
                    .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                    .map(|i| i.checked())
                    .unwrap_or(false);
                let highlight = el
                    .query_selector(".highlight-check")
                    .ok()
                    .flatten()
                    .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                    .map(|i| i.checked())
                    .unwrap_or(false);
                let display = el
                    .query_selector(".display-check")
                    .ok()
                    .flatten()
                    .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                    .map(|i| i.checked())
                    .unwrap_or(false);
                let weight = el
                    .query_selector(".field-weight")
                    .ok()
                    .flatten()
                    .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                    .and_then(|i| i.value().parse::<f64>().ok())
                    .unwrap_or(1.0);
                let label = el
                    .query_selector(".field-label")
                    .ok()
                    .flatten()
                    .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                    .map(|i| i.value())
                    .unwrap_or_else(|| field.clone());
                items.push(FieldConfigItem {
                    field,
                    searchable,
                    weight,
                    label,
                    highlight,
                    display,
                });
            }
        }
    }
    items
}

fn input_value(event: yew::events::InputEvent) -> String {
    let input: web_sys::HtmlInputElement = event.target_unchecked_into();
    input.value()
}

fn select_value(event: yew::events::Event) -> String {
    let select: web_sys::HtmlSelectElement = event.target_unchecked_into();
    select.value()
}

fn checkbox_checked(event: yew::events::Event) -> bool {
    let input: web_sys::HtmlInputElement = event.target_unchecked_into();
    input.checked()
}

fn format_number(value: u64) -> String {
    let mut s = value.to_string();
    let mut parts = vec![];
    while s.len() > 3 {
        let tail = s.split_off(s.len() - 3);
        parts.push(tail);
    }
    parts.push(s);
    parts.reverse();
    parts.join(",")
}

fn format_time(timestamp: &str) -> String {
    let now = js_sys::Date::new_0().get_time();
    let then = js_sys::Date::new(&JsValue::from_str(timestamp)).get_time();
    if !then.is_finite() {
        return timestamp.to_string();
    }
    let diff = now - then;
    if diff < 60_000.0 {
        return "刚刚".to_string();
    }
    if diff < 3_600_000.0 {
        return format!("{}分钟前", (diff / 60_000.0).floor() as i64);
    }
    if diff < 86_400_000.0 {
        return format!("{}小时前", (diff / 3_600_000.0).floor() as i64);
    }
    js_sys::Date::new(&JsValue::from_f64(then))
        .to_locale_date_string("zh-CN", &JsValue::undefined())
        .into()
}

fn hit_has_id(hit: &SearchHit) -> bool {
    hit.id.is_some() || hit.fields.get("id").is_some()
}

fn get_id_string(hit: &SearchHit) -> String {
    hit.id
        .as_ref()
        .or_else(|| hit.fields.get("id"))
        .and_then(|v| {
            if v.is_string() {
                v.as_str().map(|s| s.to_string())
            } else if v.is_number() || v.is_boolean() {
                Some(v.to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn get_doc_key(hit: &SearchHit, primary_key: &str) -> String {
    if !primary_key.trim().is_empty() {
        if let Some(val) = hit.get(primary_key) {
            return value_to_string(val);
        }
    }
    get_id_string(hit)
}

fn hit_entries(hit: &SearchHit) -> Vec<(String, Value)> {
    let mut entries: Vec<(String, Value)> = hit
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    if let Some(id) = &hit.id {
        entries.push(("id".to_string(), id.clone()));
    }
    entries
}

impl SearchHit {
    fn get(&self, key: &str) -> Option<&Value> {
        if key == "id" {
            return self.id.as_ref();
        }
        self.fields.get(key)
    }
}

#[derive(Clone)]
struct CellValue {
    html: Html,
    title: AttrValue,
}

fn get_cell_value(hit: &SearchHit, col: &str, use_highlight: bool) -> Option<CellValue> {
    let raw = if use_highlight {
        hit.formatted
            .as_ref()
            .and_then(|f| f.get(col))
            .cloned()
            .or_else(|| hit.get(col).cloned())
    } else {
        hit.get(col).cloned()
    }?;

    let (display_html, title_text) = if raw.is_array() {
        let items = raw
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| value_to_string(v))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        (items.join(", "), items.join(", "))
    } else if raw.is_object() {
        let json = serde_json::to_string(&raw).unwrap_or_default();
        (escape_html(&json), json)
    } else {
        let text = raw.as_str().map(|s| s.to_string()).unwrap_or_else(|| raw.to_string());
        if use_highlight {
            (text.clone(), strip_html(&text))
        } else {
            (escape_html(&text), text)
        }
    };

    let html = if use_highlight {
        Html::from_html_unchecked(AttrValue::from(display_html))
    } else {
        html! { display_html.clone() }
    };

    Some(CellValue {
        html,
        title: AttrValue::from(escape_html(&title_text)),
    })
}

fn value_to_string(value: &Value) -> String {
    if let Some(s) = value.as_str() {
        s.to_string()
    } else if value.is_object() {
        serde_json::to_string(value).unwrap_or_default()
    } else {
        value.to_string()
    }
}

fn value_to_string_for_edit(value: &Value) -> String {
    if value.is_null() {
        return "".to_string();
    }
    if value.is_object() || value.is_array() {
        return serde_json::to_string(value).unwrap_or_default();
    }
    value_to_string(value)
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn strip_html(input: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for ch in input.chars() {
        match ch {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(ch),
            _ => {}
        }
    }
    out
}

fn format_filter_value(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "\"\"".to_string();
    }
    if trimmed.eq_ignore_ascii_case("true") || trimmed.eq_ignore_ascii_case("false") {
        return trimmed.to_lowercase();
    }
    if trimmed.parse::<f64>().is_ok() {
        return trimmed.to_string();
    }
    format!("\"{}\"", trimmed.replace('\"', "\\\""))
}

fn normalize_operator(input: &str) -> String {
    let op = input.trim();
    let allowed = [
        "=", "!=", ">", "<", ">=", "<=", "IN", "NOT IN", "EXISTS", "NOT EXISTS",
    ];
    if allowed.iter().any(|v| *v == op) {
        op.to_string()
    } else {
        "=".to_string()
    }
}

fn sort_hits(mut hits: Vec<SearchHit>, field: &str, dir: &str) -> Vec<SearchHit> {
    let factor = if dir == "desc" { -1.0 } else { 1.0 };
    hits.sort_by(|a, b| {
        let av = normalize_sort_value(a.get(field));
        let bv = normalize_sort_value(b.get(field));
        if av.is_number && bv.is_number {
            return av.num_value.partial_cmp(&bv.num_value).unwrap_or(std::cmp::Ordering::Equal);
        }
        av.str_value.cmp(&bv.str_value)
    });
    if factor < 0.0 {
        hits.reverse();
    }
    hits
}

struct SortValue {
    is_number: bool,
    num_value: f64,
    str_value: String,
}

fn apply_auth_header(mut builder: gloo_net::http::RequestBuilder, api_key: &str) -> gloo_net::http::RequestBuilder {
    let key = api_key.trim();
    if key.is_empty() {
        return builder;
    }
    builder = builder.header("Authorization", &format!("Bearer {}", key));
    builder.header("X-Meili-API-Key", key)
}

fn normalize_sort_value(value: Option<&Value>) -> SortValue {
    if value.is_none() {
        return SortValue { is_number: false, num_value: 0.0, str_value: "".to_string() };
    }
    let value = value.unwrap();
    if let Some(n) = value.as_f64() {
        return SortValue { is_number: true, num_value: n, str_value: n.to_string() };
    }
    if value.is_array() {
        if let Some(first) = value.as_array().and_then(|arr| arr.iter().find(|v| !v.is_null())) {
            return normalize_sort_value(Some(first));
        }
    }
    if value.is_object() {
        return SortValue { is_number: false, num_value: 0.0, str_value: serde_json::to_string(value).unwrap_or_default() };
    }
    SortValue { is_number: false, num_value: 0.0, str_value: value_to_string(value) }
}

async fn connect_indexes(host: &str, api_key: &str) -> Result<ConnectData, String> {
    let url = format!("{}/indexes", host.trim_end_matches('/'));
    let builder = Request::get(&url);
    let response = apply_auth_header(builder, api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        let err = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
        return Err(err);
    }
    let data: IndexListResponse = response.json().await.map_err(|e| e.to_string())?;
    let mut indexes = vec![];
    for idx in data.results {
        let count = get_index_stats(host, api_key, &idx.uid).await.ok().and_then(|s| s.number_of_documents);
        indexes.push(IndexInfo { uid: idx.uid, count });
    }
    Ok(ConnectData { indexes })
}

async fn get_index_stats(host: &str, api_key: &str, uid: &str) -> Result<IndexStats, String> {
    let url = format!("{}/indexes/{}/stats", host.trim_end_matches('/'), uid);
    let builder = Request::get(&url);
    let response = apply_auth_header(builder, api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(response.text().await.unwrap_or_else(|_| "stats error".to_string()));
    }
    response.json().await.map_err(|e| e.to_string())
}

async fn load_index_data(host: &str, api_key: &str, uid: &str) -> Result<IndexData, String> {
    let settings_url = format!("{}/indexes/{}/settings", host.trim_end_matches('/'), uid);
    let builder = Request::get(&settings_url);
    let settings_resp = apply_auth_header(builder, api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !settings_resp.ok() {
        return Err(settings_resp.text().await.unwrap_or_else(|_| "settings error".to_string()));
    }
    let settings: IndexSettings = settings_resp.json().await.map_err(|e| e.to_string())?;

    let docs_url = format!("{}/indexes/{}/documents?limit=1", host.trim_end_matches('/'), uid);
    let docs_builder = Request::get(&docs_url);
    let docs_resp = apply_auth_header(docs_builder, api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let sample_doc = if docs_resp.ok() {
        let docs: DocumentsResponse = docs_resp.json().await.map_err(|e| e.to_string())?;
        docs.results.get(0).cloned().unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };

    let mut searchable = settings.searchable_attributes.unwrap_or_default();
    let mut displayed = settings.displayed_attributes.unwrap_or_default();
    let mut filterable = settings.filterable_attributes.unwrap_or_default();
    let mut sortable = settings.sortable_attributes.unwrap_or_default();

    searchable.retain(|f| f != "*");
    displayed.retain(|f| f != "*");
    filterable.retain(|f| f != "*");
    sortable.retain(|f| f != "*");

    let mut available: Vec<String> = vec![];
    let mut set = HashSet::new();
    for f in searchable.iter().chain(displayed.iter()) {
        if set.insert(f.clone()) {
            available.push(f.clone());
        }
    }
    if let Some(map) = sample_doc.as_object() {
        for key in map.keys() {
            if key == "*" {
                continue;
            }
            if set.insert(key.clone()) {
                available.push(key.clone());
            }
        }
    }

    Ok(IndexData {
        available_fields: available,
        search_fields: searchable.clone(),
        highlight_fields: displayed.clone(),
        display_fields: displayed,
        filterable_fields: filterable,
        sortable_attributes: sortable,
    })
}

async fn perform_search(host: &str, api_key: &str, index: &str, query: &str, mut params: Value) -> Result<SearchResponse, String> {
    let url = format!("{}/indexes/{}/search", host.trim_end_matches('/'), index);
    params["q"] = json!(query);
    let builder = Request::post(&url);
    let req = apply_auth_header(builder, api_key).json(&params).map_err(|e| e.to_string())?;
    let response = req.send().await.map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(response.text().await.unwrap_or_else(|_| "search error".to_string()));
    }
    response.json().await.map_err(|e| e.to_string())
}

async fn update_index_settings(host: &str, api_key: &str, index: &str, searchable: &[String], filterable: &[String]) -> Result<(), String> {
    let url = format!("{}/indexes/{}/settings", host.trim_end_matches('/'), index);
    let body = json!({
        "searchableAttributes": searchable,
        "filterableAttributes": filterable
    });
    let builder = Request::patch(&url);
    let req = apply_auth_header(builder, api_key).json(&body).map_err(|e| e.to_string())?;
    let response = req.send().await.map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(response.text().await.unwrap_or_else(|_| "update settings error".to_string()));
    }
    Ok(())
}

async fn load_popular_searches(host: &str, api_key: &str, index: &str, field: &str) -> Result<PopularSearchData, String> {
    let url = format!("{}/indexes/{}/search", host.trim_end_matches('/'), index);
    let body = json!({
        "q": "",
        "limit": 0,
        "facets": [field]
    });
    let builder = Request::post(&url);
    let req = apply_auth_header(builder, api_key).json(&body).map_err(|e| e.to_string())?;
    let response = req.send().await.map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(response.text().await.unwrap_or_else(|_| "popular search error".to_string()));
    }
    let res: SearchResponse = response.json().await.map_err(|e| e.to_string())?;
    let mut items: Vec<PopularItem> = vec![];
    if let Some(map) = res.facet_distribution {
        if let Some(values) = map.get(field) {
            let mut entries: Vec<(String, u64)> = values.iter().map(|(v, c)| (v.clone(), *c)).collect();
            entries.sort_by(|a, b| b.1.cmp(&a.1));
            for (val, count) in entries.into_iter().take(10) {
                items.push(PopularItem { value: val, count: Some(count) });
            }
        }
    }
    Ok(PopularSearchData { items })
}

async fn fetch_by_id(host: &str, api_key: &str, index: &str, id: &str) -> Result<SearchHit, String> {
    let url = format!("{}/indexes/{}/search", host.trim_end_matches('/'), index);
    let safe_id = if id.parse::<f64>().is_ok() { id.to_string() } else { format!("\"{}\"", id.replace('"', "\\\"")) };
    let body = json!({
        "q": "",
        "filter": [format!("id = {}", safe_id)]
    });
    let builder = Request::post(&url);
    let req = apply_auth_header(builder, api_key).json(&body).map_err(|e| e.to_string())?;
    let response = req.send().await.map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(response.text().await.unwrap_or_else(|_| "fetch by id error".to_string()));
    }
    let res: SearchResponse = response.json().await.map_err(|e| e.to_string())?;
    res.hits.into_iter().next().ok_or_else(|| "not found".to_string())
}

async fn update_documents(
    host: &str,
    api_key: &str,
    index: &str,
    primary_key: &str,
    hits: &[SearchHit],
    edits: &HashMap<String, HashMap<String, Value>>,
) -> Result<(), String> {
    let mut docs: Vec<Value> = vec![];
    for hit in hits {
        let doc_id = get_doc_key(hit, primary_key);
        if doc_id.is_empty() {
            continue;
        }
        let Some(fields) = edits.get(&doc_id) else { continue };
        let mut obj = serde_json::Map::new();
        for (k, v) in hit.fields.iter() {
            obj.insert(k.clone(), v.clone());
        }
        if let Some(id) = &hit.id {
            obj.insert("id".to_string(), id.clone());
        }
        if !primary_key.trim().is_empty() {
            if let Some(pk_val) = hit.get(primary_key) {
                obj.insert(primary_key.to_string(), pk_val.clone());
            }
        }
        for (field, value) in fields.iter() {
            let parsed = parse_edit_value(value);
            obj.insert(field.clone(), parsed);
        }
        docs.push(Value::Object(obj));
    }
    if docs.is_empty() {
        return Ok(());
    }

    let url = format!("{}/indexes/{}/documents", host.trim_end_matches('/'), index);
    let builder = Request::post(&url);
    let req = apply_auth_header(builder, api_key).json(&docs).map_err(|e| e.to_string())?;
    let response = req.send().await.map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(response.text().await.unwrap_or_else(|_| "update documents error".to_string()));
    }
    Ok(())
}

fn parse_edit_value(value: &Value) -> Value {
    match value {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Value::String("".to_string());
            }
            if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
                return parsed;
            }
            if trimmed.eq_ignore_ascii_case("true") {
                return Value::Bool(true);
            }
            if trimmed.eq_ignore_ascii_case("false") {
                return Value::Bool(false);
            }
            if let Ok(num) = trimmed.parse::<f64>() {
                return Value::Number(serde_json::Number::from_f64(num).unwrap_or_else(|| serde_json::Number::from(0)));
            }
            Value::String(s.clone())
        }
        other => other.clone(),
    }
}
