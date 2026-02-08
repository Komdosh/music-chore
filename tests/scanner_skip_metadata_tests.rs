#[cfg(test)]
mod tests {
    use music_chore::adapters::audio_formats::{read_metadata, write_metadata};
    use music_chore::core::domain::models::{MetadataValue, Track, TrackMetadata, MetadataSource};
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;
    use serde_json::Value;

    /// Helper function to create a dummy FLAC file with specified metadata.
    fn create_dummy_flac_with_metadata(
        dir: &TempDir,
        file_name: &str,
        title: Option<&str>,
        artist: Option<&str>,
        album: Option<&str>,
        year: Option<u32>,
    ) -> PathBuf {
        let file_path = dir.path().join(file_name);
        fs::copy("tests/fixtures/flac/simple/track1.flac", &file_path).unwrap();

        let metadata = TrackMetadata {
            title: title.map(|s| MetadataValue::embedded(s.to_string())),
            artist: artist.map(|s| MetadataValue::embedded(s.to_string())),
            album: album.map(|s| MetadataValue::embedded(s.to_string())),
            year: year.map(|y| MetadataValue::embedded(y)),
            format: "flac".to_string(),
            path: file_path.clone(),
            album_artist: None,
            track_number: None,
            disc_number: None,
            genre: None,
            duration: None,
        };
        write_metadata(&file_path, &metadata).unwrap();
        file_path
    }

    /// Helper function to create a dummy FLAC file without embedded metadata.
    fn create_dummy_flac_without_metadata(dir: &TempDir, file_name: &str) -> PathBuf {
        let file_path = dir.path().join(file_name);
        fs::copy("tests/fixtures/flac/simple/track1.flac", &file_path).unwrap(); // Copy a file with some content
        
        // Overwrite metadata with empty/none fields
        let metadata = TrackMetadata {
            title: None, artist: None, album: None, album_artist: None,
            track_number: None, disc_number: None, year: None, genre: None,
            duration: None,
            format: "flac".to_string(),
            path: file_path.clone(),
        };
        write_metadata(&file_path, &metadata).unwrap();
        file_path
    }

    #[test]
    fn test_scan_skip_metadata_human_readable() {
        let temp_dir = TempDir::new().unwrap();

        // File with embedded metadata
        let _file_with_meta = create_dummy_flac_with_metadata(
            &temp_dir,
            "Artist - Album - Title.flac",
            Some("Embedded Title"),
            Some("Embedded Artist"),
            Some("Embedded Album"),
            Some(2023),
        );

        // File without embedded metadata, relying on filename inference
        let _file_no_meta = create_dummy_flac_without_metadata(&temp_dir, "Inferred Artist - Inferred Album - Inferred Title.flac");

        let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("scan")
            .arg(temp_dir.path())
            .arg("--skip-metadata")
            .output()
            .expect("Failed to execute scan command with --skip-metadata");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Assertions for file with embedded metadata (should be skipped)
        assert!(!stdout.contains("Embedded Title"));
        assert!(!stdout.contains("Embedded Artist"));
        assert!(!stdout.contains("Embedded Album"));
        assert!(!stdout.contains("2023"));
        assert!(stdout.contains(&format!("Artist - Album - Title.flac [Artist - Album - Title 🤖]")));


        // Assertions for file without embedded metadata (should use filename inference)
        assert!(stdout.contains(&format!("Inferred Artist - Inferred Album - Inferred Title.flac [Inferred Artist - Inferred Album - Inferred Title 🤖]")));
    }

    #[test]
    fn test_scan_skip_metadata_json_output() {
        let temp_dir = TempDir::new().unwrap();

        // File with embedded metadata
        let _file_with_meta = create_dummy_flac_with_metadata(
            &temp_dir,
            "Artist - Album - Title.flac",
            Some("Embedded Title"),
            Some("Embedded Artist"),
            Some("Embedded Album"),
            Some(2023),
        );

        // File without embedded metadata, relying on filename inference
        let _file_no_meta = create_dummy_flac_without_metadata(&temp_dir, "Inferred Artist - Inferred Album - Inferred Title.flac");

        let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("scan")
            .arg(temp_dir.path())
            .arg("--skip-metadata")
            .arg("--json")
            .output()
            .expect("Failed to execute scan command with --skip-metadata --json");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_tracks: Vec<Track> = serde_json::from_str(&stdout).unwrap();

        assert_eq!(json_tracks.len(), 2);

        // Check file with embedded metadata (should be skipped, but filename should be used for inference)
        let track1 = json_tracks.iter().find(|t| t.file_path.file_name().unwrap().to_str().unwrap() == "Artist - Album - Title.flac").unwrap();
        assert!(track1.metadata.title.is_some()); // Title should be extracted from filename
        assert_eq!(track1.metadata.title.as_ref().unwrap().value, "Artist - Album - Title"); // Value should be from filename
        assert!(track1.metadata.artist.is_some()); // Inferred from filename
        assert!(track1.metadata.album.is_some());  // Inferred from filename
        assert!(track1.metadata.year.is_none());

        // Check file without embedded metadata (should use filename inference)
        let track2 = json_tracks.iter().find(|t| t.file_path.file_name().unwrap().to_str().unwrap() == "Inferred Artist - Inferred Album - Inferred Title.flac").unwrap();
        assert!(track2.metadata.title.is_some()); // Title should be extracted from filename
        assert_eq!(track2.metadata.title.as_ref().unwrap().value, "Inferred Artist - Inferred Album - Inferred Title"); // Value should be from filename
        assert!(track2.metadata.artist.is_some()); // Inferred from filename
        assert!(track2.metadata.album.is_some());  // Inferred from filename
        assert!(track2.metadata.year.is_none());
    }

    #[test]
    fn test_scan_default_metadata_reading() {
        let temp_dir = TempDir::new().unwrap();
        let _file_path = create_dummy_flac_with_metadata(
            &temp_dir,
            "Test - Album - Song.flac",
            Some("Original Title"),
            Some("Original Artist"),
            Some("Original Album"),
            Some(2022),
        );

        let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("scan")
            .arg(temp_dir.path())
            .arg("--json") // Use JSON to easily check metadata presence
            .output()
            .expect("Failed to execute scan command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_tracks: Vec<Track> = serde_json::from_str(&stdout).unwrap();

        assert_eq!(json_tracks.len(), 1);
        let track = &json_tracks[0];

        // Assert that embedded metadata is read
        assert_eq!(track.metadata.title.as_ref().map(|v| &v.value), Some(&"Original Title".to_string()));
        assert_eq!(track.metadata.artist.as_ref().map(|v| &v.value), Some(&"Original Artist".to_string()));
        assert_eq!(track.metadata.album.as_ref().map(|v| &v.value), Some(&"Original Album".to_string()));
        assert_eq!(track.metadata.year.as_ref().map(|v| v.value), Some(2022));
        assert_eq!(track.metadata.title.as_ref().map(|v| v.source.clone()), Some(MetadataSource::Embedded));
    }
}
