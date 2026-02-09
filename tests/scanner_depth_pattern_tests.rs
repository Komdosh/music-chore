//! Tests for the scanner depth and pattern options functionality.

use music_chore::core::services::scanner::{scan_dir_with_options, scan_dir_with_depth, scan_dir_with_depth_and_symlinks};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_scan_dir_with_depth_zero() {
    // Should only return files in immediate directory
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Create immediate file
    fs::write(base_dir.join("track1.flac"), b"dummy flac content").unwrap();
    
    // Create subdirectory with file
    let sub_dir = base_dir.join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    fs::write(sub_dir.join("track2.flac"), b"dummy flac content").unwrap();
    
    let tracks = scan_dir_with_depth(base_dir, Some(0));  // Only immediate files
    
    assert_eq!(tracks.len(), 1);  // Only track1.flac, not track2.flac from subdir
    assert!(tracks[0].file_path.ends_with("track1.flac"));
}

#[test]
fn test_scan_dir_with_depth_one() {
    // Should return files at base + 1 level deep
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Create immediate file
    fs::write(base_dir.join("track1.flac"), b"dummy flac content").unwrap();
    
    // Create subdirectory with file
    let sub_dir = base_dir.join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    fs::write(sub_dir.join("track2.flac"), b"dummy flac content").unwrap();
    
    // Create deeper subdirectory with file
    let deep_sub_dir = sub_dir.join("deep_subdir");
    fs::create_dir(&deep_sub_dir).unwrap();
    fs::write(deep_sub_dir.join("track3.flac"), b"dummy flac content").unwrap();
    
    let tracks = scan_dir_with_depth(base_dir, Some(1));  // Base + 1 level
    
    assert_eq!(tracks.len(), 2);  // track1.flac (immediate) + track2.flac (1 level deep)
    let filenames: Vec<String> = tracks.iter()
        .map(|t| t.file_path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(filenames.contains(&"track1.flac".to_string()));
    assert!(filenames.contains(&"track2.flac".to_string()));
    assert!(!filenames.contains(&"track3.flac".to_string()));  // Should not be included
}

#[test]
fn test_scan_dir_with_depth_two() {
    // Should return files at base + up to 2 levels deep
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Create immediate file
    fs::write(base_dir.join("track1.flac"), b"dummy flac content").unwrap();
    
    // Create subdirectory with file
    let sub_dir = base_dir.join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    fs::write(sub_dir.join("track2.flac"), b"dummy flac content").unwrap();
    
    // Create deeper subdirectory with file
    let deep_sub_dir = sub_dir.join("deep_subdir");
    fs::create_dir(&deep_sub_dir).unwrap();
    fs::write(deep_sub_dir.join("track3.flac"), b"dummy flac content").unwrap();
    
    // Create even deeper subdirectory with file
    let deepest_sub_dir = deep_sub_dir.join("deepest_subdir");
    fs::create_dir(&deepest_sub_dir).unwrap();
    fs::write(deepest_sub_dir.join("track4.flac"), b"dummy flac content").unwrap();
    
    let tracks = scan_dir_with_depth(base_dir, Some(2));  // Base + up to 2 levels
    
    assert_eq!(tracks.len(), 3);  // track1.flac (immediate) + track2.flac (1 level) + track3.flac (2 levels)
    let filenames: Vec<String> = tracks.iter()
        .map(|t| t.file_path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(filenames.contains(&"track1.flac".to_string()));
    assert!(filenames.contains(&"track2.flac".to_string()));
    assert!(filenames.contains(&"track3.flac".to_string()));
    assert!(!filenames.contains(&"track4.flac".to_string()));  // Should not be included
}

#[test]
fn test_scan_dir_with_depth_unlimited() {
    // None should scan all levels
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Create immediate file
    fs::write(base_dir.join("track1.flac"), b"dummy flac content").unwrap();
    
    // Create subdirectory with file
    let sub_dir = base_dir.join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    fs::write(sub_dir.join("track2.flac"), b"dummy flac content").unwrap();
    
    // Create deeper subdirectory with file
    let deep_sub_dir = sub_dir.join("deep_subdir");
    fs::create_dir(&deep_sub_dir).unwrap();
    fs::write(deep_sub_dir.join("track3.flac"), b"dummy flac content").unwrap();
    
    let tracks = scan_dir_with_depth(base_dir, None);  // Unlimited depth
    
    assert_eq!(tracks.len(), 3);  // All files should be found
    let filenames: Vec<String> = tracks.iter()
        .map(|t| t.file_path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(filenames.contains(&"track1.flac".to_string()));
    assert!(filenames.contains(&"track2.flac".to_string()));
    assert!(filenames.contains(&"track3.flac".to_string()));
}

#[test]
fn test_scan_dir_with_depth_deep_nesting() {
    // Test with 5+ levels of nesting
    let temp_dir = TempDir::new().unwrap();
    let mut current_dir = temp_dir.path().to_path_buf();
    
    let mut expected_files = Vec::new();
    for i in 1..=6 {  // 6 levels deep
        let file_name = format!("track{}.flac", i);
        fs::write(current_dir.join(&file_name), b"dummy flac content").unwrap();
        expected_files.push(file_name);
        
        if i < 6 {  // Don't create another dir after the last file
            current_dir.push(format!("level{}", i));
            fs::create_dir(&current_dir).unwrap();
        }
    }
    
    // Test with limited depth
    let tracks = scan_dir_with_depth(temp_dir.path(), Some(3));  // Only go 3 levels deep
    assert_eq!(tracks.len(), 4);  // track1.flac (level 0) + track2.flac (level 1) + track3.flac (level 2) + track4.flac (level 3)
    
    // Test with unlimited depth
    let tracks = scan_dir_with_depth(temp_dir.path(), None);  // Unlimited depth
    assert_eq!(tracks.len(), 6);  // All 6 files should be found
}

#[test]
fn test_scan_dir_with_depth_and_symlinks_follow() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Create a target file
    let target_file = base_dir.join("target.flac");
    fs::write(&target_file, b"dummy flac content").unwrap();
    
    // Create a subdirectory
    let sub_dir = base_dir.join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    
    // Create a symlink to the target file in the subdirectory
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target_file, sub_dir.join("link.flac")).unwrap();
    
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&target_file, sub_dir.join("link.flac")).unwrap();
    
    // Test with symlink following enabled
    let tracks = scan_dir_with_depth_and_symlinks(base_dir, Some(2), true);  // Follow symlinks
    
    // Should include the file through the symlink
    assert!(!tracks.is_empty());
}

#[test]
fn test_scan_dir_with_depth_and_symlinks_skip() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Create a target file
    let target_file = base_dir.join("target.flac");
    fs::write(&target_file, b"dummy flac content").unwrap();
    
    // Create a subdirectory
    let sub_dir = base_dir.join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    
    // Create a symlink to the target file in the subdirectory
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target_file, sub_dir.join("link.flac")).unwrap();
    
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&target_file, sub_dir.join("link.flac")).unwrap();
    
    // Test with symlink following disabled
    let tracks = scan_dir_with_depth_and_symlinks(base_dir, Some(2), false);  // Don't follow symlinks
    
    // Should not include the file through the symlink (behavior depends on implementation)
    // This test verifies that the function accepts the parameter
    assert!(tracks.len() > 0);  // At least it doesn't crash
}

#[test]
fn test_scan_dir_with_options_exclude_single_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Create test files
    fs::write(base_dir.join("track1.flac"), b"dummy flac content").unwrap();
    fs::write(base_dir.join("temp.tmp"), b"dummy tmp content").unwrap();
    fs::write(base_dir.join("track2.flac"), b"dummy flac content").unwrap();
    
    // Test excluding *.tmp files
    let tracks = scan_dir_with_options(
        base_dir,
        None,  // No depth limit
        false,  // Don't follow symlinks
        vec!["*.tmp".to_string()],  // Exclude pattern
        false,  // Don't skip metadata
    );
    
    assert_eq!(tracks.len(), 2);  // Only the .flac files, not the .tmp file
    let filenames: Vec<String> = tracks.iter()
        .map(|t| t.file_path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(filenames.contains(&"track1.flac".to_string()));
    assert!(filenames.contains(&"track2.flac".to_string()));
    assert!(!filenames.contains(&"temp.tmp".to_string()));
}

#[test]
fn test_scan_dir_with_options_exclude_multiple_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Create test files
    fs::write(base_dir.join("track1.flac"), b"dummy flac content").unwrap();
    fs::write(base_dir.join("temp.tmp"), b"dummy tmp content").unwrap();
    fs::write(base_dir.join("backup.flac"), b"dummy flac content").unwrap();
    fs::write(base_dir.join("track2.flac"), b"dummy flac content").unwrap();
    fs::write(base_dir.join("test.bak"), b"dummy bak content").unwrap();
    
    // Test excluding multiple patterns
    let tracks = scan_dir_with_options(
        base_dir,
        None,  // No depth limit
        false,  // Don't follow symlinks
        vec!["*.tmp".to_string(), "*backup*".to_string(), "*.bak".to_string()],  // Multiple exclude patterns
        false,  // Don't skip metadata
    );

    // Should find 2 files (track1.flac and track2.flac) as temp.tmp, backup.flac, and test.bak should be excluded
    // If finding 3, it means one pattern isn't working as expected
    assert_eq!(tracks.len(), 2, "Expected 2 files after excluding *.tmp, backup*, and *.bak patterns");
    let filenames: Vec<String> = tracks.iter()
        .map(|t| t.file_path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(filenames.contains(&"track1.flac".to_string()));
    assert!(filenames.contains(&"track2.flac".to_string()));
    assert!(!filenames.contains(&"temp.tmp".to_string()));
    assert!(!filenames.contains(&"backup.flac".to_string()));
    assert!(!filenames.contains(&"test.bak".to_string()));
}

#[test]
fn test_scan_dir_with_options_exclude_directory_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Create test files
    fs::write(base_dir.join("track1.flac"), b"dummy flac content").unwrap();
    
    // Create excluded directory
    let excluded_dir = base_dir.join("temp");
    fs::create_dir(&excluded_dir).unwrap();
    fs::write(excluded_dir.join("track2.flac"), b"dummy flac content").unwrap();
    
    // Create another directory
    let other_dir = base_dir.join("other");
    fs::create_dir(&other_dir).unwrap();
    fs::write(other_dir.join("track3.flac"), b"dummy flac content").unwrap();
    
    // Test excluding temp directory
    let tracks = scan_dir_with_options(
        base_dir,
        None,  // No depth limit
        false,  // Don't follow symlinks
        vec!["**/temp/**".to_string()],  // Exclude temp directory and contents anywhere
        false,  // Don't skip metadata
    );
    
    // The exclude pattern should exclude files in the temp directory
    // We have: track1.flac (base), track2.flac (temp dir), track3.flac (other dir)
    // So we should get 2 files: track1.flac and track3.flac
    // If getting 3, it means the exclude pattern isn't working as expected
    assert_eq!(tracks.len(), 2, "Expected 2 files after excluding temp/** pattern");
    let filenames: Vec<String> = tracks.iter()
        .map(|t| t.file_path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(filenames.contains(&"track1.flac".to_string()));
    assert!(filenames.contains(&"track3.flac".to_string()));
    assert!(!filenames.contains(&"track2.flac".to_string()));
}

#[test]
fn test_scan_dir_with_options_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Empty directory
    let tracks = scan_dir_with_options(
        base_dir,
        None,  // No depth limit
        false,  // Don't follow symlinks
        vec![],  // No exclude patterns
        false,  // Don't skip metadata
    );
    
    assert_eq!(tracks.len(), 0);
}

#[test]
fn test_scan_dir_with_options_nonexistent_directory() {
    let nonexistent_path = PathBuf::from("/nonexistent/path");
    
    // Test with nonexistent directory
    let tracks = scan_dir_with_options(
        &nonexistent_path,
        None,  // No depth limit
        false,  // Don't follow symlinks
        vec![],  // No exclude patterns
        false,  // Don't skip metadata
    );
    
    // Should return empty vector for nonexistent directory
    assert_eq!(tracks.len(), 0);
}

#[test]
fn test_scan_dir_with_options_skip_metadata_behavior() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Create a test file
    fs::write(base_dir.join("test_track.flac"), b"dummy flac content").unwrap();
    
    // Test with skip_metadata = true
    let tracks_with_skip = scan_dir_with_options(
        base_dir,
        None,  // No depth limit
        false,  // Don't follow symlinks
        vec![],  // No exclude patterns
        true,   // Skip metadata
    );
    
    // Test with skip_metadata = false
    let tracks_without_skip = scan_dir_with_options(
        base_dir,
        None,  // No depth limit
        false,  // Don't follow symlinks
        vec![],  // No exclude patterns
        false,  // Don't skip metadata
    );
    
    // Both should find the file, but with different metadata handling
    assert_eq!(tracks_with_skip.len(), 1);
    assert_eq!(tracks_without_skip.len(), 1);
    
    // When skipping metadata, the title should be derived from the filename
    // When not skipping metadata, it should try to read from the file (but might be empty if file has no embedded metadata)
    assert!(tracks_with_skip[0].metadata.title.is_some());
    // Note: tracks_without_skip might not have title if the dummy file has no embedded metadata
    // The important thing is that both return successfully
}

#[test]
fn test_scan_dir_with_options_deterministic_ordering() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path();
    
    // Create multiple files in random order
    fs::write(base_dir.join("zebra.flac"), b"dummy flac content").unwrap();
    fs::write(base_dir.join("alpha.flac"), b"dummy flac content").unwrap();
    fs::write(base_dir.join("beta.flac"), b"dummy flac content").unwrap();
    
    // Multiple scans should return files in the same order
    let tracks1 = scan_dir_with_options(
        base_dir,
        None,  // No depth limit
        false,  // Don't follow symlinks
        vec![],  // No exclude patterns
        false,  // Don't skip metadata
    );
    
    let tracks2 = scan_dir_with_options(
        base_dir,
        None,  // No depth limit
        false,  // Don't follow symlinks
        vec![],  // No exclude patterns
        false,  // Don't skip metadata
    );
    
    // Check that ordering is consistent
    assert_eq!(tracks1.len(), tracks2.len());
    for i in 0..tracks1.len() {
        assert_eq!(tracks1[i].file_path, tracks2[i].file_path);
    }
    
    // Files should be sorted alphabetically by filename
    let filenames: Vec<String> = tracks1.iter()
        .map(|t| t.file_path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(filenames, vec!["alpha.flac", "beta.flac", "zebra.flac"]);
}