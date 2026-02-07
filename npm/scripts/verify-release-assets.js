const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..", "..");
const releaseWorkflowPath = path.join(repoRoot, ".github", "workflows", "release.yml");
const postinstallPath = path.join(repoRoot, "npm", "main", "scripts", "postinstall.js");

const releaseContent = readFile(releaseWorkflowPath);
const postinstallContent = readFile(postinstallPath);

const releaseAssets = collectReleaseAssets(releaseContent);
const installerAssets = collectInstallerAssets(postinstallContent);
const hasChecksumAsset = /codexline-checksums\.txt/.test(releaseContent);

const missingInRelease = difference(installerAssets, releaseAssets);
const extraInRelease = difference(releaseAssets, installerAssets);

let failed = false;

if (installerAssets.length === 0) {
  console.error("verify-release-assets: no installer assets detected in postinstall.js");
  failed = true;
}

if (releaseAssets.length === 0) {
  console.error("verify-release-assets: no release assets detected in release.yml");
  failed = true;
}

if (missingInRelease.length > 0) {
  console.error(
    "verify-release-assets: assets required by installer but missing in release workflow:",
  );
  for (const asset of missingInRelease) {
    console.error(`  - ${asset}`);
  }
  failed = true;
}

if (extraInRelease.length > 0) {
  console.error(
    "verify-release-assets: release workflow has assets not referenced by installer:",
  );
  for (const asset of extraInRelease) {
    console.error(`  - ${asset}`);
  }
  failed = true;
}

if (!hasChecksumAsset) {
  console.error(
    "verify-release-assets: release.yml does not generate codexline-checksums.txt",
  );
  failed = true;
}

if (failed) {
  process.exit(1);
}

console.log("verify-release-assets: OK");
console.log(`  installer assets: ${installerAssets.join(", ")}`);
console.log(`  release assets:   ${releaseAssets.join(", ")}`);
console.log("  checksum file:    codexline-checksums.txt");

function readFile(filePath) {
  try {
    return fs.readFileSync(filePath, "utf8");
  } catch (error) {
    console.error(`verify-release-assets: failed to read ${filePath}: ${error.message}`);
    process.exit(1);
  }
}

function collectReleaseAssets(content) {
  const assets = new Set();
  const pattern = /^\s*asset_name:\s*([^\s#]+)\s*$/gm;

  for (const match of content.matchAll(pattern)) {
    const value = stripQuotes(match[1].trim());
    if (value.startsWith("codexline-")) {
      assets.add(value);
    }
  }

  return [...assets].sort();
}

function collectInstallerAssets(content) {
  const assets = new Set();
  const pattern = /assetName:\s*"([^"]+)"/g;

  for (const match of content.matchAll(pattern)) {
    const value = match[1].trim();
    if (value.startsWith("codexline-")) {
      assets.add(value);
    }
  }

  return [...assets].sort();
}

function stripQuotes(value) {
  if (
    (value.startsWith('"') && value.endsWith('"')) ||
    (value.startsWith("'") && value.endsWith("'"))
  ) {
    return value.slice(1, -1);
  }
  return value;
}

function difference(left, right) {
  const rightSet = new Set(right);
  return left.filter((value) => !rightSet.has(value));
}
