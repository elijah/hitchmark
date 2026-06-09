//! `hk watch` — watch bookmarked file locations and auto-repair paths on rename/move.
//!
//! Uses `notify` (FSEvents on macOS, inotify on Linux, ReadDirectoryChangesW on Windows)
//! to watch the parent directories of all bookmarked files. When a rename event is
//! detected, the bookmark database is updated automatically.
//!
//! On file removal, a warning is logged but no action is taken — use `hk gc` to
//! clean up bookmarks pointing to deleted files.

use hitchmark_core::LinkStore;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(clap::Parser)]
pub struct WatchArgs {
    /// Print each filesystem event as it is processed
    #[arg(long)]
    pub verbose: bool,
}

pub fn execute(args: WatchArgs, store_path: &Path) -> anyhow::Result<()> {
    let store_path = store_path.to_path_buf();

    println!("hk watch: starting (Ctrl+C to stop)");

    // path → list of bookmark IDs that point to it
    let path_to_ids: Arc<Mutex<HashMap<PathBuf, Vec<String>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    load_bookmarks(&store_path, &path_to_ids)?;

    let path_to_ids_clone = Arc::clone(&path_to_ids);
    let store_path_clone = store_path.clone();
    let verbose = args.verbose;

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })?;

    register_watches(&mut watcher, &path_to_ids)?;

    let mut last_refresh = Instant::now();
    let refresh_interval = Duration::from_secs(60);

    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(event) => {
                handle_event(event, &path_to_ids_clone, &store_path_clone, verbose)?;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                anyhow::bail!("File watcher channel disconnected unexpectedly");
            }
        }

        if last_refresh.elapsed() >= refresh_interval {
            if verbose {
                println!("[watch] refreshing bookmark list...");
            }
            load_bookmarks(&store_path, &path_to_ids)?;
            register_watches(&mut watcher, &path_to_ids)?;
            last_refresh = Instant::now();
        }
    }
}

fn load_bookmarks(
    store_path: &Path,
    path_to_ids: &Arc<Mutex<HashMap<PathBuf, Vec<String>>>>,
) -> anyhow::Result<()> {
    let store = LinkStore::open(store_path)?;
    let bookmarks = store.list_bookmarks()?;
    let mut map = path_to_ids.lock().unwrap();
    map.clear();
    for bm in bookmarks {
        map.entry(PathBuf::from(&bm.file_path))
            .or_default()
            .push(bm.id);
    }
    Ok(())
}

fn register_watches(
    watcher: &mut RecommendedWatcher,
    path_to_ids: &Arc<Mutex<HashMap<PathBuf, Vec<String>>>>,
) -> anyhow::Result<()> {
    let map = path_to_ids.lock().unwrap();
    let mut dirs: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    for path in map.keys() {
        if let Some(parent) = path.parent() {
            if parent.exists() {
                dirs.insert(parent.to_path_buf());
            }
        }
    }
    for dir in dirs {
        let _ = watcher.watch(&dir, RecursiveMode::NonRecursive);
    }
    Ok(())
}

fn handle_event(
    event: Event,
    path_to_ids: &Arc<Mutex<HashMap<PathBuf, Vec<String>>>>,
    store_path: &Path,
    verbose: bool,
) -> anyhow::Result<()> {
    match event.kind {
        EventKind::Modify(notify::event::ModifyKind::Name(
            notify::event::RenameMode::Both,
        )) => {
            if event.paths.len() == 2 {
                let from = &event.paths[0];
                let to = &event.paths[1];
                repair_bookmark(from, to, path_to_ids, store_path, verbose)?;
            }
        }
        EventKind::Remove(_) => {
            let map = path_to_ids.lock().unwrap();
            for path in &event.paths {
                if map.contains_key(path) {
                    eprintln!(
                        "[watch] WARNING: bookmarked file removed: {}\n  Run 'hk gc' to clean up.",
                        path.display()
                    );
                }
            }
        }
        other => {
            if verbose {
                println!("[watch] event: {other:?} — {:?}", event.paths);
            }
        }
    }
    Ok(())
}

fn repair_bookmark(
    from: &Path,
    to: &Path,
    path_to_ids: &Arc<Mutex<HashMap<PathBuf, Vec<String>>>>,
    store_path: &Path,
    verbose: bool,
) -> anyhow::Result<()> {
    let ids: Vec<String> = {
        let map = path_to_ids.lock().unwrap();
        map.get(from).cloned().unwrap_or_default()
    };

    if ids.is_empty() {
        return Ok(());
    }

    let store = LinkStore::open(store_path)?;
    let new_path = to
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Renamed path is not valid UTF-8"))?;

    for id in &ids {
        store.update_bookmark_path(id, new_path)?;
        println!(
            "[watch] updated bookmark {id}: {} → {}",
            from.display(),
            to.display()
        );
    }

    let mut map = path_to_ids.lock().unwrap();
    if let Some(entries) = map.remove(from) {
        map.insert(to.to_path_buf(), entries);
    }

    if verbose {
        println!("[watch] repaired {} bookmark(s) for rename", ids.len());
    }

    Ok(())
}

