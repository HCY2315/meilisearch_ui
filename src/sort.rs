use crate::types::SearchHit;
use crate::utils::value_to_string;
use serde_json::Value;

pub struct SortValue {
    pub is_number: bool,
    pub num_value: f64,
    pub str_value: String,
}

pub fn sort_hits(mut hits: Vec<SearchHit>, field: &str, dir: &str) -> Vec<SearchHit> {
    let factor = if dir == "desc" { -1.0 } else { 1.0 };
    hits.sort_by(|a, b| {
        let av = normalize_sort_value(a.get(field));
        let bv = normalize_sort_value(b.get(field));
        if av.is_number && bv.is_number {
            return av
                .num_value
                .partial_cmp(&bv.num_value)
                .unwrap_or(std::cmp::Ordering::Equal);
        }
        av.str_value.cmp(&bv.str_value)
    });
    if factor < 0.0 {
        hits.reverse();
    }
    hits
}

pub fn normalize_sort_value(value: Option<&Value>) -> SortValue {
    if value.is_none() {
        return SortValue {
            is_number: false,
            num_value: 0.0,
            str_value: "".to_string(),
        };
    }
    let value = value.unwrap();
    if let Some(n) = value.as_f64() {
        return SortValue {
            is_number: true,
            num_value: n,
            str_value: n.to_string(),
        };
    }
    if value.is_array() {
        if let Some(first) = value
            .as_array()
            .and_then(|arr| arr.iter().find(|v| !v.is_null()))
        {
            return normalize_sort_value(Some(first));
        }
    }
    if value.is_object() {
        return SortValue {
            is_number: false,
            num_value: 0.0,
            str_value: serde_json::to_string(value).unwrap_or_default(),
        };
    }
    SortValue {
        is_number: false,
        num_value: 0.0,
        str_value: value_to_string(value),
    }
}
