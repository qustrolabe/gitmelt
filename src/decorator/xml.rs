use super::{ContentDecorator, format_path};
use std::path::Path;

pub struct XmlDecorator;

impl ContentDecorator for XmlDecorator {
    fn before(&self, path: &Path) -> Option<String> {
        let path_str = format_path(path);
        Some(format!("<file path=\"{path_str}\">"))
    }

    fn after(&self, _path: &Path) -> Option<String> {
        Some("</file>".to_string())
    }

    fn transform(&self, _path: &Path, content: String) -> String {
        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_xml_decorator() {
        let decorator = XmlDecorator;
        let path = PathBuf::from("src/main.rs");
        let content = "println!(\"hello\");".to_string();

        let before = decorator.before(&path).unwrap();
        let after = decorator.after(&path).unwrap();
        let transformed = decorator.transform(&path, content.clone());

        assert_eq!(before, "<file path=\"src/main.rs\">");
        assert_eq!(after, "</file>");
        assert_eq!(transformed, content);
    }
}
