use crate::types::{AiConfig, HistoryItem};
use std::collections::HashMap;
use web_sys::{Document, Window};

pub fn web_window() -> Window {
    web_sys::window().expect("no window")
}

pub fn web_document() -> Document {
    web_window().document().expect("no document")
}

pub fn storage_get(key: &str) -> Option<String> {
    web_window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|storage| storage.get_item(key).ok().flatten())
}

pub fn storage_set(key: &str, value: &str) {
    if let Ok(Some(storage)) = web_window().local_storage() {
        let _ = storage.set_item(key, value);
    }
}

pub fn storage_remove(key: &str) {
    if let Ok(Some(storage)) = web_window().local_storage() {
        let _ = storage.remove_item(key);
    }
}

pub fn load_search_history() -> Vec<HistoryItem> {
    storage_get("searchHistory")
        .and_then(|value| serde_json::from_str::<Vec<HistoryItem>>(&value).ok())
        .unwrap_or_default()
}

pub fn save_search_history(history: &[HistoryItem]) {
    if let Ok(value) = serde_json::to_string(history) {
        storage_set("searchHistory", &value);
    }
}

pub fn load_field_labels(index: Option<&str>) -> HashMap<String, String> {
    let key = format!("fieldLabels:{}", index.unwrap_or("default"));
    storage_get(&key)
        .and_then(|value| serde_json::from_str::<HashMap<String, String>>(&value).ok())
        .unwrap_or_default()
}

pub fn save_field_labels(labels: &HashMap<String, String>, index: Option<&str>) {
    let key = format!("fieldLabels:{}", index.unwrap_or("default"));
    if let Ok(value) = serde_json::to_string(labels) {
        storage_set(&key, &value);
    }
}

pub fn load_popular_field(index: Option<&str>) -> String {
    let key = format!("popularSearchField:{}", index.unwrap_or("default"));
    storage_get(&key).unwrap_or_default()
}

pub fn save_popular_field(field: &str, index: Option<&str>) {
    let key = format!("popularSearchField:{}", index.unwrap_or("default"));
    if field.is_empty() {
        storage_remove(&key);
    } else {
        storage_set(&key, field);
    }
}

pub fn load_ai_config() -> AiConfig {
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

pub fn save_ai_config(config: &AiConfig) {
    if let Ok(value) = serde_json::to_string(config) {
        storage_set("aiSearchConfig", &value);
    }
}

pub fn get_theme() -> String {
    storage_get("theme").unwrap_or_else(|| "dark".to_string())
}

pub fn set_theme(theme: &str) {
    let next = if theme == "light" { "light" } else { "dark" };
    if let Some(el) = web_document().document_element() {
        let _ = el.set_attribute("data-theme", next);
    }
    storage_set("theme", next);
}

pub fn load_image_preview_enabled() -> bool {
    storage_get("imagePreviewEnabled")
        .map(|v| v == "true")
        .unwrap_or(false)
}

pub fn save_image_preview_enabled(enabled: bool) {
    storage_set(
        "imagePreviewEnabled",
        if enabled { "true" } else { "false" },
    );
}

pub fn load_image_preview_links_only() -> bool {
    storage_get("imagePreviewLinksOnly")
        .map(|v| v == "true")
        .unwrap_or(false)
}

pub fn save_image_preview_links_only(enabled: bool) {
    storage_set(
        "imagePreviewLinksOnly",
        if enabled { "true" } else { "false" },
    );
}

pub fn load_image_preview_size() -> u32 {
    storage_get("imagePreviewSize")
        .and_then(|v| v.parse().ok())
        .unwrap_or(80)
}

pub fn save_image_preview_size(size: u32) {
    storage_set("imagePreviewSize", &size.to_string());
}
