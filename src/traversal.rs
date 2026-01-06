use anyhow::Result;
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use std::path::PathBuf;

pub struct TraversalOptions {
    pub root: PathBuf,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

pub fn traverse(options: &TraversalOptions) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    log::debug!("Traversing {}", options.root.display());

    // 1. Setup exclusions
    // MATCH BEHAVIOR: OverrideBuilder::add("pattern") creates a Whitelist rule.
    // So if a file matches "pattern", result is Whitelist.
    // If it doesn't match, result is Ignore (Unmatched).
    let exclude_matcher = if options.exclude.is_empty() {
        None
    } else {
        let mut builder = OverrideBuilder::new(&options.root);
        for pattern in &options.exclude {
            builder.add(pattern)?;
        }
        Some(builder.build()?)
    };

    let mut walker = WalkBuilder::new(&options.root);
    walker.git_ignore(true); // We handle custom overrides manually below

    // 2. Setup inclusions
    let include_matcher = if options.include.is_empty() {
        None
    } else {
        let mut builder = OverrideBuilder::new(&options.root);
        for pattern in &options.include {
            builder.add(pattern)?;
        }
        Some(builder.build()?)
    };

    for result in walker.build() {
        match result {
            Ok(entry) => {
                if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                    continue;
                }

                let path = entry.path();
                // OverrideBuilder expects relative paths from the root it was built with.
                let relative_path = path.strip_prefix(&options.root).unwrap_or(path);

                log::trace!(
                    "Checking {} (rel: {})",
                    path.display(),
                    relative_path.display()
                );

                // Exclude check
                if let Some(matcher) = &exclude_matcher {
                    let res = matcher.matched(relative_path, false);
                    // If matched (Whitelist), it means it matched an exclude pattern.
                    // So we should SKIP it.
                    if res.is_whitelist() {
                        log::debug!("Excluded file {} (pattern match)", relative_path.display());
                        continue;
                    }
                }

                // Include check
                if let Some(matcher) = &include_matcher {
                    let res = matcher.matched(relative_path, false);
                    // If matched (Whitelist), it means it matched an include pattern.
                    // If NOT matched (Ignore), we should SKIP it.
                    if !res.is_whitelist() {
                        log::debug!("Skipped file {} (not included)", relative_path.display());
                        continue;
                    }
                }

                files.push(path.to_path_buf());
            }
            Err(err) => log::error!("Traversal error: {err}"),
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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

    #[test]
    fn test_traverse_recursive_exclude() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        fs::create_dir(root.join("src"))?;
        File::create(root.join("src/main.rs"))?;
        File::create(root.join("Cargo.lock"))?;
        File::create(root.join("src/test.lock"))?;

        let options = TraversalOptions {
            root: root.to_path_buf(),
            include: vec![],
            exclude: vec!["**/*.lock".to_string()],
        };

        let files = traverse(&options)?;
        for f in &files {
            eprintln!("Found: {:?}", f);
        }

        assert!(
            files.iter().any(|p| p.ends_with("main.rs")),
            "Files found: {:?}",
            files
        );
        assert!(!files.iter().any(|p| p.ends_with("Cargo.lock")));
        assert!(!files.iter().any(|p| p.ends_with("test.lock")));

        Ok(())
    }
}
