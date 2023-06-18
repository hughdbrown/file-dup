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
    let filetype_arg = Arg::new("filetype")
        .short('f')
        .long("filetype")
        .default_value(".pdf")
        .required(false);

    let command = Command::new("file_dup")
        .version("0.1.0")
        .author("Hugh Brown <hughdbrown@gmail.com>")
        .about("File deduplicator")
        .arg(filetype_arg);
    let matches = command.get_matches();

    let filetype: String = matches.get_one::<String>("filetype").unwrap().to_string();

    Ok(
        AppArgs { filetype, }
    )
}

fn collapse_strings(result: &[String]) -> String {
    result.iter()
        .filter(|s: &&String| !(**s).is_empty())
        .cloned()
        .collect::<Vec::<String>>()
        .join("\n")
}

fn run_parallel(files: &[PathBuf], ext: &str) {
    let result = files.par_iter()
        .map(|path: &PathBuf| {
            let prefix: &str = path.file_stem().unwrap().to_str().unwrap();
            let copy: Vec<PathBuf> = files
                .iter()
                .filter(|pb: &&PathBuf|
                    (**pb).file_stem()
                    .unwrap().to_str()
                    .unwrap().starts_with(prefix)
                )
                .cloned()
                .collect();
            process(path, ext, &copy)
        })
        .collect::<Vec::<String>>();
    println!("{}", collapse_strings(&result));
}

fn main() {
    let args = get_args();
    match args {
        Ok(app) => {
            // Find all the files that have the required extension.
            // Make this fast by scanning the disk only once.
            let ext: String = app.filetype;
            let pattern: String = format!("*{ext}");
            let files: Vec<PathBuf> = files_matching_pattern(&pattern);
            println!("# Processing {} {} files", files.len(), &ext);

            run_parallel(&files, &ext);
        },
        Err(err) => eprintln!("{}", err),
    }
}
