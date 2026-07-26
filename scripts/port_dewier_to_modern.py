#!/usr/bin/env python3
"""
Porta Dewiers-20k (1.8.9) → pack Paraguacraft PvP Modern (1.21.11).

- Renombra textures/blocks|items → block|item + nombres 1.13+
- Incluye destroy_stage (efecto romper bloques)
- NO mete icons/widgets 1.8 (rompen HUD 1.21); el crosshair queda el del overlay
  (sprites/hud/crosshair.png) para que los mods puedan modificarlo
- Encima aplica overlay-modern (totem/arco/ballesta/hotbar/beds/etc.)
"""

from __future__ import annotations

import hashlib
import json
import re
import shutil
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OVERLAY = ROOT / "resourcepacks-src" / "overlay-modern"
PACK_LOGO = ROOT / "resourcepacks-src" / "pack.png"
OUT = ROOT / "clientes" / "paraguacraft-pvp-modern" / "packs" / "paraguacraft-pvp-modern.zip"

DEWIER_DEFAULT = Path(
    r"D:\Amin\instalador programas\1.21.11 texture pvp\Dewiers-20k-Resource-Pack-16x-1.8.9.zip"
)

# Cuidado: no usar "old" suelto (matchea dentro de "gold_*").
SKIP_NAME = re.compile(
    r"(kopie|(?:^|/)[^/]*old\.png|\.mcmeta\.txt|__MACOSX|\.DS_Store|Thumbs\.db|"
    r"fire_layer_[01]old\.png|cobblestone_mossyOLD\.png)",
    re.I,
)

# Nombres de archivo 1.8 → 1.21 (solo basename; se aplica a .png y .png.mcmeta)
RENAME_FILE = {
    # wool
    "wool_colored_white": "white_wool",
    "wool_colored_orange": "orange_wool",
    "wool_colored_magenta": "magenta_wool",
    "wool_colored_light_blue": "light_blue_wool",
    "wool_colored_yellow": "yellow_wool",
    "wool_colored_lime": "lime_wool",
    "wool_colored_pink": "pink_wool",
    "wool_colored_gray": "gray_wool",
    "wool_colored_silver": "light_gray_wool",
    "wool_colored_cyan": "cyan_wool",
    "wool_colored_purple": "purple_wool",
    "wool_colored_blue": "blue_wool",
    "wool_colored_brown": "brown_wool",
    "wool_colored_green": "green_wool",
    "wool_colored_red": "red_wool",
    "wool_colored_black": "black_wool",
    # wood
    "planks_oak": "oak_planks",
    "planks_spruce": "spruce_planks",
    "planks_birch": "birch_planks",
    "planks_jungle": "jungle_planks",
    "planks_acacia": "acacia_planks",
    "planks_big_oak": "dark_oak_planks",
    "log_oak": "oak_log",
    "log_oak_top": "oak_log_top",
    "log_spruce": "spruce_log",
    "log_spruce_top": "spruce_log_top",
    "log_birch": "birch_log",
    "log_birch_top": "birch_log_top",
    "log_jungle": "jungle_log",
    "log_jungle_top": "jungle_log_top",
    "log_acacia": "acacia_log",
    "log_acacia_top": "acacia_log_top",
    "log_big_oak": "dark_oak_log",
    "log_big_oak_top": "dark_oak_log_top",
    # common bedwars / pvp
    "web": "cobweb",
    "fire_layer_0": "fire_0",
    "fire_layer_1": "fire_1",
    "stonebrick": "stone_bricks",
    "stonebrick_mossy": "mossy_stone_bricks",
    "stonebrick_cracked": "cracked_stone_bricks",
    "stonebrick_carved": "chiseled_stone_bricks",
    "cobblestone_mossy": "mossy_cobblestone",
    "end_stone": "end_stone",
    "slime": "slime_block",
    "reeds": "sugar_cane",
    "water": "water_still",
    "water_flow": "water_flow",
    "lava": "lava_still",
    "lava_flow": "lava_flow",
    "glass_black": "black_stained_glass",
    "glass_blue": "blue_stained_glass",
    "glass_brown": "brown_stained_glass",
    "glass_cyan": "cyan_stained_glass",
    "glass_gray": "gray_stained_glass",
    "glass_green": "green_stained_glass",
    "glass_light_blue": "light_blue_stained_glass",
    "glass_lime": "lime_stained_glass",
    "glass_magenta": "magenta_stained_glass",
    "glass_orange": "orange_stained_glass",
    "glass_pink": "pink_stained_glass",
    "glass_purple": "purple_stained_glass",
    "glass_red": "red_stained_glass",
    "glass_silver": "light_gray_stained_glass",
    "glass_white": "white_stained_glass",
    "glass_yellow": "yellow_stained_glass",
    "hardened_clay": "terracotta",
    "clay_hardened": "terracotta",
    "clay_stained_black": "black_terracotta",
    "clay_stained_blue": "blue_terracotta",
    "clay_stained_brown": "brown_terracotta",
    "clay_stained_cyan": "cyan_terracotta",
    "clay_stained_gray": "gray_terracotta",
    "clay_stained_green": "green_terracotta",
    "clay_stained_light_blue": "light_blue_terracotta",
    "clay_stained_lime": "lime_terracotta",
    "clay_stained_magenta": "magenta_terracotta",
    "clay_stained_orange": "orange_terracotta",
    "clay_stained_pink": "pink_terracotta",
    "clay_stained_purple": "purple_terracotta",
    "clay_stained_red": "red_terracotta",
    "clay_stained_silver": "light_gray_terracotta",
    "clay_stained_white": "white_terracotta",
    "clay_stained_yellow": "yellow_terracotta",
    # items
    "wood_sword": "wooden_sword",
    "wood_pickaxe": "wooden_pickaxe",
    "wood_axe": "wooden_axe",
    "wood_shovel": "wooden_shovel",
    "wood_hoe": "wooden_hoe",
    "fishing_rod_uncast": "fishing_rod",
    "fishing_rod_cast": "fishing_rod_cast",
    "bow_standby": "bow",
    "door_wood": "oak_door",
    "door_iron": "iron_door",
    "minecart_normal": "minecart",
    "minecart_chest": "chest_minecart",
    "minecart_tnt": "tnt_minecart",
    "minecart_furnace": "furnace_minecart",
    "minecart_hopper": "hopper_minecart",
    "minecart_command_block": "command_block_minecart",
    "book_normal": "book",
    "book_writable": "writable_book",
    "book_written": "written_book",
    "book_enchanted": "enchanted_book",
    "potion_bottle_drinkable": "potion",
    "potion_bottle_splash": "splash_potion",
    "potion_bottle_linger": "lingering_potion",
    "fireball": "fire_charge",
    # armadura ítems (1.8 gold_* → 1.21 golden_*)
    "gold_helmet": "golden_helmet",
    "gold_chestplate": "golden_chestplate",
    "gold_leggings": "golden_leggings",
    "gold_boots": "golden_boots",
    "gold_sword": "golden_sword",
    "gold_pickaxe": "golden_pickaxe",
    "gold_axe": "golden_axe",
    "gold_shovel": "golden_shovel",
    "gold_hoe": "golden_hoe",
    "gold_horse_armor": "golden_horse_armor",
    "apple_golden": "golden_apple",
    "carrot_golden": "golden_carrot",
}

# Armadura vestida 1.8 models/armor → 1.21 entity/equipment
# layer_1 = casco/pecho/botas (humanoid), layer_2 = pantalones (humanoid_leggings)
ARMOR_LAYER_MAP = {
    "leather_layer_1.png": "assets/minecraft/textures/entity/equipment/humanoid/leather.png",
    "leather_layer_1_overlay.png": "assets/minecraft/textures/entity/equipment/humanoid/leather_overlay.png",
    "leather_layer_2.png": "assets/minecraft/textures/entity/equipment/humanoid_leggings/leather.png",
    "leather_layer_2_overlay.png": "assets/minecraft/textures/entity/equipment/humanoid_leggings/leather_overlay.png",
    "chainmail_layer_1.png": "assets/minecraft/textures/entity/equipment/humanoid/chainmail.png",
    "chainmail_layer_2.png": "assets/minecraft/textures/entity/equipment/humanoid_leggings/chainmail.png",
    "iron_layer_1.png": "assets/minecraft/textures/entity/equipment/humanoid/iron.png",
    "iron_layer_2.png": "assets/minecraft/textures/entity/equipment/humanoid_leggings/iron.png",
    "gold_layer_1.png": "assets/minecraft/textures/entity/equipment/humanoid/gold.png",
    "gold_layer_2.png": "assets/minecraft/textures/entity/equipment/humanoid_leggings/gold.png",
    "diamond_layer_1.png": "assets/minecraft/textures/entity/equipment/humanoid/diamond.png",
    "diamond_layer_2.png": "assets/minecraft/textures/entity/equipment/humanoid_leggings/diamond.png",
}

# No portar HUD 1.8 (atlas viejo); 1.21 usa sprites y queremos crosshair del overlay/mods
SKIP_PREFIXES = (
    "assets/minecraft/textures/gui/icons.png",
    "assets/minecraft/textures/gui/widgets.png",
    "assets/minecraft/mcpatcher/font/",
    "assets/minecraft/lang/",
)

# destroy_stage_N_1.png son variantes basura en Dewiers
SKIP_DESTROY_ALT = re.compile(r"destroy_stage_\d+_1\.png", re.I)


def sha1_file(path: Path) -> str:
    h = hashlib.sha1()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def rename_basename(name: str) -> str | None:
    """name = 'foo.png' o 'foo.png.mcmeta'."""
    if SKIP_DESTROY_ALT.search(name):
        return None
    if name.endswith(".png.mcmeta"):
        stem, suf = name[: -len(".png.mcmeta")], ".png.mcmeta"
    elif name.endswith(".png"):
        stem, suf = name[: -len(".png")], ".png"
    else:
        return name
    stem2 = RENAME_FILE.get(stem, stem)
    return stem2 + suf


def map_dewier_path(src: str) -> str | None:
    p = src.replace("\\", "/")
    if SKIP_NAME.search(p):
        return None
    for pref in SKIP_PREFIXES:
        if p == pref or p.startswith(pref):
            return None

    if p.startswith("assets/minecraft/textures/blocks/"):
        rest = p.split("textures/blocks/", 1)[1]
        renamed = rename_basename(rest)
        if renamed is None:
            return None
        return f"assets/minecraft/textures/block/{renamed}"

    if p.startswith("assets/minecraft/textures/items/"):
        rest = p.split("textures/items/", 1)[1]
        # Espadas/fireball: se usan las de 9blue/overlay (las de Dewiers no gustaron).
        base = rest.split(".png", 1)[0]
        if "sword" in base or base in {"fireball", "fire_charge"}:
            return None
        renamed = rename_basename(rest)
        if renamed is None:
            return None
        return f"assets/minecraft/textures/item/{renamed}"

    if p.startswith("assets/minecraft/textures/models/armor/"):
        name = p.rsplit("/", 1)[-1]
        return ARMOR_LAYER_MAP.get(name)

    if p.startswith("assets/minecraft/textures/entity/"):
        return p  # muchas siguen válidas; beds viejos se pisan con overlay

    if p.startswith("assets/minecraft/textures/particle/"):
        return p

    if p.startswith("assets/minecraft/textures/misc/"):
        return p

    if p.startswith("assets/minecraft/mcpatcher/sky/"):
        rest = p.split("mcpatcher/sky/", 1)[1]
        return f"assets/minecraft/optifine/sky/{rest}"

    # GUI containers/etc. 1.8 suelen romper; solo dejamos map icons si existiera utilidad
    return None


def load_dewier(path: Path) -> dict[str, bytes]:
    out: dict[str, bytes] = {}
    skipped = 0
    with zipfile.ZipFile(path, "r") as zin:
        for info in zin.infolist():
            if info.is_dir():
                continue
            src = info.filename.replace("\\", "/")
            dst = map_dewier_path(src)
            if not dst:
                skipped += 1
                continue
            out[dst] = zin.read(info.filename)
    print(f"  dewier mapped={len(out)} skipped={skipped}")
    return out


def apply_overlay(entries: dict[str, bytes]) -> None:
    if PACK_LOGO.is_file():
        entries["pack.png"] = PACK_LOGO.read_bytes()
    for path in OVERLAY.rglob("*"):
        if path.is_file():
            rel = path.relative_to(OVERLAY).as_posix()
            # Crosshair lo controlan los mods del cliente; no forzar textura del pack.
            if rel.endswith("gui/sprites/hud/crosshair.png"):
                continue
            entries[rel] = path.read_bytes()
    entries.pop("assets/minecraft/textures/gui/sprites/hud/crosshair.png", None)


def write_zip(entries: dict[str, bytes], out: Path) -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    tmp = out.with_suffix(".part")
    with zipfile.ZipFile(tmp, "w", compression=zipfile.ZIP_DEFLATED) as zout:
        for name in sorted(entries.keys()):
            zout.writestr(name, entries[name])
    if out.exists():
        out.unlink()
    tmp.rename(out)


def update_catalogs(sha: str, file_name: str) -> None:
    base = "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/main/clientes/paraguacraft-pvp-modern/packs"
    fb = f"https://cdn.jsdelivr.net/gh/SantiJ10/Paraguacraft@main/clientes/paraguacraft-pvp-modern/packs/{file_name}"
    paths = [
        ROOT / "clientes" / "paraguacraft-pvp-modern" / "packs" / "catalog.json",
        ROOT / "client-modern" / "src" / "main" / "resources" / "assets" / "paraguacraftpvp-modern" / "packs" / "catalog.json",
        ROOT / "client-modern" / "src" / "main" / "resources" / "assets" / "paraguacraft-modern" / "packs" / "catalog.json",
        ROOT / "bundled" / "pvp-modern" / "packs" / "catalog.json",
        ROOT / "launcher" / "src-tauri" / "resources" / "bundled" / "pvp-modern" / "packs" / "catalog.json",
    ]
    for catalog_path in paths:
        if not catalog_path.is_file():
            continue
        data = json.loads(catalog_path.read_text(encoding="utf-8-sig"))
        data["releaseTag"] = "main"
        data["baseUrl"] = base
        packs = [p for p in data.get("packs", []) if p.get("id") != "paraguacraft-pvp"]
        packs.insert(
            0,
            {
                "id": "paraguacraft-pvp",
                "title": "Paraguacraft PvP",
                "subtitle": "Dewiers wool/armor + 9blue swords · 1.21.11",
                "badge": "16x",
                "fileName": file_name,
                "sha1": sha,
                "fallbackDownloadUrl": fb,
            },
        )
        data["packs"] = packs
        catalog_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def main() -> int:
    import sys

    dewier = Path(sys.argv[1]) if len(sys.argv) > 1 else DEWIER_DEFAULT
    if not dewier.is_file():
        raise SystemExit(f"No existe Dewiers: {dewier}")
    if not OVERLAY.is_dir():
        raise SystemExit(f"No existe overlay: {OVERLAY}")

    print(f"Portando {dewier.name} -> modern...")
    dewier_entries = load_dewier(dewier)
    entries = dict(dewier_entries)
    apply_overlay(entries)

    # Overlay no debe pisar look Dewiers (lana, destroy, armaduras).
    keep_re = re.compile(
        r"^assets/minecraft/textures/("
        r"block/((.+_wool)|destroy_stage_\d+)\.png(\.mcmeta)?|"
        r"entity/equipment/(humanoid|humanoid_leggings)/"
        r"(leather|leather_overlay|chainmail|iron|gold|diamond)\.png|"
        r"item/(leather_|chainmail_|iron_|golden_|diamond_)"
        r"(helmet|chestplate|leggings|boots)(_overlay)?\.png"
        r")$"
    )
    restored = 0
    for k, v in dewier_entries.items():
        if keep_re.match(k):
            entries[k] = v
            restored += 1
    print(f"  restored dewier wool/destroy/armor={restored}")

    # Schema 1.21.11 (resource pack_format 75): min/max como arrays.
    entries["pack.mcmeta"] = json.dumps(
        {
            "pack": {
                "pack_format": 75,
                "description": "Paraguacraft PvP Modern 1.21.11",
                "min_format": [75, 0],
                "max_format": [99, 0],
            }
        },
        indent=2,
    ).encode("utf-8") + b"\n"

    write_zip(entries, OUT)
    sha = sha1_file(OUT)
    for dest in (
        ROOT / "bundled" / "pvp-modern" / "resourcepacks" / OUT.name,
        ROOT / "bundled" / "pvp-modern" / "packs" / OUT.name,
    ):
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(OUT, dest)
    update_catalogs(sha, OUT.name)

    # sanity
    must = [
        "assets/minecraft/textures/block/destroy_stage_0.png",
        "assets/minecraft/textures/block/blue_wool.png",
        "assets/minecraft/models/item/totem_of_undying.json",
        "assets/minecraft/textures/entity/equipment/humanoid/diamond.png",
        "assets/minecraft/textures/item/diamond_sword.png",
        "assets/minecraft/textures/item/fire_charge.png",
        "assets/minecraft/textures/item/golden_helmet.png",
    ]
    with zipfile.ZipFile(OUT, "r") as z:
        names = set(z.namelist())
    for m in must:
        print(("OK  " if m in names else "MISS") + m)
    ch = "assets/minecraft/textures/gui/sprites/hud/crosshair.png"
    print(("OK no crosshair (mods)" if ch not in names else "WARN crosshair still present") )
    print(f"OK {OUT.name} sha1={sha} size={OUT.stat().st_size // 1024}KB entries={len(entries)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
