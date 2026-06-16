// Log into the admin app via Better Auth's email+password API and save the
// session as admin.storageState.json (gitignored). Run once per session; the
// saved state is reused by probe.mjs and any other harness script.
//
//   node .claude/skills/control-ui/login.mjs
//
// Reads creds from the environment (load .env first), so nothing is hardcoded:
//   E2E_ADMIN_EMAIL / E2E_ADMIN_PASSWORD   (see .env.example)
// Optional: ADMIN_BASE_URL (default http://localhost:3000).
//
// Prereq: the admin dev server is already running (the harness never starts it),
// and the E2E admin's password was attached via tools/attach-e2e-password.ts.
import { chromium } from "playwright";

const BASE = process.env.ADMIN_BASE_URL ?? "http://localhost:3000";
const email = process.env.E2E_ADMIN_EMAIL;
const password = process.env.E2E_ADMIN_PASSWORD;
const STATE = new URL("./admin.storageState.json", import.meta.url).pathname;

if (!email || !password) {
	console.error("Set E2E_ADMIN_EMAIL and E2E_ADMIN_PASSWORD (load .env first).");
	process.exit(1);
}

const browser = await chromium.launch();
const ctx = await browser.newContext();
const res = await ctx.request.post(`${BASE}/api/auth/sign-in/email`, {
	data: { email, password },
});
if (!res.ok()) {
	console.error(`sign-in failed: ${res.status()} ${await res.text()}`);
	await browser.close();
	process.exit(1);
}
await ctx.storageState({ path: STATE });
await browser.close();
console.log(`✓ Saved admin session → ${STATE}`);
