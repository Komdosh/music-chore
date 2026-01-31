use music_chore::infra::audio;
use std::fs;
use std::path::Path;

fn main() {
    // Create test directory
    fs::create_dir_all("tests/fixtures/normalization/simple").unwrap();

    // Create a simple test file with lowercase title
    let source = Path::new("tests/fixtures/flac/metadata/test_with_metadata.flac");
    let dest = Path::new("tests/fixtures/normalization/simple/01-lowercase.flac");
    fs::copy(source, dest).unwrap();

    println!("Test file created");
    println!("Now test with: ./target/debug/music-chore normalize --dry-run tests/fixtures/normalization/simple");
}
