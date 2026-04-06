use crate::storage::web_document;
use crate::types::FieldConfigItem;
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::Element;
use yew::TargetCast;

pub fn set_all_checkboxes(selector: &str, checked: bool) {
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

pub fn format_filter_value(raw: &str) -> String {
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

pub fn normalize_operator(input: &str) -> String {
    let op = input.trim();
    let allowed = [
        "=",
        "!=",
        ">",
        "<",
        ">=",
        "<=",
        "IN",
        "NOT IN",
        "EXISTS",
        "NOT EXISTS",
    ];
    if allowed.iter().any(|v| *v == op) {
        op.to_string()
    } else {
        "=".to_string()
    }
}

pub fn build_filter_expression_from_dom() -> Option<String> {
    let Ok(rows) = web_document().query_selector_all(".query-row") else {
        return None;
    };
    let mut conditions: Vec<(String, String)> = vec![];
    for i in 0..rows.length() {
        let Some(node) = rows.item(i) else { continue };
        let Some(row) = node.dyn_ref::<Element>() else {
            continue;
        };
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

pub fn build_filter_expression_from_conditions(conditions: &[(String, String)]) -> Option<String> {
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

pub fn collect_hidden_columns() -> Vec<String> {
    let mut hidden = vec![];
    if let Ok(inputs) =
        web_document().query_selector_all(".column-config-item input[type=\"checkbox\"]")
    {
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

pub fn collect_column_labels() -> HashMap<String, String> {
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

pub fn collect_field_config() -> Vec<FieldConfigItem> {
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

pub fn input_value(event: yew::events::InputEvent) -> String {
    let input: web_sys::HtmlInputElement = event.target_unchecked_into();
    input.value()
}

pub fn select_value(event: yew::events::Event) -> String {
    let select: web_sys::HtmlSelectElement = event.target_unchecked_into();
    select.value()
}

pub fn checkbox_checked(event: yew::events::Event) -> bool {
    let input: web_sys::HtmlInputElement = event.target_unchecked_into();
    input.checked()
}

pub fn format_number(value: u64) -> String {
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

pub fn format_time(timestamp: &str) -> String {
    let now = js_sys::Date::new_0().get_time();
    let then = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(timestamp)).get_time();
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
    js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(then))
        .to_locale_date_string("zh-CN", &wasm_bindgen::JsValue::undefined())
        .into()
}

pub fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn strip_html(input: &str) -> String {
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

pub fn value_to_string(value: &Value) -> String {
    if let Some(s) = value.as_str() {
        s.to_string()
    } else if value.is_object() {
        serde_json::to_string(value).unwrap_or_default()
    } else {
        value.to_string()
    }
}

pub fn value_to_string_for_edit(value: &Value) -> String {
    if value.is_null() {
        return "".to_string();
    }
    if value.is_object() || value.is_array() {
        return serde_json::to_string(value).unwrap_or_default();
    }
    value_to_string(value)
}

/// Heuristic to detect common image URLs
pub fn is_image_url(url: &str) -> bool {
    let s = url.trim().to_lowercase();
    if s.starts_with("http://") || s.starts_with("https://") {
        s.ends_with(".png")
            || s.ends_with(".jpg")
            || s.ends_with(".jpeg")
            || s.ends_with(".gif")
            || s.ends_with(".webp")
            || s.ends_with(".svg")
    } else {
        false
    }
}

/// Check if string is an HTTP/HTTPS URL
pub fn is_http_url(url: &str) -> bool {
    let s = url.trim().to_lowercase();
    s.starts_with("http://") || s.starts_with("https://")
}
