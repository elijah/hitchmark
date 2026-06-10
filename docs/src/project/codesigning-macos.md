# macOS Code Signing & Notarization

Distributing Hitchmark outside the Mac App Store requires an **Apple Developer ID** certificate and notarization by Apple's servers. This page covers the full workflow.

## Prerequisites

- Apple Developer Program membership ($99/year)
- Xcode Command Line Tools installed
- `hk` binary compiled for the target arch(es) — see `apps/macos/` for the SwiftUI app

## Step 1 — Obtain a Developer ID Certificate

1. Open **Keychain Access → Certificate Assistant → Request a Certificate From a Certificate Authority**
2. In [developer.apple.com/account/resources/certificates](https://developer.apple.com/account/resources/certificates), create a **Developer ID Application** certificate
3. Download and double-click to install it in your keychain
4. Verify: `security find-identity -p codesigning -v`

You should see something like:
```
1) ABCDEF1234567890ABCDEF1234567890ABCDEF12 "Developer ID Application: Your Name (TEAMID)"
```

## Step 2 — Sign the Binary

```bash
# Sign hk with hardened runtime (required for notarization)
codesign \
  --sign "Developer ID Application: Your Name (TEAMID)" \
  --options runtime \
  --timestamp \
  --verbose \
  target/release/hk
```

Verify:
```bash
codesign --verify --deep --strict --verbose=2 target/release/hk
spctl --assess --type execute target/release/hk
```

## Step 3 — Notarize

Notarization requires uploading to Apple's servers. You'll need an **App Store Connect API key** (JSON file).

```bash
# Create a zip for submission (notarytool requires zip, dmg, or pkg)
zip hk.zip target/release/hk

# Submit — replace KEY_ID, ISSUER_ID, KEY_FILE with your API key details
xcrun notarytool submit hk.zip \
  --key "$KEY_FILE" \
  --key-id "$KEY_ID" \
  --issuer "$ISSUER_ID" \
  --wait

# Staple (only possible for app bundles/packages, not bare binaries)
# For a bare binary, notarization result is checked online by macOS Gatekeeper
```

## Step 4 — Automate in CI

Add these secrets to your GitHub repository (Settings → Secrets → Actions):

| Secret | Value |
|--------|-------|
| `APPLE_SIGN_IDENTITY` | `Developer ID Application: Your Name (TEAMID)` |
| `APPLE_API_KEY_JSON` | Contents of your App Store Connect API key `.p8` file |
| `APPLE_API_KEY_ID` | Key ID (e.g. `ABCD1234EF`) |
| `APPLE_API_ISSUER_ID` | Issuer UUID from App Store Connect |

The CI workflow stub at `.github/workflows/release-macos.yml` uses these secrets.

## Step 5 — Sign the macOS App Bundle

For the SwiftUI app (`apps/macos/`):

```bash
swift build -c release --package-path apps/macos
APP_PATH=apps/macos/.build/release/Hitchmark.app

codesign \
  --sign "Developer ID Application: Your Name (TEAMID)" \
  --entitlements apps/macos/Hitchmark.entitlements \
  --options runtime \
  --deep \
  --timestamp \
  --verbose \
  "$APP_PATH"

# Package as DMG, notarize, and staple
hdiutil create -srcfolder "$APP_PATH" -volname "Hitchmark" hitchmark.dmg
xcrun notarytool submit hitchmark.dmg --key ... --wait
xcrun stapler staple hitchmark.dmg
```

## Entitlements

Create `apps/macos/Hitchmark.entitlements`:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.security.cs.allow-jit</key><false/>
  <key>com.apple.security.cs.allow-unsigned-executable-memory</key><false/>
  <key>com.apple.security.cs.disable-library-validation</key><false/>
</dict>
</plist>
```

## Troubleshooting

- **`CSSMERR_TP_CERT_REVOKED`** — Your signing cert is revoked; renew at developer.apple.com
- **Gatekeeper blocks binary** — Run `xattr -d com.apple.quarantine /path/to/hk` for local testing
- **Notarization fails: "binary not signed"** — Ensure `--options runtime` is present in codesign call
