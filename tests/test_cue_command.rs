use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_cue_command_basic() {
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&[
            "cue",
            "--generate",
            "tests/fixtures/flac/simple",
            "--dry-run",
        ])
        .output()
        .expect("Failed to run cue command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    assert!(stdout.contains("PERFORMER"));
    assert!(stdout.contains("TITLE"));
    assert!(stdout.contains("TRACK"));
    assert!(stdout.contains("Would write to:"));
    assert!(stdout.contains(".cue"));
}

#[test]
fn test_cue_command_writes_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cue_path = temp_dir.path().join("test.cue");

    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&[
            "cue",
            "--generate",
            "tests/fixtures/flac/simple",
            cue_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run cue command");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    assert!(stdout.contains("written to:"));
    assert!(cue_path.exists(), "Cue file should be created");

    let content = fs::read_to_string(&cue_path).expect("Failed to read cue file");
    assert!(content.contains("PERFORMER"));
    assert!(content.contains("TITLE"));
    assert!(content.contains("TRACK 01"));
}

#[test]
fn test_cue_command_exists_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cue_path = temp_dir.path().join("album.cue");

    fs::write(&cue_path, "existing content").expect("Failed to create existing cue file");

    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&[
            "cue",
            "--generate",
            "tests/fixtures/flac/simple",
            cue_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run cue command");

    assert!(
        !output.status.success(),
        "Command should fail when cue file exists"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists") || stderr.contains("exists"));
}

#[test]
fn test_cue_command_force_overwrite() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cue_path = temp_dir.path().join("album.cue");

    fs::write(&cue_path, "original content").expect("Failed to create cue file");

    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&[
            "cue",
            "--generate",
            "tests/fixtures/flac/simple",
            cue_path.to_str().unwrap(),
            "--force",
        ])
        .output()
        .expect("Failed to run cue command");

    assert!(
        output.status.success(),
        "Command should succeed with --force"
    );

    let content = fs::read_to_string(&cue_path).expect("Failed to read cue file");
    assert!(content.contains("PERFORMER"));
    assert!(!content.contains("original content"));
}

#[test]
fn test_cue_command_output_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cue_path = temp_dir.path().join("custom-name.cue");

    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&[
            "cue",
            "--generate",
            "tests/fixtures/flac/simple",
            cue_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run cue command");

    assert!(output.status.success());
    assert!(cue_path.exists());

    let content = fs::read_to_string(&cue_path).expect("Failed to read cue file");
    assert!(content.contains("PERFORMER"));
    assert!(content.contains("TITLE"));
}

#[test]
fn test_cue_command_empty_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&["cue", "--generate", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to run cue command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No music files found"));
}

#[test]
fn test_cue_content_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .args(&[
            "cue",
            "--generate",
            "tests/fixtures/flac/simple",
            "--dry-run",
        ])
        .output()
        .expect("Failed to run cue command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    assert!(stdout.contains("PERFORMER"));
    assert!(stdout.contains("TITLE"));
    assert!(stdout.contains("REM GENRE"));
    assert!(stdout.contains("FILE"));
    assert!(stdout.contains("TRACK"));
    assert!(stdout.contains("INDEX 01"));
}
