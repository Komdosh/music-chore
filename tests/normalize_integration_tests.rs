#[cfg(test)]
mod tests {
    use music_chore::to_title_case;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;
    use music_chore::core::services::normalization::CombinedNormalizationReport; // Import CombinedNormalizationReport
    use music_chore::adapters::audio_formats::{read_metadata, write_metadata};
    use music_chore::core::domain::models::{MetadataValue};

    #[test]
    fn test_to_title_case_basic() {
        assert_eq!(to_title_case("hello world"), "Hello World");
        assert_eq!(to_title_case("HELLO WORLD"), "Hello World");
        assert_eq!(to_title_case("hElLo WoRLd"), "Hello World");
    }

    #[test]
    fn test_to_title_case_with_special_chars() {
        assert_eq!(to_title_case("hello-world"), "Hello-World");
        assert_eq!(to_title_case("hello_world"), "Hello_World");
        assert_eq!(to_title_case("  hello  world  "), "  Hello  World  ");
    }

    #[test]
    fn test_to_title_case_empty() {
        assert_eq!(to_title_case(""), "");
        assert_eq!(to_title_case(" "), " ");
    }

    #[test]
    fn test_normalize_command_on_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("track1.flac");
        fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

        // Ensure track1.flac metadata is as expected for test
        let mut track_metadata = read_metadata(&test_file).unwrap().metadata;
        track_metadata.title = Some(MetadataValue::user_set("Test Song".to_string()));
        track_metadata.genre = Some(MetadataValue::user_set("Rock".to_string()));
        write_metadata(&test_file, &track_metadata).unwrap();


        // Test human-readable output
        let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("normalize")
            .arg(&test_file)
            .output()
            .expect("Failed to execute normalize command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("NO CHANGE: Title 'Test Song' already normalized in"));
        assert!(stdout.contains("Title Summary: 0 normalized, 1 no change, 0 errors"));
        assert!(stdout.contains("NO CHANGE: Genre 'Rock' already normalized in"));
        assert!(stdout.contains("Genre Summary: 0 normalized, 1 no change, 0 errors"));

        // Test JSON output
        let output_json = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("normalize")
            .arg(&test_file)
            .arg("--json")
            .output()
            .expect("Failed to execute normalize command");

        assert!(output_json.status.success());
        let stdout_json = String::from_utf8_lossy(&output_json.stdout);
        let combined_report: CombinedNormalizationReport = serde_json::from_str(&stdout_json).unwrap();
        assert_eq!(combined_report.title_reports.len(), 1);
        assert_eq!(combined_report.title_reports[0].original_title, Some("Test Song".to_string()));
        assert_eq!(combined_report.title_reports[0].normalized_title, Some("Test Song".to_string()));
        assert_eq!(combined_report.title_reports[0].changed, false);

        assert_eq!(combined_report.genre_reports.len(), 1);
        assert_eq!(combined_report.genre_reports[0].original_genre, Some("Rock".to_string()));
        assert_eq!(combined_report.genre_reports[0].normalized_genre, Some("Rock".to_string()));
        assert_eq!(combined_report.genre_reports[0].changed, false);
    }

    #[test]
    fn test_normalize_command_on_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("music");
        fs::create_dir_all(&test_dir).unwrap();

        let track1_path = test_dir.join("track1.flac"); // Will be no change
        let track2_path = test_dir.join("track2.flac"); // Will be normalized

        // Copy original files and set metadata
        fs::copy("tests/fixtures/flac/simple/track1.flac", &track1_path).unwrap();
        let mut track1_metadata = read_metadata(&track1_path).unwrap().metadata;
        track1_metadata.title = Some(MetadataValue::user_set("Test Song".to_string()));
        track1_metadata.genre = Some(MetadataValue::user_set("Rock".to_string()));
        write_metadata(&track1_path, &track1_metadata).unwrap();

        fs::copy("tests/fixtures/flac/simple/track1.flac", &track2_path).unwrap();
        let mut track2_metadata = read_metadata(&track2_path).unwrap().metadata; // Use same fixture
        track2_metadata.title = Some(MetadataValue::user_set("this is a test".to_string()));
        track2_metadata.genre = Some(MetadataValue::user_set("punk rock".to_string()));
        write_metadata(&track2_path, &track2_metadata).unwrap();


        // Test human-readable output
        let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("normalize")
            .arg(&test_dir)
            .output()
            .expect("Failed to execute normalize command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("NO CHANGE: Title 'Test Song' already normalized in"));
        assert!(stdout.contains("NORMALIZED: Title 'this is a test' -> 'This Is A Test' in"));
        assert!(stdout.contains("Title Summary: 1 normalized, 1 no change, 0 errors"));
        assert!(stdout.contains("NO CHANGE: Genre 'Rock' already normalized in"));
        assert!(stdout.contains("NORMALIZED: Genre 'punk rock' -> 'Punk' in"));
        assert!(stdout.contains("Genre Summary: 1 normalized, 1 no change, 0 errors"));


        // Test JSON output
        let output_json = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("normalize")
            .arg(&test_dir)
            .arg("--json")
            .output()
            .expect("Failed to execute normalize command");

        assert!(output_json.status.success());
        let stdout_json = String::from_utf8_lossy(&output_json.stdout);
        let combined_report: CombinedNormalizationReport = serde_json::from_str(&stdout_json).unwrap();
        assert_eq!(combined_report.title_reports.len(), 2);
        assert!(combined_report.title_reports.iter().any(|r| r.original_title == Some("Test Song".to_string()) && r.changed == false));
        assert!(combined_report.title_reports.iter().any(|r| r.original_title == Some("this is a test".to_string()) && r.normalized_title == Some("This Is A Test".to_string()) && r.changed == true));

        assert_eq!(combined_report.genre_reports.len(), 2);
        assert!(combined_report.genre_reports.iter().any(|r| r.original_genre == Some("Rock".to_string()) && r.changed == false));
        assert!(combined_report.genre_reports.iter().any(|r| r.original_genre == Some("punk rock".to_string()) && r.normalized_genre == Some("Punk".to_string()) && r.changed == true));
    }

    #[test]
    fn test_normalize_command_error_handling() {
        let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("normalize")
            .arg("tests/fixtures/nonexistent/file.flac")
            .output()
            .expect("Failed to execute normalize command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("does not exist")
                || stderr.contains("ERROR")
                || !output.status.success()
        );
    }

    #[test]
    fn test_normalize_command_help() {
        let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("normalize")
            .arg("--help")
            .output()
            .expect("Failed to execute normalize command");

        assert!(output.status.success());

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Normalize"));
    }
}