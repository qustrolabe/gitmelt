mod cloner;
mod decorator;
mod ingest;
mod traversal;

use anyhow::Result;
use clap::Parser;
use log::{LevelFilter, info};
use std::env;
use std::path::PathBuf;
use std::time::Instant;
use traversal::TraversalOptions;

use crate::decorator::{
    ContentDecorator, DefaultDecorator, FileTreeDecorator, MarkdownDecorator, XmlDecorator,
};
use crate::ingest::OutputDestination;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Preset {
    Default,
    Markdown,
    Xml,
}

#[derive(Parser)]
#[command(name = "gitmelt")]
#[command(about = "Concatenates file contents into a single digest file", long_about = None)]
#[allow(clippy::struct_excessive_bools)]
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

    /// Prologue mode (tree, list, off)
    #[arg(long, value_enum, default_value_t = crate::decorator::PrologueMode::List)]
    prologue: crate::decorator::PrologueMode,

    /// Dry run (only token estimation)
    #[arg(long)]
    dry: bool,

    /// Disable token counting
    #[arg(long)]
    no_tokens: bool,

    /// Show detailed timing information
    #[arg(short, long)]
    timing: bool,
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
    let global_start = Instant::now();
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
    let discovery_start = Instant::now();
    let files = traversal::traverse(&options)?;
    let discovery_duration = discovery_start.elapsed();

    info!("Found {} files", files.len());
    if files.is_empty() {
        info!("No files found matching patterns.");
        return Ok(());
    }

    info!("Generating digest...");

    let output_dest = if cli.dry {
        OutputDestination::Null
    } else if cli.stdout {
        OutputDestination::Stdout
    } else {
        let path = cli
            .output
            .unwrap_or_else(|| env::current_dir().unwrap().join(ingest::DIGEST_FILENAME));
        OutputDestination::File(path)
    };

    let content_decorator: Box<dyn ContentDecorator> = match cli.preset {
        Preset::Default => Box::new(DefaultDecorator),
        Preset::Markdown => Box::new(MarkdownDecorator),
        Preset::Xml => Box::new(XmlDecorator),
    };

    let global_decorator = FileTreeDecorator {
        root: options.root.clone(),
        mode: cli.prologue,
    };

    let ingest_start = Instant::now();
    let _ingest_metrics = ingest::ingest(
        &files,
        output_dest,
        content_decorator.as_ref(),
        Some(&global_decorator),
        !cli.no_tokens,
    )?;
    let ingest_duration = ingest_start.elapsed();

    info!("Done!");

    if cli.timing {
        println!("\nTiming Summary:");
        println!("----------------------------------------");
        println!("Discovery:      {discovery_duration:?}");
        println!("Ingestion:      {ingest_duration:?}");
        println!("Total Runtime:  {:?}", global_start.elapsed());
        println!("----------------------------------------");
    }

    Ok(())
}
