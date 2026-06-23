/**
 * Copia el instalador NSIS de Tauri al nombre unificado de Paraguacraft:
 * Instalar_Paraguacraft_v{version}.exe
 *
 * Elige el setup de la versión actual (evita instaladores viejos en la misma carpeta).
 */
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { resolveNsisDir } from "./lib/resolve-nsis-dir.mjs";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const conf = JSON.parse(
  fs.readFileSync(path.join(root, "src-tauri/tauri.conf.json"), "utf8"),
);
const version = conf.version;
const nsisDir = resolveNsisDir(root);
const target = `Instalar_Paraguacraft_v${version}.exe`;

if (!nsisDir) {
  console.error("[rename-installer] Carpeta NSIS no encontrada bajo src-tauri/target");
  process.exit(1);
}

const setups = fs
  .readdirSync(nsisDir)
  .filter((f) => f.toLowerCase().endsWith(".exe") && f.toLowerCase().includes("setup"))
  .map((f) => {
    const full = path.join(nsisDir, f);
    return { name: f, full, size: fs.statSync(full).size };
  })
  .sort((a, b) => b.size - a.size);

if (setups.length === 0) {
  console.error("[rename-installer] No hay *setup*.exe en", nsisDir);
  process.exit(1);
}

// Preferir nombre que contenga la versión del manifest; si no, el más grande (build actual).
const versionNeedle = version.replace(/\./g, "_");
const setup =
  setups.find((s) => s.name.includes(version) || s.name.includes(versionNeedle)) ??
  setups[0];

const dest = path.join(nsisDir, target);
fs.copyFileSync(setup.full, dest);

console.log(`[rename-installer] Listo: ${target}`);
console.log(`[rename-installer] Origen: ${setup.name} (${setup.size} bytes)`);

if (setup.size < 4_000_000) {
  console.warn(
    "[rename-installer] AVISO: el instalador pesa menos de 4 MB — puede ser un build incompleto.",
  );
}
