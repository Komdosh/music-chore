# Music Chore Testing Plan

**Document Version:** 1.0  
**Date:** 2026-02-08  
**Author:** Staff Rust Engineer  
**Purpose:** Comprehensive plan to improve test coverage across the music-chore codebase

---

## Executive Summary

This document provides a detailed analysis of the current test coverage and a concrete implementation plan for improving tests across the music-chore CLI tool. The codebase has ~82 public functions across multiple modules, with varying levels of test coverage.

**Current State:**
- 166 tests running (mix of unit and integration tests)
- Strong coverage in: `inference.rs`, `normalization.rs`, `cue.rs`
- Weak coverage in: `scanner.rs`, `format_tree.rs`, `library.rs`, `apply_metadata.rs`
- Missing tests for: Error handling paths, edge cases, MCP server functions

**Target State:**
- Minimum 90% line coverage for all modules
- Every public function must have at least one test
- All error paths must be tested
- Integration tests for all CLI commands

---

## Module-by-Module Testing Plan

### 1. Core Domain Models (`src/core/domain/models.rs`)

**Current Coverage:** Basic unit tests exist

**Public APIs to Test:**

#### MetadataValue<T>
- [x] `embedded()` - Create embedded metadata value
- [x] `inferred()` - Create inferred metadata value
- [x] `user_set()` - Create user-edited metadata value
- [x] `cue_inferred()` - Create CUE-inferred metadata value
- [ ] `Display` trait implementation for MetadataValue

**Missing Tests:**
1. Test that confidence values are correctly set for each source type
2. Test Display trait formatting for different T types (String, u32)
3. Test MetadataValue with generic types beyond String and u32

**Implementation Steps:**
```rust
#[test]
fn test_metadata_value_display() {
    let mv = MetadataValue::embedded("Test".to_string());
    assert_eq!(format!("{}", mv), "Test");
}

#[test]
fn test_metadata_value_confidence_levels() {
    assert_eq!(MetadataValue::embedded("x".to_string()).confidence, 1.0);
    assert_eq!(MetadataValue::user_set("x".to_string()).confidence, 1.0);
    assert_eq!(MetadataValue::inferred("x".to_string(), 0.5).confidence, 0.5);
}
```

---

#### Track
- [x] `new()` - Create track without checksum
- [x] `with_checksum()` - Create track with checksum
- [x] `calculate_checksum()` - Calculate SHA256 checksum

**Missing Tests:**
1. Test calculate_checksum on empty file
2. Test calculate_checksum on very large file
3. Test calculate_checksum produces different hashes for different content
4. Test calculate_checksum error handling for unreadable files

**Implementation Steps:**
```rust
#[test]
fn test_calculate_checksum_empty_file() {
    // Create empty file and verify error or specific hash
}

#[test]
fn test_calculate_checksum_large_file() {
    // Create file larger than buffer size (8192 bytes)
    // Verify complete file is hashed
}

#[test]
fn test_calculate_checksum_unreadable_file() {
    // Create file, remove read permissions, verify error
}
```

---

#### Library
- [x] `new()` - Create empty library
- [x] `add_artist()` - Add artist to library

**Missing Tests:**
1. Test that add_artist correctly increments all counters
2. Test Library default implementation
3. Test Library serialization/deserialization roundtrip

**Implementation Steps:**
```rust
#[test]
fn test_library_counters() {
    let mut lib = Library::new();
    let artist = create_test_artist_with_albums(2, 5); // 2 albums, 5 tracks each
    lib.add_artist(artist);
    assert_eq!(lib.total_artists, 1);
    assert_eq!(lib.total_albums, 2);
    assert_eq!(lib.total_tracks, 10);
}

#[test]
fn test_library_serialization_roundtrip() {
    let lib = create_test_library();
    let json = serde_json::to_string(&lib).unwrap();
    let restored: Library = serde_json::from_str(&json).unwrap();
    assert_eq!(lib, restored);
}
```

---

### 2. AudioFile Trait and Registry (`src/core/domain/traits.rs`)

**Current Coverage:** Basic tests only

**Public APIs to Test:**

#### AudioFileRegistry
- [x] `new()` - Create new registry
- [x] `register()` - Register a handler
- [x] `find_handler()` - Find handler for path
- [x] `supported_extensions()` - Get all supported extensions

**Missing Tests:**
1. Test register multiple handlers of same type (should this be allowed?)
2. Test find_handler returns first matching handler
3. Test supported_extensions deduplication logic
4. Test error handling when no handler found

**Implementation Steps:**
```rust
#[test]
fn test_registry_multiple_handlers_priority() {
    let mut registry = AudioFileRegistry::new();
    registry.register(Box::new(CustomHandler1));
    registry.register(Box::new(CustomHandler2));
    // Both can_handle .flac, verify which one is returned
}

#[test]
fn test_supported_extensions_deduplication() {
    let registry = create_audio_registry();
    let exts = registry.supported_extensions();
    let unique: HashSet<_> = exts.iter().collect();
    assert_eq!(exts.len(), unique.len());
}
```

---

### 3. Scanner Module (`src/core/services/scanner.rs`)

**Current Coverage:** Partial integration tests exist

**Public APIs to Test:**

#### Core Scan Functions
- [x] `scan_dir()` - Basic directory scanning
- [x] `scan_dir_paths()` - Scan for paths only
- [x] `scan_dir_immediate()` - Non-recursive scan
- [x] `scan_dir_with_metadata()` - Scan with full metadata
- [x] `scan_with_duplicates()` - Scan with duplicate detection
- [ ] `scan_dir_with_depth()` - Scan with depth limit
- [ ] `scan_dir_with_depth_and_symlinks()` - Scan with symlink handling
- [ ] `scan_dir_with_options()` - Full options scan
- [ ] `scan_tracks()` - Scan with formatted output

**Missing Tests (CRITICAL - 15 tests needed):**

1. **scan_dir_with_depth tests (4 tests):**
```rust
#[test]
fn test_scan_dir_with_depth_zero() {
    // Should only return files in immediate directory
}

#[test]
fn test_scan_dir_with_depth_one() {
    // Should return files at base + 1 level deep
}

#[test]
fn test_scan_dir_with_depth_unlimited() {
    // None should scan all levels
}

#[test]
fn test_scan_dir_with_depth_deep_nesting() {
    // Test with 5+ levels of nesting
}
```

2. **Symlink handling tests (3 tests):**
```rust
#[test]
fn test_scan_with_symlinks_follow() {
    // Create symlink to file, verify followed when follow_symlinks=true
}

#[test]
fn test_scan_with_symlinks_skip() {
    // Create symlink to file, verify skipped when follow_symlinks=false
}

#[test]
fn test_scan_with_broken_symlinks() {
    // Create broken symlink, verify graceful handling
}
```

3. **Pattern exclusion tests (4 tests):**
```rust
#[test]
fn test_scan_with_exclude_single_pattern() {
    // Exclude "*.tmp", verify .tmp files skipped
}

#[test]
fn test_scan_with_exclude_multiple_patterns() {
    // Exclude ["*.tmp", "temp_*", "*.bak"]
}

#[test]
fn test_scan_with_exclude_directory_pattern() {
    // Exclude "temp/*" or "backup/*"
}

#[test]
fn test_scan_with_invalid_pattern() {
    // Invalid glob pattern should be logged but not panic
}
```

4. **Edge case tests (4 tests):**
```rust
#[test]
fn test_scan_empty_directory() {
    // Should return empty Vec
}

#[test]
fn test_scan_nonexistent_directory() {
    // Should return empty Vec or error gracefully
}

#[test]
fn test_scan_with_unreadable_files() {
    // Files without read permissions
}

#[test]
fn test_scan_deterministic_ordering() {
    // Multiple scans should return same order
}
```

---

### 4. Inference Module (`src/core/services/inference.rs`)

**Current Coverage:** GOOD - Has inline unit tests

**Public APIs to Test:**
- [x] `infer_artist_from_path()` - Extract artist from path
- [x] `infer_album_from_path()` - Extract album from path
- [x] `infer_year_from_path()` - Extract year from path

**Status:** Coverage is adequate. Tests cover:
- Standard Artist/Album/track structure
- Nested directory structures
- Various separators (" - ", " – ", "—")
- Year suffixes
- Format suffixes [FLAC], [MP3]
- Unicode artist names
- Edge cases (same artist/album name)

**Minor Gaps:**
1. Test with Windows-style paths (backslashes)
2. Test with very long paths (>255 chars)
3. Test with special characters in filenames

---

### 5. Normalization Module (`src/core/services/normalization.rs`)

**Current Coverage:** GOOD - Has inline unit tests

**Public APIs to Test:**
- [x] `to_title_case()` - Convert to title case
- [x] `normalize_genre()` - Normalize genre names

**Internal Functions Missing Tests:**

1. **normalize_titles_internal()** - No direct tests
```rust
#[test]
fn test_normalize_titles_internal_single_file() {
    // Test with real audio file
}

#[test]
fn test_normalize_titles_internal_directory() {
    // Test with directory containing multiple files
}

#[test]
fn test_normalize_titles_internal_nonexistent_path() {
    // Should return error
}
```

2. **normalize_and_format()** - No tests
```rust
#[test]
fn test_normalize_and_format_json_output() {
    // Test JSON output format
}

#[test]
fn test_normalize_and_format_human_readable() {
    // Test text output format
}
```

3. **Report structs serialization (5 report types)**
```rust
#[test]
fn test_normalization_reports_serialization() {
    // Test TitleNormalizationReport, GenreNormalizationReport, etc.
    // Verify all fields serialize correctly
}
```

---

### 6. Validation Module (`src/core/services/validation.rs`)

**Current Coverage:** Partial

**Public APIs to Test:**
- [ ] `validate_path()` - Main validation entry point
- [ ] `validate_tracks()` - Track validation logic
- [ ] `build_validation_results()` - Human-readable formatting

**Missing Tests (8 tests needed):**

```rust
#[test]
fn test_validate_path_empty_directory() {
    // Should return appropriate message
}

#[test]
fn test_validate_path_json_output() {
    // Verify JSON structure
}

#[test]
fn test_validate_tracks_missing_title() {
    // Track without title should produce error
}

#[test]
fn test_validate_tracks_missing_artist() {
    // Track without artist should produce error
}

#[test]
fn test_validate_tracks_missing_album() {
    // Track without album should produce error
}

#[test]
fn test_validate_tracks_empty_fields() {
    // Fields with only whitespace should be errors
}

#[test]
fn test_validate_tracks_invalid_year() {
    // Year < 1900 or > 2100 should be warning
}

#[test]
fn test_validate_tracks_invalid_track_number() {
    // Track number 0 or > 99 should be warning
}
```

---

### 7. CUE Module (`src/core/services/cue.rs`)

**Current Coverage:** GOOD - Has inline unit tests

**Public APIs to Test:**
- [x] `generate_cue_content()` - Generate CUE content
- [x] `generate_cue_file_name()` - Generate filename
- [x] `write_cue_file()` - Write to disk
- [x] `parse_cue_file()` - Parse CUE file
- [x] `validate_cue_consistency()` - Validate against audio
- [x] `format_cue_validation_result()` - Format validation result
- [ ] `generate_cue_for_path()` - High-level generation

**Missing Tests:**

1. **generate_cue_for_path tests (3 tests):**
```rust
#[test]
fn test_generate_cue_for_path_success() {
    // Test successful generation from directory
}

#[test]
fn test_generate_cue_for_path_no_music_files() {
    // Empty directory should return NoMusicFiles error
}

#[test]
fn test_generate_cue_for_path_unreadable_files() {
    // Files that can't be read should return FileReadError
}
```

---

### 8. Library Module (`src/core/services/library.rs`)

**Current Coverage:** NO TESTS

**Public APIs to Test:**
- [ ] `build_library_hierarchy()` - Build hierarchy from tracks

**Missing Tests (5 tests needed):**

```rust
#[test]
fn test_build_library_hierarchy_empty() {
    let tracks = vec![];
    let library = build_library_hierarchy(tracks);
    assert_eq!(library.artists.len(), 0);
    assert_eq!(library.total_tracks, 0);
}

#[test]
fn test_build_library_hierarchy_single_artist() {
    // Single artist with one album and tracks
}

#[test]
fn test_build_library_hierarchy_multiple_artists() {
    // Multiple artists, verify correct grouping
}

#[test]
fn test_build_library_hierarchy_unknown_artist() {
    // Tracks without artist metadata should group under "Unknown Artist"
}

#[test]
fn test_build_library_hierarchy_unknown_album() {
    // Tracks without album metadata should group under "Unknown Album"
}

#[test]
fn test_build_library_hierarchy_preserves_metadata() {
    // Verify all metadata fields are preserved in TrackNode
}
```

---

### 9. Format Tree Module (`src/core/services/format_tree.rs`)

**Current Coverage:** NO TESTS

**Public APIs to Test:**
- [ ] `format_tree_output()` - Tree formatting
- [ ] `format_library_output()` - Library tree formatting
- [ ] `emit_structured_output()` - AI-friendly output
- [ ] `emit_by_path()` - Emit by path

**Missing Tests (10 tests needed):**

```rust
#[test]
fn test_format_tree_output_empty() {
    // Empty directory
}

#[test]
fn test_format_tree_output_flat_structure() {
    // All files in one directory
}

#[test]
fn test_format_tree_output_nested_structure() {
    // Deeply nested directories
}

#[test]
fn test_format_tree_output_with_metadata() {
    // Verify source icons are displayed
}

#[test]
fn test_format_library_output_basic() {
    // Basic library structure
}

#[test]
fn test_emit_structured_output_format() {
    // Verify output contains expected sections
}

#[test]
fn test_emit_by_path_json() {
    // JSON output structure
}

#[test]
fn test_emit_by_path_text() {
    // Text output structure
}

#[test]
fn test_emit_by_path_nonexistent() {
    // Error handling
}

#[test]
fn test_emit_by_path_empty() {
    // Empty directory handling
}
```

---

### 10. Apply Metadata Module (`src/core/services/apply_metadata.rs`)

**Current Coverage:** NO TESTS

**Public APIs to Test:**
- [ ] `write_metadata_by_path()` - Main entry point
- [ ] `apply_metadata_update()` - Apply single update

**Missing Tests (8 tests needed):**

```rust
#[test]
fn test_write_metadata_by_path_dry_run() {
    // Should not modify file
}

#[test]
fn test_write_metadata_by_path_apply() {
    // Should modify file
}

#[test]
fn test_write_metadata_by_path_apply_and_dry_run_conflict() {
    // Should return error
}

#[test]
fn test_write_metadata_by_path_nonexistent_file() {
    // Should return error
}

#[test]
fn test_write_metadata_by_path_unsupported_format() {
    // Should return error
}

#[test]
fn test_apply_metadata_update_all_fields() {
    // Test each field: title, artist, album, album_artist, track_number, disc_number, year, genre
}

#[test]
fn test_apply_metadata_update_invalid_track_number() {
    // Non-numeric should return error
}

#[test]
fn test_apply_metadata_update_invalid_year() {
    // Non-numeric should return error
}

#[test]
fn test_apply_metadata_update_unsupported_field() {
    // Unknown field should return error
}
```

---

### 11. Audio Format Handlers

#### FLAC Handler (`src/adapters/audio_formats/flac.rs`)

**Current Coverage:** Basic tests exist

**Status:** Has tests for:
- Handler creation
- can_handle method
- Unsupported format errors
- Nonexistent file errors

**Missing Tests:**

1. **Integration tests with real FLAC files (8 tests):**
```rust
#[test]
fn test_flac_read_metadata_all_fields() {
    // Use fixture file, verify all metadata fields read correctly
}

#[test]
fn test_flac_write_metadata_all_fields() {
    // Copy fixture, write metadata, read back and verify
}

#[test]
fn test_flac_write_metadata_partial_fields() {
    // Only update some fields, others should remain unchanged
}

#[test]
fn test_flac_read_basic_info() {
    // Verify duration and format extracted
}

#[test]
fn test_flac_folder_inference_fallback() {
    // FLAC with no embedded metadata should use folder inference
}

#[test]
fn test_flac_metadata_confidence() {
    // Embedded should have confidence 1.0, inferred 0.3
}
```

#### MP3 Handler (`src/adapters/audio_formats/mp3.rs`)

**Current Coverage:** Unknown - check file

**Needed Tests (6 tests):**
- Handler creation
- can_handle for .mp3 and .MP3
- Read metadata
- Write metadata
- Folder inference fallback
- Error handling for corrupted files

#### WAV Handler (`src/adapters/audio_formats/wav.rs`)

**Current Coverage:** Unknown - check file

**Needed Tests (6 tests):**
- Handler creation
- can_handle for .wav and .WAV
- Read metadata
- Write metadata
- Duration extraction
- Error handling

#### DSF Handler (`src/adapters/audio_formats/dsf.rs`)

**Current Coverage:** Basic tests exist

**Status:** Similar structure to FLAC handler

**Needed Tests (6 tests):**
- Handler creation and can_handle
- Read metadata from real DSF file
- Error handling for non-DSF files
- Error handling for corrupted files

#### WavPack Handler (`src/adapters/audio_formats/wavpack.rs`)

**Current Coverage:** Unknown - check file

**Needed Tests (6 tests):**
- Handler creation
- can_handle for .wv extension
- Read metadata
- Error handling

---

### 12. Error Module (`src/core/errors.rs`)

**Current Coverage:** NO TESTS

**Public APIs to Test:**
- [ ] All Error variants
- [ ] Display trait implementations
- [ ] From trait implementations

**Missing Tests (10 tests needed):**

```rust
#[test]
fn test_error_display_messages() {
    // Verify each error variant produces correct message
}

#[test]
fn test_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
    let err: MusicChoreError = io_err.into();
    match err {
        MusicChoreError::IoError(_) => (),
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_from_json_error() {
    // Test serde_json::Error conversion
}

#[test]
fn test_error_from_utf8_error() {
    // Test Utf8Error conversion
}

#[test]
fn test_error_from_parse_int_error() {
    // Test ParseIntError conversion
}

#[test]
fn test_error_from_parse_float_error() {
    // Test ParseFloatError conversion
}
```

---

### 13. MCP Server (`src/mcp/music_chore_server.rs`)

**Current Coverage:** NO TESTS

**Public APIs to Test:**
- [ ] `scan_directory()` - MCP tool
- [ ] `get_library_tree()` - MCP tool
- [ ] `read_file_metadata()` - MCP tool
- [ ] `normalize()` - MCP tool
- [ ] `emit_library_metadata()` - MCP tool
- [ ] `validate_library()` - MCP tool
- [ ] `find_duplicates()` - MCP tool
- [ ] `cue_file()` - MCP tool

**Note:** MCP server tests require async test framework (tokio::test)

**Missing Tests (16 tests needed):**

```rust
#[tokio::test]
async fn test_mcp_scan_directory_success() {
    // Test successful scan
}

#[tokio::test]
async fn test_mcp_scan_directory_empty() {
    // Test empty directory handling
}

#[tokio::test]
async fn test_mcp_scan_directory_json_output() {
    // Test JSON output format
}

#[tokio::test]
async fn test_mcp_get_library_tree() {
    // Test tree generation
}

#[tokio::test]
async fn test_mcp_read_file_metadata_success() {
    // Test reading valid file
}

#[tokio::test]
async fn test_mcp_read_file_metadata_not_found() {
    // Test nonexistent file
}

#[tokio::test]
async fn test_mcp_normalize() {
    // Test normalization tool
}

#[tokio::test]
async fn test_mcp_emit_library_metadata() {
    // Test emit tool
}

#[tokio::test]
async fn test_mcp_validate_library() {
    // Test validation tool
}

#[tokio::test]
async fn test_mcp_find_duplicates() {
    // Test duplicate detection tool
}

#[tokio::test]
async fn test_mcp_cue_file_generate() {
    // Test CUE generation
}

#[tokio::test]
async fn test_mcp_cue_file_parse() {
    // Test CUE parsing
}

#[tokio::test]
async fn test_mcp_cue_file_validate() {
    // Test CUE validation
}

#[tokio::test]
async fn test_mcp_cue_file_invalid_operation() {
    // Test invalid operation handling
}
```

---

## Test Implementation Priority

### Phase 1: Critical Missing Tests (Week 1)
1. **Library module** - No tests at all
2. **Apply metadata module** - No tests at all  
3. **Format tree module** - No tests at all
4. **Scanner depth/pattern options** - Core functionality

### Phase 2: Important Coverage (Week 2)
1. **Validation module** - Only partial coverage
2. **Error module** - No tests for error types
3. **Real audio file integration tests** - FLAC, MP3, WAV
4. **MCP server tests** - Async test framework setup

### Phase 3: Polish and Edge Cases (Week 3)
1. **Edge cases** - Empty files, corrupted files, permissions
2. **Performance tests** - Large directories, many files
3. **Cross-platform tests** - Windows paths, symlinks
4. **Documentation** - Update AGENTS.md with test guidelines

---

## Test Organization Recommendations

### Current Structure:
```
tests/
├── scanner_tests.rs           # Good
├── scanner_extended_tests.rs  # Good
├── flac_metadata_tests.rs     # Good
├── mp3_integration_tests.rs   # Good
├── validation_tests.rs        # Partial
├── checksum_tests.rs          # Good
├── inference_year_tests.rs    # Good
└── ... (many more)
```

### Recommended Additions:
```
tests/
├── library_tests.rs              # NEW - Test library module
├── apply_metadata_tests.rs       # NEW - Test metadata writing
├── format_tree_tests.rs          # Expand existing
├── scanner_depth_tests.rs        # NEW - Depth limit tests
├── scanner_symlink_tests.rs      # NEW - Symlink handling
├── scanner_pattern_tests.rs      # NEW - Pattern exclusion
├── error_tests.rs                # NEW - Error type tests
├── mcp_server_tests.rs           # NEW - MCP tool tests
└── integration/
    └── real_audio_files_tests.rs # NEW - Full integration tests
```

---

## Testing Guidelines

### 1. Fixture Management
- Use `tests/fixtures/` directory for real audio files
- Create temporary files with `tempfile::TempDir` for write tests
- Never modify fixture files directly

### 2. Test Naming Convention
```rust
// Format: test_<module>_<function>_<scenario>
#[test]
fn test_scanner_scan_dir_with_depth_zero() { }

#[test]
fn test_flac_handler_write_metadata_all_fields() { }

#[test]
fn test_library_build_hierarchy_empty() { }
```

### 3. Test Structure Template
```rust
#[test]
fn test_feature_description() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let test_data = create_test_data();
    
    // Act
    let result = function_under_test(&test_data);
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_value);
}
```

### 4. Async Test Pattern
```rust
#[tokio::test]
async fn test_async_feature() {
    let server = MusicChoreServer::new();
    let params = Parameters(ScanDirectoryParams { ... });
    
    let result = server.scan_directory(params).await;
    
    assert!(result.is_ok());
}
```

---

## Coverage Measurement

### Run Coverage Report:
```bash
# Install cargo-tarpaulin if not already installed
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# View coverage report
open tarpaulin-report.html
```

### Coverage Targets by Module:
| Module | Current | Target |
|--------|---------|--------|
| models.rs | 80% | 95% |
| traits.rs | 70% | 90% |
| scanner.rs | 60% | 90% |
| inference.rs | 90% | 95% |
| normalization.rs | 85% | 95% |
| validation.rs | 50% | 90% |
| cue.rs | 85% | 95% |
| library.rs | 0% | 90% |
| format_tree.rs | 0% | 90% |
| apply_metadata.rs | 0% | 90% |
| errors.rs | 0% | 80% |
| flac.rs | 70% | 90% |
| mp3.rs | ? | 90% |
| wav.rs | ? | 90% |
| mcp/*.rs | 0% | 80% |

---

## Conclusion

This testing plan identifies **approximately 80+ missing tests** across the codebase. Priority should be given to:

1. Modules with zero coverage (library, apply_metadata, format_tree)
2. Critical user-facing functionality (scanner options, validation)
3. Error handling and edge cases
4. MCP server async functions

Following this plan will bring the codebase to ~90% line coverage and ensure all public APIs are properly tested.

---

**Next Steps:**
1. Review this plan with the team
2. Set up async test framework for MCP tests
3. Begin Phase 1 implementation (Critical Missing Tests)
4. Run coverage reports weekly to track progress
