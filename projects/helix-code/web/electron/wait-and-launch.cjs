/**
 * Wait for HelixCode web UI, then launch Electron.
 * Usage: node electron/wait-and-launch.cjs
 * Env: HELIX_CODE_WEB_URL (default http://127.0.0.1:3102)
 */
const { spawn } = require("child_process");
const path = require("path");
const http = require("http");
const https = require("https");

const WEB_URL =
  process.env.HELIX_CODE_WEB_URL ||
  process.env.ELECTRON_START_URL ||
  "http://127.0.0.1:3102";
const MAX_MS = Number(process.env.HELIX_ELECTRON_WAIT_MS || 120000);
const INTERVAL_MS = 800;

function probe(url) {
  return new Promise((resolve) => {
    try {
      const lib = url.startsWith("https") ? https : http;
      const req = lib.get(url, { timeout: 2000 }, (res) => {
        res.resume();
        resolve(res.statusCode && res.statusCode < 500);
      });
      req.on("error", () => resolve(false));
      req.on("timeout", () => {
        req.destroy();
        resolve(false);
      });
    } catch {
      resolve(false);
    }
  });
}

async function waitForWeb() {
  const start = Date.now();
  process.stdout.write(`Waiting for web UI at ${WEB_URL} …\n`);
  while (Date.now() - start < MAX_MS) {
    if (await probe(WEB_URL)) {
      process.stdout.write(`Web UI ready (${Math.round((Date.now() - start) / 1000)}s)\n`);
      return true;
    }
    await new Promise((r) => setTimeout(r, INTERVAL_MS));
  }
  process.stderr.write(
    `Timed out waiting for ${WEB_URL}. Start: pnpm --filter @helixforge/helix-code-web dev\n`,
  );
  return false;
}

async function main() {
  const ok = await waitForWeb();
  if (!ok) process.exit(1);

  // Resolve electron binary via require('electron') path
  let electronPath;
  try {
    electronPath = require("electron");
  } catch (e) {
    process.stderr.write(`electron package missing: ${e.message}\n`);
    process.exit(1);
  }

  const appRoot = path.join(__dirname, "..");
  const child = spawn(electronPath, [appRoot], {
    stdio: "inherit",
    env: process.env,
    cwd: appRoot,
  });
  child.on("exit", (code) => process.exit(code ?? 0));
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
