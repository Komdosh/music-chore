//! Metadata handling and structure definitions.

use std::collections::HashMap;

/// Complete metadata structure for a music item
#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    /// Embedded metadata from the file
    pub embedded: HashMap<String, String>,

    /// Inferred metadata from folder structure or filename
    pub inferred: HashMap<String, String>,

    /// User-edited metadata
    pub user_edited: HashMap<String, String>,

    /// Confidence scores for inferred fields (key -> confidence 0.0-1.0)
    pub confidence: HashMap<String, f32>,
}

impl Metadata {
    /// Creates a new empty metadata structure
    pub fn new() -> Self {
        Self {
            embedded: HashMap::new(),
            inferred: HashMap::new(),
            user_edited: HashMap::new(),
            confidence: HashMap::new(),
        }
    }

    /// Gets a value from metadata, checking in order: user_edited, inferred, embedded
    pub fn get(&self, key: &str) -> Option<&String> {
        self.user_edited
            .get(key)
            .or_else(|| self.inferred.get(key))
            .or_else(|| self.embedded.get(key))
    }

    /// Sets a value in user_edited metadata
    pub fn set_user_edited(&mut self, key: String, value: String) {
        self.user_edited.insert(key, value);
    }

    /// Sets a confidence score for an inferred field
    pub fn set_confidence(&mut self, key: String, confidence: f32) {
        self.confidence.insert(key, confidence);
    }
}