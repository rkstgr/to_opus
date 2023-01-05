extern crate glob;
extern crate rayon;
extern crate indicatif;

use glob::glob;
use std::{error::Error, process::{Command, Stdio}, env};
use indicatif::ParallelProgressIterator;
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};

fn main() -> Result<(), Box<dyn Error>> {
    // Set the root directory
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err("expected one argument: the root directory".into());
    }
    let root_dir = &args[1];

    // Find all wav files in the root directory and its subdirectories
    let wav_paths: Vec<_> = glob(&format!("{}/**/*.wav", root_dir))?
        .filter_map(Result::ok)
        .collect();

    println!("Found {} wav files", wav_paths.len());

    // filter out the wav files that already have an opus file
    let wav_paths: Vec<_> = wav_paths
        .iter()
        .filter(|wav_path| {    
            let opus_path = wav_path.with_extension("opus");
            !opus_path.exists()
        })
        .collect();

    // Process the wav files in parallel
    wav_paths.par_iter().progress_count(wav_paths.len() as u64).for_each(|wav_path| {
        // Get the path to the opus file
        let opus_path = wav_path.with_extension("opus");

        // Use ffmpeg to convert the wav file to opus
        let status = Command::new("ffmpeg")
            .arg("-i")
            .arg(wav_path)
            .arg("-c:a")
            .arg("libopus")
            .arg("-b:a")
            .arg("96k")
            .arg("-threads")
            .arg("1")
            .arg(opus_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();

        if !status.success() {
            panic!("ffmpeg failed with status {}", status);
        }
    });

    Ok(())
}