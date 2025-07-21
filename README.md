# movcat

A fast, lossless mov file concatenation tool written in Rust.

## Features

- **Lossless concatenation**: No re-encoding, preserves original quality
- **Single binary**: Self-contained executable with no external dependencies except FFmpeg
- **Smart validation**: Analyzes input files and warns about compatibility issues
- **Detailed reporting**: Shows file information including duration, tracks, and format details
- **Cross-platform**: Works on macOS, Linux, and Windows

## Installation

### Prerequisites

movcat requires FFmpeg to be installed on your system:

- **macOS**: `brew install ffmpeg`
- **Ubuntu/Debian**: `sudo apt install ffmpeg`
- **Windows**: Download from [ffmpeg.org](https://ffmpeg.org/download.html)

### Build from source

```bash
git clone https://github.com/katsuma/movcat.git
cd movcat
cargo build --release
```

The binary will be available at `./target/release/movcat`.

## Usage

```bash
movcat -o output.mov input1.mov input2.mov input3.mov
```

### Options

- `-o, --output <OUTPUT>`: Output file path (required)
- `-h, --help`: Show help message

### Examples

```bash
# Concatenate multiple mov files
movcat -o combined.mov part1.mov part2.mov part3.mov

# Use wildcards to concatenate all mov files in a directory
movcat -o combined.mov videos/*.mov

# Mix wildcards and specific files
movcat -o final.mov intro.mov content_*.mov outro.mov

# Use complex patterns
movcat -o output.mov "path/to/videos/episode_[0-9][0-9].mov"
```

### Wildcard Support

movcat supports glob patterns for input files:
- `*.mov` - All mov files in current directory
- `videos/*.mov` - All mov files in videos directory
- `part_?.mov` - Files like part_1.mov, part_a.mov, etc.
- `episode_[0-9][0-9].mov` - Files like episode_01.mov, episode_23.mov, etc.

Files matched by patterns are automatically sorted for consistent ordering.

## How it works

1. **Analysis**: Reads and analyzes each input mov file using the `mov` Rust crate
2. **Validation**: Checks file compatibility and warns about potential issues
3. **Concatenation**: Uses FFmpeg's concat demuxer for lossless joining
4. **Output**: Creates a single mov file with all input content

## File Analysis

movcat provides detailed information about each input file:

```
File: "input1.mov"
  Duration: 120.5s
  Tracks: 2 (Video: 1, Audio: 1)
  Major Brand: isom

File: "input2.mov"
  Duration: 95.2s
  Tracks: 2 (Video: 1, Audio: 1)
  Major Brand: isom
```

## Compatibility

The tool performs compatibility checks and warns about:
- Different major brands (container formats)
- Different timescales
- Missing video or audio tracks

## Technical Details

- Built with Rust for performance and safety
- Uses the `mov` crate for mov file analysis
- Leverages FFmpeg's proven concat demuxer for reliable concatenation
- Handles various mov container formats and codecs

## Error Handling

movcat provides clear error messages for common issues:
- Missing input files
- Invalid mov files
- FFmpeg not installed
- Disk space or permission issues

## Performance

- Fast analysis using Rust's zero-cost abstractions
- Efficient concatenation via FFmpeg's optimized algorithms
- Minimal memory usage for large files

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## Changelog

### v0.1.0
- Initial release
- Basic mov file concatenation
- File analysis and validation
- FFmpeg integration
- Cross-platform support
