mod decorator;
mod ingest;
mod traversal;

use anyhow::Result;
use clap::Parser;
use log::{LevelFilter, info};
use std::env;
use std::path::PathBuf;
use traversal::TraversalOptions;

use crate::decorator::{DefaultDecorator, FileTreeDecorator};
use crate::ingest::OutputDestination;

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

    /// Output file path (default: digest.txt in current directory)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Print output to stdout instead of file
    #[arg(long, conflicts_with = "output")]
    stdout: bool,

    /// Verbose logging (info level). Default is error only.
    #[arg(short, long)]
    verbose: bool,
}

fn init_logger(verbose: bool) {
    let mut builder = env_logger::Builder::new();

    // Default to error level, unless verbose is set
    let level = if verbose {
        LevelFilter::Info
    } else {
        LevelFilter::Error
    };

    builder.filter_level(level);
    builder.init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    init_logger(cli.verbose);

    let options = TraversalOptions {
        root: cli.path.clone(),
        include: cli.include,
        exclude: cli.exclude,
    };

    info!("Traversing files in {:?}", options.root);
    let files = traversal::traverse(&options)?;

    info!("Found {} files", files.len());
    if files.is_empty() {
        info!("No files found matching patterns.");
        return Ok(());
    }

    info!("Generating digest...");

    let output_dest = if cli.stdout {
        OutputDestination::Stdout
    } else {
        let path = cli
            .output
            .unwrap_or_else(|| env::current_dir().unwrap().join(ingest::DIGEST_FILENAME));
        OutputDestination::File(path)
    };

    let content_decorator = DefaultDecorator;
    let global_decorator = FileTreeDecorator {
        root: options.root.clone(),
    };

    ingest::ingest(
        &files,
        output_dest,
        &content_decorator,
        Some(&global_decorator),
    )?;

    info!("Done!");

    Ok(())
}
