//! Hookmarks CLI: the `hk` command-line tool.
//!
//! Usage:
//!   hk link <uri-a> <uri-b> [--note "..."]
//!   hk list <uri>
//!   hk open <hook-uri>
//!   hk file <path>

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hk")]
#[command(version)]
#[command(about = "Hookmarks CLI: stable links to documents and paragraphs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a bidirectional link between two resources
    Link {
        /// First URI
        uri_a: String,
        /// Second URI
        uri_b: String,
        /// Optional note attached to the link
        #[arg(long)]
        note: Option<String>,
    },
    /// List all links for a resource
    List {
        /// URI to query
        uri: String,
    },
    /// Resolve and open a hook:// URI
    Open {
        /// Hook URI to open
        uri: String,
    },
    /// Print the hook:// URI for a local file
    File {
        /// Path to file
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Link { uri_a, uri_b, note } => {
            eprintln!("link {uri_a} <-> {uri_b} (note: {note:?})");
        }
        Commands::List { uri } => {
            eprintln!("list {uri}");
        }
        Commands::Open { uri } => {
            eprintln!("open {uri}");
        }
        Commands::File { path } => {
            eprintln!("file {path}");
        }
    }
}
