import minecraft_launcher_lib
import subprocess
import uuid
import os
import requests
import shutil
import platform
import json
import sys

def inyectar_logos_paraguacraft(minecraft_directory, version):
    pack_name = "ParaguacraftBrandPack"
    pack_dir = os.path.join(minecraft_directory, "resourcepacks", pack_name)
    textures_gui_title_dir = os.path.join(pack_dir, "assets", "minecraft", "textures", "gui", "title")
    os.makedirs(textures_gui_title_dir, exist_ok=True)

    formato = 8
    if "1.21" in version:
        formato = 34
    elif "1.20.4" in version:
        formato = 22
    elif "1.20.1" in version:
        formato = 15

    mcmeta = {"pack": {"pack_format": formato, "description": "Marca Oficial Paraguacraft"}}
    with open(os.path.join(pack_dir, "pack.mcmeta"), "w") as f:
        json.dump(mcmeta, f)

    if getattr(sys, 'frozen', False):
        base_path = sys._MEIPASS 
    else:
        base_path = os.path.dirname(os.path.abspath(__file__))
    
    activos = {
        "paraguacraft_main_menu.png": "minecraft.png", 
        "paraguacraft_startup.png": "mojangstudios.png" 
    }

    exito = True
    for src_name, dest_name in activos.items():
        logo_src = os.path.join(base_path, src_name)
        logo_dest = os.path.join(textures_gui_title_dir, dest_name)
        if os.path.exists(logo_src):
            shutil.copyfile(logo_src, logo_dest)
        else:
            exito = False

    if not exito: return

    options_path = os.path.join(minecraft_directory, "options.txt")
    if os.path.exists(options_path):
        with open(options_path, "r") as f:
            lines = f.readlines()
        
        found = False
        with open(options_path, "w") as f:
            for line in lines:
                if line.startswith("resourcePacks:"):
                    f.write(f'resourcePacks:["vanilla","file/{pack_name}"]\n')
                    found = True
                else:
                    f.write(line)
            if not found:
                 f.write(f'resourcePacks:["vanilla","file/{pack_name}"]\n')

def optimizar_graficos(minecraft_directory):
    options_path = os.path.join(minecraft_directory, "options.txt")
    if os.path.exists(options_path):
        with open(options_path, "r") as f:
            lineas = f.readlines()
        with open(options_path, "w") as f:
            for linea in lineas:
                if linea.startswith("renderDistance:"): f.write("renderDistance:6\n")
                elif linea.startswith("graphicsMode:"): f.write("graphicsMode:FAST\n")
                elif linea.startswith("particles:"): f.write("particles:2\n")
                elif linea.startswith("entityDistanceScaling:"): f.write("entityDistanceScaling:0.5\n")
                elif linea.startswith("biomeBlendRadius:"): f.write("biomeBlendRadius:0\n")
                else: f.write(linea)

# --- SISTEMA INTELIGENTE DE MODS CON LA API DE MODRINTH ---
def instalar_mods_optimode(minecraft_directory, version, progress_callback):
    mods_dir = os.path.join(minecraft_directory, "mods")
    os.makedirs(mods_dir, exist_ok=True)
    
    slugs_mods = ["fabric-api", "sodium", "lithium", "ferrite-core", "modernfix", "iris", "modmenu"]
    headers = {"User-Agent": "ParaguacraftLauncher/1.0 (contacto@paraguacraft.com)"}
    
    if progress_callback: progress_callback(f"Buscando mods perfectos para {version}...")

    for slug in slugs_mods:
        # Le preguntamos a la base de datos por los archivos exactos para tu versión
        url_api = f"https://api.modrinth.com/v2/project/{slug}/version"
        params = {
            "loaders": '["fabric"]',
            "game_versions": f'["{version}"]'
        }
        
        try:
            r = requests.get(url_api, params=params, headers=headers, timeout=10)
            if r.status_code == 200:
                data = r.json()
                if len(data) > 0:
                    # Obtenemos el link dinámico más reciente que sí funciona
                    archivo = data[0]["files"][0]
                    download_url = archivo["url"]
                    filename = archivo["filename"]
                    
                    mod_path = os.path.join(mods_dir, filename)
                    
                    if not os.path.exists(mod_path):
                        if progress_callback: progress_callback(f"Descargando {filename}...")
                        r_dl = requests.get(download_url, stream=True, headers=headers, timeout=15)
                        if r_dl.status_code == 200:
                            with open(mod_path, "wb") as f:
                                shutil.copyfileobj(r_dl.raw, f)
        except Exception as e:
            pass

def lanzar_minecraft(version="1.20.4", username="Player", max_ram="4G", gc_type="G1GC", optimizar=False, optimode=False, papa_mode=False, usar_mesa=False, mostrar_consola=False, progress_callback=None):
    minecraft_directory = minecraft_launcher_lib.utils.get_minecraft_directory()
    
    def on_progress(event):
        if progress_callback: progress_callback(event)

    version_a_lanzar = version

    if optimode:
        if progress_callback: progress_callback("Instalando Fabric (Motor de mods)...")
        
        # 1. Le pedimos a la librería que averigüe cuál es el Fabric Loader más nuevo de todos
        loader_nuevo = minecraft_launcher_lib.fabric.get_latest_loader_version()
        
        # 2. Instalamos esa versión exacta que encontró
        minecraft_launcher_lib.fabric.install_fabric(version, minecraft_directory, loader_nuevo, callback={"setStatus": on_progress})
        
        instalar_mods_optimode(minecraft_directory, version, progress_callback)
        
        # 3. Armamos el nombre perfecto para abrir el juego
        version_a_lanzar = f"fabric-loader-{loader_nuevo}-{version}"
    else:
        minecraft_launcher_lib.install.install_minecraft_version(version, minecraft_directory, callback={"setStatus": on_progress})

    inyectar_logos_paraguacraft(minecraft_directory, version)
    
    if optimizar: optimizar_graficos(minecraft_directory)

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
        "jvmArguments": jvm_arguments
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

    flags_creacion = 0
    if mostrar_consola and platform.system() == "Windows":
        flags_creacion = subprocess.CREATE_NEW_CONSOLE

    if progress_callback: progress_callback("¡Abriendo Paraguacraft!")
    proceso = subprocess.Popen(command, env=entorno, creationflags=flags_creacion)
    proceso.wait()

if __name__ == "__main__":
    pass