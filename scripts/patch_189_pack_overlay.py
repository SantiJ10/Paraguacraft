#!/usr/bin/env python3
"""Parchea el pack 1.8.9 existente con overlay-189 (sin re-fusionar dewier)."""

from __future__ import annotations

import hashlib
import json
import shutil
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OVERLAY = ROOT / "resourcepacks-src" / "overlay-189"
PACK_LOGO = ROOT / "resourcepacks-src" / "pack.png"
OUT = ROOT / "clientes" / "paraguacraft-pvp" / "packs" / "paraguacraft-pvp-189.zip"
BUNDLED = ROOT / "bundled" / "pvp" / "resourcepacks" / "paraguacraft-pvp-189.zip"

DROP_EXACT = {
    "assets/minecraft/textures/blocks/fire_layer_0 - kopie.png",
    "assets/minecraft/textures/blocks/fire_layer_1 - kopie.png",
    "assets/minecraft/textures/blocks/fire_layer_0old.png",
    "assets/minecraft/textures/blocks/fire_layer_1old.png",
}


def sha1_file(path: Path) -> str:
    h = hashlib.sha1()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def load_zip(path: Path) -> dict[str, bytes]:
    out: dict[str, bytes] = {}
    with zipfile.ZipFile(path, "r") as zin:
        for info in zin.infolist():
            if info.is_dir():
                continue
            name = info.filename.replace("\\", "/")
            out[name] = zin.read(info.filename)
    return out


def apply_overlay(entries: dict[str, bytes]) -> None:
    if PACK_LOGO.is_file():
        entries["pack.png"] = PACK_LOGO.read_bytes()
    for path in OVERLAY.rglob("*"):
        if not path.is_file():
            continue
        rel = path.relative_to(OVERLAY).as_posix()
        entries[rel] = path.read_bytes()


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
    base = "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/main/clientes/paraguacraft-pvp/packs"
    fb = f"https://cdn.jsdelivr.net/gh/SantiJ10/Paraguacraft@main/clientes/paraguacraft-pvp/packs/{file_name}"
    paths = [
        ROOT / "clientes" / "paraguacraft-pvp" / "packs" / "catalog.json",
        ROOT / "client" / "src" / "main" / "resources" / "assets" / "paraguacraft" / "packs" / "catalog.json",
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
                "subtitle": "DewierxBlue · BW/SW blocks · fire · ores · GUI",
                "badge": "16x",
                "fileName": file_name,
                "sha1": sha,
                "fallbackDownloadUrl": fb,
            },
        )
        data["packs"] = packs
        catalog_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def patch_launcher_sha(sha: str) -> None:
    rs = ROOT / "launcher" / "src-tauri" / "src" / "core" / "pvp_packs.rs"
    if not rs.is_file():
        return
    text = rs.read_text(encoding="utf-8")
    import re

    new = re.sub(
        r'pub const PACK_189_SHA1: &str = "[0-9a-fA-F]+";',
        f'pub const PACK_189_SHA1: &str = "{sha}";',
        text,
        count=1,
    )
    if new != text:
        rs.write_text(new, encoding="utf-8")
        print(f"updated PACK_189_SHA1 -> {sha}")


def main() -> int:
    if not OUT.is_file():
        raise SystemExit(f"No existe pack base: {OUT}")
    if not OVERLAY.is_dir():
        raise SystemExit(f"No existe overlay: {OVERLAY}")

    entries = load_zip(OUT)
    for drop in DROP_EXACT:
        entries.pop(drop, None)
    apply_overlay(entries)
    write_zip(entries, OUT)
    sha = sha1_file(OUT)

    BUNDLED.parent.mkdir(parents=True, exist_ok=True)
    try:
        shutil.copy2(OUT, BUNDLED)
    except OSError as e:
        print(f"WARN bundled copy: {e}")
    update_catalogs(sha, OUT.name)
    patch_launcher_sha(sha)
    print(f"OK {OUT.name} sha1={sha} size={OUT.stat().st_size // 1024}KB")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
