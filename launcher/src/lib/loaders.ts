/** MCs donde Optimized usa Forge + OptiFine (espejo de `optimized::OPTIFINE_MCS`). */
const OPTIMIZED_FORGE_MCS = new Set(["1.8.9", "1.12.2"]);

/** Loader efectivo para APIs de tienda. Con `mc`, Optimized mapea a Forge o Fabric real. */
export function storeLoader(loader: string, mc?: string): string {
  const l = loader.trim().toLowerCase().replace(/\s+/g, "-").replace(/\+/g, "-");
  if (l.includes("optimized-neoforge") || (l.includes("optimized") && l.includes("neoforge"))) {
    return "neoforge";
  }
  if (l.includes("paraguacraft-optimized") || l === "optimized") {
    if (mc && OPTIMIZED_FORGE_MCS.has(mc)) return "forge";
    return "fabric";
  }
  if (l.includes("fabric-iris") || (l.includes("fabric") && l.includes("iris"))) return "fabric";
  if (l.includes("paraguacraft-pvp-modern") || l.includes("pvp-modern")) return "fabric";
  if (l.includes("paraguacraft-pvp") || (l.includes("paraguacraft") && l.includes("pvp")) || l === "pvp") {
    return "forge";
  }
  if (l.includes("neoforge")) return "neoforge";
  if (l.includes("quilt")) return "quilt";
  if (l.includes("fabric")) return "fabric";
  if (l.includes("optifine")) return "optifine";
  if (l.includes("forge")) return "forge";
  return "vanilla";
}

/** Id canonico del loader (espeja `loaders::normalize` en Rust). */
export function normalizeLoaderId(loader: string): string {
  const l = loader.trim().toLowerCase().replace(/\s+/g, "-").replace(/\+/g, "-");
  if (l.includes("optimized-neoforge") || (l.includes("optimized") && l.includes("neoforge"))) {
    return "paraguacraft-optimized-neoforge";
  }
  if (l.includes("paraguacraft-optimized") || l === "optimized") {
    return "paraguacraft-optimized";
  }
  if (l.includes("paraguacraft-pvp-modern") || l.includes("pvp-modern")) {
    return "paraguacraft-pvp-modern";
  }
  if (l.includes("fabric-iris") || l.includes("fabric_iris") || (l.includes("fabric") && l.includes("iris"))) {
    return "fabric-iris";
  }
  if (
    l.includes("paraguacraft-pvp") ||
    l.includes("paraguacraft_pvp") ||
    (l.includes("paraguacraft") && l.includes("pvp")) ||
    l === "pvp"
  ) {
    return "paraguacraft-pvp";
  }
  if (l.includes("neoforge")) return "neoforge";
  if (l.includes("quilt")) return "quilt";
  if (l.includes("fabric")) return "fabric";
  if (l.includes("optifine")) return "optifine";
  if (l.includes("forge")) return "forge";
  return "vanilla";
}

/** Compara loaders para instalación en tienda (Optimized respeta MC si se pasa). */
export function loadersCompatible(a: string, b: string, mc?: string): boolean {
  return storeLoader(a, mc) === storeLoader(b, mc);
}
