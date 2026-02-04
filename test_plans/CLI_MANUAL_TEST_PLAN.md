# 🎵 musicctl - Manual Testing Plan (CLI Interface)

## 📋 Overview
This document provides comprehensive manual testing scenarios for the musicctl CLI interface. Each test scenario includes detailed steps and expected results to ensure full functionality coverage.

## 🎯 Test Scenarios

### **1. Scan Command Functionality**

**Test ID:** TC-SCAN-001  
**Objective:** Verify directory scanning functionality  
**Steps:**
1. Navigate to project directory: `cd /Users/komdosh/Projects/music-chore`
2. Create test directory with mixed audio files: `mkdir -p test_scan/{flac,mp3,wav,dsf,wavpack}`
3. Copy sample files to each subdirectory
4. Run: `cargo run --bin musicctl -- scan test_scan`
5. Run: `cargo run --bin musicctl -- scan test_scan --json`
6. Run: `cargo run --bin musicctl -- scan test_scan --verbose`

**Expected Results:**
- Plain output shows file paths one per line
- JSON output shows structured array of file paths
- Verbose output shows progress and summary statistics
- All 5 supported formats (flac, mp3, wav, dsf, wv) are detected

---

### **2. Tree Command Functionality**

**Test ID:** TC-TREE-002  
**Objective:** Verify hierarchical tree view functionality  
**Steps:**
1. Create organized music directory: `mkdir -p test_tree/Artist/Album/`
2. Copy sample files to album directory
3. Run: `cargo run --bin musicctl -- tree test_tree`
4. Run: `cargo run --bin musicctl -- tree test_tree --json`

**Expected Results:**
- Plain output shows artist → album → track hierarchy with emojis
- JSON output shows structured library data with schema version
- Proper inference of artist/album from directory structure

---

### **3. Read Command Functionality**

**Test ID:** TC-READ-003  
**Objective:** Verify metadata reading from audio files  
**Steps:**
1. Prepare test files in different formats (FLAC, MP3, WAV, DSF, WavPack)
2. Run: `cargo run --bin musicctl -- read tests/fixtures/flac/simple/track1.flac`
3. Run: `cargo run --bin musicctl -- read tests/fixtures/mp3/simple/track1.mp3`
4. Run: `cargo run --bin musicctl -- read tests/fixtures/wav/simple/track1.wav`
5. Run: `cargo run --bin musicctl -- read tests/fixtures/dsf/simple/track1.dsf`
6. Run: `cargo run --bin musicctl -- read tests/fixtures/wavpack/simple/track1.wv`

**Expected Results:**
- All 5 formats return metadata in JSON format with schema version
- Embedded metadata is extracted correctly
- Inferred metadata from folder structure is included
- Confidence values are properly assigned (1.0 for embedded, 0.3 for folder-inferred)

---

### **4. Write Command Functionality**

**Test ID:** TC-WRITE-004  
**Objective:** Verify metadata writing to audio files  
**Steps:**
1. Copy test file to temporary location
2. Run dry-run: `cargo run --bin musicctl -- write temp.flac --set title="Test Title" --set artist="Test Artist" --dry-run`
3. Run actual write: `cargo run --bin musicctl -- write temp.flac --set title="Test Title" --set artist="Test Artist" --apply`
4. Verify changes: `cargo run --bin musicctl -- read temp.flac`

**Expected Results:**
- Dry-run shows what would be changed without modifying files
- Apply flag actually modifies the file metadata
- Verification shows updated metadata values
- All 5 supported formats allow writing

---

### **5. Normalize Command Functionality**

**Test ID:** TC-NORMALIZE-005  
**Objective:** Verify title and genre normalization  
**Steps:**
1. Create directory with files having inconsistent titles
2. Run: `cargo run --bin musicctl -- normalize test_dir --dry-run`
3. Run: `cargo run --bin musicctl -- normalize test_dir --apply`
4. Test genre normalization: `cargo run --bin musicctl -- normalize test_dir --genres --dry-run`
5. Test genre normalization: `cargo run --bin musicctl -- normalize test_dir --genres --apply`

**Expected Results:**
- Dry-run shows what changes would be made
- Apply flag actually modifies file metadata
- Titles converted to proper title case
- Genres normalized to standard categories
- No changes made to already properly formatted metadata

---

### **6. Emit Command Functionality**

**Test ID:** TC-EMIT-006  
**Objective:** Verify library metadata export functionality  
**Steps:**
1. Create test library with multiple artists/albums
2. Run: `cargo run --bin musicctl -- emit test_lib`
3. Run: `cargo run --bin musicctl -- emit test_lib --json`

**Expected Results:**
- Plain output shows human-readable library structure
- JSON output shows structured library data with schema version
- All tracks, albums, and artists are properly represented
- Metadata values include source and confidence information

---

### **7. Validate Command Functionality**

**Test ID:** TC-VALIDATE-007  
**Objective:** Verify metadata validation functionality  
**Steps:**
1. Create directory with valid and invalid metadata files
2. Run: `cargo run --bin musicctl -- validate test_dir`
3. Run: `cargo run --bin musicctl -- validate test_dir --json`

**Expected Results:**
- Plain output shows validation summary and detailed errors/warnings
- JSON output shows structured validation results with schema version
- Files with missing required fields are flagged as errors
- Files with missing recommended fields are flagged as warnings
- Valid files are counted separately

---

### **8. Duplicates Command Functionality**

**Test ID:** TC-DUPLICATES-008  
**Objective:** Verify duplicate detection functionality  
**Steps:**
1. Create directory with duplicate files (same content, different names)
2. Run: `cargo run --bin musicctl -- duplicates test_dir`
3. Run: `cargo run --bin musicctl -- duplicates test_dir --json`

**Expected Results:**
- Plain output shows duplicate groups with file paths
- JSON output shows structured duplicate information with checksums
- Files with identical content are grouped as duplicates
- Files with different content are not flagged as duplicates

---

### **9. CUE Command Functionality**

**Test ID:** TC-CUE-009  
**Objective:** Verify CUE file operations  
**Steps:**
1. Create album directory with multiple tracks
2. Generate CUE: `cargo run --bin musicctl -- cue --generate test_album`
3. Parse CUE: `cargo run --bin musicctl -- cue --parse test_album.cue`
4. Validate CUE: `cargo run --bin musicctl -- cue --validate test_album.cue`

**Expected Results:**
- Generate creates valid CUE file with track information
- Parse shows CUE file contents in human-readable format
- Validate confirms CUE file consistency with audio files
- All operations work with --json flag for structured output

---

### **10. Help and Version Commands**

**Test ID:** TC-HELP-010  
**Objective:** Verify help and version functionality  
**Steps:**
1. Run: `cargo run --bin musicctl -- --help`
2. Run: `cargo run --bin musicctl -- --version`
3. Run: `cargo run --bin musicctl -- scan --help`

**Expected Results:**
- Help shows comprehensive usage information
- Version shows current version number
- Command-specific help shows options and arguments for that command

---

### **11. Error Handling Scenarios**

**Test ID:** TC-ERROR-011  
**Objective:** Verify proper error handling  
**Steps:**
1. Run scan on nonexistent directory: `cargo run --bin musicctl -- scan /nonexistent/path`
2. Run read on unsupported file: `cargo run --bin musicctl -- read unsupported.xyz`
3. Run write with invalid field: `cargo run --bin musicctl -- write test.flac --set invalid_field=value --dry-run`

**Expected Results:**
- Appropriate error messages are displayed
- Exit codes are non-zero for error conditions
- No crash or panic occurs

---

### **12. Edge Cases and Special Characters**

**Test ID:** TC-EDGE-012  
**Objective:** Verify handling of special cases  
**Steps:**
1. Create directory with Unicode characters in names
2. Test with files containing special characters in metadata
3. Test with deeply nested directory structures
4. Test with files containing no metadata

**Expected Results:**
- Unicode characters are handled properly
- Special characters don't cause crashes
- Nested structures are processed correctly
- Files with no metadata are handled gracefully

---

## 📊 Test Coverage Summary
- [ ] Scan command with all options
- [ ] Tree command with all options  
- [ ] Read command for all 5 formats
- [ ] Write command with dry-run and apply
- [ ] Normalize command for titles and genres
- [ ] Emit command with JSON and plain output
- [ ] Validate command with JSON and plain output
- [ ] Duplicates command with JSON and plain output
- [ ] CUE command with generate/parse/validate
- [ ] Help and version commands
- [ ] Error handling scenarios
- [ ] Edge cases with special characters