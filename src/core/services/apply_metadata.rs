use crate::adapters::audio_formats::{read_metadata, write_metadata};
use std::fmt::Write;
use std::path::Path;
use crate::core::errors::MusicChoreError;

pub fn write_metadata_by_path(
    file: &Path,
    set: Vec<String>,
    apply: bool,
    dry_run: bool,
) -> Result<String, MusicChoreError> {
    if apply && dry_run {
        return Err(MusicChoreError::Other("Cannot use both --apply and --dry-run flags simultaneously".to_string()));
    }

    // If neither flag is provided, default to dry-run behavior for safety
    let effective_apply = apply;
    let effective_dry_run = if !apply && !dry_run {
        true  // Default to dry-run when neither flag is provided
    } else {
        dry_run
    };

    // Check if file exists (we need it for both apply and dry-run modes to read current metadata)
    if !file.exists() {
        return Err(MusicChoreError::FileNotFound(file.display().to_string()));
    }

    if effective_apply && !crate::adapters::audio_formats::is_format_supported(file) {
        return Err(MusicChoreError::UnsupportedAudioFormat(file.display().to_string()));
    }

    // Read current metadata
    let mut track = match read_metadata(file) {
        Ok(track) => track,
        Err(e) => {
            return Err(MusicChoreError::MetadataParseError(format!(
                "Unsupported file format: {}, error: {}",
                file.display(),
                e
            )));
        }
    };

    let mut out = String::new();

    // Parse and apply metadata updates
    for metadata_item in set {
        if let Some((key, value)) = metadata_item.split_once('=') {
            match apply_metadata_update(&mut track.metadata, key.trim(), value.trim()) {
                Ok(()) => {
                    if effective_dry_run {
                        writeln!(out, "DRY RUN: Would set {} = {}", key.trim(), value.trim())
                            .unwrap();
                    }
                }
                Err(e) => {
                    return Err(MusicChoreError::Other(format!("Error parsing metadata '{}': {}", metadata_item, e)));
                }
            }
        } else {
            return Err(MusicChoreError::InvalidMetadataField { 
                field: "format".to_string(), 
                value: metadata_item 
            });
        }
    }

    if effective_dry_run {
        writeln!(out, "DRY RUN: No changes made to file: {}", file.display()).unwrap();
        return Ok(out);
    }

    match write_metadata(file, &track.metadata) {
        Ok(()) => {
            writeln!(out, "Successfully updated metadata: {}", file.display()).unwrap();
            Ok(out)
        }
        Err(e) => Err(MusicChoreError::Other(format!("Error writing metadata: {}", e))),
    }
}

/// Apply a metadata update to the track metadata
fn apply_metadata_update(
    metadata: &mut crate::TrackMetadata,
    key: &str,
    value: &str,
) -> Result<(), MusicChoreError> {
    use crate::core::domain::models::MetadataValue;

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
                .map_err(|_| MusicChoreError::InvalidMetadataField { 
                    field: key.to_string(), 
                    value: value.to_string() 
                })?;
            metadata.track_number = Some(MetadataValue::user_set(num));
        }
        "discnumber" | "disc_number" => {
            let num = value
                .parse::<u32>()
                .map_err(|_| MusicChoreError::InvalidMetadataField { 
                    field: key.to_string(), 
                    value: value.to_string() 
                })?;
            metadata.disc_number = Some(MetadataValue::user_set(num));
        }
        "year" => {
            let year = value
                .parse::<u32>()
                .map_err(|_| MusicChoreError::InvalidMetadataField { 
                    field: key.to_string(), 
                    value: value.to_string() 
                })?;
            metadata.year = Some(MetadataValue::user_set(year));
        }
        "genre" => {
            metadata.genre = Some(MetadataValue::user_set(value.to_string()));
        }
        _ => {
            return Err(MusicChoreError::InvalidMetadataField { 
                field: key.to_string(), 
                value: value.to_string() 
            });
        }
    }

    Ok(())
}
