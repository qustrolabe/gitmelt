use crate::decorator::{ContentDecorator, GlobalDecorator};
use anyhow::Result;
use log::{error, info};
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

pub const DIGEST_FILENAME: &str = "digest.txt";

pub enum OutputDestination {
    File(PathBuf),
    Stdout,
}

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

    if let Some(global) = global_decorator {
        if let Some(prologue) = global.prologue(files) {
            writeln!(writer, "{}", prologue)?;
        }
    }

    for path in files {
        let content_result = std::fs::read_to_string(path);
        match content_result {
            Ok(mut content) => {
                if let Some(before) = content_decorator.before(path) {
                    writeln!(writer, "{}", before)?;
                }

                content = content_decorator.transform(path, content);
                writeln!(writer, "{}", content)?;

                if let Some(after) = content_decorator.after(path) {
                    writeln!(writer, "{}", after)?;
                }
            }
            Err(e) => {
                error!("Error reading {:?}: {}", path, e);
                writeln!(writer, "----- {:?} (Error reading file) -----", path)?;
            }
        }
    }

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
