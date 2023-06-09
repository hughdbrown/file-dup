use std::{
    collections::BinaryHeap,
    error::Error,
    path::{Path, PathBuf},
};

use glob::glob;
use regex::Regex;

mod file_hash;
mod file_util;

use crate::file_util::get_creation_time;
use crate::file_hash::file_hash;

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub struct AppArgs {
    /// File extension to search for
    pub filetype: String,
}

pub fn files_matching_pattern(pattern: &str) -> Vec<PathBuf>
{
    glob(pattern).unwrap()
        .flatten()
        .collect()
}

// pub fn process(path: &Path, ext: &str, all_files: &Vec<PathBuf>) {
pub fn process(path: &Path, ext: &str, all_files: &[PathBuf]) {
    let name: String = path.file_stem().unwrap().to_string_lossy().into_owned();
    let name_re = name
        .replace('(', r"\(")
        .replace(')', r"\)");
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
        //.map(|p| p.clone())
        .cloned()
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
                println!("rm \"{}\" # {:?}", pp.display(), orig_path_str);
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


