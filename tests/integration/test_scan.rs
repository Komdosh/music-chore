#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use serde_json::{Value, to_string_pretty};
    use music_chore::infra::scan_dir;

    #[test]
    fn test_scan_dir_with_flac() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("artist/album");
        fs::create_dir_all(&sub).unwrap();
        let file1 = sub.join("track1.flac");
        fs::write(&file1, b"dummy").unwrap();
        let file2 = sub.join("track2.FLAC");
        fs::write(&file2, b"dummy").unwrap();
        let tracks = scan_dir(dir.path());
        assert_eq!(tracks.len(), 2);
        let paths: Vec<_> = tracks.into_iter().map(|t| t.file_path).collect();
        assert!(paths.iter().any(|p| p.ends_with("track1.flac")));
        assert!(paths.iter().any(|p| p.ends_with("track2.FLAC")));
    }

    #[test]
    fn test_scan_json_output() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("artist");
        fs::create_dir_all(&sub).unwrap();
        let file1 = sub.join("song.flac");
        fs::write(&file1, b"dummy").unwrap();
        let tracks = scan_dir(dir.path());
        let json = to_string_pretty(&tracks).unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 1);
        let obj = &v.as_array().unwrap()[0];
        assert_eq!(obj["title"].as_str().unwrap(), "song");
        assert_eq!(obj["provenance"].as_str().unwrap(), "Inferred");
    }
}
