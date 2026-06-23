/**
 * Renombra/copia el instalador NSIS de Tauri al nombre unificado de Paraguacraft:
 * Instalar_Paraguacraft_v{version}.exe
 *
 * Mismo nombre que usaba el launcher Python → migración transparente vía latest.json.
 */
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const conf = JSON.parse(
  fs.readFileSync(path.join(root, "src-tauri/tauri.conf.json"), "utf8"),
);
const version = conf.version;
const targetRoot = process.env.CARGO_TARGET_DIR
  ? path.join(process.env.CARGO_TARGET_DIR, "release")
  : path.join(root, "src-tauri/target/release");
const nsisDir = path.join(targetRoot, "bundle/nsis");
const target = `Instalar_Paraguacraft_v${version}.exe`;

if (!fs.existsSync(nsisDir)) {
  console.warn("[rename-installer] Carpeta NSIS no encontrada (¿build incompleto?).");
  process.exit(0);
}

const candidates = fs
  .readdirSync(nsisDir)
  .filter((f) => f.toLowerCase().endsWith(".exe") && f !== target);

const setup =
  candidates.find((f) => f.toLowerCase().includes("setup")) ?? candidates[0];

if (!setup) {
  console.error("[rename-installer] No hay .exe en", nsisDir);
  process.exit(1);
}

const src = path.join(nsisDir, setup);
const dest = path.join(nsisDir, target);

fs.copyFileSync(src, dest);
console.log(`[rename-installer] Listo: ${target}`);
console.log(`[rename-installer] Origen: ${setup}`);
