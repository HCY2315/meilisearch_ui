use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, HtmlElement};
use yew::events::{DragEvent, MouseEvent};
use yew::{html, Callback, Component, Context, Html};

mod types;
mod storage;
mod utils;
mod cell;
mod sort;
mod api;
mod parse;
mod index_panel;
mod query_editor;
mod results_table;
mod view_config;
mod ai_config;
mod history;
mod field_prefs;

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
    view_drag_over_index: Option<usize>,
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
            view_drag_over_index: None,
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
        index_panel::init();
        query_editor::init();
        results_table::init();
        view_config::init();
        ai_config::init();
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
