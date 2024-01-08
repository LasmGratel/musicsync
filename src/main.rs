use std::collections::HashSet;
use std::ffi::{OsStr};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::arg;

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rayon::prelude::*;

use walkdir::{DirEntry, WalkDir};

fn main() {
    let cmd = clap::Command::new("musicsync")
        .bin_name("musicsync")
        .about("Convert your music folder into another format, for portable purpose.")
        .version(clap::crate_version!())
        .arg_required_else_help(true)
        .args([
            arg!(<INPUT> "Input directory for walking"),
            arg!(<OUTPUT> "Output directory"),
            arg!([EXTENSION] "Output file extension, default to ogg"),
            arg!(-f --force "Force overwrite existing files"),
            arg!(--preserve "Preserve target folder files, even if they don't exist in source dir"),
            arg!(--dontcopycover "Don't copy cover images"),
            arg!(-o --options <OPTIONS> "Options to be passed to ffmpeg, default to -c:a libopus -b:a 192K -vbr on -cutoff 0 -c:v copy"),
            arg!(-t --filetype <TYPES> "Specify file types to be converted, split with a ',' character. Default to mp3,aac,aif,flac,ogg,wav"),
            arg!(--cover <COVER> "Cover image suffix (case-insensitive). Default to Cover.jpg,Cover.png,AlbumArtSmall.jpg,AlbumArtwork.png")
        ]);

    let matches = cmd.get_matches();

    walk(
        matches.get_one::<String>("INPUT").expect("No input directory specified!"),
        matches.get_one::<String>("OUTPUT").expect("No output directory specified!"),
        matches.get_one::<String>("EXTENSION").cloned().unwrap_or_else(|| "opus".to_string()),
        matches.get_flag("force"),
        matches.get_flag("preserve"),
        matches.get_flag("dontcopycover"),
        matches.get_one::<String>("OPTIONS").cloned().unwrap_or_else(|| "-c:a libopus -b:a 192K -vbr on -cutoff 0 -c:v copy".to_string()),
        matches.get_one::<String>("TYPES")
            .map(|x| x.split(',').map(|x| x.to_string()).collect())
            .unwrap_or_default(),
        matches.get_one::<String>("COVER")
            .map(|x| x.split(',').map(|x| x.to_string()).collect())
            .unwrap_or_default(),
    );
}

fn walk<P: AsRef<Path>>(input_path: P, output_path: P,
                        extension: String,
                        overwrite: bool, preserve: bool, do_not_copy_cover: bool,
                        options: String, file_types: Vec<String>, covers: Vec<String>) {
    let file_types: Vec<String> = if file_types.is_empty() {
        ["mp3", "aac", "aif", "flac", "ogg", "wav"].into_iter().map(|x| x.to_string()).collect()
    } else {
        file_types
    };
    let file_types: HashSet<&str> = file_types.iter().map(|x| x.as_str()).collect();

    let covers: Vec<String> = if covers.is_empty() {
        ["cover.jpg","cover.png","albumartsmall.jpg","albumartwork.png"].into_iter().map(|x| x.to_string()).collect()
    } else {
        covers
    };

    let covers: HashSet<&str> = covers.iter().map(|x| x.as_str()).collect();

    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    let output_files: HashSet<PathBuf> = WalkDir::new(output_path)
        .into_iter()
        .par_bridge()
        .filter_map(|e| e.ok())
        .filter(|e: &DirEntry| {
            e.path().extension()
                .and_then(|x| x.to_str())
                .map(|x| file_types.contains(&x.to_lowercase().as_str()))
                .unwrap_or_default()
        })
        .map(|e| {
            e.path().strip_prefix(output_path).expect("Cannot strip prefix").to_path_buf()
        })
        .collect();

    let input_files: HashSet<PathBuf> = WalkDir::new(input_path)
        .into_iter()
        .par_bridge()
        .filter_map(|e| e.ok())
        .filter(|e: &DirEntry| {
            e.path().extension()
                .and_then(|x| x.to_str())
                .map(|x| file_types.contains(&x.to_lowercase().as_str()))
                .unwrap_or_default()
        })
        .map(|e| {
            e.path().strip_prefix(input_path).expect("Cannot strip prefix").to_path_buf()
        })
        .collect();

    if !preserve {
        output_files.difference(&input_files)
            .par_bridge()
            .for_each(|p| {
                std::fs::remove_file(output_path.join(p).with_extension(&extension)).expect("Cannot cleanup output dir");
            });
    }

    if !do_not_copy_cover {
        WalkDir::new(input_path)
            .into_iter()
            .par_bridge()
            .filter_map(|e| e.ok())
            .filter(|e: &DirEntry| {
                e.path().file_name()
                    .and_then(|x| x.to_str())
                    .map(|x| covers.iter().any(|cover| x.to_lowercase().ends_with(cover))) // Ends with Cover.png etc
                    .unwrap_or_default()
            })
            .map(|e| {
                e.path().strip_prefix(input_path).expect("Cannot strip prefix").to_path_buf()
            })
            .for_each(|cover_path| {
                let output = output_path.join(&cover_path);
                std::fs::create_dir_all(output.parent().expect("Cannot resolve parent")).expect("Cannot create directories");
                std::fs::copy(input_path.join(cover_path), output).expect("Cannot copy cover images");
            });
    }

    let diff: HashSet<_> = input_files.difference(&output_files).collect();
    let progress = ProgressBar::new(diff.len() as u64);
    progress.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));


    diff.into_par_iter()
        .for_each(|input| {
            let output = output_path.join(input.clone().with_extension(&extension));
            let input = input_path.join(input);

            std::fs::create_dir_all(output.parent().expect("Cannot make directories")).expect("Cannot make directories");
            if output.exists() {
                if overwrite {
                    std::fs::remove_file(&output).expect("Cannot overwrite file");
                } else {
                    return;
                }
            }
            if input.exists() {
                convert(input, output, &options);
            } else {
                panic!("Input file {:?} does not exist", input);
            }
            progress.inc(1);
        });
}

fn convert<S: AsRef<OsStr>>(input: S, output: S, options: &str) {

    let child = Command::new("ffmpeg")
        .arg("-i")
        .arg(input)
        .args(options.split(' '))
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute ffmpeg");

    let output = child.wait_with_output()
        .expect("failed to wait on child");

    if !output.status.success() {
        eprintln!("FFmpeg failed with {}", output.status);
        //println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
}