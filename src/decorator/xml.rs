use super::{ContentDecorator, format_path};
use std::path::Path;

pub struct XmlDecorator;

impl ContentDecorator for XmlDecorator {
    fn before(&self, _path: &Path) -> Option<String> {
        None
    }

    fn after(&self, _path: &Path) -> Option<String> {
        None
    }

    fn transform(&self, path: &Path, content: String) -> String {
        let path_str = format_path(path);
        format!("<file path=\"{}\">\n{}\n</file>", path_str, content)
    }
}
