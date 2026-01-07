use super::{GlobalDecorator, PrologueMode, format_path};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// A global decorator that prints a file tree
pub struct FileTreeDecorator {
    pub root: PathBuf,
    pub mode: PrologueMode,
}

impl GlobalDecorator for FileTreeDecorator {
    fn prologue(&self, files: &[PathBuf]) -> Option<String> {
        match self.mode {
            PrologueMode::Off => None,
            PrologueMode::List => {
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
            PrologueMode::Tree => {
                let mut output = String::new();
                output.push_str("File structure:\n");
                let tree = build_tree(&self.root, files);
                print_tree(&tree, "", &mut output);
                output.push('\n');
                Some(output)
            }
        }
    }
}

#[derive(Debug, Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    is_file: bool,
}

fn build_tree(root: &Path, files: &[PathBuf]) -> TreeNode {
    let mut root_node = TreeNode::default();
    for file in files {
        let relative = file.strip_prefix(root).unwrap_or(file);
        let mut current = &mut root_node;
        for component in relative.components() {
            let name = component.as_os_str().to_string_lossy().to_string();
            current = current.children.entry(name).or_default();
        }
        current.is_file = true;
    }
    root_node
}

fn print_tree(node: &TreeNode, prefix: &str, output: &mut String) {
    let children_count = node.children.len();
    for (i, (name, child)) in node.children.iter().enumerate() {
        let is_last_child = i == children_count - 1;
        let connector = if is_last_child {
            "└── "
        } else {
            "├── "
        };

        // For the root's children, we don't need a special prefix at the start,
        // but for nested ones we do.
        let _ = writeln!(
            output,
            "{}{}{}{}",
            prefix,
            connector,
            name,
            if !child.is_file { "/" } else { "" }
        );

        if !child.children.is_empty() {
            let new_prefix = format!("{}{}", prefix, if is_last_child { "    " } else { "│   " });
            print_tree(child, &new_prefix, output);
        }
    }
}
