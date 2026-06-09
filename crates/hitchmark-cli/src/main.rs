//! Hitchmark CLI — command-line tool for stable, addressable document links.
//!
//! Usage:
//!   hk link <uri-a> <uri-b> [--note "..."]     Create a bidirectional link
//!   hk list <uri> [--json]                      List all links for a resource
//!   hk delete <uri-a> <uri-b> [-y]             Remove a link
//!   hk open <hook-uri>                          Open a hook:// URI
//!   hk file <path> [--bookmark]                 Print the hook:// URI for a file
//!   hk bookmark <create|show|update|list>       Manage stored bookmark URIs
//!   hk gc [--delete] [--json]                   Garbage-collect stale entries
//!   hk export [--format ndjson|json]            Export links and bookmarks
//!   hk import <file>                            Import links and bookmarks
//!   hk purple <file> [--format markdown|json]   Annotate file with purple numbers
//!   hk serve [--port 2701] [--host 127.0.0.1]  Start local HTTP API server
//!   hk watch [--verbose]                        Watch bookmarks for file moves
//!   hk completions <shell>                      Print shell completion script
//!   hk manpage [--out <dir>]                    Generate hk(1) man page

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use clap_mangen::Man;

mod commands;
mod config;
mod path;

#[derive(Parser)]
#[command(name = "hk")]
#[command(version)]
#[command(about = "Hitchmark CLI: stable links to documents and paragraphs")]
#[command(
    long_about = "hk creates and manages hook:// URIs — stable, addressable links \
    to files, web pages, and intra-document locations.\n\nExamples:\n  \
    hk file ~/docs/note.md\n  \
    hk link ~/docs/note.md ~/docs/reference.md --note \"See this section\"\n  \
    hk list ~/docs/note.md\n  \
    hk list ~/docs/note.md --json\n  \
    hk delete ~/docs/note.md ~/docs/reference.md\n  \
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

        /// Output as JSON (machine-readable)
        #[arg(long)]
        json: bool,
    },

    /// Remove a bidirectional link
    Delete {
        /// First URI (or file path)
        uri_a: String,

        /// Second URI (or file path)
        uri_b: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
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

        /// Store and return a stable bookmark URI instead of a file URI
        #[arg(long)]
        bookmark: bool,
    },

    /// Manage stored bookmark URIs
    ///
    /// Bookmark URIs are stable identifiers for files that persist even after
    /// the file is renamed or moved (requires updating via `hk bookmark update`).
    Bookmark(commands::bookmark::BookmarkArgs),

    /// Garbage-collect stale links and bookmarks
    ///
    /// Reports (or removes with --delete) any links/bookmarks pointing to files
    /// that no longer exist. Exits 1 when stale entries are found.
    Gc(commands::gc::GcArgs),

    /// Export links and bookmarks to NDJSON or JSON
    Export(commands::export::ExportArgs),

    /// Import links and bookmarks from NDJSON or JSON
    Import(commands::import::ImportArgs),

    /// Annotate a file with purple numbers (stable paragraph IDs)
    Purple {
        /// Path to file
        path: String,

        /// Output format: markdown (default) or json
        #[arg(long, default_value = "markdown")]
        format: String,
    },

    /// Start a local HTTP API server (default: http://127.0.0.1:2701)
    Serve {
        /// Port to listen on
        #[arg(long, default_value_t = 2701)]
        port: u16,

        /// Host/IP to bind
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Watch bookmarked file locations and auto-repair paths on rename/move
    Watch(commands::watch::WatchArgs),

    /// Print shell completion script to stdout
    ///
    /// Example: hk completions bash >> ~/.bashrc
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Generate and install the hk(1) man page
    ///
    /// Writes to the given directory (default: /usr/local/share/man/man1 on Unix).
    /// Example: hk manpage --out /usr/local/share/man/man1
    Manpage {
        /// Directory to write hk.1 into
        #[arg(long, default_value_t = default_man_dir())]
        out: String,
    },
}

fn default_man_dir() -> String {
    if cfg!(windows) {
        std::env::var("USERPROFILE")
            .map(|h| format!(r"{h}\man\man1"))
            .unwrap_or_else(|_| r"C:\man\man1".to_string())
    } else {
        "/usr/local/share/man/man1".to_string()
    }
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

        Commands::List { uri, json } => {
            let args = commands::list::ListArgs { uri, json };
            commands::list::execute(args, &config.store_path)?;
        }

        Commands::Delete { uri_a, uri_b, yes } => {
            let args = commands::delete::DeleteArgs { uri_a, uri_b, yes };
            commands::delete::execute(args, &config.store_path)?;
        }

        Commands::Open { uri } => {
            let args = commands::open::OpenArgs { uri };
            commands::open::execute(args, &config.store_path)?;
        }

        Commands::File { path, bookmark } => {
            let args = commands::file::FileArgs { path, bookmark };
            commands::file::execute(args, &config.store_path)?;
        }

        Commands::Bookmark(args) => {
            commands::bookmark::execute(args, &config.store_path)?;
        }

        Commands::Gc(args) => {
            commands::gc::execute(args, &config.store_path)?;
        }

        Commands::Export(args) => {
            commands::export::execute(args, &config.store_path)?;
        }

        Commands::Import(args) => {
            commands::import::execute(args, &config.store_path)?;
        }

        Commands::Purple { path, format } => {
            let args = commands::purple::PurpleArgs { path, format };
            commands::purple::execute(args)?;
        }

        Commands::Serve { port, host } => {
            let args = commands::serve::ServeArgs { port, host };
            commands::serve::execute(args, &config.store_path)?;
        }

        Commands::Watch(args) => {
            commands::watch::execute(args, &config.store_path)?;
        }

        Commands::Completions { shell } => {
            generate(shell, &mut Cli::command(), "hk", &mut std::io::stdout());
        }

        Commands::Manpage { out } => {
            let out_dir = std::path::PathBuf::from(&out);
            std::fs::create_dir_all(&out_dir)
                .map_err(|e| anyhow::anyhow!("Cannot create directory {out}: {e}"))?;
            let man = Man::new(Cli::command());
            let dest = out_dir.join("hk.1");
            let mut f = std::fs::File::create(&dest)
                .map_err(|e| anyhow::anyhow!("Cannot write {}: {e}", dest.display()))?;
            man.render(&mut f)
                .map_err(|e| anyhow::anyhow!("Man page render failed: {e}"))?;
            println!("Wrote {}", dest.display());
        }
    }

    Ok(())
}
