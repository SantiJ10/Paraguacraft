/**
 * Actualiza latest.json en la raíz del repo con size_bytes y sha256 del instalador.
 * Uso: node scripts/patch-latest-json.mjs [ruta/al/Instalar_Paraguacraft_vX.exe]
 */
import crypto from "crypto";
import fs from "fs";
import path from "path";
import { execSync } from "child_process";
import { fileURLToPath } from "url";
import { resolveNsisDir } from "./lib/resolve-nsis-dir.mjs";

const launcherRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = path.join(launcherRoot, "..");
const conf = JSON.parse(
  fs.readFileSync(path.join(launcherRoot, "src-tauri/tauri.conf.json"), "utf8"),
);
const version = conf.version;
const latestPath = path.join(repoRoot, "latest.json");

function findInstaller() {
  const nsisDir = resolveNsisDir(launcherRoot);
  if (!nsisDir) return null;
  const name = `Instalar_Paraguacraft_v${version}.exe`;
  const full = path.join(nsisDir, name);
  return fs.existsSync(full) ? full : null;
}

const exePath = process.argv[2] ? path.resolve(process.argv[2]) : findInstaller();
if (!exePath || !fs.existsSync(exePath)) {
  console.error("[patch-latest-json] No se encontró el instalador:", exePath ?? "(auto)");
  process.exit(1);
}

const bytes = fs.readFileSync(exePath);
const sha256 = crypto.createHash("sha256").update(bytes).digest("hex");
const size_bytes = bytes.length;

const latest = fs.existsSync(latestPath)
  ? JSON.parse(fs.readFileSync(latestPath, "utf8"))
  : {};

latest.version = version;
latest.download_url = `https://github.com/SantiJ10/Paraguacraft/releases/download/v${version}/Instalar_Paraguacraft_v${version}.exe`;
latest.size_bytes = size_bytes;
latest.sha256 = sha256;

// Formato dual: fallback manual (sha256) + plugin updater Tauri cuando haya firma.
latest.platforms = {
  "windows-x86_64": {
    url: latest.download_url,
    signature: latest.signature ?? "",
  },
};

if (process.env.TAURI_SIGNING_PRIVATE_KEY) {
  try {
    const sig = execSync(`npx tauri signer sign "${exePath}"`, {
      cwd: launcherRoot,
      env: { ...process.env },
      encoding: "utf8",
    }).trim();
    latest.signature = sig;
    latest.platforms["windows-x86_64"].signature = sig;
    console.log("[patch-latest-json] Firma minisign generada");
  } catch (e) {
    console.warn("[patch-latest-json] No se pudo firmar:", e?.message ?? e);
  }
}

fs.writeFileSync(latestPath, `${JSON.stringify(latest, null, 2)}\n`, "utf8");

const outCopy = path.join(launcherRoot, "dist/latest.json");
fs.mkdirSync(path.dirname(outCopy), { recursive: true });
fs.copyFileSync(latestPath, outCopy);

console.log(`[patch-latest-json] ${latestPath}`);
console.log(`[patch-latest-json] size=${size_bytes} sha256=${sha256}`);
