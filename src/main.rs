use std::collections::HashSet;
use std::ffi::OsStr;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::arg;

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rayon::prelude::*;

use walkdir::{DirEntry, WalkDir};

fn main() {
    convert("g", "f", "cover.jpg", 1280, 720);
}

fn convert<S: AsRef<OsStr>>(input: S, output: S, cover: S, width: u32, height: u32) {
    let child = Command::new("ffmpeg")
        .args(&[
            "-framerate", "1",
            "-loop", "1",
            "-y",
            "-i"
        ])
        .arg(cover)
        .arg("-i")
        .arg(input)
        .args(&["-r", "30", "-c:v", "libx264", "-tune", "stillimage", "-c:a", "aac", "-b:a", "320k"])
        .arg("-lavfi")
        .arg(format!(
            "\"[0:v]scale={width}:{height}:force_original_aspect_ratio=increase,dblur=angle=90:radius=25[bg];[0:v]scale={width}:{height}:force_original_aspect_ratio=decrease[ov];[bg][ov]overlay=(W-w)/2:(H-h)/2,crop=w={width}:h={height}\"",
            width = width, height = height
        ))
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute ffmpeg");

    let output = child.wait_with_output().expect("failed to wait on child");

    if !output.status.success() {
        eprintln!("ffmpeg failed with {}", output.status);
        //println!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}
