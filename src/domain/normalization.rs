//! Normalization logic for music metadata.

use crate::domain::metadata::Metadata;

/// Normalizes metadata by applying consistent formatting rules
pub fn normalize_metadata(metadata: &mut Metadata) {
    // Normalize artist names (capitalize first letter, lowercase the rest)
    if let Some(artist) = metadata.inferred.get("artist") {
        let normalized_artist = capitalize_words(artist);
        metadata.inferred.insert("artist".to_string(), normalized_artist);
    }

    // Normalize album titles (capitalize first letter, lowercase the rest)
    if let Some(album) = metadata.inferred.get("album") {
        let normalized_album = capitalize_words(album);
        metadata.inferred.insert("album".to_string(), normalized_album);
    }

    // Normalize track titles (capitalize first letter, lowercase the rest)
    if let Some(title) = metadata.inferred.get("title") {
        let normalized_title = capitalize_words(title);
        metadata.inferred.insert("title".to_string(), normalized_title);
    }
}

/// Capitalizes the first letter of each word in a string
fn capitalize_words(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in input.chars() {
        if ch.is_whitespace() {
            result.push(ch);
            capitalize_next = true;
        } else if capitalize_next && ch.is_alphabetic() {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
            capitalize_next = false;
        } else {
            result.push(ch);
            capitalize_next = false;
        }
    }

    result
}

/// Normalizes a single metadata field
pub fn normalize_field(field: &str, value: &str) -> String {
    match field {
        "artist" | "album" | "title" => capitalize_words(value),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_metadata() {
        let mut metadata = Metadata::new();
        metadata.inferred.insert("artist".to_string(), "the beatles".to_string());
        metadata.inferred.insert("album".to_string(), "abbey road".to_string());
        metadata.inferred.insert("title".to_string(), "come together".to_string());

        normalize_metadata(&mut metadata);

        assert_eq!(metadata.inferred.get("artist").unwrap(), "The Beatles");
        assert_eq!(metadata.inferred.get("album").unwrap(), "Abbey Road");
        assert_eq!(metadata.inferred.get("title").unwrap(), "Come Together");
    }

    #[test]
    fn test_normalize_field() {
        assert_eq!(normalize_field("artist", "the beatles"), "The Beatles");
        assert_eq!(normalize_field("album", "abbey road"), "Abbey Road");
        assert_eq!(normalize_field("title", "come together"), "Come Together");
        assert_eq!(normalize_field("genre", "rock"), "rock");
    }

    #[test]
    fn test_capitalize_words() {
        assert_eq!(capitalize_words("the beatles"), "The Beatles");
        assert_eq!(capitalize_words("abbey road"), "Abbey Road");
        assert_eq!(capitalize_words("come together"), "Come Together");
        assert_eq!(capitalize_words("hello world"), "Hello World");
    }
}