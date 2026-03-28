import minecraft_launcher_lib
import subprocess
import uuid
import os
import requests
import shutil
import platform
import json
import sys

def limpiar_cache_corrupto(version, minecraft_directory):
    version_dir = os.path.join(minecraft_directory, "versions", version)
    version_json = os.path.join(version_dir, f"{version}.json")
    
    if os.path.exists(version_json):
        try:
            with open(version_json, "r", encoding="utf-8") as f: json.load(f)
        except:
            shutil.rmtree(version_dir, ignore_errors=True)
    elif os.path.exists(version_dir):
        shutil.rmtree(version_dir, ignore_errors=True)

# TRADUCTOR DEFINITIVO: Soporte detallado para 1.21.11 y 1.8.8
def inyectar_logos_paraguacraft(game_dir, version, graficos_minimos):
    pack_name = "ParaguacraftBrandPack"
    pack_dir = os.path.join(game_dir, "resourcepacks", pack_name)
    textures_gui_title_dir = os.path.join(pack_dir, "assets", "minecraft", "textures", "gui", "title")
    os.makedirs(textures_gui_title_dir, exist_ok=True)

    try:
        # Extraemos la versión mayor (ej: '21' de '1.21.11')
        version_split = version.split(".")
        version_mayor = int(version_split[1])
        # Intentamos extraer la versión menor si existe (ej: '11' de '1.21.11')
        version_menor = int(version_split[2]) if len(version_split) > 2 else 0
    except Exception:
        version_mayor = 20
        version_menor = 0

    # --- FIX PACK FORMAT: Detailed translator including 1.21.11 ---
    formato = 8 # Formato por defecto seguro
    if version_mayor >= 21:
        # 1.21 y superiores
        if version_menor >= 1: formato = 34 # 1.21.1 y superiores (ej: 1.21.11)
        else: formato = 34 # 1.21.0 también usa 34 tentativamente
    elif version_mayor == 20: formato = 22
    elif version_mayor == 19: formato = 13
    elif version_mayor >= 18: formato = 9
    elif version_mayor >= 13: formato = 8 # 1.13 - 1.17
    elif version_mayor >= 9: formato = 2  # 1.9 - 1.12
    else: formato = 1                      # 1.8.8

    mcmeta = {"pack": {"pack_format": formato, "description": "Marca Oficial Paraguacraft"}}
    with open(os.path.join(pack_dir, "pack.mcmeta"), "w") as f: json.dump(mcmeta, f)

    if getattr(sys, 'frozen', False): base_path = sys._MEIPASS
    else: base_path = os.path.dirname(os.path.abspath(__file__))
    
    # Solo inyectamos fotos en versiones modernas (1.13+).
    if version_mayor >= 13:
        activos = {
            "paraguacraft_main_menu.png": "minecraft.png",
            "paraguacraft_startup.png": "mojangstudios.png"
        }
        for src_name, dest_name in activos.items():
            logo_src = os.path.join(base_path, src_name)
            logo_dest = os.path.join(textures_gui_title_dir, dest_name)
            if os.path.exists(logo_src): shutil.copyfile(logo_src, logo_dest)

    options_path = os.path.join(game_dir, "options.txt")
    lineas = []
    if os.path.exists(options_path):
        with open(options_path, "r") as f: lineas = f.readlines()
    
    lineas = [l for l in lineas if not l.startswith("resourcePacks:")]
    
    # El Pack solo se activa si hay gráficos mínimos (no requiere logo) o si es moderna.
    # En la 1.8.8 sin gráficos mínimos, no activa nada.
    if version_mayor < 13:
        packs = []
        if graficos_minimos: packs.append('"Pack_Graficos_Minimos.zip"')
        if len(packs) > 0:
            lineas.append(f'resourcePacks:[{",".join(packs)}]\n')
    else:
        packs = ['"vanilla"']
        if graficos_minimos: packs.append('"file/Pack_Graficos_Minimos.zip"')
        packs.append(f'"file/{pack_name}"')
        lineas.append(f'resourcePacks:[{",".join(packs)}]\n')
    
    with open(options_path, "w") as f: f.writelines(lineas)

def optimizar_graficos(game_dir, graficos_minimos, version):
    try:
        version_mayor = int(version.split(".")[1])
    except:
        version_mayor = 20

    options_path = os.path.join(game_dir, "options.txt")
    optionsof_path = os.path.join(game_dir, "optionsof.txt") 
    
    if graficos_minimos:
        if version_mayor < 13:
            opciones = {
                "renderDistance": "6", "fancyGraphics": "false", "ao": "0", 
                "particles": "2", "maxFps": "60", "fboEnable": "true", "enableVsync": "false"
            }
            opciones_of = {"ofFastMath": "true", "ofDynamicLights": "3", "ofSmoothFps": "false", "ofClouds": "3"}
        else:
            opciones = {
                "renderDistance": "6", "simulationDistance": "5", "graphicsMode": "FAST",
                "particles": "2", "entityDistanceScaling": "0.5", "biomeBlendRadius": "0",
                "maxFps": "60", "enableVsync": "false"
            }
            opciones_of = {}
    else:
        if version_mayor < 13:
            opciones = {
                "renderDistance": "12", "fancyGraphics": "true", "ao": "2",
                "particles": "0", "maxFps": "144", "enableVsync": "false"
            }
            opciones_of = {"ofFastMath": "false", "ofDynamicLights": "3"}
        else:
            opciones = {
                "renderDistance": "12", "simulationDistance": "8", "graphicsMode": "FANCY",
                "particles": "1", "entityDistanceScaling": "1.0", "biomeBlendRadius": "2",
                "maxFps": "144", "enableVsync": "false"
            }
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
        else:
            nuevas_lineas.append(line)
            
    for clave, valor in opciones.items():
        nuevas_lineas.append(f"{clave}:{valor}\n")
        
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
            else:
                nuevas_lineas_of.append(line)
                
        for clave, valor in opciones_of.items():
            nuevas_lineas_of.append(f"{clave}:{valor}\n")
            
        with open(optionsof_path, "w") as f: f.writelines(nuevas_lineas_of)

    config_dir = os.path.join(game_dir, "config")
    os.makedirs(config_dir, exist_ok=True)
    sodium_path = os.path.join(config_dir, "sodium-options.json")

    sodium_config = {}
    if os.path.exists(sodium_path):
        try:
            with open(sodium_path, "r") as f: sodium_config = json.load(f)
        except: pass

    if "quality" not in sodium_config: sodium_config["quality"] = {}
    if "performance" not in sodium_config: sodium_config["performance"] = {}
    
    if graficos_minimos:
        sodium_config["quality"]["particles"] = "MINIMAL"
        sodium_config["quality"]["smooth_lighting"] = "OFF"
        sodium_config["quality"]["entity_shadows"] = False
        sodium_config["quality"]["weather_quality"] = "FAST"
        sodium_config["quality"]["cloud_quality"] = "OFF"
    else:
        sodium_config["quality"]["particles"] = "DECREASED"
        sodium_config["quality"]["smooth_lighting"] = "HIGH"
        sodium_config["quality"]["entity_shadows"] = True
        sodium_config["quality"]["weather_quality"] = "FANCY"
        sodium_config["quality"]["cloud_quality"] = "FAST"

    sodium_config["performance"]["animate_only_visible_textures"] = True
    sodium_config["performance"]["use_entity_culling"] = True
    sodium_config["performance"]["use_fog_occlusion"] = True
    sodium_config["performance"]["use_block_face_culling"] = True

    with open(sodium_path, "w") as f: json.dump(sodium_config, f, indent=4)

def instalar_extras_graficos(game_dir, version, progress_callback, graficos_minimos):
    rp_dir = os.path.join(game_dir, "resourcepacks")
    sp_dir = os.path.join(game_dir, "shaderpacks")
    os.makedirs(rp_dir, exist_ok=True); os.makedirs(sp_dir, exist_ok=True)
    headers = {"User-Agent": "ParaguacraftLauncher/1.0"}

    if graficos_minimos:
        proyectos = [("f8thful", rp_dir, "Pack_Graficos_Minimos.zip")] 
        if progress_callback: progress_callback("Buscando Texturas 8x8...")
    else:
        proyectos = [("makeup-ultra-fast", sp_dir, "Shader_Paraguacraft.zip")]
        if progress_callback: progress_callback("Buscando Shaders...")

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

def instalar_mods_optimode(game_dir, version, progress_callback):
    mods_dir = os.path.join(game_dir, "mods")
    os.makedirs(mods_dir, exist_ok=True)
    slugs_mods = ["fabric-api", "sodium", "lithium", "ferrite-core", "modernfix", "iris", "modmenu", "entityculling", "immediatelyfast"]
    headers = {"User-Agent": "ParaguacraftLauncher/1.0"}
    
    try:
        if int(version.split(".")[1]) < 14: return
    except: pass

    if progress_callback: progress_callback("Buscando mods compatibles...")

    for slug in slugs_mods:
        url_api = f"https://api.modrinth.com/v2/project/{slug}/version"
        params = {"loaders": '["fabric"]', "game_versions": f'["{version}"]'}
        try:
            r = requests.get(url_api, params=params, headers=headers, timeout=5)
            if r.status_code == 200:
                data = r.json()
                if len(data) > 0:
                    archivo = data[0]["files"][0]
                    mod_path = os.path.join(mods_dir, archivo["filename"])
                    if not os.path.exists(mod_path):
                        r_dl = requests.get(archivo["url"], stream=True, headers=headers, timeout=10)
                        if r_dl.status_code == 200:
                            with open(mod_path, "wb") as f: shutil.copyfileobj(r_dl.raw, f)
        except Exception: pass

def lanzar_minecraft(version="1.20.4", username="Player", max_ram="4G", gc_type="G1GC", optimizar=False, optimode=False, papa_mode=False, usar_mesa=False, mostrar_consola=False, progress_callback=None, uuid_real=None, token_real=None):
    minecraft_directory = minecraft_launcher_lib.utils.get_minecraft_directory()
    
    def on_progress(event):
        if progress_callback: progress_callback(event)

    limpiar_cache_corrupto(version, minecraft_directory)
    version_a_lanzar = version

    soporta_fabric = True
    try:
        if int(version.split(".")[1]) < 14: soporta_fabric = False
    except: pass

    if optimode and soporta_fabric:
        if progress_callback: progress_callback("Instalando Fabric (Motor de mods)...")
        try:
            loader_nuevo = minecraft_launcher_lib.fabric.get_latest_loader_version()
            limpiar_cache_corrupto(f"fabric-loader-{loader_nuevo}-{version}", minecraft_directory)
            minecraft_launcher_lib.fabric.install_fabric(version, minecraft_directory, loader_nuevo, callback={"setStatus": on_progress})
            version_a_lanzar = f"fabric-loader-{loader_nuevo}-{version}"
        except Exception:
            version_a_lanzar = version
    else:
        if "OptiFine" not in version:
            if progress_callback: progress_callback("Instalando versión base...")
            minecraft_launcher_lib.install.install_minecraft_version(version, minecraft_directory, callback={"setStatus": on_progress})

    folder_name = f"Paraguacraft_{version_a_lanzar}".replace(".", "_").replace("-", "_")
    game_dir = os.path.join(minecraft_directory, "instancias", folder_name)
    os.makedirs(game_dir, exist_ok=True)

    if optimode and soporta_fabric:
        instalar_mods_optimode(game_dir, version, progress_callback)
    
    instalar_extras_graficos(game_dir, version, progress_callback, optimizar)

    inyectar_logos_paraguacraft(game_dir, version, optimizar)
    optimizar_graficos(game_dir, optimizar, version)

    jvm_arguments = [f"-Xmx{max_ram}", f"-Xms{max_ram}", "-XX:+UnlockExperimentalVMOptions", "-XX:+DisableExplicitGC", "-XX:+AlwaysPreTouch"]
    
    if gc_type == "G1GC":
        jvm_arguments.extend([
            "-XX:+UseG1GC", "-XX:G1NewSizePercent=30", "-XX:G1MaxNewSizePercent=40", "-XX:G1HeapRegionSize=8M",
            "-XX:G1ReservePercent=20", "-XX:G1HeapWastePercent=5", "-XX:G1MixedGCCountTarget=4",
            "-XX:InitiatingHeapOccupancyPercent=15", "-XX:G1MixedGCLiveThresholdPercent=90",
            "-XX:G1RSetUpdatingPauseTimePercent=5", "-XX:SurvivorRatio=32", "-XX:+PerfDisableSharedMem",
            "-XX:MaxTenuringThreshold=1", "-Dusing.aikars.flags=https://mcflags.emc.gs", "-Daikars.new.flags=true"
        ])
    elif gc_type == "ZGC": jvm_arguments.append("-XX:+UseZGC")
    elif gc_type == "Shenandoah": jvm_arguments.append("-XX:+UseShenandoahGC")
    elif gc_type == "CMS": jvm_arguments.append("-XX:+UseConcMarkSweepGC")

    options = {
        "username": username,
        "uuid": uuid_real if uuid_real else str(uuid.uuid4()),
        "token": token_real if token_real else "",
        "jvmArguments": jvm_arguments,
        "gameDirectory": game_dir
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

    if mostrar_consola and platform.system() == "Windows": 
        flags_creacion = subprocess.CREATE_NEW_CONSOLE
        salida = None
    else:
        flags_creacion = subprocess.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
        salida = subprocess.DEVNULL 

    if progress_callback: progress_callback("¡Abriendo Paraguacraft!")
    
    proceso = subprocess.Popen(command, env=entorno, creationflags=flags_creacion, stdout=salida, stderr=salida, stdin=subprocess.DEVNULL)
    proceso.wait()

if __name__ == "__main__":
    pass