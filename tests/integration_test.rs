#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs::File;
use std::io::Write;

#[test]
fn test_help_command() {
    Command::cargo_bin("file-dup")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("File deduplicator"))
        .stdout(predicate::str::contains("--filetype"))
        .stdout(predicate::str::contains("--dir"));
}

#[test]
fn test_version_command() {
    Command::cargo_bin("file-dup")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.2"));
}

#[test]
fn test_invalid_filetype_without_dot() {
    Command::cargo_bin("file-dup")
        .unwrap()
        .args(&["--filetype", "pdf"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("must start with a dot"));
}

#[test]
fn test_nonexistent_directory() {
    Command::cargo_bin("file-dup")
        .unwrap()
        .args(&["--dir", "/nonexistent/directory/that/does/not/exist"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("file-dup")
        .unwrap()
        .args(&["--dir", temp_dir.path().to_str().unwrap(), "--filetype", ".pdf"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanning for files"))
        .stdout(predicate::str::contains("Processing 0 .pdf files"));
}

#[test]
fn test_directory_with_files_no_duplicates() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create a single PDF file
    let file1 = dir_path.join("test1.pdf");
    {
        let mut f = File::create(&file1).unwrap();
        f.write_all(b"content1").unwrap();
    }

    Command::cargo_bin("file-dup")
        .unwrap()
        .args(&["--dir", dir_path.to_str().unwrap(), "--filetype", ".pdf"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Processing 1 .pdf files"));
}

#[test]
fn test_directory_with_exact_duplicates() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create base file
    let base = dir_path.join("document.pdf");
    {
        let mut f = File::create(&base).unwrap();
        f.write_all(b"same content").unwrap();
    }

    // Create duplicate with same content
    let dup = dir_path.join("document (1).pdf");
    {
        let mut f = File::create(&dup).unwrap();
        f.write_all(b"same content").unwrap();
    }

    let output = Command::cargo_bin("file-dup")
        .unwrap()
        .args(&["--dir", dir_path.to_str().unwrap(), "--filetype", ".pdf"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Processing 2 .pdf files"))
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Should contain rm command for the duplicate
    assert!(stdout.contains("rm"));
    assert!(stdout.contains("document (1).pdf"));
}

#[test]
fn test_directory_with_different_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create files with different extensions
    File::create(dir_path.join("file1.pdf")).unwrap();
    File::create(dir_path.join("file2.txt")).unwrap();
    File::create(dir_path.join("file3.pdf")).unwrap();

    Command::cargo_bin("file-dup")
        .unwrap()
        .args(&["--dir", dir_path.to_str().unwrap(), "--filetype", ".pdf"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Processing 2 .pdf files"));
}

#[test]
fn test_custom_extension() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create files with custom extension
    File::create(dir_path.join("archive1.zip")).unwrap();
    File::create(dir_path.join("archive2.zip")).unwrap();
    File::create(dir_path.join("document.pdf")).unwrap();

    Command::cargo_bin("file-dup")
        .unwrap()
        .args(&["--dir", dir_path.to_str().unwrap(), "--filetype", ".zip"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Processing 2 .zip files"));
}

#[test]
fn test_file_instead_of_directory() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    File::create(&file_path).unwrap();

    Command::cargo_bin("file-dup")
        .unwrap()
        .args(&["--dir", file_path.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not a directory"));
}

#[test]
fn test_output_format_with_comments() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create base file and duplicate
    let base = dir_path.join("test.pdf");
    {
        let mut f = File::create(&base).unwrap();
        f.write_all(b"content").unwrap();
    }

    let dup = dir_path.join("test (1).pdf");
    {
        let mut f = File::create(&dup).unwrap();
        f.write_all(b"content").unwrap();
    }

    let output = Command::cargo_bin("file-dup")
        .unwrap()
        .args(&["--dir", dir_path.to_str().unwrap(), "--filetype", ".pdf"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Output should contain comment lines starting with #
    assert!(stdout.contains("# Scanning"));
    assert!(stdout.contains("# Processing"));
    // Should have hash comments in the output
    assert!(stdout.contains("# ------"));
}
