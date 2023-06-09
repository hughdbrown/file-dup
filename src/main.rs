use std::{
    path::PathBuf,
};

use clap::{Command, Arg};

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

    let filetype: &String = matches.get_one("filetype").unwrap();

    Ok(
        AppArgs {
            filetype: filetype.clone(),
        }
    )
}

fn run(app: AppArgs) {
    // Find all the files that have the required extension.
    // Make this fast by scanning the disk only once.
    let ext = app.filetype;
    let pattern = format!("*{ext}");
    let files: Vec<PathBuf> = files_matching_pattern(&pattern);

    for path in files.iter() {
        // FIXME: Can't pass a reference because PathBuf does not implement Copy.
        // So clone a copy in memory for each call instead ...
        let copy = files.clone();
        process(&path, &ext, &copy);
    }
}
fn main() {
    let args = get_args();
    match args {
        Ok(app) => run(app),
        Err(err) => eprintln!("{}", err),
    }
}
