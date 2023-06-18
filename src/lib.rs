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

pub fn process(path: &Path, ext: &str, all_files: &[PathBuf]) -> String {
    let name: String = path.file_stem().unwrap().to_string_lossy().into_owned();
    let name_re: String = name
        .replace('(', r"\(")
        .replace(')', r"\)");
    let orig_path_str: &str = path.to_str().unwrap();
    let regex_str: String = format!(r"{name_re} \(\d+\){ext}");
    let re: Regex = Regex::new(&regex_str).unwrap();
    let files: Vec<PathBuf> = all_files
        .iter()
        .filter(|p: &&PathBuf| {
            if let Some(path_str) = p.to_str() {
                return re.is_match(path_str);
            }
            false
        })
        .cloned()
        .collect();

    let mut result: Vec<String> = vec![];
    if !files.is_empty() {
        // SHA1 hash of base file
        let orig_hash: String = file_hash(path).unwrap();
        result.push(
            format!("# {} {} {}", "-".repeat(30), path.display(), orig_hash)
        );
        let mut heap = BinaryHeap::new();

        for file_path in files {
            let pp: &Path = file_path.as_path();
            let copy_hash: String = file_hash(pp).unwrap();
            result.push(
                format!("# {} {}", pp.display(), copy_hash)
            );
            if copy_hash == orig_hash {
                result.push(
                    format!("rm \"{}\" # {:?}", pp.display(), orig_path_str)
                );
            }
            else {
                let creation_time = get_creation_time(pp).unwrap();
                if let Some(x) = file_path.to_str() {
                    heap.push((creation_time, x.to_owned()));
                }
            }
        }

        // Store file paths in a max-heap that is sorted by file creation date.
        // The file path with the most recent creation will be at the root.
        // Save that one and delete all others.
        if let Some(max_val) = heap.pop() {
            result.push(
                format!("rm \"{}\"", orig_path_str)
            );
            while ! heap.is_empty() {
                if let Some(other_val) = heap.pop() {
                    result.push(
                        format!("rm \"{}\"", other_val.1)
                    );
                }
            }
            result.push(
                format!("mv \"{}\" \"{}\"", max_val.1, orig_path_str)
            );
        }
    }
    result.join("\n")
}
