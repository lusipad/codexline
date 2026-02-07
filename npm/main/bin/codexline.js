#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");
const childProcess = require("node:child_process");

const exe = process.platform === "win32" ? "codexline.exe" : "codexline";
const binaryPath = path.join(__dirname, "..", "vendor", exe);

if (!fs.existsSync(binaryPath)) {
  console.error("codexline binary is missing.");
  console.error("Try reinstalling package: npm install -g codexline");
  process.exit(1);
}

const child = childProcess.spawn(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code == null ? 1 : code);
});
