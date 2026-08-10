#!/usr/bin/env python3
"""
Oleada 2 del pack oficial Paraguacraft PvP:

1) Armor entity (otros jugadores)
   - 1.8.9: models/armor/*_layer_*.png desde Dewier
   - Modern: entity/equipment/humanoid(+_leggings) incl. netherite (9blue / modern zip)

2) Partículas crit más legibles en 1.8.9
   - atlas vanilla 128×128 con celdas crit/magicCrit reforzadas

3) Bridge / arena limpia 1.8.9
   - sand, gravel, stone, cobble, stonebrick*, slabs, brick, endstone
"""

from __future__ import annotations

import io
import os
import shutil
import struct
import zipfile
from pathlib import Path

from PIL import Image, ImageEnhance, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
OVERLAY_189 = ROOT / "resourcepacks-src" / "overlay-189"
OVERLAY_MODERN = ROOT / "resourcepacks-src" / "overlay-modern"

DEWIER = Path(
    os.environ.get(
        "DEWIER_ZIP",
        Path(os.environ.get("APPDATA", ""))
        / ".minecraft"
        / "instancias"
        / "Prueba_Paraguacraft"
        / "resourcepacks"
        / "dewier-20k.zip",
    )
)
VANILLA_189 = Path(
    os.environ.get(
        "MC_189_JAR",
        Path(os.environ.get("APPDATA", "")) / ".minecraft" / "versions" / "1.8.9" / "1.8.9.jar",
    )
)
NINEBLUE_DIR = ROOT / ".tmp-rp-9blue"
OFFICIAL_MODERN = (
    ROOT / "clientes" / "paraguacraft-pvp-modern" / "packs" / "paraguacraft-pvp-modern.zip"
)


def read_zip_file(zip_path: Path, inner: str) -> bytes | None:
    if not zip_path.is_file():
        return None
    with zipfile.ZipFile(zip_path, "r") as z:
        try:
            return z.read(inner)
        except KeyError:
            return None


def write_bytes(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def copy_zip_to_overlay(zip_path: Path, inner: str, dest: Path) -> bool:
    data = read_zip_file(zip_path, inner)
    if data is None:
        return False
    write_bytes(dest, data)
    return True


def copy_file(src: Path, dest: Path) -> bool:
    if not src.is_file():
        return False
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dest)
    return True


def sync_armor_189() -> int:
    """Dewier armor layers → overlay-189."""
    materials = [
        "chainmail_layer_1.png",
        "chainmail_layer_2.png",
        "diamond_layer_1.png",
        "diamond_layer_2.png",
        "gold_layer_1.png",
        "gold_layer_2.png",
        "iron_layer_1.png",
        "iron_layer_2.png",
        "leather_layer_1.png",
        "leather_layer_1_overlay.png",
        "leather_layer_2.png",
        "leather_layer_2_overlay.png",
    ]
    n = 0
    for name in materials:
        inner = f"assets/minecraft/textures/models/armor/{name}"
        dest = OVERLAY_189 / "assets" / "minecraft" / "textures" / "models" / "armor" / name
        if copy_zip_to_overlay(DEWIER, inner, dest):
            n += 1
            print(f"  armor189 {name}")
        else:
            print(f"  MISS armor189 {name}")
    return n


def sync_armor_modern() -> int:
    """
    Equipment modern (1.21 uses entity/equipment, no models/armor path).
    Fuente: pack oficial actual + 9blue extract para netherite.
    """
    n = 0
    mats = ["chainmail", "diamond", "gold", "iron", "leather", "netherite"]
    for mat in mats:
        for folder in ("humanoid", "humanoid_leggings"):
            for suffix in ("", "_overlay") if mat == "leather" else ("",):
                if mat == "leather" and suffix == "_overlay":
                    name = "leather_overlay.png"
                else:
                    name = f"{mat}{suffix}.png" if suffix else f"{mat}.png"
                rel = f"assets/minecraft/textures/entity/equipment/{folder}/{name}"
                dest = OVERLAY_MODERN / Path(rel)
                # prefer 9blue for netherite; else official zip; else 9blue
                ok = False
                if NINEBLUE_DIR.is_dir():
                    ok = copy_file(NINEBLUE_DIR / rel, dest)
                if not ok:
                    ok = copy_zip_to_overlay(OFFICIAL_MODERN, rel, dest)
                if not ok and mat != "netherite":
                    # dual: models/armor in 9blue can be mapped to equipment
                    layer = "1" if folder == "humanoid" else "2"
                    legacy = (
                        NINEBLUE_DIR
                        / "assets"
                        / "minecraft"
                        / "textures"
                        / "models"
                        / "armor"
                        / f"{mat}_layer_{layer}.png"
                    )
                    ok = copy_file(legacy, dest)
                if ok:
                    n += 1
                    print(f"  armorMod {folder}/{name}")
                else:
                    print(f"  MISS armorMod {folder}/{name}")
    return n


# UV indices in 1.8.9 particles.png (8×8 cells, 16 columns on 128 atlas)
# crit / magicCrit use texture index 65 (EntityCrit2FX)
CRIT_INDICES = (65, 66, 67)  # crit + vecinos por si el atlas difiere un poco


def cell_box(index: int) -> tuple[int, int, int, int]:
    col = index % 16
    row = index // 16
    x0, y0 = col * 8, row * 8
    return x0, y0, x0 + 8, y0 + 8


def make_bright_crit_sprite(base: Image.Image | None) -> Image.Image:
    """Sprite 8×8 tipo estrella/crit, alto contraste."""
    if base is not None:
        img = base.convert("RGBA").resize((8, 8), Image.Resampling.NEAREST)
        # subir brillo/contraste
        img = ImageEnhance.Contrast(img).enhance(1.35)
        img = ImageEnhance.Brightness(img).enhance(1.25)
        return img

    # estrella procedural
    out = Image.new("RGBA", (8, 8), (0, 0, 0, 0))
    px = out.load()
    # white/gold cross + diagonals
    colors = [
        ((3, 0), (255, 255, 200, 255)),
        ((3, 1), (255, 255, 180, 255)),
        ((3, 2), (255, 240, 120, 255)),
        ((3, 3), (255, 255, 255, 255)),
        ((3, 4), (255, 240, 120, 255)),
        ((3, 5), (255, 255, 180, 255)),
        ((3, 6), (255, 255, 200, 255)),
        ((0, 3), (255, 255, 200, 255)),
        ((1, 3), (255, 255, 180, 255)),
        ((2, 3), (255, 240, 120, 255)),
        ((4, 3), (255, 240, 120, 255)),
        ((5, 3), (255, 255, 180, 255)),
        ((6, 3), (255, 255, 200, 255)),
        ((2, 2), (255, 220, 80, 220)),
        ((4, 2), (255, 220, 80, 220)),
        ((2, 4), (255, 220, 80, 220)),
        ((4, 4), (255, 220, 80, 220)),
    ]
    for (x, y), c in colors:
        px[x, y] = c
    return out


def patch_particles_189() -> bool:
    if not VANILLA_189.is_file():
        print("  MISS vanilla 1.8.9.jar — skip particles")
        return False
    data = read_zip_file(VANILLA_189, "assets/minecraft/textures/particle/particles.png")
    if not data:
        print("  MISS particles in jar")
        return False
    atlas = Image.open(io.BytesIO(data)).convert("RGBA")

    # modern critical as seed
    crit_src = None
    crit_bytes = read_zip_file(
        OFFICIAL_MODERN, "assets/minecraft/textures/particle/critical_hit.png"
    )
    if crit_bytes:
        crit_src = Image.open(io.BytesIO(crit_bytes)).convert("RGBA")
    bright = make_bright_crit_sprite(crit_src)

    # also boost magicCrit-like cells near 65 and the "spell" row for visibility
    for idx in CRIT_INDICES:
        box = cell_box(idx)
        atlas.paste(bright, box[:2], bright)

    # damage heart-ish index 17 often used for hearts — leave alone
    # boost index 0..7 generic if needed - skip

    dest = (
        OVERLAY_189
        / "assets"
        / "minecraft"
        / "textures"
        / "particle"
        / "particles.png"
    )
    dest.parent.mkdir(parents=True, exist_ok=True)
    atlas.save(dest, format="PNG")
    print(f"  particles189 {atlas.size} -> {dest.relative_to(ROOT)}")
    return True


def sync_modern_particles() -> int:
    """Asegura particles crit/sweep/damage del overlay (ya existen; reafirma desde pack)."""
    names = [
        "critical_hit.png",
        "damage.png",
        "enchanted_hit.png",
        "sweep_0.png",
        "sweep_1.png",
        "sweep_2.png",
        "sweep_3.png",
        "sweep_4.png",
        "sweep_5.png",
        "sweep_6.png",
        "sweep_7.png",
    ]
    n = 0
    particle_dir = OVERLAY_MODERN / "assets" / "minecraft" / "textures" / "particle"
    for name in names:
        dest = particle_dir / name
        if dest.is_file():
            # boost contrast of existing
            im = Image.open(dest).convert("RGBA")
            im = ImageEnhance.Contrast(im).enhance(1.2)
            im = ImageEnhance.Brightness(im).enhance(1.15)
            im.save(dest, format="PNG")
            n += 1
            print(f"  particleMod boost {name}")
            continue
        inner = f"assets/minecraft/textures/particle/{name}"
        if copy_zip_to_overlay(OFFICIAL_MODERN, inner, dest):
            n += 1
            print(f"  particleMod copy {name}")
    return n


def sync_bridge_189() -> int:
    """Bloques de bridging / arena desde Dewier."""
    blocks = [
        "sand.png",
        "gravel.png",
        "stone.png",
        "cobblestone.png",
        "stonebrick.png",
        "stonebrick_cracked.png",
        "stonebrick_mossy.png",
        "stonebrick_carved.png",
        "stone_slab_side.png",
        "stone_slab_top.png",
        "brick.png",
        "clay.png",
        # extras utiles en SW/BW hypixel maps
        "sponge.png",
        "sponge_wet.png",
        "quartz_block_side.png",
        "quartz_block_top.png",
        "quartz_block_bottom.png",
        "quartz_block_chiseled.png",
        "quartz_block_chiseled_top.png",
        "quartz_block_lines.png",
        "quartz_block_lines_top.png",
        "netherrack.png",
        "glowstone.png",
        "soul_sand.png",
    ]
    n = 0
    for name in blocks:
        inner = f"assets/minecraft/textures/blocks/{name}"
        dest = OVERLAY_189 / "assets" / "minecraft" / "textures" / "blocks" / name
        if copy_zip_to_overlay(DEWIER, inner, dest):
            n += 1
            print(f"  bridge189 {name}")
        else:
            print(f"  skip bridge189 {name}")
    return n


def sync_bridge_modern_from_pack() -> int:
    """Trae sand/stone/cobble del pack moderno actual al overlay (si faltan)."""
    names = [
        "sand.png",
        "gravel.png",
        "stone.png",
        "cobblestone.png",
        "stone_bricks.png",
        "cracked_stone_bricks.png",
        "mossy_stone_bricks.png",
        "chiseled_stone_bricks.png",
        "smooth_stone.png",
        "smooth_stone_slab_side.png",
        "brick.png",
        "bricks.png",
        "andesite.png",
        "diorite.png",
        "granite.png",
        "polished_andesite.png",
        "polished_diorite.png",
        "polished_granite.png",
        "deepslate.png",
        "cobbled_deepslate.png",
    ]
    n = 0
    block_dir = OVERLAY_MODERN / "assets" / "minecraft" / "textures" / "block"
    for name in names:
        dest = block_dir / name
        if dest.is_file():
            continue
        # try official pack first, then 9blue
        inner = f"assets/minecraft/textures/block/{name}"
        if copy_zip_to_overlay(OFFICIAL_MODERN, inner, dest):
            n += 1
            print(f"  bridgeMod {name}")
            continue
        if copy_file(NINEBLUE_DIR / inner, dest):
            n += 1
            print(f"  bridgeMod9b {name}")
    # also copy dewier stone path remapped if still missing sand - already in modern pack
    return n


def main() -> int:
    print("=== Wave 2: armor / particles / bridge ===")
    print(f"Dewier: {DEWIER} exists={DEWIER.is_file()}")
    print(f"Vanilla jar: {VANILLA_189} exists={VANILLA_189.is_file()}")
    print(f"9blue dir: {NINEBLUE_DIR} exists={NINEBLUE_DIR.is_dir()}")

    print("\n[1] Armor 1.8.9")
    a1 = sync_armor_189()
    print("\n[2] Armor modern (+netherite)")
    a2 = sync_armor_modern()
    print("\n[3] Particles 1.8.9")
    p1 = patch_particles_189()
    print("\n[4] Particles modern boost")
    p2 = sync_modern_particles()
    print("\n[5] Bridge 1.8.9")
    b1 = sync_bridge_189()
    print("\n[6] Bridge modern fill")
    b2 = sync_bridge_modern_from_pack()

    print(
        f"\nDone armor189={a1} armorMod={a2} particles189={p1} "
        f"particleMod={p2} bridge189={b1} bridgeMod={b2}"
    )
    print("Next: python scripts/patch_189_pack_overlay.py && python scripts/patch_modern_pack_overlay.py")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
