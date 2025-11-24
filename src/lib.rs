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

pub type MyResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn files_matching_pattern(dir: &str, pattern: &str) -> MyResult<Vec<PathBuf>>
{
    let glob_pattern = format!("{dir}/{pattern}");
    let paths = glob(&glob_pattern)
        .map_err(|e| format!("Invalid glob pattern '{}': {}", glob_pattern, e))?
        .flatten()
        .collect();
    Ok(paths)
}

pub fn process(path: &Path, ext: &str, all_files: &[PathBuf]) -> MyResult<String> {
    let name: String = path.file_stem()
        .ok_or_else(|| format!("Invalid file path: {}", path.display()))?
        .to_string_lossy()
        .into_owned();
    let name_re: String = name
        .replace('(', r"\(")
        .replace(')', r"\)");
    let orig_path_str: &str = path.to_str()
        .ok_or_else(|| format!("Path contains invalid UTF-8: {}", path.display()))?;
    let regex_str: String = format!(r"{name_re} \(\d+\){ext}");
    let re: Regex = Regex::new(&regex_str)
        .map_err(|e| format!("Failed to compile regex '{}': {}", regex_str, e))?;
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
        // BLAKE3 hash of base file
        let orig_hash: String = file_hash(path)
            .map_err(|e| format!("Failed to hash {}: {}", path.display(), e))?;
        result.push(
            format!("# {} {} {}", "-".repeat(30), path.display(), orig_hash)
        );
        let mut heap = BinaryHeap::new();

        for file_path in files {
            let copy_hash: String = file_hash(&file_path)
                .map_err(|e| format!("Failed to hash {}: {}", file_path.display(), e))?;
            result.push(
                format!("# {} {}", file_path.display(), copy_hash)
            );
            if copy_hash == orig_hash {
                result.push(
                    format!("rm \"{}\" # {}", file_path.display(), orig_path_str)
                );
            } else {
                let creation_time = get_creation_time(&file_path)
                    .map_err(|e| format!("Failed to get creation time for {}: {}", file_path.display(), e))?;
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
            while let Some(other_val) = heap.pop() {
                result.push(
                    format!("rm \"{}\"", other_val.1)
                );
            }
            result.push(
                format!("mv \"{}\" \"{}\"", max_val.1, orig_path_str)
            );
        }
    }
    Ok(result.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_files_matching_pattern() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test files
        File::create(dir_path.join("test1.pdf")).unwrap();
        File::create(dir_path.join("test2.pdf")).unwrap();
        File::create(dir_path.join("other.txt")).unwrap();

        // Test
        let results = files_matching_pattern(
            dir_path.to_str().unwrap(),
            "*.pdf"
        ).unwrap();

        assert_eq!(results.len(), 2);

        // Verify all results are PDF files
        for path in &results {
            assert!(path.to_str().unwrap().ends_with(".pdf"));
        }
    }

    #[test]
    fn test_files_matching_pattern_empty() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // No files created
        let results = files_matching_pattern(
            dir_path.to_str().unwrap(),
            "*.pdf"
        ).unwrap();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_files_matching_pattern_invalid_pattern() {
        let result = files_matching_pattern("/tmp", "[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_process_with_exact_duplicates() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create base file
        let base = dir_path.join("doc.pdf");
        {
            let mut f = File::create(&base).unwrap();
            f.write_all(b"test content").unwrap();
        }

        // Create duplicate with same content
        let dup = dir_path.join("doc (1).pdf");
        {
            let mut f = File::create(&dup).unwrap();
            f.write_all(b"test content").unwrap();
        }

        // Small delay to ensure different creation times if needed
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Collect files
        let files = vec![base.clone(), dup.clone()];

        // Test
        let result = process(&base, ".pdf", &files).unwrap();

        // Should contain rm command for the duplicate
        assert!(result.contains("rm"));
        assert!(result.contains("doc (1).pdf"));
        // Should show hash information
        assert!(result.contains("#"));
    }

    #[test]
    fn test_process_with_different_content() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create base file
        let base = dir_path.join("doc.pdf");
        {
            let mut f = File::create(&base).unwrap();
            f.write_all(b"original content").unwrap();
        }

        // Create file with different content
        let dup = dir_path.join("doc (1).pdf");
        {
            let mut f = File::create(&dup).unwrap();
            f.write_all(b"different content").unwrap();
        }

        // Collect files
        let files = vec![base.clone(), dup.clone()];

        // Test
        let result = process(&base, ".pdf", &files).unwrap();

        // Should contain rm and mv commands for different files
        assert!(result.contains("rm"));
        assert!(result.contains("mv"));
    }

    #[test]
    fn test_process_no_duplicates() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create single file
        let base = dir_path.join("doc.pdf");
        {
            let mut f = File::create(&base).unwrap();
            f.write_all(b"content").unwrap();
        }

        // Collect files - only base file
        let files = vec![base.clone()];

        // Test
        let result = process(&base, ".pdf", &files).unwrap();

        // Should return empty string (no duplicates found)
        assert!(result.is_empty());
    }

    #[test]
    fn test_process_multiple_duplicates_same_hash() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create base file
        let base = dir_path.join("doc.pdf");
        {
            let mut f = File::create(&base).unwrap();
            f.write_all(b"same content").unwrap();
        }

        // Create multiple duplicates with same content
        let dup1 = dir_path.join("doc (1).pdf");
        {
            let mut f = File::create(&dup1).unwrap();
            f.write_all(b"same content").unwrap();
        }

        let dup2 = dir_path.join("doc (2).pdf");
        {
            let mut f = File::create(&dup2).unwrap();
            f.write_all(b"same content").unwrap();
        }

        // Collect files
        let files = vec![base.clone(), dup1.clone(), dup2.clone()];

        // Test
        let result = process(&base, ".pdf", &files).unwrap();

        // Should have rm commands for both duplicates
        let rm_count = result.matches("rm").count();
        assert!(rm_count >= 2, "Expected at least 2 rm commands, got {}", rm_count);
    }

    #[test]
    fn test_process_with_special_chars_in_filename() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create base file with parentheses in name
        let base = dir_path.join("doc(test).pdf");
        {
            let mut f = File::create(&base).unwrap();
            f.write_all(b"content").unwrap();
        }

        // Create duplicate
        let dup = dir_path.join("doc(test) (1).pdf");
        {
            let mut f = File::create(&dup).unwrap();
            f.write_all(b"content").unwrap();
        }

        // Collect files
        let files = vec![base.clone(), dup.clone()];

        // Test - should handle special characters properly
        let result = process(&base, ".pdf", &files);
        assert!(result.is_ok());
    }
}
