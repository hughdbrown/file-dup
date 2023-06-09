use std::{
    fs,
    io,
    path::Path,
    time::SystemTime,
};

pub fn get_creation_time(file_path: &Path) -> io::Result<SystemTime> {
    let metadata = fs::metadata(file_path)?;
    let creation_time = metadata.created()?;
    Ok(creation_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Write,
    };
    use tempfile::NamedTempFile;
    use chrono::prelude::{DateTime, Utc, Local};

    fn iso8601(st: &std::time::SystemTime) -> String {
        let dt: DateTime<Utc> = st.clone().into();
        format!("{}", dt.format("%+"))
        // formats like "2001-07-08T00:34:60.026490+09:30"
    }

    #[test]
    fn test_creation_time() {
        // Create a temporary file
        let mut file = NamedTempFile::new().unwrap();

        // Write some data to the file
        file.write_all(b"Hello, World!\n").unwrap();
        
        let file_path = file.path();

        // Calculate the SHA1 hash
        let x = get_creation_time(file_path).unwrap();
        let today = iso8601(&x);

        let t = Local::now();
        let s = t.to_rfc3339();
        assert!(today.starts_with(&s[0..10]));
    }
}
