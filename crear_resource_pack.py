"""
Genera el resource pack 'Paraguacraft' que reemplaza el logo del título
de Minecraft con el logo de Paraguacraft.

Uso:  python crear_resource_pack.py
Salida: Paraguacraft_ResourcePack.zip  (en la misma carpeta)
"""

import json
import shutil
import zipfile
from pathlib import Path

try:
    from PIL import Image
    PIL_OK = True
except ImportError:
    PIL_OK = False

# ── Rutas ──────────────────────────────────────────────────────────────────
BASE_DIR  = Path(__file__).parent
LOGO_SRC  = BASE_DIR / "web" / "assets" / "paraguacraft_logo.png"
OUT_DIR   = BASE_DIR / "_rp_build"
OUT_ZIP   = BASE_DIR / "Paraguacraft_ResourcePack.zip"

# Ruta dentro del resource pack donde va el logo del título
TITLE_TEX = OUT_DIR / "assets" / "minecraft" / "textures" / "gui" / "title"

# ── pack.mcmeta ─────────────────────────────────────────────────────────────
# pack_format por versión:
#   15 → 1.20 / 1.20.1
#   22 → 1.20.4
#   32 → 1.21
#   34 → 1.21.1 / 1.21.4
PACK_META = {
    "pack": {
        "pack_format": 34,
        "supported_formats": [15, 34],
        "description": "§aTítulo personalizado de §bParaguacraft"
    }
}


def build():
    # Limpia carpeta de build
    if OUT_DIR.exists():
        shutil.rmtree(OUT_DIR)
    TITLE_TEX.mkdir(parents=True)

    # ── 1. Crea pack.mcmeta ──────────────────────────────────────────────
    (OUT_DIR / "pack.mcmeta").write_text(
        json.dumps(PACK_META, indent=4, ensure_ascii=False), encoding="utf-8")

    # ── 2. pack.png (ícono del pack en la lista de resource packs) ──────
    shutil.copy(LOGO_SRC, OUT_DIR / "pack.png")

    # ── 3. Textura del título: assets/.../title/minecraft.png ───────────
    if PIL_OK:
        img = Image.open(LOGO_SRC).convert("RGBA")

        # Minecraft renderiza el título a ~256×64 px de referencia.
        # Escalamos manteniendo proporción para que entre bien.
        TARGET_W = 256
        ratio = TARGET_W / img.width
        TARGET_H = int(img.height * ratio)
        # Aseguramos altura mínima de 44px para que el juego lo acepte bien
        if TARGET_H < 44:
            TARGET_H = 44

        img = img.resize((TARGET_W, TARGET_H), Image.LANCZOS)
        img.save(TITLE_TEX / "minecraft.png", format="PNG")
        print(f"  Logo redimensionado a {TARGET_W}×{TARGET_H}px")
    else:
        # Sin Pillow: copia el logo tal cual (funciona en la mayoría de versiones)
        shutil.copy(LOGO_SRC, TITLE_TEX / "minecraft.png")
        print("  [!] Pillow no instalado — logo copiado sin redimensionar.")
        print("      Instalá Pillow con:  pip install Pillow")

    # ── 4. Empaqueta en .zip ─────────────────────────────────────────────
    if OUT_ZIP.exists():
        OUT_ZIP.unlink()

    with zipfile.ZipFile(OUT_ZIP, "w", zipfile.ZIP_DEFLATED) as zf:
        for f in OUT_DIR.rglob("*"):
            if f.is_file():
                zf.write(f, f.relative_to(OUT_DIR))

    # Limpia carpeta temporal
    shutil.rmtree(OUT_DIR)

    print(f"\n✅ Resource pack creado: {OUT_ZIP.name}")
    print("\n📦 Cómo usarlo:")
    print("  1. Copiá 'Paraguacraft_ResourcePack.zip' a:")
    print("     .minecraft\\resourcepacks\\")
    print("  2. Abrí Minecraft → Opciones → Resource Packs → activalo")
    print("  3. El título del juego mostrará PARAGUACRAFT en lugar de MINECRAFT")


if __name__ == "__main__":
    build()
