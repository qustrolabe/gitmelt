use anyhow::Result;
use glob::Pattern;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub struct TraversalOptions {
    pub root: PathBuf,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

pub fn traverse(options: &TraversalOptions) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let includes: Result<Vec<Pattern>, _> =
        options.include.iter().map(|p| Pattern::new(p)).collect();
    let includes = includes?;

    let excludes: Result<Vec<Pattern>, _> =
        options.exclude.iter().map(|p| Pattern::new(p)).collect();
    let excludes = excludes?;

    let walker = WalkBuilder::new(&options.root)
        // Hidden files are ignored by default in WalkBuilder (standard_filters: true)
        // .hidden(false) // Toggle if we want hidden files
        .git_ignore(true)
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                    continue;
                }

                let path = entry.path();

                // Get path relative to root for pattern matching/checking
                // or match on file name? User said "*.cs, *.py" which are usually extensions/filenames.
                // glob patterns match against whole path string usually, or filename?
                // `glob::Pattern::matches` matches against a string.
                // If the pattern is `*.cs`, it matches `foo.cs` if we check filename,
                // but `src/foo.cs` might not match `*.cs` if we use full path?
                // Actually `glob` usually handles `*` as within component, `**` as recursive.
                // If user gives `*.rs`, they probably mean matching the filename.
                // Let's assume matching against the file name for simple wildcard extensions.
                // But full paths `src/*.rs` are also possible.
                //
                // Let's try matching against the file name first if pattern has no path separators?
                // Or just use the standard behavior: `matches_path`?
                // `Pattern::matches_path` is not available in `glob` crate directly?
                // Wait, `glob` crate `Pattern::matches` takes a `&str`.

                // To be safe and "smart", let's try to match against the relative path from root.
                // But for `*.rs`, if we pass `src/main.rs`, it won't match `*.rs` in standard glob unless it's `**/*.rs`.
                // However, users often stick to `*.rs` expecting it to work everywhere.
                // If pattern starts with `*` and has no slashes, maybe we match against filename?

                let relative_path = path.strip_prefix(&options.root).unwrap_or(path);
                let relative_str = relative_path.to_string_lossy();
                let filename_str = path
                    .file_name()
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_default();

                let is_excluded = excludes
                    .iter()
                    .any(|p| p.matches(&relative_str) || p.matches(&filename_str));
                if is_excluded {
                    continue;
                }

                let is_included = if includes.is_empty() {
                    true
                } else {
                    includes
                        .iter()
                        .any(|p| p.matches(&relative_str) || p.matches(&filename_str))
                };

                if is_included {
                    files.push(path.to_path_buf());
                }
            }
            Err(err) => eprintln!("Error: {}", err),
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_traverse_basic() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        File::create(root.join("test.rs"))?;
        File::create(root.join("skip.txt"))?;

        let options = TraversalOptions {
            root: root.to_path_buf(),
            include: vec!["*.rs".to_string()],
            exclude: vec![],
        };

        let files = traverse(&options)?;
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("test.rs"));

        Ok(())
    }

    #[test]
    fn test_traverse_exclude() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        File::create(root.join("test.rs"))?;
        File::create(root.join("bad.rs"))?;

        let options = TraversalOptions {
            root: root.to_path_buf(),
            include: vec!["*.rs".to_string()],
            exclude: vec!["bad.rs".to_string()],
        };

        let files = traverse(&options)?;
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("test.rs"));

        Ok(())
    }
}
