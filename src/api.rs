use std::collections::HashMap;
use serde_json::{json, Value};
use gloo_net::http::Request;
use crate::types::{ConnectData, IndexData, IndexListResponse, IndexInfo, IndexStats, IndexSettings, DocumentsResponse, SearchResponse, SearchHit, PopularSearchData, PopularItem};

pub fn apply_auth_header(mut builder: gloo_net::http::RequestBuilder, api_key: &str) -> gloo_net::http::RequestBuilder {
    let key = api_key.trim();
    if key.is_empty() {
        return builder;
    }
    builder = builder.header("Authorization", &format!("Bearer {}", key));
    builder.header("X-Meili-API-Key", key)
}

pub async fn connect_indexes(host: &str, api_key: &str) -> Result<ConnectData, String> {
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

pub async fn get_index_stats(host: &str, api_key: &str, uid: &str) -> Result<IndexStats, String> {
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

pub async fn load_index_data(host: &str, api_key: &str, uid: &str) -> Result<IndexData, String> {
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
    let mut set = std::collections::HashSet::new();
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

pub async fn perform_search(host: &str, api_key: &str, index: &str, query: &str, mut params: Value) -> Result<SearchResponse, String> {
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

pub async fn update_index_settings(host: &str, api_key: &str, index: &str, searchable: &[String], filterable: &[String]) -> Result<(), String> {
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

pub async fn load_popular_searches(host: &str, api_key: &str, index: &str, field: &str) -> Result<PopularSearchData, String> {
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

pub async fn fetch_by_id(host: &str, api_key: &str, index: &str, id: &str) -> Result<SearchHit, String> {
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

pub async fn update_documents(
    host: &str,
    api_key: &str,
    index: &str,
    primary_key: &str,
    hits: &[SearchHit],
    edits: &HashMap<String, HashMap<String, Value>>,
) -> Result<(), String> {
    use crate::cell::get_doc_key;
    use crate::parse::parse_edit_value;

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

// Create a new index with optional primary key
pub async fn create_index(host: &str, api_key: &str, uid: &str, primary_key: Option<&str>) -> Result<(), String> {
    let url = format!("{}/indexes", host.trim_end_matches('/'));
    let mut body = json!({ "uid": uid });
    if let Some(pk) = primary_key {
        body = json!({ "uid": uid, "primaryKey": pk });
    }
    let builder = Request::post(&url);
    // Build request progressively to avoid type inference issues
    let req = apply_auth_header(builder, api_key).json(&body);
    let req = match req {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    if resp.ok() {
        Ok(())
    } else {
        Err(resp.text().await.unwrap_or_else(|_| "create index failed".to_string()))
    }
}

// Set primary key for an existing index
pub async fn set_index_primary_key(host: &str, api_key: &str, uid: &str, primary_key: &str) -> Result<(), String> {
    let url = format!("{}/indexes/{}/primaryKey", host.trim_end_matches('/'), uid);
    let body = json!({ "primaryKey": primary_key });
    let builder = Request::put(&url);
    let req = apply_auth_header(builder, api_key).json(&body);
    let req = match req {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    if resp.ok() {
        Ok(())
    } else {
        Err(resp.text().await.unwrap_or_else(|_| "set primaryKey failed".to_string()))
    }
}

pub async fn batch_import_documents(
    host: &str,
    api_key: &str,
    index: &str,
    primary_key: &str,
    json_data: &str,
) -> Result<String, String> {
    let docs: Vec<serde_json::Value> = serde_json::from_str(json_data)
        .map_err(|e| format!("JSON解析失败: {}", e))?;
    
    if docs.is_empty() {
        return Err("JSON数组为空".to_string());
    }
    
    let count = docs.len();
    
    let url = format!("{}/indexes/{}/documents", host.trim_end_matches('/'), index);
    let builder = Request::post(&url);
    let req = apply_auth_header(builder, api_key).json(&docs);
    let req = match req {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    
    if resp.ok() {
        Ok(format!("成功导入 {} 条数据到索引 {}", count, index))
    } else {
        let err_text = resp.text().await.unwrap_or_else(|_| "批量导入失败".to_string());
        Err(format!("批量导入失败: {}", err_text))
    }
}
