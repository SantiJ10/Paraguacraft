// Genera la union de nombres de comandos IPC a partir del registro de Rust.
//
// `invoke` recibe el nombre del comando como string, asi que renombrar o borrar
// un comando en Rust no rompia la compilacion del frontend: fallaba recien al
// apretar el boton. Con la union generada, vue-tsc lo marca.
//
//   node scripts/gen-ipc-commands.mjs          escribe el archivo
//   node scripts/gen-ipc-commands.mjs --check  falla si quedo desactualizado

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const SOURCE = join(root, "src-tauri", "src", "lib.rs");
const OUTPUT = join(root, "src", "lib", "ipc-commands.ts");

/** Extrae el bloque `generate_handler![ ... ]` balanceando los corchetes. */
function handlerBlock(rust) {
  const marker = "generate_handler![";
  const start = rust.indexOf(marker);
  if (start === -1) throw new Error(`No se encontro generate_handler! en ${SOURCE}`);
  let depth = 0;
  for (let i = start + marker.length - 1; i < rust.length; i++) {
    if (rust[i] === "[") depth++;
    else if (rust[i] === "]") {
      depth--;
      if (depth === 0) return rust.slice(start + marker.length, i);
    }
  }
  throw new Error("generate_handler! quedo sin cerrar");
}

function commandNames(block) {
  const names = block
    .replace(/\/\/[^\n]*/g, "") // comentarios de seccion
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean)
    .map((entry) => entry.split("::").pop().trim())
    .filter((name) => /^[a-z_][a-z0-9_]*$/.test(name));

  const unique = [...new Set(names)].sort();
  if (unique.length !== names.length) {
    const seen = new Set();
    const dupes = names.filter((n) => (seen.has(n) ? true : (seen.add(n), false)));
    throw new Error(`Comandos duplicados en el registro: ${[...new Set(dupes)].join(", ")}`);
  }
  return unique;
}

function render(names) {
  return [
    "// Generado por scripts/gen-ipc-commands.mjs a partir de src-tauri/src/lib.rs.",
    "// No editar a mano: correr `npm run gen:ipc`.",
    "",
    "/** Comandos registrados en el backend de Rust. */",
    "export type IpcCommand =",
    ...names.map((name) => `  | ${JSON.stringify(name)}`),
    "  ;",
    "",
  ].join("\n");
}

const names = commandNames(handlerBlock(readFileSync(SOURCE, "utf8")));
const rendered = render(names);

if (process.argv.includes("--check")) {
  let current = "";
  try {
    current = readFileSync(OUTPUT, "utf8");
  } catch {
    /* todavia no existe */
  }
  if (current !== rendered) {
    console.error("ipc-commands.ts esta desactualizado. Corre `npm run gen:ipc`.");
    process.exit(1);
  }
  console.log(`ipc-commands.ts al dia (${names.length} comandos).`);
} else {
  writeFileSync(OUTPUT, rendered);
  console.log(`Escritos ${names.length} comandos en ${OUTPUT}.`);
}
