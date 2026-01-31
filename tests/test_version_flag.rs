use std::process::Command;

#[test]
fn test_version_flag_short() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "musicctl", "--", "-v"])
        .output()
        .expect("Failed to run musicctl -v");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(stdout.starts_with("musicctl "));
    assert!(stdout.contains("0.1."));
}

#[test]
fn test_version_flag_long() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "musicctl", "--", "--version"])
        .output()
        .expect("Failed to run musicctl --version");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(stdout.starts_with("musicctl "));
    assert!(stdout.contains("0.1."));
}

#[test]
fn test_version_flag_format() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "musicctl", "--", "-v"])
        .output()
        .expect("Failed to run musicctl -v");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    // Should be exactly "musicctl X.Y.Z\n" format
    let version_line = stdout.trim();
    assert!(version_line.starts_with("musicctl "));
    assert!(version_line.len() > 10); // "musicctl 0.1.1" = 15 chars minimum

    // Version should follow semantic versioning
    let version_part = &version_line[9..]; // Skip "musicctl "
    let parts: Vec<&str> = version_part.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "Version should have major.minor.patch format"
    );

    // Each part should be numeric
    for part in parts {
        assert!(
            part.chars().all(|c| c.is_ascii_digit()),
            "Version parts should be numeric"
        );
    }
}

#[test]
fn test_version_flag_takes_precedence() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "musicctl", "--", "-v", "scan", "/fake/path"])
        .output()
        .expect("Failed to run musicctl -v scan");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(stdout.starts_with("musicctl "));
    // Should only show version, not execute scan command
    assert!(!stdout.contains("scan"));
}

#[test]
fn test_help_shows_version_option() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "musicctl", "--", "--help"])
        .output()
        .expect("Failed to run musicctl --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(stdout.contains("-v, --version"));
    assert!(stdout.contains("Show version information"));
}

#[test]
fn test_no_command_shows_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "musicctl", "--"])
        .output()
        .expect("Failed to run musicctl without command");

    // Should show help information even without explicit --help
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    assert!(stdout.contains("Deterministic, AI‑friendly music metadata compiler"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("scan"));
    assert!(stdout.contains("tree"));
    assert!(stdout.contains("read"));
    assert!(stdout.contains("write"));
    assert!(stdout.contains("normalize"));
    assert!(stdout.contains("emit"));
    assert!(stdout.contains("Options:"));
    assert!(stdout.contains("-v, --version"));
}
