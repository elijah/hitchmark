//! Hookmarks daemon: Linux background service for hook:// URI handling.
//!
//! On Linux, registers as a DBus service (org.not_hookmarks.Daemon) and handles
//! hook:// URIs via xdg-open integration.

#[cfg(target_os = "linux")]
fn main() {
    println!("Hookmarks daemon starting...");
    // TODO: Implement Linux daemon with zbus and DBus interface
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("Hookmarks daemon is only supported on Linux");
    std::process::exit(1);
}
