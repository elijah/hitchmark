//! `hk file` — print the hook:// URI for a file.

#[derive(clap::Parser)]
pub struct FileArgs {
    /// Path to file
    pub path: String,
}

pub fn execute(args: FileArgs) -> anyhow::Result<()> {
    let uri = crate::path::path_to_uri(&args.path)?;
    println!("{uri}");
    Ok(())
}
