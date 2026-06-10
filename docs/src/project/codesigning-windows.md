# Windows Code Signing (Authenticode)

Distributing `hk.exe` and `hitchmark-tray.exe` without signing triggers Windows SmartScreen warnings. This page covers Authenticode signing with `signtool.exe`.

## Prerequisites

- Windows 10/11 or Windows Server with Windows SDK installed (`signtool.exe`)
- An **Authenticode code signing certificate** from a trusted CA (e.g. DigiCert, Sectigo, GlobalSign)
  - Extended Validation (EV) certificates bypass SmartScreen immediately
  - Standard OV certificates build reputation over time
- The certificate installed in the Windows Certificate Store (Personal)

## Step 1 — Install signtool.exe

signtool.exe ships with the Windows SDK. Verify it's on PATH:
```powershell
where signtool
# e.g. C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe
```

## Step 2 — Sign the Binary

```powershell
signtool sign `
  /tr http://timestamp.digicert.com `
  /td sha256 `
  /fd sha256 `
  /n "Your Organization Name" `
  /v `
  target\release\hk.exe

signtool sign `
  /tr http://timestamp.digicert.com `
  /td sha256 `
  /fd sha256 `
  /n "Your Organization Name" `
  /v `
  apps\windows-tray\target\release\hitchmark-tray.exe
```

Verify:
```powershell
signtool verify /pa /v target\release\hk.exe
```

## Step 3 — Sign the MSI Installer

Build the MSI first (see `apps/windows/hitchmark.wxs`), then sign it:
```powershell
signtool sign `
  /tr http://timestamp.digicert.com `
  /td sha256 /fd sha256 `
  /n "Your Organization Name" `
  hitchmark-setup.msi
```

## Step 4 — Automate in CI

Add these secrets to GitHub (Settings → Secrets → Actions):

| Secret | Value |
|--------|-------|
| `WINDOWS_CERT_PFX_BASE64` | Base64-encoded `.pfx` certificate file |
| `WINDOWS_CERT_PASSWORD` | Password for the `.pfx` file |

The CI stub at `.github/workflows/release-windows.yml` uses these secrets to sign binaries before packaging.

Export your cert to PFX (from Cert Manager → Personal → right-click cert → Export):
```powershell
# Encode for GitHub secret
[Convert]::ToBase64String([IO.File]::ReadAllBytes("hitchmark.pfx")) | clip
```

## Step 5 — SmartScreen Reputation

Even after signing, SmartScreen may warn on first run until the binary accumulates download reputation. To accelerate:

1. Use an **EV certificate** — SmartScreen trust is immediate
2. Submit to [Microsoft's malware analysis portal](https://www.microsoft.com/en-us/wdsi/filesubmission) for manual review
3. Publish to winget (Microsoft Winget package manager) — improves reputation

## Winget Package

Create `apps/windows/hitchmark.winget.yaml` and submit a PR to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs):

```yaml
PackageIdentifier: Hitchmark.Hitchmark
PackageVersion: 0.3.0
PackageName: Hitchmark
Publisher: Hitchmark Contributors
License: MIT
ShortDescription: Stable, addressable links to documents via hook:// URIs
Installers:
  - Architecture: x64
    InstallerType: msi
    InstallerUrl: https://github.com/elijah/hitchmark/releases/download/v0.3.0/hitchmark-setup.msi
    InstallerSha256: <SHA256_AFTER_SIGNING>
ManifestType: singleton
ManifestVersion: 1.6.0
```
