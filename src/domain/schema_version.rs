//! Schema version wrapper for JSON output

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersionWrapper<T> {
    /// Schema version for this API
    #[serde(rename = "__schema_version")]
    pub schema_version: String,
    /// The actual data
    #[serde(flatten)]
    pub data: T,
}

impl<T> SchemaVersionWrapper<T> {
    /// Create a new wrapper with the current schema version
    pub fn new(data: T) -> Self {
        Self {
            schema_version: "1.0.0".to_string(),
            data,
        }
    }
}

/// Create a schema version wrapper with the current version
pub fn with_schema_version<T>(data: T) -> SchemaVersionWrapper<T> {
    SchemaVersionWrapper::new(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_wrapper_creation() {
        let data = vec!["item1", "item2"];
        let wrapper = SchemaVersionWrapper::new(data.clone());

        assert_eq!(wrapper.schema_version, "1.0.0");
        assert_eq!(wrapper.data, data);
    }

    #[test]
    fn test_with_schema_version() {
        let data = "test data";
        let wrapper = with_schema_version(data);

        assert_eq!(wrapper.schema_version, "1.0.0");
        assert_eq!(wrapper.data, data);
    }

    #[test]
    fn test_schema_version_serialization() {
        #[derive(Serialize, Deserialize)]
        struct TestData {
            field1: String,
            field2: i32,
        }

        let data = TestData {
            field1: "value1".to_string(),
            field2: 42,
        };
        let wrapper = with_schema_version(data);

        let json = serde_json::to_string(&wrapper).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["__schema_version"], "1.0.0");
        assert_eq!(parsed["field1"], "value1");
        assert_eq!(parsed["field2"], 42);
    }

    #[test]
    fn test_schema_version_deserialization() {
        let json_str = r#"{"__schema_version": "1.0.0", "name": "Test Album"}"#;
        let parsed: SchemaVersionWrapper<serde_json::Value> =
            serde_json::from_str(json_str).unwrap();

        assert_eq!(parsed.schema_version, "1.0.0");
        assert_eq!(parsed.data["name"], "Test Album");
    }

    #[test]
    fn test_schema_version_with_complex_type() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestStruct {
            id: i32,
            name: String,
        }

        let data = TestStruct {
            id: 123,
            name: "Test".to_string(),
        };
        let wrapper = with_schema_version(data);

        assert_eq!(wrapper.schema_version, "1.0.0");
        assert_eq!(wrapper.data.id, 123);
        assert_eq!(wrapper.data.name, "Test");
    }
}
