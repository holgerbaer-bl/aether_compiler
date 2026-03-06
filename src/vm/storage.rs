use std::fs;
use std::path::Path;
use serde_json::Value;

const STORAGE_DIR: &str = ".knoten_data/storage";

pub fn store_value(key: &str, value: &Value) -> Result<(), String> {
    // Stellt sicher, dass das Verzeichnis existiert
    fs::create_dir_all(STORAGE_DIR).map_err(|e| e.to_string())?;
    
    let path = format!("{}/{}.json", STORAGE_DIR, key);
    let data = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_value(key: &str) -> Result<Value, String> {
    let path = format!("{}/{}.json", STORAGE_DIR, key);
    if !Path::new(&path).exists() {
        return Ok(Value::Null); // Fallback, falls noch nichts gespeichert wurde
    }
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}
