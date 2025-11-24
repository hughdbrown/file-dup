# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`file-dup` is a Rust CLI tool that identifies and generates bash scripts to remove duplicate files. It finds files with similar names (e.g., "document.pdf" and "document (1).pdf") and intelligently determines which to keep based on file hashes and creation dates.

The tool generates bash scripts with detailed comments explaining its reasoning, allowing users to review changes before executing them.

## Build and Test Commands

```bash
# Build the project
cargo build

# Build optimized release binary
cargo build --release

# Run the application (defaults to searching for .pdf files in current directory)
cargo run

# Run with specific file type
cargo run -- --filetype=".zip"

# Run with custom directory
cargo run -- --dir="./downloads" --filetype=".pdf"

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_calculate_hash_file

# Security audit
cargo audit
```

## Architecture

### Module Structure

The codebase is organized into three main modules:

- **`main.rs`**: Entry point, CLI argument parsing (using clap), and parallel processing orchestration
- **`lib.rs`**: Core business logic for file processing and duplicate detection
- **`file_hash.rs`**: Optimized file hashing using BLAKE3 with adaptive strategies for small vs large files
- **`file_util.rs`**: File metadata utilities (creation time extraction)

### Parallel Processing Strategy

The application uses Rayon for parallel processing with adaptive threading:

1. Thread pool size is dynamically calculated based on CPU cores and workload (see `main.rs:114-120`)
2. Chunk sizes are computed to balance work distribution (see `main.rs:64-67`)
3. File hashing uses different strategies:
   - Small files (<1MB): Buffered reading with 64KB buffer
   - Large files (≥1MB): Memory-mapped I/O with Rayon-parallelized BLAKE3 hashing

### Duplicate Detection Logic (lib.rs:33-100)

The `process()` function implements the core algorithm:

1. Builds a regex pattern from the base filename to find numbered copies (e.g., "file (1).pdf", "file (2).pdf")
2. Hashes all matching files using BLAKE3
3. For exact duplicates (same hash as original): generates `rm` commands
4. For non-duplicates with similar names:
   - Uses a max-heap sorted by creation time
   - Keeps the most recently created file
   - Generates commands to remove older versions and rename the newest to the original name

### Performance Optimizations

- **Pre-computation**: File stems are extracted once and reused (main.rs:54-61)
- **Parallel hashing**: BLAKE3 with Rayon support for large files
- **Memory mapping**: Used for files ≥1MB for faster I/O
- **Adaptive buffering**: 64KB buffers for small files
- **Thread pool tuning**: Dynamically sized based on workload

## Key Implementation Details

### Hash Function Selection

The project uses BLAKE3 (not SHA-1) for cryptographic hashing because:
- Significantly faster than SHA-1/SHA-256
- Native Rayon support for parallel hashing
- Produces unique hashes suitable for duplicate detection

### File Naming Pattern

The regex pattern expects duplicates in the format: `basename (N).extension` where N is a digit sequence. This matches the naming convention used by browsers and operating systems when downloading duplicate files.

### Output Format

Generated bash scripts include:
- Comments with 30-dash separators showing base file path and hash
- Comments for each duplicate with its path and hash
- `rm` commands for files to delete (with inline comments referencing original)
- `mv` commands to rename kept files back to the original name

## Configuration

The release profile in `Cargo.toml` enables:
- Debug symbols in release builds (for profiling)
- Link-time optimization (LTO) for better performance
