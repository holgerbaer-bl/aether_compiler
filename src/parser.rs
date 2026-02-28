use crate::ast::Node;
use std::fs;
use std::io::Error as IoError;

pub struct Parser;

impl Parser {
    /// Loads a compiled KnotenCore AST from a JSON file on disk.
    pub fn parse_file(path: &str) -> Result<Node, String> {
        let text_data =
            fs::read(path).map_err(|e: IoError| format!("Failed to read file {}: {}", path, e))?;
        Self::parse_bytes(&text_data)
    }

    /// Deserializes in-memory JSON bytes into a structural Node.
    pub fn parse_bytes(data: &[u8]) -> Result<Node, String> {
        serde_json::from_slice(data).map_err(|e| format!("JSON parser error: {}", e))
    }
}
