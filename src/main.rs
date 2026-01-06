mod cloner;
mod decorator;
mod ingest;
mod traversal;

use anyhow::Result;
use clap::Parser;
use log::{LevelFilter, info};
use std::env;
use std::path::PathBuf;
use traversal::TraversalOptions;

use crate::decorator::{ContentDecorator, DefaultDecorator, FileTreeDecorator, XmlDecorator};
use crate::ingest::OutputDestination;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Preset {
    Default,
    Xml,
}

#[derive(Parser)]
#[command(name = "git-melt")]
#[command(about = "Concatenates file contents into a single digest file", long_about = None)]
struct Cli {
    /// Path to traverse or Git URL
    #[arg(default_value = ".")]
    input: String,

    /// Git branch to clone (if input is a git URL)
    #[arg(long)]
    branch: Option<String>,

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

    /// Output preset
    #[arg(long, value_enum, default_value_t = Preset::Default)]
    preset: Preset,
}

fn init_logger(verbose: bool) {
    let mut builder = env_logger::Builder::new();

    // Default to error level, unless verbose is set
    let level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };

    builder.filter_level(level);
    builder.init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    init_logger(cli.verbose);

    let temp_dir_handle = if cli.input.starts_with("http") || cli.input.starts_with("git@") {
        Some(cloner::clone_repo(&cli.input, cli.branch.as_deref())?)
    } else {
        None
    };

    let root_path = if let Some(ref temp) = temp_dir_handle {
        temp.path().to_path_buf()
    } else {
        PathBuf::from(&cli.input)
    };

    let options = TraversalOptions {
        root: root_path.clone(),
        include: cli.include,
        exclude: cli.exclude,
    };

    info!("Traversing files in {}", options.root.display());
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

    let content_decorator: Box<dyn ContentDecorator> = match cli.preset {
        Preset::Default => Box::new(DefaultDecorator),
        Preset::Xml => Box::new(XmlDecorator),
    };

    let global_decorator = FileTreeDecorator {
        root: options.root.clone(),
    };

    ingest::ingest(
        &files,
        output_dest,
        content_decorator.as_ref(),
        Some(&global_decorator),
    )?;

    info!("Done!");

    Ok(())
}
