use std::path::Path;

pub mod default;
pub mod file_tree;
pub mod xml;

pub use default::DefaultDecorator;
pub use file_tree::FileTreeDecorator;
pub use xml::XmlDecorator;

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

/// Helper to ensure paths always use forward slashes for the digest
pub fn format_path(path: &Path) -> String {
    // 1. Strip the "." prefix if it exists
    let path = path.strip_prefix(".").unwrap_or(path);

    // 2. Rebuild using forward slashes
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_path_unix() {
        let path = Path::new("src/main.rs");
        assert_eq!(format_path(path), "src/main.rs");
    }

    #[test]
    #[cfg(windows)]
    fn test_format_path_windows() {
        let path = PathBuf::from("src\\main.rs");
        assert_eq!(format_path(&path), "src/main.rs");
    }

    #[test]
    fn test_format_path_with_dot() {
        let path = Path::new("./src/main.rs");
        assert_eq!(format_path(path), "src/main.rs");
    }
}
