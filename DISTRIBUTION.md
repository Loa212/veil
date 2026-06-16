# Distribution

Veil ships as a signed, notarized `.dmg` via a Homebrew cask. Local unsigned
builds work today; signing/notarization/cask are **deferred** until Apple
Developer credentials are available.

## Auto-update (in-app "Update available" — like jean)

Veil checks for updates on launch (release builds) and via the menubar
**Check for updates…** item. If a newer version exists it offers to download +
install it and restart.

**You do NOT need a server.** GitHub Releases is the update host: each release
uploads the `.dmg`, its `.sig` signature, and a `latest.json` manifest as static
files. The app fetches `latest.json` from
`https://github.com/Loa212/veil/releases/latest/download/latest.json` (set in
[src-tauri/tauri.conf.json](src-tauri/tauri.conf.json) → `plugins.updater`).

The updater needs a **free** minisign signing key (separate from Apple — this is
just so a hijacked release can't push malware; the app verifies the signature).

### One-time setup

1. Generate the keypair (free, local, no account):
   ```bash
   bun run tauri signer generate -w ~/.tauri/veil-updater.key
   ```
   It prints a **public key** and writes a password-protected **private key**.
2. Put the **public key** in
   [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json) →
   `plugins.updater.pubkey` (replace `REPLACE_WITH_TAURI_SIGNER_PUBLIC_KEY`).
3. Add the **private key** + its password as GitHub repo secrets:
   `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
4. The release workflow signs the update artifacts and uploads `latest.json`
   automatically (`createUpdaterArtifacts: true` is already set, and
   `tauri-action` emits + uploads them when those secrets are present).

That's it — no backend, no hosting cost. Tag a release → users get the in-app
update prompt.

Note: auto-update is independent of Apple signing. It works even on an unsigned
build; the minisign key is its own (free) thing.

## Build locally (unsigned)

```bash
bun run tauri build              # .app + .dmg (unsigned), full release profile
bun run tauri:build:fast         # .dmg only, faster non-distribution profile
```

Output: `src-tauri/target/release/bundle/{macos,dmg}/`.

An unsigned build runs locally but Gatekeeper warns on other machines — that's
why release distribution needs signing + notarization.

## Enable signing + notarization (TODO — needs Apple credentials)

Provide these, then wire them up:

1. **Apple Developer Team ID**, a **Developer ID Application** certificate, and
   a **notarytool** API key (or Apple ID + app-specific password).
2. In [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json) → `bundle.macOS`:
   - set `signingIdentity` to `"Developer ID Application: NAME (TEAMID)"`
   - set `providerShortName` to the Team ID
   - `entitlements` already points at
     [src-tauri/entitlements.plist](src-tauri/entitlements.plist) (hardened
     runtime).
3. In [.github/workflows/release.yml](.github/workflows/release.yml): add repo
   secrets (`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`,
   `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`) and
   uncomment the env block in the build step. `tauri-action` then signs +
   notarizes + staples automatically.

Note: Veil uses the **private** `SACLockScreenImmediate` API, so it cannot ship
on the Mac App Store — Developer ID + Homebrew only. That's intentional.

## Homebrew cask (TODO — after the first signed release)

1. Publish the signed `.dmg` to GitHub Releases.
2. Create a tap repo `homebrew-veil` with `Casks/veil.rb` pointing at the
   release artifact + its sha256.
3. Users install via `brew install loa212/veil/veil`.
4. Automate the cask version/sha bump from `release.yml` on each tagged release.

## CLI (post-MVP)

MVP CLI is just "launch the app". A `veil` shim that talks to a running instance
(`veil lock` / `veil status` over a local socket) is deferred.
