use crate::services::formats::{read_metadata, write_metadata};
use std::fmt::Write;
use std::path::PathBuf;

pub fn write_metadata_by_path(
    file: &PathBuf,
    set: Vec<String>,
    apply: bool,
    dry_run: bool,
) -> Result<String, String> {
    if apply && dry_run {
        return Err("Error: Cannot use both --apply and --dry-run flags simultaneously".into());
    }

    if !apply && !dry_run {
        return Err("Error: Must specify either --apply or --dry-run flag".into());
    }

    // Check if file exists and is supported
    if !file.exists() {
        return Err(format!("Error: File does not exist: {}", file.display()));
    }

    if !crate::services::formats::is_format_supported(&file) {
        return Err(format!(
            "Error: Unsupported file format: {}",
            file.display()
        ));
    }

    // Read current metadata
    let mut track = match read_metadata(&file) {
        Ok(track) => track,
        Err(e) => {
            return Err(format!(
                "Error: Unsupported file format: {}, error: {}",
                file.display(),
                e
            ));
        }
    };

    let mut out = String::new();

    // Parse and apply metadata updates
    for metadata_item in set {
        if let Some((key, value)) = metadata_item.split_once('=') {
            match apply_metadata_update(&mut track.metadata, key.trim(), value.trim()) {
                Ok(()) => {
                    if dry_run {
                        writeln!(out, "DRY RUN: Would set {} = {}", key.trim(), value.trim())
                            .unwrap();
                    }
                }
                Err(e) => {
                    return Err(format!("Error parsing metadata '{}': {}", metadata_item, e));
                }
            }
        } else {
            return Err(format!(
                "Error: Unsupported metadata format: {}",
                metadata_item
            ));
        }
    }

    if dry_run {
        writeln!(out, "DRY RUN: No changes made to file: {}", file.display()).unwrap();
        return Ok(out);
    }

    match write_metadata(&file, &track.metadata) {
        Ok(()) => {
            writeln!(out, "Successfully updated metadata: {}", file.display()).unwrap();
            Ok(out)
        }
        Err(e) => Err(format!("Error writing metadata: {}", e)),
    }
}

/// Apply a metadata update to the track metadata
fn apply_metadata_update(
    metadata: &mut crate::TrackMetadata,
    key: &str,
    value: &str,
) -> Result<(), String> {
    use crate::domain::models::MetadataValue;

    match key.to_lowercase().as_str() {
        "title" => {
            metadata.title = Some(MetadataValue::user_set(value.to_string()));
        }
        "artist" => {
            metadata.artist = Some(MetadataValue::user_set(value.to_string()));
        }
        "album" => {
            metadata.album = Some(MetadataValue::user_set(value.to_string()));
        }
        "albumartist" | "album_artist" => {
            metadata.album_artist = Some(MetadataValue::user_set(value.to_string()));
        }
        "tracknumber" | "track_number" => {
            let num = value
                .parse::<u32>()
                .map_err(|_| format!("Invalid track number: {}", value))?;
            metadata.track_number = Some(MetadataValue::user_set(num));
        }
        "discnumber" | "disc_number" => {
            let num = value
                .parse::<u32>()
                .map_err(|_| format!("Invalid disc number: {}", value))?;
            metadata.disc_number = Some(MetadataValue::user_set(num));
        }
        "year" => {
            let year = value
                .parse::<u32>()
                .map_err(|_| format!("Invalid year: {}", value))?;
            metadata.year = Some(MetadataValue::user_set(year));
        }
        "genre" => {
            metadata.genre = Some(MetadataValue::user_set(value.to_string()));
        }
        _ => {
            return Err(format!("Unsupported metadata field: {}", key));
        }
    }

    Ok(())
}
