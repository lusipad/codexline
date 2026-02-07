const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");
const https = require("node:https");
const { pipeline } = require("node:stream/promises");

const pkg = require("../package.json");
const version = process.env.CODEXLINE_VERSION || pkg.version;
const baseUrl =
  process.env.CODEXLINE_BASE_URL ||
  "https://github.com/lusipad/codexline/releases/download/v";
const downloadRetries = readPositiveInt(process.env.CODEXLINE_DOWNLOAD_RETRIES, 3);
const timeoutMs = readPositiveInt(process.env.CODEXLINE_DOWNLOAD_TIMEOUT_MS, 20000);
const verifyChecksum = process.env.CODEXLINE_VERIFY_CHECKSUM !== "0";
const requireChecksum = process.env.CODEXLINE_REQUIRE_CHECKSUM === "1";

if (process.env.CODEXLINE_SKIP_DOWNLOAD === "1") {
  process.exit(0);
}

const target = resolveTarget(process.platform, process.arch);
if (!target) {
  console.warn("codexline: unsupported platform", process.platform, process.arch);
  process.exit(0);
}

const releaseBase = trimTrailingSlash(baseUrl) + version + "/";
const binaryUrl = releaseBase + target.assetName;
const checksumUrl = releaseBase + "codexline-checksums.txt";

const vendorDir = path.join(__dirname, "..", "vendor");
const outPath = path.join(vendorDir, target.outputName);

fs.mkdirSync(vendorDir, { recursive: true });

install().catch((error) => {
  removeFileIfExists(outPath);
  removeFileIfExists(outPath + ".tmp");
  console.error("codexline: failed to install binary", error.message);
  process.exit(1);
});

async function install() {
  await withRetry("binary download", downloadRetries, () =>
    downloadBinary(binaryUrl, outPath, timeoutMs),
  );

  if (verifyChecksum) {
    await verifyBinaryChecksum({
      checksumUrl,
      assetName: target.assetName,
      outputPath: outPath,
      retries: downloadRetries,
      timeoutMs,
      requireChecksum,
    });
  }

  if (process.platform !== "win32") {
    fs.chmodSync(outPath, 0o755);
  }

  console.log("codexline: installed", target.outputName);
}

function resolveTarget(platform, arch) {
  if (platform === "win32" && arch === "x64") {
    return { assetName: "codexline-windows-x64.exe", outputName: "codexline.exe" };
  }
  if (platform === "linux" && arch === "x64") {
    return { assetName: "codexline-linux-x64", outputName: "codexline" };
  }
  if (platform === "linux" && arch === "arm64") {
    return { assetName: "codexline-linux-arm64", outputName: "codexline" };
  }
  if (platform === "darwin" && arch === "x64") {
    return { assetName: "codexline-macos-x64", outputName: "codexline" };
  }
  if (platform === "darwin" && arch === "arm64") {
    return { assetName: "codexline-macos-arm64", outputName: "codexline" };
  }
  return null;
}

async function verifyBinaryChecksum({
  checksumUrl,
  assetName,
  outputPath,
  retries,
  timeoutMs,
  requireChecksum,
}) {
  let checksums;
  try {
    checksums = await withRetry("checksum download", retries, () =>
      downloadText(checksumUrl, timeoutMs),
    );
  } catch (error) {
    if (requireChecksum) {
      throw new Error(`checksum file unavailable: ${error.message}`);
    }
    console.warn(`codexline: checksum skipped (${error.message})`);
    return;
  }

  const expected = parseChecksum(checksums, assetName);
  if (!expected) {
    if (requireChecksum) {
      throw new Error(`checksum entry missing for ${assetName}`);
    }
    console.warn(`codexline: checksum entry missing for ${assetName}, skipped`);
    return;
  }

  const actual = await sha256File(outputPath);
  if (actual.toLowerCase() !== expected.toLowerCase()) {
    throw new Error(
      `checksum mismatch for ${assetName}: expected ${expected}, got ${actual}`,
    );
  }
}

function parseChecksum(content, assetName) {
  const lines = content.split(/\r?\n/);
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) {
      continue;
    }

    const match = trimmed.match(/^([a-fA-F0-9]{64})\s+\*?(.+)$/);
    if (!match) {
      continue;
    }

    const fileName = match[2].trim();
    if (fileName === assetName || path.basename(fileName) === assetName) {
      return match[1];
    }
  }
  return null;
}

async function withRetry(name, attempts, task) {
  let lastError;
  for (let i = 1; i <= attempts; i += 1) {
    try {
      return await task();
    } catch (error) {
      lastError = error;
      if (i < attempts) {
        console.warn(
          `codexline: ${name} failed (${i}/${attempts}), retrying: ${error.message}`,
        );
        await sleep(400 * i);
      }
    }
  }

  throw new Error(
    `${name} failed after ${attempts} attempts: ${
      lastError ? lastError.message : "unknown error"
    }`,
  );
}

async function downloadBinary(url, dest, timeoutMs) {
  const tempPath = dest + ".tmp";
  removeFileIfExists(tempPath);

  try {
    const response = await getResponse(url, timeoutMs, 5);
    await pipeline(response, fs.createWriteStream(tempPath));
    removeFileIfExists(dest);
    fs.renameSync(tempPath, dest);
  } catch (error) {
    removeFileIfExists(tempPath);
    throw error;
  }
}

async function downloadText(url, timeoutMs) {
  const response = await getResponse(url, timeoutMs, 5);
  const chunks = [];
  for await (const chunk of response) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }
  return Buffer.concat(chunks).toString("utf8");
}

function getResponse(url, timeoutMs, redirectsLeft) {
  return new Promise((resolve, reject) => {
    const req = https.get(url, (res) => {
      const status = res.statusCode || 0;

      if (status >= 300 && status < 400 && res.headers.location) {
        if (redirectsLeft <= 0) {
          res.resume();
          reject(new Error(`too many redirects while fetching ${url}`));
          return;
        }

        const nextUrl = new URL(res.headers.location, url).toString();
        res.resume();
        getResponse(nextUrl, timeoutMs, redirectsLeft - 1)
          .then(resolve)
          .catch(reject);
        return;
      }

      if (status !== 200) {
        res.resume();
        reject(new Error(`HTTP ${status} from ${url}`));
        return;
      }

      resolve(res);
    });

    req.setTimeout(timeoutMs, () => {
      req.destroy(new Error(`request timed out after ${timeoutMs}ms`));
    });

    req.on("error", reject);
  });
}

function sha256File(filePath) {
  return new Promise((resolve, reject) => {
    const hash = crypto.createHash("sha256");
    const stream = fs.createReadStream(filePath);

    stream.on("error", reject);
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("end", () => resolve(hash.digest("hex")));
  });
}

function trimTrailingSlash(value) {
  return value.replace(/\/+$/, "");
}

function readPositiveInt(value, fallback) {
  const parsed = Number.parseInt(value || "", 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return fallback;
  }
  return parsed;
}

function removeFileIfExists(filePath) {
  try {
    fs.unlinkSync(filePath);
  } catch (error) {
    if (!error || error.code !== "ENOENT") {
      throw error;
    }
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
