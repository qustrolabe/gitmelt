use crate::decorator::{ContentDecorator, GlobalDecorator};
use anyhow::Result;
use content_inspector;
use log::{error, info};
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use tiktoken_rs::cl100k_base;

pub const DIGEST_FILENAME: &str = "digest.txt";

pub enum OutputDestination {
    File(PathBuf),
    Stdout,
}

pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

pub fn ingest(
    files: &[PathBuf],
    output_dest: OutputDestination,
    content_decorator: &dyn ContentDecorator,
    global_decorator: Option<&dyn GlobalDecorator>,
) -> Result<()> {
    match &output_dest {
        OutputDestination::File(path) => info!("Writing digest to {:?}", path),
        OutputDestination::Stdout => info!("Writing digest to stdout"),
    }

    let mut writer: Box<dyn Write> = match output_dest {
        OutputDestination::File(path) => Box::new(File::create(path)?),
        OutputDestination::Stdout => Box::new(io::stdout()),
    };

    let mut total_tokens = 0;

    if let Some(global) = global_decorator {
        if let Some(prologue) = global.prologue(files) {
            writeln!(writer, "{}", prologue)?;
        }
    }

    let bpe = cl100k_base().ok();

    for path in files {
        // 1. Check file size
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() > MAX_FILE_SIZE {
                error!("Skipping large file: {:?} ({} bytes)", path, metadata.len());
                writeln!(writer, "----- {:?} (Skipped: >10MB) -----", path)?;
                continue;
            }
        }

        // 2. Check for binary content
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                error!("Error opening {:?}: {}", path, e);
                writeln!(writer, "----- {:?} (Error opening file) -----", path)?;
                continue;
            }
        };

        // Read first chunk (up to 1KB) to check for binary
        // We use a small buffer. If file is small, this reads it all.
        // We want to verify it's text before strictly reading everything into a string.
        let mut buffer = [0u8; 1024];
        let n = match std::io::Read::read(&mut file, &mut buffer) {
            Ok(n) => n,
            Err(e) => {
                error!("Error reading prelude of {:?}: {}", path, e);
                writeln!(writer, "----- {:?} (Error reading prelude) -----", path)?;
                continue;
            }
        };

        if n > 0 && content_inspector::inspect(&buffer[..n]).is_binary() {
            log::warn!("Skipping binary file: {:?}", path);
            writeln!(writer, "----- {:?} (Skipped: Binary) -----", path)?;
            continue;
        }

        // Reset cursor to 0 to read full content
        // Note: `File` implements `Seek`
        if let Err(e) = std::io::Seek::seek(&mut file, std::io::SeekFrom::Start(0)) {
            error!("Error seeking {:?}: {}", path, e);
            continue;
        }

        // now read to string safely
        let mut content = String::new();
        if let Err(e) = std::io::Read::read_to_string(&mut file, &mut content) {
            error!("Error reading {:?}: {}", path, e);
            writeln!(writer, "----- {:?} (Error reading content) -----", path)?;
            continue;
        }

        if let Some(before) = content_decorator.before(path) {
            writeln!(writer, "{}", before)?;
        }

        content = content_decorator.transform(path, content);
        writeln!(writer, "{}", content)?;

        if let Some(after) = content_decorator.after(path) {
            writeln!(writer, "{}", after)?;
        }

        // Count tokens
        if let Some(ref tokenizer) = bpe {
            let tokens = tokenizer.encode_with_special_tokens(&content);
            total_tokens += tokens.len();
        }
    }

    info!("Total estimated tokens: {}", total_tokens);
    println!("Total estimated tokens: {}", total_tokens);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decorator::DefaultDecorator;
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

        let output_path = root.join(DIGEST_FILENAME);
        let decorator = DefaultDecorator;

        ingest(
            &[file1.clone(), file2.clone()],
            OutputDestination::File(output_path.clone()),
            &decorator,
            None,
        )?;

        assert!(output_path.exists());

        let content = std::fs::read_to_string(output_path)?;
        assert!(content.contains("file1.txt"));
        assert!(content.contains("Hello"));
        assert!(content.contains("file2.txt"));
        assert!(content.contains("World"));

        Ok(())
    }
}
