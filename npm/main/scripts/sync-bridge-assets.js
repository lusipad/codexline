const fs = require("node:fs");
const path = require("node:path");

const packageRoot = path.resolve(__dirname, "..");
const repoRoot = path.resolve(packageRoot, "..", "..");
const sourceDir = path.join(repoRoot, "scripts", "bridge");
const targetDir = path.join(packageRoot, "bridge");

const files = [
  "README.md",
  "install-bridge.ps1",
  "uninstall-bridge.ps1",
  "install-bridge.sh",
  "uninstall-bridge.sh",
];

for (const file of files) {
  const sourcePath = path.join(sourceDir, file);
  if (!fs.existsSync(sourcePath)) {
    throw new Error("Bridge asset missing: " + sourcePath);
  }
}

fs.rmSync(targetDir, { recursive: true, force: true });
fs.mkdirSync(targetDir, { recursive: true });

for (const file of files) {
  const sourcePath = path.join(sourceDir, file);
  const targetPath = path.join(targetDir, file);
  fs.copyFileSync(sourcePath, targetPath);
}

console.log("Synced bridge assets to", targetDir);
