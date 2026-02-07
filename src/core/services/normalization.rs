//! Text normalization services.

use crate::MetadataValue;
use crate::adapters::audio_formats as formats;
use crate::core::domain::models::Track; // Ensure Track is imported
use crate::core::services::scanner::{scan_dir, scan_dir_with_metadata};
use serde::{Deserialize, Serialize}; // Added for report structs
use std::path::{Path, PathBuf};

// Define new structs for reporting normalization outcomes
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TitleNormalizationReport {
    pub original_path: PathBuf,
    pub original_title: Option<String>,
    pub normalized_title: Option<String>,
    pub changed: bool,
    pub error: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenreNormalizationReport {
    pub original_path: PathBuf,
    pub original_genre: Option<String>,
    pub normalized_genre: Option<String>,
    pub changed: bool,
    pub error: Option<String>,
}

pub const STANDARD_GENRES: &[&str] = &[
    "Acoustic",
    "Alternative",
    "Ambient",
    "Avant-Garde",
    "Blues",
    "Classical",
    "Country",
    "Dance",
    "Electronic",
    "Folk",
    "Hip-Hop",
    "House",
    "Indie",
    "Jazz",
    "Metal",
    "Pop",
    "Punk",
    "R&B",
    "Reggae",
    "Rock",
    "Soul",
    "Techno",
    "World",
    "Soundtrack",
    "New Age",
    "Funk",
    "Disco",
    "Swing",
    "Opera",
    "Musical",
    "Children's",
    "Spoken Word",
    "Comedy",
    "Speech",
    "Podcast",
    "Audiobook",
];

const GENRE_ALIASES: &[(&[&str], &str)] = &[
    (
        &[
            "rock and roll",
            "rock & roll",
            "rock'n'roll",
            "rock'n'roll",
            "rock & roll",
            "rock",
        ],
        "Rock",
    ),
    (&["pop rock", "pop-rock", "pop/rock"], "Pop"),
    (
        &[
            "alternative rock",
            "alternative-rock",
            "alt rock",
            "alternative",
        ],
        "Alternative",
    ),
    (
        &[
            "electronic",
            "electronica",
            "electro",
            "edm",
            "electronic dance music",
        ],
        "Electronic",
    ),
    (&["hip hop", "hip-hop", "hiphop", "rap"], "Hip-Hop"),
    (&["r & b", "r&b", "rn b", "rnb", "rhythm and blues"], "R&B"),
    (
        &["classical", "orchestral", "symphony", "chamber music"],
        "Classical",
    ),
    (
        &[
            "jazz",
            "jazz fusion",
            "smooth jazz",
            "free jazz",
            "bebop",
            "swing",
        ],
        "Jazz",
    ),
    (
        &["blues", "delta blues", "chicago blues", "electric blues"],
        "Blues",
    ),
    (
        &["country", "country music", "country & western", "c&w", "nashville"],
        "Country",
    ),
    (
        &["folk", "folk rock", "contemporary folk", "traditional folk"],
        "Folk",
    ),
    (
        &[
            "metal",
            "heavy metal",
            "thrash metal",
            "death metal",
            "black metal",
            "doom metal",
            "metalcore",
        ],
        "Metal",
    ),
    (
        &[
            "punk",
            "punk rock",
            "hardcore punk",
            "post-punk",
            "garage punk",
        ],
        "Punk",
    ),
    (&["reggae", "dancehall", "dub", "rocksteady"], "Reggae"),
    (&["soul", "neo soul", "southern soul", "motown"], "Soul"),
    (
        &["funk", "funk soul", "psychedelic funk", "afrobeat"],
        "Funk",
    ),
    (&["disco", "dance disco", "eurodance"], "Disco"),
    (
        &[
            "ambient",
            "ambient music",
            "chillout",
            "downtempo",
            "lounge",
        ],
        "Ambient",
    ),
    (
        &["techno", "techno music", "detroit techno", "minimal techno"],
        "Techno",
    ),
    (
        &[
            "house",
            "house music",
            "deep house",
            "progressive house",
            "tech house",
        ],
        "House",
    ),
    (
        &[
            "indie",
            "indie rock",
            "indie pop",
            "indie folk",
            "alternative indie",
        ],
        "Indie",
    ),
    (&["acoustic", "acoustic music", "unplugged"], "Acoustic"),
    (
        &[
            "soundtrack",
            "movie soundtrack",
            "film score",
            "original soundtrack",
            "ost",
        ],
        "Soundtrack",
    ),
    (
        &["world", "world music", "international", "ethnic"],
        "World",
    ),
    (
        &["new age", "newage", "relaxation", "meditation"],
        "New Age",
    ),
    (&["spoken word", "poetry", "readings"], "Spoken Word"),
    (&["audiobook", "audio book", "books"], "Audiobook"),
    (
        &["children", "children's", "kids", "for children"],
        "Children's",
    ),
    (&["comedy", "comedic", "humor"], "Comedy"),
    (
        &["musical", "musical theater", "musical theatre", "broadway"],
        "Musical",
    ),
    (&["opera", "operatic", "opera singer"], "Opera"),
    (
        &["avantgarde", "avant garde", "experimental", "avant-garde"],
        "Avant-Garde",
    ),
];

pub fn normalize_genre(genre: &str) -> Option<String> {
    let normalized: Vec<String> = genre
        .trim()
        .split('/')
        .map(|g| {
            let g = g.trim().to_lowercase();
            for (aliases, standard) in GENRE_ALIASES {
                if aliases.iter().any(|a| *a == g) {
                    return standard.to_string();
                }
            }
            let capitalized = to_title_case(&g);
            if STANDARD_GENRES
                .iter()
                .any(|s| s.to_lowercase() == capitalized.to_lowercase())
            {
                STANDARD_GENRES
                    .iter()
                    .find(|s| s.to_lowercase() == capitalized.to_lowercase())
                    .unwrap()
                    .to_string()
            } else {
                capitalized
            }
        })
        .filter(|g| !g.is_empty())
        .collect();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized.join("/"))
    }
}

pub fn normalize_genres_in_library(path: &Path, json: bool) -> Result<String, String> {
    let tracks = if path.is_file() {
        vec![
            formats::read_metadata(path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?,
        ]
    } else if path.is_dir() {
        scan_dir(path)
    } else {
        return Err(format!("Path does not exist: {}", path.display()));
    };

    let mut reports = Vec::new();

    for track in tracks {
        let original_path = track.file_path.clone();
        let original_genre = track.metadata.genre.as_ref().map(|v| v.value.clone());
        let mut changed = false;
        let mut error = None;
        let normalized_genre = if let Some(ref genre_value) = original_genre {
            match normalize_genre(genre_value) {
                Some(new_genre) => {
                    if new_genre != *genre_value {
                        changed = true;
                    }
                    Some(new_genre)
                }
                None => {
                    error = Some(format!("Could not normalize genre '{}'", genre_value));
                    None
                }
            }
        } else {
            error = Some("No genre found".to_string());
            None
        };

        reports.push(GenreNormalizationReport {
            original_path,
            original_genre,
            normalized_genre,
            changed,
            error,
        });
    }

    if json {
        serde_json::to_string_pretty(&reports).map_err(|e| format!("Error serializing reports to JSON: {}", e))
    } else {
        let mut out = String::new();
        let mut updated_count = 0;
        let mut no_change_count = 0;
        let mut error_count = 0;

        for report in reports {
            if report.error.is_some() {
                out.push_str(&format!("ERROR: {} for {}\n", report.error.unwrap(), report.original_path.display()));
                error_count += 1;
            } else if report.changed {
                out.push_str(&format!(
                    "NORMALIZED: Genre '{}' -> '{}' in {}\n",
                    report.original_genre.unwrap_or_default(),
                    report.normalized_genre.unwrap_or_default(),
                    report.original_path.display()
                ));
                updated_count += 1;
            } else {
                out.push_str(&format!(
                    "NO CHANGE: Genre '{}' already normalized in {}\n",
                    report.original_genre.unwrap_or_default(),
                    report.original_path.display()
                ));
                no_change_count += 1;
            }
        }
        out.push_str(&format!(
            "\nSummary: {} normalized, {} no change, {} errors\n",
            updated_count, no_change_count, error_count
        ));
        Ok(out)
    }
}

pub fn to_title_case(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut capitalize_next = true;

    for c in input.chars() {
        if c.is_whitespace() || c == '-' || c == '_' {
            capitalize_next = true;
            result.push(c);
        } else if capitalize_next {
            for uppercase_char in c.to_uppercase() {
                result.push(uppercase_char);
            }
            capitalize_next = false;
        } else {
            for lowercase_char in c.to_lowercase() {
                result.push(lowercase_char);
            }
        }
    }

    result
}

/// Normalize track titles to title case with options
pub fn normalize(path: PathBuf, json: bool) -> Result<String, String> {
    let mut reports = Vec::new();

    // Check if path is a file or directory
    if path.is_file() {
        // Single file
        match formats::read_metadata(&path) {
            Ok(track) => reports.push(normalize_single_track(track)),
            Err(e) => reports.push(TitleNormalizationReport {
                original_path: path.clone(),
                original_title: None,
                normalized_title: None,
                changed: false,
                error: Some(format!("Failed to read {}: {}", path.display(), e)),
            }),
        }
    } else if path.is_dir() {
        // Directory - scan for supported audio files
        let tracks = scan_dir_with_metadata(&path);
        match tracks {
            Ok(tracks) => {
                for track in tracks {
                    reports.push(normalize_single_track(track));
                }
            }
            Err(e) => {
                // If scanning fails for a directory, report an overall error
                return Err(format!("Error scanning directory {}: {}", path.display(), e));
            }
        }
    } else {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    if json {
        serde_json::to_string_pretty(&reports).map_err(|e| format!("Error serializing reports to JSON: {}", e))
    } else {
        let mut out = String::new();
        let mut updated_count = 0;
        let mut no_change_count = 0;
        let mut error_count = 0;

        for report in reports {
            if report.error.is_some() {
                out.push_str(&format!("ERROR: {} for {}\n", report.error.unwrap(), report.original_path.display()));
                error_count += 1;
            } else if report.changed {
                out.push_str(&format!(
                    "NORMALIZED: Title '{}' -> '{}' in {}\n",
                    report.original_title.unwrap_or_default(),
                    report.normalized_title.unwrap_or_default(),
                    report.original_path.display()
                ));
                updated_count += 1;
            } else {
                out.push_str(&format!(
                    "NO CHANGE: Title '{}' already normalized in {}\n",
                    report.original_title.unwrap_or_default(),
                    report.original_path.display()
                ));
                no_change_count += 1;
            }
        }
        out.push_str(&format!(
            "\nSummary: {} normalized, {} no change, {} errors\n",
            updated_count, no_change_count, error_count
        ));
        Ok(out)
    }
}

/// Normalize a single track's title
fn normalize_single_track(track: Track) -> TitleNormalizationReport {
    let original_path = track.file_path.clone();
    let original_title_from_metadata = track.metadata.title.as_ref().map(|v| v.value.clone());

    let current_title_string_value = if let Some(title) = original_title_from_metadata.as_ref() {
        title.clone() // Clone here to own the string
    } else {
        if let Some(file_stem_str) = original_path.file_stem().and_then(|s| s.to_str()) {
            // Check if file_stem is empty or just an extension
            if file_stem_str.is_empty() || file_stem_str.starts_with('.') {
                return TitleNormalizationReport {
                    original_path,
                    original_title: original_title_from_metadata, // This is None
                    normalized_title: None,
                    changed: false,
                    error: Some("No meaningful title found in metadata or filename".to_string()),
                };
            }

            let extracted_title = if let Some(pos) = file_stem_str.find(" - ") {
                let title_part = &file_stem_str[pos + 3..];
                let title_trimmed = title_part.trim();
                if !title_trimmed.is_empty() {
                    title_trimmed
                } else {
                    file_stem_str
                }
            } else {
                file_stem_str
            };
            extracted_title.to_string() // Own the string
        } else {
            // This branch handles cases where file_stem() is truly None (e.g., just "/dir/").
            // For ".flac", file_stem() returns ".flac", which is handled by the starts_with('.') check above.
            return TitleNormalizationReport {
                original_path,
                original_title: original_title_from_metadata,
                normalized_title: None,
                changed: false,
                error: Some("No title found in metadata or filename".to_string()),
            };
        }
    };
    
    let original_title_for_report = Some(current_title_string_value.clone()); // Store for reporting

    let normalized_title_value = to_title_case(&current_title_string_value); // Borrow `current_title_string_value`
    let changed = current_title_string_value != normalized_title_value;

    TitleNormalizationReport {
        original_path, // Move `original_path` here, no active borrows
        original_title: original_title_for_report,
        normalized_title: Some(normalized_title_value),
        changed,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::models::TrackMetadata;
    use crate::core::domain::models::MetadataSource;

    #[test]
    fn test_to_title_case() {
        assert_eq!(to_title_case("hello world"), "Hello World");
        assert_eq!(to_title_case("HELLO WORLD"), "Hello World");
        assert_eq!(to_title_case("hello-world_test"), "Hello-World_Test");
        assert_eq!(to_title_case("  leading  spaces  "), "  Leading  Spaces  ");
        assert_eq!(to_title_case(""), "");
        assert_eq!(to_title_case("a"), "A");
        assert_eq!(to_title_case("already Title Case"), "Already Title Case");
    }

    #[test]
    fn test_normalize_genre_rock_aliases() {
        assert_eq!(normalize_genre("rock and roll"), Some("Rock".to_string()));
        assert_eq!(normalize_genre("rock & roll"), Some("Rock".to_string()));
        assert_eq!(normalize_genre("rock'n'roll"), Some("Rock".to_string()));
    }

    #[test]
    fn test_normalize_genre_hip_hop_aliases() {
        assert_eq!(normalize_genre("hip hop"), Some("Hip-Hop".to_string()));
        assert_eq!(normalize_genre("hip-hop"), Some("Hip-Hop".to_string()));
        assert_eq!(normalize_genre("hiphop"), Some("Hip-Hop".to_string()));
    }

    #[test]
    fn test_normalize_genre_electronic_aliases() {
        assert_eq!(
            normalize_genre("electronic"),
            Some("Electronic".to_string())
        );
        assert_eq!(
            normalize_genre("electronica"),
            Some("Electronic".to_string())
        );
        assert_eq!(normalize_genre("edm"), Some("Electronic".to_string()));
    }

    #[test]
    fn test_normalize_genre_standard_genres() {
        assert_eq!(normalize_genre("rock"), Some("Rock".to_string()));
        assert_eq!(normalize_genre("jazz"), Some("Jazz".to_string()));
        assert_eq!(normalize_genre("classical"), Some("Classical".to_string()));
    }

    #[test]
    fn test_normalize_genre_case_insensitive() {
        assert_eq!(normalize_genre("ROCK"), Some("Rock".to_string()));
        assert_eq!(normalize_genre("Jazz"), Some("Jazz".to_string()));
        assert_eq!(
            normalize_genre("ELECTRONIC"),
            Some("Electronic".to_string())
        );
    }

    #[test]
    fn test_normalize_genre_slash_separated() {
        assert_eq!(
            normalize_genre("rock/electronic"),
            Some("Rock/Electronic".to_string())
        );
        assert_eq!(
            normalize_genre("hip hop / soul"),
            Some("Hip-Hop/Soul".to_string())
        );
    }

    #[test]
    fn test_normalize_genre_unknown() {
        assert_eq!(
            normalize_genre("Custom Genre"),
            Some("Custom Genre".to_string())
        );
        assert_eq!(normalize_genre(""), None);
    }

    #[test]
    fn test_normalize_genre_jazz_aliases() {
        assert_eq!(normalize_genre("smooth jazz"), Some("Jazz".to_string()));
        assert_eq!(normalize_genre("bebop"), Some("Jazz".to_string()));
        assert_eq!(normalize_genre("swing"), Some("Jazz".to_string()));
    }

    #[test]
    fn test_normalize_single_track_title_casing() {
        let track = Track {
            file_path: PathBuf::from("/music/artist/album/track.flac"),
            metadata: TrackMetadata {
                title: Some(MetadataValue::user_set("a test title".to_string())),
                artist: None, album: None, album_artist: None, track_number: None, disc_number: None, year: None, genre: None, duration: None, format: "flac".to_string(), path: PathBuf::from(""),
            },
            checksum: None,
        };
        let report = normalize_single_track(track);
        assert!(report.changed);
        assert_eq!(report.original_title, Some("a test title".to_string()));
        assert_eq!(report.normalized_title, Some("A Test Title".to_string()));
        assert!(report.error.is_none());
    }

    #[test]
    fn test_normalize_single_track_no_change() {
        let track = Track {
            file_path: PathBuf::from("/music/artist/album/track.flac"),
            metadata: TrackMetadata {
                title: Some(MetadataValue::user_set("Already Normalized".to_string())),
                artist: None, album: None, album_artist: None, track_number: None, disc_number: None, year: None, genre: None, duration: None, format: "flac".to_string(), path: PathBuf::from(""),
            },
            checksum: None,
        };
        let report = normalize_single_track(track);
        assert!(!report.changed);
        assert_eq!(report.original_title, Some("Already Normalized".to_string()));
        assert_eq!(report.normalized_title, Some("Already Normalized".to_string()));
        assert!(report.error.is_none());
    }

    #[test]
    fn test_normalize_single_track_title_inferred_from_filename() {
        let track = Track {
            file_path: PathBuf::from("/music/file_without_title.flac"),
            metadata: TrackMetadata {
                title: None, // Explicitly no title in metadata
                artist: None, album: None, album_artist: None, track_number: None, disc_number: None, year: None, genre: None, duration: None, format: "flac".to_string(), path: PathBuf::from(""),
            },
            checksum: None,
        };
        let report = normalize_single_track(track);
        assert!(report.changed); // Expect change because "file_without_title" is normalized
        assert_eq!(report.original_title, Some("file_without_title".to_string())); // This is the title derived from filename that was processed
        assert_eq!(report.normalized_title, Some("File_Without_Title".to_string())); // Normalized value
        assert!(report.error.is_none());
    }

    // Add a new test case for when literally no title can be found (e.g., empty file_stem)
    #[test]
    fn test_normalize_single_track_no_title_at_all() {
        let track = Track {
            file_path: PathBuf::from("/music/.flac"), // File with no stem
            metadata: TrackMetadata {
                title: None,
                artist: None, album: None, album_artist: None, track_number: None, disc_number: None, year: None, genre: None, duration: None, format: "flac".to_string(), path: PathBuf::from(""),
            },
            checksum: None,
        };
        let report = normalize_single_track(track);
        assert!(!report.changed);
        assert_eq!(report.original_title, None);
        assert_eq!(report.normalized_title, None);
        assert!(report.error.is_some());
        assert_eq!(
            report.error.unwrap(),
            "No meaningful title found in metadata or filename".to_string()
        );
    }
}