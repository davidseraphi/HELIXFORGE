import type { NextConfig } from "next";
import path from "node:path";
import fs from "node:fs";
import os from "node:os";

/**
 * Load secrets from ~/Desktop/.keys/helixforge/.env.local (never from the repo).
 */
function loadKeysEnv(): void {
  const home = os.homedir();
  const keysPath = path.join(home, "Desktop", ".keys", "helixforge", ".env.local");
  if (!fs.existsSync(keysPath)) {
    console.log(`[helixforge] no keys file at ${keysPath} — using process env only`);
    return;
  }
  const text = fs.readFileSync(keysPath, "utf8");
  let count = 0;
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const eq = trimmed.indexOf("=");
    if (eq <= 0) continue;
    const key = trimmed.slice(0, eq).trim();
    const value = trimmed.slice(eq + 1).trim();
    if (!(key in process.env)) {
      process.env[key] = value;
      count += 1;
    }
  }
  console.log(`[helixforge] loaded env from ${keysPath} (${count} vars)`);
}

loadKeysEnv();

const nextConfig: NextConfig = {
  reactStrictMode: true,
  transpilePackages: ["@helixforge/ui", "@helixforge/sdk-ts"],
  env: {
    NEXT_PUBLIC_GATEWAY_URL:
      process.env.NEXT_PUBLIC_GATEWAY_URL ?? "http://127.0.0.1:8080",
  },
};

export default nextConfig;
