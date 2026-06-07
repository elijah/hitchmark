# Security Policy

## Reporting a Vulnerability

**Do not open a public issue for security vulnerabilities.** Instead, please email the maintainers directly. We take security seriously and will respond promptly.

### What to Include

- Description of the vulnerability
- Steps to reproduce (if applicable)
- Potential impact
- Suggested fix (if you have one)

## Security Considerations for Users

### Local Storage

- Link data and purple-number mappings are stored in SQLite at `~/.config/hookmarks/store.db` (configurable)
- This database is **not encrypted by default** — treat it like any local application database
- If you sync this database via cloud services (iCloud, Dropbox), those services' security practices apply
- Do not store sensitive credentials in link metadata or notes

### URI Security

- `hook://` URIs can reference any file on your system that your user can access
- Be cautious when opening URIs from untrusted sources — they can open any file
- The CLI (`hk open`) respects your system's file permissions; it cannot access files your user account cannot read

### macOS App

- The app registers the `hook://` URL scheme globally
- Any application can invoke `hook://` URIs via `NSWorkspace.openURL()` or similar
- The app performs no additional validation beyond file permissions

### Linux Daemon

- The daemon exposes a DBus interface at `org.not_hookmarks.Daemon`
- Any process running under your user can call DBus methods
- The daemon respects file permissions enforced by the kernel

## Dependencies

We keep our dependency tree minimal:
- **Core library**: `rusqlite`, `serde`, `sha2`, `base58`, `thiserror`, `url`
- **CLI**: adds `clap`, `opener`, `dirs`
- **Daemon**: adds `zbus`, `tokio`, `notify`, `xdg`

We regularly audit dependencies with `cargo audit`. If a vulnerability is found in a dependency:
1. We update the dependency if a patch is available
2. If not, we file an issue with the dependency maintainers
3. We document the issue here until resolved

## Code Security

- We forbid `unsafe` code in the workspace (enforced at lint time)
- All input validation is performed at public API boundaries
- We use strong, industry-standard libraries where possible (SHA-256, base58)
- URIs are validated against the spec before processing

## Release Process

- Releases are tagged and signed with GPG
- CI (GitHub Actions) builds and tests all changes before release
- Release notes include security updates (if any)

## Version Support

- **Latest release**: receives all security updates
- **Previous major version**: receives critical security fixes only
- **Older versions**: no longer supported

---

**Last updated**: 2025-06-06
