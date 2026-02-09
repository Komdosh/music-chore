//! Tests for the error module functionality.

use std::str::from_utf8;
use music_chore::core::errors::MusicChoreError;

#[test]
fn test_error_display_formatting() {
    // Test that each error variant formats correctly
    assert_eq!(
        format!("{}", MusicChoreError::IoError("file not found".to_string())),
        "I/O error: file not found"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::FormatNotSupported("mp4".to_string())),
        "Format not supported: mp4"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::FileNotFound("/path/to/file".to_string())),
        "File not found: /path/to/file"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::MetadataParseError("invalid format".to_string())),
        "Metadata parsing error: invalid format"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::InvalidMetadataField { 
            field: "title".to_string(), 
            value: "some value".to_string() 
        }),
        "Invalid value 'some value' for field 'title'"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::DirectoryAccessError("access denied".to_string())),
        "Directory access error: access denied"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::PermissionDenied("no permission".to_string())),
        "Permission denied: no permission"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::InvalidPath("/bad/path".to_string())),
        "Invalid path: /bad/path"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::UnsupportedAudioFormat("mp4".to_string())),
        "Unsupported audio format: mp4"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::InvalidConfiguration("invalid setting".to_string())),
        "Invalid configuration: invalid setting"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::ValidationError("missing field".to_string())),
        "Validation error: missing field"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::ProcessingError("failed".to_string())),
        "Processing error: failed"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::ConversionError("conversion failed".to_string())),
        "Conversion error: conversion failed"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::ChecksumError("checksum mismatch".to_string())),
        "Checksum error: checksum mismatch"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::CueFileError("cue parsing failed".to_string())),
        "CUE file error: cue parsing failed"
    );
    
    assert_eq!(
        format!("{}", MusicChoreError::Other("miscellaneous error".to_string())),
        "Error: miscellaneous error"
    );
}

#[test]
fn test_error_debug_formatting() {
    let error = MusicChoreError::FileNotFound("test.txt".to_string());
    let debug_str = format!("{:?}", error);
    
    assert!(debug_str.contains("FileNotFound"));
    assert!(debug_str.contains("test.txt"));
}

#[test]
fn test_error_equality_comparison() {
    let error1 = MusicChoreError::FileNotFound("file1.txt".to_string());
    let error2 = MusicChoreError::FileNotFound("file1.txt".to_string());
    let error3 = MusicChoreError::FileNotFound("file2.txt".to_string());
    
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[test]
fn test_error_serialization() {
    let error = MusicChoreError::IoError("test io error".to_string());
    let serialized = serde_json::to_string(&error).unwrap();
    
    assert!(serialized.contains("IoError"));
    assert!(serialized.contains("test io error"));
}

#[test]
fn test_error_from_io_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let converted_error: MusicChoreError = io_err.into();
    
    match converted_error {
        MusicChoreError::IoError(msg) => {
            assert!(msg.contains("file not found"));
        }
        _ => panic!("Expected IoError variant, got {:?}", converted_error),
    }
}

#[test]
fn test_error_from_json_error_conversion() {
    let json_err = serde_json::from_str::<serde_json::Value>("{ invalid json }");
    match json_err {
        Err(json_err) => {
            let converted_error: MusicChoreError = json_err.into();
            
            match converted_error {
                MusicChoreError::Other(msg) => {
                    assert!(msg.contains("JSON error"));
                }
                _ => panic!("Expected Other variant with JSON error, got {:?}", converted_error),
            }
        }
        Ok(_) => panic!("Expected JSON parsing error"),
    }
}

#[test]
fn test_error_from_utf8_error_conversion() {
    let bytes = vec![0xff, 0xff];
    let utf8_err = std::str::Utf8Error::from(from_utf8(&bytes).unwrap_err());
    let converted_error: MusicChoreError = utf8_err.into();
    
    match converted_error {
        MusicChoreError::Other(msg) => {
            assert!(msg.contains("UTF-8 error"));
        }
        _ => panic!("Expected Other variant with UTF-8 error, got {:?}", converted_error),
    }
}

#[test]
fn test_error_from_parse_int_error_conversion() {
    let parse_err = "abc".parse::<u32>().unwrap_err();
    let converted_error: MusicChoreError = parse_err.into();
    
    match converted_error {
        MusicChoreError::Other(msg) => {
            assert!(msg.contains("Integer parsing error"));
        }
        _ => panic!("Expected Other variant with Integer parsing error, got {:?}", converted_error),
    }
}

#[test]
fn test_error_from_parse_float_error_conversion() {
    let parse_err = "abc".parse::<f64>().unwrap_err();
    let converted_error: MusicChoreError = parse_err.into();
    
    match converted_error {
        MusicChoreError::Other(msg) => {
            assert!(msg.contains("Float parsing error"));
        }
        _ => panic!("Expected Other variant with Float parsing error, got {:?}", converted_error),
    }
}

#[test]
fn test_error_clone_functionality() {
    let original_error = MusicChoreError::ValidationError("test validation error".to_string());
    let cloned_error = original_error.clone();
    
    assert_eq!(original_error, cloned_error);
}

#[test]
fn test_error_enum_variants_exist() {
    // Test that all error variants can be created
    let _io_error = MusicChoreError::IoError("test error".to_string());
    let _format_error = MusicChoreError::FormatNotSupported("test format".to_string());
    let _file_not_found_error = MusicChoreError::FileNotFound("test file".to_string());
    let _metadata_error = MusicChoreError::MetadataParseError("test metadata".to_string());
    let _invalid_field_error = MusicChoreError::InvalidMetadataField { 
        field: "test_field".to_string(), 
        value: "test_value".to_string() 
    };
    let _dir_access_error = MusicChoreError::DirectoryAccessError("test dir".to_string());
    let _perm_error = MusicChoreError::PermissionDenied("test perm".to_string());
    let _path_error = MusicChoreError::InvalidPath("test path".to_string());
    let _audio_format_error = MusicChoreError::UnsupportedAudioFormat("test audio".to_string());
    let _config_error = MusicChoreError::InvalidConfiguration("test config".to_string());
    let _validation_error = MusicChoreError::ValidationError("test validation".to_string());
    let _processing_error = MusicChoreError::ProcessingError("test processing".to_string());
    let _conversion_error = MusicChoreError::ConversionError("test conversion".to_string());
    let _checksum_error = MusicChoreError::ChecksumError("test checksum".to_string());
    let _cue_error = MusicChoreError::CueFileError("test cue".to_string());
    let _other_error = MusicChoreError::Other("test other".to_string());

    // Verify they all exist and can be created
    assert!(true); // Just verifying all variants can be created without compilation errors
}