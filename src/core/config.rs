//! Configuration system for the music chore application.

use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Maximum recursion depth for directory scanning (0 = immediate files only)
    pub max_recursion_depth: usize,
    
    /// Whether to follow symbolic links during scanning
    pub follow_symlinks: bool,
    
    /// Confidence threshold for folder-inferred metadata
    pub default_inference_confidence: f32,
    
    /// Maximum file size in MB that the application will process
    pub max_file_size_mb: u64,
    
    /// Whether to enable verbose output by default
    pub verbose_by_default: bool,
    
    /// Whether to skip metadata reading by default (use filename inference only)
    pub skip_metadata_by_default: bool,
    
    /// List of file extensions to exclude from processing
    pub excluded_extensions: Vec<String>,
    
    /// List of directory names to exclude from scanning
    pub excluded_directories: Vec<String>,
    
    /// Whether to show source indicators (emojis) in output
    pub show_source_indicators: bool,
    
    /// Whether to process CUE files by default
    pub process_cue_files: bool,
}

// Define constants that were previously used in the codebase
pub const MAX_TRACK_NUMBER: u32 = 999;
pub const MAX_DISC_NUMBER: u32 = 99;
pub const MIN_YEAR: u32 = 1000;
pub const MAX_YEAR: u32 = 3000;
pub const MAX_DURATION_SECONDS: f64 = 36000.0; // 10 hours max
pub const FOLDER_INFERRED_CONFIDENCE: f32 = 0.3;
pub const FILE_BUFFER_SIZE: usize = 8192;
pub const MAX_FILE_SIZE_MB: u64 = 100;

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: 10,
            follow_symlinks: false,
            default_inference_confidence: 0.3,
            max_file_size_mb: 100,
            verbose_by_default: false,
            skip_metadata_by_default: false,
            excluded_extensions: vec![
                "tmp".to_string(),
                "log".to_string(),
                "bak".to_string(),
                "cache".to_string(),
            ],
            excluded_directories: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                ".svn".to_string(),
            ],
            show_source_indicators: true,
            process_cue_files: true,
        }
    }
}

impl AppConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Load configuration from a file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, crate::core::errors::MusicChoreError> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), crate::core::errors::MusicChoreError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Validate the configuration values
    pub fn validate(&self) -> Result<(), crate::core::errors::MusicChoreError> {
        if self.max_recursion_depth > 100 {
            return Err(crate::core::errors::MusicChoreError::InvalidConfiguration(
                "Maximum recursion depth should not exceed 100".to_string()
            ));
        }
        
        if self.default_inference_confidence < 0.0 || self.default_inference_confidence > 1.0 {
            return Err(crate::core::errors::MusicChoreError::InvalidConfiguration(
                "Default inference confidence must be between 0.0 and 1.0".to_string()
            ));
        }
        
        if self.max_file_size_mb == 0 {
            return Err(crate::core::errors::MusicChoreError::InvalidConfiguration(
                "Maximum file size must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Configuration builder for fluent configuration creation
#[derive(Default)]
pub struct AppConfigBuilder {
    config: AppConfig,
}

impl AppConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }
    
    /// Set the maximum recursion depth
    pub fn max_recursion_depth(mut self, depth: usize) -> Self {
        self.config.max_recursion_depth = depth;
        self
    }
    
    /// Set whether to follow symbolic links
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.config.follow_symlinks = follow;
        self
    }
    
    /// Set the default inference confidence
    pub fn default_inference_confidence(mut self, confidence: f32) -> Self {
        self.config.default_inference_confidence = confidence;
        self
    }
    
    /// Set the maximum file size in MB
    pub fn max_file_size_mb(mut self, size: u64) -> Self {
        self.config.max_file_size_mb = size;
        self
    }
    
    /// Set whether to enable verbose output by default
    pub fn verbose_by_default(mut self, verbose: bool) -> Self {
        self.config.verbose_by_default = verbose;
        self
    }
    
    /// Set whether to skip metadata reading by default
    pub fn skip_metadata_by_default(mut self, skip: bool) -> Self {
        self.config.skip_metadata_by_default = skip;
        self
    }
    
    /// Set the list of excluded file extensions
    pub fn excluded_extensions(mut self, extensions: Vec<String>) -> Self {
        self.config.excluded_extensions = extensions;
        self
    }
    
    /// Set the list of excluded directories
    pub fn excluded_directories(mut self, directories: Vec<String>) -> Self {
        self.config.excluded_directories = directories;
        self
    }
    
    /// Set whether to show source indicators
    pub fn show_source_indicators(mut self, show: bool) -> Self {
        self.config.show_source_indicators = show;
        self
    }
    
    /// Set whether to process CUE files by default
    pub fn process_cue_files(mut self, process: bool) -> Self {
        self.config.process_cue_files = process;
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> Result<AppConfig, crate::core::errors::MusicChoreError> {
        self.config.validate()?;
        Ok(self.config)
    }
}