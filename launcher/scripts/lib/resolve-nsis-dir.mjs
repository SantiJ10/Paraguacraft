/**
 * Resuelve la carpeta NSIS de Tauri en distintos layouts de target/
 * (release nativo, triple MSVC, CARGO_TARGET_DIR de rust-cache, etc.).
 */
import fs from "fs";
import path from "path";

function existsNsis(dir) {
  const nsis = path.join(dir, "bundle/nsis");
  return fs.existsSync(nsis) ? nsis : null;
}

export function resolveNsisDir(launcherRoot) {
  const candidates = [];

  const cargoTarget = process.env.CARGO_TARGET_DIR;
  if (cargoTarget) {
    candidates.push(path.join(cargoTarget, "release"));
    candidates.push(path.join(cargoTarget, "x86_64-pc-windows-msvc/release"));
  }

  const base = path.join(launcherRoot, "src-tauri/target");
  candidates.push(path.join(base, "release"));
  candidates.push(path.join(base, "x86_64-pc-windows-msvc/release"));

  for (const root of candidates) {
    const nsis = existsNsis(root);
    if (nsis) return nsis;
  }

  return null;
}
