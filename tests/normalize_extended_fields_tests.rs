#[cfg(test)]
mod tests {
    use music_chore::adapters::audio_formats::{read_metadata, write_metadata};
    use music_chore::core::domain::models::MetadataValue;
    use music_chore::core::services::normalization::CombinedNormalizationReport;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    /// Helper function to create a dummy FLAC file with specified metadata.
    fn create_dummy_flac(
        dir: &TempDir,
        file_name: &str,
        title: Option<&str>,
        artist: Option<&str>,
        album: Option<&str>,
        genre: Option<&str>,
        year: Option<u32>,
    ) -> std::path::PathBuf {
        let file_path = dir.path().join(file_name);
        fs::copy("tests/fixtures/flac/simple/track1.flac", &file_path).unwrap();

        let mut metadata = read_metadata(&file_path).unwrap().metadata;
        metadata.title = title.map(|s| MetadataValue::user_set(s.to_string()));
        metadata.artist = artist.map(|s| MetadataValue::user_set(s.to_string()));
        metadata.album = album.map(|s| MetadataValue::user_set(s.to_string()));
        metadata.genre = genre.map(|s| MetadataValue::user_set(s.to_string()));
        metadata.year = year.map(|y| MetadataValue::user_set(y));
        write_metadata(&file_path, &metadata).unwrap();
        file_path
    }

    #[test]
    fn test_normalize_extended_fields_human_readable() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_dummy_flac(
            &temp_dir,
            "track1.flac",
            Some("a title"),
            Some("an artist"),
            Some("an album"),
            Some("rock"),
            Some(2020),
        );

        let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("normalize")
            .arg(file_path)
            .output()
            .expect("Failed to execute normalize command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Assertions for title, artist, album, genre, and year normalization
        assert!(stdout.contains("NORMALIZED: Title 'a title' -> 'A Title'"));
        assert!(stdout.contains("NORMALIZED: Artist 'an artist' -> 'An Artist'"));
        assert!(stdout.contains("NORMALIZED: Album 'an album' -> 'An Album'"));
        assert!(stdout.contains("NORMALIZED: Genre 'rock' -> 'Rock'"));
        assert!(stdout.contains("NO CHANGE: Year '2020' already normalized"));
    }

    #[test]
    fn test_normalize_extended_fields_json_output() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_dummy_flac(
            &temp_dir,
            "track1.flac",
            Some("a title"),
            Some("an artist"),
            Some("an album"),
            Some("rock"),
            Some(2020),
        );

        let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("normalize")
            .arg(file_path)
            .arg("--json")
            .output()
            .expect("Failed to execute normalize command with --json");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let report: CombinedNormalizationReport = serde_json::from_str(&stdout).unwrap();

        // Title assertions
        let title_reports = report.title_reports;
        assert_eq!(title_reports.len(), 1);
        assert_eq!(title_reports[0].original_title, Some("a title".to_string()));
        assert_eq!(
            title_reports[0].normalized_title,
            Some("A Title".to_string())
        );
        assert!(title_reports[0].changed);

        // Artist assertions
        let artist_reports_list = report.artist_reports;
        assert_eq!(artist_reports_list.len(), 1);
        assert_eq!(
            artist_reports_list[0].original_artist,
            Some("an artist".to_string())
        );
        assert_eq!(
            artist_reports_list[0].normalized_artist,
            Some("An Artist".to_string())
        );
        assert!(artist_reports_list[0].changed);

        // Album assertions
        let album_reports = report.album_reports;
        assert_eq!(album_reports.len(), 1);
        assert_eq!(
            album_reports[0].original_album,
            Some("an album".to_string())
        );
        assert_eq!(
            album_reports[0].normalized_album,
            Some("An Album".to_string())
        );
        assert!(album_reports[0].changed);

        // Genre assertions
        let genre_reports = report.genre_reports;
        assert_eq!(genre_reports.len(), 1);
        assert_eq!(genre_reports[0].original_genre, Some("rock".to_string()));
        assert_eq!(genre_reports[0].normalized_genre, Some("Rock".to_string()));
        assert!(genre_reports[0].changed);

        // Year assertions
        let year_reports = report.year_reports;
        assert_eq!(year_reports.len(), 1);
        assert_eq!(year_reports[0].original_year, Some(2020));
        assert_eq!(year_reports[0].normalized_year, Some(2020));
        assert!(!year_reports[0].changed);
    }
}
