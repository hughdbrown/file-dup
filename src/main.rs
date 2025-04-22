use std::{
    path::PathBuf,
};

use clap::{Command, Arg};
use rayon::prelude::*;

use file_dup::{
    AppArgs,
    process,
    files_matching_pattern,
    MyResult,
};

fn get_args() -> MyResult<AppArgs> {
    let dir_arg = Arg::new("dir")
        .short('d')
        .long("dir")
        .default_value(".")
        .required(false);

    let filetype_arg = Arg::new("filetype")
        .short('f')
        .long("filetype")
        .default_value(".pdf")
        .required(false);

    let command = Command::new("file_dup")
        .version("0.1.0")
        .author("Hugh Brown <hughdbrown@gmail.com>")
        .about("File deduplicator")
        .arg(filetype_arg)
        .arg(dir_arg);
    let matches = command.get_matches();

    let filetype: String = matches.get_one::<String>("filetype").unwrap().to_string();
    let dir: String = matches.get_one::<String>("dir").unwrap().to_string();

    Ok(
        AppArgs { filetype, dir, }
    )
}

fn collapse_strings(result: &[String]) -> String {
    result.iter()
        .filter(|s: &&String| !(**s).is_empty())
        .cloned()
        .collect::<Vec<String>>()
        .join("\n")
}

fn run_parallel(files: &[PathBuf], ext: &str) {
    // Create a lookup map for faster file stem access
    let file_stems: Vec<_> = files.iter()
        .map(|path| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string()
        })
        .collect();

    // Calculate chunk size based on number of files and available CPUs
    let chunk_size = std::cmp::max(
        1,
        files.len() / rayon::current_num_threads().max(1)
    );

    let result = files.par_iter()
        .with_min_len(chunk_size) // Adaptive chunk size
        .map(|path: &PathBuf| {
            let path_idx = files.iter().position(|p| p == path).unwrap_or(0);
            let prefix = &file_stems[path_idx];
            
            // Pre-filter the files to avoid repeated string operations
            let copy: Vec<PathBuf> = files.iter()
                .enumerate()
                .filter_map(|(idx, pb)| {
                    if file_stems[idx].starts_with(prefix) {
                        Some(pb.clone())
                    } else {
                        None
                    }
                })
                .collect();
            
            process(path, ext, &copy)
        })
        .collect::<Vec<String>>();
    
    println!("{}", collapse_strings(&result));
}

fn main() {
    let args = get_args();
    match args {
        Ok(app) => {
            // Find all the files that have the required extension.
            // Make this fast by scanning the disk only once.
            let ext: String = app.filetype;
            let dir: String = app.dir;
            let pattern: String = format!("*{ext}");
            
            // Use rayon to parallelize the file discovery process
            println!("# Scanning for files...");
            let files: Vec<PathBuf> = files_matching_pattern(&dir, &pattern);
            println!("# Processing {} {} files", files.len(), &ext);

            // Set optimal thread count based on CPU cores and workload
            let num_cpus = num_cpus::get();
            let thread_count = std::cmp::min(num_cpus, std::cmp::max(1, files.len() / 10));
            
            rayon::ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build_global()
                .unwrap_or_else(|e| eprintln!("Thread pool error: {}", e));
                
            run_parallel(&files, &ext);
        },
        Err(err) => eprintln!("{}", err),
    }
}
