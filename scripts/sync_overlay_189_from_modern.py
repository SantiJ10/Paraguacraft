#!/usr/bin/env python3
"""Copia bloques PvP del overlay-modern al overlay-189 con nombres 1.8.9."""

from __future__ import annotations

import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "resourcepacks-src" / "overlay-modern" / "assets" / "minecraft" / "textures" / "block"
DST = ROOT / "resourcepacks-src" / "overlay-189" / "assets" / "minecraft" / "textures" / "blocks"


def main() -> int:
    DST.mkdir(parents=True, exist_ok=True)

    mapping: dict[str, str] = {
        "glass.png": "glass.png",
        "glass_pane_top.png": "glass_pane_top.png",
        "obsidian.png": "obsidian.png",
        "ladder.png": "ladder.png",
        "cobweb.png": "web.png",
        "tnt_side.png": "tnt_side.png",
        "tnt_top.png": "tnt_top.png",
        "tnt_bottom.png": "tnt_bottom.png",
        "oak_planks.png": "planks_oak.png",
        "spruce_planks.png": "planks_spruce.png",
        "birch_planks.png": "planks_birch.png",
        "jungle_planks.png": "planks_jungle.png",
        "acacia_planks.png": "planks_acacia.png",
        "dark_oak_planks.png": "planks_big_oak.png",
        "end_stone.png": "end_stone.png",
        "end_stone_bricks.png": "end_bricks.png",
        "slime_block.png": "slime.png",
        "ice.png": "ice.png",
        "packed_ice.png": "ice_packed.png",
        "water_still.png": "water_still.png",
        "water_flow.png": "water_flow.png",
    }

    colors = [
        ("white", "white"),
        ("orange", "orange"),
        ("magenta", "magenta"),
        ("light_blue", "light_blue"),
        ("yellow", "yellow"),
        ("lime", "lime"),
        ("pink", "pink"),
        ("gray", "gray"),
        ("light_gray", "silver"),
        ("cyan", "cyan"),
        ("purple", "purple"),
        ("blue", "blue"),
        ("brown", "brown"),
        ("green", "green"),
        ("red", "red"),
        ("black", "black"),
    ]
    for modern_c, leg_c in colors:
        mapping[f"{modern_c}_stained_glass.png"] = f"glass_{leg_c}.png"
        mapping[f"{modern_c}_stained_glass_pane_top.png"] = f"glass_pane_top_{leg_c}.png"
        mapping[f"{modern_c}_wool.png"] = f"wool_colored_{leg_c}.png"

    copied = 0
    missing: list[str] = []
    for src_name, dst_name in sorted(mapping.items()):
        src = SRC / src_name
        if not src.is_file():
            missing.append(src_name)
            continue
        shutil.copy2(src, DST / dst_name)
        copied += 1
        print(f"  {src_name} -> {dst_name}")

    print(f"copied {copied}")
    if missing:
        print("missing sources:")
        for m in missing:
            print(f"  {m}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
