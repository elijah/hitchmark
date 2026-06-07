//! Hookmarks CLI — command-line tool for stable, addressable document links.
//!
//! Usage:
//!   hk link <uri-a> <uri-b> [--note "..."]     Create a bidirectional link
//!   hk list <uri>                               List all links for a resource
//!   hk open <hook-uri>                          Open a hook:// URI
//!   hk file <path>                              Print the hook:// URI for a file
//!   hk purple <file> [--format markdown|json]   Annotate file with purple numbers

use clap::{Parser, Subcommand};

mod commands;
mod config;
mod path;

#[derive(Parser)]
#[command(name = "hk")]
#[command(version)]
#[command(about = "Hookmarks CLI: stable links to documents and paragraphs")]
#[command(
    long_about = "hk creates and manages hook:// URIs — stable, addressable links \
    to files, web pages, and intra-document locations.\n\nExamples:\n  \
    hk file ~/docs/note.md\n  \
    hk link ~/docs/note.md ~/docs/reference.md --note \"See this section\"\n  \
    hk list ~/docs/note.md\n  \
    hk open \"hook://file/L3Zhci9sb2cvZG9jcy9ub3RlLm1k\""
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a bidirectional link between two resources
    Link {
        /// First URI (or file path)
        uri_a: String,

        /// Second URI (or file path)
        uri_b: String,

        /// Optional note attached to the link
        #[arg(long)]
        note: Option<String>,
    },

    /// List all links for a resource
    List {
        /// URI to query (or file path)
        uri: String,
    },

    /// Resolve and open a hook:// URI
    Open {
        /// Hook URI to open
        uri: String,
    },

    /// Print the hook:// URI for a file
    File {
        /// Path to file
        path: String,
    },

    /// Annotate a file with purple numbers (stable paragraph IDs)
    Purple {
        /// Path to file
        path: String,

        /// Output format: markdown (default) or json
        #[arg(long, default_value = "markdown")]
        format: String,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let config = config::Config::load()?;
    config.ensure_dir()?;

    match cli.command {
        Commands::Link { uri_a, uri_b, note } => {
            let args = commands::link::LinkArgs { uri_a, uri_b, note };
            commands::link::execute(args, &config.store_path)?;
        }

        Commands::List { uri } => {
            let args = commands::list::ListArgs { uri };
            commands::list::execute(args, &config.store_path)?;
        }

        Commands::Open { uri } => {
            let args = commands::open::OpenArgs { uri };
            commands::open::execute(args)?;
        }

        Commands::File { path } => {
            let args = commands::file::FileArgs { path };
            commands::file::execute(args)?;
        }

        Commands::Purple { path, format } => {
            let args = commands::purple::PurpleArgs { path, format };
            commands::purple::execute(args)?;
        }
    }

    Ok(())
}
