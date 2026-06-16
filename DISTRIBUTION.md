# Distribution

Veil ships as a signed, notarized `.dmg` via a Homebrew cask. Local unsigned
builds work today; signing/notarization/cask are **deferred** until Apple
Developer credentials are available.

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
