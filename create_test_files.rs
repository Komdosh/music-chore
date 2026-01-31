use music_chore::infra::audio;
use std::fs;
use std::path::Path;

// Create a test FLAC file with lowercase title
fn create_lowercase_test_file() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = Path::new("tests/fixtures/normalization/01-lowercase-title.flac");

    if let Some(parent) = test_file.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copy existing metadata file and modify title
    fs::copy(
        "tests/fixtures/flac/metadata/test_with_metadata.flac",
        test_file,
    )?;

    Ok(())
}

// Create a test FLAC file with already title case
fn create_title_case_test_file() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = Path::new("tests/fixtures/normalization/02-title-case.flac");

    if let Some(parent) = test_file.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copy existing metadata file (already has title case)
    fs::copy(
        "tests/fixtures/flac/metadata/test_with_metadata.flac",
        test_file,
    )?;

    Ok(())
}

// Create a test FLAC file with mixed case title
fn create_mixed_case_test_file() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = Path::new("tests/fixtures/normalization/03-mixed-case-title.flac");

    if let Some(parent) = test_file.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copy existing metadata file and modify title
    fs::copy(
        "tests/fixtures/flac/metadata/test_with_metadata.flac",
        test_file,
    )?;

    Ok(())
}

fn main() {
    create_lowercase_test_file().unwrap();
    create_title_case_test_file().unwrap();
    create_mixed_case_test_file().unwrap();

    println!("Test files created in tests/fixtures/normalization/");
    println!("Run: ./target/debug/music-chore normalize --dry-run tests/fixtures/normalization/");
}
