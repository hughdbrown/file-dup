use std::{
    collections::BinaryHeap,
    error::Error,
    path::{Path, PathBuf},
};

use clap::{Command, Arg};
use glob::glob;
use regex::Regex;

mod file_hash;
mod file_util;

use crate::file_util::get_creation_time;
use crate::file_hash::file_hash;

#[derive(Debug, Clone)]
struct AppArgs {
    /// File extension to search for
    filetype: String,
}

fn files_matching_pattern(pattern: &str) -> Vec<PathBuf>
{
    glob(pattern).unwrap()
        .flatten()
        .collect()
}

fn process(path: &Path, ext: &str, all_files: &Vec<PathBuf>) {
    let name: String = path.file_stem().unwrap().to_string_lossy().into_owned();
    let name_re = name.replace("(", "\\(").replace(")", "\\)");
    let orig_path_str = path.to_str().unwrap();
    let regex_str = format!(r"{name_re} \(\d+\){ext}");
    let re = Regex::new(&regex_str).unwrap();
    let files: Vec<PathBuf> = all_files
        .iter()
        .filter(|p| {
            if let Some(path_str) = p.to_str() {
                return re.is_match(path_str);
            }
            return false;
        })
        .map(|p| p.clone())
        .collect();

    if !files.is_empty() {
        // SHA1 hash of base file
        let orig_hash = file_hash(path).unwrap();
        println!("# {} {} {}", "-".repeat(30), path.display(), orig_hash);
        let mut heap = BinaryHeap::new();

        for file_path in files {
            let pp = file_path.as_path();
            let copy_hash = file_hash(pp).unwrap();
            println!("# {} {}", pp.display(), copy_hash);
            if copy_hash == orig_hash {
                println!("rm \"{:?}\" # {:?}", pp, orig_path_str);
            }
            else {
                let creation_time = get_creation_time(pp).unwrap();
                if let Some(x) = file_path.to_str() {
                    heap.push((creation_time, x.to_owned()));
                }
            }
        }
        if ! heap.is_empty() {
            if let Some(max_val) = heap.pop() {
                println!("rm \"{}\"", orig_path_str);
                while ! heap.is_empty() {
                    if let Some(other_val) = heap.pop() {
                        println!("rm \"{}\"", other_val.1);
                    }
                }
                println!("mv \"{}\" \"{}\"", max_val.1, orig_path_str);
            }
        }
    }
}

type MyResult<T> = Result<T, Box<dyn Error>>;


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
