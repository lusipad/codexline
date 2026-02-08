#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");
const childProcess = require("node:child_process");

const exe = process.platform === "win32" ? "codexline.exe" : "codexline";
const binaryPath = path.join(__dirname, "..", "vendor", exe);
const bridgePath = path.join(__dirname, "codexline-bridge.js");
const args = process.argv.slice(2);

if (args[0] === "bridge") {
  if (!fs.existsSync(bridgePath)) {
    console.error("codexline-bridge helper is missing.");
    console.error("Try reinstalling package: npm install -g codexline");
    process.exit(1);
  }

  const child = childProcess.spawn(
    process.execPath,
    [bridgePath, ...args.slice(1)],
    { stdio: "inherit" },
  );
  attachExit(child);
  return;
}

if (!fs.existsSync(binaryPath)) {
  console.error("codexline binary is missing.");
  console.error("Try reinstalling package: npm install -g codexline");
  process.exit(1);
}

const runArgs = args.length === 0 ? ["--plain"] : args;
const child = childProcess.spawn(binaryPath, runArgs, {
  stdio: "inherit",
});

attachExit(child);

function attachExit(childProcessHandle) {
  childProcessHandle.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }
    process.exit(code == null ? 1 : code);
  });
}
