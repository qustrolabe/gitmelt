use crate::decorator::{ContentDecorator, GlobalDecorator};
use anyhow::Result;
use crossbeam_channel::bounded;
use log::{error, info, warn};
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use tiktoken_rs::{CoreBPE, cl100k_base};

pub const DIGEST_FILENAME: &str = "digest.txt";

pub enum OutputDestination {
    File(PathBuf),
    Stdout,
    Null,
}

pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

pub struct IngestMetrics {
    pub total_tokens: usize,
}

struct ProcessedFile {
    index: usize,
    content: String,
    tokens: usize,
}

pub fn ingest(
    files: &[PathBuf],
    output_dest: OutputDestination,
    content_decorator: &dyn ContentDecorator,
    global_decorator: Option<&dyn GlobalDecorator>,
) -> Result<Option<IngestMetrics>> {
    match &output_dest {
        OutputDestination::File(path) => info!("Writing digest to {}", path.display()),
        OutputDestination::Stdout => info!("Writing digest to stdout"),
        OutputDestination::Null => info!("Dry run: only token estimation will be performed"),
    }

    // Pre-load tokenizer
    let tokenizer = cl100k_base().ok();

    let (tx, rx) = bounded(32); // Buffer some results to keep cores busy

    let metrics = crossbeam::scope(|scope| -> Result<IngestMetrics> {
        // Spawn writer thread
        let rx = rx; // Move rx into the scope, but it's shared
        let writer_handle = scope.spawn(move |_| -> Result<usize> {
            let mut writer: Option<Box<dyn Write>> = match output_dest {
                OutputDestination::File(path) => {
                    Some(Box::new(BufWriter::new(File::create(path)?)))
                }
                OutputDestination::Stdout => Some(Box::new(io::stdout())),
                OutputDestination::Null => None,
            };

            let mut total_tokens = 0;
            let mut pending = BTreeMap::new();
            let mut next_index = 0;

            if let Some(prologue) = global_decorator.and_then(|g| g.prologue(files)) {
                if let Some(ref mut w) = writer {
                    writeln!(w, "{prologue}")?;
                }
            }

            while next_index < files.len() {
                // Check if we already have the next segment
                while let Some(processed) = pending.remove(&next_index) {
                    let processed: ProcessedFile = processed;
                    if let Some(ref mut w) = writer {
                        writeln!(w, "{}", processed.content)?;
                    }
                    total_tokens += processed.tokens;
                    next_index += 1;
                }

                if next_index >= files.len() {
                    break;
                }

                // Wait for more results
                if let Ok(processed) = rx.recv() {
                    let processed: ProcessedFile = processed;
                    pending.insert(processed.index, processed);
                } else {
                    break; // Channel closed
                }
            }

            if let Some(ref mut w) = writer {
                w.flush()?;
            }

            Ok(total_tokens)
        });

        // Process files in parallel
        files.par_iter().enumerate().for_each(|(idx, path)| {
            if let Some(processed) =
                process_single_file(idx, path, content_decorator, tokenizer.as_ref())
            {
                let _ = tx.send(processed);
            }
        });

        drop(tx); // Signal completion

        let total_tokens = writer_handle
            .join()
            .map_err(|_| anyhow::anyhow!("Writer thread panicked"))??;

        Ok(IngestMetrics { total_tokens })
    })
    .map_err(|e| anyhow::anyhow!("Scope error: {:?}", e))??;

    info!("Total estimated tokens: {}", metrics.total_tokens);
    println!("Total estimated tokens: {}", metrics.total_tokens);

    Ok(Some(metrics))
}

fn process_single_file(
    index: usize,
    path: &PathBuf,
    content_decorator: &dyn ContentDecorator,
    tokenizer: Option<&CoreBPE>,
) -> Option<ProcessedFile> {
    // 1. Check file size
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.len() > MAX_FILE_SIZE {
            error!(
                "Skipping large file: {} ({} bytes)",
                path.display(),
                metadata.len()
            );
            return Some(ProcessedFile {
                index,
                content: format!("----- {} (Skipped: >10MB) -----", path.display()),
                tokens: 0,
            });
        }
    }

    // 2. Check for binary content & Read
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            error!("Error opening {}: {e}", path.display());
            return Some(ProcessedFile {
                index,
                content: format!("----- {} (Error opening file) -----", path.display()),
                tokens: 0,
            });
        }
    };

    let mut prelude_buffer = [0u8; 1024];
    let n = match std::io::Read::read(&mut file, &mut prelude_buffer) {
        Ok(n) => n,
        Err(e) => {
            error!("Error reading prelude of {}: {e}", path.display());
            return Some(ProcessedFile {
                index,
                content: format!("----- {} (Error reading prelude) -----", path.display()),
                tokens: 0,
            });
        }
    };

    if n > 0 && content_inspector::inspect(&prelude_buffer[..n]).is_binary() {
        warn!("Skipping binary file: {}", path.display());
        return Some(ProcessedFile {
            index,
            content: format!("----- {} (Skipped: Binary) -----", path.display()),
            tokens: 0,
        });
    }

    // Reset cursor
    if let Err(e) = std::io::Seek::seek(&mut file, std::io::SeekFrom::Start(0)) {
        error!("Error seeking {}: {e}", path.display());
        return None;
    }

    let mut content = String::new();
    if let Err(e) = std::io::Read::read_to_string(&mut file, &mut content) {
        error!("Error reading {}: {e}", path.display());
        return Some(ProcessedFile {
            index,
            content: format!("----- {} (Error reading content) -----", path.display()),
            tokens: 0,
        });
    }

    // Apply decoration
    let mut final_output = String::new();
    if let Some(before) = content_decorator.before(path) {
        final_output.push_str(&before);
        final_output.push('\n');
    }

    let transformed_content = content_decorator.transform(path, content);
    final_output.push_str(&transformed_content);
    final_output.push('\n');

    if let Some(after) = content_decorator.after(path) {
        final_output.push_str(&after);
        final_output.push('\n');
    }

    // Count tokens
    let tokens = if let Some(tokenizer) = tokenizer {
        tokenizer
            .encode_with_special_tokens(&transformed_content)
            .len()
    } else {
        0
    };

    Some(ProcessedFile {
        index,
        content: final_output.trim_end().to_string(),
        tokens,
    })
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
