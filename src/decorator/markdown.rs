use super::{ContentDecorator, format_path};
use std::path::Path;

pub struct MarkdownDecorator;

impl ContentDecorator for MarkdownDecorator {
    fn before(&self, path: &Path) -> Option<String> {
        let path_str = format_path(path);
        // Extract extension for syntax highlighting (e.g., "rs", "toml")
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        Some(format!("## File: {}\n```{}", path_str, ext))
    }

    fn after(&self, _path: &Path) -> Option<String> {
        Some("```".to_string())
    }

    fn transform(&self, _path: &Path, content: String) -> String {
        content
    }
}
