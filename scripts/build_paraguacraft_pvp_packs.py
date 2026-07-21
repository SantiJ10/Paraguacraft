#!/usr/bin/env python3
"""
Fusiona Dewier + 9Blue en packs Paraguacraft PvP oficiales.

Reglas:
- Terreno/bloques de construcción: NO se incluyen (vanilla fidelity).
- Sí: ores delineados, fuego bajo, cielo custom, GUI transparente, ítems/tools,
  lana (BedWars + tema azul), partículas reducidas (solo Modern).
- Modern: base 9blue + aportes convertidos de dewier + modelos de herramientas chicas.
- 1.8.9: base dewier filtrada (más simple que Modern).
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OVERLAY_189 = ROOT / "resourcepacks-src" / "overlay-189"
OVERLAY_MODERN = ROOT / "resourcepacks-src" / "overlay-modern"
PACK_LOGO = ROOT / "resourcepacks-src" / "pack.png"
OUT_189 = ROOT / "clientes" / "paraguacraft-pvp" / "packs" / "paraguacraft-pvp-189.zip"
OUT_MODERN = ROOT / "clientes" / "paraguacraft-pvp-modern" / "packs" / "paraguacraft-pvp-modern.zip"

DEWIER = Path(os.environ.get("APPDATA", "")) / ".minecraft" / "instancias" / "Paraguacraft_1.8.9_PvP" / "resourcepacks" / "dewier-20k.zip"
NINEBLUE = Path(os.environ.get("APPDATA", "")) / ".minecraft" / "instancias" / "Paraguacraft_1.21.11_PvP" / "resourcepacks" / "9blue1fault-8f16x80.zip"

SKIP_IN_ZIP = re.compile(r"(^|/)(\.DS_Store|Thumbs\.db|__MACOSX|\.git)", re.I)

# Bloques custom que SÍ queremos (visibilidad competitiva + tema azul BedWars)
BLOCK_ALLOW = re.compile(
    r"(ore|fire|wool_colored|wool\.png)",
    re.I,
)

ALWAYS_PREFIXES_189 = (
    "assets/minecraft/mcpatcher/sky/",
    "assets/minecraft/mcpatcher/font/",
    "assets/minecraft/textures/gui/",
    "assets/minecraft/textures/items/",
)

ALWAYS_PREFIXES_MODERN = (
    "assets/minecraft/optifine/sky/",
    "assets/minecraft/textures/gui/",
    "assets/minecraft/textures/item/",
    "assets/minecraft/textures/particle/",
)

MODERN_TOOLS = [
    "wooden_sword", "stone_sword", "iron_sword", "golden_sword", "diamond_sword", "netherite_sword",
    "wooden_axe", "stone_axe", "iron_axe", "golden_axe", "diamond_axe", "netherite_axe",
    "wooden_pickaxe", "stone_pickaxe", "iron_pickaxe", "golden_pickaxe", "diamond_pickaxe", "netherite_pickaxe",
    "wooden_shovel", "stone_shovel", "iron_shovel", "golden_shovel", "diamond_shovel", "netherite_shovel",
    "wooden_hoe", "stone_hoe", "iron_hoe", "golden_hoe", "diamond_hoe", "netherite_hoe",
    "mace", "trident", "bow", "crossbow", "fishing_rod", "shield",
]

SMALL_TOOL_MODEL = """{{
  "parent": "minecraft:item/{parent}",
  "textures": {{
    "layer0": "minecraft:item/{item}"
  }},
  "display": {{
    "thirdperson_righthand": {{ "rotation": [0, -90, 55], "translation": [0, 4.0, 0.5], "scale": [0.50, 0.50, 0.50] }},
    "thirdperson_lefthand": {{ "rotation": [0, 90, -55], "translation": [0, 4.0, 0.5], "scale": [0.50, 0.50, 0.50] }},
    "firstperson_righthand": {{ "rotation": [0, -90, 25], "translation": [1.13, 3.2, 1.13], "scale": [0.50, 0.50, 0.50] }},
    "firstperson_lefthand": {{ "rotation": [0, 90, -25], "translation": [1.13, 3.2, 1.13], "scale": [0.50, 0.50, 0.50] }},
    "ground": {{ "translation": [0, 2, 0], "scale": [0.42, 0.42, 0.42] }},
    "fixed": {{ "scale": [0.52, 0.52, 0.52] }},
    "gui": {{ "scale": [0.82, 0.82, 0.82] }}
  }}
}}
"""


def sha1_file(path: Path) -> str:
    h = hashlib.sha1()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def norm(path: str) -> str:
    return path.replace("\\", "/")


def should_skip(path: str) -> bool:
    return bool(SKIP_IN_ZIP.search(path))


def block_path_ok(path: str) -> bool:
    name = Path(path).name
    if not name.endswith((".png", ".mcmeta")):
        return False
    base = name.replace(".mcmeta", "")
    return bool(BLOCK_ALLOW.search(base))


def include_189(path: str) -> bool:
    p = norm(path)
    if should_skip(p):
        return False
    for pref in ALWAYS_PREFIXES_189:
        if p.startswith(pref):
            return True
    if "/textures/blocks/" in p:
        return block_path_ok(p)
    return False


def include_modern(path: str) -> bool:
    p = norm(path)
    if should_skip(p):
        return False
    for pref in ALWAYS_PREFIXES_MODERN:
        if p.startswith(pref):
            return True
    if "/textures/block/" in p:
        return block_path_ok(p)
    return False


def dewier_to_modern(path: str) -> str | None:
    """Convierte rutas 1.8.9 → 1.21 donde tenga sentido."""
    p = norm(path)
    if p.startswith("assets/minecraft/textures/blocks/"):
        rest = p.split("textures/blocks/", 1)[1]
        if not block_path_ok(p):
            return None
        return f"assets/minecraft/textures/block/{rest}"
    if p.startswith("assets/minecraft/textures/items/"):
        rest = p.split("textures/items/", 1)[1]
        return f"assets/minecraft/textures/item/{rest}"
    if p.startswith("assets/minecraft/mcpatcher/sky/"):
        rest = p.split("mcpatcher/sky/", 1)[1]
        return f"assets/minecraft/optifine/sky/{rest}"
    return None


def read_zip_filtered(zip_path: Path, predicate) -> dict[str, bytes]:
    out: dict[str, bytes] = {}
    with zipfile.ZipFile(zip_path, "r") as zin:
        for info in zin.infolist():
            if info.is_dir():
                continue
            p = norm(info.filename)
            if predicate(p):
                out[p] = zin.read(info.filename)
    return out


def merge_entries(*layers: dict[str, bytes]) -> dict[str, bytes]:
    merged: dict[str, bytes] = {}
    for layer in layers:
        merged.update(layer)
    return merged


def apply_overlay(entries: dict[str, bytes], overlay_dir: Path) -> None:
    if PACK_LOGO.is_file():
        entries["pack.png"] = PACK_LOGO.read_bytes()
    for path in overlay_dir.rglob("*"):
        if path.is_file():
            rel = path.relative_to(overlay_dir).as_posix()
            entries[rel] = path.read_bytes()


def write_modern_tool_models(entries: dict[str, bytes]) -> None:
    for item in MODERN_TOOLS:
        parent = "builtin/entity" if item == "shield" else "handheld"
        rel = f"assets/minecraft/models/item/{item}.json"
        entries[rel] = SMALL_TOOL_MODEL.format(parent=parent, item=item).encode("utf-8")


def write_zip(entries: dict[str, bytes], out: Path) -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    tmp = out.with_suffix(".part")
    with zipfile.ZipFile(tmp, "w", compression=zipfile.ZIP_DEFLATED) as zout:
        for name in sorted(entries.keys()):
            zout.writestr(name, entries[name])
    if out.exists():
        out.unlink()
    tmp.rename(out)


def build_189() -> tuple[Path, str]:
    if not DEWIER.is_file():
        raise FileNotFoundError(DEWIER)
    entries = read_zip_filtered(DEWIER, include_189)
    apply_overlay(entries, OVERLAY_189)
    write_zip(entries, OUT_189)
    return OUT_189, sha1_file(OUT_189)


def build_modern() -> tuple[Path, str]:
    if not NINEBLUE.is_file():
        raise FileNotFoundError(NINEBLUE)

    dewier_modern: dict[str, bytes] = {}
    with zipfile.ZipFile(DEWIER, "r") as zin:
        for info in zin.infolist():
            if info.is_dir():
                continue
            src = norm(info.filename)
            if not include_189(src):
                continue
            dst = dewier_to_modern(src)
            if dst:
                dewier_modern[dst] = zin.read(info.filename)

    nineblue = read_zip_filtered(NINEBLUE, include_modern)
    entries = merge_entries(dewier_modern, nineblue)
    apply_overlay(entries, OVERLAY_MODERN)
    write_modern_tool_models(entries)
    write_zip(entries, OUT_MODERN)
    return OUT_MODERN, sha1_file(OUT_MODERN)


def update_catalog(catalog_path: Path, pack_id: str, title: str, subtitle: str, file_name: str, sha1: str) -> None:
    if not catalog_path.is_file():
        return
    data = json.loads(catalog_path.read_text(encoding="utf-8"))
    entry = {
        "id": pack_id,
        "title": title,
        "subtitle": subtitle,
        "badge": "16x",
        "fileName": file_name,
        "sha1": sha1,
    }
    packs = [p for p in data.get("packs", []) if p.get("id") != pack_id]
    packs.insert(0, entry)
    data["packs"] = packs
    catalog_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def sync_catalogs(file_189: str, sha_189: str, file_modern: str, sha_modern: str) -> None:
    subtitle_189 = "Dewier×Blue · cielo · fuego bajo · ores · tools chicas"
    subtitle_modern = "Dewier×Blue · completo 1.21 · partículas · tools chicas"
    paths_189 = [
        ROOT / "clientes" / "paraguacraft-pvp" / "packs" / "catalog.json",
        ROOT / "client" / "src" / "main" / "resources" / "assets" / "paraguacraft" / "packs" / "catalog.json",
    ]
    paths_modern = [
        ROOT / "clientes" / "paraguacraft-pvp-modern" / "packs" / "catalog.json",
        ROOT / "client-modern" / "src" / "main" / "resources" / "assets" / "paraguacraftpvp-modern" / "packs" / "catalog.json",
        ROOT / "client-modern" / "src" / "main" / "resources" / "assets" / "paraguacraft-modern" / "packs" / "catalog.json",
        ROOT / "bundled" / "pvp-modern" / "packs" / "catalog.json",
        ROOT / "launcher" / "src-tauri" / "resources" / "bundled" / "pvp-modern" / "packs" / "catalog.json",
    ]
    for p in paths_189:
        update_catalog(p, "paraguacraft-pvp", "Paraguacraft PvP", subtitle_189, file_189, sha_189)
    for p in paths_modern:
        update_catalog(p, "paraguacraft-pvp", "Paraguacraft PvP", subtitle_modern, file_modern, sha_modern)


def main() -> int:
    global DEWIER, NINEBLUE
    if len(sys.argv) > 1:
        DEWIER = Path(sys.argv[1])
    if len(sys.argv) > 2:
        NINEBLUE = Path(sys.argv[2])

    print("=== Paraguacraft PvP pack fusion ===")
    print(f"Dewier:   {DEWIER}")
    print(f"9Blue:    {NINEBLUE}")

    out189, sha189 = build_189()
    print(f"1.8.9  -> {out189.name}  sha1={sha189}  size={out189.stat().st_size // 1024}KB")

    out_modern, sha_modern = build_modern()
    print(f"1.21   -> {out_modern.name}  sha1={sha_modern}  size={out_modern.stat().st_size // 1024}KB")

    sync_catalogs(out189.name, sha189, out_modern.name, sha_modern)
    print("Catálogos actualizados. Subí los .zip a GitHub Releases cuando publiques.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
