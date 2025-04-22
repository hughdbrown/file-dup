use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};
use blake3;
use data_encoding::HEXLOWER;

// Optimized file hashing function
pub fn file_hash(file_path: &Path) -> Result<String, io::Error> {
    const SMALL_FILE_THRESHOLD: u64 = 1_000_000; // 1MB threshold
    const BUFFER_SIZE: usize = 64 * 1024; // 64KB buffer for small files

    // Open the file once
    let file = File::open(file_path)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();
    
    if file_size < SMALL_FILE_THRESHOLD {
        // For small files, use buffered reading with a single hasher
        let mut hasher = blake3::Hasher::new();
        let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
        let mut buffer = [0; BUFFER_SIZE];
        
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        let hash = hasher.finalize();
        Ok(HEXLOWER.encode(hash.as_bytes()))
    } else {
        // For large files, use memory mapping which is faster for large files
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        
        // Use rayon to parallelize the hashing of large files
        let hash = blake3::Hasher::new()
            .update_rayon(&mmap)
            .finalize();
            
        Ok(HEXLOWER.encode(hash.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calculate_hash_file() {
        // Create a temporary file
        let mut file = NamedTempFile::new().unwrap();

        // Write some data to the file
        file.write_all("Hello, World!\n".as_bytes()).unwrap();
        
        let file_path = file.path();

        // Calculate the hash
        let hash = file_hash(file_path).unwrap();

        // With BLAKE3, we need to update the expected hash value
        let expected_hash = blake3::hash("Hello, World!\n".as_bytes());
        assert_eq!(hash, HEXLOWER.encode(expected_hash.as_bytes()));
    }
}
