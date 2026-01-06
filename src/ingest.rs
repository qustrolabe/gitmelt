use anyhow::Result;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const DIGEST_FILENAME: &str = "digest.txt";

pub fn ingest(files: &[PathBuf], output_dir: &Path) -> Result<()> {
    let output_path = output_dir.join(DIGEST_FILENAME);
    let mut output_file = File::create(&output_path)?;

    for path in files {
        let content = std::fs::read_to_string(path);
        match content {
            Ok(c) => {
                writeln!(output_file, "----- {:?} -----", path)?;
                writeln!(output_file, "{}", c)?;
                writeln!(output_file)?;
            }
            Err(e) => {
                eprintln!("Error reading {:?}: {}", path, e);
                writeln!(output_file, "----- {:?} (Error reading file) -----", path)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ingest() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();
        let file1 = root.join("file1.txt");
        let file2 = root.join("file2.txt");

        let mut f1 = File::create(&file1)?;
        writeln!(f1, "Hello")?;

        let mut f2 = File::create(&file2)?;
        writeln!(f2, "World")?;

        ingest(&[file1, file2], root)?;

        let digest_path = root.join(DIGEST_FILENAME);
        assert!(digest_path.exists());

        let content = std::fs::read_to_string(digest_path)?;
        assert!(content.contains("file1.txt"));
        assert!(content.contains("Hello"));
        assert!(content.contains("file2.txt"));
        assert!(content.contains("World"));

        Ok(())
    }
}
