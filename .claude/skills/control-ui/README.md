# Control UI — Setup Recipe (Playwright)

Companion to [`SKILL.md`](./SKILL.md). `SKILL.md` is the generic harness pattern; this README is the
concrete setup for driving and inspecting a **local web app's UI** with Playwright — screenshots,
accessibility snapshots, reproducing UI bugs, and verifying changes **as a logged-in user**.

> **Portable by design.** This is written so it can be copy-pasted into other projects with a similar
> stack (Next.js + Playwright + Better Auth). The auth section uses **Better Auth** as the concrete
> example. If your project uses a different auth library, only the **"Establishing a logged-in
> session"** section needs adapting — profile isolation, `storageState`, and the probe loop are
> auth-agnostic.

---

## 0. Prerequisites

- **Playwright + a browser binary.** Often already present (even transitively). Check with
  `node -e "require('playwright')"` or just try `import { chromium } from "playwright"`.
  - If missing: `pnpm add -D playwright` then `pnpm exec playwright install chromium`.
  - Per the skill's guardrails, **don't** add Playwright to `package.json` *just* for a one-off probe
    if it's already resolvable from `node_modules`.
- **The dev server is started by the user, not the harness.** Assume the documented dev URL (e.g.
  `http://localhost:3000`) is already serving. The harness connects; it never runs the dev command
  and never probes the port to "check" it's up.
- **Use the test/dev database only** for any user seeding — never production.

---

## 1. A separate browser that never touches your real Chrome

Playwright **does not use your installed Google Chrome or its profile** by default — it launches the
**bundled Chromium** with a clean profile. Two modes:

| Mode | Call | Persistence | Use when |
|------|------|-------------|----------|
| **Ephemeral** | `chromium.launch()` → `browser.newContext()` | Nothing persists | One-off probes that load a saved `storageState` (see §2) |
| **Persistent profile** | `chromium.launchPersistentContext(profileDir, …)` | Cookies/localStorage persist in `profileDir` | A long-lived isolated browser that "stays logged in" |

**Isolation guarantee:** in persistent mode you pass a `profileDir` you own (e.g. a gitignored
`profile/` next to your scripts). Chromium reads/writes only there. Your system Chrome data is never
opened. Never add a flag pointing at the real profile.

```js
import { chromium } from "playwright";

const PROFILE_DIR = new URL("./profile", import.meta.url).pathname; // an isolated dir you own
const ctx = await chromium.launchPersistentContext(PROFILE_DIR, {
  headless: false,                       // headed so you can watch; true for quiet/CI runs
  viewport: { width: 1280, height: 900 },
  // bundled Chromium is implied — NOT your installed Chrome.
});
const page = ctx.pages()[0] ?? (await ctx.newPage());
await page.goto("http://localhost:3000");
// … drive the UI …
await ctx.close();
```

> For readability (e.g. low-vision audiences) bump `viewport` and/or set `deviceScaleFactor`.

---

## 2. Keeping sessions — local, locked, disposable

Two things can persist, with different storage:

1. **The whole profile** (`profile/`) — persistent-context mode stays logged in across runs. Good
   for interactive poking.
2. **Just the auth state** (`*.storageState.json`) — a small JSON of cookies + localStorage for one
   logged-in user. **Recommended** reuse unit: each run starts from a clean ephemeral context and
   *loads* the JSON, so runs are deterministic and don't drift.

**Both hold live session cookies → both are secrets.** Gitignore them, keep them local, and treat a
leak as a credential leak (blast radius = a test user on the test environment). They're disposable:
delete them and re-run login to regenerate.

```js
// Save auth state after a login (once per role):
await ctx.storageState({ path: "customer.storageState.json" });

// Reuse it later in a fast ephemeral context:
const browser = await chromium.launch({ headless: true });
const ctx = await browser.newContext({ storageState: "customer.storageState.json" });
const page = await ctx.newPage();
await page.goto("http://localhost:3000/account"); // already authenticated
```

---

## 3. Establishing a logged-in session — bsas concrete wiring

> This repo is already wired. The generic Better Auth recipe below (§3.1+) is kept
> for reference / portability, but for bsas you only need this:
>
> 1. **Backend:** `emailAndPassword` is enabled in [packages/auth/src/server.ts](../../../packages/auth/src/server.ts).
>    The `/accedi` UIs render **no** password field (magic-link + Google only) — the
>    password backend exists solely so this harness can POST `/api/auth/sign-in/email`.
> 2. **Creds (you, once):** add `E2E_ADMIN_EMAIL` (an existing superadmin, `role='admin'`)
>    and `E2E_ADMIN_PASSWORD` (`openssl rand -base64 24`) to `.env`, then attach a
>    Better-Auth-hashed credential to that user:
>    ```bash
>    cd packages/auth && bun --env-file=../../.env run attach-e2e-password.ts
>    ```
>    (one-shot + disposable — delete it after it runs). It refuses to create a user;
>    it only attaches a password to the existing one. Then `cd ../..`.
> 3. **Dev server (you):** start the admin app (`bun run dev:admin`, port 3000). The
>    harness never starts it.
> 4. **Log in + save state (harness):**
>    ```bash
>    export $(grep -E '^E2E_ADMIN_' .env | xargs) && node .claude/skills/control-ui/login.mjs
>    ```
>    → writes `admin.storageState.json` (gitignored). (Narrow the `grep` to the
>    `E2E_ADMIN_` vars — `grep -v '^#' .env | xargs` chokes on values with spaces
>    or `#`.) The session is DB-backed (30-day expiry), so it survives a dev-server
>    restart; just re-run this if a probe ever bounces to `/accedi`.
> 5. **Probe, watch, or screenshot (harness):**
>    ```bash
>    node .claude/skills/control-ui/probe.mjs /admin          # HEADLESS: screenshot → .tmp/, then exits
>    node .claude/skills/control-ui/probe.mjs /admin --headed # VISIBLE: screenshot + window STAYS OPEN
>    node .claude/skills/control-ui/watch.mjs /admin          # VISIBLE: just opens it for you to drive (no screenshot)
>    ```
>    - `probe.mjs` defaults to **headless** — runs, screenshots to `.tmp/`, closes. (This
>      is why you see no browser window unless you pass `--headed`.)
>    - `--headed` / `watch.mjs` launch Playwright's **own bundled Chromium in a clean
>      isolated profile** — NOT your real Chrome, and NOT an attached/shared tab. Close
>      the window (or Ctrl+C in the terminal) to exit. Run `watch.mjs` in the background
>      (`&` or a background task) to keep clicking while the terminal is free.
>    - To **impersonate a user**, open `/admin` and click **"Entra"** on a shop (it's a
>      server action + redirect — `waitForURL` off `/admin`, don't screenshot the
>      "Apertura…" mid-state). See the multi-shop / impersonation entry in the root
>      `CLAUDE.md`.
>
> ⚠️ The admin dev server points at `.env` = the **PROD** DB. The only write this
> flow makes is your one-shot `attach-e2e-password.ts` (a `credential` account row
> on an existing user) + Better Auth's normal session row on login. Do **not** seed
> users here.

---

### 3.1 Generic Better Auth recipe (reference / other projects)

> **The only auth-library-specific section.** Adapt for non-Better-Auth projects.

### Check what the login UI actually exposes first

A backend can enable email+password while the **UI only ships magic-link / social** (no password
form). When that's the case you can't script "fill password, submit" — drive the auth **API**
instead. (To check: look at the sign-in component for `signIn.magicLink` / `signIn.social` vs. a
password field.)

### Step A — Seed test users *with a password* (test DB only)

A repo's "create admin" script often sets only a role with **no password** (magic-link expected). For
password auth, create users through Better Auth's own API so the hash matches what the server
validates. A one-off seed script should:

1. Import the configured instance: `import { auth } from "@/lib/auth"` (reuses the project's hashing +
   adapter).
2. `await auth.api.signUpEmail({ body: { email, password, name } })` per user — writes the `user` row
   plus an `account` row with `providerId: "credential"` and the correct password hash.
3. For an admin, promote the role afterward (project's `create:admin` script, or set `role: "admin"`
   on the `user` row).

Run against the **test DB** (load the test env; allow remote writes if the DB is remote), e.g.:

```bash
export $(grep -v '^#' .env | xargs) && ALLOW_REMOTE_DB=1 pnpm tsx seed-test-users.mjs
```

Use stable, obviously-fake credentials, e.g. `customer@e2e.local` / `admin@e2e.local` with a fixed
test password.

### Step B — Log in via the API, save `storageState` (once per role)

Hit Better Auth's email sign-in endpoint with Playwright's **context request object** so `Set-Cookie`
lands in the context, then save state:

```js
import { chromium } from "playwright";

const BASE = "http://localhost:3000";
const ROLE = process.argv[2] ?? "customer";
const CREDS = {
  customer: { email: "customer@e2e.local", password: "e2e-test-password-123" },
  admin:    { email: "admin@e2e.local",    password: "e2e-test-password-123" },
}[ROLE];

const browser = await chromium.launch();
const ctx = await browser.newContext();
const res = await ctx.request.post(`${BASE}/api/auth/sign-in/email`, {
  data: { email: CREDS.email, password: CREDS.password },
});
if (!res.ok()) throw new Error(`sign-in failed: ${res.status()} ${await res.text()}`);
await ctx.storageState({ path: `${ROLE}.storageState.json` });
await browser.close();
```

> **Other auth libraries:** only *how you obtain a valid session cookie* changes. NextAuth/Auth.js,
> Lucia, Clerk, a custom JWT cookie — each has its own sign-in endpoint or token-mint step; the
> save-and-reuse `storageState` flow is identical. If there's no password path at all, fall back to
> **seeding a session row directly** in the DB + `ctx.addCookies([...])` — faster, but it bypasses the
> library's validation and can break silently, so prefer the API path when one exists.

### Step C — Reuse it in probes

```js
import { chromium } from "playwright";
const ROLE = process.argv[2] ?? "customer";
const browser = await chromium.launch({ headless: false });
const ctx = await browser.newContext({ storageState: `${ROLE}.storageState.json` });
const page = await ctx.newPage();
await page.goto("http://localhost:3000/account"); // or an admin route for the admin role
await page.screenshot({ path: ".tmp/probe.png", fullPage: true }); // screenshots → .tmp/ only
await browser.close();
```

---

## 4. Interaction loop (from `SKILL.md`)

1. Screenshot/snapshot **before** acting.
2. Pick a target (prefer roles / labels / stable `data-*`, not coordinates).
3. Do **one** action (click / type / scroll / navigate).
4. Screenshot/snapshot **after**.
5. Verify the change; save before/after artifacts when you need proof.

---

## 5. Tracking choices

- **Commit (tracked dev tooling):** the scripts (`login.mjs`, `probe.mjs`, `watch.mjs`) + this
  README (reusable across machines/sessions).
- **Gitignore (live cookies / disposable):** `profile/`, `*.storageState.json`.
- **Screenshots / heap snapshots → `.tmp/` only**, never committed; clean periodically.

---

## 6. Guardrails

- Never point the browser at your real Chrome profile — use an owned `profile/` (persistent) or a
  fresh ephemeral context. Both use bundled Chromium.
- Seed/auth against the **test/dev DB only**, never production.
- The dev server is the user's — don't start it or probe its port; assume it's up.
- Treat `*.storageState.json` and `profile/` as secrets; gitignore them.
- Reset auth by deleting `profile/` + `*.storageState.json`; clear `.tmp/` periodically.
- Don't hard-code another project's selectors/ports — rediscover the current app's markers.
