mod ingest;
mod traversal;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use traversal::TraversalOptions;

#[derive(Parser)]
#[command(name = "git-melt")]
#[command(about = "Concatenates file contents into a single digest file", long_about = None)]
struct Cli {
    /// Path to traverse
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Include patterns (glob)
    #[arg(short, long)]
    include: Vec<String>,

    /// Exclude patterns (glob)
    #[arg(short, long)]
    exclude: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let options = TraversalOptions {
        root: cli.path.clone(),
        include: cli.include,
        exclude: cli.exclude,
    };

    println!("Traversing files in {:?}", options.root);
    let files = traversal::traverse(&options)?;

    println!("Found {} files", files.len());
    if files.is_empty() {
        println!("No files found matching patterns.");
        return Ok(());
    }

    println!("Generating digest...");
    ingest::ingest(&files, &options.root)?;

    println!(
        "Done! Digest written to {:?}",
        options.root.join(ingest::DIGEST_FILENAME)
    );

    Ok(())
}
