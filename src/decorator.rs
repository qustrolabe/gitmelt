use std::path::Path;

/// Trait for decorating individual file content
pub trait ContentDecorator {
    /// Initial text to appear before the file content
    fn before(&self, path: &Path) -> Option<String>;

    /// Text to appear after the file content
    fn after(&self, path: &Path) -> Option<String>;

    /// Transform the content of the file itself
    fn transform(&self, path: &Path, content: String) -> String;
}

/// Trait for global decorations on the digest (e.g. at the very start)
pub trait GlobalDecorator {
    /// Text to appear at the very beginning of the digest
    fn prologue(&self, files: &[std::path::PathBuf]) -> Option<String>;
}

/// The default decorator that mimics the original behavior
pub struct DefaultDecorator;

impl ContentDecorator for DefaultDecorator {
    fn before(&self, path: &Path) -> Option<String> {
        Some(format!("----- {:?} -----", path))
    }

    fn after(&self, _path: &Path) -> Option<String> {
        // The original implementation adds an empty line after content
        Some(String::new())
    }

    fn transform(&self, _path: &Path, content: String) -> String {
        content
    }
}

pub struct XmlDecorator;

impl ContentDecorator for XmlDecorator {
    fn before(&self, _path: &Path) -> Option<String> {
        None
    }

    fn after(&self, _path: &Path) -> Option<String> {
        None
    }

    fn transform(&self, path: &Path, content: String) -> String {
        format!("<file path=\"{}\">\n{}\n</file>", path.display(), content)
    }
}

/// A global decorator that prints a file tree
pub struct FileTreeDecorator {
    pub root: std::path::PathBuf,
}

impl GlobalDecorator for FileTreeDecorator {
    fn prologue(&self, files: &[std::path::PathBuf]) -> Option<String> {
        let mut output = String::new();
        output.push_str("Files included in this digest:\n");
        for file in files {
            // Try to make path relative to root for cleaner output
            let display_path = file.strip_prefix(&self.root).unwrap_or(file);
            output.push_str(&format!("- {}\n", display_path.display()));
        }
        output.push('\n');
        Some(output)
    }
}
