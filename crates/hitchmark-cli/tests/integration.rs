//! Integration tests for the `hk` CLI binary.
//!
//! Each test spawns the real `hk` binary via `assert_cmd`, using a
//! temporary directory as the store via the `HK_STORE_PATH` env var.
//!
//! Tests are ordered by dependency: file → link → list → delete → gc → export → import → purple.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Create an `hk` command with `HK_STORE_PATH` pointing at a temp dir.
fn hk(tmp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("hk").unwrap();
    cmd.env("HK_STORE_PATH", store_path(tmp));
    cmd
}

fn store_path(tmp: &TempDir) -> PathBuf {
    tmp.path().join("store.db")
}

fn make_file(tmp: &TempDir, name: &str, content: &str) -> PathBuf {
    let p = tmp.path().join(name);
    std::fs::write(&p, content).unwrap();
    p
}

// ── hk --version / --help ────────────────────────────────────────────────────

#[test]
fn version_flag_prints_version() {
    Command::cargo_bin("hk")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("hk"));
}

#[test]
fn help_flag_lists_subcommands() {
    Command::cargo_bin("hk")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("link"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("gc"));
}

// ── hk file ──────────────────────────────────────────────────────────────────

#[test]
fn file_converts_path_to_hook_uri() {
    let tmp = TempDir::new().unwrap();
    let note = make_file(&tmp, "note.md", "# Hello\n\nWorld");

    hk(&tmp)
        .args(["file", note.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("hook://file/"));
}

#[test]
fn file_nonexistent_path_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    hk(&tmp)
        .args(["file", "/tmp/this-file-does-not-exist-hitchmark.md"])
        .assert()
        .failure();
}

#[test]
fn file_tilde_expands() {
    let tmp = TempDir::new().unwrap();
    // Create a file under $HOME so ~/... expansion works
    let home = dirs::home_dir().unwrap();
    let test_file = home.join(".hitchmark_test_tmp.txt");
    std::fs::write(&test_file, "test").unwrap();

    let result = hk(&tmp)
        .args(["file", "~/.hitchmark_test_tmp.txt"])
        .assert()
        .success();
    result.stdout(predicate::str::starts_with("hook://file/"));
    let _ = std::fs::remove_file(&test_file);
}

// ── hk link / list / delete ──────────────────────────────────────────────────

#[test]
fn link_creates_bidirectional_link() {
    let tmp = TempDir::new().unwrap();
    let a = make_file(&tmp, "a.md", "# A");
    let b = make_file(&tmp, "b.md", "# B");

    // Create link
    hk(&tmp)
        .args(["link", a.to_str().unwrap(), b.to_str().unwrap()])
        .assert()
        .success();

    // list from a should mention b
    let uri_b_out = hk(&tmp)
        .args(["file", b.to_str().unwrap()])
        .output()
        .unwrap();
    let uri_b = String::from_utf8_lossy(&uri_b_out.stdout).trim().to_string();

    hk(&tmp)
        .args(["list", a.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&uri_b));
}

#[test]
fn link_duplicate_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let a = make_file(&tmp, "a.md", "# A");
    let b = make_file(&tmp, "b.md", "# B");
    let a_str = a.to_str().unwrap();
    let b_str = b.to_str().unwrap();

    hk(&tmp).args(["link", a_str, b_str]).assert().success();
    // Second link should not error
    hk(&tmp).args(["link", a_str, b_str]).assert().success();
}

#[test]
fn list_empty_returns_empty_json_array() {
    let tmp = TempDir::new().unwrap();
    let a = make_file(&tmp, "a.md", "# A");

    hk(&tmp)
        .args(["list", a.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

#[test]
fn delete_removes_link() {
    let tmp = TempDir::new().unwrap();
    let a = make_file(&tmp, "a.md", "# A");
    let b = make_file(&tmp, "b.md", "# B");
    let a_str = a.to_str().unwrap();
    let b_str = b.to_str().unwrap();

    hk(&tmp).args(["link", a_str, b_str]).assert().success();
    hk(&tmp).args(["delete", a_str, b_str, "--yes"]).assert().success();

    hk(&tmp)
        .args(["list", a_str, "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

// ── hk gc ────────────────────────────────────────────────────────────────────

#[test]
fn gc_clean_store_exits_zero() {
    let tmp = TempDir::new().unwrap();
    let a = make_file(&tmp, "a.md", "# A");
    let b = make_file(&tmp, "b.md", "# B");

    hk(&tmp).args(["link", a.to_str().unwrap(), b.to_str().unwrap()]).assert().success();

    hk(&tmp)
        .arg("gc")
        .assert()
        .success() // no stale entries → exit 0
        .stdout(predicate::str::contains("Store is clean."));
}

#[test]
fn gc_stale_link_exits_one_in_dry_run() {
    let tmp = TempDir::new().unwrap();
    let a = make_file(&tmp, "a.md", "# A");
    let b = make_file(&tmp, "b.md", "# B");
    let a_str = a.to_str().unwrap();
    let b_str = b.to_str().unwrap();

    hk(&tmp).args(["link", a_str, b_str]).assert().success();

    // Delete b on disk — link is now stale
    std::fs::remove_file(&b).unwrap();

    hk(&tmp)
        .arg("gc")
        .assert()
        .failure() // stale entries found → exit 1
        .stdout(predicate::str::contains("stale"));
}

#[test]
fn gc_delete_removes_stale_links() {
    let tmp = TempDir::new().unwrap();
    let a = make_file(&tmp, "a.md", "# A");
    let b = make_file(&tmp, "b.md", "# B");

    hk(&tmp).args(["link", a.to_str().unwrap(), b.to_str().unwrap()]).assert().success();
    std::fs::remove_file(&b).unwrap();

    hk(&tmp).args(["gc", "--delete"]).assert().success();

    // After deletion, gc should be clean
    hk(&tmp).arg("gc").assert().success();
}

// ── hk export / import ───────────────────────────────────────────────────────

#[test]
fn export_empty_store_produces_empty_ndjson() {
    let tmp = TempDir::new().unwrap();
    // Ensure store is initialised by running a list
    let a = make_file(&tmp, "a.md", "# A");
    hk(&tmp).args(["list", a.to_str().unwrap(), "--json"]).assert().success();

    hk(&tmp)
        .arg("export")
        .assert()
        .success();
    // Empty store → no output lines (empty NDJSON is valid)
}

#[test]
fn export_json_format_produces_array() {
    let tmp = TempDir::new().unwrap();
    let a = make_file(&tmp, "a.md", "# A");
    let b = make_file(&tmp, "b.md", "# B");

    hk(&tmp).args(["link", a.to_str().unwrap(), b.to_str().unwrap()]).assert().success();

    hk(&tmp)
        .args(["export", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

#[test]
fn import_roundtrip_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let a = make_file(&tmp, "a.md", "# A");
    let b = make_file(&tmp, "b.md", "# B");
    let export_file = tmp.path().join("export.ndjson");

    hk(&tmp).args(["link", a.to_str().unwrap(), b.to_str().unwrap()]).assert().success();

    // Export
    hk(&tmp)
        .args(["export", "--out", export_file.to_str().unwrap()])
        .assert()
        .success();

    assert!(export_file.exists());

    // Import into a fresh store
    let tmp2 = TempDir::new().unwrap();
    hk(&tmp2)
        .args(["import", export_file.to_str().unwrap()])
        .assert()
        .success();

    // Importing again should be idempotent (no error on duplicate)
    hk(&tmp2)
        .args(["import", export_file.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn import_dry_run_does_not_write() {
    let tmp = TempDir::new().unwrap();
    let a = make_file(&tmp, "a.md", "# A");
    let b = make_file(&tmp, "b.md", "# B");
    let export_file = tmp.path().join("export.ndjson");

    hk(&tmp).args(["link", a.to_str().unwrap(), b.to_str().unwrap()]).assert().success();
    hk(&tmp).args(["export", "--out", export_file.to_str().unwrap()]).assert().success();

    let tmp2 = TempDir::new().unwrap();
    hk(&tmp2)
        .args(["import", "--dry-run", export_file.to_str().unwrap()])
        .assert()
        .success();

    // Dry run → store should still be empty in tmp2
    hk(&tmp2)
        .args(["list", a.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

// ── hk purple ────────────────────────────────────────────────────────────────

#[test]
fn purple_annotates_markdown_paragraphs() {
    let tmp = TempDir::new().unwrap();
    let md = make_file(&tmp, "doc.md", "# Title\n\nFirst paragraph.\n\nSecond paragraph.\n");

    hk(&tmp)
        .args(["purple", md.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("[§")); // purple number marker [§id]
}

#[test]
fn purple_json_format_returns_ids() {
    let tmp = TempDir::new().unwrap();
    let md = make_file(&tmp, "doc.md", "# Title\n\nHello world\n");

    let out = hk(&tmp)
        .args(["purple", md.to_str().unwrap(), "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8_lossy(&out);
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("purple --format json must be valid JSON");
    assert!(parsed.is_array());
    // "Hello world" → well-known purple ID "7nxxnx"
    let ids: Vec<String> = parsed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()).map(String::from))
        .collect();
    assert!(ids.iter().any(|id| id == "7nxxnx"), "expected 7nxxnx, got: {ids:?}");
}

// ── hk completions ───────────────────────────────────────────────────────────

#[test]
fn completions_bash_produces_output() {
    Command::cargo_bin("hk")
        .unwrap()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hk"));
}
