use crate::types::SearchHit;
use crate::utils::{escape_html, is_http_url, is_image_url, strip_html, value_to_string};
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

pub fn get_cell_value(
    hit: &SearchHit,
    col: &str,
    use_highlight: bool,
    image_preview_enabled: bool,
    image_preview_links_only: bool,
) -> Option<CellValue> {
    let raw = if use_highlight {
        hit.formatted
            .as_ref()
            .and_then(|f| f.get(col))
            .cloned()
            .or_else(|| hit.get(col).cloned())
    } else {
        hit.get(col).cloned()
    }?;

    // If image preview is enabled, render images or links depending on thumbnail mode flag
    if image_preview_enabled {
        if image_preview_links_only {
            if let Some(cell) = try_render_image_cell(&raw) {
                return Some(cell);
            }
        } else if let Some(link_cell) = render_image_links_cell(&raw) {
            return Some(link_cell);
        }
        // If neither image nor link rendering applied, fall through to normal rendering
    } else {
        // Legacy behavior: only render if it looks like an image URL
        if let Some(s) = raw.as_str() {
            if is_image_url(s) {
                let src = s.to_string();
                return Some(CellValue {
                    html: html! {
                        <img class="cell-thumbnail" src={src.clone()} alt={"image"} loading="lazy" />
                    },
                    title: AttrValue::from(escape_html(&src)),
                });
            }
        }
    }

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

fn try_render_image_cell(raw: &Value) -> Option<CellValue> {
    if raw.is_array() {
        let urls: Vec<String> = raw
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .filter(|s| !s.trim().is_empty() && is_http_url(s))
            .take(3)
            .collect();

        if urls.is_empty() {
            return None;
        }

        let img_tags: Vec<Html> = urls
            .iter()
            .map(|url| {
                html! {
                    <img
                        class="cell-thumbnail"
                        src={url.clone()}
                        alt={"thumbnail"}
                        loading="lazy"
                    />
                }
            })
            .collect();

        let title_text = urls.join(", ");
        return Some(CellValue {
            html: html! {
                <div class="cell-thumbnail-scroll">
                    { for img_tags }
                </div>
            },
            title: AttrValue::from(escape_html(&title_text)),
        });
    }

    if let Some(s) = raw.as_str() {
        if s.trim().is_empty() || !is_http_url(s) {
            return None;
        }
        return Some(CellValue {
            html: html! {
                <img
                    class="cell-thumbnail"
                    src={s.to_string()}
                    alt={"thumbnail"}
                    loading="lazy"
                />
            },
            title: AttrValue::from(escape_html(s)),
        });
    }

    None
}

fn render_image_links_cell(raw: &Value) -> Option<CellValue> {
    fn extract_filename(url: &str) -> Option<String> {
        if url.starts_with("http://") || url.starts_with("https://") {
            Some(url.split('/').last().unwrap_or(url).to_string())
        } else {
            None
        }
    }

    if raw.is_array() {
        let urls: Vec<String> = raw
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .filter(|s| !s.trim().is_empty())
            .take(3)
            .collect();
        if urls.is_empty() {
            return None;
        }
        let links: Vec<Html> = urls.iter().map(|u| {
            if let Some(display) = extract_filename(u) {
                html! { <a href={u.clone()} target="_blank" rel="noopener noreferrer">{ display }</a> }
            } else {
                html! { <span>{ u.clone() }</span> }
            }
        }).collect();
        let title_text = urls.join(", ");
        return Some(CellValue {
            html: html! { <div class="cell-thumbnail-scroll">{ for links }</div> },
            title: AttrValue::from(escape_html(&title_text)),
        });
    }
    if let Some(s) = raw.as_str() {
        if s.trim().is_empty() {
            return None;
        }
        if let Some(display) = extract_filename(s) {
            return Some(CellValue {
                html: html! { <a href={s.to_string()} target="_blank" rel="noopener noreferrer">{ display }</a> },
                title: AttrValue::from(escape_html(s)),
            });
        } else {
            return Some(CellValue {
                html: html! { <span>{ s.to_string() }</span> },
                title: AttrValue::from(escape_html(s)),
            });
        }
    }
    None
}
