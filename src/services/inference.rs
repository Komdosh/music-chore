//! Path-based metadata inference services.

use std::path::Path;

/// Extract artist from a string using common separators
/// Looks for patterns like "Artist - Album" or "Artist – Album"
fn extract_artist_from_name(name: &str) -> Option<String> {
    // Common separators: " - ", " – ", "—"
    let separators = [" - ", " – ", " — "];

    for sep in &separators {
        if let Some((left, right)) = name.split_once(sep) {
            let artist = left.trim();
            let right_part = right.trim();

            // Validation: Artist should not be just a number or year
            if is_valid_artist_name(artist) {
                // Additional check: the right part should look like an album, not a track number
                if looks_like_album_name(right_part) {
                    // Clean up the artist name (remove year suffix like "Artist 2024")
                    let artist_cleaned = clean_artist_name(artist);
                    return Some(artist_cleaned);
                }
            }
        }
    }

    None
}

/// Check if a string looks like a valid artist name (not just a number)
fn is_valid_artist_name(name: &str) -> bool {
    if name.is_empty() || name.len() < 2 {
        return false;
    }

    // Check if it starts with a digit (like "01. Track" or "06. Artist")
    if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }

    // Check if it's just a number (track number, year, etc.)
    if name.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Check if it's a year (4 digits)
    if name.len() == 4 && name.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Should contain at least one letter
    if !name.chars().any(|c| c.is_alphabetic()) {
        return false;
    }

    true
}

/// Check if a string looks like an album name (not just a track number)
fn looks_like_album_name(name: &str) -> bool {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return false;
    }

    // Single digits or just numbers are likely track numbers, not albums
    if trimmed.chars().all(|c| c.is_ascii_digit()) && trimmed.len() <= 2 {
        return false;
    }

    true
}

/// Clean artist name by removing common suffixes like years and format indicators
fn clean_artist_name(name: &str) -> String {
    let mut cleaned = name.trim();

    // First, remove format indicators anywhere in the name: " [FLAC]", " [MP3]", " [WAV]", etc.
    let format_patterns = [
        " [FLAC]", "[FLAC]", " [flac]", "[flac]", " [MP3]", "[MP3]", " [mp3]", "[mp3]", " [WAV]",
        "[WAV]", " [wav]", "[wav]", " [DSF]", "[DSF]", " [dsf]", "[dsf]", " - FLAC", " - flac",
        " - MP3", " - mp3", " - WAV", " - wav", " - DSF", " - dsf",
    ];

    for pattern in &format_patterns {
        if let Some(pos) = cleaned.rfind(pattern) {
            cleaned = cleaned[..pos].trim();
            break; // Only remove one format indicator
        }
    }

    // Match pattern: word + space + 4-digit year
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    if words.len() >= 2 {
        if let Ok(_year) = words.last().unwrap().parse::<u32>() {
            // Last word is a 4-digit number (year)
            if words.last().unwrap().len() == 4 {
                return words[..words.len() - 1].join(" ").trim().to_string();
            }
        }
    }

    cleaned.to_string()
}

/// Extract album from a string using common separators
fn extract_album_from_name(name: &str) -> Option<String> {
    let separators = [" - ", " – ", " — ", "-", "–", "—"];

    for sep in &separators {
        if let Some((_left, right)) = name.split_once(sep) {
            let album = right.trim();
            if !album.is_empty() {
                // Clean up album (remove format suffixes, parentheses with metadata)
                return Some(clean_album_name(album));
            }
        }
    }

    None
}

/// Clean album name by removing format suffixes and metadata in parentheses
fn clean_album_name(name: &str) -> String {
    let mut cleaned = name.trim();

    // First pass: Remove common format suffixes: " - FLAC", " [FLAC]", " (FLAC)", " - MP3", etc.
    // We do this FIRST so that "One Take (2009) - FLAC" becomes "One Take (2009)"
    let format_patterns = [
        " - FLAC", " [FLAC]", " (FLAC)", "[FLAC]", " - flac", " [flac]", " (flac)", "[flac]",
        " - MP3", " [MP3]", " (MP3)", "[MP3]", " - mp3", " [mp3]", " (mp3)", "[mp3]", " - WAV",
        " [WAV]", " (WAV)", "[WAV]", " - wav", " [wav]", " (wav)", "[wav]", " - DSF", " [DSF]",
        " (DSF)", "[DSF]", " - dsf", " [dsf]", " (dsf)", "[dsf]",
    ];

    for pattern in &format_patterns {
        if let Some(pos) = cleaned.rfind(pattern) {
            cleaned = cleaned[..pos].trim();
            break; // Only remove one format indicator
        }
    }

    // Second pass: Remove parenthetical metadata: "Album (2009, metadata)" -> "Album"
    // Check if there are parentheses in the middle/end
    if let Some(open_idx) = cleaned.find('(') {
        // Find the matching closing paren
        if let Some(close_idx) = cleaned[open_idx..].find(')') {
            // Calculate actual close position
            let close_pos = open_idx + close_idx;
            // Safe to slice between open_idx and close_pos since they are byte positions in the same string
            let paren_content = &cleaned[open_idx + 1..close_pos];

            // Check if this looks like metadata (has commas or contains non-year info)
            // We remove it if it has commas OR if it contains non-digit characters (not just a year)
            if paren_content.contains(',')
                || (paren_content
                    .chars()
                    .any(|c| !c.is_ascii_digit() && c != ' ')
                    && paren_content.len() > 4)
            {
                // Remove everything from the opening paren onwards
                cleaned = cleaned[..open_idx].trim();
            }
        }
    }

    // Third pass: Remove simple year-only parentheses: "Album (2009)" -> "Album"
    // This runs after the metadata removal and format suffix removal
    // Check if string ends with " (YYYY)" pattern (space + parenthesis + 4 digits + parenthesis = 7 chars)
    // Use rfind to safely locate the pattern without byte slicing issues
    if cleaned.len() >= 7 {
        // Look for the pattern " (YYYY)" at the end
        // First check if it ends with ")"
        if cleaned.ends_with(')') {
            // Find the last " (" before the final ")"
            if let Some(open_paren_pos) = cleaned.rfind(" (") {
                // Check if this " (" is followed by exactly 4 digits and then ")"
                let paren_start = open_paren_pos + 2; // Position after " ("
                if paren_start + 4 < cleaned.len() {
                    // Make sure we have 4 digits before ")"
                    let potential_year = &cleaned[paren_start..paren_start + 4];
                    let after_year = &cleaned[paren_start + 4..];

                    // Check if we have 4 digits followed by ")"
                    if potential_year.chars().all(|c| c.is_ascii_digit()) && after_year == ")" {
                        if let Ok(year) = potential_year.parse::<u32>() {
                            if year >= 1900 && year <= 2100 {
                                // Safe to slice at open_paren_pos since we've verified it's followed by ASCII
                                cleaned = cleaned[..open_paren_pos].trim();
                            }
                        }
                    }
                }
            }
        }
    }

    cleaned.to_string()
}

/// Infer artist name from track file path
pub fn infer_artist_from_path(track_path: &Path) -> Option<String> {
    // Strategy 1: Try to extract artist from parent directory name (pattern: "Artist - Album")
    if let Some(parent) = track_path.parent() {
        if let Some(folder_name) = parent.file_name().and_then(|n| n.to_str()) {
            if let Some(artist) = extract_artist_from_name(folder_name) {
                return Some(artist);
            }
        }
    }

    // Strategy 2: Try to extract artist from filename (pattern: "Artist - Title.ext")
    if let Some(filename) = track_path.file_stem().and_then(|n| n.to_str()) {
        if let Some(artist) = extract_artist_from_name(filename) {
            return Some(artist);
        }
    }

    // Strategy 3: Handle common organized structures like Artist/Albums/Album/track
    let components: Vec<&str> = track_path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    // Check for Artist/Albums/Album/track or Artist/Singles & EPs/Album/track structure
    if components.len() >= 4 {
        let grandparent = components[components.len() - 3];
        let potential_artist = components[components.len() - 4];

        // If grandparent is a collection folder, use its parent as artist
        if grandparent == "Albums" || grandparent == "Singles & EPs" || grandparent == "Singles" {
            if !potential_artist.is_empty() {
                let cleaned_artist = clean_artist_name(potential_artist);
                return Some(cleaned_artist);
            }
        }
    }

    // Strategy 4: Legacy fallback - strict Artist/Album/track structure
    if components.len() >= 3 {
        let album_name = components[components.len() - 2];
        let potential_artist = components[components.len() - 3];

        if !potential_artist.is_empty() && !album_name.is_empty() && potential_artist != album_name
        {
            // Clean the artist name to remove format suffixes and year suffixes
            let cleaned_artist = clean_artist_name(potential_artist);
            return Some(cleaned_artist);
        }
    }

    None
}

/// Infer album name from track file path
pub fn infer_album_from_path(track_path: &Path) -> Option<String> {
    // Strategy 1: Extract album from parent directory name
    if let Some(parent) = track_path.parent() {
        if let Some(folder_name) = parent.file_name().and_then(|n| n.to_str()) {
            // Check for "Artist - Album" pattern
            if let Some(album) = extract_album_from_name(folder_name) {
                return Some(album);
            }

            // If no separator found, use the folder name as album (cleaned)
            let cleaned = clean_album_name(folder_name);
            if !cleaned.is_empty() {
                return Some(cleaned);
            }
        }
    }

    // Strategy 2: Try to extract album from filename (pattern: "Artist - Album")
    if let Some(filename) = track_path.file_stem().and_then(|n| n.to_str()) {
        if let Some(album) = extract_album_from_name(filename) {
            return Some(album);
        }
    }

    None
}

/// Infer year from track file path
pub fn infer_year_from_path(track_path: &Path) -> Option<u32> {
    // Strategy 1: Look for year in parent directory name
    if let Some(parent) = track_path.parent() {
        if let Some(folder_name) = parent.file_name().and_then(|n| n.to_str()) {
            // Look for year patterns: "2008 - Album", "Album (2009)", "Artist 2024 - Album"
            if let Some(year) = extract_year_from_name(folder_name) {
                return Some(year);
            }
        }
    }

    // Strategy 2: Look for year in filename
    if let Some(filename) = track_path.file_name().and_then(|n| n.to_str()) {
        if let Some(year) = extract_year_from_name(filename) {
            return Some(year);
        }
    }

    None
}

/// Extract 4-digit year from a string
fn extract_year_from_name(name: &str) -> Option<u32> {
    // Pattern 1: Year at start followed by separator (e.g., "2008 - Album")
    let words: Vec<&str> = name.split_whitespace().collect();
    if !words.is_empty() {
        let first_word = words[0];
        if first_word.len() == 4 {
            if let Ok(year) = first_word.parse::<u32>() {
                if year >= 1900 && year <= 2100 {
                    return Some(year);
                }
            }
        }
    }

    // Pattern 2: Year in parentheses (e.g., "Album (2009)", "Album (2009, metadata)")
    if let Some(open_idx) = name.find("(") {
        if let Some(close_idx) = name[open_idx..].find(")") {
            let paren_content = &name[open_idx + 1..open_idx + close_idx];
            // Split by comma in case there are multiple items
            for part in paren_content.split(',') {
                let part = part.trim();
                if part.len() == 4 {
                    if let Ok(year) = part.parse::<u32>() {
                        if year >= 1900 && year <= 2100 {
                            return Some(year);
                        }
                    }
                }
            }
        }
    }

    // Pattern 3: Year embedded in name (e.g., "Artist 2024 - Album")
    for word in name.split_whitespace() {
        if word.len() == 4 {
            if let Ok(year) = word.parse::<u32>() {
                if year >= 1900 && year <= 2100 {
                    return Some(year);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_clean_artist_name() {
        // Basic cleanup
        assert_eq!(clean_artist_name("Artist Name"), "Artist Name");

        // Remove format suffixes with brackets
        assert_eq!(clean_artist_name("Artist Name [FLAC]"), "Artist Name");
        assert_eq!(clean_artist_name("Artist Name[FLAC]"), "Artist Name");
        assert_eq!(clean_artist_name("Artist Name [MP3]"), "Artist Name");
        assert_eq!(clean_artist_name("Artist Name[WAV]"), "Artist Name");

        // Remove format suffixes with dash
        assert_eq!(clean_artist_name("Artist Name - FLAC"), "Artist Name");
        assert_eq!(clean_artist_name("Artist Name - MP3"), "Artist Name");

        // Remove year suffix
        assert_eq!(clean_artist_name("Artist 2024"), "Artist");
        assert_eq!(clean_artist_name("Artist 1999"), "Artist");

        // Combined cleanup: format + year
        assert_eq!(clean_artist_name("Artist 2024 - FLAC"), "Artist");
        assert_eq!(clean_artist_name("Artist Name [FLAC] 2023"), "Artist Name");
    }

    #[test]
    fn test_extract_artist_from_name() {
        // Standard pattern
        assert_eq!(
            extract_artist_from_name("Entering - Album"),
            Some("Entering".to_string())
        );

        // Pattern with year suffix that should be cleaned
        assert_eq!(
            extract_artist_from_name("Entering 2024 - Album"),
            Some("Entering".to_string())
        );

        // Pattern with dash and spaces
        assert_eq!(
            extract_artist_from_name("Some guy - Second Take"),
            Some("Some guy".to_string())
        );
    }

    #[test]
    fn test_extract_album_from_name() {
        // Standard pattern
        assert_eq!(
            extract_album_from_name("Second - Album"),
            Some("Album".to_string())
        );

        // Pattern with format suffix
        assert_eq!(
            extract_album_from_name("Some Guy - Second Take (2009) - FLAC"),
            Some("Second Take".to_string())
        );

        // Pattern with year prefix
        assert_eq!(
            extract_album_from_name("2008 - Darkest Mem (2008, Someone)"),
            Some("Darkest Mem".to_string())
        );
    }

    #[test]
    fn test_extract_year_from_name() {
        // Year at start
        assert_eq!(extract_year_from_name("2008 - Album"), Some(2008));

        // Year in parentheses
        assert_eq!(extract_year_from_name("Album (2009)"), Some(2009));
        assert_eq!(extract_year_from_name("Album (2009, metadata)"), Some(2009));

        // Year embedded
        assert_eq!(extract_year_from_name("Artist 2024 - Album"), Some(2024));

        // No year
        assert_eq!(extract_year_from_name("Artist - Album"), None);
    }

    #[test]
    fn test_infer_artist_from_path_real_world_examples() {
        let path = PathBuf::from("/Some/Folder/Artist 2022 - Name/08. Track14.mp3");
        assert_eq!(infer_artist_from_path(&path), Some("Artist".to_string()));

        let path = PathBuf::from("/Some/Folder/Artisssimo 2016 - Name/08. Track14.flac");
        assert_eq!(
            infer_artist_from_path(&path),
            Some("Artisssimo".to_string())
        );

        let path = PathBuf::from("/Some/Artist/Artist 2022 - Name - 08./sdf Track14.mp3");
        assert_eq!(infer_artist_from_path(&path), Some("Artist".to_string()));
    }

    #[test]
    fn test_infer_album_from_path_real_world_examples() {
        let path = PathBuf::from("/Some/Folder/Artist 2022 - Name/08. Track14.mp3");
        assert_eq!(infer_album_from_path(&path), Some("Name".to_string()));

        // Example 2: Folder "Year - Album", filename has "Artist - Album"
        let path = PathBuf::from("/music/2008 - Darkest (2008, Someone, CD 617_08)/Alan - Aw.flac");
        assert_eq!(infer_album_from_path(&path), Some("Darkest".to_string()));

        let path = PathBuf::from("/music/Some guy - Second Take (2009) - FLAC/13 - Alan.flac");
        assert_eq!(
            infer_album_from_path(&path),
            Some("Second Take".to_string())
        );

        let path = PathBuf::from("/music/Some guy [FLAC]/track.flac");
        assert_eq!(infer_album_from_path(&path), Some("Some guy".to_string()));

        // Example 4b: Bracket format suffix no space "Artist[FLAC]"
        let path = PathBuf::from("/music/Artist[FLAC]/track.flac");
        assert_eq!(infer_album_from_path(&path), Some("Artist".to_string()));

        // Example 5: Bracket with separator "Artist - Album [FLAC]"
        let path = PathBuf::from("/music/Artist Name - Album Title [FLAC]/track.flac");
        assert_eq!(
            infer_album_from_path(&path),
            Some("Album Title".to_string())
        );
    }

    #[test]
    fn test_infer_year_from_path_real_world_examples() {
        let path = PathBuf::from("/music/Artist 2024 - Album/Enter.mp3");
        assert_eq!(infer_year_from_path(&path), Some(2024));

        let path = PathBuf::from("/music/2008 - Some Album (2008, something)/went.flac");
        assert_eq!(infer_year_from_path(&path), Some(2008));

        let path = PathBuf::from("/music/Some guy - Second Take (2009) - FLAC/13 - Alan.flac");
        assert_eq!(infer_year_from_path(&path), Some(2009));
    }

    #[test]
    fn test_infer_artist_from_path() {
        // Valid Artist/Album/track.flac structure
        let path = PathBuf::from("The Beatles/Abbey Road/01 - Come Together.flac");
        assert_eq!(
            infer_artist_from_path(&path),
            Some("The Beatles".to_string())
        );

        // Nested directory structure
        let path = PathBuf::from("/music/The Beatles/Abbey Road/01 - Come Together.flac");
        assert_eq!(
            infer_artist_from_path(&path),
            Some("The Beatles".to_string())
        );

        // Deep nested structure
        let path = PathBuf::from("/home/user/music/Genre/Artist/Album/01 - Track.flac");
        assert_eq!(infer_artist_from_path(&path), Some("Artist".to_string()));

        // Artist from filename pattern
        let path = PathBuf::from("music/Unknown Folder/Artist Name - Song Title.mp3");
        assert_eq!(
            infer_artist_from_path(&path),
            Some("Artist Name".to_string())
        );

        // Invalid: Just track.flac - only 2 components, no artist in filename
        let path = PathBuf::from("01 - Come Together.flac");
        assert_eq!(infer_artist_from_path(&path), None);

        // Edge case: Artist and Album have same name
        let path = PathBuf::from("Greatest Hits/Greatest Hits/01 - Song.flac");
        assert_eq!(infer_artist_from_path(&path), None);

        // Unicode artist names
        let path = PathBuf::from("Björk/Vespertine/01 - Cocoon.flac");
        assert_eq!(infer_artist_from_path(&path), Some("Björk".to_string()));

        // Artist with special characters and numbers
        let path = PathBuf::from("The-artist_123/Album (2023)/01 - Track.flac");
        assert_eq!(
            infer_artist_from_path(&path),
            Some("The-artist_123".to_string())
        );
    }

    #[test]
    fn test_infer_album_from_path() {
        // Valid Artist/Album/track.flac structure
        let path = PathBuf::from("The Beatles/Abbey Road/01 - Come Together.flac");
        assert_eq!(infer_album_from_path(&path), Some("Abbey Road".to_string()));

        // Nested directory structure
        let path = PathBuf::from("/music/The Beatles/Abbey Road/01 - Come Together.flac");
        assert_eq!(infer_album_from_path(&path), Some("Abbey Road".to_string()));

        // Album from folder name without separator - uses folder name directly
        let path = PathBuf::from("music/Abbey Road/01 - Song.mp3");
        assert_eq!(infer_album_from_path(&path), Some("Abbey Road".to_string()));

        // Album with spaces and hyphens
        let path = PathBuf::from("Artist/The Dark Side of the Moon/01 - Track.flac");
        assert_eq!(
            infer_album_from_path(&path),
            Some("The Dark Side of the Moon".to_string())
        );
    }
}
