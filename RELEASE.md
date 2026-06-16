# Releasing Veil (first-timer checklist)

Goal: cut a versioned release on GitHub so (a) the in-app updater works and
(b) people can `brew install` it. No Apple Developer account needed — users
clear one "downloaded from the internet" warning on first launch.

Do the steps in order. ☐ = you do it once, ever. The rest repeat per release.

---

## Part A — one-time setup (do this once, ever)

### A1. Generate the updater signing key (free, ~1 min)

```bash
bun run tauri signer generate -w ~/.tauri/veil-updater.key
```

- It prints a **public key** (a long base64 string) and a **password** you
  choose, and writes the private key to `~/.tauri/veil-updater.key`.
- **Keep the password.** You'll paste it into a GitHub secret below.

### A2. Put the public key in the app

- Open [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json).
- Replace `REPLACE_WITH_TAURI_SIGNER_PUBLIC_KEY` (under `plugins.updater.pubkey`)
  with the public key from A1.
- Commit + push:
  ```bash
  git add src-tauri/tauri.conf.json && git commit -m "Add updater public key" && git push
  ```

### A3. Add the two signing secrets to GitHub

Browser → your repo → **Settings → Secrets and variables → Actions → New
repository secret**. Add two:

| Name                                 | Value                                                                                                    |
| ------------------------------------ | -------------------------------------------------------------------------------------------------------- |
| `TAURI_SIGNING_PRIVATE_KEY`          | the **contents** of `~/.tauri/veil-updater.key` (run `cat ~/.tauri/veil-updater.key` and copy all of it) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | the password you chose in A1                                                                             |

### A4. Create the Homebrew tap repo (for `brew install`)

- On GitHub, create a **new public repo** named exactly `homebrew-veil`
  (under your account: `Loa212/homebrew-veil`).
- It can start empty. You'll add the cask file in Part C.

That's all the one-time setup. ✅

---

## Part B — cut a release (repeat each version)

### B1. Bump the version (keep all three in sync)

Edit the `version` in **package.json**, **src-tauri/Cargo.toml**, and
**src-tauri/tauri.conf.json** to the new number (e.g. `0.1.0` → `0.1.1`).
Commit + push:

```bash
git add -A && git commit -m "Release v0.1.1" && git push
```

### B2. Tag it — this triggers the build

```bash
git tag v0.1.1
git push origin v0.1.1
```

Pushing the `v*` tag kicks off `.github/workflows/release.yml`. Watch it at
**your repo → Actions**. It builds the `.dmg`, signs the update artifacts, and
creates a **draft** GitHub Release with the `.dmg`, `latest.json`, and `.sig`.
(~5–10 min.)

### B3. Publish the release

- Repo → **Releases** → you'll see a **Draft** for `v0.1.1`.
- Click **Edit** (pencil), add notes if you like, then **Publish release**.
- Publishing makes `latest.json` reachable at the "latest" URL the app checks —
  so the **in-app updater now works** for everyone on the old version. 🎉

### B4. Sanity-check the updater

Open an older installed copy of Veil → menubar → **Check for updates…** → it
should offer the new version. (On the very first release there's nothing to
update _from_, so just confirm the release has the `.dmg` + `latest.json` + a
`.sig` file attached.)

---

## Part C — Homebrew cask (first release, then automate)

After B3 (release is published):

### C1. Get the dmg's sha256

```bash
# download the released dmg, then:
shasum -a 256 ~/Downloads/Veil_0.1.1_aarch64.dmg
```

### C2. Fill in the cask and push it to the tap

- Copy [Casks/veil.rb](Casks/veil.rb) into your `homebrew-veil` repo at
  `Casks/veil.rb`.
- Set `version` to `0.1.1` and `sha256` to the value from C1.
- Commit + push to `homebrew-veil`.

### C3. Test the install

```bash
brew install --cask Loa212/veil/veil
```

Anyone can now run that command. On first launch macOS shows the
"downloaded from the internet" prompt once — right-click the app → **Open**, or
just click **Open** in the dialog.

---

## What you have after this

- **In-app updates** for everyone (free, no server, no Apple account).
- **`brew install --cask Loa212/veil/veil`** for new users.
- Each future release: bump version → tag → publish draft → bump the cask
  sha256. (Automating the cask bump from CI is a nice later step.)

## Later (optional, costs $99/yr)

Enroll in the Apple Developer Program to sign + notarize, so the Gatekeeper
warning disappears entirely. Steps are in
[DISTRIBUTION.md](DISTRIBUTION.md) — uncomment the `APPLE_*` env in
`release.yml` and add those secrets. Everything else stays the same.
