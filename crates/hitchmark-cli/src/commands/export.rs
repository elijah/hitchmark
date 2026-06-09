//! `hk export` — export links and bookmarks to NDJSON or JSON.
//!
//! NDJSON (newline-delimited JSON) is the default: one record per line,
//! easy to pipe, grep, and process with `jq`. Use `--format json` for a
//! pretty-printed JSON array.

use hitchmark_core::LinkStore;
use std::io::Write;
use std::path::Path;

#[derive(clap::Parser)]
pub struct ExportArgs {
    /// Output format: ndjson (default, one record per line) or json (pretty array)
    #[arg(long, default_value = "ndjson", value_parser = ["ndjson", "json"])]
    pub format: String,

    /// Restrict export to links or bookmarks only
    #[arg(long, value_name = "TYPE", value_parser = ["links", "bookmarks"])]
    pub only: Option<String>,

    /// Write to FILE instead of stdout
    #[arg(long, value_name = "FILE")]
    pub out: Option<String>,
}

pub fn execute(args: ExportArgs, store_path: &Path) -> anyhow::Result<()> {
    let store = LinkStore::open(store_path)?;

    let export_links = args.only.as_deref() != Some("bookmarks");
    let export_bookmarks = args.only.as_deref() != Some("links");

    let mut records: Vec<serde_json::Value> = Vec::new();

    if export_links {
        for link in store.list_all_links()? {
            records.push(serde_json::json!({
                "type": "link",
                "source": link.source,
                "target": link.target,
                "note": link.note,
                "created_at": link.created_at
            }));
        }
    }

    if export_bookmarks {
        for bm in store.list_bookmarks()? {
            records.push(serde_json::json!({
                "type": "bookmark",
                "id": bm.id,
                "file_path": bm.file_path,
                "created_at": bm.created_at
            }));
        }
    }

    let output: Box<dyn Write> = match &args.out {
        Some(path) => Box::new(
            std::fs::File::create(path)
                .map_err(|e| anyhow::anyhow!("Cannot create output file '{path}': {e}"))?,
        ),
        None => Box::new(std::io::stdout()),
    };
    let mut writer = std::io::BufWriter::new(output);

    match args.format.as_str() {
        "json" => {
            writeln!(writer, "{}", serde_json::to_string_pretty(&records)?)?;
        }
        _ => {
            for record in &records {
                writeln!(writer, "{}", serde_json::to_string(record)?)?;
            }
        }
    }

    Ok(())
}

