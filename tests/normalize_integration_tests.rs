#[cfg(test)]
mod tests {
    use music_chore::to_title_case;

    use std::path::Path;
    use std::process::Command;

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
        // Test the normalize command on an existing file
        let test_file = "tests/fixtures/flac/simple/track1.flac";

        if Path::new(test_file).exists() {
            // Test dry-run mode
            let output = Command::new("./target/debug/music-chore")
                .arg("normalize")
                .arg(test_file)
                .arg("--dry-run")
                .output()
                .expect("Failed to execute normalize command");

            // Should not error even if no changes are needed
            assert!(output.status.success());

            // Output should be empty for files with proper title case
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.is_empty() || stdout.contains("NO CHANGE"));
        }
    }

    #[test]
    fn test_normalize_command_on_directory() {
        // Test the normalize command on a directory
        let test_dir = "tests/fixtures/flac/simple";

        if Path::new(test_dir).exists() {
            // Test dry-run mode on directory
            let output = Command::new("./target/debug/music-chore")
                .arg("normalize")
                .arg(test_dir)
                .arg("--dry-run")
                .output()
                .expect("Failed to execute normalize command");

            // Should not error even if no changes are needed
            assert!(output.status.success());
        }
    }

    #[test]
    fn test_normalize_command_error_handling() {
        // Test error handling for non-existent paths
        let output = Command::new("./target/debug/music-chore")
            .arg("normalize")
            .arg("tests/fixtures/nonexistent/file.flac")
            .arg("--dry-run")
            .output()
            .expect("Failed to execute normalize command");

        // Should handle error gracefully (either exit with error or print error message)
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("does not exist")
                || stderr.contains("ERROR")
                || !output.status.success()
        );
    }

    #[test]
    fn test_normalize_command_help() {
        // Test that normalize command shows help when needed
        let output = Command::new("./target/debug/music-chore")
            .arg("normalize")
            .arg("--help")
            .output()
            .expect("Failed to execute normalize command");

        // Should succeed and show help
        assert!(output.status.success());

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Normalize") && stdout.contains("title"));
    }
}
