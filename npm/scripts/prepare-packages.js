const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..", "..");
const cargoToml = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
const versionMatch = cargoToml.match(/version\s*=\s*"([^"]+)"/);

if (!versionMatch) {
  throw new Error("Failed to read version from Cargo.toml");
}

const version = versionMatch[1];
const pkgPath = path.join(root, "npm", "main", "package.json");
const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
pkg.version = version;

fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
console.log("Updated npm package version to", version);
