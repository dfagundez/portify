#!/usr/bin/env node
/**
 * Verifies that the package-manager manifests point at files that exist and
 * whose contents match the checksums they declare.
 *
 *   node scripts/check-packaging.mjs
 *
 * The failure this catches is the one that actually happens: a release goes
 * out, the manifests keep the previous version's URL and hash, and the first
 * person to run `scoop install portify` gets last month's binary — or a hash
 * mismatch and no install at all. Nobody notices until a stranger reports it.
 *
 * Deliberately does NOT compare against the workspace version in Cargo.toml.
 * These manifests describe the released artefacts, so between a version bump
 * and the release that follows it they are *supposed* to lag.
 *
 * Schema validation is left to microsoft/winget-pkgs, whose CI is the
 * authority on its own format and runs on every submission.
 */

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");

const failures = [];
const fail = (message) => failures.push(message);

/** Reads `Key: value` out of a flat YAML file. Enough for winget manifests. */
function yamlField(text, key) {
  const match = text.match(new RegExp(`^\\s*${key}:\\s*(.+?)\\s*$`, "m"));
  return match ? match[1].replace(/^["']|["']$/g, "") : undefined;
}

async function checkAsset(label, url, expected) {
  const response = await fetch(url, { redirect: "follow" });
  if (!response.ok) {
    fail(`${label}: ${url} returned HTTP ${response.status}`);
    return;
  }
  const bytes = Buffer.from(await response.arrayBuffer());
  const actual = createHash("sha256").update(bytes).digest("hex");
  if (actual.toLowerCase() !== expected.toLowerCase()) {
    fail(`${label}: checksum mismatch\n    declared ${expected}\n    actual   ${actual}`);
    return;
  }
  console.log(`  ok  ${label}  (${(bytes.length / 1024).toFixed(0)} KB, sha256 matches)`);
}

/* ---------- Scoop ---------- */

console.log("scoop bucket/portify.json");
const scoop = JSON.parse(readFileSync(join(ROOT, "bucket/portify.json"), "utf8"));
const scoop64 = scoop.architecture?.["64bit"];

if (!scoop64?.url || !scoop64?.hash) {
  fail("scoop: manifest has no 64bit url/hash");
} else {
  // The URL carries the version, so a bumped `version` with a stale URL — the
  // exact shape of a half-finished release bump — is caught here.
  if (!scoop64.url.includes(`/v${scoop.version}/`)) {
    fail(`scoop: version is ${scoop.version} but the URL is not from tag v${scoop.version}\n    ${scoop64.url}`);
  }
  await checkAsset("scoop 64bit", scoop64.url, scoop64.hash);
}

/* ---------- winget ---------- */

console.log("winget packaging/winget/");
const wingetFiles = {
  version: "DiegoFagundez.Portify.yaml",
  installer: "DiegoFagundez.Portify.installer.yaml",
  locale: "DiegoFagundez.Portify.locale.en-US.yaml",
};
const winget = Object.fromEntries(
  Object.entries(wingetFiles).map(([kind, name]) => [
    kind,
    readFileSync(join(ROOT, "packaging/winget", name), "utf8"),
  ]),
);

// winget rejects a submission whose three files disagree, and the error it
// gives is not obvious. Cheaper to catch here.
for (const field of ["PackageIdentifier", "PackageVersion"]) {
  const values = new Set(Object.values(winget).map((text) => yamlField(text, field)));
  if (values.size !== 1) {
    fail(`winget: ${field} differs across the manifests: ${[...values].join(", ")}`);
  }
}

const wingetVersion = yamlField(winget.version, "PackageVersion");
const installerUrl = yamlField(winget.installer, "InstallerUrl");
const installerHash = yamlField(winget.installer, "InstallerSha256");

if (!installerUrl || !installerHash) {
  fail("winget: installer manifest has no InstallerUrl/InstallerSha256");
} else {
  if (!installerUrl.includes(`/v${wingetVersion}/`)) {
    fail(`winget: PackageVersion is ${wingetVersion} but the installer URL is not from tag v${wingetVersion}\n    ${installerUrl}`);
  }
  await checkAsset("winget x64 installer", installerUrl, installerHash);
}

/* ---------- verdict ---------- */

if (failures.length > 0) {
  console.error(`\n${failures.length} problem(s):\n`);
  for (const message of failures) console.error(`  - ${message}`);
  process.exit(1);
}
console.log("\npackaging manifests are consistent with the published artefacts");
