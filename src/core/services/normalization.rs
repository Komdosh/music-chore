//! Text normalization services.

use crate::core::domain::models::{OperationResult, Track};
use crate::adapters::audio_formats as formats;
use std::path::{Path, PathBuf};
use crate::MetadataValue;

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
        &[
            "country",
            "country music",
            "country & western",
            "c&w",
            "nashville",
        ],
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

pub fn normalize_genres_in_library(path: &Path, dry_run: bool) -> Result<String, String> {
    let mut out = String::new();

    let tracks = if path.is_file() {
        vec![
            formats::read_metadata(path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?,
        ]
    } else if path.is_dir() {
        crate::core::services::scanner::scan_dir(path)
    } else {
        return Err(format!("Path does not exist: {}", path.display()));
    };

    let mut updated_count = 0;
    let mut no_change_count = 0;
    let mut error_count = 0;

    for track in tracks {
        if let Some(ref genre) = track.metadata.genre {
            let normalized = normalize_genre(&genre.value);

            match normalized {
                Some(ref new_genre) if new_genre != &genre.value => {
                    if dry_run {
                        out.push_str(&format!(
                            "DRY RUN: Would normalize '{}' -> '{}' in {}\n",
                            genre.value,
                            new_genre,
                            track.file_path.display()
                        ));
                    } else {
                        let mut updated_metadata = track.metadata.clone();
                        updated_metadata.genre = Some(
                            MetadataValue::user_set(new_genre.clone()),
                        );

                        match formats::write_metadata(&track.file_path, &updated_metadata) {
                            Ok(()) => {
                                out.push_str(&format!(
                                    "NORMALIZED: '{}' -> '{}' in {}\n",
                                    genre.value,
                                    new_genre,
                                    track.file_path.display()
                                ));
                            }
                            Err(e) => {
                                out.push_str(&format!(
                                    "ERROR: {} in {}\n",
                                    e,
                                    track.file_path.display()
                                ));
                                error_count += 1;
                                continue;
                            }
                        }
                    }
                    updated_count += 1;
                }
                Some(_) => {
                    no_change_count += 1;
                }
                None => {
                    out.push_str(&format!(
                        "ERROR: Could not normalize genre '{}' in {}\n",
                        genre.value,
                        track.file_path.display()
                    ));
                    error_count += 1;
                }
            }
        }
    }

    out.push_str(&format!(
        "\nSummary: {} normalized, {} no change, {} errors\n",
        updated_count, no_change_count, error_count
    ));

    Ok(out)
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
pub fn normalize(path: PathBuf, dry_run: bool) -> Result<String, String> {
    let mut out = String::new();

    match normalize_track_titles_with_options(&path, dry_run) {
        Ok(results) => {
            for result in results {
                match result {
                    OperationResult::Updated {
                        track,
                        old_title,
                        new_title,
                    } => {
                        if dry_run {
                            out.push_str(&format!(
                                "DRY RUN: Would normalize '{}' -> '{}' in {}\n",
                                track.file_path.display(),
                                old_title,
                                new_title
                            ));
                        } else {
                            out.push_str(&format!(
                                "NORMALIZED: '{}' -> '{}' in {}\n",
                                track.file_path.display(),
                                old_title,
                                new_title
                            ));
                        }
                    }

                    OperationResult::NoChange { track } => {
                        if !dry_run {
                            out.push_str(&format!(
                                "NO CHANGE: Title already title case in {}\n",
                                track.file_path.display()
                            ));
                        }
                    }

                    OperationResult::Error { track, error } => {
                        out.push_str(&format!(
                            "ERROR: {} in {}\n",
                            error,
                            track.file_path.display()
                        ));
                    }
                }
            }

            Ok(out)
        }

        Err(e) => Err(format!("Error normalizing titles: {}\n", e)),
    }
}
fn normalize_track_titles_with_options(
    path: &Path,
    dry_run: bool,
) -> Result<Vec<OperationResult>, String> {
    let mut results = Vec::new();

    // Check if path is a file or directory
    if path.is_file() {
        // Single file
        let track = formats::read_metadata(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        results.push(normalize_single_track(track, dry_run));
    } else if path.is_dir() {
        // Directory - scan for supported audio files
        let tracks = crate::core::services::scanner::scan_dir(path);
        for track in tracks {
            results.push(normalize_single_track(track, dry_run));
        }
    } else {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    Ok(results)
}

/// Normalize a single track's title
fn normalize_single_track(track: Track, dry_run: bool) -> OperationResult {
    let current_title = match &track.metadata.title {
        Some(title) => &title.value,
        None => {
            // If no embedded title, try to extract from filename
            if let Some(file_stem) = track.file_path.file_stem().and_then(|s| s.to_str()) {
                // Extract title from filename patterns like "01 - Title" or "Title"
                let extracted_title = if let Some(pos) = file_stem.find(" - ") {
                    // Pattern: "01 - Title" or "Artist - Title"
                    let title_part = &file_stem[pos + 3..];
                    if !title_part.trim().is_empty() {
                        title_part.trim()
                    } else {
                        // If after " - " is empty, use the whole filename
                        file_stem
                    }
                } else {
                    // No " - " pattern, use the whole filename stem
                    file_stem
                };

                if !extracted_title.is_empty() {
                    extracted_title
                } else {
                    return OperationResult::Error {
                        track,
                        error: "No title found".to_string(),
                    };
                }
            } else {
                return OperationResult::Error {
                    track,
                    error: "No title found".to_string(),
                };
            }
        }
    };

    let normalized_title = to_title_case(current_title);
    let old_title = current_title.to_string();

    // Check if title needs to be changed
    if current_title == &normalized_title {
        return OperationResult::NoChange { track };
    }

    if dry_run {
        // Just return what would be changed
        OperationResult::Updated {
            track,
            old_title,
            new_title: normalized_title,
        }
    } else {
        // Actually update the metadata
        let mut updated_metadata = track.metadata.clone();
        updated_metadata.title = Some(crate::core::domain::models::MetadataValue::user_set(
            normalized_title,
        ));

        match formats::write_metadata(&track.file_path, &updated_metadata) {
            Ok(()) => OperationResult::Updated {
                track,
                old_title,
                new_title: updated_metadata.title.unwrap().value,
            },
            Err(e) => OperationResult::Error {
                track,
                error: format!("Failed to write metadata: {}", e),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
