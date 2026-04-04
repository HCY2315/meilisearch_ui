use crate::types::SearchHit;
use crate::utils::{escape_html, strip_html, value_to_string};
use serde_json::Value;
use yew::{html, AttrValue, Html};

#[derive(Clone)]
pub struct CellValue {
    pub html: Html,
    pub title: AttrValue,
}

pub fn hit_has_id(hit: &SearchHit) -> bool {
    hit.id.is_some() || hit.fields.get("id").is_some()
}

pub fn get_id_string(hit: &SearchHit) -> String {
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

pub fn get_doc_key(hit: &SearchHit, primary_key: &str) -> String {
    if !primary_key.trim().is_empty() {
        if let Some(val) = hit.get(primary_key) {
            return value_to_string(val);
        }
    }
    get_id_string(hit)
}

pub fn hit_entries(hit: &SearchHit) -> Vec<(String, Value)> {
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
    pub fn get(&self, key: &str) -> Option<&Value> {
        if key == "id" {
            return self.id.as_ref();
        }
        self.fields.get(key)
    }
}

pub fn get_cell_value(hit: &SearchHit, col: &str, use_highlight: bool) -> Option<CellValue> {
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
        let text = raw
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| raw.to_string());
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
