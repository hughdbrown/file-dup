use std::{
    fs::File,
    io,
    path::Path,
};
use blake3;
use data_encoding::HEXLOWER;

pub fn file_hash(file_path: &Path) -> Result<String, io::Error> {
    // Use memory mapping for large files
    let file = File::open(file_path)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();
    
    // For small files, use direct reading
    if file_size < 10_000_000 { // 10MB threshold
        let mut hasher = blake3::Hasher::new();
        let _ = io::copy(&mut File::open(file_path)?, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(HEXLOWER.encode(hash.as_bytes()))
    } else {
        // For large files, use memory mapping
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let hash = blake3::hash(&mmap);
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
