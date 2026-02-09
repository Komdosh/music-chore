//! Tests for AppConfig functionality

use music_chore::core::configuration::{AppConfig, AppConfigBuilder};
use music_chore::core::errors::MusicChoreError;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();

    assert_eq!(config.max_recursion_depth, 10);
    assert_eq!(config.follow_symlinks, false);
    assert_eq!(config.default_inference_confidence, 0.3);
    assert_eq!(config.max_file_size_mb, 100);
    assert_eq!(config.verbose_by_default, false);
    assert_eq!(config.skip_metadata_by_default, false);
    assert_eq!(config.show_source_indicators, true);
    
    // Check excluded extensions
    assert!(config.excluded_extensions.contains(&"tmp".to_string()));
    assert!(config.excluded_extensions.contains(&"log".to_string()));
    assert!(config.excluded_extensions.contains(&"bak".to_string()));
    
    // Check excluded directories
    assert!(config.excluded_directories.contains(&".git".to_string()));
    assert!(config.excluded_directories.contains(&"node_modules".to_string()));
    assert!(config.excluded_directories.contains(&".DS_Store".to_string()));
}

#[test]
fn test_app_config_validate_valid() {
    let config = AppConfig::default();
    let result = config.validate();

    assert!(result.is_ok());
}

#[test]
fn test_app_config_validate_max_recursion_depth_exceeded() {
    let mut config = AppConfig::default();
    config.max_recursion_depth = 101;

    let result = config.validate();

    assert!(result.is_err());
}

#[test]
fn test_app_config_validate_max_recursion_at_boundary() {
    let mut config = AppConfig::default();
    config.max_recursion_depth = 100;

    let result = config.validate();

    assert!(result.is_ok());
}

#[test]
fn test_app_config_validate_negative_confidence() {
    let mut config = AppConfig::default();
    config.default_inference_confidence = -0.1;

    let result = config.validate();

    assert!(result.is_err());
}

#[test]
fn test_app_config_validate_confidence_above_max() {
    let mut config = AppConfig::default();
    config.default_inference_confidence = 1.1;

    let result = config.validate();

    assert!(result.is_err());
}

#[test]
fn test_app_config_validate_confidence_at_boundaries() {
    let mut config = AppConfig::default();
    
    config.default_inference_confidence = 0.0;
    assert!(config.validate().is_ok());
    
    config.default_inference_confidence = 1.0;
    assert!(config.validate().is_ok());
}

#[test]
fn test_app_config_validate_zero_file_size() {
    let mut config = AppConfig::default();
    config.max_file_size_mb = 0;

    let result = config.validate();

    assert!(result.is_err());
}

#[test]
fn test_app_config_validate_positive_file_size() {
    let mut config = AppConfig::default();
    config.max_file_size_mb = 1;

    let result = config.validate();

    assert!(result.is_ok());
}

#[test]
fn test_app_config_load_from_file_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    
    // Create original config
    let mut original_config = AppConfig::default();
    original_config.max_recursion_depth = 5;
    original_config.follow_symlinks = true;
    original_config.default_inference_confidence = 0.5;
    original_config.max_file_size_mb = 50;
    original_config.verbose_by_default = true;
    original_config.skip_metadata_by_default = true;
    original_config.excluded_extensions = vec!["tmp".to_string()];
    original_config.excluded_directories = vec![];
    
    // Save config
    original_config.save_to_file(&config_path).unwrap();
    
    // Load config
    let loaded_config = AppConfig::load_from_file(&config_path).unwrap();
    
    // Verify roundtrip
    assert_eq!(loaded_config.max_recursion_depth, original_config.max_recursion_depth);
    assert_eq!(loaded_config.follow_symlinks, original_config.follow_symlinks);
    assert_eq!(loaded_config.default_inference_confidence, original_config.default_inference_confidence);
    assert_eq!(loaded_config.max_file_size_mb, original_config.max_file_size_mb);
    assert_eq!(loaded_config.verbose_by_default, original_config.verbose_by_default);
    assert_eq!(loaded_config.skip_metadata_by_default, original_config.skip_metadata_by_default);
    assert_eq!(loaded_config.show_source_indicators, original_config.show_source_indicators);
}

#[test]
fn test_app_config_load_from_file_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.json");
    
    // Write invalid JSON
    std::fs::write(&config_path, "{ invalid json }").unwrap();
    
    let result = AppConfig::load_from_file(&config_path);

    assert!(result.is_err());
}

#[test]
fn test_app_config_load_from_file_missing_file() {
    let result = AppConfig::load_from_file(PathBuf::from("/nonexistent/config.json"));

    assert!(result.is_err());
}

#[test]
fn test_app_config_save_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    
    let config = AppConfig::default();
    config.save_to_file(&config_path).unwrap();

    assert!(config_path.exists());
    
    // Verify file content is valid JSON
    let content = std::fs::read_to_string(&config_path).unwrap();
    let loaded: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(loaded.is_object());
}

#[test]
fn test_app_config_builder_new() {
    let builder = AppConfigBuilder::new();
    
    // Should create builder with default config
    let default_config = AppConfig::default();
    assert_eq!(builder.config.max_recursion_depth, default_config.max_recursion_depth);
}

#[test]
fn test_app_config_builder_max_recursion_depth() {
    let builder = AppConfigBuilder::new()
        .max_recursion_depth(15);

    assert_eq!(builder.config.max_recursion_depth, 15);
}

#[test]
fn test_app_config_builder_follow_symlinks() {
    let builder = AppConfigBuilder::new()
        .follow_symlinks(true);

    assert_eq!(builder.config.follow_symlinks, true);
}

#[test]
fn test_app_config_builder_default_inference_confidence() {
    let builder = AppConfigBuilder::new()
        .default_inference_confidence(0.5);

    assert_eq!(builder.config.default_inference_confidence, 0.5);
}

#[test]
fn test_app_config_builder_max_file_size() {
    let builder = AppConfigBuilder::new()
        .max_file_size_mb(250);

    assert_eq!(builder.config.max_file_size_mb, 250);
}

#[test]
fn test_app_config_builder_verbose_by_default() {
    let builder = AppConfigBuilder::new()
        .verbose_by_default(true);

    assert_eq!(builder.config.verbose_by_default, true);
}

#[test]
fn test_app_config_builder_skip_metadata_by_default() {
    let builder = AppConfigBuilder::new()
        .skip_metadata_by_default(true);

    assert_eq!(builder.config.skip_metadata_by_default, true);
}

#[test]
fn test_app_config_builder_excluded_extensions() {
    let builder = AppConfigBuilder::new()
        .excluded_extensions(vec!["tmp".to_string(), "log".to_string()])
        .build();

    assert!(builder.is_ok());
    let config = builder.unwrap();
    assert_eq!(config.excluded_extensions.len(), 2);
    assert!(config.excluded_extensions.contains(&"tmp".to_string()));
}

#[test]
fn test_app_config_builder_excluded_directories() {
    let builder = AppConfigBuilder::new()
        .excluded_directories(vec!["node_modules".to_string()])
        .build();

    assert!(builder.is_ok());
    let config = builder.unwrap();
    assert_eq!(config.excluded_directories.len(), 1);
    assert_eq!(config.excluded_directories[0], "node_modules");
}

#[test]
fn test_app_config_builder_show_source_indicators() {
    let builder = AppConfigBuilder::new()
        .show_source_indicators(false);

    assert_eq!(builder.config.show_source_indicators, false);
}

#[test]
fn test_app_config_builder_chaining() {
    let builder = AppConfigBuilder::new()
        .max_recursion_depth(5)
        .follow_symlinks(true)
        .default_inference_confidence(0.5)
        .max_file_size_mb(50)
        .verbose_by_default(true)
        .skip_metadata_by_default(false)
        .show_source_indicators(true)
        .build();

    assert!(builder.is_ok());
    let config = builder.unwrap();
    assert_eq!(config.max_recursion_depth, 5);
    assert_eq!(config.follow_symlinks, true);
    assert_eq!(config.default_inference_confidence, 0.5);
    assert_eq!(config.max_file_size_mb, 50);
    assert_eq!(config.verbose_by_default, true);
    assert_eq!(config.skip_metadata_by_default, false);
    assert_eq!(config.show_source_indicators, true);
}

#[test]
fn test_app_config_builder_build_valid() {
    let builder = AppConfigBuilder::new()
        .max_recursion_depth(15)
        .build();

    assert!(builder.is_ok());
    let config = builder.unwrap();
    assert_eq!(config.max_recursion_depth, 15);
}

#[test]
fn test_app_config_builder_edge_cases() {
    // Test minimum valid values
    let result1 = AppConfigBuilder::new()
        .max_recursion_depth(1)
        .default_inference_confidence(0.0)
        .max_file_size_mb(1)
        .build();

    assert!(matches!(&result1, Ok(_)));

    // Test maximum valid values
    let result2 = AppConfigBuilder::new()
        .max_recursion_depth(100)
        .default_inference_confidence(1.0)
        .max_file_size_mb(10000)
        .build();

    assert!(matches!(&result2, Ok(_)));
}

#[test]
fn test_app_config_serialize_deserialize() {
    let config = AppConfig::default();

    // Serialize
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("max_recursion_depth"));
    assert!(json.contains("follow_symlinks"));

    // Deserialize
    let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.max_recursion_depth, config.max_recursion_depth);
}
