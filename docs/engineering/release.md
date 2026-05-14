# Release

## TL;DR

```bash
# One-time setup
pnpm tauri signer generate -w ~/.tauri/devdock.key
# Copy the printed public key into src-tauri/tauri.conf.json
# Set "active": true in the same plugins.updater section.

# Per-release
# 1. Bump version in package.json AND src-tauri/tauri.conf.json
# 2. Build with the signing key in your environment:
export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/devdock.key)"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="<your-password>"
pnpm tauri build --target universal-apple-darwin
# 3. Upload artifacts + latest.json to GitHub Releases (see below)
```

## Local / dev builds

```bash
pnpm tauri build
```

Produces an unsigned `.app` and `.dmg` under
`src-tauri/target/release/bundle/`. Suitable for local install and for handing
to a small alpha group who can right-click → Open to bypass Gatekeeper once.

## Auto-update setup (one-time)

### 1. Generate a signing key

```bash
pnpm tauri signer generate -w ~/.tauri/devdock.key
```

This writes the **private** key (encrypted with the password you choose). The
command also prints the **public** key as a single base64 line. Treat the
private key like a release credential — losing it means existing installs can
never validate updates again.

### 2. Configure `tauri.conf.json`

Edit `src-tauri/tauri.conf.json` under `plugins.updater`:

```json
"updater": {
  "active": true,
  "endpoints": [
    "https://github.com/<your-user>/<your-repo>/releases/latest/download/latest.json"
  ],
  "pubkey": "<paste the public key here>"
}
```

The `pubkey` is committed to git. The private key never is.

### 3. Pick a hosting target

GitHub Releases is the easiest free path. The flow:

1. `pnpm tauri build` with the signing env vars produces:
   - `DevDock_X.Y.Z_<arch>.dmg` — the installer
   - `DevDock_X.Y.Z_<arch>.app.tar.gz` — the update payload
   - `DevDock_X.Y.Z_<arch>.app.tar.gz.sig` — Tauri signature
2. Upload all three to a GitHub Release tagged `vX.Y.Z`.
3. Also upload a `latest.json` manifest (see template below).
4. Existing installs query the endpoint, compare versions, fetch and verify.

S3 / Cloudflare R2 / any static host works the same way — only the URL changes.

### 4. `latest.json` template

```json
{
  "version": "0.2.0",
  "notes": "What's new in this release.",
  "pub_date": "2026-05-13T12:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "<contents of DevDock_0.2.0_aarch64.app.tar.gz.sig>",
      "url": "https://github.com/<user>/<repo>/releases/download/v0.2.0/DevDock_0.2.0_aarch64.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "<contents of DevDock_0.2.0_x64.app.tar.gz.sig>",
      "url": "https://github.com/<user>/<repo>/releases/download/v0.2.0/DevDock_0.2.0_x64.app.tar.gz"
    }
  }
}
```

For universal builds, both target keys point at the same `universal` archive.

## Code signing & notarization (separate from updater signing)

Updater signing protects against tampered update payloads. Apple code signing
protects against Gatekeeper warnings on first launch. Both are needed for a
polished public release.

1. Apple Developer account ($99/year).
2. Generate a "Developer ID Application" certificate via the Developer portal;
   install in Keychain.
3. Configure `bundle.macOS` in `tauri.conf.json`:

   ```json
   "macOS": {
     "minimumSystemVersion": "12.0",
     "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
     "providerShortName": "TEAM_ID",
     "entitlements": null
   }
   ```

4. Set env vars before `pnpm tauri build`:

   ```bash
   export APPLE_ID="you@example.com"
   export APPLE_PASSWORD="<app-specific password>"
   export APPLE_TEAM_ID="TEAM_ID"
   ```

   Tauri runs `codesign` + `notarytool` automatically when those are present.

## Versioning

- Frontend version in `package.json`.
- Bundle version in `src-tauri/tauri.conf.json` (`version`).

Bump both together. Tag the commit `vX.Y.Z`. Pre-1.0 the project follows
`0.MINOR.PATCH` where MINOR may include breaking schema changes gated by
migrations.

## Release checklist

- [ ] All tests pass (`pnpm test`, `cargo test`).
- [ ] All lints pass (`cargo clippy --all-targets -- -D warnings`).
- [ ] Versions bumped in `package.json` and `src-tauri/tauri.conf.json`.
- [ ] Migrations added (not edited) for any schema change.
- [ ] Docs updated alongside behavior changes.
- [ ] **`CHANGELOG.md` updated**: `[Unreleased]` renamed to `[X.Y.Z] - YYYY-MM-DD`,
      a fresh empty `[Unreleased]` section added above it, footnote links updated.
      The AI policy in `AGENTS.md` requires this on every version bump.
- [ ] Signing env vars exported (`TAURI_SIGNING_PRIVATE_KEY`,
      `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`).
- [ ] `pnpm tauri build --target universal-apple-darwin` succeeds.
- [ ] Built `.app` launches and the tray icon appears.
- [ ] GitHub Release published with `.dmg`, `.app.tar.gz`, `.app.tar.gz.sig`, and `latest.json`.
- [ ] In-app "Check for updates" detects the new version from a previous install.
