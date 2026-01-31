#!/usr/bin/env python3
"""
Create a simple FLAC file with lowercase title for testing.
This script uses FLAC command-line tools instead of pyflac.
"""
import os
import subprocess
import tempfile

def create_flac_with_metadata(output_path, metadata):
    """Create a FLAC file using command-line tools."""
    
    # Create a simple WAV file first (using sox or ffmpeg)
    wav_file = tempfile.NamedTemporaryFile(suffix='.wav', delete=False)
    wav_file.close()
    
    try:
        # Create a simple sine wave using sox (if available) or ffmpeg
        try:
            subprocess.run([
                'sox', '-n', '-r', '44100', '-c', '1', wav_file.name,
                'synth', '1', 'sine', '440'
            ], check=True, capture_output=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            try:
                subprocess.run([
                    'ffmpeg', '-f', 'lavfi', '-i', 'sine=frequency=440:duration=1',
                    '-y', wav_file.name
                ], check=True, capture_output=True)
            except (subprocess.CalledProcessError, FileNotFoundError):
                print("Neither sox nor ffmpeg available. Creating empty FLAC file.")
                wav_file.name = None
        
        # Convert to FLAC and add metadata
        if wav_file.name:
            cmd = ['flac', '-f', wav_file.name, '-o', output_path]
        else:
            cmd = ['flac', '-f', '--output-name=output.flac']
        
        # Add metadata tags
        for key, value in metadata.items():
            cmd.extend(['-T', f'{key}={value}'])
        
        if wav_file.name:
            subprocess.run(cmd, check=True, capture_output=True)
        else:
            # Just create an empty FLAC file with metadata
            with open(output_path, 'wb') as f:
                f.write(b'')
            
            # Try to add metadata with metaflac
            for key, value in metadata.items():
                try:
                    subprocess.run([
                        'metaflac', '--set-tag', f'{key}={value}', output_path
                    ], check=True, capture_output=True)
                except subprocess.CalledProcessError:
                    pass
        
        print(f"Created FLAC file: {output_path}")
        
    finally:
        if wav_file.name and os.path.exists(wav_file.name):
            os.unlink(wav_file.name)

def main():
    # Create output directory
    output_dir = "tests/fixtures/flac/lowercase"
    os.makedirs(output_dir, exist_ok=True)
    
    # Create test FLAC with lowercase title (needs normalization)
    metadata = {
        "TITLE": "my awesome song title",  # Lowercase - should be normalized
        "ARTIST": "Test Artist", 
        "ALBUM": "Test Album",
        "ALBUMARTIST": "Test Album Artist",
        "TRACKNUMBER": "1",
        "DISCNUMBER": "1", 
        "DATE": "2023",
        "GENRE": "Test Genre"
    }
    
    output_path = os.path.join(output_dir, "lowercase_title.flac")
    create_flac_with_metadata(output_path, metadata)
    
    print(f"\nCreated test file for normalization testing:")
    print(f"  {output_path} - with lowercase title")
    for key, value in metadata.items():
        print(f"    {key}: {value}")

if __name__ == "__main__":
    main()