use std::{
    path::{Path, PathBuf},
};

use clap::Parser;
use rayon::prelude::*;

use file_dup::{
    process,
    files_matching_pattern,
    MyResult,
};

#[derive(Parser, Debug)]
#[command(name = "file_dup")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Hugh Brown <hughdbrown@gmail.com>")]
#[command(about = "File deduplicator")]
struct AppArgs {
    /// File extension to search for
    #[arg(short, long, default_value = ".pdf")]
    filetype: String,

    /// Directory to scan
    #[arg(short, long, default_value = ".")]
    dir: String,
}

fn validate_args(args: &AppArgs) -> MyResult<()> {
    // Validate that filetype starts with a dot
    if !args.filetype.starts_with('.') {
        return Err(format!("File extension must start with a dot (e.g., '.pdf'), got '{}'", args.filetype).into());
    }

    // Validate that directory exists and is readable
    let dir_path = Path::new(&args.dir);
    if !dir_path.exists() {
        return Err(format!("Directory does not exist: {}", args.dir).into());
    }
    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {}", args.dir).into());
    }

    Ok(())
}

fn collapse_strings(result: &[String]) -> String {
    result.iter()
        .filter(|s: &&String| !s.is_empty())
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn run_parallel(files: &[PathBuf], ext: &str) -> MyResult<()> {
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

    let result: Result<Vec<String>, _> = files.par_iter()
        .with_min_len(chunk_size) // Adaptive chunk size
        .map(|path: &PathBuf| -> MyResult<String> {
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
        .collect();

    let result = result?;
    println!("{}", collapse_strings(&result));
    Ok(())
}

fn main() {
    let app = AppArgs::parse();

    if let Err(e) = run(&app) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(app: &AppArgs) -> MyResult<()> {
    // Validate arguments
    validate_args(app)?;

    // Find all the files that have the required extension.
    // Make this fast by scanning the disk only once.
    let pattern = format!("*{}", app.filetype);

    // Scan for files
    println!("# Scanning for files in {}...", app.dir);
    let files = files_matching_pattern(&app.dir, &pattern)?;
    println!("# Processing {} {} files", files.len(), &app.filetype);

    if files.is_empty() {
        println!("No matching files found. Check the directory path and file extension.");
        return Ok(());
    }

    // Set optimal thread count based on CPU cores and workload
    let num_cpus = num_cpus::get();
    let thread_count = std::cmp::min(num_cpus, std::cmp::max(1, files.len() / 10));

    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .map_err(|e| format!("Failed to build thread pool: {}", e))?
        .install(|| run_parallel(&files, &app.filetype))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_validate_args_valid() {
        let temp_dir = TempDir::new().unwrap();
        let args = AppArgs {
            filetype: ".pdf".to_string(),
            dir: temp_dir.path().to_str().unwrap().to_string(),
        };

        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_missing_dot() {
        let args = AppArgs {
            filetype: "pdf".to_string(),
            dir: ".".to_string(),
        };

        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must start with a dot"));
    }

    #[test]
    fn test_validate_args_nonexistent_dir() {
        let args = AppArgs {
            filetype: ".pdf".to_string(),
            dir: "/nonexistent/directory/path".to_string(),
        };

        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_validate_args_file_not_dir() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let args = AppArgs {
            filetype: ".pdf".to_string(),
            dir: file_path.to_str().unwrap().to_string(),
        };

        let result = validate_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }

    #[test]
    fn test_collapse_strings_filters_empty() {
        let input = vec![
            "line1".to_string(),
            "".to_string(),
            "line2".to_string(),
            "".to_string(),
            "line3".to_string(),
        ];

        let result = collapse_strings(&input);
        assert_eq!(result, "line1\nline2\nline3");
    }

    #[test]
    fn test_collapse_strings_all_empty() {
        let input = vec!["".to_string(), "".to_string()];
        let result = collapse_strings(&input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_run_parallel_with_real_files() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test files
        let file1 = dir_path.join("test1.pdf");
        {
            let mut f = File::create(&file1).unwrap();
            f.write_all(b"content1").unwrap();
        }

        let file2 = dir_path.join("test2.pdf");
        {
            let mut f = File::create(&file2).unwrap();
            f.write_all(b"content2").unwrap();
        }

        let files = vec![file1, file2];

        // Should not panic or error
        let result = run_parallel(&files, ".pdf");
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_parallel_empty_list() {
        let files: Vec<PathBuf> = vec![];
        let result = run_parallel(&files, ".pdf");
        assert!(result.is_ok());
    }
}
