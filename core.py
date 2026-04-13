import minecraft_launcher_lib
import subprocess
import uuid
import os
import requests
import shutil
import platform
import json
import sys
import bsdiff4
try:
    from src.nube import GestorNube
except Exception:
    GestorNube = None

def inyectar_splash_paraguacraft(game_dir, version_base, motor_elegido, progress_callback=None):
    """
    Instala splash de carga personalizado de Paraguacraft según el loader.
    - Fabric: descarga 'custom-splash-screen' de Modrinth + config
    - Forge / NeoForge: escribe forge.cfg / neoforge.toml con colores Paraguacraft
    - Vanilla / OptiFine: no es posible (sin mod loader)
    """
    def _cb(msg):
        if progress_callback:
            try: progress_callback(msg)
            except Exception: pass

    motor_lower = motor_elegido.lower()
    _PC_GREEN  = "#2ECC71"
    _PC_BG     = "#0A0A0A"
    _PC_ACCENT = "#1A7A40"

    # ── Fabric: custom-splash-screen (Modrinth) ───────────────────────────
    # Versiones soportadas según Modrinth: 1.16.x, 1.17.x, 1.18.x,
    # 1.19/1.19.1/1.19.2/1.19.4, 1.20/1.20.1, 1.21/1.21.1/1.21.3
    def _splash_compatible(ver):
        _p = ver.strip().split(".")
        try:
            _maj = int(_p[0])
            _min = int(_p[1]) if len(_p) > 1 else 0
            _pat = int(_p[2]) if len(_p) > 2 else 0
        except (ValueError, IndexError):
            return False
        if _maj != 1:
            return False
        if _min in (16, 17, 18):
            return True
        if _min == 19:
            return _pat in (0, 1, 2, 4)
        if _min == 20:
            return _pat in (0, 1)
        if _min == 21:
            return _pat in (0, 1, 3)
        return False

    if "fabric" in motor_lower:
        mods_dir = os.path.join(game_dir, "mods")
        os.makedirs(mods_dir, exist_ok=True)
        existing = [f for f in os.listdir(mods_dir)
                    if "splash" in f.lower() and f.endswith(".jar")]
        if not existing and _splash_compatible(version_base):
            try:
                _h = {"User-Agent": "ParaguacraftLauncher/1.0"}
                _versions = []
                # 1st attempt: exact version
                _r = requests.get(
                    "https://api.modrinth.com/v2/project/custom-splash-screen/version",
                    params={"game_versions": f'["{version_base}"]',
                            "loaders": '["fabric"]'},
                    headers=_h, timeout=6)
                if _r.status_code == 200:
                    _versions = _r.json()
                # 2nd attempt: major.minor only (e.g. "26.1" for "26.1.2")
                if not _versions:
                    _parts = version_base.split(".")
                    _minor = ".".join(_parts[:2]) if len(_parts) >= 2 else version_base
                    _r2 = requests.get(
                        "https://api.modrinth.com/v2/project/custom-splash-screen/version",
                        params={"game_versions": f'["{_minor}"]',
                                "loaders": '["fabric"]'},
                        headers=_h, timeout=6)
                    if _r2.status_code == 200:
                        _versions = _r2.json()
                # Si no hay versión compatible con este MC, no instalar el mod
                if _versions:
                    _file = _versions[0]["files"][0]
                    _dest = os.path.join(mods_dir, _file["filename"])
                    if not os.path.exists(_dest):
                        _cb("[Splash] Instalando custom-splash-screen...")
                        _dl = requests.get(_file["url"], headers=_h, timeout=20)
                        if _dl.status_code == 200:
                            with open(_dest, "wb") as _f:
                                _f.write(_dl.content)
                            _cb("[Splash] custom-splash-screen instalado.")
            except Exception as _e:
                _cb(f"[Splash] No se pudo instalar: {_e}")
        # Configuración: fondo oscuro + acento verde Paraguacraft
        _cfg_dir    = os.path.join(game_dir, "config")
        _img_dir    = os.path.join(_cfg_dir, "customsplashscreen")
        _cfg_path   = os.path.join(_cfg_dir, "customsplashscreen.json")
        os.makedirs(_img_dir, exist_ok=True)
        # Copy Paraguacraft logos into the mod's image folder
        if getattr(sys, 'frozen', False):
            _base = sys._MEIPASS
        else:
            _base = os.path.dirname(os.path.abspath(__file__))
        _wide_src   = os.path.join(_base, "web", "assets", "LOGO_SPLASH.png")
        _icon_src   = os.path.join(_base, "web", "assets", "iconomc.png")
        for _src, _dst_name in [(_wide_src, "wide_logo.png"),
                                 (_icon_src, "square_logo.png")]:
            _dst = os.path.join(_img_dir, _dst_name)
            if os.path.exists(_src):
                try:
                    shutil.copyfile(_src, _dst)
                except Exception:
                    pass
        try:
            _cfg = {
                    "progressBarType": "Vanilla",
                    "logoStyle": "Mojang",
                    "backgroundImage": False,
                    "logoBlend": True,
                    "splashBackgroundColor": _PC_BG,
                    "splashProgressBarColor": _PC_GREEN,
                    "splashProgressFrameColor": _PC_ACCENT,
                    "splashProgressBackgroundColor": "#000000",
                    "progressBarBackground": False,
                    "customProgressBarMode": "Linear",
                    "spinningCircleSize": 2,
                    "spinningCircleSpeed": 4,
                    "spinningCircleTrail": 5,
                }
            with open(_cfg_path, "w") as _f:
                json.dump(_cfg, _f, indent=2)
        except Exception:
            pass

    # ── Forge / NeoForge: no se inyecta nada (Forge no soporta config de splash vía TOML) ──


def carpeta_instancia_paraguacraft(version_base, motor_elegido):
    """Mismo criterio en todo el launcher (logs, mods, RPC)."""
    nombre_limpio_motor = motor_elegido.replace(" ", "_").replace("+", "Plus")
    return f"Paraguacraft_{version_base}_{nombre_limpio_motor}"

def aplicar_delta_patch(ruta_archivo_viejo, ruta_parche, ruta_archivo_nuevo):
    """
    Motor Delta Patching: Funde un archivo viejo con un parche binario (.patch) 
    para construir la nueva versión del archivo sin descargar todo de cero.
    """
    try:
        # Reconstruye el archivo byte a byte
        bsdiff4.file_patch(ruta_archivo_viejo, ruta_archivo_nuevo, ruta_parche)
        
        # Limpieza del disco: borramos el parche que ya no sirve
        if os.path.exists(ruta_parche):
            os.remove(ruta_parche) 
            
        return True
    except Exception as e:
        print(f"Error en la reconstrucción binaria (Delta Patch): {e}")
        return False

def inyectar_logos_paraguacraft(game_dir, version, graficos_minimos, progress_callback=None):
    def _cb(msg):
        if progress_callback:
            try: progress_callback(msg)
            except Exception: pass

    pack_name = "ParaguacraftBrandPack"
    pack_dir = os.path.join(game_dir, "resourcepacks", pack_name)
    textures_gui_title_dir = os.path.join(pack_dir, "assets", "minecraft", "textures", "gui", "title")
    os.makedirs(textures_gui_title_dir, exist_ok=True)

    def _copy_logo_minecraft_png(src_path, dest_path, version_mayor_local):
        if not os.path.exists(src_path):
            _cb(f"[Logo] No encontrado: {src_path}")
            return
        try:
            from PIL import Image
            import zipfile as _zf, io as _zio

            im = Image.open(src_path)
            if im.mode != "RGBA":
                im = im.convert("RGBA")

            target_w, target_h = 256, 256
            try:
                mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                _jar = os.path.join(mc_dir, "versions", version, version + ".jar")
                _jar_key = "assets/minecraft/textures/gui/title/minecraft.png"
                if os.path.exists(_jar):
                    with _zf.ZipFile(_jar) as _z:
                        if _jar_key in _z.namelist():
                            van = Image.open(_zio.BytesIO(_z.read(_jar_key)))
                            target_w, target_h = van.size
            except Exception:
                pass
            if version_mayor_local < 16:
                target_w = max(target_w, 256)
                target_h = max(target_h, 256)

            w, h = im.size
            if w <= 0 or h <= 0:
                raise ValueError("invalid image size")

            out = Image.new("RGBA", (target_w, target_h), (0, 0, 0, 0))

            if target_w >= 512:
                # 1.21.4+: large single-piece texture (1024x176+), center logo in canvas
                LOGO_W = target_w
                LOGO_H = target_h
                bbox = im.getbbox()
                if bbox: im = im.crop(bbox)
                w, h = im.size
                scale = min(LOGO_W / w, LOGO_H / h)
                new_w = max(1, int(round(w * scale)))
                new_h = max(1, int(round(h * scale)))
                logo_scaled = im.resize((new_w, new_h), Image.LANCZOS)
                px = (LOGO_W - new_w) // 2
                py = (LOGO_H - new_h) // 2
                out.paste(logo_scaled, (px, py), logo_scaled)
            elif version_mayor_local >= 16:
                # 1.16 - 1.21.3: vanilla texture is 256x256, MC still uses two-row
                # side-by-side blitting. main_menu.png is a single horizontal logo,
                # so LEFT/RIGHT split -> ROW1(left on screen) + ROW2(right on screen)
                # = full logo displayed as one piece, centered.
                HALF_W = 155
                RENDER_H = 44
                ROW2_Y = 45
                bbox = im.getbbox()
                if bbox: im = im.crop(bbox)
                w, h = im.size
                scale = min((HALF_W * 2) / w, RENDER_H / h)
                new_w = max(1, int(round(w * scale)))
                new_h = max(1, int(round(h * scale)))
                logo_scaled = im.resize((new_w, new_h), Image.LANCZOS)
                canvas = Image.new("RGBA", (HALF_W * 2, RENDER_H), (0, 0, 0, 0))
                px = (HALF_W * 2 - new_w) // 2
                py = (RENDER_H - new_h) // 2
                canvas.paste(logo_scaled, (px, py), logo_scaled)
                out.paste(canvas.crop((0, 0, HALF_W, RENDER_H)), (0, 0))
                out.paste(canvas.crop((HALF_W, 0, HALF_W * 2, RENDER_H)), (0, ROW2_Y))
            else:
                # 1.0-1.15.2: legacy.png has PARAGUA and CRAFT stacked vertically.
                # Both words may reside entirely in the top portion (bottom can be
                # fully transparent). Crop to bbox first, then find the gap between
                # the two words by scanning row alpha, composite side-by-side, and
                # LEFT/RIGHT split -> ROW1 (screen left) + ROW2 (screen right).
                HALF_W = 155
                RENDER_H = 44
                ROW2_Y = 45
                GAP = 2
                bbox = im.getbbox()
                if bbox:
                    im = im.crop(bbox)
                w, h = im.size
                pix = im.load()
                row_alpha = [sum(1 for x in range(w) if pix[x, y][3] > 10) for y in range(h)]
                s0, s1 = h // 4, 3 * h // 4
                min_val = min(row_alpha[s0:s1]) if s0 < s1 else 0
                gap_rows = [i for i in range(s0, s1) if row_alpha[i] == min_val]
                split_y = gap_rows[len(gap_rows) // 2] if gap_rows else h // 2
                top_half = im.crop((0, 0, w, split_y))
                bot_half = im.crop((0, split_y, w, h))
                bbox_top = top_half.getbbox()
                bbox_bot = bot_half.getbbox()
                if bbox_top: top_half = top_half.crop(bbox_top)
                if bbox_bot: bot_half = bot_half.crop(bbox_bot)
                tw, th = top_half.size
                bw, bh = bot_half.size
                # Scale each word to fill RENDER_H independently (same visual height),
                # then shrink both uniformly if combined width overflows the canvas.
                new_tw = max(1, int(round(tw * RENDER_H / th)))
                new_bw = max(1, int(round(bw * RENDER_H / bh)))
                if new_tw + GAP + new_bw > HALF_W * 2:
                    fit = (HALF_W * 2 - GAP) / (new_tw + new_bw)
                    new_tw = max(1, int(round(new_tw * fit)))
                    new_bw = max(1, int(round(new_bw * fit)))
                    new_th = max(1, int(round(RENDER_H * fit)))
                    new_bh = new_th
                else:
                    new_th = RENDER_H
                    new_bh = RENDER_H
                top_scaled = top_half.resize((new_tw, new_th), Image.LANCZOS)
                bot_scaled = bot_half.resize((new_bw, new_bh), Image.LANCZOS)
                combined_w = new_tw + GAP + new_bw
                canvas = Image.new("RGBA", (HALF_W * 2, RENDER_H), (0, 0, 0, 0))
                start_x = (HALF_W * 2 - combined_w) // 2
                py_top = (RENDER_H - new_th) // 2
                py_bot = (RENDER_H - new_bh) // 2
                canvas.paste(top_scaled, (start_x, py_top), top_scaled)
                canvas.paste(bot_scaled, (start_x + new_tw + GAP, py_bot), bot_scaled)
                out.paste(canvas.crop((0, 0, HALF_W, RENDER_H)), (0, 0))
                out.paste(canvas.crop((HALF_W, 0, HALF_W * 2, RENDER_H)), (0, ROW2_Y))

            out.save(dest_path)
            _cb(f"[Logo] OK {os.path.basename(dest_path)} ({target_w}x{target_h})")
            return
        except Exception as e:
            _cb(f"[Logo] Error PIL: {e}. Copiando directo.")
            try:
                shutil.copyfile(src_path, dest_path)
            except Exception as e2:
                _cb(f"[Logo] Error copia: {e2}")

    try:
        version_split = version.split(".")
        if version_split[0] == "26":
            version_mayor = 26
            version_minor = int(version_split[1]) if len(version_split) >= 2 else 0
        else:
            version_mayor = int(version_split[1]) if len(version_split) >= 2 else 20
            version_minor = int(version_split[2]) if len(version_split) >= 3 else 0
    except Exception:
        version_mayor = 20
        version_minor = 0

    usa_nuevo_schema = version_mayor >= 26 or (version_mayor == 21 and version_minor >= 5)

    def _pf():
        if version_mayor == 21:
            if version_minor >= 4: return 46
            if version_minor >= 2: return 42
            return 34
        if version_mayor == 20:
            if version_minor >= 5: return 32
            if version_minor >= 3: return 22
            if version_minor >= 2: return 18
            return 15   # 1.20, 1.20.1
        if version_mayor == 19:
            if version_minor >= 4: return 13
            if version_minor >= 3: return 12
            return 9
        if version_mayor == 18: return 8
        if version_mayor == 17: return 7
        return 6 if version_minor >= 2 else 5  # 1.16.x

    if version_mayor >= 16:
        if usa_nuevo_schema:
            pack_block = {"description": "Marca Oficial Paraguacraft", "min_format": 70, "max_format": 99999}
        else:
            pf = _pf()
            soporta_range = version_mayor > 20 or (version_mayor == 20 and version_minor >= 2)
            if soporta_range:
                pack_block = {"pack_format": pf, "description": "Marca Oficial Paraguacraft", "supported_formats": [pf, 9999]}
            else:
                pack_block = {"pack_format": pf, "description": "Marca Oficial Paraguacraft"}
    elif version_mayor >= 15:  # 1.15.x
        pack_block = {"pack_format": 5, "description": "Marca Oficial Paraguacraft"}
    elif version_mayor >= 13:  # 1.13 - 1.14.x
        pack_block = {"pack_format": 4, "description": "Marca Oficial Paraguacraft"}
    elif version_mayor >= 11:  # 1.11 - 1.12.2
        pack_block = {"pack_format": 3, "description": "Marca Oficial Paraguacraft"}
    elif version_mayor >= 9:   # 1.9 - 1.10.2
        pack_block = {"pack_format": 2, "description": "Marca Oficial Paraguacraft"}
    elif version_mayor >= 6:   # 1.6.1 - 1.8.9
        pack_block = {"pack_format": 1, "description": "Marca Oficial Paraguacraft"}
    else:
        pack_block = {"pack_format": 1, "description": "Marca Oficial Paraguacraft"}
    mcmeta_str = json.dumps({"pack": pack_block})
    with open(os.path.join(pack_dir, "pack.mcmeta"), "w") as f: f.write(mcmeta_str)

    if getattr(sys, 'frozen', False): base_path = sys._MEIPASS
    else: base_path = os.path.dirname(os.path.abspath(__file__))

    # Marca en menú: todas las versiones (1.0-1.5.x usan textura pack clásico)
    aplica_marca_menu = True
    if aplica_marca_menu:
        import hashlib as _hashlib
        stamp_file = os.path.join(pack_dir, ".logo_stamp_v2")
        _main = "paraguacraft_main_menu.png" if version_mayor >= 16 else "paraguacraft_legacy.png"
        main_src = os.path.join(base_path, _main)
        _LOGO_VER = ":r25"
        try:
            with open(main_src, "rb") as _sf:
                current_stamp = _hashlib.md5(_sf.read()).hexdigest() + _LOGO_VER
        except Exception:
            current_stamp = ""
        try:
            with open(stamp_file, "r") as _sf:
                cached_stamp = _sf.read().strip()
        except Exception:
            cached_stamp = ""

        logo_dest_mc = os.path.join(textures_gui_title_dir, "minecraft.png")
        if current_stamp and current_stamp == cached_stamp and os.path.exists(logo_dest_mc):
            _cb("[Logo] Sin cambios, saltando.")
        else:
            activos = {
                _main: "minecraft.png",
                "paraguacraft_startup.png": "mojangstudios.png",
            }
            for src_name, dest_name in activos.items():
                logo_src = os.path.join(base_path, src_name)
                logo_dest = os.path.join(textures_gui_title_dir, dest_name)
                if os.path.exists(logo_src):
                    if dest_name == "minecraft.png":
                        _copy_logo_minecraft_png(logo_src, logo_dest, version_mayor)
                    else:
                        shutil.copyfile(logo_src, logo_dest)
            edicion_src = os.path.join(base_path, "paraguacraft_edition.png")
            edicion_dst = os.path.join(textures_gui_title_dir, "edition.png")
            if os.path.exists(edicion_src):
                shutil.copyfile(edicion_src, edicion_dst)
            elif version_mayor >= 12:
                try:
                    from PIL import Image as _PilImgEd
                    _PilImgEd.new("RGBA", (128, 16), (0, 0, 0, 0)).save(edicion_dst)
                except Exception:
                    pass
            if version_mayor < 6 and os.path.exists(logo_dest_mc):
                import zipfile as _zf_tp
                _tp_dir = os.path.join(game_dir, "texturepacks")
                os.makedirs(_tp_dir, exist_ok=True)
                _tp_zip = os.path.join(_tp_dir, pack_name + ".zip")
                try:
                    with _zf_tp.ZipFile(_tp_zip, 'w', _zf_tp.ZIP_DEFLATED) as _ztp:
                        _ztp.write(logo_dest_mc, "gui/title/minecraft.png")
                    _cb("[Logo] Textura pack clásico creado.")
                except Exception as _e:
                    _cb(f"[Logo] Error textura pack: {_e}")
            if current_stamp:
                try:
                    with open(stamp_file, "w") as _sf:
                        _sf.write(current_stamp)
                except Exception:
                    pass

    mc_dir_global = minecraft_launcher_lib.utils.get_minecraft_directory()
    _offline_skin = os.path.join(mc_dir_global, "paraguacraft_offline_skin.png")
    if os.path.isfile(_offline_skin):
        _sk_pack = os.path.join(game_dir, "resourcepacks", pack_name)
        _sk_wide = os.path.join(_sk_pack, "assets", "minecraft", "textures", "entity", "player", "wide")
        _sk_slim = os.path.join(_sk_pack, "assets", "minecraft", "textures", "entity", "player", "slim")
        _sk_old  = os.path.join(_sk_pack, "assets", "minecraft", "textures", "entity")
        try:
            for _d in [_sk_wide, _sk_slim, _sk_old]:
                os.makedirs(_d, exist_ok=True)
            shutil.copy2(_offline_skin, os.path.join(_sk_wide, "steve.png"))
            shutil.copy2(_offline_skin, os.path.join(_sk_slim, "alex.png"))
            shutil.copy2(_offline_skin, os.path.join(_sk_old,  "steve.png"))
        except Exception:
            pass

    if usa_nuevo_schema and aplica_marca_menu:
        import zipfile as _zf_pack
        zip_path = os.path.join(game_dir, "resourcepacks", pack_name + ".zip")
        try:
            with _zf_pack.ZipFile(zip_path, 'w', _zf_pack.ZIP_DEFLATED) as _zf:
                _zf.writestr("pack.mcmeta", mcmeta_str)
                for _fname in ["minecraft.png", "mojangstudios.png", "edition.png"]:
                    _src = os.path.join(textures_gui_title_dir, _fname)
                    if os.path.exists(_src):
                        _zf.write(_src, f"assets/minecraft/textures/gui/title/{_fname}")
            _cb(f"[Logo] ZIP creado: {pack_name}.zip")
        except Exception as _ze:
            _cb(f"[Logo] Error ZIP: {_ze}")

    options_path = os.path.join(game_dir, "options.txt")
    lineas = []
    if os.path.exists(options_path):
        with open(options_path, "r") as f: lineas = f.readlines()
    
    lineas = [l for l in lineas if not l.startswith("resourcePacks:")]
    lineas = [l for l in lineas if not l.startswith("incompatibleResourcePacks:")]
    lineas = [l for l in lineas if not l.startswith("texturepack:")]
    
    if version_mayor < 6:
        if aplica_marca_menu:
            lineas.append(f'texturepack:{pack_name}.zip\n')
    elif version_mayor < 13:
        packs = []
        if graficos_minimos: packs.append('"Pack_Graficos_Minimos.zip"')
        if aplica_marca_menu: packs.append(f'"{pack_name}"')
        if len(packs) > 0: lineas.append(f'resourcePacks:[{",".join(packs)}]\n')
    else:
        packs = ['"vanilla"']
        if graficos_minimos: packs.append('"file/Pack_Graficos_Minimos.zip"')
        if aplica_marca_menu:
            if usa_nuevo_schema:
                packs.append(f'"file/{pack_name}.zip"')
            else:
                packs.append(f'"file/{pack_name}"')
        lineas.append(f'resourcePacks:[{",".join(packs)}]\n')
    
    with open(options_path, "w") as f: f.writelines(lineas)

def optimizar_graficos(game_dir, graficos_minimos, version):
    try:
        p = version.split(".")
        version_mayor = 26 if p[0] == "26" else int(p[1])
    except Exception:
        version_mayor = 20

    options_path = os.path.join(game_dir, "options.txt")
    optionsof_path = os.path.join(game_dir, "optionsof.txt") 
    
    if graficos_minimos:
        if version_mayor < 13:
            opciones = {"renderDistance": "6", "fancyGraphics": "false", "ao": "0", "particles": "2", "maxFps": "60", "fboEnable": "true", "enableVsync": "false"}
            opciones_of = {"ofFastMath": "true", "ofDynamicLights": "3", "ofSmoothFps": "false", "ofClouds": "3"}
        else:
            opciones = {"renderDistance": "6", "simulationDistance": "5", "graphicsMode": "FAST", "particles": "2", "entityDistanceScaling": "0.5", "biomeBlendRadius": "0", "maxFps": "60", "enableVsync": "false"}
            opciones_of = {}
    else:
        if version_mayor < 13:
            opciones = {"renderDistance": "12", "fancyGraphics": "true", "ao": "2", "particles": "0", "maxFps": "144", "enableVsync": "false"}
            opciones_of = {"ofFastMath": "false", "ofDynamicLights": "3"}
        else:
            opciones = {"renderDistance": "12", "simulationDistance": "8", "graphicsMode": "FANCY", "particles": "1", "entityDistanceScaling": "1.0", "biomeBlendRadius": "2", "maxFps": "144", "enableVsync": "false"}
            opciones_of = {}

    lineas = []
    if os.path.exists(options_path):
        with open(options_path, "r") as f: lineas = f.readlines()

    nuevas_lineas = []
    for line in lineas:
        clave = line.split(":")[0]
        if clave in opciones:
            nuevas_lineas.append(f"{clave}:{opciones[clave]}\n")
            del opciones[clave]
        else: nuevas_lineas.append(line)
            
    for clave, valor in opciones.items(): nuevas_lineas.append(f"{clave}:{valor}\n")
    with open(options_path, "w") as f: f.writelines(nuevas_lineas)

    if opciones_of:
        lineas_of = []
        if os.path.exists(optionsof_path):
            with open(optionsof_path, "r") as f: lineas_of = f.readlines()
        nuevas_lineas_of = []
        for line in lineas_of:
            clave = line.split(":")[0]
            if clave in opciones_of:
                nuevas_lineas_of.append(f"{clave}:{opciones_of[clave]}\n")
                del opciones_of[clave]
            else: nuevas_lineas_of.append(line)
        for clave, valor in opciones_of.items(): nuevas_lineas_of.append(f"{clave}:{valor}\n")
        with open(optionsof_path, "w") as f: f.writelines(nuevas_lineas_of)

def instalar_extras_graficos(game_dir, version, progress_callback, graficos_minimos):
    rp_dir = os.path.join(game_dir, "resourcepacks")
    sp_dir = os.path.join(game_dir, "shaderpacks")
    os.makedirs(rp_dir, exist_ok=True); os.makedirs(sp_dir, exist_ok=True)

    if not graficos_minimos:
        return

    if os.path.exists(os.path.join(rp_dir, "Pack_Graficos_Minimos.zip")): return

    headers = {"User-Agent": "ParaguacraftLauncher/1.0"}
    if progress_callback: progress_callback("Buscando Texturas Faithful 32x...")
    url_api = "https://api.modrinth.com/v2/project/faithful-32x/version"
    params = {"game_versions": f'["{version}"]'}
    try:
        r = requests.get(url_api, params=params, headers=headers, timeout=4)
        if r.status_code == 200:
            data = r.json()
            if len(data) > 0:
                archivo = data[0]["files"][0]
                ruta_archivo = os.path.join(rp_dir, "Pack_Graficos_Minimos.zip")
                if not os.path.exists(ruta_archivo):
                    if progress_callback: progress_callback("Descargando texturas...")
                    r_dl = requests.get(archivo["url"], stream=True, headers=headers, timeout=10)
                    if r_dl.status_code == 200:
                        with open(ruta_archivo, "wb") as f: shutil.copyfileobj(r_dl.raw, f)
    except Exception: pass

def _offline_uuid(username):
    """Deterministic offline UUID — same algorithm as vanilla Minecraft servers.
    Guarantees the same UUID for the same username across sessions."""
    import hashlib
    data = ("OfflinePlayer:" + username).encode("utf-8")
    md5 = bytearray(hashlib.md5(data).digest())
    md5[6] = (md5[6] & 0x0f) | 0x30  # version 3
    md5[8] = (md5[8] & 0x3f) | 0x80  # variant bits
    return str(uuid.UUID(bytes=bytes(md5)))

def _descargar_mods_fabric_slugs(mods_dir, version, slugs, progress_callback, headers):
    os.makedirs(mods_dir, exist_ok=True)
    _existing_lower = {f.lower() for f in os.listdir(mods_dir) if f.endswith(".jar") or f.endswith(".jar.disabled")}
    for slug in slugs:
        # Skip API call if a file matching this slug name already exists locally
        if any(slug.lower() in f for f in _existing_lower):
            continue
        url_api = f"https://api.modrinth.com/v2/project/{slug}/version"
        params = {"loaders": '["fabric"]', "game_versions": f'["{version}"]'}
        try:
            r = requests.get(url_api, params=params, headers=headers, timeout=10)
            if r.status_code != 200:
                continue
            data = r.json()
            if len(data) == 0:
                continue
            archivo = data[0]["files"][0]
            fn = archivo["filename"]
            mod_path = os.path.join(mods_dir, fn)
            if os.path.exists(mod_path) or os.path.exists(mod_path + ".disabled"):
                continue
            if progress_callback:
                progress_callback(f"Descargando {slug}...")
            r_dl = requests.get(archivo["url"], stream=True, headers=headers, timeout=30)
            if r_dl.status_code == 200:
                with open(mod_path, "wb") as f:
                    shutil.copyfileobj(r_dl.raw, f)
        except Exception:
            pass

_BUNDLE_CACHE_DAYS = 30  # invalidate mod cache after this many days

def instalar_bundle_fabric_iris_cache(minecraft_directory, game_dir, version, progress_callback, lan_distancia=False):
    """
    Caché global por versión de MC: los .jar se descargan una sola vez y se copian a la instancia.
    El caché expira cada 30 días para recibir actualizaciones de mods desde Modrinth.
    """
    import datetime as _dt2
    cache_root = os.path.join(minecraft_directory, "Paraguacraft_cache", "fabric_iris", version.replace(os.sep, "_"))
    os.makedirs(cache_root, exist_ok=True)
    mods_dir = os.path.join(game_dir, "mods")
    os.makedirs(mods_dir, exist_ok=True)

    slugs = ["fabric-api", "sodium", "iris", "lithium", "ferrite-core", "entityculling", "immediatelyfast", "modmenu"]
    if lan_distancia:
        slugs.append("e4mc")

    marker = os.path.join(cache_root, ".cache_ok")

    cache_valid = False
    cache_expired = False
    if os.path.exists(marker):
        try:
            age_days = (_dt2.datetime.now() - _dt2.datetime.fromtimestamp(os.path.getmtime(marker))).days
            cache_valid = age_days < _BUNDLE_CACHE_DAYS
            cache_expired = not cache_valid
        except Exception:
            cache_valid = True

    if not cache_valid:
        headers = {"User-Agent": "ParaguacraftLauncher/2.1"}
        if cache_expired:
            if progress_callback:
                progress_callback("Actualizando mods Fabric + Iris (cache expirado)...")
            for _old in [f for f in os.listdir(cache_root) if f.endswith(".jar")]:
                try: os.remove(os.path.join(cache_root, _old))
                except Exception: pass
            # Only remove enabled .jar files — leave .jar.disabled so user preferences are kept
            _inst_files = [f for f in os.listdir(mods_dir) if f.endswith(".jar")]
            for slug in slugs:
                slug_l = slug.lower()
                for _if in _inst_files:
                    if slug_l in _if.lower():
                        try: os.remove(os.path.join(mods_dir, _if))
                        except Exception: pass
            try: os.remove(marker)
            except Exception: pass
        else:
            if progress_callback:
                progress_callback("Descargando mods Fabric + Iris...")
        _descargar_mods_fabric_slugs(cache_root, version, slugs, progress_callback, headers)
        try:
            with open(marker, "w") as _m: _m.write("ok")
        except Exception:
            pass

    # Build slug-aware lookup of what's already in the instance (enabled OR disabled)
    _inst_lower = {ff.lower() for ff in os.listdir(mods_dir)} if os.path.isdir(mods_dir) else set()
    cache_jars = [f for f in os.listdir(cache_root) if f.endswith(".jar")]
    for f in cache_jars:
        dest = os.path.join(mods_dir, f)
        if os.path.exists(dest) or os.path.exists(dest + ".disabled"):
            continue
        # Skip if any version of this mod (by slug) is already present — respects disable/delete
        _slug = next((s for s in slugs if s.lower() in f.lower()), None)
        if _slug and any(_slug.lower() in ff for ff in _inst_lower):
            continue
        try:
            shutil.copy2(os.path.join(cache_root, f), dest)
        except Exception:
            pass

def instalar_mods_por_motor(game_dir, version, motor_elegido, progress_callback, lan_distancia=False, minecraft_directory=None):
    """Mods según add-on: Fabric solo = API; Fabric+Iris = caché global; Vanilla/Forge/OptiFine no usan Modrinth aquí."""
    if minecraft_directory is None:
        minecraft_directory = minecraft_launcher_lib.utils.get_minecraft_directory()

    mods_dir = os.path.join(game_dir, "mods")
    os.makedirs(mods_dir, exist_ok=True)
    motor_lower = motor_elegido.lower()
    headers = {"User-Agent": "ParaguacraftLauncher/2.1"}

    if motor_lower == "vanilla" or motor_lower.startswith("optifine") or "neoforge" in motor_lower or ("forge" in motor_lower and "neo" not in motor_lower and "fabric" not in motor_lower and "iris" not in motor_lower and "optifine" not in motor_lower):
        return

    if "fabric" in motor_lower and "iris" in motor_lower:
        instalar_bundle_fabric_iris_cache(minecraft_directory, game_dir, version, progress_callback, lan_distancia)
        return

    if "fabric" in motor_lower:
        if progress_callback:
            progress_callback(f"Configurando Fabric (solo API): {motor_elegido}...")
        _descargar_mods_fabric_slugs(mods_dir, version, ["fabric-api"], progress_callback, headers)
        if lan_distancia:
            _descargar_mods_fabric_slugs(mods_dir, version, ["e4mc"], progress_callback, headers)
        return

def _java_exe_para_mc(minecraft_directory):
    """Devuelve la ruta al ejecutable Java del runtime bundled de Minecraft, o 'java' como fallback.
    Prefiere java-runtime-* (Java 17/21) sobre jre-legacy (Java 8) para el instalador de OptiFine."""
    java_name = "java.exe" if platform.system() == "Windows" else "java"
    runtime_root = os.path.join(minecraft_directory, "runtime")
    if os.path.isdir(runtime_root):
        plat_key = "windows" if platform.system() == "Windows" else ("mac-os" if platform.system() == "Darwin" else "linux")
        # Ordenar: java-runtime-* primero (modernos, mayor key=1), jre-legacy ultimo (key=0)
        entries = sorted(
            [e for e in os.listdir(runtime_root) if e.startswith("java") or e.startswith("jre")],
            key=lambda e: (1 if e.startswith("java") else 0, e),
            reverse=True
        )
        for entry in entries:
            bin_path = os.path.join(runtime_root, entry, plat_key, entry, "bin", java_name)
            if os.path.exists(bin_path):
                return bin_path
    return "java"

def _java_exe_para_version(minecraft_directory, version_base):
    """Picks the correct bundled Java runtime for a given MC version.
    <1.17 → jre-legacy (Java 8) | 1.17-1.20 → gamma/beta (Java 17) | 1.21+/26.x → delta/loom (Java 21)"""
    java_name = "java.exe" if platform.system() == "Windows" else "java"
    plat_key = "windows" if platform.system() == "Windows" else ("mac-os" if platform.system() == "Darwin" else "linux")
    runtime_root = os.path.join(minecraft_directory, "runtime")
    try:
        parts = version_base.split(".")
        is_new = parts[0] == "26"
        minor = 0 if is_new else (int(parts[1]) if len(parts) >= 2 else 20)
    except Exception:
        minor, is_new = 20, False
    if is_new or minor >= 21:
        candidates = ["java-runtime-delta", "java-runtime-loom", "java-runtime-gamma"]
    elif minor >= 17:
        candidates = ["java-runtime-gamma", "java-runtime-beta", "java-runtime-delta"]
    else:
        candidates = ["jre-legacy", "java-runtime-gamma", "java-runtime-beta"]
    for entry in candidates:
        bin_path = os.path.join(runtime_root, entry, plat_key, entry, "bin", java_name)
        if os.path.exists(bin_path):
            return bin_path
    return "java"


def _asegurar_java_runtime(version_id, minecraft_directory, progress_callback):
    """Lee el JSON de la version, sigue la cadena inheritsFrom, y descarga el runtime Java
    correcto (jre-legacy, java-runtime-gamma, java-runtime-delta, etc.) si no esta instalado."""
    try:
        versions_dir = os.path.join(minecraft_directory, "versions")
        java_name = "java.exe" if platform.system() == "Windows" else "java"
        plat_key = "windows" if platform.system() == "Windows" else ("mac-os" if platform.system() == "Darwin" else "linux")

        # Seguir cadena inheritsFrom para encontrar el campo javaVersion
        java_component = None
        _visited = set()
        _current = version_id
        while _current and _current not in _visited:
            _visited.add(_current)
            ver_json_path = os.path.join(versions_dir, _current, _current + ".json")
            if not os.path.exists(ver_json_path):
                break
            with open(ver_json_path, "r", encoding="utf-8") as _f:
                _data = json.load(_f)
            if "javaVersion" in _data:
                java_component = _data["javaVersion"].get("component")
                break
            _current = _data.get("inheritsFrom")

        if not java_component:
            return

        # Comprobar si el runtime ya esta instalado
        runtime_root = os.path.join(minecraft_directory, "runtime")
        bin_path = os.path.join(runtime_root, java_component, plat_key, java_component, "bin", java_name)
        if os.path.exists(bin_path):
            return

        # Descargar el runtime correcto usando minecraft_launcher_lib
        if progress_callback:
            progress_callback(f"Descargando Java requerido ({java_component})...")
        minecraft_launcher_lib.runtime.install_jvm_runtime(
            java_component, minecraft_directory,
            callback={"setStatus": lambda e: (progress_callback(e) if progress_callback else None)}
        )
    except Exception as e:
        print(f"[Java Runtime] Error asegurando runtime para {version_id}: {e}")


def _instalar_optifine(version_base, minecraft_directory, progress_callback):
    """Descarga e instala OptiFine para version_base. Devuelve el version ID instalado o None."""
    import urllib.request as _ur
    import json as _json
    import tempfile
    versions_dir = os.path.join(minecraft_directory, "versions")

    if progress_callback: progress_callback(f"Descargando Minecraft {version_base} (base para OptiFine)...")
    try:
        _install_mc_with_retry(version_base, minecraft_directory,
            {"setStatus": lambda e: (progress_callback(e) if progress_callback else None)},
            progress_callback)
    except Exception as e:
        if progress_callback: progress_callback(f"Error instalando base vanilla: {e}")
        return None

    if progress_callback: progress_callback(f"Buscando OptiFine para {version_base}...")
    try:
        req = _ur.Request(
            f"https://bmclapi2.bangbang93.com/optifine/{version_base}",
            headers={"User-Agent": "ParaguacraftLauncher/2.7"})
        with _ur.urlopen(req, timeout=12) as resp:
            builds = _json.loads(resp.read())
    except Exception as e:
        if progress_callback: progress_callback(f"No se pudo obtener lista de OptiFine: {e}")
        return None

    if not builds:
        if progress_callback: progress_callback(f"Sin OptiFine disponible para {version_base}")
        return None

    build = builds[0]
    of_type  = build.get("type",  "HD_U")
    of_patch = build.get("patch", "")
    of_filename = build.get("filename", f"OptiFine_{version_base}_{of_type}_{of_patch}.jar")
    dl_url = f"https://bmclapi2.bangbang93.com/optifine/{version_base}/{of_type}/{of_patch}"

    if progress_callback: progress_callback(f"Descargando {of_filename}...")
    tmpdir = tempfile.mkdtemp(prefix="paraguacraft_of_")
    of_jar = os.path.join(tmpdir, of_filename)
    try:
        req = _ur.Request(dl_url, headers={"User-Agent": "ParaguacraftLauncher/2.7"})
        with _ur.urlopen(req, timeout=90) as resp, open(of_jar, "wb") as f:
            shutil.copyfileobj(resp, f)
    except Exception as e:
        if progress_callback: progress_callback(f"Error descargando OptiFine: {e}")
        try: shutil.rmtree(tmpdir)
        except Exception: pass
        return None

    if progress_callback: progress_callback("Instalando OptiFine...")
    java_exe = _java_exe_para_mc(minecraft_directory)
    try:
        _flags = subprocess.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
        subprocess.run(
            [java_exe, "-cp", of_jar, "optifine.Installer", minecraft_directory],
            capture_output=True, timeout=120, creationflags=_flags)
    except Exception as e:
        if progress_callback: progress_callback(f"Error ejecutando instalador OptiFine: {e}")
    finally:
        try: shutil.rmtree(tmpdir)
        except Exception: pass

    if os.path.isdir(versions_dir):
        for v in sorted(os.listdir(versions_dir), reverse=True):
            if version_base in v and "OptiFine" in v:
                if os.path.exists(os.path.join(versions_dir, v, v + ".json")):
                    if progress_callback: progress_callback(f"OptiFine instalado: {v}")
                    return v

    if progress_callback: progress_callback("OptiFine no pudo instalarse correctamente.")
    return None


def _instalar_neoforge(version_base, minecraft_directory, progress_callback, on_progress):
    """Instala NeoForge para version_base usando minecraft_launcher_lib.neoforge. Devuelve el version ID o None."""
    versions_dir = os.path.join(minecraft_directory, "versions")

    # 1) Base vanilla primero
    if progress_callback: progress_callback(f"Descargando Minecraft {version_base} (base para NeoForge)...")
    try:
        _install_mc_with_retry(version_base, minecraft_directory, {"setStatus": on_progress}, progress_callback)
    except Exception as _be:
        if progress_callback: progress_callback(f"Advertencia descargando base: {_be}")

    # 2) Java runtime bundled
    _asegurar_java_runtime(version_base, minecraft_directory, progress_callback)
    _java_exe = _java_exe_para_mc(minecraft_directory)
    _java_bin = os.path.dirname(_java_exe) if _java_exe != "java" else ""
    _orig_path = os.environ.get("PATH", "")
    if _java_bin and _java_bin not in _orig_path:
        os.environ["PATH"] = _java_bin + os.pathsep + _orig_path

    try:
        if progress_callback: progress_callback(f"Buscando NeoForge para {version_base}...")
        nf_versions = []
        try:
            nf_versions = minecraft_launcher_lib.neoforge.get_neoforge_versions(version_base)
        except AttributeError:
            if progress_callback: progress_callback("NeoForge no está soportado en esta versión de la librería.")
            return None
        except Exception as _e:
            if progress_callback: progress_callback(f"Error buscando NeoForge: {_e}")
            return None

        if not nf_versions:
            if progress_callback: progress_callback(f"NeoForge no está disponible aún para {version_base}.")
            return None

        nf_latest = nf_versions[0]
        if progress_callback: progress_callback(f"Instalando NeoForge {nf_latest}...")
        try:
            minecraft_launcher_lib.neoforge.install_neoforge_version(
                nf_latest, minecraft_directory, java=_java_exe, callback={"setStatus": on_progress})
        except TypeError:
            minecraft_launcher_lib.neoforge.install_neoforge_version(
                nf_latest, minecraft_directory, callback={"setStatus": on_progress})

        # Buscar la versión real creada (el prefijo MC sin el "1." inicial, ej: "21.1" para "1.21.1")
        _vp = version_base.split(".")
        _nf_prefix = ".".join(_vp[1:]) if len(_vp) >= 2 else version_base
        if os.path.isdir(versions_dir):
            for _nfv in sorted(os.listdir(versions_dir), reverse=True):
                if "neoforge" in _nfv.lower() and _nf_prefix in _nfv:
                    if os.path.exists(os.path.join(versions_dir, _nfv, _nfv + ".json")):
                        if progress_callback: progress_callback(f"NeoForge instalado: {_nfv}")
                        return _nfv
            # Fallback: ID canónico
            _fallback = f"neoforge-{nf_latest}"
            if os.path.exists(os.path.join(versions_dir, _fallback, _fallback + ".json")):
                return _fallback
    finally:
        os.environ["PATH"] = _orig_path


def _install_mc_with_retry(version, mc_dir, cb_dict, progress_cb=None):
    """install_minecraft_version con reintento automático (hasta 3 intentos).
    Detecta archivos con hash corrupto, los elimina y reintenta la descarga."""
    import re as _re_r
    for _attempt in range(3):
        try:
            minecraft_launcher_lib.install.install_minecraft_version(version, mc_dir, callback=cb_dict)
            return
        except Exception as _ie:
            _m = _re_r.match(r'^(.*?) has the wrong', str(_ie))
            if _m and _attempt < 2:
                _bad = _m.group(1).strip()
                if progress_cb:
                    progress_cb(f"Archivo corrupto, limpiando y reintentando ({_attempt + 2}/3)...")
                if os.path.exists(_bad):
                    try: os.remove(_bad)
                    except Exception: pass
            else:
                raise


def lanzar_minecraft(version="1.20.4", username="Player", max_ram="4G", gc_type="G1GC", optimizar=False, motor_elegido="Vanilla", papa_mode=False, usar_mesa=False, mostrar_consola=False, progress_callback=None, uuid_real=None, token_real=None, lan_distancia=False, fabric_loader_override=None, server_ip="", java_path=None):
    minecraft_directory = minecraft_launcher_lib.utils.get_minecraft_directory()

    def on_progress(event):
        if progress_callback: progress_callback(event)

    version_base = version.strip()

    # 1. Determinamos el Loader base leyendo el nombre del motor (NeoForge antes que Forge)
    motor_lower = motor_elegido.lower()
    if "neoforge" in motor_lower:
        tipo_cliente_base = "NeoForge"
    elif "fabric" in motor_lower:
        tipo_cliente_base = "Fabric"
    elif "forge" in motor_lower:
        tipo_cliente_base = "Forge"
    elif "optifine" in motor_lower:
        tipo_cliente_base = "OptiFine"
    else:
        tipo_cliente_base = "Vanilla"

    version_a_lanzar = version_base

    # --- BYPASS DE ARRANQUE RÁPIDO ---
    version_instalada = False
    _versions_dir = os.path.join(minecraft_directory, "versions")
    _versiones_locales = os.listdir(_versions_dir) if os.path.exists(_versions_dir) else []
    if tipo_cliente_base == "Fabric":
        for v in _versiones_locales:
            if v.startswith("fabric-loader-") and version_base in v:
                _fjson = os.path.join(_versions_dir, v, v + ".json")
                if os.path.exists(_fjson):
                    _base_jar = os.path.join(_versions_dir, version_base, version_base + ".jar")
                    if not (os.path.exists(_base_jar) and os.path.getsize(_base_jar) > 4096):
                        continue
                    if fabric_loader_override:
                        _installed_loader = v[len("fabric-loader-"):-(len(version_base) + 1)]
                        if _installed_loader != fabric_loader_override:
                            continue
                    version_a_lanzar = v
                    version_instalada = True
                    break
                else:
                    try:
                        shutil.rmtree(os.path.join(_versions_dir, v))
                    except Exception:
                        pass
    elif tipo_cliente_base == "OptiFine":
        for v in sorted(_versiones_locales, reverse=True):
            if version_base in v and "OptiFine" in v:
                if os.path.exists(os.path.join(_versions_dir, v, v + ".json")):
                    version_a_lanzar = v
                    version_instalada = True
                    break
    elif tipo_cliente_base == "Forge":
        for v in sorted(_versiones_locales, reverse=True):
            if version_base in v and "forge" in v.lower() and "neoforge" not in v.lower():
                if os.path.exists(os.path.join(_versions_dir, v, v + ".json")):
                    version_a_lanzar = v
                    version_instalada = True
                    break
    elif tipo_cliente_base == "NeoForge":
        _vp_nf = version_base.split(".")
        _nf_prefix = ".".join(_vp_nf[1:]) if len(_vp_nf) >= 2 else version_base
        for v in sorted(_versiones_locales, reverse=True):
            if "neoforge" in v.lower() and _nf_prefix in v:
                if os.path.exists(os.path.join(_versions_dir, v, v + ".json")):
                    version_a_lanzar = v
                    version_instalada = True
                    break
    else:
        ruta_version = os.path.join(_versions_dir, version_base)
        version_json_path = os.path.join(ruta_version, version_base + ".json")
        version_jar_path = os.path.join(ruta_version, version_base + ".jar")
        _jar_ok = os.path.exists(version_jar_path) and os.path.getsize(version_jar_path) > 4096
        if os.path.exists(ruta_version) and os.path.exists(version_json_path) and _jar_ok:
            version_instalada = True
        elif os.path.exists(ruta_version):
            try:
                shutil.rmtree(ruta_version)
            except Exception:
                pass

    # Si NO está instalada, ejecutamos el motor de descarga
    if not version_instalada:
        if tipo_cliente_base == "Fabric":
            if progress_callback: progress_callback("Instalando Fabric (Motor base)...")
            loader_nuevo = fabric_loader_override or minecraft_launcher_lib.fabric.get_latest_loader_version()
            minecraft_launcher_lib.fabric.install_fabric(version_base, minecraft_directory, loader_nuevo, callback={"setStatus": on_progress})
            version_a_lanzar = f"fabric-loader-{loader_nuevo}-{version_base}"

        elif tipo_cliente_base == "Forge":
            # 1) Instalar base vanilla primero (el installer de Forge la necesita)
            if progress_callback: progress_callback(f"Descargando Minecraft {version_base} base (requerido por Forge)...")
            try:
                _install_mc_with_retry(version_base, minecraft_directory, {"setStatus": on_progress}, progress_callback)
            except Exception as _be:
                if progress_callback: progress_callback(f"Advertencia descargando base: {_be}")
            # 2) Asegurar Java runtime bundled antes de correr el installer
            _asegurar_java_runtime(version_base, minecraft_directory, progress_callback)
            # 3) Agregar Java bundled al PATH para que el Forge installer lo encuentre
            _java_exe_forge = _java_exe_para_mc(minecraft_directory)
            _java_bin_dir = os.path.dirname(_java_exe_forge) if _java_exe_forge != "java" else ""
            _orig_path = os.environ.get("PATH", "")
            if _java_bin_dir and _java_bin_dir not in _orig_path:
                os.environ["PATH"] = _java_bin_dir + os.pathsep + _orig_path
            try:
                if progress_callback: progress_callback("Instalando Forge...")
                forge_version = minecraft_launcher_lib.forge.find_forge_version(version_base)
                if not forge_version:
                    if progress_callback: progress_callback(f"Forge no está disponible aún para {version_base}.")
                else:
                    try:
                        minecraft_launcher_lib.forge.install_forge_version(forge_version, minecraft_directory, java=_java_exe_forge, callback={"setStatus": on_progress})
                    except TypeError:
                        minecraft_launcher_lib.forge.install_forge_version(forge_version, minecraft_directory, callback={"setStatus": on_progress})
                    # Buscar la version real creada (el nombre puede diferir de forge_version)
                    _vdir = os.path.join(minecraft_directory, "versions")
                    _found_forge = None
                    for _fv in sorted(os.listdir(_vdir), reverse=True):
                        if version_base in _fv and "forge" in _fv.lower() and "neoforge" not in _fv.lower():
                            if os.path.exists(os.path.join(_vdir, _fv, _fv + ".json")):
                                _found_forge = _fv
                                break
                    version_a_lanzar = _found_forge if _found_forge else forge_version
            finally:
                os.environ["PATH"] = _orig_path

        elif tipo_cliente_base == "OptiFine":
            of_ver = _instalar_optifine(version_base, minecraft_directory, progress_callback)
            if of_ver:
                version_a_lanzar = of_ver

        elif tipo_cliente_base == "NeoForge":
            nf_ver = _instalar_neoforge(version_base, minecraft_directory, progress_callback, on_progress)
            if nf_ver:
                version_a_lanzar = nf_ver

        else:
            if progress_callback: progress_callback(f"Descargando juego base {version_base}...")
            _install_mc_with_retry(version_base, minecraft_directory, {"setStatus": on_progress}, progress_callback)

    # 1.5 DEPENDENCIAS: para OptiFine y Forge, asegurar que todos los JARs estén
    # descargados (launchwrapper, etc.) ya que get_minecraft_command los necesita
    # en disco. install_minecraft_version es idempotente: no re-descarga lo que ya existe.
    if tipo_cliente_base in ("OptiFine", "Forge", "NeoForge") and version_a_lanzar != version_base:
        try:
            if progress_callback: progress_callback("Verificando dependencias del loader...")
            _install_mc_with_retry(
                version_a_lanzar, minecraft_directory,
                {"setStatus": lambda e: (progress_callback(e) if progress_callback else None)},
                progress_callback)
        except Exception as _dep_e:
            if progress_callback: progress_callback(f"Advertencia verificando dependencias: {_dep_e}")

    # 2. AISLAMIENTO DE INSTANCIAS DINÁMICO
    folder_name = carpeta_instancia_paraguacraft(version_base, motor_elegido)
    
    game_dir = os.path.join(minecraft_directory, "instancias", folder_name)
    os.makedirs(game_dir, exist_ok=True)
    os.makedirs(os.path.join(game_dir, "server-resource-packs"), exist_ok=True)

    # 3. MODS Y GRÁFICOS (Llamamos a la nueva función inteligente)
    instalar_mods_por_motor(game_dir, version_base, motor_elegido, progress_callback, lan_distancia, minecraft_directory)
        
    instalar_extras_graficos(game_dir, version_base, progress_callback, optimizar)
    inyectar_logos_paraguacraft(game_dir, version_base, optimizar, progress_callback)
    inyectar_splash_paraguacraft(game_dir, version_base, motor_elegido, progress_callback)

    if optimizar: 
        optimizar_graficos(game_dir, optimizar, version_base)

    # (A PARTIR DE ACÁ SIGUE TU CÓDIGO NORMAL CON jvm_arguments Y GestorNube)
    # 4. ARGUMENTOS DE JAVA
    try:
        _vp = version_base.split(".")
        if _vp[0] == "26":
            _vm = 26
        else:
            _vm = int(_vp[1]) if len(_vp) >= 2 else 20
    except Exception:
        _vm = 20

    # Auto GC: elige el GC óptimo según versión de Java y RAM disponible
    if gc_type in ("Auto", "CMS", ""):
        try:
            _ram_num = int(str(max_ram).replace("G", "").replace("M", "").strip())
            _ram_gb = _ram_num if "M" not in str(max_ram) else _ram_num / 1024
        except Exception:
            _ram_gb = 4
        if _vm < 17:
            gc_type = "G1GC"          # Java 8: solo G1GC seguro
        elif _vm >= 17 and _ram_gb >= 8:
            gc_type = "ZGC"           # Java 17/21 + RAM alta: ZGC óptimo
        else:
            gc_type = "G1GC"          # caso general: G1GC con Aikars flags

    if _vm < 17 and gc_type in ("ZGC", "Shenandoah"):
        gc_type = "G1GC"

    if _vm < 17:
        # Java 8 era (MC <= 1.16): jre-legacy bundled by Mojang is 8u51.
        # Many Aikars G1GC flags (e.g. G1RSetUpdatingPauseTimePercent) were added
        # in 8u60+ and are unrecognized on 8u51 → JVM exits with code 1.
        # Use a conservative set known to work on Java 8u51.
        jvm_arguments = [
            f"-Xmx{max_ram}", f"-Xms{max_ram}",
            "-XX:+UnlockExperimentalVMOptions",
            "-XX:+UseG1GC",
            "-XX:G1NewSizePercent=20",
            "-XX:G1ReservePercent=20",
            "-XX:MaxGCPauseMillis=50",
            "-XX:G1HeapRegionSize=32M",
            "-XX:InitiatingHeapOccupancyPercent=15",
            "-XX:+DisableExplicitGC",
        ]
    else:
        jvm_arguments = [f"-Xmx{max_ram}", f"-Xms{max_ram}", "-XX:+UnlockExperimentalVMOptions", "-XX:+DisableExplicitGC", "-XX:+AlwaysPreTouch"]
        if gc_type == "G1GC":
            jvm_arguments.extend(["-XX:+UseG1GC", "-XX:G1NewSizePercent=30", "-XX:G1MaxNewSizePercent=40", "-XX:G1HeapRegionSize=8M", "-XX:G1ReservePercent=20", "-XX:G1HeapWastePercent=5", "-XX:G1MixedGCCountTarget=4", "-XX:InitiatingHeapOccupancyPercent=15", "-XX:G1MixedGCLiveThresholdPercent=90", "-XX:G1RSetUpdatingPauseTimePercent=5", "-XX:SurvivorRatio=32", "-XX:+PerfDisableSharedMem", "-XX:MaxTenuringThreshold=1", "-Dusing.aikars.flags=https://mcflags.emc.gs", "-Daikars.new.flags=true"])
        elif gc_type == "ZGC": jvm_arguments.append("-XX:+UseZGC")
        elif gc_type == "Shenandoah": jvm_arguments.append("-XX:+UseShenandoahGC")

    # 5. CONFIGURACIÓN FINAL DEL JUEGO
    options = {
        "username": username,
        "uuid": uuid_real if uuid_real else _offline_uuid(username),
        "token": token_real if token_real else "",
        "jvmArguments": jvm_arguments,
        "gameDirectory": game_dir # <- Acá está la magia de los perfiles aislados
    }

    if papa_mode:
        options["customResolution"] = True
        options["resolutionWidth"] = "800"
        options["resolutionHeight"] = "600"

    # 4.5 JAVA RUNTIME: asegurar que el Java correcto esta instalado
    _asegurar_java_runtime(version_a_lanzar, minecraft_directory, progress_callback)
    _java_for_launch = _java_exe_para_version(minecraft_directory, version_base)
    if _java_for_launch != "java":
        options["executablePath"] = _java_for_launch
    if java_path and os.path.isfile(java_path):
        options["executablePath"] = java_path

    command = minecraft_launcher_lib.command.get_minecraft_command(version_a_lanzar, minecraft_directory, options)

    if server_ip and isinstance(server_ip, str) and server_ip.strip():
        ip_clean = server_ip.strip()
        if ":" in ip_clean:
            parts = ip_clean.rsplit(":", 1)
            command += ["--server", parts[0], "--port", parts[1]]
        else:
            command += ["--server", ip_clean, "--port", "25565"]

    entorno = os.environ.copy()
    if usar_mesa:
        entorno["MESA_GL_VERSION_OVERRIDE"] = "3.3"
        entorno["MESA_GLSL_VERSION_OVERRIDE"] = "330"

    options_txt_path = os.path.join(game_dir, "options.txt")
    servers_dat_path = os.path.join(game_dir, "servers.dat")

    if GestorNube is not None:
        try:
            import threading as _threading
            _threading.Thread(
                target=lambda: GestorNube().descargar_datos(username, options_txt_path, servers_dat_path),
                daemon=True).start()
        except Exception:
            pass

    if progress_callback:
        progress_callback("¡Abriendo Paraguacraft!")

    logs_dir = os.path.join(game_dir, "logs")
    os.makedirs(logs_dir, exist_ok=True)
    game_log_path = os.path.join(logs_dir, "paraguacraft_launch.log")

    if mostrar_consola and platform.system() == "Windows":
        flags_creacion = subprocess.CREATE_NEW_CONSOLE
        log_file = None
        salida = None
    else:
        flags_creacion = subprocess.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
        log_file = open(game_log_path, "w", encoding="utf-8", errors="replace")
        salida = log_file

    proceso = subprocess.Popen(command, env=entorno, creationflags=flags_creacion, stdout=salida, stderr=salida, stdin=subprocess.DEVNULL, cwd=game_dir)

    try:
        import psutil
        p = psutil.Process(proceso.pid)
        if sys.platform == "win32":
            p.nice(psutil.HIGH_PRIORITY_CLASS)
    except Exception:
        pass

    rc = proceso.wait()
    if log_file:
        try: log_file.close()
        except Exception: pass

    if rc != 0 and progress_callback:
        crash_dir = os.path.join(game_dir, "crash-reports")
        crash_hint = ""
        if os.path.isdir(crash_dir):
            reports = sorted(
                [f for f in os.listdir(crash_dir) if f.endswith(".txt")],
                key=lambda f: os.path.getmtime(os.path.join(crash_dir, f)),
                reverse=True)
            if reports:
                crash_hint = f" | crash: {reports[0]}"
        progress_callback(f"Juego terminó con error (código {rc}){crash_hint}. Ver: {game_log_path}")
    if GestorNube is not None:
        try:
            GestorNube().subir_datos(username, options_txt_path, servers_dat_path)
        except Exception:
            pass