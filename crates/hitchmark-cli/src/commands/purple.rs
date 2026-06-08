//! `hk purple` — annotate a file with purple numbers.

use hitchmark_core::purple::{split_paragraphs, PurpleNumberGenerator};
use std::fs;

#[derive(clap::Parser)]
pub struct PurpleArgs {
    /// Path to file to annotate
    pub path: String,

    /// Output format
    #[arg(long, default_value = "markdown")]
    pub format: String,
}

pub fn execute(args: PurpleArgs) -> anyhow::Result<()> {
    let content = fs::read_to_string(&args.path)?;
    let paragraphs = split_paragraphs(&content);

    let mut generator = PurpleNumberGenerator::new();
    let mut purple_map = Vec::new();

    for para in &paragraphs {
        let id = generator.generate(para)?;
        purple_map.push((para.clone(), id));
    }

    match args.format.as_str() {
        "markdown" => {
            for (para, id) in purple_map {
                println!("{}\n[§{}]\n", para, id.as_str());
            }
        }
        "json" => {
            let json: Vec<_> = purple_map
                .iter()
                .map(|(para, id)| {
                    serde_json::json!({
                        "id": id.as_str(),
                        "text": para,
                        "uri_fragment": format!("para-{}", id.as_str())
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => anyhow::bail!("Unknown format: {}", args.format),
    }

    Ok(())
}
