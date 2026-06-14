# Release procedure

## Signing key custody

A minisign keypair is used to sign every update artifact.

- **Private key**: `~/.tauri/basic-presence.key` — **never commit this file**. Store it in a password manager as a backup. Losing it makes it impossible to deliver updates to installed copies of the app.
- **Public key**: committed in `src-tauri/tauri.conf.json` → `plugins.updater.pubkey`.

> **Version source of truth**: `package.json` → `version` is the **single place** to bump. `src-tauri/tauri.conf.json` reads it via `"version": "../package.json"`, and that config value takes precedence over `src-tauri/Cargo.toml`. So the Rust crate version in `Cargo.toml` is **not** release-relevant and does not need bumping.
- **GitHub secret** `TAURI_SIGNING_PRIVATE_KEY`: content of the private key file.
- **GitHub secret** `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: password chosen during key generation (empty string if none was set).

## Releasing a new version

1. Bump the version in **`package.json`** → `version`. This is the **single source of truth**: `src-tauri/tauri.conf.json` reads it through `"version": "../package.json"`, so there is nothing else to sync (`Cargo.toml` is not release-relevant — see the note above).
2. Commit: `git commit -am "chore: bump to vX.Y.Z"`.
3. Tag and push: `git tag vX.Y.Z && git push --tags`.
4. The `.github/workflows/release.yml` workflow builds four matrix jobs (macOS arm64, macOS x86_64, Linux, Windows), signs updater artifacts, and creates a **draft** GitHub release with `latest.json`.
5. Review the draft release on GitHub, then **publish** it. Publishing is the gate: `releases/latest/download/latest.json` only resolves once the release is non-draft and non-prerelease.
6. Verify the update loop from an installed copy of the previous version (see below).

### Linux note

The `tauri-plugin-updater` v2.10+ supports deb, rpm, and AppImage. However, deb/rpm updates require elevated privileges via `pkexec`. Recommend distributing the **AppImage** to end users for the smoothest self-update experience.

## Testing the update loop locally (before the first real release)

Use `src-tauri/tauri.localtest.conf.json` to serve updates from localhost:

```bash
# 1. Build current version (e.g. 0.1.0) with the test config
TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/basic-presence.key) \
  pnpm tauri build --config src-tauri/tauri.localtest.conf.json

# 2. Install the resulting AppImage (Linux) or run the installer
#    e.g. src-tauri/target/release/bundle/appimage/basic-presence_0.1.0_amd64.AppImage

# 3. Bump package.json version to 0.1.1, rebuild the same way

# 4. Create latest.json manually:
#    - version: "0.1.1"
#    - pub_date: current ISO date
#    - notes: "Test update"
#    - platforms["linux-x86_64"]:
#        url: "http://localhost:8000/basic-presence_0.1.1_amd64.AppImage"
#        signature: <contents of the .sig file, not its path>
#
# Example structure:
# {
#   "version": "0.1.1",
#   "pub_date": "2026-06-11T00:00:00Z",
#   "notes": "Test update",
#   "platforms": {
#     "linux-x86_64": {
#       "url": "http://localhost:8000/basic-presence_0.1.1_amd64.AppImage",
#       "signature": "<contents of .AppImage.sig>"
#     }
#   }
# }

# 5. Serve the bundle dir
cd src-tauri/target/release/bundle/appimage
python -m http.server 8000
```

Checklist during local test:
- [ ] Startup check fires automatically and stays silent (no dialog) — the header **version badge** turns warning (ochre) when an update exists
- [ ] Clicking the badge opens the dialog with version and notes (install is a manual action)
- [ ] Clicking the badge when up to date shows the "up to date" dialog
- [ ] Progress bar renders during download
- [ ] After dismissing with "Later", the badge stays warning
- [ ] `relaunch()` boots v0.1.1
- [ ] After update: vault and OS keychain device key still open (update must not touch app data)

## Security note

The auto-updater is the app's **first outbound network access**. The check runs **automatically on every startup** (silent — it only flags the header version badge); **installing** an update is always a **manual** action from that badge. It connects over HTTPS to `github.com` only, from the Rust side (not the WebView). Every artifact is verified with a minisign signature before installation. The WebView CSP is unchanged.

See `documentation/auth.md` for the full threat model.
