/** Loader efectivo para APIs de tienda (fabric-iris usa mods de Fabric). */
export function storeLoader(loader: string): string {
  const l = loader.trim().toLowerCase().replace(/\s+/g, "-").replace(/\+/g, "-");
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

/** Compara loaders para instalación en tienda (fabric-iris ≡ fabric). */
export function loadersCompatible(a: string, b: string): boolean {
  return storeLoader(a) === storeLoader(b);
}
