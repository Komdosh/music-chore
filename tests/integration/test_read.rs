#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use music_chore::infra::audio::flac::read_flac_metadata;

    #[test]
    fn test_read_flac_metadata_invalid() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("track.flac");
        // Create an empty file to simulate a FLAC file.
        fs::write(&file_path, b"").unwrap();

        let result = read_flac_metadata(&file_path);
        assert!(result.is_err(), "Expected error for empty FLAC file");
    }
}
