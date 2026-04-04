use crate::{
    build_filter_expression_from_dom, input_value, normalize_operator, select_value, App, Msg,
    QueryRow,
};
use yew::{html, Context, Html};

/// Add a new query row to the app state
pub fn add_query_row(app: &mut App) {
    let id = app.next_query_id;
    app.next_query_id += 1;
    app.query_rows.push(QueryRow {
        id,
        field: String::new(),
        operator: "=".to_string(),
        value: String::new(),
        logic: "AND".to_string(),
    });
}

/// Render a single query row
pub fn render_query_row(app: &App, ctx: &Context<App>, row: &QueryRow) -> Html {
    let id = row.id;
    let filter_fields = get_filter_fields_for_query(app);
    let operator_value = normalize_operator(&row.operator);
    let logic_value = if row.logic.trim().is_empty() {
        "AND".to_string()
    } else {
        row.logic.clone()
    };
    html! {
        <div class="query-row" key={row.id} data-row-id={row.id.to_string()}>
            <select
                class="form-control query-field"
                value={row.field.clone()}
                onchange={ctx.link().callback(move |e: yew::events::Event| Msg::UpdateQueryField(id, select_value(e)))}
            >
                <option value="">{ "选择字段" }</option>
                { for filter_fields.iter().map(|field| {
                    let label = app.field_labels.get(field).cloned().unwrap_or_else(|| field.clone());
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

/// Get filter fields for the query dropdown
pub fn get_filter_fields_for_query(app: &App) -> Vec<String> {
    if !app.filterable_fields.is_empty() {
        app.filterable_fields.clone()
    } else {
        app.search_fields.clone()
    }
}

/// Refresh query rows to match available filter fields
pub fn refresh_query_rows(app: &mut App) {
    let fields = get_filter_fields_for_query(app);
    for row in &mut app.query_rows {
        if !fields.contains(&row.field) {
            row.field = String::new();
        }
    }
}

/// Sync query row state from DOM elements
pub fn sync_query_rows_from_dom(app: &mut App) {
    use wasm_bindgen::JsCast;
    let Ok(rows) = crate::web_document().query_selector_all(".query-row") else {
        return;
    };
    for i in 0..rows.length() {
        let Some(node) = rows.item(i) else { continue };
        let Some(row_el) = node.dyn_ref::<web_sys::Element>() else {
            continue;
        };
        let id_attr = row_el.get_attribute("data-row-id").unwrap_or_default();
        let Ok(row_id) = id_attr.parse::<u64>() else {
            continue;
        };
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
        if let Some(row) = app.query_rows.iter_mut().find(|r| r.id == row_id) {
            row.field = field;
            row.operator = operator;
            row.value = value;
            row.logic = logic;
        }
    }
}

/// Build the filter preview text shown below query builder
pub fn filter_preview_text(app: &App) -> String {
    let keyword = app.search_input.trim();
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

pub fn init() {
    // no-op for skeleton
}
