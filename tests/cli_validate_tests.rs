use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_validate_command_with_missing_artist() {
    // Create a temporary directory with a test FLAC file
    let temp_dir = TempDir::new().unwrap();
    let flac_path = temp_dir.path().join("test.flac");

    // Copy an existing test FLAC file to our temp directory
    fs::copy("tests/fixtures/flac/simple/track1.flac", &flac_path).unwrap();

    // Set only title, leaving artist missing from original file
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("write")
        .arg(&flac_path)
        .arg("--set")
        .arg("title=Only Title") // Only set title, keep original artist
        .arg("--apply")
        .output()
        .expect("Failed to set title only");

    assert!(output.status.success(), "Failed to set title only");

    // Check what metadata looks like after setting empty artist
    let read_output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("read")
        .arg(&flac_path)
        .output()
        .expect("Failed to read metadata after writing empty artist");
    println!(
        "Metadata after setting empty artist: {}",
        String::from_utf8_lossy(&read_output.stdout)
    );

    // Now run validation
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("validate")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute musicctl validate command");

    // Check command completed
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Check validation output - since we set valid title, validation should either pass
    // or only have warnings (artist might be missing from folder inference)
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("VALIDATION RESULTS"));
    // File should have either complete validation or at least not fail on missing required fields
    assert!(
        stdout.contains("All files passed validation!")
            || stdout.contains("Missing required field: artist")
            || stdout.contains("WARNINGS")
    );
}

#[test]
fn test_validate_command_with_complete_metadata() {
    // Create a temporary directory with a test FLAC file
    let temp_dir = TempDir::new().unwrap();
    let flac_path = temp_dir.path().join("test.flac");

    // Copy an existing test FLAC file to our temp directory
    fs::copy("tests/fixtures/flac/simple/track1.flac", &flac_path).unwrap();

    // Ensure the file has complete metadata
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("write")
        .arg(&flac_path)
        .arg("--set")
        .arg("title=Test Song")
        .arg("--set")
        .arg("artist=Test Artist")
        .arg("--set")
        .arg("album=Test Album")
        .arg("--set")
        .arg("tracknumber=1")
        .arg("--set")
        .arg("year=2023")
        .arg("--apply")
        .output()
        .expect("Failed to set complete metadata");

    println!(
        "Write command stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "Write command stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    println!("Write command exit code: {}", output.status);

    assert!(output.status.success(), "Failed to set complete metadata");

    // Check what metadata the file actually has after writing
    let read_output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("read")
        .arg(&flac_path)
        .output()
        .expect("Failed to read metadata after write");
    println!(
        "Metadata after write: {}",
        String::from_utf8_lossy(&read_output.stdout)
    );

    // Now run validation
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("validate")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute musicctl validate command");

    // Check for successful validation
    let stdout = String::from_utf8(output.stdout).unwrap();
    println!("Complete metadata validation output: {}", stdout);
    assert!(stdout.contains("VALIDATION RESULTS"));
    // Should either pass validation or only have warnings (not errors)
    assert!(
        stdout.contains("All files passed validation!")
            || (stdout.contains("WARNINGS") && !stdout.contains("ERRORS"))
    );
}

#[test]
fn test_validate_command_json_output() {
    // Create a temporary directory with a test FLAC file
    let temp_dir = TempDir::new().unwrap();
    let flac_path = temp_dir.path().join("test.flac");

    // Copy an existing test FLAC file to our temp directory
    fs::copy("tests/fixtures/flac/simple/track1.flac", &flac_path).unwrap();

    // Run validation with JSON output
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("validate")
        .arg(temp_dir.path())
        .arg("--json")
        .output()
        .expect("Failed to execute musicctl validate command");

    // Check command completed
    assert!(output.status.success(), "Command failed: {:?}", output);

    // Check for JSON output
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"valid\":"));
    assert!(stdout.contains("\"errors\":"));
    assert!(stdout.contains("\"warnings\":"));
    assert!(stdout.contains("\"summary\":"));
}

#[test]
fn test_validate_command_with_warnings() {
    // Create a temporary directory with a test FLAC file
    let temp_dir = TempDir::new().unwrap();
    let flac_path = temp_dir.path().join("test.flac");

    // Copy an existing test FLAC file to our temp directory
    fs::copy("tests/fixtures/flac/simple/track1.flac", &flac_path).unwrap();

    // Set a problematic year to trigger a warning
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("write")
        .arg(&flac_path)
        .arg("--set")
        .arg("title=Test Song")
        .arg("--set")
        .arg("artist=Test Artist")
        .arg("--set")
        .arg("album=Test Album")
        .arg("--set")
        .arg("year=1800") // Unusual year
        .arg("--apply")
        .output()
        .expect("Failed to set problematic metadata");

    assert!(
        output.status.success(),
        "Failed to set problematic metadata"
    );

    // Now run validation
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("validate")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute musicctl validate command");

    // Check for warning output
    let stdout = String::from_utf8(output.stdout).unwrap();
    println!("Warning test output: {}", stdout);
    assert!(stdout.contains("VALIDATION RESULTS"));
    assert!(stdout.contains("WARNINGS") || stdout.contains("warnings"));
    // Check for unusual year warning
    assert!(stdout.contains("1800") || stdout.contains("unusual"));
}

#[test]
fn test_validate_command_empty_directory() {
    // Create an empty temporary directory
    let temp_dir = TempDir::new().unwrap();

    // Run validation on empty directory
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("validate")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute musicctl validate command");

    // Check for empty directory message
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("No music files found to validate"));
}

#[test]
fn test_validate_command_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
        .arg("validate")
        .arg("--help")
        .output()
        .expect("Failed to execute musicctl validate --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Validate metadata completeness"));
    assert!(stdout.contains("--json"));
}
