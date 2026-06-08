#!/usr/bin/env node
/**
 * Package the web extension Resources/ directory as a .zip suitable for:
 *   - Chrome Web Store (Developer Dashboard upload)
 *   - Microsoft Edge Add-ons (Partner Center upload)
 *   - Any Chromium-based browser's "load unpacked" flow
 *
 * Usage:
 *   node scripts/package-chrome.mjs           → hitchmark-chrome-<version>.zip
 *   node scripts/package-chrome.mjs --edge    → hitchmark-edge-<version>.zip
 */

import { createWriteStream, readdirSync, statSync, readFileSync } from "fs";
import { join, relative, resolve } from "path";
import { createGzip } from "zlib";

// Minimal zip writer — avoids a build dependency on archiver/jszip.
// For a real release pipeline, `npm i -D archiver` and use that instead.
// This script shells out to the system `zip` which is available on macOS/Linux/WSL.

import { execSync } from "child_process";
import { fileURLToPath } from "url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const root = resolve(__dirname, "..");

const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf-8"));
const version = pkg.version;
const isEdge = process.argv.includes("--edge");
const browser = isEdge ? "edge" : "chrome";
const outFile = join(root, `hitchmark-${browser}-${version}.zip`);

const resourcesDir = join(root, "Resources");

// Files/dirs to exclude from the package
const EXCLUDE = [
  "*.map",        // sourcemaps (not needed in production package)
];

const excludeArgs = EXCLUDE.map((p) => `--exclude='${p}'`).join(" ");

const cmd = `cd "${resourcesDir}" && zip -r "${outFile}" . ${excludeArgs}`;
console.log(`Packaging ${browser} extension v${version}…`);
execSync(cmd, { stdio: "inherit" });
console.log(`\n✓ Created: ${outFile}`);
console.log(`\nTo install:`);
if (browser === "chrome") {
  console.log(`  Chrome: chrome://extensions → Enable Developer Mode → Load unpacked → select Resources/`);
  console.log(`  Or upload ${outFile} to https://chrome.google.com/webstore/devconsole`);
} else {
  console.log(`  Edge: edge://extensions → Developer Mode → Load unpacked → select Resources/`);
  console.log(`  Or upload ${outFile} to https://partner.microsoft.com/dashboard/microsoftedge`);
}
