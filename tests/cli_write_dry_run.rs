use music_chore::adapters::audio_formats::read_metadata;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_write_dry_run_prevents_file_writes() {
    // Create a temporary directory with a test FLAC file
    let temp_dir = TempDir::new().unwrap();
    let flac_path = temp_dir.path().join("test.flac");

    // Copy an existing test FLAC file to our temp directory
    std::fs::copy("tests/fixtures/flac/simple/track1.flac", &flac_path).unwrap();

    // Get the original title
    let original_title = get_file_title(&flac_path);

    // Run write command with --dry-run
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("write")
        .arg(&flac_path)
        .arg("--set")
        .arg("title=Dry Run Test Title")
        .arg("--dry-run")
        .output()
        .expect("Failed to execute musicctl write command");

    // Check command succeeded
    assert!(output.status.success(), "Command failed: {:?}", output);

    // Verify dry-run output
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("DRY RUN: Would set title = Dry Run Test Title"));
    assert!(stdout.contains("DRY RUN: No changes made"));

    // Verify file was NOT changed by checking that the title is still the same
    let current_title = get_file_title(&flac_path);
    assert_eq!(
        original_title, current_title,
        "File was modified during dry run"
    );
}

#[test]
fn test_write_apply_modifies_file() {
    // Create a temporary directory with a test FLAC file
    let temp_dir = TempDir::new().unwrap();
    let flac_path = temp_dir.path().join("test.flac");

    // Copy an existing test FLAC file to our temp directory
    std::fs::copy("tests/fixtures/flac/simple/track1.flac", &flac_path).unwrap();

    // Get the original title
    let original_title = get_file_title(&flac_path);

    // Run write command with --apply
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("write")
        .arg(&flac_path)
        .arg("--set")
        .arg("title=Applied Test Title")
        .arg("--apply")
        .output()
        .expect("Failed to execute musicctl write command");

    // Check command succeeded
    assert!(output.status.success(), "Command failed: {:?}", output);

    // Verify apply output
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Successfully updated metadata"));

    // Verify file WAS changed by checking that the title is different
    let current_title = get_file_title(&flac_path);
    assert_ne!(
        original_title, current_title,
        "File was not modified after apply"
    );
    assert_eq!(current_title, "Applied Test Title");
}

#[test]
fn test_write_defaults_to_dry_run_when_no_flags_specified() {
    // Create a temporary directory with a test FLAC file
    let temp_dir = TempDir::new().unwrap();
    let flac_path = temp_dir.path().join("test.flac");

    // Copy an existing test FLAC file to our temp directory
    std::fs::copy("tests/fixtures/flac/simple/track1.flac", &flac_path).unwrap();

    // Get the original title
    let original_title = get_file_title(&flac_path);

    // Run write command without --apply or --dry-run (should default to dry-run)
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("write")
        .arg(&flac_path)
        .arg("--set")
        .arg("title=Test Title")
        .output()
        .expect("Failed to execute musicctl write command");

    // Check command succeeded (it should succeed with dry-run behavior)
    assert!(output.status.success(), "Command failed: {:?}", output);

    // Verify dry-run output is shown in stdout
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("DRY RUN: Would set title = Test Title"));
    assert!(stdout.contains("DRY RUN: No changes made"));

    // Verify file was NOT changed by checking that the title is still the same
    let current_title = get_file_title(&flac_path);
    assert_eq!(
        original_title, current_title,
        "File was modified when it should have been in dry-run mode"
    );
}

#[test]
fn test_write_prevents_both_apply_and_dry_run() {
    // Create a temporary directory with a test FLAC file
    let temp_dir = TempDir::new().unwrap();
    let flac_path = temp_dir.path().join("test.flac");

    // Copy an existing test FLAC file to our temp directory
    std::fs::copy("tests/fixtures/flac/simple/track1.flac", &flac_path).unwrap();

    // Run write command with both --apply and --dry-run
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("write")
        .arg(&flac_path)
        .arg("--set")
        .arg("title=Test Title")
        .arg("--apply")
        .arg("--dry-run")
        .output()
        .expect("Failed to execute musicctl write command");

    // Verify error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Cannot use both --apply and --dry-run flags simultaneously"),
        "Expected error message not found in: {}",
        stderr
    );
}

fn get_file_title(flac_path: &Path) -> String {
    // Read the current metadata
    let track = read_metadata(flac_path).unwrap();

    // Extract the title, return empty string if not present
    track.metadata.title.map(|t| t.value).unwrap_or_default()
}
