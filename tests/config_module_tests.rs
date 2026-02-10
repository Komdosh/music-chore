//! Tests for the configuration module functionality.

use music_chore::core::config::{
    AppConfig, AppConfigBuilder, FOLDER_INFERRED_CONFIDENCE, MAX_DISC_NUMBER, MAX_DURATION_SECONDS,
    MAX_TRACK_NUMBER, MAX_YEAR, MIN_YEAR,
};

#[test]
fn test_app_config_default_values() {
    let config = AppConfig::default();

    assert_eq!(config.max_recursion_depth, 10);
    assert_eq!(config.follow_symlinks, false);
    assert_eq!(config.default_inference_confidence, 0.3);
    assert_eq!(config.max_file_size_mb, 100);
    assert_eq!(config.verbose_by_default, false);
    assert_eq!(config.skip_metadata_by_default, false);
    assert!(config.excluded_extensions.contains(&"tmp".to_string()));
    assert!(config.excluded_directories.contains(&".git".to_string()));
    assert_eq!(config.show_source_indicators, true);
    assert_eq!(config.process_cue_files, true);
}

#[test]
fn test_app_config_new_creates_default() {
    let config = AppConfig::new();
    let default_config = AppConfig::default();

    assert_eq!(
        config.max_recursion_depth,
        default_config.max_recursion_depth
    );
    assert_eq!(config.follow_symlinks, default_config.follow_symlinks);
    assert_eq!(
        config.default_inference_confidence,
        default_config.default_inference_confidence
    );
    assert_eq!(config.max_file_size_mb, default_config.max_file_size_mb);
    assert_eq!(config.verbose_by_default, default_config.verbose_by_default);
    assert_eq!(
        config.skip_metadata_by_default,
        default_config.skip_metadata_by_default
    );
    assert_eq!(
        config.excluded_extensions,
        default_config.excluded_extensions
    );
    assert_eq!(
        config.excluded_directories,
        default_config.excluded_directories
    );
    assert_eq!(
        config.show_source_indicators,
        default_config.show_source_indicators
    );
    assert_eq!(config.process_cue_files, default_config.process_cue_files);
}

#[test]
fn test_app_config_builder_fluent_interface() {
    let config = AppConfigBuilder::new()
        .max_recursion_depth(5)
        .follow_symlinks(true)
        .default_inference_confidence(0.7)
        .max_file_size_mb(50)
        .verbose_by_default(true)
        .skip_metadata_by_default(true)
        .excluded_extensions(vec!["tmp".to_string(), "log".to_string()])
        .excluded_directories(vec![".git".to_string(), "temp".to_string()])
        .show_source_indicators(false)
        .process_cue_files(false)
        .build()
        .unwrap();

    assert_eq!(config.max_recursion_depth, 5);
    assert_eq!(config.follow_symlinks, true);
    assert_eq!(config.default_inference_confidence, 0.7);
    assert_eq!(config.max_file_size_mb, 50);
    assert_eq!(config.verbose_by_default, true);
    assert_eq!(config.skip_metadata_by_default, true);
    assert_eq!(
        config.excluded_extensions,
        vec!["tmp".to_string(), "log".to_string()]
    );
    assert_eq!(
        config.excluded_directories,
        vec![".git".to_string(), "temp".to_string()]
    );
    assert_eq!(config.show_source_indicators, false);
    assert_eq!(config.process_cue_files, false);
}

#[test]
fn test_app_config_builder_default_values() {
    let config = AppConfigBuilder::new().build().unwrap();
    let default_config = AppConfig::default();

    assert_eq!(
        config.max_recursion_depth,
        default_config.max_recursion_depth
    );
    assert_eq!(config.follow_symlinks, default_config.follow_symlinks);
    assert_eq!(
        config.default_inference_confidence,
        default_config.default_inference_confidence
    );
    assert_eq!(config.max_file_size_mb, default_config.max_file_size_mb);
    assert_eq!(config.verbose_by_default, default_config.verbose_by_default);
    assert_eq!(
        config.skip_metadata_by_default,
        default_config.skip_metadata_by_default
    );
    assert_eq!(
        config.excluded_extensions,
        default_config.excluded_extensions
    );
    assert_eq!(
        config.excluded_directories,
        default_config.excluded_directories
    );
    assert_eq!(
        config.show_source_indicators,
        default_config.show_source_indicators
    );
    assert_eq!(config.process_cue_files, default_config.process_cue_files);
}

#[test]
fn test_app_config_validation_valid_values() {
    let config = AppConfigBuilder::new()
        .max_recursion_depth(50)
        .default_inference_confidence(0.5)
        .max_file_size_mb(50)
        .build()
        .unwrap();

    assert!(config.validate().is_ok());
}

#[test]
fn test_app_config_validation_invalid_recursion_depth() {
    let result = AppConfigBuilder::new()
        .max_recursion_depth(101) // Too high
        .build();

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Maximum recursion depth should not exceed")
    );
}

#[test]
fn test_app_config_validation_invalid_confidence_low() {
    let result = AppConfigBuilder::new()
        .default_inference_confidence(-0.1) // Below 0
        .build();

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Default inference confidence must be between")
    );
}

#[test]
fn test_app_config_validation_invalid_confidence_high() {
    let result = AppConfigBuilder::new()
        .default_inference_confidence(1.1) // Above 1
        .build();

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Default inference confidence must be between")
    );
}

#[test]
fn test_app_config_validation_zero_file_size() {
    let result = AppConfigBuilder::new()
        .max_file_size_mb(0) // Zero is invalid
        .build();

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Maximum file size must be greater than")
    );
}

#[test]
fn test_app_config_constants_exist() {
    // Test that all constants are accessible and have expected values
    assert_eq!(MAX_TRACK_NUMBER, 999);
    assert_eq!(MAX_DISC_NUMBER, 99);
    assert_eq!(MIN_YEAR, 1000);
    assert_eq!(MAX_YEAR, 3000);
    assert_eq!(MAX_DURATION_SECONDS, 36000.0);
    assert_eq!(FOLDER_INFERRED_CONFIDENCE, 0.3);
}

#[test]
fn test_app_config_save_and_load() {
    use tempfile::TempDir;
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let original_config = AppConfigBuilder::new()
        .max_recursion_depth(15)
        .follow_symlinks(true)
        .default_inference_confidence(0.8)
        .max_file_size_mb(200)
        .build()
        .unwrap();

    // Save the config
    assert!(original_config.save_to_file(&config_path).is_ok());

    // Load the config
    let loaded_config = AppConfig::load_from_file(&config_path).unwrap();

    assert_eq!(
        original_config.max_recursion_depth,
        loaded_config.max_recursion_depth
    );
    assert_eq!(
        original_config.follow_symlinks,
        loaded_config.follow_symlinks
    );
    assert_eq!(
        original_config.default_inference_confidence,
        loaded_config.default_inference_confidence
    );
    assert_eq!(
        original_config.max_file_size_mb,
        loaded_config.max_file_size_mb
    );
    assert_eq!(
        original_config.verbose_by_default,
        loaded_config.verbose_by_default
    );
    assert_eq!(
        original_config.skip_metadata_by_default,
        loaded_config.skip_metadata_by_default
    );
    assert_eq!(
        original_config.excluded_extensions,
        loaded_config.excluded_extensions
    );
    assert_eq!(
        original_config.excluded_directories,
        loaded_config.excluded_directories
    );
    assert_eq!(
        original_config.show_source_indicators,
        loaded_config.show_source_indicators
    );
    assert_eq!(
        original_config.process_cue_files,
        loaded_config.process_cue_files
    );
}

#[test]
fn test_app_config_load_nonexistent_file() {
    let nonexistent_path = std::path::PathBuf::from("/nonexistent/config.json");

    let result = AppConfig::load_from_file(&nonexistent_path);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("does not exist") || error_msg.contains("No such file"));
}

#[test]
fn test_app_config_save_permissions_error() {
    // Try to save to a path we don't have permissions for
    let protected_path = std::path::PathBuf::from("/config.json");

    let config = AppConfig::default();

    let result = config.save_to_file(&protected_path);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("I/O error"));
}

#[test]
fn test_app_config_builder_individual_methods() {
    let config = AppConfigBuilder::new()
        .max_recursion_depth(20)
        .build()
        .unwrap();
    assert_eq!(config.max_recursion_depth, 20);

    let config = AppConfigBuilder::new()
        .follow_symlinks(true)
        .build()
        .unwrap();
    assert_eq!(config.follow_symlinks, true);

    let config = AppConfigBuilder::new()
        .default_inference_confidence(0.9)
        .build()
        .unwrap();
    assert_eq!(config.default_inference_confidence, 0.9);

    let config = AppConfigBuilder::new()
        .max_file_size_mb(250)
        .build()
        .unwrap();
    assert_eq!(config.max_file_size_mb, 250);

    let config = AppConfigBuilder::new()
        .verbose_by_default(true)
        .build()
        .unwrap();
    assert_eq!(config.verbose_by_default, true);

    let config = AppConfigBuilder::new()
        .skip_metadata_by_default(true)
        .build()
        .unwrap();
    assert_eq!(config.skip_metadata_by_default, true);

    let config = AppConfigBuilder::new()
        .show_source_indicators(false)
        .build()
        .unwrap();
    assert_eq!(config.show_source_indicators, false);

    let config = AppConfigBuilder::new()
        .process_cue_files(false)
        .build()
        .unwrap();
    assert_eq!(config.process_cue_files, false);
}
