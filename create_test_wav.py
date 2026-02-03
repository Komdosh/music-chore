#!/usr/bin/env python3
"""
Create test WAV files for music-chore testing.
Creates minimal WAV files with metadata in INFO chunks.
"""

import os
import struct
from pathlib import Path

def create_minimal_wav(output_path: Path, title: str = "Test Track", 
                      artist: str = "Test Artist", album: str = "Test Album",
                      track_number: str = "1", year: str = "2024", 
                      genre: str = "Test Genre"):
    """Create a minimal test WAV file with INFO chunks in LIST chunk."""
    
    # Create directories if needed
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    # WAV format parameters
    sample_rate = 44100
    channels = 2
    bits_per_sample = 16
    bytes_per_sample = bits_per_sample // 8
    
    # Create minimal audio data (100 samples of silence)
    num_samples = 100
    audio_data_size = num_samples * channels * bytes_per_sample
    audio_data = b'\x00' * audio_data_size
    
    # Create INFO list chunk with metadata
    info_chunks = []
    
    def add_info_chunk(tag: str, text: str):
        """Add an INFO chunk."""
        data = text.encode('utf-8') + b'\x00'  # Null-terminated
        # Ensure even length (INFO chunks must be word-aligned)
        if len(data) % 2 != 0:
            data += b'\x00'
        info_chunks.append((tag, data))
    
    # Add metadata INFO chunks
    add_info_chunk('INAM', title)    # Title
    add_info_chunk('IART', artist)   # Artist
    add_info_chunk('IPRD', album)    # Album/Product
    add_info_chunk('ITRK', track_number)  # Track Number
    add_info_chunk('ICRD', year)     # Creation Date/Year
    add_info_chunk('IGNR', genre)    # Genre
    
    # Build LIST chunk
    list_data = b'INFO'
    for tag, data in info_chunks:
        list_data += tag.encode('ascii') + struct.pack('<I', len(data)) + data
    
    # Ensure LIST chunk is word-aligned
    if len(list_data) % 2 != 0:
        list_data += b'\x00'
    
    list_chunk_size = len(list_data)
    list_chunk = b'LIST' + struct.pack('<I', list_chunk_size) + list_data
    
    # Calculate sizes
    fmt_chunk_size = 16  # Standard for PCM
    data_chunk_size = audio_data_size
    
    # Build WAV file
    # RIFF header
    riff_size = 4 + (8 + fmt_chunk_size) + (8 + list_chunk_size) + (8 + data_chunk_size)
    riff_header = b'RIFF' + struct.pack('<I', riff_size) + b'WAVE'
    
    # fmt chunk (PCM format)
    fmt_chunk = (
        b'fmt ' + 
        struct.pack('<I', fmt_chunk_size) +  # Chunk size
        struct.pack('<H', 1) +              # Audio format (1 = PCM)
        struct.pack('<H', channels) +        # Number of channels
        struct.pack('<I', sample_rate) +     # Sample rate
        struct.pack('<I', sample_rate * channels * bytes_per_sample) +  # Byte rate
        struct.pack('<H', channels * bytes_per_sample) +                # Block align
        struct.pack('<H', bits_per_sample)      # Bits per sample
    )
    
    # data chunk
    data_chunk = b'data' + struct.pack('<I', data_chunk_size) + audio_data
    
    # Write file
    with open(output_path, 'wb') as f:
        f.write(riff_header)
        f.write(fmt_chunk)
        f.write(list_chunk)
        f.write(data_chunk)
    
    print(f"Created test WAV: {output_path}")

def create_test_wav_files():
    """Create various test WAV files."""
    base_dir = Path("tests/fixtures/wav")
    
    # Simple test file with metadata
    create_minimal_wav(
        base_dir / "simple" / "track1.wav",
        title="Test Song",
        artist="Test Artist", 
        album="Test Album",
        track_number="1",
        year="2023",
        genre="Test Genre"
    )
    
    create_minimal_wav(
        base_dir / "simple" / "track2.wav",
        title="Test Song 2",
        artist="Test Artist",
        album="Test Album", 
        track_number="2",
        year="2023",
        genre="Test Genre"
    )
    
    # Nested structure like FLAC/MP3 tests
    create_minimal_wav(
        base_dir / "nested" / "The Beatles" / "Abbey Road" / "01 - Come Together.wav",
        title="Come Together",
        artist="The Beatles",
        album="Abbey Road",
        track_number="1", 
        year="1969",
        genre="Rock"
    )
    
    create_minimal_wav(
        base_dir / "nested" / "The Beatles" / "Abbey Road" / "02 - Something.wav",
        title="Something", 
        artist="The Beatles",
        album="Abbey Road",
        track_number="2",
        year="1969", 
        genre="Rock"
    )
    
    # Unicode test
    create_minimal_wav(
        base_dir / "unicode" / "José González" / "album" / "track.wav",
        title="Crosses",
        artist="José González",
        album="In Our Nature",
        track_number="1",
        year="2007",
        genre="Folk"
    )

if __name__ == "__main__":
    create_test_wav_files()