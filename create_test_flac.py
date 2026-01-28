#!/usr/bin/env python3
"""
Generate a real FLAC file with metadata for testing.
Requires: pyflac (pip install pyflac)
"""
import subprocess
import sys
import os

def check_dependencies():
    try:
        import flac
        return True
    except ImportError:
        print("Installing pyflac...")
        subprocess.run([sys.executable, "-m", "pip", "install", "pyflac"], check=True)
        import flac
        return True

def create_flac_with_metadata(output_path, metadata):
    """Create a FLAC file with specified metadata."""
    from flac import FLAC
    
    # Create a simple sine wave audio data (minimal)
    import numpy as np
    
    # Generate 1 second of 440Hz sine wave at 44100Hz
    sample_rate = 44100
    duration = 1.0
    frequency = 440.0
    
    t = np.linspace(0, duration, int(sample_rate * duration), False)
    audio_data = (np.sin(2 * np.pi * frequency * t) * 32767).astype(np.int16)
    
    # Create FLAC file
    flac_file = FLAC()
    flac_file.sample_rate = sample_rate
    flac_file.channels = 1
    flac_file.bits_per_sample = 16
    
    # Add metadata
    for key, value in metadata.items():
        flac_file[key] = str(value)
    
    # Write the file
    flac_file.save(output_path, audio_data)
    print(f"Created FLAC file: {output_path}")

def main():
    if not check_dependencies():
        print("Failed to install dependencies")
        sys.exit(1)
    
    # Create output directory
    output_dir = "tests/fixtures/flac/metadata"
    os.makedirs(output_dir, exist_ok=True)
    
    # Create test FLAC with metadata
    metadata = {
        "TITLE": "Test Song",
        "ARTIST": "Test Artist", 
        "ALBUM": "Test Album",
        "ALBUMARTIST": "Test Album Artist",
        "TRACKNUMBER": "1",
        "DISCNUMBER": "1", 
        "DATE": "2023",
        "GENRE": "Test Genre"
    }
    
    output_path = os.path.join(output_dir, "test_with_metadata.flac")
    create_flac_with_metadata(output_path, metadata)
    
    # Print metadata for verification
    print(f"\nCreated FLAC file with metadata:")
    for key, value in metadata.items():
        print(f"  {key}: {value}")

if __name__ == "__main__":
    main()