#!/usr/bin/env python3
"""
Create test MP3 files for music-chore testing.
Creates minimal MP3 files with proper ID3 tags.
"""

import os
import struct
from pathlib import Path

def create_minimal_mp3(output_path: Path, title: str = "Test Track", 
                      artist: str = "Test Artist", album: str = "Test Album",
                      track_number: str = "1", year: str = "2024", 
                      genre: str = "Test Genre"):
    """Create a minimal test MP3 file with ID3v2.3 tags."""
    
    # Create directories if needed
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    # Create ID3v2.3 tag header and frames
    frames = []
    
    def add_frame(frame_id: str, text: str):
        """Add an ID3 frame."""
        data = text.encode('utf-8')
        frame = frame_id.encode('ascii') + struct.pack('>I', len(data) + 1)[1:] + b'\x00' + data
        frames.append(frame)
    
    # Add metadata frames
    add_frame('TIT2', title)    # Title
    add_frame('TPE1', artist)   # Artist
    add_frame('TALB', album)    # Album  
    add_frame('TRCK', track_number)  # Track Number
    add_frame('TDRC', year)      # Year
    add_frame('TCON', genre)     # Genre
    add_frame('TPE2', artist)    # Album Artist
    
    frames_data = b''.join(frames)
    
    # ID3v2.3 header: "ID3" + version + flags + size
    header = b'ID3' + b'\x03\x00' + b'\x00' + struct.pack('>I', len(frames_data))[1:]
    
    # Create minimal MP3 frame (very short silent audio)
    mp3_frame = b'\xff\xfb\x90\x00' + b'\x00' * 100  # Simple MP3 header + silent data
    
    # Write file
    with open(output_path, 'wb') as f:
        f.write(header)
        f.write(frames_data)
        f.write(mp3_frame)
    
    print(f"Created test MP3: {output_path}")

def create_test_mp3_files():
    """Create various test MP3 files."""
    base_dir = Path("tests/fixtures/mp3")
    
    # Simple test file with metadata
    create_minimal_mp3(
        base_dir / "simple" / "track1.mp3",
        title="Test Track 1",
        artist="Test Artist", 
        album="Test Album",
        track_number="1",
        year="2024",
        genre="Test Genre"
    )
    
    create_minimal_mp3(
        base_dir / "simple" / "track2.mp3",
        title="Test Track 2",
        artist="Test Artist",
        album="Test Album", 
        track_number="2",
        year="2024",
        genre="Test Genre"
    )
    
    # Nested structure like the FLAC tests
    create_minimal_mp3(
        base_dir / "nested" / "The Beatles" / "Abbey Road" / "01 - Come Together.mp3",
        title="Come Together",
        artist="The Beatles",
        album="Abbey Road",
        track_number="1", 
        year="1969",
        genre="Rock"
    )
    
    create_minimal_mp3(
        base_dir / "nested" / "The Beatles" / "Abbey Road" / "02 - Something.mp3",
        title="Something", 
        artist="The Beatles",
        album="Abbey Road",
        track_number="2",
        year="1969", 
        genre="Rock"
    )
    
    # Unicode test
    create_minimal_mp3(
        base_dir / "unicode" / "José González" / "album" / "track.mp3",
        title="Crosses",
        artist="José González",
        album="In Our Nature",
        track_number="1",
        year="2007",
        genre="Folk"
    )

if __name__ == "__main__":
    create_test_mp3_files()