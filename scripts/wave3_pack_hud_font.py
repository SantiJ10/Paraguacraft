#!/usr/bin/env python3
"""
Oleada 3 del pack oficial:
- HUD: icons/widgets (hearts, hunger, armor, crosshair) 1.8 más nítidos
- HUD modern: hearts/food/hotbar desde 9blue + contraste
- Fuente: ascii más limpia/delgada (sin grasa innecesaria)
"""

from __future__ import annotations

import io
import os
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
VANILLA_189 = Path(
    os.environ.get(
        "MC_189_JAR",
        Path(os.environ.get("APPDATA", "")) / ".minecraft" / "versions" / "1.8.9" / "1.8.9.jar",
    )
)
NINEBLUE = ROOT / ".tmp-rp-9blue"
OFFICIAL_MODERN = (
    ROOT / "clientes" / "paraguacraft-pvp-modern" / "packs" / "paraguacraft-pvp-modern.zip"
)


def read_zip(path: Path, inner: str) -> bytes | None:
    if not path.is_file():
        return None
    with zipfile.ZipFile(path) as z:
        try:
            return z.read(inner)
        except KeyError:
            return None


def save_png(path: Path, img: Image.Image) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    img.save(path, format="PNG")


def enhance_icons_189() -> bool:
    """
    icons.png 1.8: sheet clásico.
    Filas típicas (y aprox.) en 256px: hearts ~ y 0-8, hunger ~9-18, armor ~ y 9? Actually layout:
    - hearts empty/full half at y=0
    - hunger at y=27
    - armor at y=9
    - air bubbles y=18
    Boost only the heart/hunger/armor rows slightly for clarity without recoloring theme.
    """
    data = read_zip(DEWIER, "assets/minecraft/textures/gui/icons.png")
    if not data:
        data = read_zip(VANILLA_189, "assets/minecraft/textures/gui/icons.png")
    if not data:
        print("  MISS icons.png")
        return False
    img = Image.open(io.BytesIO(data)).convert("RGBA")
    # ligero contraste global
    r, g, b, a = img.split()
    rgb = Image.merge("RGB", (r, g, b))
    rgb = ImageEnhance.Contrast(rgb).enhance(1.18)
    rgb = ImageEnhance.Sharpness(rgb).enhance(1.25)
    out = rgb.convert("RGBA")
    out.putalpha(a)

    # reforzar filas de corazones (top ~18 px band)
    def boost_band(y0: int, y1: int, factor: float = 1.2) -> None:
        band = out.crop((0, y0, out.width, y1))
        br, bg, bb, ba = band.split()
        brgb = ImageEnhance.Brightness(Image.merge("RGB", (br, bg, bb))).enhance(factor)
        brgb = ImageEnhance.Contrast(brgb).enhance(1.1)
        band2 = brgb.convert("RGBA")
        band2.putalpha(ba)
        out.paste(band2, (0, y0))

    boost_band(0, 18, 1.15)   # hearts
    boost_band(27, 45, 1.12)  # hunger region typical
    boost_band(9, 18, 1.1)    # armor icons overlap

    dest = OVERLAY_189 / "assets/minecraft/textures/gui/icons.png"
    save_png(dest, out)
    print(f"  icons189 {out.size}")

    # widgets hotbar
    wdata = read_zip(DEWIER, "assets/minecraft/textures/gui/widgets.png")
    if wdata:
        w = Image.open(io.BytesIO(wdata)).convert("RGBA")
        wr, wg, wb, wa = w.split()
        wrgb = ImageEnhance.Contrast(Image.merge("RGB", (wr, wg, wb))).enhance(1.1)
        w2 = wrgb.convert("RGBA")
        w2.putalpha(wa)
        save_png(OVERLAY_189 / "assets/minecraft/textures/gui/widgets.png", w2)
        print(f"  widgets189 {w2.size}")
    return True


def thin_font_ascii(src: Image.Image) -> Image.Image:
    """
    Reduce grasa de glifos: erode ligero de opacidad alta en ascii sheet.
    Conserva negrita útil pero limpia smear.
    """
    img = src.convert("RGBA")
    # sube contraste y un poco sharpen; no dilatar
    r, g, b, a = img.split()
    rgb = Image.merge("RGB", (r, g, b))
    rgb = ImageEnhance.Contrast(rgb).enhance(1.2)
    rgb = ImageEnhance.Sharpness(rgb).enhance(1.35)
    # erode alpha sobre glifos muy blancos para thinning sutil
    pixels = img.load()
    out = Image.new("RGBA", img.size, (0, 0, 0, 0))
    op = out.load()
    w, h = img.size
    for y in range(h):
        for x in range(w):
            pr, pg, pb, pa = pixels[x, y]
            if pa < 8:
                continue
            # si es borde (vecino transparente), bajar alpha un poco
            border = False
            for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < w and 0 <= ny < h:
                    if pixels[nx, ny][3] < 8:
                        border = True
                        break
                else:
                    border = True
            na = pa
            if border and max(pr, pg, pb) > 180:
                na = max(0, int(pa * 0.55))
            # interior: ligera recorte de alpha muy alto para "thin"
            elif max(pr, pg, pb) > 230 and pa > 200:
                na = min(pa, 230)
            op[x, y] = (pr, pg, pb, na)
    # re-aplicar contraste RGB sobre resultado
    r2, g2, b2, a2 = out.split()
    rgb2 = ImageEnhance.Contrast(Image.merge("RGB", (r2, g2, b2))).enhance(1.05)
    final = rgb2.convert("RGBA")
    final.putalpha(a2)
    return final


def font_189() -> int:
    n = 0
    for name in ("ascii.png", "ascii_sga.png"):
        data = read_zip(DEWIER, f"assets/minecraft/textures/font/{name}")
        if not data:
            continue
        im = thin_font_ascii(Image.open(io.BytesIO(data)))
        save_png(OVERLAY_189 / f"assets/minecraft/textures/font/{name}", im)
        # también mcpatcher path (OptiFine HD font)
        save_png(OVERLAY_189 / f"assets/minecraft/mcpatcher/font/{name}", im)
        print(f"  font189 {name} {im.size}")
        n += 1
        # properties
        for base in (
            f"assets/minecraft/textures/font/{name.replace('.png', '.properties')}",
            f"assets/minecraft/mcpatcher/font/{name.replace('.png', '.properties')}",
        ):
            pdata = read_zip(DEWIER, base)
            if pdata:
                dest = OVERLAY_189 / base
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_bytes(pdata)
    return n


def modern_hud() -> int:
    n = 0
    root = NINEBLUE / "assets/minecraft/textures/gui/sprites/hud"
    if not root.is_dir():
        print("  MISS 9blue hud dir")
        return 0
    for src in root.rglob("*.png"):
        rel = src.relative_to(NINEBLUE)
        dest = OVERLAY_MODERN / rel
        im = Image.open(src).convert("RGBA")
        # contrast boost for hearts/food/crosshair
        r, g, b, a = im.split()
        rgb = ImageEnhance.Contrast(Image.merge("RGB", (r, g, b))).enhance(1.15)
        rgb = ImageEnhance.Brightness(rgb).enhance(1.05)
        out = rgb.convert("RGBA")
        out.putalpha(a)
        save_png(dest, out)
        n += 1
    print(f"  modern hud sprites {n}")

    # particle heart
    ph = NINEBLUE / "assets/minecraft/textures/particle/heart.png"
    if ph.is_file():
        im = Image.open(ph).convert("RGBA")
        r, g, b, a = im.split()
        rgb = ImageEnhance.Contrast(Image.merge("RGB", (r, g, b))).enhance(1.2)
        out = rgb.convert("RGBA")
        out.putalpha(a)
        save_png(OVERLAY_MODERN / "assets/minecraft/textures/particle/heart.png", out)
        n += 1
        print("  particle heart.png")
    return n


def modern_font() -> int:
    """1.21: textures/font/ascii.png si existe en 9blue o vanilla-like; si no skip."""
    candidates = [
        NINEBLUE / "assets/minecraft/textures/font/ascii.png",
        NINEBLUE / "assets/minecraft/font/ascii.png",
    ]
    for c in candidates:
        if c.is_file():
            im = thin_font_ascii(Image.open(c))
            save_png(OVERLAY_MODERN / "assets/minecraft/textures/font/ascii.png", im)
            print(f"  font modern ascii {im.size}")
            return 1
    # fallback: dewier ascii ported (works for default font provider in some packs)
    data = read_zip(DEWIER, "assets/minecraft/textures/font/ascii.png")
    if data:
        im = thin_font_ascii(Image.open(io.BytesIO(data)))
        save_png(OVERLAY_MODERN / "assets/minecraft/textures/font/ascii.png", im)
        print(f"  font modern from dewier ascii {im.size}")
        return 1
    print("  skip modern font (no source)")
    return 0


def main() -> int:
    print("=== Wave 3: HUD hearts + font ===")
    print(f"Dewier exists={DEWIER.is_file()} 9blue={NINEBLUE.is_dir()}")
    enhance_icons_189()
    f1 = font_189()
    h = modern_hud()
    f2 = modern_font()
    print(f"Done font189={f1} modernHud={h} modernFont={f2}")
    print("Next: patch_189 + patch_modern")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
