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
    // Create a thread pool with a reasonable number of threads
    let result = files.par_iter()
        .with_min_len(4) // Process at least 4 items per thread to reduce overhead
        .map(|path: &PathBuf| {
            let prefix: &str = path.file_stem().unwrap().to_str().unwrap();
            
            // Pre-filter the files to avoid repeated string operations
            let copy: Vec<PathBuf> = {
                let prefix_owned = prefix.to_string();
                files.par_iter()
                    .filter(|pb: &&PathBuf| {
                        let stem = pb.file_stem().unwrap().to_str().unwrap();
                        stem.starts_with(&prefix_owned)
                    })
                    .cloned()
                    .collect()
            };
            
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
            let files: Vec<PathBuf> = files_matching_pattern(&dir, &pattern);
            println!("# Processing {} {} files", files.len(), &ext);

            run_parallel(&files, &ext);
        },
        Err(err) => eprintln!("{}", err),
    }
}
