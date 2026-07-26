#!/usr/bin/env python3
"""Copia assets PvP/SkyWars/BedWars/Pillars al overlay-modern desde tom1xi + 9blue + vanilla."""

from __future__ import annotations

import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OVERLAY = ROOT / "resourcepacks-src" / "overlay-modern"

TOM = Path(r"D:\Amin\instalador programas\1.21.11 texture pvp\tom1xi remade.zip")
NINE = Path(r"D:\Amin\instalador programas\1.21.11 texture pvp\9Blue1fault 8f16x80.zip")
VANILLA = Path.home() / "AppData/Roaming/.minecraft/versions/1.21.11/1.21.11.jar"

# 9blue usa nombres 1.8; renombrar a 1.21
RENAME_9BLUE = {
    "assets/minecraft/textures/block/planks_oak.png": "assets/minecraft/textures/block/oak_planks.png",
    "assets/minecraft/textures/block/planks_spruce.png": "assets/minecraft/textures/block/spruce_planks.png",
    "assets/minecraft/textures/block/planks_birch.png": "assets/minecraft/textures/block/birch_planks.png",
    "assets/minecraft/textures/block/planks_jungle.png": "assets/minecraft/textures/block/jungle_planks.png",
    "assets/minecraft/textures/block/planks_acacia.png": "assets/minecraft/textures/block/acacia_planks.png",
    "assets/minecraft/textures/block/planks_big_oak.png": "assets/minecraft/textures/block/dark_oak_planks.png",
    "assets/minecraft/textures/block/log_oak.png": "assets/minecraft/textures/block/oak_log.png",
    "assets/minecraft/textures/block/log_oak_top.png": "assets/minecraft/textures/block/oak_log_top.png",
    "assets/minecraft/textures/block/log_spruce.png": "assets/minecraft/textures/block/spruce_log.png",
    "assets/minecraft/textures/block/log_spruce_top.png": "assets/minecraft/textures/block/spruce_log_top.png",
    "assets/minecraft/textures/block/log_birch.png": "assets/minecraft/textures/block/birch_log.png",
    "assets/minecraft/textures/block/log_birch_top.png": "assets/minecraft/textures/block/birch_log_top.png",
    "assets/minecraft/textures/block/log_jungle.png": "assets/minecraft/textures/block/jungle_log.png",
    "assets/minecraft/textures/block/log_jungle_top.png": "assets/minecraft/textures/block/jungle_log_top.png",
    "assets/minecraft/textures/block/log_acacia.png": "assets/minecraft/textures/block/acacia_log.png",
    "assets/minecraft/textures/block/log_acacia_top.png": "assets/minecraft/textures/block/acacia_log_top.png",
    "assets/minecraft/textures/block/log_big_oak.png": "assets/minecraft/textures/block/dark_oak_log.png",
    "assets/minecraft/textures/block/log_big_oak_top.png": "assets/minecraft/textures/block/dark_oak_log_top.png",
}

WOOLS = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
]

BOW_MODEL = {
    "parent": "minecraft:item/generated",
    "textures": {"layer0": "minecraft:item/bow"},
    "display": {
        "thirdperson_righthand": {
            "rotation": [-80, 260, -40],
            "translation": [-1, -2, 2.5],
            "scale": [0.72, 0.72, 0.72],
        },
        "thirdperson_lefthand": {
            "rotation": [-80, -280, 40],
            "translation": [-1, -2, 2.5],
            "scale": [0.72, 0.72, 0.72],
        },
        "firstperson_righthand": {
            "rotation": [0, -90, 25],
            "translation": [1.13, 3.2, 1.13],
            "scale": [0.52, 0.52, 0.52],
        },
        "firstperson_lefthand": {
            "rotation": [0, 90, -25],
            "translation": [1.13, 3.2, 1.13],
            "scale": [0.52, 0.52, 0.52],
        },
        "gui": {"scale": [0.9, 0.9, 0.9]},
        "ground": {"translation": [0, 2, 0], "scale": [0.45, 0.45, 0.45]},
        "fixed": {"scale": [0.55, 0.55, 0.55]},
    },
}

ROD_DISPLAY = {
    "thirdperson_righthand": {
        "rotation": [0, 90, 55],
        "translation": [0, 4.0, 2.5],
        "scale": [0.68, 0.68, 0.68],
    },
    "thirdperson_lefthand": {
        "rotation": [0, -90, -55],
        "translation": [0, 4.0, 2.5],
        "scale": [0.68, 0.68, 0.68],
    },
    "firstperson_righthand": {
        "rotation": [0, 90, 25],
        "translation": [0, 1.6, 0.8],
        "scale": [0.48, 0.48, 0.48],
    },
    "firstperson_lefthand": {
        "rotation": [0, -90, -25],
        "translation": [0, 1.6, 0.8],
        "scale": [0.48, 0.48, 0.48],
    },
    "gui": {"scale": [0.9, 0.9, 0.9]},
}


def write_bytes(rel: str, data: bytes) -> None:
    dest = OVERLAY / rel
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(data)


def write_json(rel: str, obj: object) -> None:
    write_bytes(rel, (json.dumps(obj, indent=2) + "\n").encode("utf-8"))


def copy_from_zip(zpath: Path, mapping: dict[str, str]) -> int:
    n = 0
    with zipfile.ZipFile(zpath, "r") as zin:
        names = {i.filename.replace("\\", "/") for i in zin.infolist() if not i.is_dir()}
        for src, dst in mapping.items():
            if src not in names:
                print(f"  skip missing {src}")
                continue
            write_bytes(dst, zin.read(src))
            n += 1
    return n


def main() -> int:
    if not TOM.is_file():
        raise SystemExit(f"Missing tom1xi: {TOM}")
    if not NINE.is_file():
        raise SystemExit(f"Missing 9blue: {NINE}")
    if not VANILLA.is_file():
        raise SystemExit(f"Missing vanilla jar: {VANILLA}")

    tom_map: dict[str, str] = {}
    for color in WOOLS:
        p = f"assets/minecraft/textures/block/{color}_wool.png"
        tom_map[p] = p
    for name in ("cobweb.png", "cobweb_sides.png", "water_still.png", "slime_block.png"):
        p = f"assets/minecraft/textures/block/{name}"
        tom_map[p] = p
    for name in (
        "critical_hit.png",
        "enchanted_hit.png",
        "damage.png",
        *[f"sweep_{i}.png" for i in range(8)],
    ):
        p = f"assets/minecraft/textures/particle/{name}"
        tom_map[p] = p

    nine_map: dict[str, str] = {
        "assets/minecraft/textures/misc/enchanted_glint_item.png": "assets/minecraft/textures/misc/enchanted_glint_item.png",
        "assets/minecraft/textures/misc/enchanted_glint_armor.png": "assets/minecraft/textures/misc/enchanted_glint_armor.png",
        "assets/minecraft/textures/misc/enchanted_glint_entity.png": "assets/minecraft/textures/misc/enchanted_glint_entity.png",
        "assets/minecraft/textures/misc/enchanted_glint_item.png.mcmeta": "assets/minecraft/textures/misc/enchanted_glint_item.png.mcmeta",
        "assets/minecraft/textures/misc/enchanted_glint_armor.png.mcmeta": "assets/minecraft/textures/misc/enchanted_glint_armor.png.mcmeta",
        "assets/minecraft/textures/misc/enchanted_glint_entity.png.mcmeta": "assets/minecraft/textures/misc/enchanted_glint_entity.png.mcmeta",
        "assets/minecraft/textures/block/end_stone.png": "assets/minecraft/textures/block/end_stone.png",
        "assets/minecraft/textures/block/glass.png": "assets/minecraft/textures/block/glass.png",
        "assets/minecraft/textures/block/ladder.png": "assets/minecraft/textures/block/ladder.png",
        "assets/minecraft/textures/item/bow.png": "assets/minecraft/textures/item/bow.png",
        "assets/minecraft/textures/item/bow_pulling_0.png": "assets/minecraft/textures/item/bow_pulling_0.png",
        "assets/minecraft/textures/item/bow_pulling_1.png": "assets/minecraft/textures/item/bow_pulling_1.png",
        "assets/minecraft/textures/item/bow_pulling_2.png": "assets/minecraft/textures/item/bow_pulling_2.png",
        "assets/minecraft/textures/item/fishing_rod.png": "assets/minecraft/textures/item/fishing_rod.png",
        "assets/minecraft/textures/item/fishing_rod_cast.png": "assets/minecraft/textures/item/fishing_rod_cast.png",
        "assets/minecraft/textures/item/fire_charge.png": "assets/minecraft/textures/item/fire_charge.png",
        "assets/minecraft/textures/item/water_bucket.png": "assets/minecraft/textures/item/water_bucket.png",
        "assets/minecraft/textures/item/lava_bucket.png": "assets/minecraft/textures/item/lava_bucket.png",
        "assets/minecraft/textures/item/shears.png": "assets/minecraft/textures/item/shears.png",
        "assets/minecraft/textures/item/arrow.png": "assets/minecraft/textures/item/arrow.png",
        "assets/minecraft/textures/item/flint_and_steel.png": "assets/minecraft/textures/item/flint_and_steel.png",
        "assets/minecraft/textures/item/bed.png": "assets/minecraft/textures/item/bed.png",
    }
    nine_map.update(RENAME_9BLUE)

    print("tom1xi…")
    print(f"  copied {copy_from_zip(TOM, tom_map)}")
    print("9blue…")
    print(f"  copied {copy_from_zip(NINE, nine_map)}")

    with zipfile.ZipFile(VANILLA, "r") as zin:
        for rel in (
            "assets/minecraft/items/bow.json",
            "assets/minecraft/items/fishing_rod.json",
        ):
            write_bytes(rel, zin.read(rel))

    write_json("assets/minecraft/models/item/bow.json", BOW_MODEL)
    for i, tex in enumerate(("bow_pulling_0", "bow_pulling_1", "bow_pulling_2")):
        write_json(
            f"assets/minecraft/models/item/bow_pulling_{i}.json",
            {
                "parent": "minecraft:item/bow",
                "textures": {"layer0": f"minecraft:item/{tex}"},
            },
        )

    write_json(
        "assets/minecraft/models/item/fishing_rod.json",
        {
            "parent": "minecraft:item/handheld",
            "textures": {"layer0": "minecraft:item/fishing_rod"},
            "display": ROD_DISPLAY,
        },
    )
    write_json(
        "assets/minecraft/models/item/fishing_rod_cast.json",
        {
            "parent": "minecraft:item/handheld",
            "textures": {"layer0": "minecraft:item/fishing_rod_cast"},
            "display": ROD_DISPLAY,
        },
    )

    for item in (
        "shears",
        "flint_and_steel",
        "fire_charge",
        "water_bucket",
        "lava_bucket",
        "arrow",
    ):
        write_json(
            f"assets/minecraft/items/{item}.json",
            {"model": {"type": "minecraft:model", "model": f"minecraft:item/{item}"}},
        )
        write_json(
            f"assets/minecraft/models/item/{item}.json",
            {
                "parent": "minecraft:item/smaller_util",
                "textures": {"layer0": f"minecraft:item/{item}"},
            },
        )

    print(f"OK overlay -> {OVERLAY}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
