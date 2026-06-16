// Open the admin app as the logged-in superadmin (reusing admin.storageState.json
// from login.mjs) and screenshot a route. Default route is /admin — the platform
// superadmin dashboard, the entry point for impersonating users.
//
//   node .claude/skills/control-ui/probe.mjs [path] [--headed]
//   node .claude/skills/control-ui/probe.mjs /admin
//   node .claude/skills/control-ui/probe.mjs / --headed
//
// Optional env: ADMIN_BASE_URL (default http://localhost:3000).
// Screenshots land in .tmp/ (gitignored) — never committed.
import { existsSync } from "node:fs";
import { chromium } from "playwright";

const BASE = process.env.ADMIN_BASE_URL ?? "http://localhost:3000";
const STATE = new URL("./admin.storageState.json", import.meta.url).pathname;
const args = process.argv.slice(2);
const headed = args.includes("--headed");
const path = args.find((a) => !a.startsWith("--")) ?? "/admin";

if (!existsSync(STATE)) {
	console.error(`No saved session at ${STATE}. Run login.mjs first.`);
	process.exit(1);
}

const repoRoot = new URL("../../../", import.meta.url).pathname;
const safe = path.replace(/[^a-z0-9]+/gi, "_").replace(/^_+|_+$/g, "") || "root";
const out = `${repoRoot}.tmp/probe-${safe}.png`;

const browser = await chromium.launch({ headless: !headed });
const ctx = await browser.newContext({
	storageState: STATE,
	viewport: { width: 1280, height: 900 },
});
const page = await ctx.newPage();
await page.goto(`${BASE}${path}`, { waitUntil: "networkidle" });
await page.screenshot({ path: out, fullPage: true });
console.log(`✓ ${page.url()} → ${out}`);

if (headed) {
	// Keep the visible window open so you can watch / click around. Closes when
	// you close the window or press Ctrl+C in the terminal.
	console.log("Window open — close it (or Ctrl+C) to exit.");
	await page.waitForEvent("close", { timeout: 0 }).catch(() => {});
}
await browser.close();
