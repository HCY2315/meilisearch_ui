use crate::value_to_string;
use gloo_net::http::Request;
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

/// Search parameters for export (mirrors what build_search_params produces)
#[derive(Clone, Debug)]
pub struct ExportParams {
    pub index_name: Option<String>,
    pub query: String,
    pub filter: Option<Value>,
    pub sort: Option<Value>,
    pub attributes_to_retrieve: Option<Value>,
    pub attributes_to_highlight: Option<Value>,
    pub highlight_pre_tag: Option<Value>,
    pub highlight_post_tag: Option<Value>,
    pub hybrid: Option<Value>,
    pub show_ranking_score: Option<Value>,
    pub show_ranking_score_details: Option<Value>,
    pub facets: Option<Value>,
}

/// Export ALL matching results from MeiliSearch via paginated fetching,
/// then build a CSV and trigger download.
pub async fn export_all_csv(
    host: &str,
    api_key: &str,
    params: &ExportParams,
    on_progress: impl Fn(u64, u64),
) -> Result<String, String> {
    let base_url = if host.ends_with('/') {
        format!("{}indexes", host)
    } else {
        format!("{}/indexes", host)
    };

    // We need the index name to build search URL
    // Extract index from the host URL path or use a default approach
    // Actually, we need to pass the index name separately
    // Let's use the search endpoint pattern: {host}/indexes/{index_uid}/search
    // Since we don't have the index here, we need to pass it in params
    // For now, let's use a different approach: POST to /indexes/{index}/search

    // The index name should be part of the params - let's add it
    let index_name = params.index_name.as_ref()
        .ok_or("Index name is required for export")?;

    let search_url = if host.ends_with('/') {
        format!("{}indexes/{}/search", host, index_name)
    } else {
        format!("{}/indexes/{}/search", host, index_name)
    };

    let limit = 1000; // MeiliSearch max per request
    let mut all_hits: Vec<Value> = Vec::new();
    let mut offset: u64 = 0;
    let mut total_estimated: u64 = 0;

    loop {
        let mut body = json!({
            "q": params.query,
            "limit": limit,
            "offset": offset,
            "matchingStrategy": "last"
        });

        if let Some(ref filter) = params.filter {
            body["filter"] = filter.clone();
        }
        if let Some(ref sort) = params.sort {
            body["sort"] = sort.clone();
        }
        if let Some(ref attrs) = params.attributes_to_retrieve {
            body["attributesToRetrieve"] = attrs.clone();
        }
        if let Some(ref hybrid) = params.hybrid {
            body["hybrid"] = hybrid.clone();
        }
        if let Some(ref v) = params.show_ranking_score {
            body["showRankingScore"] = v.clone();
        }
        if let Some(ref v) = params.show_ranking_score_details {
            body["showRankingScoreDetails"] = v.clone();
        }
        if let Some(ref facets) = params.facets {
            body["facets"] = facets.clone();
        }

        let resp = Request::post(&search_url)
            .header("Authorization", &format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .map_err(|e| format!("JSON serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("HTTP request error: {}", e))?;

        if !resp.ok() {
            let err_text = resp.text().await.unwrap_or_else(|_| "unknown error".to_string());
            return Err(format!("MeiliSearch 返回错误 (HTTP {}): {}", resp.status(), err_text));
        }

        let text = resp.text().await.map_err(|e| format!("HTTP read error: {}", e))?;
        let res: Value = serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))?;

        if let Some(hits) = res.get("hits").and_then(|v| v.as_array()) {
            if hits.is_empty() {
                break;
            }
            total_estimated = res.get("totalHits")
                .and_then(|v| v.as_u64())
                .unwrap_or_else(|| offset + hits.len() as u64);

            for hit in hits {
                // Strip highlight tags if present
                all_hits.push(strip_highlight_tags(hit, &params.highlight_pre_tag, &params.highlight_post_tag));
            }

            on_progress(all_hits.len() as u64, total_estimated);

            if hits.len() < limit as usize {
                break;
            }
        } else {
            break;
        }

        offset += limit as u64;
    }

    if all_hits.is_empty() {
        return Err("没有匹配的数据".to_string());
    }

    // Collect all unique keys from hits for CSV columns
    let mut columns: Vec<String> = Vec::new();
    for hit in &all_hits {
        if let Some(obj) = hit.as_object() {
            for key in obj.keys() {
                if !columns.contains(key) {
                    columns.push(key.clone());
                }
            }
        }
    }

    // Build CSV with UTF-8 BOM
    let mut csv_content = String::from("\u{FEFF}");
    csv_content.push_str(
        &columns.iter().map(|c| escape_csv_field(c)).collect::<Vec<_>>().join(","),
    );
    csv_content.push('\n');

    for hit in &all_hits {
        let row = columns.iter().map(|col| {
            let val = hit.get(col).cloned().unwrap_or(Value::Null);
            escape_csv_field(&value_to_string(&val))
        }).collect::<Vec<_>>().join(",");
        csv_content.push_str(&row);
        csv_content.push('\n');
    }

    let filename = format!("export_{}.csv", timestamp());
    trigger_download(&csv_content, &filename);

    Ok(filename)
}

fn strip_highlight_tags(hit: &Value, pre_tag: &Option<Value>, post_tag: &Option<Value>) -> Value {
    let pre = pre_tag.as_ref().and_then(|v| v.as_str()).unwrap_or("");
    let post = post_tag.as_ref().and_then(|v| v.as_str()).unwrap_or("");
    if pre.is_empty() && post.is_empty() {
        return hit.clone();
    }
    strip_tags_recursive(hit, pre, post)
}

fn strip_tags_recursive(value: &Value, pre: &str, post: &str) -> Value {
    match value {
        Value::String(s) => {
            let mut result = s.clone();
            while let Some(pos) = result.find(pre) {
                if let Some(end_pos) = result[pos + pre.len()..].find(post) {
                    result.replace_range(pos..pos + pre.len() + end_pos + post.len(), "");
                } else {
                    break;
                }
            }
            Value::String(result)
        }
        Value::Object(map) => {
            let new_map: serde_json::Map<String, Value> = map.iter()
                .map(|(k, v)| (k.clone(), strip_tags_recursive(v, pre, post)))
                .collect();
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| strip_tags_recursive(v, pre, post)).collect())
        }
        other => other.clone(),
    }
}

/// Escape a CSV field value (handle commas, quotes, newlines)
fn escape_csv_field(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        let escaped = value.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        value.to_string()
    }
}

/// Generate a timestamp string for filename (YYYYMMDD_HHMMSS)
fn timestamp() -> String {
    let date = js_sys::Date::new_0();
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    let seconds = date.get_seconds();
    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        year, month, day, hours, minutes, seconds
    )
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = triggerCsvDownload)]
    fn trigger_csv_download(content: &str, filename: &str);
}

fn trigger_download(content: &str, filename: &str) {
    trigger_csv_download(content, filename);
}
