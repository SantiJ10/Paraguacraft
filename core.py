import minecraft_launcher_lib
import subprocess
import uuid
import os
import requests
import shutil
import platform
import json
import sys

def inyectar_logos_paraguacraft(game_dir, version):
    pack_name = "ParaguacraftBrandPack"
    pack_dir = os.path.join(game_dir, "resourcepacks", pack_name)
    textures_gui_title_dir = os.path.join(pack_dir, "assets", "minecraft", "textures", "gui", "title")
    os.makedirs(textures_gui_title_dir, exist_ok=True)

    formato = 8
    if "1.21" in version: formato = 34
    elif "1.20.4" in version: formato = 22
    elif "1.20.1" in version: formato = 15

    mcmeta = {"pack": {"pack_format": formato, "description": "Marca Oficial Paraguacraft"}}
    with open(os.path.join(pack_dir, "pack.mcmeta"), "w") as f:
        json.dump(mcmeta, f)

    if getattr(sys, 'frozen', False): base_path = sys._MEIPASS
    else: base_path = os.path.dirname(os.path.abspath(__file__))
    
    activos = {
        "paraguacraft_main_menu.png": "minecraft.png",
        "paraguacraft_startup.png": "mojangstudios.png"
    }

    exito = True
    for src_name, dest_name in activos.items():
        logo_src = os.path.join(base_path, src_name)
        logo_dest = os.path.join(textures_gui_title_dir, dest_name)
        if os.path.exists(logo_src): shutil.copyfile(logo_src, logo_dest)
        else: exito = False

    if not exito: return

    options_path = os.path.join(game_dir, "options.txt")
    if os.path.exists(options_path):
        with open(options_path, "r") as f: lines = f.readlines()
        
        found = False
        with open(options_path, "w") as f:
            for line in lines:
                if line.startswith("resourcePacks:"):
                    f.write(f'resourcePacks:["vanilla","file/{pack_name}"]\n')
                    found = True
                else: f.write(line)
            if not found: f.write(f'resourcePacks:["vanilla","file/{pack_name}"]\n')

def optimizar_graficos(game_dir):
    options_path = os.path.join(game_dir, "options.txt")
    if os.path.exists(options_path):
        with open(options_path, "r") as f: lineas = f.readlines()
        with open(options_path, "w") as f:
            for linea in lineas:
                if linea.startswith("renderDistance:"): f.write("renderDistance:7\n")
                elif linea.startswith("simulationDistance:"): f.write("simulationDistance:5\n")
                elif linea.startswith("graphicsMode:"): f.write("graphicsMode:FAST\n")
                elif linea.startswith("particles:"): f.write("particles:2\n")
                elif linea.startswith("entityDistanceScaling:"): f.write("entityDistanceScaling:0.5\n")
                elif linea.startswith("biomeBlendRadius:"): f.write("biomeBlendRadius:0\n")
                elif linea.startswith("maxFps:"): f.write("maxFps:75\n")
                elif linea.startswith("enableVsync:"): f.write("enableVsync:false\n")
                else: f.write(linea)

    config_dir = os.path.join(game_dir, "config")
    os.makedirs(config_dir, exist_ok=True)
    sodium_path = os.path.join(config_dir, "sodium-options.json")

    sodium_config = {}
    if os.path.exists(sodium_path):
        try:
            with open(sodium_path, "r") as f: sodium_config = json.load(f)
        except: pass

    if "quality" not in sodium_config: sodium_config["quality"] = {}
    sodium_config["quality"]["particles"] = "MINIMAL"
    sodium_config["quality"]["smooth_lighting"] = "OFF"
    sodium_config["quality"]["entity_shadows"] = False
    sodium_config["quality"]["vignette"] = False
    sodium_config["quality"]["weather_quality"] = "FAST"
    sodium_config["quality"]["mipmap_levels"] = 0
    sodium_config["quality"]["cloud_quality"] = "OFF"

    if "performance" not in sodium_config: sodium_config["performance"] = {}
    sodium_config["performance"]["animate_only_visible_textures"] = True
    sodium_config["performance"]["use_entity_culling"] = True
    sodium_config["performance"]["use_fog_occlusion"] = True
    sodium_config["performance"]["use_block_face_culling"] = True

    with open(sodium_path, "w") as f: json.dump(sodium_config, f, indent=4)

def instalar_extras_graficos(game_dir, version, progress_callback, graficos_minimos):
    rp_dir = os.path.join(game_dir, "resourcepacks")
    sp_dir = os.path.join(game_dir, "shaderpacks")
    os.makedirs(rp_dir, exist_ok=True)
    os.makedirs(sp_dir, exist_ok=True)

    headers = {"User-Agent": "ParaguacraftLauncher/1.0"}

    if graficos_minimos:
        proyectos = [("f8thful", rp_dir)] 
        if progress_callback: progress_callback("Instalando Texturas 8x8 (Boost FPS)...")
    else:
        proyectos = [("makeup-ultra-fast", sp_dir)]
        if progress_callback: progress_callback("Instalando Shaders Ultra Rápidos...")

    for slug, carpeta_destino in proyectos:
        url_api = f"https://api.modrinth.com/v2/project/{slug}/version"
        params = {"game_versions": f'["{version}"]'}
        
        try:
            r = requests.get(url_api, params=params, headers=headers, timeout=10)
            if r.status_code == 200:
                data = r.json()
                if len(data) > 0:
                    archivo = data[0]["files"][0]
                    ruta_archivo = os.path.join(carpeta_destino, archivo["filename"])
                    
                    if not os.path.exists(ruta_archivo):
                        if progress_callback: progress_callback(f"Descargando {archivo['filename']}...")
                        r_dl = requests.get(archivo["url"], stream=True, headers=headers, timeout=15)
                        if r_dl.status_code == 200:
                            with open(ruta_archivo, "wb") as f:
                                shutil.copyfileobj(r_dl.raw, f)
        except Exception as e:
            pass

def instalar_mods_optimode(game_dir, version, progress_callback):
    mods_dir = os.path.join(game_dir, "mods")
    os.makedirs(mods_dir, exist_ok=True)
    
    slugs_mods = [
        "fabric-api", "sodium", "lithium", "ferrite-core", 
        "modernfix", "iris", "modmenu", "entityculling", "immediatelyfast"
    ]
    headers = {"User-Agent": "ParaguacraftLauncher/1.0"}
    
    if progress_callback: progress_callback(f"Buscando mods de ultra-optimización para {version}...")

    for slug in slugs_mods:
        url_api = f"https://api.modrinth.com/v2/project/{slug}/version"
        params = {"loaders": '["fabric"]', "game_versions": f'["{version}"]'}
        
        try:
            r = requests.get(url_api, params=params, headers=headers, timeout=10)
            if r.status_code == 200:
                data = r.json()
                if len(data) > 0:
                    archivo = data[0]["files"][0]
                    mod_path = os.path.join(mods_dir, archivo["filename"])
                    
                    if not os.path.exists(mod_path):
                        if progress_callback: progress_callback(f"Descargando {archivo['filename']}...")
                        r_dl = requests.get(archivo["url"], stream=True, headers=headers, timeout=15)
                        if r_dl.status_code == 200:
                            with open(mod_path, "wb") as f:
                                shutil.copyfileobj(r_dl.raw, f)
        except: pass

def lanzar_minecraft(version="1.20.4", username="Player", max_ram="4G", gc_type="G1GC", optimizar=False, optimode=False, papa_mode=False, usar_mesa=False, mostrar_consola=False, progress_callback=None):
    base_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
    
    def on_progress(event):
        if progress_callback: progress_callback(event)

    # 1. GESTOR AUTOMÁTICO DE JAVA (Tus amigos no necesitan instalar Java, el launcher lo hace)
    if progress_callback: progress_callback("Verificando motor Java (Automático)...")
    minecraft_launcher_lib.install.install_jvm_runtime(version, base_dir, callback={"setStatus": on_progress})

    version_a_lanzar = version

    if optimode:
        if progress_callback: progress_callback("Preparando el Motor de Paraguacraft...")
        loader_nuevo = minecraft_launcher_lib.fabric.get_latest_loader_version()
        minecraft_launcher_lib.fabric.install_fabric(version, base_dir, loader_nuevo, callback={"setStatus": on_progress})
        version_a_lanzar = f"fabric-loader-{loader_nuevo}-{version}"
    else:
        minecraft_launcher_lib.install.install_minecraft_version(version, base_dir, callback={"setStatus": on_progress})

    # 2. AISLAMIENTO DE INSTANCIAS (Carpeta única para esta versión)
    game_dir = os.path.join(base_dir, "instancias", f"Paraguacraft_{version_a_lanzar}")
    os.makedirs(game_dir, exist_ok=True)

    if optimode:
        instalar_mods_optimode(game_dir, version, progress_callback)
        instalar_extras_graficos(game_dir, version, progress_callback, optimizar)

    inyectar_logos_paraguacraft(game_dir, version)
    if optimizar: optimizar_graficos(game_dir)

    jvm_arguments = [
        f"-Xmx{max_ram}", f"-Xms{max_ram}",
        "-XX:+UnlockExperimentalVMOptions", "-XX:+DisableExplicitGC", "-XX:+AlwaysPreTouch"
    ]
    
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
        "uuid": str(uuid.uuid4()),
        "token": "",
        "jvmArguments": jvm_arguments,
        "gameDirectory": game_dir  # ¡LA MAGIA DEL AISLAMIENTO ESTÁ ACÁ!
    }

    if papa_mode:
        options["customResolution"] = True
        options["resolutionWidth"] = "800"
        options["resolutionHeight"] = "600"

    command = minecraft_launcher_lib.command.get_minecraft_command(version_a_lanzar, base_dir, options)

    entorno = os.environ.copy()
    if usar_mesa:
        entorno["MESA_GL_VERSION_OVERRIDE"] = "3.3"
        entorno["MESA_GLSL_VERSION_OVERRIDE"] = "330"

    flags_creacion = 0
    if mostrar_consola and platform.system() == "Windows": flags_creacion = subprocess.CREATE_NEW_CONSOLE

    if progress_callback: progress_callback("¡Despegando Paraguacraft!")
    proceso = subprocess.Popen(command, env=entorno, creationflags=flags_creacion)
    proceso.wait()

if __name__ == "__main__":
    pass