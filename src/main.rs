#![allow(unused)]
use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, HtmlElement};
use yew::events::{DragEvent, MouseEvent};
use yew::{html, Callback, Component, Context, Html};
use yew::TargetCast;

mod types;
mod storage;
mod utils;
mod cell;
mod sort;
mod api;
mod parse;
mod query_editor;
mod results_table;
mod export;
mod components;

pub use types::*;
pub use storage::*;
pub use utils::*;
pub use cell::*;
pub use sort::*;
pub use api::*;
pub use parse::*;

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
    max_results_per_page: u32,
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
    new_index_uid: String,
    new_index_pk: String,
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
    view_drag_over_index: Option<usize>,
    export_downloading: bool,
    export_progress: u64,
    export_total: u64,
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
    image_preview_enabled: bool,
    image_preview_links_only: bool,
    image_preview_size: u32,
    current_tab: String,
    asset_form: DeviceAsset,
    asset_list: Vec<DeviceAsset>,
    asset_modal_open: bool,
    asset_detail: Option<DeviceAsset>,
    assets_loading: bool,
    upload_modal_open: bool,
    upload_data: String,
    file_input_listener: Option<EventListener>,
    upload_file_name: String,
    upload_preview_data: Vec<serde_json::Value>,
    upload_preview_page: usize,
    upload_preview_page_size: usize,
    upload_progress: f32,
    upload_loading: bool,
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
            max_results_per_page: 1000,
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
            view_drag_over_index: None,
            export_downloading: false,
            export_progress: 0,
            export_total: 0,
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
            image_preview_enabled: false,
            image_preview_links_only: false,
            image_preview_size: 80,
            new_index_uid: String::new(),
            new_index_pk: String::new(),
            current_tab: "search".to_string(),
            asset_form: DeviceAsset::new(String::new(), String::new()),
            asset_list: vec![],
            asset_modal_open: false,
            asset_detail: None,
            assets_loading: false,
            upload_modal_open: false,
            upload_data: String::new(),
            file_input_listener: None,
            upload_file_name: String::new(),
            upload_preview_data: vec![],
            upload_preview_page: 1,
            upload_preview_page_size: 10,
            upload_progress: 0.0,
            upload_loading: false,
        };
        // Load image preview settings from storage
        app.image_preview_enabled = storage::load_image_preview_enabled();
        app.image_preview_links_only = storage::load_image_preview_links_only();
        app.image_preview_size = storage::load_image_preview_size();

        query_editor::add_query_row(&mut app);
        app.initialize_theme();
        // Load image preview setting
        // We'll call storage function later in init phase after App is constructed
        query_editor::init();
        results_table::init();
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
                        query_editor::refresh_query_rows(self);
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
                query_editor::add_query_row(self);
                true
            }
            Msg::RemoveQueryRow(id) => {
                query_editor::sync_query_rows_from_dom(self);
                self.query_rows.retain(|row| row.id != id);
                if self.query_rows.is_empty() {
                    query_editor::add_query_row(self);
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
                query_editor::add_query_row(self);
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
                query_editor::refresh_query_rows(self);

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
            Msg::ViewFieldDragOver(_column_idx, item_index) => {
                self.view_drag_over_index = Some(item_index);
                true
            }
            Msg::DropViewField(column_idx) => {
                if let Some(field) = self.view_drag_field.clone() {
                    self.view_drag_field = None;
                    self.view_drag_from_column = None;
                    // Remove field from all columns first
                    for col in self.view_layout_working.iter_mut() {
                        col.retain(|f| f != &field);
                    }
                    if column_idx < self.view_layout_working.len() {
                        let insert_index = self.view_drag_over_index
                            .unwrap_or(self.view_layout_working[column_idx].len())
                            .min(self.view_layout_working[column_idx].len());
                        self.view_layout_working[column_idx].insert(insert_index, field);
                    }
                    self.view_drag_over_index = None;
                    return true;
                }
                false
            }
            Msg::DropViewFieldToPool => {
                if let Some(field) = self.view_drag_field.clone() {
                    self.view_drag_field = None;
                    self.view_drag_from_column = None;
                    self.view_drag_over_index = None;
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
                query_editor::add_query_row(self);
                if let Some(row) = self.query_rows.first_mut() {
                    row.field = self.popular_search_field.clone();
                    row.operator = "=".to_string();
                    row.value = value;
                }
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::ExportCsv => {
                if self.export_downloading {
                    return true;
                }
                self.export_downloading = true;
                self.export_progress = 0;
                self.export_total = 0;
                let host = self.host_input.trim().to_string();
                let api_key = self.api_key_input.trim().to_string();
                let params = self.build_search_params_for_export();
                let link = ctx.link().clone();
                spawn_local(async move {
                    match export::export_all_csv(&host, &api_key, &params, |progress, total| {
                        link.send_message(Msg::ExportCsvProgress(progress, total));
                    }).await {
                        Ok(filename) => {
                            link.send_message(Msg::ExportCsvFinished(Ok(filename)));
                        }
                        Err(e) => {
                            link.send_message(Msg::ExportCsvFinished(Err(e)));
                        }
                    }
                });
                self.push_toast("开始导出 CSV...".to_string(), crate::ToastType::Warning, ctx);
                true
            }
            Msg::ExportCsvProgress(progress, total) => {
                self.export_progress = progress;
                self.export_total = total;
                true
            }
            Msg::ExportCsvFinished(Ok(filename)) => {
                self.export_downloading = false;
                self.export_progress = 0;
                self.export_total = 0;
                self.push_toast(format!("CSV 文件已导出: {}", filename), crate::ToastType::Success, ctx);
                true
            }
            Msg::ExportCsvFinished(Err(e)) => {
                self.export_downloading = false;
                self.export_progress = 0;
                self.export_total = 0;
                self.push_toast(format!("导出失败: {}", e), crate::ToastType::Error, ctx);
                true
            }
            Msg::SetImagePreview(enabled) => {
                self.image_preview_enabled = enabled;
                // When enabling previews, show links by default (not thumbnails)
                self.image_preview_links_only = false;
                save_image_preview_enabled(enabled);
                save_image_preview_links_only(false);
                true
            }
            Msg::SetImagePreviewLinksOnly(links_only) => {
                self.image_preview_links_only = links_only;
                save_image_preview_links_only(links_only);
                true
            }
            Msg::SetImagePreviewSize(size) => {
                self.image_preview_size = size;
                save_image_preview_size(size);
                true
            }
            Msg::SetCurrentTab(tab) => {
                self.current_tab = tab;
                if self.current_tab == "assets" {
                    self.asset_list = load_device_assets();
                }
                true
            }
            Msg::OpenAssetModal(open) => {
                self.asset_modal_open = open;
                if !open {
                    let id = js_sys::Math::random().to_string();
                    self.asset_form = DeviceAsset::new(id, String::new());
                }
                true
            }
            Msg::SetAssetForm(asset) => {
                self.asset_form = asset;
                self.asset_modal_open = true;
                true
            }
            Msg::SaveAsset => {
                let mut assets = load_device_assets();
                if let Some(pos) = assets.iter().position(|a| a.id == self.asset_form.id) {
                    assets[pos] = self.asset_form.clone();
                } else {
                    assets.push(self.asset_form.clone());
                }
                save_device_assets(&assets);
                self.asset_list = assets;
                self.asset_modal_open = false;
                self.push_toast("设备保存成功".to_string(), ToastType::Success, ctx);
                true
            }
            Msg::DeleteAsset(id) => {
                let mut assets = load_device_assets();
                assets.retain(|a| a.id != id);
                save_device_assets(&assets);
                self.asset_list = assets;
                self.push_toast("设备已删除".to_string(), ToastType::Success, ctx);
                true
            }
            Msg::OpenAssetDetail(asset) => {
                self.asset_detail = Some(asset);
                true
            }
            Msg::CloseAssetDetail => {
                self.asset_detail = None;
                true
            }
            Msg::ImportAssets(json_str) => {
                if json_str.trim().is_empty() {
                    true
                } else {
                    // Validate that all items contain the primary key field
                    let pk = self.primary_key_field.clone();
                    if pk.is_empty() {
                        self.push_toast("请先设置主键字段后再导入数据".to_string(), ToastType::Error, ctx);
                        return true;
                    }
                    let items: Vec<Value> = match serde_json::from_str::<Vec<Value>>(&json_str) {
                        Ok(v) => v,
                        Err(e) => {
                            self.push_toast(format!("导入失败: {}", e), ToastType::Error, ctx);
                            return true;
                        }
                    };
                    if items.iter().any(|v| v.get(&pk).is_none()) {
                        self.push_toast(format!("导入失败：每个对象都必须包含主键字段 '{}'", pk), ToastType::Error, ctx);
                        return true;
                    }
                    // Deserialize into DeviceAsset, assuming fields exist
                    let mut assets: Vec<DeviceAsset> = Vec::new();
                    for v in items {
                        match serde_json::from_value::<DeviceAsset>(v) {
                            Ok(a) => assets.push(a),
                            Err(_) => {
                                // skip invalid entries but warn
                                self.push_toast("导入数据包含无效条目，已跳过".to_string(), ToastType::Warning, ctx);
                            }
                        }
                    }
                    if assets.is_empty() {
                        self.push_toast("导入失败：无有效设备数据".to_string(), ToastType::Error, ctx);
                        return true;
                    }
                    let mut existing = load_device_assets();
                    for asset in assets {
                        if !existing.iter().any(|a| a.id == asset.id) {
                            existing.push(asset);
                        }
                    }
                    save_device_assets(&existing);
                    self.asset_list = existing;
                    self.push_toast("导入成功".to_string(), ToastType::Success, ctx);
                    true
                }
            }
            Msg::ExportAssets => {
                let assets = load_device_assets();
                match serde_json::to_string_pretty(&assets) {
                    Ok(json) => {
                        let window = web_sys::window().unwrap();
                        let document = window.document().unwrap();
                        let arr = js_sys::Array::new();
                        arr.push(&wasm_bindgen::JsValue::from_str(&json));
                        let blob = web_sys::Blob::new_with_str_sequence(&arr).unwrap();
                        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
                        if let Some(a) = document.create_element("a").ok() {
                            let _ = a.set_attribute("href", &url);
                            let _ = a.set_attribute("download", "device_assets.json");
                            let _ = a.dyn_ref::<web_sys::HtmlElement>().map(|el| el.click());
                        }
                        self.push_toast("导出成功".to_string(), ToastType::Success, ctx);
                    }
                    Err(e) => {
                        self.push_toast(format!("导出失败: {}", e), ToastType::Error, ctx);
                    }
                }
                true
            }
            Msg::TriggerFileImport(_) => {
                self.push_toast("请在下方文本框中粘贴JSON数据导入，或先导出模板".to_string(), ToastType::Info, ctx);
                true
            }
            Msg::FileImportTriggered(_) => {
                true
            }
            Msg::SetNewIndexUid(uid) => {
                self.new_index_uid = uid;
                true
            }
            Msg::SetNewIndexPk(pk) => {
                self.new_index_pk = pk;
                true
            }
            Msg::CreateIndexLocal => {
                if self.new_index_uid.trim().is_empty() {
                    self.push_toast("请提供索引 UID".to_string(), ToastType::Error, ctx);
                    return true;
                }
                
                let host = self.host_input.trim().to_string();
                let api_key = self.api_key_input.trim().to_string();
                let uid = self.new_index_uid.clone();
                let primary_key = if self.new_index_pk.trim().is_empty() { None } else { Some(self.new_index_pk.clone()) };
                
                self.loading = true;
                let link = ctx.link().clone();
                spawn_local(async move {
                    // Create index with optional primary key
                    let res = create_index(&host, &api_key, &uid, primary_key.as_ref().map(|s| s.as_str())).await;
                    link.send_message(Msg::IndexCreated(res));
                });
                true
            }
            Msg::IndexCreated(result) => {
                self.loading = false;
                match result {
                    Ok(_) => {
                        self.push_toast("索引创建成功".to_string(), ToastType::Success, ctx);
                        // If a primary key was specified during creation, set it
                        if !self.new_index_pk.trim().is_empty() {
                            self.primary_key_field = self.new_index_pk.clone();
                        }
                        // Clear the input fields
                        self.new_index_uid.clear();
                        self.new_index_pk.clear();
                    }
                    Err(e) => {
                        self.push_toast(format!("创建索引失败: {}", e), ToastType::Error, ctx);
                    }
                }
                true
            }
            Msg::UpdateMaxResults(value) => {
                let v = value.parse::<u32>().unwrap_or(1000);
                self.max_results_per_page = v.min(10000);
                ctx.link().send_message(Msg::PerformSearch);
                true
            }
            Msg::SaveAssetFinished(_) | Msg::DeleteAssetFinished(_) | Msg::ExportAssetsFinished(_) | Msg::TriggerFileImport(_) | Msg::FileImportTriggered(_) => true,
            Msg::SetPrimaryKey(_) => true,
            Msg::OpenUploadModal(open) => {
                self.upload_modal_open = open;
                if !open {
                    self.upload_data.clear();
                    self.file_input_listener = None;
                } else {
                    let link = ctx.link().clone();
                    let window = web_sys::window().unwrap();
                    let listener = gloo_events::EventListener::new(&window, "file-selected", move |_event| {
                        if let (Ok(name), Ok(data)) = (
                            js_sys::eval("window.__meiliFileName__ || ''"),
                            js_sys::eval("window.__meiliFileData__ || ''")
                        ) {
                            if let (Some(name_str), Some(data_str)) = (name.as_string(), data.as_string()) {
                                if !name_str.is_empty() && !data_str.is_empty() {
                                    link.send_message(Msg::SetUploadFile(name_str, data_str));
                                }
                            }
                        }
                    });
                    self.file_input_listener = Some(listener);
                }
                true
            }
            Msg::SetUploadData(data) => {
                self.upload_data = data;
                true
            }
            Msg::FileInputChanged(_json_content) => {
                if let (Ok(name), Ok(data)) = (
                    js_sys::eval("window.__meiliFileName__ || ''"),
                    js_sys::eval("window.__meiliFileData__ || ''")
                ) {
                    if let (Some(name_str), Some(data_str)) = (name.as_string(), data.as_string()) {
                        if !name_str.is_empty() && !data_str.is_empty() {
                            self.upload_file_name = name_str;
                            self.upload_data = data_str.clone();
                            self.upload_loading = true;
                            match serde_json::from_str::<Vec<serde_json::Value>>(&data_str) {
                                Ok(items) => {
                                    self.upload_preview_data = items;
                                    self.upload_preview_page = 1;
                                    self.upload_loading = false;
                                }
                                Err(e) => {
                                    self.push_toast(format!("JSON解析失败: {}", e), ToastType::Error, ctx);
                                    self.upload_loading = false;
                                    self.upload_preview_data = vec![];
                                }
                            }
                        }
                    }
                }
                true
            }
            Msg::SetUploadFile(name, data) => {
                self.upload_file_name = name;
                self.upload_data = data.clone();
                self.upload_progress = 0.0;
                self.upload_loading = true;
                
                match serde_json::from_str::<Vec<serde_json::Value>>(&data) {
                    Ok(items) => {
                        self.upload_preview_data = items;
                        self.upload_preview_page = 1;
                        self.upload_preview_page_size = 10;
                        self.upload_loading = false;
                        self.upload_progress = 100.0;
                    }
                    Err(e) => {
                        self.push_toast(format!("JSON解析失败: {}", e), ToastType::Error, ctx);
                        self.upload_loading = false;
                        self.upload_preview_data = vec![];
                    }
                }
                true
            }
            Msg::SetUploadPreviewData(data) => {
                self.upload_preview_data = data;
                self.upload_preview_page = 1;
                true
            }
            Msg::SetUploadPage(page) => {
                self.upload_preview_page = page;
                true
            }
            Msg::BatchImportToMeiliSearch => {
                if self.current_index.trim().is_empty() {
                    self.push_toast("请先选择一个索引".to_string(), ToastType::Error, ctx);
                    return true;
                }
                
                if self.upload_data.trim().is_empty() {
                    self.push_toast("请先上传JSON文件".to_string(), ToastType::Error, ctx);
                    return true;
                }
                
                let host = self.host_input.trim().to_string();
                let api_key = self.api_key_input.trim().to_string();
                let index = self.current_index.clone();
                let pk = self.primary_key_field.clone();
                let json_data = self.upload_data.clone();
                // Prepare preview data for UI (pagination)
                if let Ok(preview_vec) = serde_json::from_str::<Vec<serde_json::Value>>(&json_data) {
                    self.upload_preview_data = preview_vec;
                    self.upload_preview_page = 1;
                    self.upload_preview_page_size = 10;
                }
                
                self.loading = true;
                self.upload_progress = 0.0;
                let link = ctx.link().clone();
                let host2 = host.clone();
                let api_key2 = api_key.clone();
                let index2 = index.clone();
                let pk2 = pk.clone();
                let json_data2 = json_data.clone();
                spawn_local(async move {
                    // initial progress update
                    link.send_message(Msg::BatchImportProgress(40.0));
                    match batch_import_documents(&host2, &api_key2, &index2, &pk2, &json_data2).await {
                        Ok(msg) => {
                            link.send_message(Msg::BatchImportProgress(100.0));
                            link.send_message(Msg::BatchImportFinished(Ok(msg)));
                        }
                        Err(e) => link.send_message(Msg::BatchImportFinished(Err(e))),
                    }
                });
                true
            }
            Msg::BatchImportProgress(progress) => {
                self.upload_progress = progress;
                true
            }
            Msg::BatchImportFinished(result) => {
                self.loading = false;
                self.upload_progress = 0.0;
                match result {
                    Ok(msg) => {
                        self.push_toast(msg, ToastType::Success, ctx);
                        self.upload_data.clear();
                        // Keep preview data so user can review what was uploaded
                        self.upload_file_name.clear();
                    }
                    Err(e) => {
                        self.push_toast(format!("批量导入失败: {}", e), ToastType::Error, ctx);
                    }
                }
                true
            }
            _ => false
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme_is_dark = get_theme() == "dark";

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

                <nav class="tab-nav">
                    <button 
                        class={if self.current_tab == "search" { "tab-btn active" } else { "tab-btn" }}
                        onclick={ctx.link().callback(|_| Msg::SetCurrentTab("search".to_string()))}
                    >
                        { "🔍 搜索" }
                    </button>
                    <button 
                        class={if self.current_tab == "assets" { "tab-btn active" } else { "tab-btn" }}
                        onclick={ctx.link().callback(|_| Msg::SetCurrentTab("assets".to_string()))}
                    >
                        { "📦 批量新增" }
                    </button>
                </nav>

                { if self.current_tab == "search" {
                    html! {
                        <>
                            { components::connection_panel::render_connection_panel(self, ctx) }
                            { components::search_section::render_search_section(self, ctx) }
                        </>
                    }
                } else {
                    html! {
                        { self.render_asset_management(ctx) }
                    }
                } }

                <footer class="footer">
                    <p>{ "作者: 冰城拓 Copyright © 2025. 保留所有权利." }</p>
                </footer>

                { components::overlay::render_toasts(self) }
                { components::overlay::render_loading(self) }

                { components::modals::render_field_config_modal(self, ctx) }
                { components::modals::render_column_config_modal(self, ctx) }
                { components::modals::render_view_config_modal(self, ctx) }
                { components::modals::render_result_modal(self, ctx) }
                { self.render_asset_modal(ctx) }
                { self.render_asset_detail_modal(ctx) }
                { self.render_upload_modal(ctx) }

                { components::filter_drawer::render_filter_drawer(self, ctx) }
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

    fn build_search_params(&self) -> Value {
        let effective_limit = self.page_size.min(self.max_results_per_page);
        let mut params = json!({
            "limit": effective_limit,
            "offset": (self.current_page.saturating_sub(1)) * effective_limit,
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

    fn build_search_params_for_export(&self) -> export::ExportParams {
        let mut params = export::ExportParams {
            index_name: Some(self.current_index.clone()),
            query: self.search_input.clone(),
            filter: None,
            sort: None,
            attributes_to_retrieve: None,
            attributes_to_highlight: None,
            highlight_pre_tag: None,
            highlight_post_tag: None,
            hybrid: None,
            show_ranking_score: None,
            show_ranking_score_details: None,
            facets: None,
        };

        if self.highlight_enabled {
            let targets = if !self.highlight_fields.is_empty() {
                self.highlight_fields.clone()
            } else if !self.search_fields.is_empty() {
                self.search_fields.clone()
            } else {
                vec!["*".to_string()]
            };
            params.attributes_to_highlight = Some(json!(targets));
            params.highlight_pre_tag = Some(json!("<em class=\"highlight\">"));
            params.highlight_post_tag = Some(json!("</em>"));
        }

        if self.show_ranking_score {
            params.show_ranking_score = Some(json!(true));
            params.show_ranking_score_details = Some(json!(true));
        }

        if self.ai_config.ai_enabled && self.ai_config.ai_weight > 0 {
            let ratio = (self.ai_config.ai_weight.min(100) as f64) / 100.0;
            params.hybrid = Some(json!({
                "semanticRatio": ratio,
                "embedder": "bgem3"
            }));
        }

        if !self.sort_value.is_empty() {
            params.sort = Some(json!([self.sort_value.clone()]));
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
            params.filter = Some(json!(expr));
        }

        if !self.display_fields.is_empty() {
            let mut attrs = self.display_fields.clone();
            if !attrs.contains(&"id".to_string()) {
                attrs.push("id".to_string());
            }
            if !self.primary_key_field.is_empty() && !attrs.contains(&self.primary_key_field) {
                attrs.push(self.primary_key_field.clone());
            }
            params.attributes_to_retrieve = Some(json!(attrs));
        } else {
            params.attributes_to_retrieve = Some(json!(["*"]));
        }

        if !self.filterable_fields.is_empty() {
            params.facets = Some(json!(self.filterable_fields.clone()));
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
                    <p>{ "当前列已全部隐藏，请在列设置中勾选显示。" }</p>
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

                    if let Some(cell) = get_cell_value(hit, col, self.highlight_enabled, self.image_preview_enabled, self.image_preview_links_only, self.image_preview_size) {
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
                    { for fields.iter().enumerate().map(|(item_idx, field)| {
                        let label = self.field_labels.get(field).cloned().unwrap_or_else(|| field.clone());
                        let drag_field = field.clone();
                        let col_idx = idx;
                        html! {
                            <div
                                class="view-field-item view-field-selected"
                                draggable="true"
                                ondragstart={ctx.link().callback(move |e: DragEvent| {
                                    if let Some(dt) = e.data_transfer() {
                                        let _ = dt.set_data("text/plain", &drag_field);
                                    }
                                    Msg::StartViewFieldDragInColumn(drag_field.clone(), col_idx)
                                })}
                                ondragover={ctx.link().callback(move |e: DragEvent| {
                                    e.prevent_default();
                                    Msg::ViewFieldDragOver(col_idx, item_idx)
                                })}
                                ondrop={ctx.link().callback(move |e: DragEvent| {
                                    e.prevent_default();
                                    Msg::DropViewField(col_idx)
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
            ToastType::Info => "toast info",
        };
        let icon = match toast.kind {
            ToastType::Success => "✅",
            ToastType::Error => "❌",
            ToastType::Warning => "⚠️",
            ToastType::Info => "ℹ️",
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

    
    fn render_asset_management(&self, ctx: &Context<Self>) -> Html {
        let total_pages = (self.upload_preview_data.len() as f64 / self.upload_preview_page_size as f64).ceil() as usize;
        let start_idx = (self.upload_preview_page.saturating_sub(1)) * self.upload_preview_page_size;
        let end_idx = (start_idx + self.upload_preview_page_size).min(self.upload_preview_data.len());
        let page_data = &self.upload_preview_data[start_idx..end_idx];
        
        let preview_rows: Vec<Html> = page_data.iter().enumerate().map(|(i, item)| {
            let json_str = serde_json::to_string_pretty(item).unwrap_or_default();
            let row_num = start_idx + i + 1;
            html! {
                <div style="background:var(--bg-secondary);padding:8px;margin-bottom:8px;border-radius:4px;font-size:12px;">
                    <div style="color:var(--text-muted);margin-bottom:4px;">{ format!("#{} ", row_num) }</div>
                    <pre style="margin:0;white-space:pre-wrap;word-break:break-all;max-height:120px;overflow:auto;">{ json_str }</pre>
                </div>
            }
        }).collect();
        
        let pagination: Vec<Html> = if total_pages > 1 {
            let mut items = vec![];
            let max_buttons = 5;
            let current_page = self.upload_preview_page;
            let start_page = ((current_page - 1) / max_buttons) * max_buttons + 1;
            let end_page = (start_page + max_buttons - 1).min(total_pages);
            
            if start_page > 1 {
                items.push(html! { <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SetUploadPage(1))}>{ "«" }</button> });
                items.push(html! { <button class="btn btn-secondary" onclick={ctx.link().callback(move |_| Msg::SetUploadPage(current_page.saturating_sub(1)))}>{ "‹" }</button> });
            }
            
            for p in start_page..=end_page {
                let is_active = p == current_page;
                let page_num = p;
                items.push(html! {
                    <button 
                        class={if is_active { "btn btn-primary" } else { "btn btn-secondary" }}
                        onclick={ctx.link().callback(move |_| Msg::SetUploadPage(page_num))}
                    >
                        { p }
                    </button>
                });
            }
            
            if end_page < total_pages {
                let next_page = current_page + 1;
                let last_page = total_pages;
                items.push(html! { <button class="btn btn-secondary" onclick={ctx.link().callback(move |_| Msg::SetUploadPage(next_page))}>{ "›" }</button> });
                items.push(html! { <button class="btn btn-secondary" onclick={ctx.link().callback(move |_| Msg::SetUploadPage(last_page))}>{ "»" }</button> });
            }
            items
        } else {
            vec![]
        };
        
        let progress_bar = if self.loading && self.upload_progress > 0.0 {
            html! {
                <div style="margin-top:12px;">
                    <div style="background:var(--bg-secondary);height:8px;border-radius:4px;overflow:hidden;">
                        <div style={format!("width:{}%;height:100%;background:var(--primary-color);transition:width 0.3s;", self.upload_progress)}></div>
                    </div>
                    <p style="margin:8px 0 0 0;font-size:12px;color:var(--text-muted);">{ format!("导入进度: {:.0}%", self.upload_progress) }</p>
                </div>
            }
        } else {
            html! {}
        };
        
        let import_button = if !self.upload_preview_data.is_empty() && !self.loading {
            html! {
                <button 
                    class="btn btn-primary" 
                    style="padding:12px 24px;font-size:14px;"
                    onclick={ctx.link().callback(|_| Msg::BatchImportToMeiliSearch)}
                >
                    { format!("📤 确认导入 (共 {} 条)", self.upload_preview_data.len()) }
                </button>
            }
        } else if self.loading {
            html! {
                <button class="btn btn-primary" disabled={true} style="padding:12px 24px;font-size:14px;">
                    { "导入中..." }
                </button>
            }
        } else {
            html! {}
        };
        
        html! {
            <div class="asset-management">
                <div class="asset-header">
                    <h2>{ "📦 批量新增" }</h2>
                </div>
                
                <div style="background:var(--surface);padding:16px;border-radius:var(--radius);margin-bottom:20px;">
                    <h3 style="margin:0 0 12px 0;font-size:15px;color:var(--text-primary);">{"创建新索引"}</h3>
                    <div style="display:flex;gap:12px;align-items:flex-end;flex-wrap:wrap;">
                        <div class="form-group" style="flex:0 0 280px;margin:0;">
                            <label style="font-size:13px;">{"索引 UID"}</label>
                            <input 
                                class="form-control" 
                                value={self.new_index_uid.clone()} 
                                placeholder="my_index"
                                oninput={ctx.link().callback(|e: yew::events::InputEvent| {
                                    let val = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                                    Msg::SetNewIndexUid(val)
                                })} 
                            />
                        </div>
                        <div class="form-group" style="flex:0 0 200px;margin:0;">
                            <label style="font-size:13px;">{"主键字段"}</label>
                            <input 
                                class="form-control" 
                                value={self.new_index_pk.clone()} 
                                placeholder="id"
                                oninput={ctx.link().callback(|e: yew::events::InputEvent| {
                                    let val = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                                    Msg::SetNewIndexPk(val)
                                })} 
                            />
                        </div>
                        <button 
                            class="btn btn-primary" 
                            onclick={ctx.link().callback(|_| Msg::CreateIndexLocal)} 
                            style="margin-bottom:4px;"
                        >
                            { "创建索引" }
                        </button>
                    </div>
                </div>
                
                { if self.current_index.trim().is_empty() {
                    html! {
                        <div style="background:rgba(245,158,11,0.1);padding:16px;border-radius:var(--radius);border-left:3px solid var(--warning-color);">
                            <p style="margin:0;font-size:14px;color:var(--text-secondary);">
                                <strong style="color:var(--warning-color);">{"请选择或创建索引后进行批量导入"}</strong>
                            </p>
                        </div>
                    }
                } else {
                    html! {
                        <>
                            <div style="background:var(--surface);padding:12px 16px;border-radius:var(--radius);margin-bottom:16px;display:flex;align-items:center;gap:12px;">
                                <span style="font-weight:600;">{"当前索引:"}</span>
                                <span style="color:var(--primary-color);font-weight:600;">{ &self.current_index }</span>
                                <span style="color:var(--text-muted);">{" | "}</span>
                                <span style="font-weight:600;">{"主键字段:"}</span>
                                <span style="color:var(--secondary-color);">{ &self.primary_key_field }</span>
                            </div>
                            
                            <div style="background:var(--surface);padding:16px;border-radius:var(--radius);margin-bottom:16px;">
                                <h3 style="margin:0 0 12px 0;font-size:15px;color:var(--text-primary);">{"选择 JSON 文件"}</h3>
                                <input 
                                    type="file" 
                                    accept=".json"
                                    id="json-file-input"
                                    onchange={ctx.link().callback(move |_| {
                                        let script = r#"
                                            (function() {
                                                var input = document.getElementById('json-file-input');
                                                if (input && input.files[0]) {
                                                    var file = input.files[0];
                                                    var reader = new FileReader();
                                                    reader.onload = function(e) {
                                                        var content = e.target.result;
                                                        var jsonData = JSON.parse(content);
                                                        var count = Array.isArray(jsonData) ? jsonData.length : 1;
                                                        window.__meiliFileName__ = file.name;
                                                        window.__meiliFileData__ = JSON.stringify(jsonData);
                                                        window.__meiliFileCount__ = count;
                                                        window.dispatchEvent(new CustomEvent('file-selected', {detail: {name: file.name, data: JSON.stringify(jsonData)}}));
                                                    };
                                                    reader.readAsText(file);
                                                }
                                            })();
                                        "#;
                                        let _ = js_sys::eval(script);
                                        Msg::FileInputChanged(String::new())
                                    })}
                                />
                                <p style="margin:8px 0 0 0;font-size:12px;color:var(--text-muted);">{"支持 JSON 数组格式"}</p>
                                
                                { if self.upload_loading {
                                    html! {
                                        <div style="margin-top:12px;text-align:center;color:var(--text-muted);">
                                            <p>{"正在解析文件..."}</p>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                } }
                                
                                { if !self.upload_file_name.is_empty() {
                                    html! {
                                        <div style="margin-top:12px;padding:8px 12px;background:var(--bg-secondary);border-radius:4px;display:flex;align-items:center;gap:8px;">
                                            <span style="color:var(--success-color);">{"✓"}</span>
                                            <span>{ format!("已选择: {} ({} 条数据)", self.upload_file_name, self.upload_preview_data.len()) }</span>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                } }
                            </div>
                            
                            { progress_bar }
                            
                            { if !self.upload_preview_data.is_empty() {
                                html! {
                                    <div style="background:var(--surface);padding:16px;border-radius:var(--radius);margin-bottom:16px;">
                                        <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
                                            <h3 style="margin:0;font-size:15px;color:var(--text-primary);">
                                                { format!("数据预览 (第 {} 条 - 第 {} 条，共 {} 条)", start_idx + 1, end_idx, self.upload_preview_data.len()) }
                                            </h3>
                                        </div>
                                        <div style="max-height:400px;overflow-y:auto;">
                                            { for preview_rows }
                                        </div>
                                        { if !pagination.is_empty() {
                                            html! {
                                                <div style="display:flex;gap:8px;justify-content:center;margin-top:16px;flex-wrap:wrap;">
                                                    { for pagination }
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        } }
                                    </div>
                                }
                            } else {
                                html! {}
                            } }
                            
                            <div style="margin-top:20px;text-align:center;">
                                { import_button }
                            </div>
                        </>
                    }
                } }
            </div>
        }
    }

    fn render_asset_modal(&self, ctx: &Context<Self>) -> Html {
        if !self.asset_modal_open {
            return html! {};
        }

        html! {
            <div class="modal-overlay" onclick={ctx.link().callback(|_| Msg::OpenAssetModal(false))}>
                <div class="modal" onclick={ctx.link().callback(|_| Msg::DocumentClick)}>
                    <div class="modal-header">
                        <h3>{ "添加设备" }</h3>
                        <button class="modal-close" onclick={ctx.link().callback(|_| Msg::OpenAssetModal(false))}>{"X"}</button>
                    </div>
                    <div class="modal-body">
                        <div class="form-group">
                            <label>{ "设备名称" }</label>
                            <input type="text" class="form-control" value={self.asset_form.name.clone()} />
                        </div>
                        <div class="form-group">
                            <label>{ "品牌" }</label>
                            <input type="text" class="form-control" value={self.asset_form.brand.clone()} />
                        </div>
                        <div class="form-group">
                            <label>{ "型号" }</label>
                            <input type="text" class="form-control" value={self.asset_form.model.clone()} />
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::OpenAssetModal(false))}>{"取消"}</button>
                        <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::SaveAsset)}>{"保存"}</button>
                    </div>
                </div>
            </div>
        }
    }

    fn render_asset_detail_modal(&self, ctx: &Context<Self>) -> Html {
        if let Some(_asset) = &self.asset_detail {
            html! {
                <div class="modal-overlay" onclick={ctx.link().callback(|_| Msg::CloseAssetDetail)}>
                    <div class="modal modal-lg" onclick={ctx.link().callback(|_| Msg::DocumentClick)}>
                        <div class="modal-header">
                            <h3>{ "设备详情" }</h3>
                            <button class="modal-close" onclick={ctx.link().callback(|_| Msg::CloseAssetDetail)}>{"X"}</button>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::CloseAssetDetail)}>{"关闭"}</button>
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    }

    fn render_upload_modal(&self, ctx: &Context<Self>) -> Html {
        if !self.upload_modal_open {
            return html! {};
        }
        
        let debug_info = format!("upload_modal_open={}", self.upload_modal_open);
        
        html! {
            <div style="
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0,0,0,0.7);
                z-index: 99999;
                display: flex;
                align-items: center;
                justify-content: center;
            " onclick={ctx.link().callback(|_| Msg::OpenUploadModal(false))}>
                <div style="
                    background: var(--surface, #1a1a2e);
                    border-radius: 8px;
                    width: 600px;
                    max-width: 90vw;
                    max-height: 90vh;
                    overflow: auto;
                    box-shadow: 0 4px 20px rgba(0,0,0,0.5);
                " onclick={ctx.link().callback(|_| Msg::DocumentClick)}>
                    <div style="display:flex;justify-content:space-between;align-items:center;padding:16px;border-bottom:1px solid #333;">
                        <h3 style="margin:0;color:#fff;">{ "📤 批量导入 JSON" }</h3>
                        <button style="
                            background:none;border:none;color:#fff;font-size:20px;cursor:pointer;padding:4px 8px;
                        " onclick={ctx.link().callback(|_| Msg::OpenUploadModal(false))}>{"×"}</button>
                    </div>
                    <div style="padding:20px;">
                        <p style="color:#888;font-size:12px;margin:0 0 16px 0;">{ debug_info }</p>
                        <div style="margin-bottom:16px;">
                            <label style="display:block;margin-bottom:8px;color:#fff;font-weight:600;">{ "目标索引" }</label>
                            <p style="color:#06b6d4;font-weight:600;margin:0;">{ &self.current_index }{ " (主键: " }{ &self.primary_key_field }{ " )" }</p>
                        </div>
                        <div style="margin-bottom:16px;">
                            <label style="display:block;margin-bottom:8px;color:#fff;font-weight:600;">{ "选择 JSON 文件" }</label>
                            <input 
                                type="file" 
                                accept=".json"
                                id="json-file-input"
                                style="color:#fff;"
                            />
                        </div>
                        <div id="file-preview" style="background:#16213e;padding:12px;border-radius:4px;min-height:60px;color:#fff;">
                            <p style="color:#888;margin:0;">{ "请选择文件后预览" }</p>
                        </div>
                    </div>
                    <div style="padding:16px;border-top:1px solid #333;display:flex;gap:12px;justify-content:flex-end;">
                        <button 
                            style="padding:8px 16px;border-radius:4px;border:none;background:#555;color:#fff;cursor:pointer;"
                            onclick={ctx.link().callback(|_| Msg::OpenUploadModal(false))}
                        >
                            { "取消" }
                        </button>
                        <button 
                            style="padding:8px 16px;border-radius:4px;border:none;background:#06b6d4;color:#000;font-weight:600;cursor:pointer;"
                            onclick={ctx.link().callback(|_| Msg::BatchImportToMeiliSearch)}
                        >
                            { "确认导入" }
                        </button>
                    </div>
                </div>
            </div>
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

    fn view_field_pool_is_empty(&self) -> bool {
        let mut used = HashSet::new();
        for col in &self.view_layout_working {
            for f in col {
                used.insert(f.clone());
            }
        }
        self.available_fields.iter().all(|f| used.contains(f))
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
