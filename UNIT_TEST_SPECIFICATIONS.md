# Unit Test Specifications - Music Chore

**Version:** 2.0  
**Format:** Method-by-method specifications with concrete test cases

---

## How to Read This Document

Each test specification includes:
- **Method:** The function being tested
- **Test ID:** Unique identifier
- **Description:** What the test verifies
- **Given:** Input conditions/setup
- **When:** Action being performed
- **Then:** Expected outcomes and assertions
- **Priority:** Critical/High/Medium/Low

---

## Module: core::domain::models

### Struct: MetadataValue<T>

#### Test: metadata_value_creation_embedded
- **Method:** `MetadataValue::embedded()`
- **Test ID:** MV001
- **Priority:** High
- **Description:** Verify embedded metadata value creation sets correct defaults
- **Given:** A string value "Test Title"
- **When:** Creating `MetadataValue::embedded("Test Title".to_string())`
- **Then:**
  - `value` == "Test Title"
  - `source` == `MetadataSource::Embedded`
  - `confidence` == 1.0

#### Test: metadata_value_creation_inferred
- **Method:** `MetadataValue::inferred()`
- **Test ID:** MV002
- **Priority:** High
- **Description:** Verify inferred metadata value creation with custom confidence
- **Given:** Value "Artist Name" and confidence 0.3
- **When:** Creating `MetadataValue::inferred("Artist Name".to_string(), 0.3)`
- **Then:**
  - `value` == "Artist Name"
  - `source` == `MetadataSource::FolderInferred`
  - `confidence` == 0.3

#### Test: metadata_value_creation_user_set
- **Method:** `MetadataValue::user_set()`
- **Test ID:** MV003
- **Priority:** High
- **Description:** Verify user-edited metadata value creation
- **Given:** A string value "User Album"
- **When:** Creating `MetadataValue::user_set("User Album".to_string())`
- **Then:**
  - `value` == "User Album"
  - `source` == `MetadataSource::UserEdited`
  - `confidence` == 1.0

#### Test: metadata_value_creation_cue_inferred
- **Method:** `MetadataValue::cue_inferred()`
- **Test ID:** MV004
- **Priority:** High
- **Description:** Verify CUE-inferred metadata value creation
- **Given:** Value "Track Title" and confidence 1.0
- **When:** Creating `MetadataValue::cue_inferred("Track Title".to_string(), 1.0)`
- **Then:**
  - `value` == "Track Title"
  - `source` == `MetadataSource::CueInferred`
  - `confidence` == 1.0

#### Test: metadata_value_display_string
- **Method:** `MetadataValue<T> as Display`
- **Test ID:** MV005
- **Priority:** Medium
- **Description:** Verify Display trait formats String values correctly
- **Given:** `MetadataValue::embedded("Hello World".to_string())`
- **When:** Calling `format!("{}", metadata_value)`
- **Then:** Result == "Hello World"

#### Test: metadata_value_display_u32
- **Method:** `MetadataValue<T> as Display`
- **Test ID:** MV006
- **Priority:** Medium
- **Description:** Verify Display trait formats u32 values correctly
- **Given:** `MetadataValue::embedded(2024u32)`
- **When:** Calling `format!("{}", metadata_value)`
- **Then:** Result == "2024"

#### Test: metadata_value_display_empty_string
- **Method:** `MetadataValue<T> as Display`
- **Test ID:** MV007
- **Priority:** Low
- **Description:** Verify Display handles empty string
- **Given:** `MetadataValue::embedded("".to_string())`
- **When:** Calling `format!("{}", metadata_value)`
- **Then:** Result == ""

---

### Struct: Track

#### Test: track_new_without_checksum
- **Method:** `Track::new()`
- **Test ID:** TR001
- **Priority:** High
- **Description:** Verify Track creation without checksum
- **Given:** Path "/music/artist/album/track.flac" and TrackMetadata with title="Song"
- **When:** Calling `Track::new(path, metadata)`
- **Then:**
  - `file_path` == path
  - `metadata.title.value` == "Song"
  - `checksum` == None

#### Test: track_with_checksum
- **Method:** `Track::with_checksum()`
- **Test ID:** TR002
- **Priority:** High
- **Description:** Verify Track creation with checksum
- **Given:** Path, metadata, and checksum "abc123..." (64 hex chars)
- **When:** Calling `Track::with_checksum(path, metadata, checksum.clone())`
- **Then:**
  - `file_path` == path
  - `checksum` == Some(checksum)
  - `checksum.as_ref().unwrap().len()` == 64

#### Test: track_calculate_checksum_valid_file
- **Method:** `Track::calculate_checksum()`
- **Test ID:** TR003
- **Priority:** Critical
- **Description:** Verify SHA256 checksum calculation for existing file
- **Given:** 
  - Copy `tests/fixtures/flac/simple/track1.flac` to temp directory
  - Create Track pointing to the file
- **When:** Calling `track.calculate_checksum()`
- **Then:**
  - Result.is_ok() == true
  - Result.unwrap().len() == 64 (valid SHA256 hex string)
  - All characters are hex digits (0-9, a-f)

#### Test: track_calculate_checksum_deterministic
- **Method:** `Track::calculate_checksum()`
- **Test ID:** TR004
- **Priority:** High
- **Description:** Verify same file produces same checksum
- **Given:** Same file from TR003
- **When:** Calling `calculate_checksum()` twice
- **Then:**
  - First checksum == Second checksum
  - Both are valid 64-char hex strings

#### Test: track_calculate_checksum_different_files
- **Method:** `Track::calculate_checksum()`
- **Test ID:** TR005
- **Priority:** High
- **Description:** Verify different files produce different checksums
- **Given:** 
  - Copy `track1.flac` and `track2.flac` to temp directory
  - Create two Tracks
- **When:** Calling `calculate_checksum()` on both
- **Then:**
  - checksum1 != checksum2
  - Both are valid 64-char hex strings

#### Test: track_calculate_checksum_modified_file
- **Method:** `Track::calculate_checksum()`
- **Test ID:** TR006
- **Priority:** High
- **Description:** Verify modified file produces different checksum
- **Given:**
  - Copy `track1.flac` to temp file
  - Calculate initial checksum
  - Modify file metadata using write_metadata
- **When:** Calculate checksum again
- **Then:**
  - New checksum != Initial checksum

#### Test: track_calculate_checksum_nonexistent_file
- **Method:** `Track::calculate_checksum()`
- **Test ID:** TR007
- **Priority:** High
- **Description:** Verify error handling for nonexistent file
- **Given:** Track with path `/nonexistent/file.flac`
- **When:** Calling `calculate_checksum()`
- **Then:**
  - Result.is_err() == true
  - Error should indicate file not found

#### Test: track_calculate_checksum_empty_file
- **Method:** `Track::calculate_checksum()`
- **Test ID:** TR008
- **Priority:** Medium
- **Description:** Verify handling of empty file
- **Given:**
  - Create empty file in temp directory
  - Create Track pointing to it
- **When:** Calling `calculate_checksum()`
- **Then:**
  - Result.is_ok() == true
  - Checksum is valid SHA256 of empty file (e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855)

#### Test: track_calculate_checksum_unreadable_file
- **Method:** `Track::calculate_checksum()`
- **Test ID:** TR009
- **Priority:** Medium
- **Description:** Verify error handling for unreadable file
- **Given:**
  - Copy fixture file to temp
  - Remove read permissions (chmod 000)
  - Create Track
- **When:** Calling `calculate_checksum()`
- **Then:**
  - Result.is_err() == true
  - Error indicates permission denied

---

### Struct: Library

#### Test: library_new_empty
- **Method:** `Library::new()`
- **Test ID:** LB001
- **Priority:** High
- **Description:** Verify empty library creation
- **Given:** No arguments
- **When:** Calling `Library::new()`
- **Then:**
  - `artists` is empty Vec
  - `total_tracks` == 0
  - `total_artists` == 0
  - `total_albums` == 0
  - `total_files` == 0

#### Test: library_default_empty
- **Method:** `Library::default()`
- **Test ID:** LB002
- **Priority:** Medium
- **Description:** Verify Default trait implementation
- **Given:** No arguments
- **When:** Calling `Library::default()`
- **Then:**
  - All counters are 0
  - `artists` is empty

#### Test: library_add_artist_single
- **Method:** `Library::add_artist()`
- **Test ID:** LB003
- **Priority:** Critical
- **Description:** Verify adding single artist updates counters
- **Given:** 
  ```rust
  ArtistNode {
    name: "The Beatles",
    albums: vec![AlbumNode {
      title: "Abbey Road",
      tracks: vec![TrackNode, TrackNode, TrackNode], // 3 tracks
      files: HashSet with 3 paths,
      ...
    }]
  }
  ```
- **When:** Calling `library.add_artist(artist)`
- **Then:**
  - `total_artists` == 1
  - `total_albums` == 1
  - `total_tracks` == 3
  - `total_files` == 3
  - `artists.len()` == 1

#### Test: library_add_artist_multiple_albums
- **Method:** `Library::add_artist()`
- **Test ID:** LB004
- **Priority:** High
- **Description:** Verify counters with multiple albums
- **Given:** Artist with 3 albums:
  - Album 1: 5 tracks
  - Album 2: 8 tracks
  - Album 3: 12 tracks
- **When:** Adding artist to library
- **Then:**
  - `total_artists` == 1
  - `total_albums` == 3
  - `total_tracks` == 25

#### Test: library_add_artist_multiple_artists
- **Method:** `Library::add_artist()`
- **Test ID:** LB005
- **Priority:** High
- **Description:** Verify counters accumulate across multiple artists
- **Given:** Adding 3 artists:
  - Artist 1: 2 albums, 10 tracks total
  - Artist 2: 1 album, 5 tracks
  - Artist 3: 3 albums, 15 tracks
- **When:** Adding all artists
- **Then:**
  - `total_artists` == 3
  - `total_albums` == 6
  - `total_tracks` == 30

#### Test: library_add_artist_empty_albums
- **Method:** `Library::add_artist()`
- **Test ID:** LB006
- **Priority:** Medium
- **Description:** Verify handling of artist with no albums
- **Given:** ArtistNode with empty albums vector
- **When:** Adding to library
- **Then:**
  - `total_artists` == 1
  - `total_albums` == 0
  - `total_tracks` == 0

#### Test: library_add_artist_empty_tracks
- **Method:** `Library::add_artist()`
- **Test ID:** LB007
- **Priority:** Medium
- **Description:** Verify handling of album with no tracks
- **Given:** Artist with one album having empty tracks vector
- **When:** Adding to library
- **Then:**
  - `total_artists` == 1
  - `total_albums` == 1
  - `total_tracks` == 0

#### Test: library_serialization_roundtrip
- **Method:** Library serialization
- **Test ID:** LB008
- **Priority:** Medium
- **Description:** Verify Library can be serialized and deserialized
- **Given:** Library with multiple artists and albums
- **When:** 
  1. Serialize to JSON
  2. Deserialize back to Library
- **Then:**
  - Original == Deserialized
  - All counters match
  - All metadata preserved

---

## Module: core::domain::traits

### Struct: AudioFileRegistry

#### Test: registry_new_empty
- **Method:** `AudioFileRegistry::new()`
- **Test ID:** REG001
- **Priority:** High
- **Description:** Verify empty registry creation
- **Given:** No arguments
- **When:** Calling `AudioFileRegistry::new()`
- **Then:**
  - `handlers` is empty Vec
  - `supported_extensions()` returns empty Vec

#### Test: registry_register_handler
- **Method:** `AudioFileRegistry::register()`
- **Test ID:** REG002
- **Priority:** High
- **Description:** Verify handler registration
- **Given:** Empty registry, FlacHandler instance
- **When:** Calling `registry.register(Box::new(FlacHandler::new()))`
- **Then:**
  - Handler is added to internal handlers vector
  - `supported_extensions()` returns ["flac"]

#### Test: registry_register_multiple
- **Method:** `AudioFileRegistry::register()`
- **Test ID:** REG003
- **Priority:** High
- **Description:** Verify multiple handler registration
- **Given:** Empty registry
- **When:** Registering FlacHandler, Mp3Handler, WavHandler
- **Then:**
  - `supported_extensions()` contains "flac", "mp3", "wav"
  - Extensions are sorted alphabetically
  - No duplicates

#### Test: registry_find_handler_existing
- **Method:** `AudioFileRegistry::find_handler()`
- **Test ID:** REG004
- **Priority:** Critical
- **Description:** Verify finding handler for supported file
- **Given:** Registry with FlacHandler and Mp3Handler registered
- **When:** Calling `find_handler(Path::new("song.flac"))`
- **Then:**
  - Result.is_ok() == true
  - Handler.can_handle("song.flac") == true

#### Test: registry_find_handler_unsupported
- **Method:** `AudioFileRegistry::find_handler()`
- **Test ID:** REG005
- **Priority:** Critical
- **Description:** Verify error for unsupported file
- **Given:** Registry with only FlacHandler
- **When:** Calling `find_handler(Path::new("song.mp3"))`
- **Then:**
  - Result.is_err() == true
  - Error == `AudioFileError::UnsupportedFormat`

#### Test: registry_find_handler_case_insensitive
- **Method:** `AudioFileRegistry::find_handler()`
- **Test ID:** REG006
- **Priority:** High
- **Description:** Verify case-insensitive extension matching
- **Given:** Registry with FlacHandler
- **When:** Calling `find_handler()` for:
  - "song.flac"
  - "song.FLAC"
  - "song.Flac"
- **Then:**
  - All calls return Ok with FlacHandler

#### Test: registry_find_handler_no_extension
- **Method:** `AudioFileRegistry::find_handler()`
- **Test ID:** REG007
- **Priority:** Medium
- **Description:** Verify error for file without extension
- **Given:** Registry with handlers
- **When:** Calling `find_handler(Path::new("song"))`
- **Then:**
  - Result.is_err() == true
  - Error == `AudioFileError::UnsupportedFormat`

#### Test: registry_supported_extensions_empty
- **Method:** `AudioFileRegistry::supported_extensions()`
- **Test ID:** REG008
- **Priority:** Medium
- **Description:** Verify empty registry returns empty extensions
- **Given:** Empty registry
- **When:** Calling `supported_extensions()`
- **Then:**
  - Result is empty Vec<String>

#### Test: registry_supported_extensions_deduplication
- **Method:** `AudioFileRegistry::supported_extensions()`
- **Test ID:** REG009
- **Priority:** Medium
- **Description:** Verify duplicate extensions are removed
- **Given:** Registry with two handlers both supporting "flac"
- **When:** Calling `supported_extensions()`
- **Then:**
  - "flac" appears only once in result

#### Test: registry_supported_extensions_sorted
- **Method:** `AudioFileRegistry::supported_extensions()`
- **Test ID:** REG010
- **Priority:** Low
- **Description:** Verify extensions are sorted
- **Given:** Registry with handlers supporting "mp3", "flac", "wav" (registered in that order)
- **When:** Calling `supported_extensions()`
- **Then:**
  - Result == vec!["flac", "mp3", "wav"]

#### Test: registry_first_handler_wins
- **Method:** `AudioFileRegistry::find_handler()`
- **Test ID:** REG011
- **Priority:** Medium
- **Description:** Verify first matching handler is returned
- **Given:** Registry with CustomHandler1 then CustomHandler2, both handling .flac
- **When:** Calling `find_handler("song.flac")`
- **Then:**
  - Returns CustomHandler1 (first registered)

---

## Module: core::services::scanner

### Function: scan_dir

#### Test: scan_dir_empty_directory
- **Method:** `scan_dir()`
- **Test ID:** SC001
- **Priority:** High
- **Description:** Verify empty directory returns empty vector
- **Given:** Temp directory with no files
- **When:** Calling `scan_dir(&path, false)`
- **Then:**
  - Result is empty Vec<Track>
  - Result.len() == 0

#### Test: scan_dir_single_flac_file
- **Method:** `scan_dir()`
- **Test ID:** SC002
- **Priority:** Critical
- **Description:** Verify single FLAC file is found
- **Given:** 
  - Temp directory structure: `temp/artist/album/song.flac`
  - Copy fixture file to location
- **When:** Calling `scan_dir(&path, false)`
- **Then:**
  - Result.len() == 1
  - Track.file_path ends with "song.flac"
  - Track.metadata.format == "flac"
  - Track.metadata.artist.value == "artist" (inferred)
  - Track.metadata.album.value == "album" (inferred)

#### Test: scan_dir_multiple_files
- **Method:** `scan_dir()`
- **Test ID:** SC003
- **Priority:** Critical
- **Description:** Verify multiple files are all found
- **Given:** Temp directory with 5 FLAC files in various subdirectories
- **When:** Calling `scan_dir(&path, false)`
- **Then:**
  - Result.len() == 5
  - All tracks have unique file paths
  - All tracks have format == "flac"

#### Test: scan_dir_mixed_formats
- **Method:** `scan_dir()`
- **Test ID:** SC004
- **Priority:** High
- **Description:** Verify only supported formats are returned
- **Given:** Directory with:
  - 3 FLAC files
  - 2 MP3 files
  - 1 WAV file
  - 1 OGG file (unsupported)
  - 1 TXT file
- **When:** Calling `scan_dir(&path, false)`
- **Then:**
  - Result.len() == 6 (3 FLAC + 2 MP3 + 1 WAV)
  - No OGG or TXT files in results

#### Test: scan_dir_deterministic_order
- **Method:** `scan_dir()`
- **Test ID:** SC005
- **Priority:** High
- **Description:** Verify results are sorted deterministically
- **Given:** Directory with files: "z.flac", "a.flac", "m.flac"
- **When:** Calling `scan_dir()` twice
- **Then:**
  - Both calls return same order: "a.flac", "m.flac", "z.flac"
  - Tracks sorted by filename

#### Test: scan_dir_deep_nesting
- **Method:** `scan_dir()`
- **Test ID:** SC006
- **Priority:** High
- **Description:** Verify recursive scanning works
- **Given:** Directory structure 5 levels deep with FLAC files at each level
- **When:** Calling `scan_dir(&base_path, false)`
- **Then:**
  - All files at all levels are found
  - Result.len() == number of FLAC files

#### Test: scan_dir_nonexistent_path
- **Method:** `scan_dir()`
- **Test ID:** SC007
- **Priority:** Medium
- **Description:** Verify handling of nonexistent path
- **Given:** Path `/nonexistent/directory`
- **When:** Calling `scan_dir(&path, false)`
- **Then:**
  - Result is empty Vec (or graceful handling)
  - No panic

#### Test: scan_dir_skip_metadata_true
- **Method:** `scan_dir()`
- **Test ID:** SC008
- **Priority:** Critical
- **Description:** Verify skip_metadata=true doesn't read file metadata
- **Given:** FLAC file with embedded title "Real Title"
- **When:** Calling `scan_dir(&path, true)`
- **Then:**
  - Track.metadata.title is None OR uses filename
  - Track.metadata.artist.source == FolderInferred
  - No file reading errors

#### Test: scan_dir_skip_metadata_false
- **Method:** `scan_dir()`
- **Test ID:** SC009
- **Priority:** Critical
- **Description:** Verify skip_metadata=false reads embedded metadata
- **Given:** FLAC file with embedded title "Embedded Title"
- **When:** Calling `scan_dir(&path, false)`
- **Then:**
  - Track.metadata.title.value == "Embedded Title"
  - Track.metadata.title.source == Embedded

#### Test: scan_dir_empty_files_skipped
- **Method:** `scan_dir()`
- **Test ID:** SC010
- **Priority:** Medium
- **Description:** Verify empty files are skipped with warning
- **Given:** Directory with:
  - 1 valid FLAC file
  - 1 empty FLAC file (0 bytes)
- **When:** Calling `scan_dir(&path, false)`
- **Then:**
  - Result.len() == 1
  - Empty file is not included
  - Warning is logged

---

### Function: scan_dir_paths

#### Test: scan_dir_paths_basic
- **Method:** `scan_dir_paths()`
- **Test ID:** SCP001
- **Priority:** High
- **Description:** Verify returns only paths without metadata
- **Given:** Directory with 3 FLAC files
- **When:** Calling `scan_dir_paths(&path)`
- **Then:**
  - Result.len() == 3
  - All items are PathBuf
  - Results are sorted

#### Test: scan_dir_paths_no_metadata_read
- **Method:** `scan_dir_paths()`
- **Test ID:** SCP002
- **Priority:** High
- **Description:** Verify no metadata parsing occurs
- **Given:** Directory with corrupted FLAC file (unreadable metadata)
- **When:** Calling `scan_dir_paths(&path)`
- **Then:**
  - Corrupted file is still included (path only)
  - No errors about unreadable metadata

---

### Function: scan_dir_immediate

#### Test: scan_dir_immediate_non_recursive
- **Method:** `scan_dir_immediate()`
- **Test ID:** SCI001
- **Priority:** Critical
- **Description:** Verify only immediate files are returned
- **Given:** Directory structure:
  ```
  base/
    file1.flac
    file2.flac
    subdir/
      file3.flac
  ```
- **When:** Calling `scan_dir_immediate(&base)`
- **Then:**
  - Result.len() == 2
  - file1.flac and file2.flac included
  - file3.flac NOT included

#### Test: scan_dir_immediate_empty
- **Method:** `scan_dir_immediate()`
- **Test ID:** SCI002
- **Priority:** Medium
- **Description:** Verify empty directory returns empty
- **Given:** Empty directory
- **When:** Calling `scan_dir_immediate(&path)`
- **Then:**
  - Result is empty Vec

#### Test: scan_dir_immediate_nonexistent
- **Method:** `scan_dir_immediate()`
- **Test ID:** SCI003
- **Priority:** Medium
- **Description:** Verify nonexistent path returns empty
- **Given:** Nonexistent path
- **When:** Calling `scan_dir_immediate(&path)`
- **Then:**
  - Result is empty Vec
  - No panic

---

### Function: scan_dir_with_metadata

#### Test: scan_dir_with_metadata_success
- **Method:** `scan_dir_with_metadata()`
- **Test ID:** SCM001
- **Priority:** Critical
- **Description:** Verify full metadata extraction
- **Given:** Directory with FLAC file containing:
  - Title: "Test Song"
  - Artist: "Test Artist"
  - Album: "Test Album"
- **When:** Calling `scan_dir_with_metadata(&path)`
- **Then:**
  - Result.is_ok() == true
  - First track has all metadata fields populated
  - All metadata sources are Embedded

#### Test: scan_dir_with_metadata_partial
- **Method:** `scan_dir_with_metadata()`
- **Test ID:** SCM002
- **Priority:** High
- **Description:** Verify files with partial metadata
- **Given:** FLAC file with only title set
- **When:** Calling `scan_dir_with_metadata(&path)`
- **Then:**
  - Title is Some(Embedded)
  - Artist is Some(FolderInferred) (from folder name)
  - Album is Some(FolderInferred)

#### Test: scan_dir_with_metadata_empty_directory
- **Method:** `scan_dir_with_metadata()`
- **Test ID:** SCM003
- **Priority:** Medium
- **Description:** Verify empty directory handling
- **Given:** Empty directory
- **When:** Calling `scan_dir_with_metadata(&path)`
- **Then:**
  - Result.is_ok() == true
  - Result.unwrap() is empty Vec

#### Test: scan_dir_with_metadata_unreadable_file
- **Method:** `scan_dir_with_metadata()`
- **Test ID:** SCM004
- **Priority:** Medium
- **Description:** Verify graceful handling of unreadable files
- **Given:** Directory with:
  - 1 valid FLAC file
  - 1 corrupted FLAC file
- **When:** Calling `scan_dir_with_metadata(&path)`
- **Then:**
  - Result.is_ok() == true
  - Valid file is in results
  - Corrupted file is skipped with warning

---

### Function: scan_with_duplicates

#### Test: scan_with_duplicates_no_duplicates
- **Method:** `scan_with_duplicates()`
- **Test ID:** SCD001
- **Priority:** Critical
- **Description:** Verify handling when no duplicates exist
- **Given:** Directory with 3 unique FLAC files
- **When:** Calling `scan_with_duplicates(&path)`
- **Then:**
  - Returns (tracks, duplicates)
  - tracks.len() == 3
  - duplicates is empty Vec
  - All tracks have checksum populated

#### Test: scan_with_duplicates_single_duplicate
- **Method:** `scan_with_duplicates()`
- **Test ID:** SCD002
- **Priority:** Critical
- **Description:** Verify detection of single duplicate pair
- **Given:** 
  - Copy same FLAC file twice: file1.flac, file2.flac
- **When:** Calling `scan_with_duplicates(&path)`
- **Then:**
  - tracks.len() == 2
  - duplicates.len() == 1
  - duplicates[0].len() == 2
  - Both tracks in duplicates[0] have same checksum

#### Test: scan_with_duplicates_multiple_groups
- **Method:** `scan_with_duplicates()`
- **Test ID:** SCD003
- **Priority:** High
- **Description:** Verify multiple duplicate groups
- **Given:**
  - Group 1: 3 copies of file A
  - Group 2: 2 copies of file B
  - 1 unique file C
- **When:** Calling `scan_with_duplicates(&path)`
- **Then:**
  - tracks.len() == 6
  - duplicates.len() == 2
  - duplicates[0].len() == 3
  - duplicates[1].len() == 2

#### Test: scan_with_duplicates_checksum_failure
- **Method:** `scan_with_duplicates()`
- **Test ID:** SCD004
- **Priority:** Medium
- **Description:** Verify handling when checksum calculation fails
- **Given:** Directory with:
  - 1 readable file
  - 1 file with permissions removed
- **When:** Calling `scan_with_duplicates(&path)`
- **Then:**
  - Both tracks in results
  - Readable track has checksum
  - Unreadable track has None checksum
  - Warning is logged for failed checksum

---

### Function: scan_dir_with_depth

#### Test: scan_dir_with_depth_zero
- **Method:** `scan_dir_with_depth()`
- **Test ID:** SCDP001
- **Priority:** Critical
- **Description:** Verify depth 0 only returns immediate files
- **Given:**
  ```
  base/
    file1.flac
    subdir1/
      file2.flac
      subdir2/
        file3.flac
  ```
- **When:** Calling `scan_dir_with_depth(&base, Some(0))`
- **Then:**
  - Only file1.flac is returned
  - Results.len() == 1

#### Test: scan_dir_with_depth_one
- **Method:** `scan_dir_with_depth()`
- **Test ID:** SCDP002
- **Priority:** Critical
- **Description:** Verify depth 1 returns base + 1 level deep
- **Given:** Same structure as SCDP001
- **When:** Calling `scan_dir_with_depth(&base, Some(1))`
- **Then:**
  - file1.flac and file2.flac are returned
  - file3.flac is NOT returned
  - Results.len() == 2

#### Test: scan_dir_with_depth_unlimited
- **Method:** `scan_dir_with_depth()`
- **Test ID:** SCDP003
- **Priority:** High
- **Description:** Verify None depth scans all levels
- **Given:** Same structure as SCDP001
- **When:** Calling `scan_dir_with_depth(&base, None)`
- **Then:**
  - All 3 files are returned
  - Results.len() == 3

#### Test: scan_dir_with_depth_deep
- **Method:** `scan_dir_with_depth()`
- **Test ID:** SCDP004
- **Priority:** Medium
- **Description:** Verify depth limiting with deep nesting
- **Given:** 10 levels of nesting with 1 file at each level
- **When:** Calling `scan_dir_with_depth(&base, Some(5))`
- **Then:**
  - Returns files from levels 0-5
  - Does NOT return files from levels 6-9

---

### Function: format_track_name_for_scan_output

#### Test: format_track_name_cue_inferred
- **Method:** `format_track_name_for_scan_output()`
- **Test ID:** FTN001
- **Priority:** High
- **Description:** Verify CUE-inferred title formatting
- **Given:** Track with:
  - title: Some(MetadataValue::cue_inferred("Song Title", 1.0))
  - file_path: "/music/track.flac"
- **When:** Calling `format_track_name_for_scan_output(&track)`
- **Then:**
  - Result contains "Song Title (track.flac)"
  - Result contains "📄" icon

#### Test: format_track_name_embedded
- **Method:** `format_track_name_for_scan_output()`
- **Test ID:** FTN002
- **Priority:** High
- **Description:** Verify embedded title formatting
- **Given:** Track with:
  - title: Some(MetadataValue::embedded("Embedded Song"))
  - file_path: "/music/track.flac"
- **When:** Calling `format_track_name_for_scan_output(&track)`
- **Then:**
  - Result contains "Embedded Song"
  - Result contains "🎯" icon
  - Does NOT contain filename

#### Test: format_track_name_folder_inferred
- **Method:** `format_track_name_for_scan_output()`
- **Test ID:** FTN003
- **Priority:** High
- **Description:** Verify folder-inferred fallback formatting
- **Given:** Track with:
  - title: None
  - artist: Some(MetadataValue::inferred("Artist", 0.3))
  - file_path: "/music/track.flac"
- **When:** Calling `format_track_name_for_scan_output(&track)`
- **Then:**
  - Result contains "track.flac" (filename)
  - Result contains "🤖" icon

#### Test: format_track_name_user_edited
- **Method:** `format_track_name_for_scan_output()`
- **Test ID:** FTN004
- **Priority:** Medium
- **Description:** Verify user-edited title formatting
- **Given:** Track with:
  - title: Some(MetadataValue::user_set("User Title"))
- **When:** Calling `format_track_name_for_scan_output(&track)`
- **Then:**
  - Result contains "User Title"
  - Result contains "👤" icon

---

## Module: core::services::inference

### Function: infer_artist_from_path

#### Test: infer_artist_from_folder_pattern
- **Method:** `infer_artist_from_path()`
- **Test ID:** INF001
- **Priority:** Critical
- **Description:** Extract artist from "Artist - Album" folder pattern
- **Given:** Path: "/music/The Beatles - Abbey Road/01.flac"
- **When:** Calling `infer_artist_from_path(&path)`
- **Then:**
  - Result == Some("The Beatles")

#### Test: infer_artist_from_filename_pattern
- **Method:** `infer_artist_from_path()`
- **Test ID:** INF002
- **Priority:** Critical
- **Description:** Extract artist from "Artist - Title" filename pattern
- **Given:** Path: "/music/Unknown/The Beatles - Help.mp3"
- **When:** Calling `infer_artist_from_path(&path)`
- **Then:**
  - Result == Some("The Beatles")

#### Test: infer_artist_from_standard_structure
- **Method:** `infer_artist_from_path()`
- **Test ID:** INF003
- **Priority:** Critical
- **Description:** Extract artist from Artist/Album/track structure
- **Given:** Path: "/music/Pink Floyd/Dark Side/01.flac"
- **When:** Calling `infer_artist_from_path(&path)`
- **Then:**
  - Result == Some("Pink Floyd")

#### Test: infer_artist_from_collection_folder
- **Method:** `infer_artist_from_path()`
- **Test ID:** INF004
- **Priority:** High
- **Description:** Extract artist from Artist/Albums/Album structure
- **Given:** Path: "/music/Metallica/Albums/Master of Puppets/01.flac"
- **When:** Calling `infer_artist_from_path(&path)`
- **Then:**
  - Result == Some("Metallica")

#### Test: infer_artist_no_artist_found
- **Method:** `infer_artist_from_path()`
- **Test ID:** INF005
- **Priority:** High
- **Description:** Return None when no artist can be inferred
- **Given:** Path: "/music/01 - Track.flac" (no artist info)
- **When:** Calling `infer_artist_from_path(&path)`
- **Then:**
  - Result == None

#### Test: infer_artist_invalid_artist_name
- **Method:** `infer_artist_from_path()`
- **Test ID:** INF006
- **Priority:** Medium
- **Description:** Reject numeric-only or invalid artist names
- **Given:** Path: "/music/2024 - Album/01.flac"
- **When:** Calling `infer_artist_from_path(&path)`
- **Then:**
  - Result == None ("2024" is not a valid artist)

#### Test: infer_artist_cleans_format_suffix
- **Method:** `infer_artist_from_path()`
- **Test ID:** INF007
- **Priority:** Medium
- **Description:** Remove [FLAC], [MP3], etc. suffixes
- **Given:** Path: "/music/Artist [FLAC]/Album/track.flac"
- **When:** Calling `infer_artist_from_path(&path)`
- **Then:**
  - Result == Some("Artist")

#### Test: infer_artist_cleans_year_suffix
- **Method:** `infer_artist_from_path()`
- **Test ID:** INF008
- **Priority:** Medium
- **Description:** Remove year suffixes like "Artist 2024"
- **Given:** Path: "/music/Band 2024 - Album/track.flac"
- **When:** Calling `infer_artist_from_path(&path)`
- **Then:**
  - Result == Some("Band")

---

### Function: infer_album_from_path

#### Test: infer_album_from_folder_pattern
- **Method:** `infer_album_from_path()`
- **Test ID:** INF009
- **Priority:** Critical
- **Description:** Extract album from "Artist - Album" folder pattern
- **Given:** Path: "/music/The Beatles - Abbey Road/01.flac"
- **When:** Calling `infer_album_from_path(&path)`
- **Then:**
  - Result == Some("Abbey Road")

#### Test: infer_album_from_simple_folder
- **Method:** `infer_album_from_path()`
- **Test ID:** INF010
- **Priority:** Critical
- **Description:** Use folder name as album when no separator
- **Given:** Path: "/music/Artist/Abbey Road/01.flac"
- **When:** Calling `infer_album_from_path(&path)`
- **Then:**
  - Result == Some("Abbey Road")

#### Test: infer_album_cleans_format_suffix
- **Method:** `infer_album_from_path()`
- **Test ID:** INF011
- **Priority:** Medium
- **Description:** Remove [FLAC], - FLAC, etc. from album name
- **Given:** Path: "/music/Album Title [FLAC]/track.flac"
- **When:** Calling `infer_album_from_path(&path)`
- **Then:**
  - Result == Some("Album Title")

#### Test: infer_album_cleans_year_parentheses
- **Method:** `infer_album_from_path()`
- **Test ID:** INF012
- **Priority:** Medium
- **Description:** Remove year in parentheses from album
- **Given:** Path: "/music/Album Title (2009)/track.flac"
- **When:** Calling `infer_album_from_path(&path)`
- **Then:**
  - Result == Some("Album Title")

#### Test: infer_album_cleans_metadata_parentheses
- **Method:** `infer_album_from_path()`
- **Test ID:** INF013
- **Priority:** Medium
- **Description:** Remove metadata like "Album (2009, CD, Remaster)"
- **Given:** Path: "/music/Album (2009, CD, Remaster)/track.flac"
- **When:** Calling `infer_album_from_path(&path)`
- **Then:**
  - Result == Some("Album")

---

### Function: infer_year_from_path

#### Test: infer_year_from_folder_prefix
- **Method:** `infer_year_from_path()`
- **Test ID:** INF014
- **Priority:** High
- **Description:** Extract year from "YYYY - Album" folder name
- **Given:** Path: "/music/1969 - Abbey Road/track.flac"
- **When:** Calling `infer_year_from_path(&path)`
- **Then:**
  - Result == Some(1969)

#### Test: infer_year_from_folder_parentheses
- **Method:** `infer_year_from_path()`
- **Test ID:** INF015
- **Priority:** High
- **Description:** Extract year from "Album (YYYY)" folder name
- **Given:** Path: "/music/Abbey Road (1969)/track.flac"
- **When:** Calling `infer_year_from_path(&path)`
- **Then:**
  - Result == Some(1969)

#### Test: infer_year_from_filename
- **Method:** `infer_year_from_path()`
- **Test ID:** INF016
- **Priority:** Medium
- **Description:** Extract year from filename
- **Given:** Path: "/music/track 1969.flac"
- **When:** Calling `infer_year_from_path(&path)`
- **Then:**
  - Result == Some(1969)

#### Test: infer_year_invalid_range
- **Method:** `infer_year_from_path()`
- **Test ID:** INF017
- **Priority:** Medium
- **Description:** Reject years outside 1900-2100 range
- **Given:** Path: "/music/1850 - Album/track.flac"
- **When:** Calling `infer_year_from_path(&path)`
- **Then:**
  - Result == None

#### Test: infer_year_no_year_found
- **Method:** `infer_year_from_path()`
- **Test ID:** INF018
- **Priority:** Low
- **Description:** Return None when no year present
- **Given:** Path: "/music/Album Name/track.flac"
- **When:** Calling `infer_year_from_path(&path)`
- **Then:**
  - Result == None

---

## Module: core::services::normalization

### Function: to_title_case

#### Test: to_title_case_basic
- **Method:** `to_title_case()`
- **Test ID:** NRM001
- **Priority:** Critical
- **Description:** Convert lowercase to title case
- **Given:** "hello world"
- **When:** Calling `to_title_case(input)`
- **Then:**
  - Result == "Hello World"

#### Test: to_title_case_uppercase
- **Method:** `to_title_case()`
- **Test ID:** NRM002
- **Priority:** High
- **Description:** Convert ALL CAPS to title case
- **Given:** "HELLO WORLD"
- **When:** Calling `to_title_case(input)`
- **Then:**
  - Result == "Hello World"

#### Test: to_title_case_mixed_separators
- **Method:** `to_title_case()`
- **Test ID:** NRM003
- **Priority:** High
- **Description:** Handle spaces, hyphens, and underscores
- **Given:** "hello-world_test string"
- **When:** Calling `to_title_case(input)`
- **Then:**
  - Result == "Hello-World_Test String"

#### Test: to_title_case_already_correct
- **Method:** `to_title_case()`
- **Test ID:** NRM004
- **Priority:** Medium
- **Description:** Handle already correct title case
- **Given:** "Already Title Case"
- **When:** Calling `to_title_case(input)`
- **Then:**
  - Result == "Already Title Case"

#### Test: to_title_case_empty
- **Method:** `to_title_case()`
- **Test ID:** NRM005
- **Priority:** Low
- **Description:** Handle empty string
- **Given:** ""
- **When:** Calling `to_title_case(input)`
- **Then:**
  - Result == ""

#### Test: to_title_case_single_char
- **Method:** `to_title_case()`
- **Test ID:** NRM006
- **Priority:** Low
- **Description:** Handle single character
- **Given:** "a"
- **When:** Calling `to_title_case(input)`
- **Then:**
  - Result == "A"

#### Test: to_title_case_leading_trailing_spaces
- **Method:** `to_title_case()`
- **Test ID:** NRM007
- **Priority:** Low
- **Description:** Preserve leading/trailing spaces
- **Given:** "  hello world  "
- **When:** Calling `to_title_case(input)`
- **Then:**
  - Result == "  Hello World  "

---

### Function: normalize_genre

#### Test: normalize_genre_rock_alias
- **Method:** `normalize_genre()`
- **Test ID:** NRG001
- **Priority:** Critical
- **Description:** Normalize rock aliases to "Rock"
- **Given:** "rock and roll"
- **When:** Calling `normalize_genre(input)`
- **Then:**
  - Result == Some("Rock")

#### Test: normalize_genre_standard_list
- **Method:** `normalize_genre()`
- **Test ID:** NRG002
- **Priority:** Critical
- **Description:** Standard genres remain standard
- **Given:** Each of STANDARD_GENRES list
- **When:** Calling `normalize_genre(input.to_lowercase())`
- **Then:**
  - Result == Some(input) for all standard genres

#### Test: normalize_genre_unknown
- **Method:** `normalize_genre()`
- **Test ID:** NRG003
- **Priority:** High
- **Description:** Unknown genres get title-cased
- **Given:** "custom genre name"
- **When:** Calling `normalize_genre(input)`
- **Then:**
  - Result == Some("Custom Genre Name")

#### Test: normalize_genre_slash_separated
- **Method:** `normalize_genre()`
- **Test ID:** NRG004
- **Priority:** High
- **Description:** Handle slash-separated genres
- **Given:** "rock/electronic/jazz"
- **When:** Calling `normalize_genre(input)`
- **Then:**
  - Result == Some("Rock/Electronic/Jazz")

#### Test: normalize_genre_empty
- **Method:** `normalize_genre()`
- **Test ID:** NRG005
- **Priority:** Medium
- **Description:** Handle empty string
- **Given:** ""
- **When:** Calling `normalize_genre(input)`
- **Then:**
  - Result == None

#### Test: normalize_genre_whitespace_only
- **Method:** `normalize_genre()`
- **Test ID:** NRG006
- **Priority:** Low
- **Description:** Handle whitespace-only string
- **Given:** "   "
- **When:** Calling `normalize_genre(input)`
- **Then:**
  - Result == None

#### Test: normalize_genre_case_insensitive
- **Method:** `normalize_genre()`
- **Test ID:** NRG007
- **Priority:** Medium
- **Description:** Handle various cases
- **Given:** "ROCK", "Rock", "rock", "rOcK"
- **When:** Calling `normalize_genre(input)` for each
- **Then:**
  - All return Some("Rock")

---

## Module: core::services::library

### Function: build_library_hierarchy

#### Test: build_hierarchy_empty
- **Method:** `build_library_hierarchy()`
- **Test ID:** LIB001
- **Priority:** Critical
- **Description:** Handle empty track list
- **Given:** Empty Vec<Track>
- **When:** Calling `build_library_hierarchy(tracks)`
- **Then:**
  - Library.total_artists == 0
  - Library.total_albums == 0
  - Library.total_tracks == 0
  - Library.artists is empty

#### Test: build_hierarchy_single_artist_single_album
- **Method:** `build_library_hierarchy()`
- **Test ID:** LIB002
- **Priority:** Critical
- **Description:** Build hierarchy with single artist and album
- **Given:** 3 tracks with:
  - artist: "Artist A"
  - album: "Album X"
- **When:** Calling `build_library_hierarchy(tracks)`
- **Then:**
  - Library.total_artists == 1
  - Library.total_albums == 1
  - Library.total_tracks == 3
  - library.artists[0].name == "Artist A"
  - library.artists[0].albums[0].title == "Album X"
  - library.artists[0].albums[0].tracks.len() == 3

#### Test: build_hierarchy_multiple_artists
- **Method:** `build_library_hierarchy()`
- **Test ID:** LIB003
- **Priority:** Critical
- **Description:** Group tracks by artist
- **Given:**
  - 2 tracks: artist="Artist A"
  - 3 tracks: artist="Artist B"
  - 1 track: artist="Artist C"
- **When:** Calling `build_library_hierarchy(tracks)`
- **Then:**
  - Library.total_artists == 3
  - library.artists.len() == 3
  - Each artist has correct number of tracks

#### Test: build_hierarchy_multiple_albums_per_artist
- **Method:** `build_library_hierarchy()`
- **Test ID:** LIB004
- **Priority:** High
- **Description:** Group tracks by album within artist
- **Given:**
  - Artist A:
    - 2 tracks: album="Album 1"
    - 3 tracks: album="Album 2"
- **When:** Calling `build_library_hierarchy(tracks)`
- **Then:**
  - library.artists[0].albums.len() == 2
  - Album 1 has 2 tracks
  - Album 2 has 3 tracks

#### Test: build_hierarchy_unknown_artist
- **Method:** `build_library_hierarchy()`
- **Test ID:** LIB005
- **Priority:** High
- **Description:** Handle tracks without artist metadata
- **Given:** 2 tracks with artist=None
- **When:** Calling `build_library_hierarchy(tracks)`
- **Then:**
  - library.artists contains one with name="Unknown Artist"
  - "Unknown Artist" has 2 tracks

#### Test: build_hierarchy_unknown_album
- **Method:** `build_library_hierarchy()`
- **Test ID:** LIB006
- **Priority:** High
- **Description:** Handle tracks without album metadata
- **Given:** Track with artist="Artist A", album=None
- **When:** Calling `build_library_hierarchy(tracks)`
- **Then:**
  - Artist A has album with title="Unknown Album"

#### Test: build_hierarchy_preserves_year
- **Method:** `build_library_hierarchy()`
- **Test ID:** LIB007
- **Priority:** Medium
- **Description:** Extract year from track metadata
- **Given:** Tracks with year=Some(2020) in metadata
- **When:** Calling `build_library_hierarchy(tracks)`
- **Then:**
  - AlbumNode.year == Some(2020)

#### Test: build_hierarchy_preserves_file_paths
- **Method:** `build_library_hierarchy()`
- **Test ID:** LIB008
- **Priority:** Medium
- **Description:** Verify all file paths are preserved
- **Given:** 5 tracks with unique paths
- **When:** Calling `build_library_hierarchy(tracks)`
- **Then:**
  - All 5 file paths present in AlbumNode.files HashSet
  - All 5 TrackNodes have correct file_path

---

## Module: core::services::validation

### Function: validate_tracks

#### Test: validate_tracks_all_valid
- **Method:** `validate_tracks()`
- **Test ID:** VAL001
- **Priority:** Critical
- **Description:** Validate tracks with all required fields
- **Given:** 3 tracks with title, artist, album all set
- **When:** Calling `validate_tracks(tracks)`
- **Then:**
  - result.valid == true
  - result.errors is empty
  - result.summary.total_files == 3
  - result.summary.valid_files == 3
  - result.summary.files_with_errors == 0

#### Test: validate_tracks_missing_title
- **Method:** `validate_tracks()`
- **Test ID:** VAL002
- **Priority:** Critical
- **Description:** Detect missing title field
- **Given:** Track with artist and album, but title=None
- **When:** Calling `validate_tracks(vec![track])`
- **Then:**
  - result.valid == false
  - result.errors.len() == 1
  - result.errors[0].field == "title"
  - result.errors[0].message contains "Missing required field"
  - result.summary.files_with_errors == 1

#### Test: validate_tracks_missing_artist
- **Method:** `validate_tracks()`
- **Test ID:** VAL003
- **Priority:** Critical
- **Description:** Detect missing artist field
- **Given:** Track with title and album, but artist=None
- **When:** Calling `validate_tracks(vec![track])`
- **Then:**
  - result.valid == false
  - result.errors.len() == 1
  - result.errors[0].field == "artist"

#### Test: validate_tracks_missing_album
- **Method:** `validate_tracks()`
- **Test ID:** VAL004
- **Priority:** Critical
- **Description:** Detect missing album field
- **Given:** Track with title and artist, but album=None
- **When:** Calling `validate_tracks(vec![track])`
- **Then:**
  - result.valid == false
  - result.errors.len() == 1
  - result.errors[0].field == "album"

#### Test: validate_tracks_multiple_missing
- **Method:** `validate_tracks()`
- **Test ID:** VAL005
- **Priority:** High
- **Description:** Track missing all required fields
- **Given:** Track with title=None, artist=None, album=None
- **When:** Calling `validate_tracks(vec![track])`
- **Then:**
  - result.valid == false
  - result.errors.len() == 3
  - Errors for title, artist, and album

#### Test: validate_tracks_empty_title
- **Method:** `validate_tracks()`
- **Test ID:** VAL006
- **Priority:** High
- **Description:** Detect empty title field
- **Given:** Track with title="" (empty string)
- **When:** Calling `validate_tracks(vec![track])`
- **Then:**
  - result.valid == false
  - Error for title field with message "Title field is empty"

#### Test: validate_tracks_whitespace_only_title
- **Method:** `validate_tracks()`
- **Test ID:** VAL007
- **Priority:** Medium
- **Description:** Detect whitespace-only title
- **Given:** Track with title="   " (spaces only)
- **When:** Calling `validate_tracks(vec![track])`
- **Then:**
  - result.valid == false
  - Error for title field

#### Test: validate_tracks_missing_track_number_warning
- **Method:** `validate_tracks()`
- **Test ID:** VAL008
- **Priority:** Medium
- **Description:** Warn about missing track number
- **Given:** Track with all required fields, track_number=None
- **When:** Calling `validate_tracks(vec![track])`
- **Then:**
  - result.valid == true (not an error)
  - result.warnings.len() == 1
  - result.warnings[0].field == "track_number"
  - result.summary.files_with_warnings == 1

#### Test: validate_tracks_missing_year_warning
- **Method:** `validate_tracks()`
- **Test ID:** VAL009
- **Priority:** Medium
- **Description:** Warn about missing year
- **Given:** Track with all required fields, year=None
- **When:** Calling `validate_tracks(vec![track])`
- **Then:**
  - result.valid == true
  - result.warnings.len() >= 1
  - Warning for year field

#### Test: validate_tracks_invalid_year_warning
- **Method:** `validate_tracks()`
- **Test ID:** VAL010
- **Priority:** Medium
- **Description:** Warn about unusual year values
- **Given:** Track with year=1800 (before 1900)
- **When:** Calling `validate_tracks(vec![track])`
- **Then:**
  - result.warnings contains warning about year
  - Warning message mentions "seems unusual"

#### Test: validate_tracks_invalid_track_number_warning
- **Method:** `validate_tracks()`
- **Test ID:** VAL011
- **Priority:** Medium
- **Description:** Warn about unusual track numbers
- **Given:** Track with track_number=0 or track_number=150
- **When:** Calling `validate_tracks(vec![track])`
- **Then:**
  - result.warnings contains warning about track_number

#### Test: validate_tracks_very_long_title_warning
- **Method:** `validate_tracks()`
- **Test ID:** VAL012
- **Priority:** Low
- **Description:** Warn about very long titles
- **Given:** Track with title of 250+ characters
- **When:** Calling `validate_tracks(vec![track])`
- **Then:**
  - result.warnings contains warning about title length

---

### Function: validate_path

#### Test: validate_path_success
- **Method:** `validate_path()`
- **Test ID:** VALP001
- **Priority:** High
- **Description:** Validate directory with valid files
- **Given:** Directory with 3 valid FLAC files
- **When:** Calling `validate_path(&path, false)`
- **Then:**
  - Result.is_ok() == true
  - Output contains "All files passed validation"

#### Test: validate_path_with_errors
- **Method:** `validate_path()`
- **Test ID:** VALP002
- **Priority:** High
- **Description:** Validate directory with invalid files
- **Given:** Directory with:
  - 1 valid file
  - 1 file missing title
  - 1 file missing artist
- **When:** Calling `validate_path(&path, false)`
- **Then:**
  - Result.is_ok() == true (operation succeeded)
  - Output contains errors
  - Output shows "Validation failed with X errors"

#### Test: validate_path_empty_directory
- **Method:** `validate_path()`
- **Test ID:** VALP003
- **Priority:** Medium
- **Description:** Handle empty directory
- **Given:** Empty directory
- **When:** Calling `validate_path(&path, false)`
- **Then:**
  - Result.is_err() == true
  - Error message contains "No music files found"

#### Test: validate_path_json_output
- **Method:** `validate_path()`
- **Test ID:** VALP004
- **Priority:** High
- **Description:** Verify JSON output format
- **Given:** Directory with 2 valid files and 1 invalid
- **When:** Calling `validate_path(&path, true)` (json=true)
- **Then:**
  - Result.is_ok() == true
  - Output is valid JSON
  - JSON contains: valid, errors, warnings, summary fields
  - summary.total_files == 3
  - summary.files_with_errors == 1

---

## Module: core::services::apply_metadata

### Function: write_metadata_by_path

#### Test: write_metadata_dry_run
- **Method:** `write_metadata_by_path()`
- **Test ID:** AMD001
- **Priority:** Critical
- **Description:** Dry run mode does not modify file
- **Given:**
  - Copy fixture FLAC file to temp
  - Original metadata title="Original"
- **When:** Calling with:
  - set=["title=New Title"]
  - apply=false
  - dry_run=true
- **Then:**
  - Result.is_ok() == true
  - Output contains "DRY RUN"
  - File metadata unchanged (title still "Original")

#### Test: write_metadata_apply_changes
- **Method:** `write_metadata_by_path()`
- **Test ID:** AMD002
- **Priority:** Critical
- **Description:** Apply mode actually modifies file
- **Given:**
  - Copy fixture FLAC file to temp
  - Current title="Original"
- **When:** Calling with:
  - set=["title=New Title"]
  - apply=true
  - dry_run=false
- **Then:**
  - Result.is_ok() == true
  - Output contains "Successfully updated"
  - File metadata updated (title="New Title")

#### Test: write_metadata_default_dry_run
- **Method:** `write_metadata_by_path()`
- **Test ID:** AMD003
- **Priority:** High
- **Description:** Default behavior is dry run when no flags
- **Given:**
  - Copy fixture file
- **When:** Calling with:
  - set=["title=New"]
  - apply=false
  - dry_run=false (both false)
- **Then:**
  - Acts as dry run
  - File not modified

#### Test: write_metadata_apply_and_dry_run_conflict
- **Method:** `write_metadata_by_path()`
- **Test ID:** AMD004
- **Priority:** High
- **Description:** Cannot use both apply and dry-run
- **Given:** Any file path
- **When:** Calling with apply=true AND dry_run=true
- **Then:**
  - Result.is_err() == true
  - Error message about conflicting flags

#### Test: write_metadata_nonexistent_file
- **Method:** `write_metadata_by_path()`
- **Test ID:** AMD005
- **Priority:** High
- **Description:** Error on nonexistent file
- **Given:** Path `/nonexistent/file.flac`
- **When:** Calling function
- **Then:**
  - Result.is_err() == true
  - Error message contains "does not exist"

#### Test: write_metadata_unsupported_format
- **Method:** `write_metadata_by_path()`
- **Test ID:** AMD006
- **Priority:** High
- **Description:** Error on unsupported format
- **Given:** File with .ogg extension (unsupported)
- **When:** Calling with apply=true
- **Then:**
  - Result.is_err() == true
  - Error message contains "Unsupported file format"

#### Test: write_metadata_multiple_fields
- **Method:** `write_metadata_by_path()`
- **Test ID:** AMD007
- **Priority:** High
- **Description:** Update multiple fields at once
- **Given:** Copy of fixture file
- **When:** Calling with set=[
  "title=New Title",
  "artist=New Artist",
  "album=New Album"
]
- **Then:**
  - All three fields updated in file
  - Metadata sources are UserEdited

#### Test: write_metadata_invalid_field_format
- **Method:** `write_metadata_by_path()`
- **Test ID:** AMD008
- **Priority:** Medium
- **Description:** Error on invalid set format
- **Given:** set=["invalid-format-no-equals"]
- **When:** Calling function
- **Then:**
  - Result.is_err() == true
  - Error about "Unsupported metadata format"

---

### Function: apply_metadata_update

#### Test: apply_update_title
- **Method:** `apply_metadata_update()`
- **Test ID:** AMU001
- **Priority:** Critical
- **Description:** Update title field
- **Given:** metadata with title=None, key="title", value="New Title"
- **When:** Calling `apply_metadata_update(&mut metadata, key, value)`
- **Then:**
  - metadata.title.is_some() == true
  - metadata.title.value == "New Title"
  - metadata.title.source == UserEdited

#### Test: apply_update_artist
- **Method:** `apply_metadata_update()`
- **Test ID:** AMU002
- **Priority:** Critical
- **Description:** Update artist field
- **Given:** key="artist", value="New Artist"
- **When:** Calling function
- **Then:**
  - metadata.artist.value == "New Artist"
  - metadata.artist.source == UserEdited

#### Test: apply_update_album
- **Method:** `apply_metadata_update()`
- **Test ID:** AMU003
- **Priority:** Critical
- **Description:** Update album field
- **Given:** key="album", value="New Album"
- **When:** Calling function
- **Then:**
  - metadata.album.value == "New Album"

#### Test: apply_update_album_artist
- **Method:** `apply_metadata_update()`
- **Test ID:** AMU004
- **Priority:** High
- **Description:** Update album_artist field (aliases: albumartist, album_artist)
- **Given:** 
  - Case 1: key="albumartist", value="Band"
  - Case 2: key="album_artist", value="Band"
- **When:** Calling function for both cases
- **Then:**
  - Both update metadata.album_artist
  - metadata.album_artist.value == "Band"

#### Test: apply_update_track_number
- **Method:** `apply_metadata_update()`
- **Test ID:** AMU005
- **Priority:** High
- **Description:** Update track number (numeric)
- **Given:** key="tracknumber", value="5"
- **When:** Calling function
- **Then:**
  - metadata.track_number.value == 5u32
  - metadata.track_number.source == UserEdited

#### Test: apply_update_invalid_track_number
- **Method:** `apply_metadata_update()`
- **Test ID:** AMU006
- **Priority:** High
- **Description:** Error on non-numeric track number
- **Given:** key="tracknumber", value="abc"
- **When:** Calling function
- **Then:**
  - Result.is_err() == true
  - Error message contains "Invalid track number"

#### Test: apply_update_year
- **Method:** `apply_metadata_update()`
- **Test ID:** AMU007
- **Priority:** High
- **Description:** Update year (numeric)
- **Given:** key="year", value="2024"
- **When:** Calling function
- **Then:**
  - metadata.year.value == 2024u32

#### Test: apply_update_invalid_year
- **Method:** `apply_metadata_update()`
- **Test ID:** AMU008
- **Priority:** High
- **Description:** Error on non-numeric year
- **Given:** key="year", value="not-a-year"
- **When:** Calling function
- **Then:**
  - Result.is_err() == true
  - Error message contains "Invalid year"

#### Test: apply_update_genre
- **Method:** `apply_metadata_update()`
- **Test ID:** AMU009
- **Priority:** High
- **Description:** Update genre field
- **Given:** key="genre", value="Rock"
- **When:** Calling function
- **Then:**
  - metadata.genre.value == "Rock"

#### Test: apply_update_unsupported_field
- **Method:** `apply_metadata_update()`
- **Test ID:** AMU010
- **Priority:** Medium
- **Description:** Error on unknown field
- **Given:** key="unsupported_field", value="value"
- **When:** Calling function
- **Then:**
  - Result.is_err() == true
  - Error message contains "Unsupported metadata field"

#### Test: apply_update_case_insensitive_keys
- **Method:** `apply_metadata_update()`
- **Test ID:** AMU011
- **Priority:** Medium
- **Description:** Keys are case-insensitive
- **Given:**
  - Case 1: key="TITLE", value="Upper"
  - Case 2: key="Title", value="Mixed"
  - Case 3: key="title", value="Lower"
- **When:** Calling function for each case
- **Then:**
  - All three update metadata.title

---

## Module: core::services::format_tree

### Function: format_tree_output

#### Test: format_tree_empty_directory
- **Method:** `format_tree_output()`
- **Test ID:** FTO001
- **Priority:** High
- **Description:** Format empty directory tree
- **Given:** Empty temp directory
- **When:** Calling `format_tree_output(&path)`
- **Then:**
  - Result contains "📁" (folder icon)
  - Result contains "Files: 0"
  - Result contains "Tracks: 0"

#### Test: format_tree_single_file
- **Method:** `format_tree_output()`
- **Test ID:** FTO002
- **Priority:** Critical
- **Description:** Format tree with single file
- **Given:** 
  ```
  temp/
    └── song.flac
  ```
- **When:** Calling `format_tree_output(&temp)`
- **Then:**
  - Result contains "📁 temp"
  - Result contains "🎵" (music icon)
  - Result contains "song.flac"
  - Result contains format info like "[FLAC]"

#### Test: format_tree_nested_structure
- **Method:** `format_tree_output()`
- **Test ID:** FTO003
- **Priority:** Critical
- **Description:** Format deeply nested tree
- **Given:**
  ```
  base/
    ├── Artist A/
    │   └── Album 1/
    │       └── track1.flac
    └── Artist B/
        └── Album 2/
            └── track2.flac
  ```
- **When:** Calling `format_tree_output(&base)`
- **Then:**
  - Result shows tree hierarchy with proper indentation
  - Contains both artists and albums
  - Contains track icons

#### Test: format_tree_source_icons
- **Method:** `format_tree_output()`
- **Test ID:** FTO004
- **Priority:** High
- **Description:** Display correct source icons
- **Given:** Tracks with different metadata sources:
  - CUE-inferred: 📄
  - Embedded: 🎯
  - Folder-inferred: 🤖
  - User-edited: 👤
- **When:** Calling `format_tree_output(&path)`
- **Then:**
  - Each track displays correct icon
  - Icons are in correct format `[icon]`

#### Test: format_tree_summary_counts
- **Method:** `format_tree_output()`
- **Test ID:** FTO005
- **Priority:** Medium
- **Description:** Verify summary statistics
- **Given:** Directory with:
  - 3 files
  - In 2 subdirectories
- **When:** Calling `format_tree_output(&path)`
- **Then:**
  - Result contains "Files: 3"
  - Result contains "Tracks: 3"
  - Result contains "Folders: 2"

---

### Function: emit_by_path

#### Test: emit_by_path_json
- **Method:** `emit_by_path()`
- **Test ID:** EBP001
- **Priority:** Critical
- **Description:** Generate JSON output
- **Given:** Directory with FLAC files
- **When:** Calling `emit_by_path(&path, true)` (json=true)
- **Then:**
  - Result.is_ok() == true
  - Output is valid JSON
  - JSON contains Library structure with schema_version
  - Contains artists, albums, tracks

#### Test: emit_by_path_text
- **Method:** `emit_by_path()`
- **Test ID:** EBP002
- **Priority:** Critical
- **Description:** Generate human-readable text output
- **Given:** Directory with FLAC files
- **When:** Calling `emit_by_path(&path, false)` (json=false)
- **Then:**
  - Result.is_ok() == true
  - Output contains "=== MUSIC LIBRARY METADATA ==="
  - Output contains "ARTIST:"
  - Output contains "ALBUM:"
  - Output contains "TRACK:"

#### Test: emit_by_path_empty
- **Method:** `emit_by_path()`
- **Test ID:** EBP003
- **Priority:** Medium
- **Description:** Handle empty directory
- **Given:** Empty directory
- **When:** Calling `emit_by_path(&path, true)`
- **Then:**
  - Result.is_err() == true OR returns empty structure
  - Error mentions "No music files"

---

## Module: core::services::cue

### Function: generate_cue_content

#### Test: generate_cue_basic
- **Method:** `generate_cue_content()`
- **Test ID:** CUE001
- **Priority:** Critical
- **Description:** Generate basic CUE content
- **Given:** AlbumNode with:
  - title: "Test Album"
  - year: Some(2024)
  - 2 tracks with title, artist, format
- **When:** Calling `generate_cue_content(&album)`
- **Then:**
  - Result contains `PERFORMER "Artist"`
  - Result contains `TITLE "Test Album"`
  - Result contains `REM DATE 2024`
  - Result contains `TRACK 01 AUDIO`
  - Result contains `TRACK 02 AUDIO`
  - Result contains `FILE "track1.flac" WAVE`

#### Test: generate_cue_no_year
- **Method:** `generate_cue_content()`
- **Test ID:** CUE002
- **Priority:** High
- **Description:** Generate CUE without year
- **Given:** AlbumNode with year=None
- **When:** Calling function
- **Then:**
  - Result does NOT contain `REM DATE`
  - Other fields present

#### Test: generate_cue_no_genre
- **Method:** `generate_cue_content()`
- **Test ID:** CUE003
- **Priority:** High
- **Description:** Generate CUE without genre
- **Given:** AlbumNode with tracks having no genre
- **When:** Calling function
- **Then:**
  - Result does NOT contain `REM GENRE`

#### Test: generate_cue_single_file_multiple_tracks
- **Method:** `generate_cue_content()`
- **Test ID:** CUE004
- **Priority:** High
- **Description:** Handle single audio file with multiple tracks
- **Given:** 3 tracks all with file_path="album.flac"
- **When:** Calling function
- **Then:**
  - Result contains only ONE `FILE "album.flac" WAVE` line
  - Contains 3 `TRACK XX AUDIO` lines
  - Track timing indices increment (00:00:00, 00:02:00, 00:04:00)

#### Test: generate_cue_multiple_files
- **Method:** `generate_cue_content()`
- **Test ID:** CUE005
- **Priority:** High
- **Description:** Handle multiple audio files
- **Given:** 3 tracks with different file paths
- **When:** Calling function
- **Then:**
  - Result contains 3 `FILE "XX.flac" WAVE` lines
  - Each FILE has one TRACK

#### Test: generate_cue_title_case_normalization
- **Method:** `generate_cue_content()`
- **Test ID:** CUE006
- **Priority:** Medium
- **Description:** Normalize text fields to title case
- **Given:** Album with title="UPPERCASE ALBUM", artist="LOWERCASE ARTIST"
- **When:** Calling function
- **Then:**
  - Result contains `TITLE "Uppercase Album"`
  - Result contains `PERFORMER "Lowercase Artist"`

---

### Function: parse_cue_file

#### Test: parse_cue_basic
- **Method:** `parse_cue_file()`
- **Test ID:** CUEP001
- **Priority:** Critical
- **Description:** Parse well-formed CUE file
- **Given:** CUE file content:
  ```
  PERFORMER "Artist"
  TITLE "Album"
  FILE "track.flac" WAVE
    TRACK 01 AUDIO
      TITLE "Song"
      INDEX 01 00:00:00
  ```
- **When:** Calling `parse_cue_file(&path)`
- **Then:**
  - Result.is_ok() == true
  - cue_file.performer == Some("Artist")
  - cue_file.title == Some("Album")
  - cue_file.files == vec!["track.flac"]
  - cue_file.tracks.len() == 1
  - cue_file.tracks[0].number == 1
  - cue_file.tracks[0].title == Some("Song")

#### Test: parse_cue_multiple_tracks
- **Method:** `parse_cue_file()`
- **Test ID:** CUEP002
- **Priority:** High
- **Description:** Parse CUE with multiple tracks
- **Given:** CUE file with 3 tracks
- **When:** Calling function
- **Then:**
  - cue_file.tracks.len() == 3
  - Each track has correct number
  - Tracks are in correct order

#### Test: parse_cue_with_genre_and_date
- **Method:** `parse_cue_file()`
- **Test ID:** CUEP003
- **Priority:** High
- **Description:** Parse REM GENRE and REM DATE fields
- **Given:** CUE with:
  ```
  REM GENRE Rock
  REM DATE 2024
  ```
- **When:** Calling function
- **Then:**
  - cue_file.genre == Some("Rock")
  - cue_file.date == Some("2024")

#### Test: parse_cue_nonexistent_file
- **Method:** `parse_cue_file()`
- **Test ID:** CUEP004
- **Priority:** High
- **Description:** Error on nonexistent file
- **Given:** Path to nonexistent CUE file
- **When:** Calling function
- **Then:**
  - Result.is_err() == true
  - Error message contains "Failed to read"

#### Test: parse_cue_malformed
- **Method:** `parse_cue_file()`
- **Test ID:** CUEP005
- **Priority:** Medium
- **Description:** Error on malformed CUE
- **Given:** CUE file with invalid syntax:
  ```
  INVALID LINE
  PERFORMER Missing Quote
  ```
- **When:** Calling function
- **Then:**
  - Result.is_err() == true
  - Error message about malformed line

---

### Function: validate_cue_consistency

#### Test: validate_cue_valid
- **Method:** `validate_cue_consistency()`
- **Test ID:** CUEV001
- **Priority:** Critical
- **Description:** Validate correct CUE against matching audio files
- **Given:**
  - Valid CUE file referencing "track1.flac", "track2.flac"
  - Both audio files exist
- **When:** Calling `validate_cue_consistency(&cue_path, &[&track1, &track2])`
- **Then:**
  - result.is_valid == true
  - result.parsing_error == false
  - result.file_missing == false
  - result.track_count_mismatch == false

#### Test: validate_cue_missing_file
- **Method:** `validate_cue_consistency()`
- **Test ID:** CUEV002
- **Priority:** Critical
- **Description:** Detect missing referenced audio file
- **Given:**
  - CUE referencing "missing.flac"
  - Only "existing.flac" provided
- **When:** Calling function
- **Then:**
  - result.is_valid == false
  - result.file_missing == true

#### Test: validate_cue_track_count_mismatch
- **Method:** `validate_cue_consistency()`
- **Test ID:** CUEV003
- **Priority:** High
- **Description:** Detect track/file count mismatch
- **Given:**
  - CUE referencing 2 files
  - 3 audio files provided
- **When:** Calling function
- **Then:**
  - result.is_valid == false
  - result.track_count_mismatch == true

#### Test: validate_cue_parsing_error
- **Method:** `validate_cue_consistency()`
- **Test ID:** CUEV004
- **Priority:** High
- **Description:** Handle unparsable CUE file
- **Given:** Invalid CUE file content
- **When:** Calling function
- **Then:**
  - result.is_valid == false
  - result.parsing_error == true

---

## Module: core::errors

### Enum: MusicChoreError

#### Test: error_display_io
- **Method:** `MusicChoreError::fmt()` (Display)
- **Test ID:** ERR001
- **Priority:** Medium
- **Description:** Format IoError variant
- **Given:** `MusicChoreError::IoError("disk full".to_string())`
- **When:** Calling `format!("{}", error)`
- **Then:**
  - Result == "I/O error: disk full"

#### Test: error_display_file_not_found
- **Method:** `MusicChoreError::fmt()`
- **Test ID:** ERR002
- **Priority:** Medium
- **Description:** Format FileNotFound variant
- **Given:** `MusicChoreError::FileNotFound("/music/song.flac".to_string())`
- **When:** Calling format
- **Then:**
  - Result == "File not found: /music/song.flac"

#### Test: error_from_io_error
- **Method:** `From<std::io::Error>`
- **Test ID:** ERR003
- **Priority:** High
- **Description:** Convert std::io::Error to MusicChoreError
- **Given:** `std::io::Error::new(NotFound, "file missing")`
- **When:** Converting: `let err: MusicChoreError = io_err.into()`
- **Then:**
  - Matches MusicChoreError::IoError(_)
  - Contains "file missing"

#### Test: error_from_json_error
- **Method:** `From<serde_json::Error>`
- **Test ID:** ERR004
- **Priority:** Medium
- **Description:** Convert JSON parse error
- **Given:** Invalid JSON string parsed with serde_json
- **When:** Converting the error
- **Then:**
  - Results in MusicChoreError::Other containing "JSON error"

---

## Summary

### Test Count by Module

| Module | Test Count | Priority |
|--------|-----------|----------|
| models::MetadataValue | 7 | High |
| models::Track | 9 | Critical |
| models::Library | 8 | Critical |
| traits::AudioFileRegistry | 11 | Critical |
| scanner | 25 | Critical |
| inference | 18 | High |
| normalization | 12 | High |
| library | 8 | Critical |
| validation | 16 | High |
| apply_metadata | 21 | Critical |
| format_tree | 8 | High |
| cue | 15 | High |
| errors | 4 | Medium |
| **Total** | **162** | |

### Implementation Order

**Phase 1 (Critical):** models, Track, Library, traits, apply_metadata - 56 tests
**Phase 2 (High):** scanner, validation, inference, normalization - 71 tests  
**Phase 3 (Complete):** cue, format_tree, errors, edge cases - 35 tests
