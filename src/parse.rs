use serde_json::Value;

pub fn parse_edit_value(value: &Value) -> Value {
    match value {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Value::String("".to_string());
            }
            if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
                return parsed;
            }
            if trimmed.eq_ignore_ascii_case("true") {
                return Value::Bool(true);
            }
            if trimmed.eq_ignore_ascii_case("false") {
                return Value::Bool(false);
            }
            if let Ok(num) = trimmed.parse::<f64>() {
                return Value::Number(
                    serde_json::Number::from_f64(num)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                );
            }
            Value::String(s.clone())
        }
        other => other.clone(),
    }
}
