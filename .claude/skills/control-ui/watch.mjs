// Open a VISIBLE browser, logged in as the saved admin session, and leave it
// open so you can watch and click around yourself. Nothing auto-closes it.
//
//   node .claude/skills/control-ui/watch.mjs [path]
//   node .claude/skills/control-ui/watch.mjs /admin
//
// This is Playwright's own bundled Chromium in a clean profile — NOT your real
// Chrome, and not an attached/shared tab. Close the window (or Ctrl+C in the
// terminal) to exit. Optional env: ADMIN_BASE_URL (default http://localhost:3000).
import { existsSync } from "node:fs";
import { chromium } from "playwright";

const BASE = process.env.ADMIN_BASE_URL ?? "http://localhost:3000";
const STATE = new URL("./admin.storageState.json", import.meta.url).pathname;
const path = process.argv.slice(2).find((a) => !a.startsWith("--")) ?? "/admin";

if (!existsSync(STATE)) {
	console.error(`No saved session at ${STATE}. Run login.mjs first.`);
	process.exit(1);
}

const browser = await chromium.launch({ headless: false });
const ctx = await browser.newContext({
	storageState: STATE,
	viewport: { width: 1280, height: 900 },
});
const page = await ctx.newPage();
await page.goto(`${BASE}${path}`, { waitUntil: "networkidle" });
console.log(`✓ Open at ${page.url()} — drive it yourself. Close the window (or Ctrl+C) to exit.`);

// Stay alive until the window/context is closed.
await new Promise((resolve) => {
	browser.on("disconnected", resolve);
	ctx.on("close", resolve);
});
