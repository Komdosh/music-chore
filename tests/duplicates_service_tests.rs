use music_chore::core::services::duplicates::find_duplicates;

#[test]
fn test_find_duplicates_no_duplicates_returns_ok_text() {
    let result = find_duplicates(std::path::Path::new("tests/fixtures/flac/simple"), false, false, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "No duplicate tracks found.");
}

#[test]
fn test_find_duplicates_no_duplicates_returns_ok_empty_json() {
    let result = find_duplicates(std::path::Path::new("tests/fixtures/flac/simple"), true, false, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "[]");
}
