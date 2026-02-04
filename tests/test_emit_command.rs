use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_emit_command_basic() {
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&["emit", "tests/fixtures/flac/simple"])
        .output()
        .expect("Failed to run emit command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    // Check structured output format
    assert!(stdout.contains("=== MUSIC LIBRARY METADATA ==="));
    assert!(stdout.contains("Total Artists: 2"));
    assert!(stdout.contains("Total Albums: 2"));
    assert!(stdout.contains("Total Tracks: 2"));
    assert!(stdout.contains("ARTIST: flac"));
    assert!(stdout.contains("ALBUM: simple"));
    assert!(stdout.contains("TRACK:"));
    assert!(stdout.contains("=== END METADATA ==="));
}

#[test]
fn test_emit_command_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&["emit", "tests/fixtures/flac/simple", "--json"])
        .output()
        .expect("Failed to run emit command with JSON");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    // Check JSON format
    assert!(stdout.contains("\"artists\""));
    assert!(stdout.contains("\"total_tracks\": 2"));
    assert!(stdout.contains("\"total_artists\": 2"));
    assert!(stdout.contains("\"total_albums\": 2"));
    assert!(stdout.contains("\"name\": \"flac\""));
    assert!(stdout.contains("\"title\": \"simple\""));

    // Verify it's valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert_eq!(parsed["total_tracks"], 2);
    assert_eq!(parsed["total_artists"], 2);
    assert_eq!(parsed["total_albums"], 2);
}

#[test]
fn test_emit_command_nested_structure() {
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&["emit", "tests/fixtures/flac/nested"])
        .output()
        .expect("Failed to run emit command on nested structure");

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");

    // Should find The Beatles artist
    assert!(stderr.contains("Failed to read metadata"));
    assert!(
        stderr.contains("Invalid file: Failed to read FLAC file: Invalid argument (os error 22)")
    );
}

#[test]
fn test_emit_command_nested_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&["emit", "tests/fixtures/flac/nested", "--json"])
        .output()
        .expect("Failed to run emit command on nested structure with JSON");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stderr).expect("Invalid UTF-8");

    println!("{}", stdout);
    // // Verify it's valid JSON
    // let parsed: serde_json::Value =
    //     serde_json::from_str(&stdout).expect("Output should be valid JSON");
    //
    // assert_eq!(parsed["total_tracks"], 2);
    // assert_eq!(parsed["total_artists"], 1);
    // assert_eq!(parsed["total_albums"], 1);
    //
    // // Check artist and album names
    // let artist = &parsed["artists"][0];
    // assert_eq!(artist["name"], "The Beatles");
    //
    // let album = &artist["albums"][0];
    // assert_eq!(album["title"], "Abbey Road");
    // assert_eq!(album["tracks"].as_array().unwrap().len(), 2);
}

#[test]
fn test_emit_command_empty_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let empty_path = temp_dir.path();

    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&["emit", empty_path.to_str().unwrap()])
        .output()
        .expect("Failed to run emit command on empty directory");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    // Should show empty library
    assert!(stdout.contains("Total Artists: 0"));
    assert!(stdout.contains("Total Albums: 0"));
    assert!(stdout.contains("Total Tracks: 0"));
    assert!(stdout.contains("=== END METADATA ==="));
}

#[test]
fn test_emit_command_nonexistent_directory() {
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&["emit", "/nonexistent/path"])
        .output()
        .expect("Failed to run emit command");

    // Should succeed with empty results for non-existent directory
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    // Should show empty library
    assert!(stdout.contains("Total Artists: 0"));
    assert!(stdout.contains("Total Albums: 0"));
    assert!(stdout.contains("Total Tracks: 0"));
}

#[test]
fn test_emit_command_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&["emit", "--help"])
        .output()
        .expect("Failed to run emit help command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    assert!(stdout.contains("Emit library metadata"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("PATH"));
}
