//! `hk import` — import links and bookmarks from NDJSON or JSON.
//!
//! Import is idempotent: duplicate links and bookmarks are silently skipped.
//! Use `--dry-run` to validate a file without writing anything.

use hitchmark_core::{Error, LinkStore};
use std::io::{Read};
use std::path::Path;

#[derive(clap::Parser)]
pub struct ImportArgs {
    /// Input file path (use - for stdin)
    pub file: String,

    /// Input format: ndjson (default) or json (array)
    #[arg(long, default_value = "ndjson", value_parser = ["ndjson", "json"])]
    pub format: String,

    /// Validate format and report counts without writing to the store
    #[arg(long)]
    pub dry_run: bool,
}

pub fn execute(args: ImportArgs, store_path: &Path) -> anyhow::Result<()> {
    let records = read_records(&args.file, &args.format)?;

    let total = records.len();
    let mut link_count = 0usize;
    let mut bookmark_count = 0usize;
    let mut skipped = 0usize;

    // Validate record types before opening the store
    for (i, rec) in records.iter().enumerate() {
        let typ = rec
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Record {i}: missing 'type' field"))?;
        match typ {
            "link" => {
                require_str(rec, "source", i)?;
                require_str(rec, "target", i)?;
                link_count += 1;
            }
            "bookmark" => {
                require_str(rec, "id", i)?;
                require_str(rec, "file_path", i)?;
                bookmark_count += 1;
            }
            other => anyhow::bail!("Record {i}: unknown type '{other}'"),
        }
    }

    if args.dry_run {
        println!(
            "Dry run: {total} records ({link_count} links, {bookmark_count} bookmarks) — no changes written."
        );
        return Ok(());
    }

    let store = LinkStore::open(store_path)?;

    for rec in &records {
        match rec["type"].as_str().unwrap() {
            "link" => {
                let source = rec["source"].as_str().unwrap();
                let target = rec["target"].as_str().unwrap();
                let note = rec.get("note").and_then(|v| v.as_str());
                match store.create_link(source, target, note) {
                    Ok(_) => {}
                    Err(Error::LinkAlreadyExists { .. }) => skipped += 1,
                    Err(e) => return Err(e.into()),
                }
            }
            "bookmark" => {
                let id = rec["id"].as_str().unwrap();
                let file_path = rec["file_path"].as_str().unwrap();
                match store.import_bookmark(id, file_path) {
                    Ok(_) => {}
                    Err(Error::BookmarkAlreadyExists { .. }) => skipped += 1,
                    Err(e) => return Err(e.into()),
                }
            }
            _ => unreachable!(),
        }
    }

    let imported = total - skipped;
    println!("Imported {imported} record(s) ({skipped} skipped as duplicates).");

    Ok(())
}

fn read_records(file: &str, format: &str) -> anyhow::Result<Vec<serde_json::Value>> {
    let content = if file == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(file)
            .map_err(|e| anyhow::anyhow!("Cannot read '{file}': {e}"))?
    };

    match format {
        "json" => {
            let arr: Vec<serde_json::Value> = serde_json::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Invalid JSON array: {e}"))?;
            Ok(arr)
        }
        _ => {
            // NDJSON: one JSON object per non-empty line
            content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .enumerate()
                .map(|(i, line)| {
                    serde_json::from_str(line)
                        .map_err(|e| anyhow::anyhow!("Line {}: invalid JSON: {e}", i + 1))
                })
                .collect()
        }
    }
}

fn require_str<'a>(
    rec: &'a serde_json::Value,
    field: &str,
    idx: usize,
) -> anyhow::Result<&'a str> {
    rec.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Record {idx}: missing or non-string field '{field}'"))
}

