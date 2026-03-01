use crate::executor::RelType;
use std::collections::HashMap;

/// Reads a file and returns its contents as a String.
pub fn fs_read_file(path: String) -> String {
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("[KnotenCore FS] Error reading '{}': {}", path, e);
        String::new()
    })
}

/// Parses a JSON string into a nested RelType structure.
/// - JSON Object → RelType::Object(HashMap)
/// - JSON Array → RelType::Array(Vec)
/// - JSON String → RelType::Str
/// - JSON Number → RelType::Int or RelType::Float
/// - JSON Bool → RelType::Bool
/// - JSON Null → RelType::Void
pub fn fs_parse_json(json_str: &str) -> RelType {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(value) => json_value_to_reltype(&value),
        Err(e) => {
            eprintln!("[KnotenCore FS] JSON parse error: {}", e);
            RelType::Void
        }
    }
}

fn json_value_to_reltype(value: &serde_json::Value) -> RelType {
    match value {
        serde_json::Value::Null => RelType::Void,
        serde_json::Value::Bool(b) => RelType::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                RelType::Int(i)
            } else if let Some(f) = n.as_f64() {
                RelType::Float(f)
            } else {
                RelType::Int(0)
            }
        }
        serde_json::Value::String(s) => RelType::Str(s.clone()),
        serde_json::Value::Array(arr) => {
            RelType::Array(arr.iter().map(json_value_to_reltype).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_value_to_reltype(v));
            }
            RelType::Object(map)
        }
    }
}
