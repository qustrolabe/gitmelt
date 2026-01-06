use super::{GlobalDecorator, format_path};
use std::fmt::Write;
use std::path::PathBuf;

/// A global decorator that prints a file tree
pub struct FileTreeDecorator {
    pub root: PathBuf,
}

impl GlobalDecorator for FileTreeDecorator {
    fn prologue(&self, files: &[PathBuf]) -> Option<String> {
        let mut output = String::new();
        output.push_str("Files included in this digest:\n");
        for file in files {
            // Try to make path relative to root for cleaner output
            let display_path = file.strip_prefix(&self.root).unwrap_or(file);
            let _ = writeln!(output, "- {}", format_path(display_path));
        }
        output.push('\n');
        Some(output)
    }
}
