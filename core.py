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
import os
from src.nube import GestorNube

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

def inyectar_logos_paraguacraft(game_dir, version, graficos_minimos):
    pack_name = "ParaguacraftBrandPack"
    pack_dir = os.path.join(game_dir, "resourcepacks", pack_name)
    textures_gui_title_dir = os.path.join(pack_dir, "assets", "minecraft", "textures", "gui", "title")
    os.makedirs(textures_gui_title_dir, exist_ok=True)

    def _copy_logo_minecraft_png(src_path, dest_path, version_mayor_local):
        if not os.path.exists(src_path):
            return
        if 16 <= version_mayor_local <= 20:
            try:
                from PIL import Image
                import zipfile as _zf, io as _io

                im = Image.open(src_path)
                if im.mode != "RGBA":
                    im = im.convert("RGBA")

                # --- Determinar dimensiones exactas del PNG vanilla desde el JAR ---
                # Esto garantiza que nuestro PNG de reemplazo tenga el mismo tamaño
                # que espera Minecraft, sin importar la versión exacta (1.16-1.20).
                target_w, target_h = 256, 64  # fallback razonable
                try:
                    mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                    jar = os.path.join(mc_dir, "versions", version, version + ".jar")
                    with _zf.ZipFile(jar, "r") as z:
                        with z.open("assets/minecraft/textures/gui/title/minecraft.png") as f:
                            van = Image.open(_io.BytesIO(f.read()))
                            target_w, target_h = van.size
                except Exception:
                    pass

                # Minecraft 1.16-1.20 renders the logo with 2 blit calls:
                #   blit(k,   l, u=0, v=0,  155, 44) → screen LEFT,  reads texture[x=0..154, y=0..43]
                #   blit(k+155, l, u=0, v=45, 155, 44) → screen RIGHT, reads texture[x=0..154, y=45..88]
                # Total displayed area = 310 × 44 px.
                # Fix: scale the logo to 310×44, put LEFT 155px in row1 and RIGHT 155px in row2.
                HALF_W = 155
                RENDER_H = 44
                ROW2_Y = 45

                w, h = im.size
                if w <= 0 or h <= 0:
                    raise ValueError("invalid image")

                # Scale to fill the combined 310×44 display area
                scale = min((HALF_W * 2) / w, RENDER_H / h)
                new_w = max(1, int(round(w * scale)))
                new_h = max(1, int(round(h * scale)))
                logo_scaled = im.resize((new_w, new_h), Image.LANCZOS)

                # Full 310×44 canvas (logo centred)
                canvas = Image.new("RGBA", (HALF_W * 2, RENDER_H), (0, 0, 0, 0))
                px = (HALF_W * 2 - new_w) // 2
                py = (RENDER_H - new_h) // 2
                canvas.paste(logo_scaled, (px, py), logo_scaled)

                out = Image.new("RGBA", (target_w, target_h), (0, 0, 0, 0))
                # Row 1 (y=0..43): left 155 px → blit reads this for the left screen half
                out.paste(canvas.crop((0, 0, HALF_W, RENDER_H)), (0, 0))
                # Row 2 (y=45+): right 155 px → blit reads this for the right screen half
                out.paste(canvas.crop((HALF_W, 0, HALF_W * 2, RENDER_H)), (0, ROW2_Y))

                out.save(dest_path)
                print(f"[Paraguacraft] Logo legacy generado: {dest_path} ({target_w}x{target_h})")
                return
            except Exception as e:
                print(f"[Paraguacraft] Error logo legacy 1.16-1.20: {e}. Copia directa.")
        shutil.copyfile(src_path, dest_path)

    try:
        version_split = version.split(".")
        if version_split[0] == "26":
            version_mayor = 26
        else:
            version_mayor = int(version_split[1])
    except Exception:
        version_mayor = 20

    formato = 8
    if version_mayor >= 21 or version_mayor == 26: formato = 34
    elif version_mayor == 20: formato = 22
    elif version_mayor == 19: formato = 9
    elif version_mayor == 18: formato = 8
    elif version_mayor == 17: formato = 7
    elif version_mayor == 16: formato = 6
    elif version_mayor >= 13: formato = 8
    elif version_mayor >= 9: formato = 2
    else: formato = 1

    pack_block = {"pack_format": formato, "description": "Marca Oficial Paraguacraft"}
    if version_mayor >= 20:
        pack_block["supported_formats"] = {"min_inclusive": 8, "max_inclusive": 99}
    mcmeta = {"pack": pack_block}
    with open(os.path.join(pack_dir, "pack.mcmeta"), "w") as f: json.dump(mcmeta, f)

    if getattr(sys, 'frozen', False): base_path = sys._MEIPASS
    else: base_path = os.path.dirname(os.path.abspath(__file__))

    # Marca en menú: 1.16.1 en adelante (incluye 26.x por versión_mayor=26)
    aplica_marca_menu = version_mayor >= 16
    if aplica_marca_menu:
        activos = {
            "paraguacraft_main_menu.png": "minecraft.png",
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

    options_path = os.path.join(game_dir, "options.txt")
    lineas = []
    if os.path.exists(options_path):
        with open(options_path, "r") as f: lineas = f.readlines()
    
    lineas = [l for l in lineas if not l.startswith("resourcePacks:")]
    
    if version_mayor < 13:
        packs = []
        if graficos_minimos: packs.append('"Pack_Graficos_Minimos.zip"')
        if len(packs) > 0: lineas.append(f'resourcePacks:[{",".join(packs)}]\n')
    else:
        packs = ['"vanilla"']
        if graficos_minimos: packs.append('"file/Pack_Graficos_Minimos.zip"')
        if aplica_marca_menu:
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
    
    if graficos_minimos and os.path.exists(os.path.join(rp_dir, "Pack_Graficos_Minimos.zip")): return
    if not graficos_minimos and os.path.exists(os.path.join(sp_dir, "Shader_Paraguacraft.zip")): return

    headers = {"User-Agent": "ParaguacraftLauncher/1.0"}
    if graficos_minimos:
        proyectos = [("faithful-32x", rp_dir, "Pack_Graficos_Minimos.zip")] 
        if progress_callback: progress_callback("Buscando Texturas Faithful 32x...")
    else:
        proyectos = [("makeup-ultra-fast", sp_dir, "Shader_Paraguacraft.zip")]
        if progress_callback: progress_callback("Buscando Shaders MakeUp...")

    for slug, carpeta_destino, nombre_final in proyectos:
        url_api = f"https://api.modrinth.com/v2/project/{slug}/version"
        params = {"game_versions": f'["{version}"]'}
        try:
            r = requests.get(url_api, params=params, headers=headers, timeout=5)
            if r.status_code == 200:
                data = r.json()
                if len(data) > 0:
                    archivo = data[0]["files"][0]
                    ruta_archivo = os.path.join(carpeta_destino, nombre_final)
                    if not os.path.exists(ruta_archivo):
                        if progress_callback: progress_callback(f"Descargando mejoras visuales...")
                        r_dl = requests.get(archivo["url"], stream=True, headers=headers, timeout=10)
                        if r_dl.status_code == 200:
                            with open(ruta_archivo, "wb") as f: shutil.copyfileobj(r_dl.raw, f)
        except Exception: pass

def _descargar_mods_fabric_slugs(mods_dir, version, slugs, progress_callback, headers):
    os.makedirs(mods_dir, exist_ok=True)
    for slug in slugs:
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

def instalar_bundle_fabric_iris_cache(minecraft_directory, game_dir, version, progress_callback, lan_distancia=False):
    """
    Caché global por versión de MC: los .jar se descargan una sola vez y se copian a la instancia.
    Así no se re-descargan al cambiar de perfil y se evitan mezclas raras con otras descargas.
    """
    cache_root = os.path.join(minecraft_directory, "Paraguacraft_cache", "fabric_iris", version.replace(os.sep, "_"))
    os.makedirs(cache_root, exist_ok=True)
    mods_dir = os.path.join(game_dir, "mods")
    os.makedirs(mods_dir, exist_ok=True)

    slugs = ["fabric-api", "sodium", "iris", "lithium", "ferrite-core", "entityculling", "immediatelyfast", "modmenu"]
    if lan_distancia:
        slugs.append("e4mc")

    headers = {"User-Agent": "ParaguacraftLauncher/2.1"}
    if progress_callback:
        progress_callback("Comprobando caché Fabric + Iris...")

    _descargar_mods_fabric_slugs(cache_root, version, slugs, progress_callback, headers)

    for f in os.listdir(cache_root):
        if not f.endswith(".jar"):
            continue
        dest = os.path.join(mods_dir, f)
        if not os.path.exists(dest) and not os.path.exists(dest + ".disabled"):
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

    if motor_lower == "vanilla" or motor_lower.startswith("optifine"):
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

def lanzar_minecraft(version="1.20.4", username="Player", max_ram="4G", gc_type="G1GC", optimizar=False, motor_elegido="Vanilla", papa_mode=False, usar_mesa=False, mostrar_consola=False, progress_callback=None, uuid_real=None, token_real=None, lan_distancia=False, fabric_loader_override=None):
    minecraft_directory = minecraft_launcher_lib.utils.get_minecraft_directory()
    
    def on_progress(event):
        if progress_callback: progress_callback(event)

    version_base = version.strip()
    
    # 1. Determinamos el Loader base (Fabric, Forge o Vanilla) leyendo el nombre del motor
    motor_lower = motor_elegido.lower()
    if "fabric" in motor_lower:
        tipo_cliente_base = "Fabric"
    elif "forge" in motor_lower:
        tipo_cliente_base = "Forge"
    else:
        tipo_cliente_base = "Vanilla"

    version_a_lanzar = version_base

    # --- BYPASS DE ARRANQUE RÁPIDO ---
    version_instalada = False
    if tipo_cliente_base == "Fabric":
        versiones_locales = os.listdir(os.path.join(minecraft_directory, "versions")) if os.path.exists(os.path.join(minecraft_directory, "versions")) else []
        for v in versiones_locales:
            if v.startswith("fabric-loader-") and version_base in v:
                version_a_lanzar = v
                version_instalada = True
                break
    else:
        ruta_version = os.path.join(minecraft_directory, "versions", version_base)
        if os.path.exists(ruta_version): version_instalada = True

    # Si NO está instalada, ejecutamos el motor de descarga
    if not version_instalada:
        if tipo_cliente_base == "Fabric":
            if progress_callback: progress_callback("Instalando Fabric (Motor base)...")
            loader_nuevo = fabric_loader_override or minecraft_launcher_lib.fabric.get_latest_loader_version()
            minecraft_launcher_lib.fabric.install_fabric(version_base, minecraft_directory, loader_nuevo, callback={"setStatus": on_progress})
            version_a_lanzar = f"fabric-loader-{loader_nuevo}-{version_base}"
            
        elif tipo_cliente_base == "Forge":
            if progress_callback: progress_callback("Instalando Forge...")
            forge_version = minecraft_launcher_lib.forge.find_forge_version(version_base)
            if forge_version:
                minecraft_launcher_lib.forge.install_forge_version(forge_version, minecraft_directory, callback={"setStatus": on_progress})
                version_a_lanzar = forge_version
        else:
            if progress_callback: progress_callback(f"Descargando juego base {version_base}...")
            minecraft_launcher_lib.install.install_minecraft_version(version_base, minecraft_directory, callback={"setStatus": on_progress})

    # 2. AISLAMIENTO DE INSTANCIAS DINÁMICO
    folder_name = carpeta_instancia_paraguacraft(version_base, motor_elegido)
    
    game_dir = os.path.join(minecraft_directory, "instancias", folder_name)
    os.makedirs(game_dir, exist_ok=True)
    os.makedirs(os.path.join(game_dir, "server-resource-packs"), exist_ok=True)

    # 3. MODS Y GRÁFICOS (Llamamos a la nueva función inteligente)
    instalar_mods_por_motor(game_dir, version_base, motor_elegido, progress_callback, lan_distancia, minecraft_directory)
        
    instalar_extras_graficos(game_dir, version_base, progress_callback, optimizar)
    inyectar_logos_paraguacraft(game_dir, version_base, optimizar)
    if optimizar: 
        optimizar_graficos(game_dir, optimizar, version_base)

    # (A PARTIR DE ACÁ SIGUE TU CÓDIGO NORMAL CON jvm_arguments Y GestorNube)
    # 4. ARGUMENTOS DE JAVA
    jvm_arguments = [f"-Xmx{max_ram}", f"-Xms{max_ram}", "-XX:+UnlockExperimentalVMOptions", "-XX:+DisableExplicitGC", "-XX:+AlwaysPreTouch"]
    if gc_type == "G1GC":
        jvm_arguments.extend(["-XX:+UseG1GC", "-XX:G1NewSizePercent=30", "-XX:G1MaxNewSizePercent=40", "-XX:G1HeapRegionSize=8M", "-XX:G1ReservePercent=20", "-XX:G1HeapWastePercent=5", "-XX:G1MixedGCCountTarget=4", "-XX:InitiatingHeapOccupancyPercent=15", "-XX:G1MixedGCLiveThresholdPercent=90", "-XX:G1RSetUpdatingPauseTimePercent=5", "-XX:SurvivorRatio=32", "-XX:+PerfDisableSharedMem", "-XX:MaxTenuringThreshold=1", "-Dusing.aikars.flags=https://mcflags.emc.gs", "-Daikars.new.flags=true"])
    elif gc_type == "ZGC": jvm_arguments.append("-XX:+UseZGC")
    elif gc_type == "Shenandoah": jvm_arguments.append("-XX:+UseShenandoahGC")
    elif gc_type == "CMS": jvm_arguments.append("-XX:+UseConcMarkSweepGC")

    # 5. CONFIGURACIÓN FINAL DEL JUEGO
    options = {
        "username": username,
        "uuid": uuid_real if uuid_real else str(uuid.uuid4()),
        "token": token_real if token_real else "",
        "jvmArguments": jvm_arguments,
        "gameDirectory": game_dir # <- Acá está la magia de los perfiles aislados
    }

    if papa_mode:
        options["customResolution"] = True
        options["resolutionWidth"] = "800"
        options["resolutionHeight"] = "600"

    command = minecraft_launcher_lib.command.get_minecraft_command(version_a_lanzar, minecraft_directory, options)

    entorno = os.environ.copy()
    if usar_mesa:
        entorno["MESA_GL_VERSION_OVERRIDE"] = "3.3"
        entorno["MESA_GLSL_VERSION_OVERRIDE"] = "330"

    gestor_nube = GestorNube()
    options_txt_path = os.path.join(game_dir, "options.txt")
    servers_dat_path = os.path.join(game_dir, "servers.dat")

    if progress_callback:
        progress_callback("☁️ Sincronizando datos de la nube...")
    gestor_nube.descargar_datos(username, options_txt_path, servers_dat_path)

    # La nube puede sobrescribir options.txt sin el resource pack; lo re-inyectamos.
    inyectar_logos_paraguacraft(game_dir, version_base, optimizar)

    if progress_callback:
        progress_callback("¡Abriendo Paraguacraft!")

    if mostrar_consola and platform.system() == "Windows":
        flags_creacion = subprocess.CREATE_NEW_CONSOLE
        salida = None
    else:
        flags_creacion = subprocess.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
        salida = subprocess.DEVNULL

    proceso = subprocess.Popen(command, env=entorno, creationflags=flags_creacion, stdout=salida, stderr=salida, stdin=subprocess.DEVNULL)

    try:
        import psutil
        p = psutil.Process(proceso.pid)
        if sys.platform == "win32":
            p.nice(psutil.HIGH_PRIORITY_CLASS)
    except Exception:
        pass

    proceso.wait()
    gestor_nube.subir_datos(username, options_txt_path, servers_dat_path)