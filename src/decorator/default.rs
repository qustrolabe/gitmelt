use super::{ContentDecorator, format_path};
use std::path::Path;

/// The default decorator that mimics the original behavior
pub struct DefaultDecorator;

impl ContentDecorator for DefaultDecorator {
    fn before(&self, path: &Path) -> Option<String> {
        let path_str = format_path(path);
        Some(format!(
            "================================================\nFILE: {}\n================================================\n",
            path_str
        ))
    }

    fn after(&self, _path: &Path) -> Option<String> {
        // The original implementation adds an empty line after content
        Some(String::new())
    }

    fn transform(&self, _path: &Path, content: String) -> String {
        content
    }
}
