#[cfg(test)]
mod wav_metadata_tests {
    use music_chore::adapters::audio_formats::read_metadata;
    use std::path::Path;

    #[test]
    fn test_read_wav_metadata_with_real_file() {
        let fixture_path = Path::new("tests/fixtures/wav/simple/track1.wav");
        if !fixture_path.exists() {
            return; // Skip test if fixture doesn't exist
        }

        match read_metadata(fixture_path) {
            Ok(track) => {
                // Verify that track has basic metadata populated
                let metadata = &track.metadata;

                // Check that format is correctly identified
                assert_eq!(metadata.format, "wav");

                // Check that duration is extracted from the file
                assert!(
                    metadata.duration.is_some(),
                    "Duration should be set for WAV file"
                );

                // Verify the path is correct
                assert_eq!(metadata.path, fixture_path);

                // For WAV files with INFO chunks, metadata might be minimal or missing
                // But we should still have proper structure
                println!("WAV metadata reading successful for: {:?}", fixture_path);
            }
            Err(e) => {
                panic!("Failed to read WAV metadata: {:?}", e);
            }
        }
    }

    #[test]
    fn test_read_wav_nested_structure() {
        let fixture_path =
            Path::new("tests/fixtures/wav/nested/The Beatles/Abbey Road/01 - Come Together.wav");
        if !fixture_path.exists() {
            return; // Skip test if fixture doesn't exist
        }

        match read_metadata(fixture_path) {
            Ok(track) => {
                let metadata = &track.metadata;

                // Verify format
                assert_eq!(metadata.format, "wav");

                // Check path-based inference working
                // Since INFO chunks might not contain metadata, fallback inference should work
                assert!(
                    metadata.artist.is_some() || metadata.album.is_some(),
                    "Should have at least artist or album from path inference"
                );

                // If metadata exists, it should come from embedded or inferred source
                if let Some(ref artist) = metadata.artist {
                    match artist.source {
                        music_chore::core::domain::models::MetadataSource::Embedded => {
                            println!("Found embedded artist: {}", artist.value);
                        }
                        music_chore::core::domain::models::MetadataSource::FolderInferred => {
                            assert_eq!(artist.value, "The Beatles");
                        }
                        music_chore::core::domain::models::MetadataSource::UserEdited => {
                            // This shouldn't happen in tests, but handle it
                            println!("Found user-edited artist: {}", artist.value);
                        }
                        music_chore::core::domain::models::MetadataSource::CueInferred => {
                            panic!("Unexpected MetadataSource::CueInferred for artist in WAV test");
                        }
                    }
                }

                if let Some(ref album) = metadata.album {
                    match album.source {
                        music_chore::core::domain::models::MetadataSource::Embedded => {
                            println!("Found embedded album: {}", album.value);
                        }
                        music_chore::core::domain::models::MetadataSource::FolderInferred => {
                            assert_eq!(album.value, "Abbey Road");
                        }
                        music_chore::core::domain::models::MetadataSource::UserEdited => {
                            // This shouldn't happen in tests, but handle it
                            println!("Found user-edited album: {}", album.value);
                        }
                        music_chore::core::domain::models::MetadataSource::CueInferred => {
                            panic!("Unexpected MetadataSource::CueInferred for album in WAV test");
                        }
                    }
                }
            }
            Err(e) => {
                panic!("Failed to read nested WAV metadata: {:?}", e);
            }
        }
    }

    #[test]
    fn test_read_wav_unicode_support() {
        let fixture_path = Path::new("tests/fixtures/wav/unicode/José González/album/track.wav");
        if !fixture_path.exists() {
            return; // Skip test if fixture doesn't exist
        }

        match read_metadata(fixture_path) {
            Ok(track) => {
                let metadata = &track.metadata;

                // Verify format
                assert_eq!(metadata.format, "wav");

                // Verify path contains Unicode characters correctly
                assert!(metadata.path.to_string_lossy().contains("José González"));

                // Check that Unicode path inference works
                assert!(
                    metadata.artist.is_some(),
                    "Should have artist from path with Unicode characters"
                );
            }
            Err(e) => {
                panic!("Failed to read WAV Unicode metadata: {:?}", e);
            }
        }
    }

    #[test]
    fn test_wav_handler_supported_extensions() {
        use music_chore::core::domain::traits::AudioFile;
        use music_chore::adapters::audio_formats::wav::WavHandler;

        let handler = WavHandler::new();
        let extensions = handler.supported_extensions();

        assert!(extensions.contains(&"wav"));
        assert_eq!(extensions.len(), 1);
    }

    #[test]
    fn test_wav_handler_can_handle() {
        use music_chore::core::domain::traits::AudioFile;
        use music_chore::adapters::audio_formats::wav::WavHandler;
        use std::path::PathBuf;

        let handler = WavHandler::new();

        // Test positive cases
        assert!(handler.can_handle(&PathBuf::from("test.wav")));
        assert!(handler.can_handle(&PathBuf::from("TEST.WAV"))); // Case insensitive
        assert!(handler.can_handle(&PathBuf::from("/path/to/song.wav")));

        // Test negative cases
        assert!(!handler.can_handle(&PathBuf::from("test.mp3")));
        assert!(!handler.can_handle(&PathBuf::from("test.flac")));
        assert!(!handler.can_handle(&PathBuf::from("test.txt")));
        assert!(!handler.can_handle(&PathBuf::from("testwav"))); // No extension
    }

    #[test]
    fn test_wav_read_basic_info() {
        use music_chore::core::domain::traits::AudioFile;
        use music_chore::adapters::audio_formats::wav::WavHandler;

        let handler = WavHandler::new();
        let fixture_path = Path::new("tests/fixtures/wav/simple/track1.wav");

        if !fixture_path.exists() {
            return; // Skip test if fixture doesn't exist
        }

        match handler.read_basic_info(fixture_path) {
            Ok(metadata) => {
                // Should only have basic info (format, duration, path)
                assert_eq!(metadata.format, "wav");
                assert!(metadata.duration.is_some());
                assert_eq!(metadata.path, fixture_path);

                // Other fields should be None in basic info
                assert!(metadata.title.is_none());
                assert!(metadata.artist.is_none());
                assert!(metadata.album.is_none());
            }
            Err(e) => {
                panic!("Failed to read WAV basic info: {:?}", e);
            }
        }
    }
}