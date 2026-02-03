use lofty::{
    config::WriteOptions,
    file::{AudioFile, TaggedFileExt},
    read_from_path,
};
use std::fs;
use std::path::Path;

fn main() {
    let source_path = Path::new("tests/fixtures/flac/simple/track1.flac");
    let dest_path =
        Path::new("tests/fixtures/artist_bracket/Some guy [FLAC]/05. Shard/no_metadata.flac");

    // Ensure destination directory exists
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create destination directory");
    }

    // Copy the file first
    fs::copy(source_path, dest_path).expect("Failed to copy file");

    // Read the copied file
    let mut tagged_file = read_from_path(dest_path).expect("Failed to read FLAC file");

    // Get the primary tag and clear all items
    if let Some(tag) = tagged_file.primary_tag_mut() {
        // Clear all items from the tag
        let keys_to_remove: Vec<_> = tag.items().map(|item| item.key().clone()).collect();
        for key in keys_to_remove {
            tag.remove_key(&key);
        }
    }

    // Save the file without metadata
    let write_options = WriteOptions::default();
    tagged_file
        .save_to_path(dest_path, write_options)
        .expect("Failed to save FLAC file");

    println!(
        "Successfully created {} with all metadata removed",
        dest_path.display()
    );
}
