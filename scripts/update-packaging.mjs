#!/usr/bin/env node
/**
 * Rewrites the package-manager manifests to point at a published release.
 *
 *   node scripts/update-packaging.mjs v0.2.0
 *
 * Run automatically by .github/workflows/packaging.yml when a release is
 * published. Doing it by hand means computing two SHA-256 sums and editing
 * four files in lockstep, which is exactly the kind of chore that gets done
 * correctly for two releases and then silently half-done forever.
 *
 * It reads the *published* release, not the tag, because Portify's releases are
 * created as drafts: until someone publishes one, its assets are not
 * downloadable and there is nothing to checksum.
 *
 * Re-running it for the current version is a no-op, which is what makes it
 * safe to test.
 */

import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const REPO = "dfagundez/portify";

const tag = process.argv[2];
if (!tag || !/^v\d+\.\d+\.\d+/.test(tag)) {
  console.error("usage: node scripts/update-packaging.mjs v<major>.<minor>.<patch>");
  process.exit(2);
}
const version = tag.slice(1);

/* ---------- read the release ---------- */

const headers = { accept: "application/vnd.github+json" };
if (process.env.GITHUB_TOKEN) headers.authorization = `Bearer ${process.env.GITHUB_TOKEN}`;

const response = await fetch(`https://api.github.com/repos/${REPO}/releases/tags/${tag}`, { headers });
if (!response.ok) {
  console.error(`could not read release ${tag}: HTTP ${response.status}`);
  process.exit(1);
}
const release = await response.json();
if (release.draft) {
  console.error(`release ${tag} is still a draft; its assets are not downloadable yet`);
  process.exit(1);
}

const assets = new Map(release.assets.map((asset) => [asset.name, asset.browser_download_url]));

async function sha256(name) {
  const url = assets.get(name);
  if (!url) {
    console.error(`release ${tag} has no asset named ${name}`);
    console.error(`  it has: ${[...assets.keys()].join(", ")}`);
    process.exit(1);
  }
  const bytes = Buffer.from(await (await fetch(url, { redirect: "follow" })).arrayBuffer());
  return { url, hash: createHash("sha256").update(bytes).digest("hex"), size: bytes.length };
}

const cliZip = `portify-cli-windows-x86_64.zip`;
const setupExe = `Portify_${version}_x64-setup.exe`;

const cli = await sha256(cliZip);
const setup = await sha256(setupExe);
const releaseDate = (release.published_at ?? release.created_at).slice(0, 10);

console.log(`${tag}  published ${releaseDate}`);
console.log(`  ${cliZip}   ${(cli.size / 1024).toFixed(0)} KB  ${cli.hash}`);
console.log(`  ${setupExe}  ${(setup.size / 1024).toFixed(0)} KB  ${setup.hash}`);

/* ---------- rewrite ---------- */

const changed = [];

function write(relative, next) {
  const path = join(ROOT, relative);
  if (readFileSync(path, "utf8") === next) return;
  writeFileSync(path, next);
  changed.push(relative);
}

// Scoop: lowercase hex, which is what `scoop install` compares against.
const scoopPath = "bucket/portify.json";
const scoop = JSON.parse(readFileSync(join(ROOT, scoopPath), "utf8"));
scoop.version = version;
scoop.architecture["64bit"].url = cli.url;
scoop.architecture["64bit"].hash = cli.hash;
write(scoopPath, `${JSON.stringify(scoop, null, 4)}\n`);

// winget: uppercase hex, per its own manifests. Flat `Key: value` files, so a
// line-anchored replace is enough and leaves every comment in place.
const upper = setup.hash.toUpperCase();
const substitutions = {
  "packaging/winget/DiegoFagundez.Portify.yaml": {
    PackageVersion: version,
  },
  "packaging/winget/DiegoFagundez.Portify.installer.yaml": {
    PackageVersion: version,
    ReleaseDate: `"${releaseDate}"`,
    InstallerUrl: setup.url,
    InstallerSha256: upper,
  },
  "packaging/winget/DiegoFagundez.Portify.locale.en-US.yaml": {
    PackageVersion: version,
    ReleaseNotesUrl: `https://github.com/${REPO}/releases/tag/${tag}`,
  },
};

for (const [relative, fields] of Object.entries(substitutions)) {
  let text = readFileSync(join(ROOT, relative), "utf8");
  for (const [key, value] of Object.entries(fields)) {
    const pattern = new RegExp(`^(\\s*)${key}:.*$`, "m");
    if (!pattern.test(text)) {
      console.error(`${relative}: expected a "${key}:" line and found none`);
      process.exit(1);
    }
    text = text.replace(pattern, `$1${key}: ${value}`);
  }
  write(relative, text);
}

/* ---------- report ---------- */

if (changed.length === 0) {
  console.log("\nmanifests already describe this release; nothing to do");
} else {
  console.log(`\nupdated:\n${changed.map((f) => `  ${f}`).join("\n")}`);
}
