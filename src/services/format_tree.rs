use std::path::PathBuf;
use serde_json::to_string_pretty;
use crate::{build_library_hierarchy, Library, TrackNode};
use crate::services::scanner::scan_dir;

/// Print library tree in human-readable format
///
pub fn format_tree_output(library: &Library) -> String {
    let mut output = String::new();

    for artist in &library.artists {
        output.push_str(&format!("📁 {}\n", artist.name));

        for album in &artist.albums {
            let year_str = album.year.map(|y| format!(" ({})", y)).unwrap_or_default();
            output.push_str(&format!("├── 📂 {}{}\n", album.title, year_str));

            for (i, track) in album.tracks.iter().enumerate() {
                let is_last = i == album.tracks.len() - 1;
                let prefix = if is_last {
                    "└─── 🎵"
                } else {
                    "├─── 🎵"
                };

                let track_info = format_track_info(track);
                output.push_str(&format!(
                    "{}   {} {}\n",
                    prefix,
                    track
                        .file_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                    track_info
                ));
            }
        }
        output.push('\n');
    }

    // Print summary
    output.push_str("📊 Library Summary:\n");
    output.push_str(&format!("   Artists: {}\n", library.total_artists));
    output.push_str(&format!("   Albums: {}\n", library.total_albums));
    output.push_str(&format!("   Tracks: {}\n", library.total_tracks));

    output
}

/// Format track information for tree display
fn format_track_info(track: &TrackNode) -> String {
    let mut info = Vec::new();

    if let Some(duration) = &track.metadata.duration {
        let minutes = (duration.value / 60.0) as u32;
        let seconds = (duration.value % 60.0) as u32;
        info.push(format!("{}:{:02}", minutes, seconds));
    }

    if let Some(track_number) = &track.metadata.track_number {
        info.push(format!("#{}", track_number.value));
    }

    if let Some(format_str) = track.metadata.format.strip_prefix(".") {
        info.push(format_str.to_uppercase());
    } else {
        info.push(track.metadata.format.to_uppercase());
    }

    let source = match track
        .metadata
        .title
        .as_ref()
        .map(|t| &t.source)
        .unwrap_or(&crate::MetadataSource::FolderInferred)
    {
        crate::MetadataSource::Embedded => "🎯",
        crate::MetadataSource::FolderInferred => "🤖",
        crate::MetadataSource::UserEdited => "👤",
    };

    format!("[{}] {}", source, info.join(" | "))
}

/// Emit structured output optimized for AI agents
pub fn emit_structured_output(library: &Library) -> String {
    let mut out = String::new();

    out.push_str("=== MUSIC LIBRARY METADATA ===\n");
    out.push_str(&format!("Total Artists: {}\n", library.total_artists));
    out.push_str(&format!("Total Albums: {}\n", library.total_albums));
    out.push_str(&format!("Total Tracks: {}\n\n", library.total_tracks));

    for artist in &library.artists {
        out.push_str(&format!("ARTIST: {}\n", artist.name));

        for album in &artist.albums {
            let year_str = album.year.map(|y| format!(" ({})", y)).unwrap_or_default();
            out.push_str(&format!("  ALBUM: {}{}\n", album.title, year_str));

            for track in &album.tracks {
                let title = track
                    .metadata
                    .title
                    .as_ref()
                    .map(|t| t.value.as_str())
                    .unwrap_or("[Unknown Title]");

                let duration = track
                    .metadata
                    .duration
                    .as_ref()
                    .map(|d| {
                        let total_seconds = d.value as u64;
                        let minutes = total_seconds / 60;
                        let seconds = total_seconds % 60;
                        format!("{}:{:02}", minutes, seconds)
                    })
                    .unwrap_or_else(|| "0:00".to_string());

                let file_path = track.file_path.to_string_lossy();

                out.push_str(&format!(
                    "    TRACK: \"{}\" | Duration: {} | File: {}\n",
                    title, duration, file_path
                ));
            }
        }

        out.push_str("\n");
    }

    out.push_str("=== END METADATA ===\n");

    out
}

pub fn emit_by_path(path: &PathBuf, json: bool) -> Result<String, String> {
    let tracks = scan_dir(&path);
    let library = build_library_hierarchy(tracks);

    if json {
        match to_string_pretty(&library) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!("Error serializing to JSON: {}", e)),
        }
    } else {
        // Default to structured text output for AI agents
        Ok(emit_structured_output(&library))
    }
}