#[cfg(test)]
mod tests {
    use music_chore::to_title_case;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;

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

        // Test human-readable output
        let output = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("normalize")
            .arg(&test_file)
            .output()
            .expect("Failed to execute normalize command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("NO CHANGE: Title 'Test Song' already normalized in"));
        assert!(stdout.contains("Summary: 0 normalized, 1 no change, 0 errors"));

        // Test JSON output
        let output_json = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("normalize")
            .arg(&test_file)
            .arg("--json")
            .output()
            .expect("Failed to execute normalize command");

        assert!(output_json.status.success());
        let stdout_json = String::from_utf8_lossy(&output_json.stdout);
        let reports: Vec<serde_json::Value> = serde_json::from_str(&stdout_json).unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0]["original_title"], "Test Song");
        assert_eq!(reports[0]["normalized_title"], "Test Song");
        assert_eq!(reports[0]["changed"], false);
    }

    #[test]
    fn test_normalize_command_on_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("music");
        fs::create_dir_all(&test_dir).unwrap();

        let track1_path = test_dir.join("track1.flac");
        let track2_path = test_dir.join("track2.flac"); // Will be normalized

        // Copy original files
        fs::copy("tests/fixtures/flac/simple/track1.flac", &track1_path).unwrap();
        fs::copy("tests/fixtures/flac/simple/track1.flac", &track2_path).unwrap(); // Use same fixture, will infer title from filename

        // Manually set title for track2 to trigger normalization
        use music_chore::adapters::audio_formats::{read_metadata, write_metadata};
        use music_chore::core::domain::models::{MetadataSource, MetadataValue};

        let mut track2_metadata = read_metadata(&track2_path).unwrap().metadata;
        track2_metadata.title = Some(MetadataValue::user_set("this is a test".to_string()));
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
        assert!(stdout.contains("Summary: 1 normalized, 1 no change, 0 errors"));


        // Test JSON output
        let output_json = Command::new(env!("CARGO_BIN_EXE_musicctl"))
            .arg("normalize")
            .arg(&test_dir)
            .arg("--json")
            .output()
            .expect("Failed to execute normalize command");

        assert!(output_json.status.success());
        let stdout_json = String::from_utf8_lossy(&output_json.stdout);
        let reports: Vec<serde_json::Value> = serde_json::from_str(&stdout_json).unwrap();
        assert_eq!(reports.len(), 2);
        assert!(reports.iter().any(|r| r["original_title"] == "Test Song" && r["changed"] == false));
        assert!(reports.iter().any(|r| r["original_title"] == "this is a test" && r["normalized_title"] == "This Is A Test" && r["changed"] == true));
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