use crate::core::domain::with_schema_version;
use crate::core::services::scanner::{scan_dir, scan_dir_with_metadata};
use crate::{build_library_hierarchy, Library, MetadataSource, Track, TrackNode};
use serde_json::to_string_pretty;
use std::collections::BTreeMap;
use std::path::Path;

/// Directory tree node
#[derive(Debug)]
struct DirNode {
    name: String,
    subdirs: BTreeMap<String, DirNode>,
    tracks: Vec<Track>,
}

/// Build directory tree from tracks
fn build_dir_tree(base_path: &Path, tracks: Vec<Track>) -> DirNode {
    let mut root = DirNode {
        name: base_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("root")
            .to_string(),
        subdirs: BTreeMap::new(),
        tracks: Vec::new(),
    };

    for track in tracks {
        let rel_path = track
            .file_path
            .strip_prefix(base_path)
            .unwrap_or(&track.file_path);
        let components: Vec<&str> = rel_path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        if components.len() > 1 {
            let (dirs, _filename) = components.split_at(components.len() - 1);
            let mut current = &mut root;

            for dir in dirs {
                if !current.subdirs.contains_key(*dir) {
                    current.subdirs.insert(
                        dir.to_string(),
                        DirNode {
                            name: dir.to_string(),
                            subdirs: BTreeMap::new(),
                            tracks: Vec::new(),
                        },
                    );
                }
                current = current.subdirs.get_mut(*dir).unwrap();
            }

            current.tracks.push(track);
        }
    }

    root
}

/// Format directory tree as string
fn format_dir_tree(node: &DirNode, indent: &str, is_last: bool) -> String {
    let mut output = String::new();

    let prefix = if indent.is_empty() {
        String::new()
    } else if is_last {
        "└──".to_string()
    } else {
        "├──".to_string()
    };

    let full_prefix = format!("{}{}", indent, prefix);

    if !node.name.is_empty() && indent.is_empty() {
        output.push_str(&format!("📁 {}\n", node.name));
    } else if !node.name.is_empty() {
        output.push_str(&format!("{} 📂 {}\n", full_prefix, node.name));
    }

    let child_indent = if indent.is_empty() {
        String::new()
    } else if is_last {
        format!("{}   ", indent)
    } else {
        format!("{}│  ", indent)
    };

    let subdir_count = node.subdirs.len();
    let mut index = 0;

    for subdir in node.subdirs.values() {
        index += 1;
        let sub_indent = if indent.is_empty() {
            if index == subdir_count {
                "   ".to_string()
            } else {
                "│  ".to_string()
            }
        } else {
            child_indent.clone()
        };
        output.push_str(&format_dir_tree(subdir, &sub_indent, index == subdir_count));
    }

    for (i, track) in node.tracks.iter().enumerate() {
        let is_last_track = i == node.tracks.len() - 1 && subdir_count == 0;
        let track_prefix = if is_last_track {
            format!("{}└─── 🎵", child_indent)
        } else {
            format!("{}├─── 🎵", child_indent)
        };

        let track_info = format_track_info_for_dir(track);
        let filename = track
            .file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        output.push_str(&format!("{}   {} {}\n", track_prefix, filename, track_info));
    }

    output
}

/// Format track info for directory tree view
fn format_track_info_for_dir(track: &Track) -> String {
    let mut info = Vec::new();

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
        .unwrap_or(&MetadataSource::FolderInferred)
    {
        MetadataSource::Embedded => "🎯",
        MetadataSource::FolderInferred => "🤖",
        MetadataSource::UserEdited => "👤",
    };

    format!("[{}] {}", source, info.join(" | "))
}

/// Print library tree in human-readable format (preserving directory structure)
pub fn format_tree_output(base_path: &Path) -> String {
    let tracks = scan_dir(base_path);
    let dir_tree = build_dir_tree(base_path, tracks);
    let mut output = format_dir_tree(&dir_tree, "", true);

    // Print summary
    output.push_str("📊 Library Summary:\n");
    output.push_str(&format!("   Files: {}\n", count_tracks_in_tree(&dir_tree)));
    output.push_str(&format!("   Folders: {}\n", count_dirs_in_tree(&dir_tree)));

    output
}

fn count_tracks_in_tree(node: &DirNode) -> usize {
    let count = node.tracks.len();
    let subdir_count: usize = node.subdirs.values().map(count_tracks_in_tree).sum();
    count + subdir_count
}

fn count_dirs_in_tree(node: &DirNode) -> usize {
    let subdir_count = node.subdirs.len();
    let nested_count: usize = node.subdirs.values().map(count_dirs_in_tree).sum();
    subdir_count + nested_count
}

/// Print library tree in human-readable format (metadata-based, deprecated)
/// Use format_tree_output(base_path) instead for directory-based view
pub fn format_library_output(library: &Library) -> String {
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
        .unwrap_or(&MetadataSource::FolderInferred)
    {
        MetadataSource::Embedded => "🎯",
        MetadataSource::FolderInferred => "🤖",
        MetadataSource::UserEdited => "👤",
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

        out.push('\n');
    }

    out.push_str("=== END METADATA ===\n");

    out
}

pub fn emit_by_path(path: &Path, json: bool) -> Result<String, String> {
    log::info!("emit_by_path called with path: {}", path.display());

    let tracks = match scan_dir_with_metadata(path) {
        Ok(tracks) => tracks,
        Err(e) => return Err(format!("Failed to scan directory: {}", e)),
    };
    log::info!("Found {} tracks", tracks.len());

    let library = build_library_hierarchy(tracks);

    if json {
        let wrapper = with_schema_version(&library);
        match to_string_pretty(&wrapper) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!("Error serializing to JSON: {}", e)),
        }
    } else {
        // Default to structured text output for AI agents
        Ok(emit_structured_output(&library))
    }
}
