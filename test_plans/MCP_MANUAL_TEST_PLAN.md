# 🤖 musicctl-mcp - Manual Testing Plan (MCP Interface)

## 📋 Overview
This document provides comprehensive manual testing scenarios for the musicctl-mcp MCP server interface. Each test scenario includes detailed steps and expected results to ensure full functionality coverage of all MCP tools.

## 🎯 Test Scenarios

### **1. MCP Server Initialization**

**Test ID:** TC-MCP-INIT-001  
**Objective:** Verify MCP server starts and initializes correctly  
**Steps:**
1. Start MCP server: `cargo run --bin musicctl-mcp`
2. Connect MCP client to the server
3. Verify initialization handshake completes successfully
4. Check that all tools are advertised in capabilities

**Expected Results:**
- Server starts without errors
- Initialization handshake completes with protocol version exchange
- Server advertises all 9 tools (scan_directory, get_library_tree, read_file_metadata, write_file_metadata, normalize_titles, emit_library_metadata, validate_library, find_duplicates, cue_file)
- Client receives proper server information

---

### **2. scan_directory Tool Functionality**

**Test ID:** TC-SCAN-002  
**Objective:** Verify directory scanning via MCP  
**Steps:**
1. Start MCP server if not already running
2. Send tool call: `{"method": "tools/call", "params": {"name": "scan_directory", "arguments": {"path": "tests/fixtures/flac/simple", "json_output": false}}}`
3. Send tool call with JSON output: `{"method": "tools/call", "params": {"name": "scan_directory", "arguments": {"path": "tests/fixtures/flac/simple", "json_output": true}}}`
4. Test with invalid path: `{"method": "tools/call", "params": {"name": "scan_directory", "arguments": {"path": "/nonexistent/path", "json_output": false}}}`

**Expected Results:**
- Plain output returns array of file paths as strings
- JSON output returns structured array with schema version wrapper
- Invalid path returns appropriate error message
- All 5 supported formats are detected when present
- Follows symbolic links setting respected if implemented

---

### **3. get_library_tree Tool Functionality**

**Test ID:** TC-TREE-003  
**Objective:** Verify library tree generation via MCP  
**Steps:**
1. Send tool call: `{"method": "tools/call", "params": {"name": "get_library_tree", "arguments": {"path": "tests/fixtures/flac/nested", "json_output": false}}}`
2. Send tool call with JSON: `{"method": "tools/call", "params": {"name": "get_library_tree", "arguments": {"path": "tests/fixtures/flac/nested", "json_output": true}}}`
3. Test with empty directory: `{"method": "tools/call", "params": {"name": "get_library_tree", "arguments": {"path": "tests/fixtures/empty", "json_output": false}}}`

**Expected Results:**
- Plain output returns hierarchical tree representation
- JSON output returns structured library object with schema version
- Proper artist → album → track hierarchy is maintained
- Empty directory returns appropriate empty structure

---

### **4. read_file_metadata Tool Functionality**

**Test ID:** TC-READ-004  
**Objective:** Verify file metadata reading via MCP  
**Steps:**
1. Test with FLAC file: `{"method": "tools/call", "params": {"name": "read_file_metadata", "arguments": {"file_path": "tests/fixtures/flac/simple/track1.flac"}}}`
2. Test with MP3 file: `{"method": "tools/call", "params": {"name": "read_file_metadata", "arguments": {"file_path": "tests/fixtures/mp3/simple/track1.mp3"}}}`
3. Test with WAV file: `{"method": "tools/call", "params": {"name": "read_file_metadata", "arguments": {"file_path": "tests/fixtures/wav/simple/track1.wav"}}}`
4. Test with DSF file: `{"method": "tools/call", "params": {"name": "read_file_metadata", "arguments": {"file_path": "tests/fixtures/dsf/simple/track1.dsf"}}}`
5. Test with WavPack file: `{"method": "tools/call", "params": {"name": "read_file_metadata", "arguments": {"file_path": "tests/fixtures/wavpack/simple/track1.wv"}}}`
6. Test with nonexistent file: `{"method": "tools/call", "params": {"name": "read_file_metadata", "arguments": {"file_path": "/nonexistent/file.flac"}}}`

**Expected Results:**
- Returns complete metadata for all 5 supported formats
- Metadata includes source and confidence information
- Nonexistent file returns appropriate error
- JSON output includes schema version wrapper
- Embedded and inferred metadata properly differentiated

---

### **5. write_file_metadata Tool Functionality**

**Test ID:** TC-WRITE-005  
**Objective:** Verify file metadata writing via MCP  
**Steps:**
1. Prepare test file with original metadata
2. Send dry-run write: `{"method": "tools/call", "params": {"name": "write_file_metadata", "arguments": {"file_path": "test.flac", "set": ["title=New Title", "artist=New Artist"], "dry_run": true, "apply": false}}}`
3. Send actual write: `{"method": "tools/call", "params": {"name": "write_file_metadata", "arguments": {"file_path": "test.flac", "set": ["title=New Title", "artist=New Artist"], "dry_run": false, "apply": true}}}`
4. Verify changes with read_file_metadata tool
5. Test with invalid field: `{"method": "tools/call", "params": {"name": "write_file_metadata", "arguments": {"file_path": "test.flac", "set": ["invalid_field=Test"], "dry_run": true, "apply": false}}}`

**Expected Results:**
- Dry-run returns what would be changed without modifying file
- Apply operation modifies actual file metadata
- Verification shows updated values
- Invalid fields return appropriate error
- All 5 formats support write operations
- JSON output includes schema version wrapper

---

### **6. normalize_titles Tool Functionality**

**Test ID:** TC-NORM-006  
**Objective:** Verify title normalization via MCP  
**Steps:**
1. Create directory with inconsistently titled files
2. Send dry-run normalization: `{"method": "tools/call", "params": {"name": "normalize_titles", "arguments": {"path": "test_dir", "dry_run": true}}}`
3. Send actual normalization: `{"method": "tools/call", "params": {"name": "normalize_titles", "arguments": {"path": "test_dir", "dry_run": false}}}`
4. Test with genre normalization: `{"method": "tools/call", "params": {"name": "normalize_genres", "arguments": {"path": "test_dir", "dry_run": true}}}`
5. Verify changes by reading metadata after normalization

**Expected Results:**
- Dry-run returns list of changes that would be made
- Actual normalization modifies file metadata
- Titles converted to proper title case
- Genres normalized to standard categories
- JSON output includes schema version wrapper
- No changes made to already properly formatted metadata

---

### **7. emit_library_metadata Tool Functionality**

**Test ID:** TC-EMIT-007  
**Objective:** Verify library metadata export via MCP  
**Steps:**
1. Send tool call: `{"method": "tools/call", "params": {"name": "emit_library_metadata", "arguments": {"path": "tests/fixtures/flac/nested", "json_output": false}}}`
2. Send tool call with JSON: `{"method": "tools/call", "params": {"name": "emit_library_metadata", "arguments": {"path": "tests/fixtures/flac/nested", "json_output": true}}}`
3. Test with empty directory: `{"method": "tools/call", "params": {"name": "emit_library_metadata", "arguments": {"path": "tests/fixtures/empty", "json_output": false}}}`

**Expected Results:**
- Plain output returns human-readable library structure
- JSON output returns structured library data with schema version
- Complete artist → album → track hierarchy is exported
- All metadata fields are included with source/confidence information
- Empty directory returns appropriate empty structure

---

### **8. validate_library Tool Functionality**

**Test ID:** TC-VALIDATE-008  
**Objective:** Verify library validation via MCP  
**Steps:**
1. Send validation call: `{"method": "tools/call", "params": {"name": "validate_library", "arguments": {"path": "tests/fixtures/flac/simple", "json_output": false}}}`
2. Send validation with JSON: `{"method": "tools/call", "params": {"name": "validate_library", "arguments": {"path": "tests/fixtures/flac/simple", "json_output": true}}}`
3. Test with directory containing validation issues: `{"method": "tools/call", "params": {"name": "validate_library", "arguments": {"path": "tests/fixtures/invalid_metadata", "json_output": false}}}`

**Expected Results:**
- Plain output shows validation summary and detailed issues
- JSON output returns structured validation results with schema version
- Files with missing required fields are flagged as errors
- Files with missing recommended fields are flagged as warnings
- Valid files are counted separately
- Summary statistics are accurate

---

### **9. find_duplicates Tool Functionality**

**Test ID:** TC-DUPL-009  
**Objective:** Verify duplicate detection via MCP  
**Steps:**
1. Create directory with duplicate files
2. Send duplicate detection: `{"method": "tools/call", "params": {"name": "find_duplicates", "arguments": {"path": "test_dir_with_dups", "json_output": false}}}`
3. Send duplicate detection with JSON: `{"method": "tools/call", "params": {"name": "find_duplicates", "arguments": {"path": "test_dir_with_dups", "json_output": true}}}`
4. Test with directory containing no duplicates: `{"method": "tools/call", "params": {"name": "find_duplicates", "arguments": {"path": "test_dir_unique", "json_output": false}}}`

**Expected Results:**
- Plain output shows duplicate groups with file paths
- JSON output returns structured duplicate information with checksums
- Files with identical content are grouped as duplicates
- Files with different content are not flagged as duplicates
- No duplicates case returns appropriate message

---

### **10. cue_file Tool Functionality**

**Test ID:** TC-CUE-010  
**Objective:** Verify CUE file operations via MCP  
**Steps:**
1. **Generate CUE:** `{"method": "tools/call", "params": {"name": "cue_file", "arguments": {"path": "test_album_dir", "operation": "generate", "dry_run": true, "force": false}}}`
2. **Parse CUE:** `{"method": "tools/call", "params": {"name": "cue_file", "arguments": {"path": "test_album.cue", "operation": "parse", "json_output": false}}}`
3. **Validate CUE:** `{"method": "tools/call", "params": {"name": "cue_file", "arguments": {"path": "test_album.cue", "operation": "validate", "audio_dir": "test_audio_dir", "json_output": true}}}`
4. Test invalid operations and paths

**Expected Results:**
- Generate operation creates CUE content (dry-run) or file (apply)
- Parse operation returns CUE file structure in plain or JSON format
- Validate operation returns validation results with schema version
- Invalid operations return appropriate error messages
- All operations respect JSON/plain output preference

---

### **11. Error Handling Scenarios**

**Test ID:** TC-ERROR-011  
**Objective:** Verify proper error handling in MCP  
**Steps:**
1. Call tool with missing required arguments
2. Call tool with invalid argument types
3. Call tool with nonexistent file paths
4. Call tool with unsupported file formats
5. Send malformed JSON requests

**Expected Results:**
- Appropriate error messages returned for each scenario
- Error responses follow MCP error format
- Server continues running after errors
- No crashes or panics occur

---

### **12. Concurrent Requests Handling**

**Test ID:** TC-CONCUR-012  
**Objective:** Verify server handles concurrent requests  
**Steps:**
1. Start MCP server
2. Send multiple simultaneous requests for different tools
3. Send multiple requests to same tool with different paths
4. Monitor server stability during concurrent operations

**Expected Results:**
- All requests are processed correctly
- No race conditions occur
- Server remains stable under concurrent load
- Responses are correctly matched to requests

---

### **13. Schema Version Consistency**

**Test ID:** TC-SCHEMA-013  
**Objective:** Verify all JSON responses include schema version  
**Steps:**
1. Make various tool calls requesting JSON output
2. Verify each response includes schema version wrapper
3. Check that schema version is consistent across all tools
4. Verify deserialization works for all response types

**Expected Results:**
- All JSON responses include schema version wrapper
- Schema version is consistent (e.g., "musicctl-v0.3.0")
- Deserialization succeeds for all response types
- Backward compatibility is maintained

---

### **14. Performance Under Load**

**Test ID:** TC-PERF-014  
**Objective:** Verify performance with large libraries  
**Steps:**
1. Create large test library (1000+ files)
2. Run scan_directory on large library
3. Run validate_library on large library
4. Measure response times

**Expected Results:**
- Operations complete within reasonable time (under 30 seconds for 1000 files)
- Memory usage remains reasonable
- No timeouts occur
- Progress information provided for long operations (where applicable)

---

## 📊 Test Coverage Summary
- [ ] MCP server initialization and capabilities
- [ ] scan_directory tool with all parameters
- [ ] get_library_tree tool with all parameters
- [ ] read_file_metadata for all 5 formats
- [ ] write_file_metadata with dry-run and apply modes
- [ ] normalize_titles and normalize_genres tools
- [ ] emit_library_metadata tool with JSON/plain output
- [ ] validate_library tool with JSON/plain output
- [ ] find_duplicates tool with JSON/plain output
- [ ] cue_file tool with generate/parse/validate operations
- [ ] Error handling for all tools
- [ ] Concurrent request handling
- [ ] Schema version consistency
- [ ] Performance under load