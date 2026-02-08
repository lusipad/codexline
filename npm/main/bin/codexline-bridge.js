#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");
const childProcess = require("node:child_process");

const command = process.argv[2] || "help";
let extraArgs = process.argv.slice(3);

if (command === "help" || command === "--help" || command === "-h") {
  printHelp(0);
}

if (command !== "install" && command !== "uninstall") {
  console.error("Unsupported command: " + command);
  printHelp(1);
}

if (process.platform === "win32") {
  extraArgs = normalizeWindowsArgs(extraArgs);
}

const scriptPath = resolveScriptPath(command);
if (!fs.existsSync(scriptPath)) {
  console.error("Bridge script not found: " + scriptPath);
  console.error("Try reinstalling package: npm install -g codexline");
  process.exit(1);
}

if (process.platform === "win32") {
  const powershell = findPowerShell();
  if (!powershell) {
    console.error("PowerShell is required but was not found (pwsh/powershell).");
    process.exit(1);
  }

  run(
    powershell,
    ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", scriptPath].concat(extraArgs),
  );
} else {
  run("bash", [scriptPath].concat(extraArgs));
}

function resolveScriptPath(cmd) {
  const ext = process.platform === "win32" ? "ps1" : "sh";
  return path.join(__dirname, "..", "bridge", cmd + "-bridge." + ext);
}

function normalizeWindowsArgs(args) {
  const map = {
    "--refresh-ms": "-RefreshMs",
    "--task-name": "-TaskName",
    "--skip-start": "-SkipStart",
    "--remove-loop-script": "-RemoveLoopScript",
  };

  const normalized = [];
  for (const arg of args) {
    if (arg === "--") {
      continue;
    }
    if (arg === "--help" || arg === "-h") {
      normalized.push("-?");
      continue;
    }
    normalized.push(map[arg] || arg);
  }
  return normalized;
}

function findPowerShell() {
  const candidates = ["pwsh", "pwsh.exe", "powershell", "powershell.exe"];
  for (const candidate of candidates) {
    const result = childProcess.spawnSync(candidate, ["-NoProfile", "-Command", "exit"], {
      stdio: "ignore",
    });

    if (!result.error && typeof result.status === "number") {
      return candidate;
    }
  }

  return null;
}

function run(executable, args) {
  const child = childProcess.spawn(executable, args, { stdio: "inherit" });
  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }
    process.exit(code == null ? 1 : code);
  });
}

function printHelp(exitCode) {
  console.log("codexline-bridge <install|uninstall> [script options]");
  console.log("");
  console.log("Examples:");
  console.log("  codexline-bridge install");
  console.log("  codexline-bridge uninstall");
  console.log("  codexline-bridge install --refresh-ms 500");
  console.log("  codexline-bridge install --help");
  process.exit(exitCode);
}
