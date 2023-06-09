use std::{
    fs::File,
    //io::{self, Read},
    io,
    path::Path,
};
use sha1::{self, Sha1, Digest};
use data_encoding::HEXLOWER;


pub fn file_hash(file_path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(file_path)?;
    //let filesize = file.metadata().unwrap().len();
    //let mut buffer = Vec::<u8>::with_capacity(filesize.try_into().unwrap());
    //file.read_to_end(&mut buffer)?;
    //let hash = Sha1::digest(&buffer);
    let mut hasher = Sha1::new();
    let _ = io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(HEXLOWER.encode(hash.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calculate_sha1_file() {
        // Create a temporary file
        let mut file = NamedTempFile::new().unwrap();

        // Write some data to the file
        file.write_all("Hello, World!\n".as_bytes()).unwrap();
        
        let file_path = file.path();

        // Calculate the SHA1 hash
        let hash = file_hash(file_path).unwrap();

        // Verify the expected hash value
        assert_eq!(hash, "60fde9c2310b0d4cad4dab8d126b04387efba289");
    }
}
