use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt;

use crate::shared::error::domain_error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogMetadata {
    data: Map<String, Value>,
}

impl LogMetadata {
    pub fn new() -> Self {
        Self {
            data: Map::new(),
        }
    }

    pub fn empty() -> Self {
        Self::new()
    }

    pub fn with_duration(duration_ms: u64) -> Self {
        let mut metadata = Self::new();
        metadata.add_duration_ms(duration_ms);
        metadata
    }

    pub fn from_json(value: Value) -> Result<Self, DomainError> {
        match value {
            Value::Object(map) => Ok(Self { data: map }),
            _ => Err(DomainError::InvalidValue(
                "Metadata must be a JSON object".to_string(),
            )),
        }
    }

    pub fn from_hashmap(map: HashMap<String, Value>) -> Self {
        Self {
            data: map.into_iter().collect(),
        }
    }

    pub fn add_string(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), Value::String(value.to_string()));
    }

    pub fn add_number(&mut self, key: &str, value: i64) {
        self.data.insert(key.to_string(), Value::Number(value.into()));
    }

    pub fn add_bool(&mut self, key: &str, value: bool) {
        self.data.insert(key.to_string(), Value::Bool(value));
    }

    pub fn add_duration_ms(&mut self, duration_ms: u64) {
        self.add_number("duration_ms", duration_ms as i64);
    }

    pub fn add_error_info(&mut self, error_type: &str, error_message: &str) {
        self.add_string("error_type", error_type);
        self.add_string("error_message", error_message);
    }

    pub fn add_database_info(&mut self, table: &str, operation: &str, affected_rows: Option<u64>) {
        self.add_string("db_table", table);
        self.add_string("db_operation", operation);
        if let Some(rows) = affected_rows {
            self.add_number("db_affected_rows", rows as i64);
        }
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.data.get(key)?.as_str()
    }

    pub fn get_number(&self, key: &str) -> Option<i64> {
        self.data.get(key)?.as_i64()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)?.as_bool()
    }

    pub fn get_duration_ms(&self) -> Option<u64> {
        self.get_number("duration_ms").map(|n| n as u64)
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn to_json(&self) -> Value {
        Value::Object(self.data.clone())
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }
}

impl Default for LogMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for LogMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.data.is_empty() {
            write!(f, "{{}}")
        } else {
            write!(f, "{}", serde_json::to_string(&self.data).unwrap_or_default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_log_metadata_new() {
        let metadata = LogMetadata::new();
        assert!(metadata.is_empty());
        assert_eq!(metadata.len(), 0);
    }

    #[test]
    fn test_add_values() {
        let mut metadata = LogMetadata::new();
        metadata.add_string("key1", "value1");
        metadata.add_number("key2", 42);
        metadata.add_bool("key3", true);

        assert_eq!(metadata.get_string("key1"), Some("value1"));
        assert_eq!(metadata.get_number("key2"), Some(42));
        assert_eq!(metadata.get_bool("key3"), Some(true));
        assert_eq!(metadata.len(), 3);
    }

    #[test]
    fn test_duration() {
        let metadata = LogMetadata::with_duration(1500);
        assert_eq!(metadata.get_duration_ms(), Some(1500));
    }

    #[test]
    fn test_error_info() {
        let mut metadata = LogMetadata::new();
        metadata.add_error_info("ValidationError", "Invalid email format");

        assert_eq!(metadata.get_string("error_type"), Some("ValidationError"));
        assert_eq!(metadata.get_string("error_message"), Some("Invalid email format"));
    }

    #[test]
    fn test_from_json() {
        let json_value = json!({
            "test_key": "test_value",
            "number_key": 123
        });

        let metadata = LogMetadata::from_json(json_value).unwrap();
        assert_eq!(metadata.get_string("test_key"), Some("test_value"));
        assert_eq!(metadata.get_number("number_key"), Some(123));
    }

    #[test]
    fn test_from_json_invalid() {
        let json_value = json!("not an object");
        assert!(LogMetadata::from_json(json_value).is_err());
    }

    #[test]
    fn test_database_info() {
        let mut metadata = LogMetadata::new();
        metadata.add_database_info("users", "INSERT", Some(1));

        assert_eq!(metadata.get_string("db_table"), Some("users"));
        assert_eq!(metadata.get_string("db_operation"), Some("INSERT"));
        assert_eq!(metadata.get_number("db_affected_rows"), Some(1));
    }
}