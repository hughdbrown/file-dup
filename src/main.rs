use std::{
    path::PathBuf,
};

use clap::{Command, Arg, ArgAction};
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
    let parallel_arg = Arg::new("parallel")
        .short('p')
        .long("parallel")
        .action(ArgAction::SetTrue);


    let command = Command::new("file_dup")
        .version("0.1.0")
        .author("Hugh Brown <hughdbrown@gmail.com>")
        .about("File deduplicator")
        .arg(filetype_arg)
        .arg(parallel_arg);
    let matches = command.get_matches();

    let filetype: &String = matches.get_one("filetype").unwrap();
    let parallel: bool = *matches.get_one("parallel").unwrap();
    let filetype: String = filetype.clone();

    Ok(
        AppArgs { filetype, parallel, }
    )
}

fn collapse_strings(result: &[String]) -> String {
    result.iter()
        .filter(|s| !s.is_empty())
        .cloned()
        .collect::<Vec::<String>>()
        .join("\n")
}

fn run_serial(files: &[PathBuf], ext: &str) {
    let result = files.iter()
        .map(|path| {
            // FIXME: Can't pass a reference because PathBuf does not implement Copy.
            // So clone a copy in memory for each call instead ...
            let copy = files.clone();
            process(path, ext, copy)
        })
        .collect::<Vec::<String>>();
    println!("{}", collapse_strings(&result));
}

fn run_parallel(files: &[PathBuf], ext: &str) {
    let result = files.par_iter()
        .map(|path| {
            // FIXME: Can't pass a reference because PathBuf does not implement Copy.
            // So clone a copy in memory for each call instead ...
            let copy = files.clone();
            process(path, ext, copy)
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
            let ext = app.filetype;
            let pattern = format!("*{ext}");
            let files: Vec<PathBuf> = files_matching_pattern(&pattern);
            println!("# Processing {} {} files", files.len(), &ext);

            if app.parallel {
                run_parallel(&files, &ext);
            }
            else {
                run_serial(&files, &ext);
            }
        },
        Err(err) => eprintln!("{}", err),
    }
}
