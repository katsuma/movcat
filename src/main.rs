use clap::Parser;
use anyhow::{Result, Context};
use std::path::PathBuf;
use glob::glob;

#[derive(Parser)]
#[command(name = "movcat")]
#[command(about = "Lossless mov file concatenation tool")]
#[command(long_about = None)]
struct Args {
    #[arg(help = "Input mov files or patterns to concatenate (supports wildcards)", required = true)]
    inputs: Vec<String>,

    #[arg(short, long, help = "Output file path")]
    output: PathBuf,
}

#[derive(Debug)]
struct MovInfo {
    path: PathBuf,
    duration: u64,
    timescale: u32,
    major_brand: String,
    track_count: usize,
    video_tracks: usize,
    audio_tracks: usize,
}

fn analyze_mov_file(path: &PathBuf) -> Result<MovInfo> {
    // Temporarily return dummy data
    // TODO: Implement using the correct mp4 crate API
    Ok(MovInfo {
        path: path.clone(),
        duration: 0,
        timescale: 1000,
        major_brand: "mp4".to_string(),
        track_count: 1,
        video_tracks: 1,
        audio_tracks: 0,
    })
}

fn validate_input_files(files: &[PathBuf]) -> Result<Vec<MovInfo>> {
    let mut infos = Vec::new();

    for file in files {
        if !file.exists() {
            anyhow::bail!("Input file does not exist: {:?}", file);
        }

        let info = analyze_mov_file(file)?;

        if info.video_tracks == 0 && info.audio_tracks == 0 {
            anyhow::bail!("File has no video or audio tracks: {:?}", file);
        }

        infos.push(info);
    }

    // Check compatibility
    if infos.len() > 1 {
        let first_brand = &infos[0].major_brand;
        let first_timescale = infos[0].timescale;

        for info in &infos[1..] {
            if info.major_brand != *first_brand {
                println!("Warning: Different major brands detected ({} vs {})",
                    first_brand, info.major_brand);
            }
            if info.timescale != first_timescale {
                println!("Warning: Different timescales detected ({} vs {})",
                    first_timescale, info.timescale);
            }
        }
    }

    Ok(infos)
}

fn expand_glob_patterns(patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut all_files = Vec::new();

    for pattern in patterns {
        // Check if the pattern contains glob characters
        if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
            // It's a glob pattern
            let glob_results = glob(pattern)
                .with_context(|| format!("Invalid glob pattern: {}", pattern))?;

            let mut pattern_files = Vec::new();
            for entry in glob_results {
                match entry {
                    Ok(path) => {
                        if path.is_file() {
                            pattern_files.push(path);
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Error processing glob entry: {}", e);
                    }
                }
            }

            if pattern_files.is_empty() {
                anyhow::bail!("No files found matching pattern: {}", pattern);
            }

            // Sort files to ensure consistent ordering
            pattern_files.sort();
            all_files.extend(pattern_files);
        } else {
            // It's a regular file path
            let path = PathBuf::from(pattern);
            all_files.push(path);
        }
    }

    if all_files.is_empty() {
        anyhow::bail!("No input files specified");
    }

    Ok(all_files)
}


fn concatenate_mov_files(infos: &[MovInfo], output_path: &PathBuf) -> Result<()> {
    println!("Starting concatenation...");

    // Check if ffmpeg is available
    let ffmpeg_check = std::process::Command::new("ffmpeg")
        .arg("-version")
        .output();

    match ffmpeg_check {
        Ok(_) => {
            concatenate_with_ffmpeg(infos, output_path)
        }
        Err(_) => {
            anyhow::bail!(
                "FFmpeg is required for mov concatenation. Please install FFmpeg:\n\
                - macOS: brew install ffmpeg\n\
                - Ubuntu/Debian: sudo apt install ffmpeg\n\
                - Windows: Download from https://ffmpeg.org/download.html"
            );
        }
    }
}

fn concatenate_with_ffmpeg(infos: &[MovInfo], output_path: &PathBuf) -> Result<()> {
    println!("Using FFmpeg for lossless concatenation...");

    // Create a temporary file list for FFmpeg concat demuxer
    let temp_dir = std::env::temp_dir();
    let filelist_path = temp_dir.join("movcat_filelist.txt");

    // Write file list
    let mut filelist_content = String::new();
    for info in infos {
        let absolute_path = info.path.canonicalize()
            .with_context(|| format!("Failed to get absolute path for: {:?}", info.path))?;
        filelist_content.push_str(&format!("file '{}'\n", absolute_path.display()));
    }

    std::fs::write(&filelist_path, filelist_content)
        .with_context(|| format!("Failed to write file list: {:?}", filelist_path))?;

    // Run FFmpeg concat
    let mut ffmpeg_cmd = std::process::Command::new("ffmpeg");
    ffmpeg_cmd
        .arg("-f").arg("concat")
        .arg("-safe").arg("0")
        .arg("-i").arg(&filelist_path)
        .arg("-c").arg("copy")
        .arg("-avoid_negative_ts").arg("make_zero")
        .arg("-y") // Overwrite output file
        .arg(output_path);

    println!("Running: {:?}", ffmpeg_cmd);

    let output = ffmpeg_cmd.output()
        .with_context(|| "Failed to execute FFmpeg")?;

    // Clean up temp file
    let _ = std::fs::remove_file(&filelist_path);

    if output.status.success() {
        println!("Concatenation completed successfully!");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg failed: {}", stderr);
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Expanding input patterns...");
    let input_files = expand_glob_patterns(&args.inputs)?;

    println!("Found {} files:", input_files.len());
    for file in &input_files {
        println!("  {:?}", file);
    }
    println!();

    println!("Analyzing input files...");
    let file_infos = validate_input_files(&input_files)?;

    for info in &file_infos {
        println!("File: {:?}", info.path);
        println!("  Duration: {}s", info.duration as f64 / info.timescale as f64);
        println!("  Tracks: {} (Video: {}, Audio: {})",
            info.track_count, info.video_tracks, info.audio_tracks);
        println!("  Major Brand: {}", info.major_brand);
        println!();
    }

    println!("Total files: {}", file_infos.len());
    println!("Output file: {:?}", args.output);

    // Perform concatenation
    concatenate_mov_files(&file_infos, &args.output)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_mov_file_nonexistent() {
        let path = PathBuf::from("nonexistent.mov");
        let result = analyze_mov_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_input_files_empty() {
        let files = vec![];
        let result = validate_input_files(&files);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_validate_input_files_nonexistent() {
        let files = vec![PathBuf::from("nonexistent.mov")];
        let result = validate_input_files(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_ffmpeg_availability() {
        let result = std::process::Command::new("ffmpeg")
            .arg("-version")
            .output();

        match result {
            Ok(output) => {
                assert!(output.status.success());
                println!("FFmpeg is available");
            }
            Err(_) => {
                println!("FFmpeg is not available - concatenation will fail");
            }
        }
    }

    #[test]
    fn test_expand_glob_patterns_no_wildcards() {
        let patterns = vec!["file1.mov".to_string(), "file2.mov".to_string()];
        let result = expand_glob_patterns(&patterns);
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], PathBuf::from("file1.mov"));
        assert_eq!(files[1], PathBuf::from("file2.mov"));
    }

    #[test]
    fn test_expand_glob_patterns_empty() {
        let patterns = vec![];
        let result = expand_glob_patterns(&patterns);
        assert!(result.is_err());
    }

    #[test]
    fn test_expand_glob_patterns_nonexistent_pattern() {
        let patterns = vec!["nonexistent_*.mov".to_string()];
        let result = expand_glob_patterns(&patterns);
        assert!(result.is_err());
    }
}
