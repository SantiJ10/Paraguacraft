#!/usr/bin/env python3
"""
Oleada 3 del pack oficial:
- HUD hearts/hunger/hotbar (modern sprites 9blue)
- icons.png / widgets.png reafirmados desde Dewier (1.8)
- fuente ASCII más “fina” (delgada) en 1.8 via reconstruir ascii.png
- crosshair más limpio si hay dewier
"""

from __future__ import annotations

import io
import os
import shutil
import zipfile
from pathlib import Path

from PIL import Image, ImageEnhance, ImageFilter, ImageOps

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
NINEBLUE = ROOT / ".tmp-rp-9blue"
VANILLA_189 = Path(
    os.environ.get(
        "MC_189_JAR",
        Path(os.environ.get("APPDATA", "")) / ".minecraft" / "versions" / "1.8.9" / "1.8.9.jar",
    )
)


def zip_read(zpath: Path, inner: str) -> bytes | None:
    if not zpath.is_file():
        return None
    with zipfile.ZipFile(zpath, "r") as z:
        try:
            return z.read(inner)
        except KeyError:
            return None


def write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def sync_icons_189() -> int:
    n = 0
    for name in ("icons.png", "widgets.png"):
        inner = f"assets/minecraft/textures/gui/{name}"
        data = zip_read(DEWIER, inner)
        if not data:
            print(f"  MISS dew {name}")
            continue
        # contrast slightly for HUD readability
        im = Image.open(io.BytesIO(data)).convert("RGBA")
        im = ImageEnhance.Contrast(im).enhance(1.08)
        im = ImageEnhance.Sharpness(im).enhance(1.2)
        dest = OVERLAY_189 / "assets" / "minecraft" / "textures" / "gui" / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        im.save(dest, format="PNG")
        n += 1
        print(f"  icons189 {name} {im.size}")
    return n


def thin_ascii_font_189() -> bool:
    """
    Fuente ASCII más fina: reduce el “peso” de cada glifo (erosiona alpha)
    sin cambiar métricas. Fuente: dewier font/ascii.png o vanilla.
    """
    candidates = [
        ("zip", DEWIER, "assets/minecraft/mcpatcher/font/ascii.png"),
        ("zip", DEWIER, "assets/minecraft/textures/font/ascii.png"),
        ("zip", VANILLA_189, "assets/minecraft/textures/font/ascii.png"),
    ]
    data = None
    for _, zpath, inner in candidates:
        data = zip_read(zpath, inner)
        if data:
            print(f"  font source {inner}")
            break
    if not data:
        print("  MISS ascii font source")
        return False

    im = Image.open(io.BytesIO(data)).convert("RGBA")
    # Erosionar solo el canal alpha → glifos más delgados
    r, g, b, a = im.split()
    # dilate mask of solid pixels then combine: keep center
    solid = a.point(lambda p: 255 if p > 32 else 0)
    # erode by min-filter on alpha
    thinned = a.filter(ImageFilter.MinFilter(3))
    # prefer original where thinned too weak
    a2 = Image.composite(thinned, a, solid)
    # slight contrast on color
    rgb = Image.merge("RGB", (r, g, b))
    rgb = ImageEnhance.Contrast(rgb).enhance(1.05)
    r, g, b = rgb.split()
    out = Image.merge("RGBA", (r, g, b, a2))

    # write both OptiFine/mcpatcher and vanilla paths
    paths = [
        OVERLAY_189 / "assets/minecraft/mcpatcher/font/ascii.png",
        OVERLAY_189 / "assets/minecraft/textures/font/ascii.png",
    ]
    for p in paths:
        p.parent.mkdir(parents=True, exist_ok=True)
        out.save(p, format="PNG")
        print(f"  font -> {p.relative_to(ROOT)}")
    return True


def sync_hud_modern() -> int:
    src_root = NINEBLUE / "assets" / "minecraft" / "textures" / "gui"
    if not src_root.is_dir():
        print("  MISS 9blue gui")
        return 0
    dest_root = OVERLAY_MODERN / "assets" / "minecraft" / "textures" / "gui"
    n = 0
    # hearts, food, hotbar, experience if present
    patterns = [
        "sprites/hud/heart",
        "sprites/hud/food_*.png",
        "sprites/hud/hotbar*.png",
        "sprites/hud/jump_bar*.png",
        "sprites/hud/experience_bar*.png",
        "sprites/hud/armor*.png",
        "sprites/hud/air*.png",
    ]
    files: list[Path] = []
    hud = src_root / "sprites" / "hud"
    if hud.is_dir():
        files.extend(hud.rglob("*.png"))
    for f in files:
        rel = f.relative_to(src_root)
        dest = dest_root / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        # mild contrast boost
        im = Image.open(f).convert("RGBA")
        im = ImageEnhance.Contrast(im).enhance(1.1)
        im = ImageEnhance.Brightness(im).enhance(1.05)
        im.save(dest, format="PNG")
        n += 1
    print(f"  modern hud sprites {n}")
    return n


def sync_modern_icons_fallback() -> int:
    """Si 9blue tiene icons/widgets legacy, copiar."""
    n = 0
    for name in ("icons.png", "widgets.png"):
        src = NINEBLUE / "assets/minecraft/textures/gui" / name
        if not src.is_file():
            continue
        dest = OVERLAY_MODERN / "assets/minecraft/textures/gui" / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)
        n += 1
        print(f"  modern {name}")
    return n


def main() -> int:
    print("=== Wave 3: HUD / font ===")
    print(f"Dewier={DEWIER.is_file()} 9blue={NINEBLUE.is_dir()}")
    print("\n[1] 1.8 icons/widgets")
    sync_icons_189()
    print("\n[2] 1.8 thinner ascii font")
    thin_ascii_font_189()
    print("\n[3] modern HUD hearts/food/hotbar")
    sync_hud_modern()
    print("\n[4] modern icons fallback")
    sync_modern_icons_fallback()
    # mcmeta
    (OVERLAY_189 / "pack.mcmeta").write_text(
        """{
  "pack": {
    "pack_format": 1,
    "description": "§9Paraguacraft §fPvP §8· §7HUD · armor · crit · bridge"
  }
}
""",
        encoding="utf-8",
    )
    (OVERLAY_MODERN / "pack.mcmeta").write_text(
        """{
  "pack": {
    "pack_format": 75,
    "description": "Paraguacraft PvP · HUD hearts · armor · crit · bridge",
    "min_format": [75, 0],
    "max_format": [99, 0]
  }
}
""",
        encoding="utf-8",
    )
    print("\nNext: patch_189 + patch_modern")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
