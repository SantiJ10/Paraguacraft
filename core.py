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
    version_jar = os.path.join(version_dir, f"{version}.jar")
    
    if not os.path.exists(version_dir): return
    
    if not os.path.exists(version_json):
        shutil.rmtree(version_dir, ignore_errors=True)
        return
        
    try:
        with open(version_json, "r", encoding="utf-8") as f: data = json.load(f)
    except:
        shutil.rmtree(version_dir, ignore_errors=True)
        return
        
    if "inheritsFrom" not in data:
        if not os.path.exists(version_jar):
            shutil.rmtree(version_dir, ignore_errors=True)
            return

def inyectar_logos_paraguacraft(game_dir, version, graficos_minimos):
    pack_name = "ParaguacraftBrandPack"
    pack_dir = os.path.join(game_dir, "resourcepacks", pack_name)
    textures_gui_title_dir = os.path.join(pack_dir, "assets", "minecraft", "textures", "gui", "title")
    os.makedirs(textures_gui_title_dir, exist_ok=True)

    try:
        version_split = version.split(".")
        version_mayor = int(version_split[1])
    except Exception:
        version_mayor = 20

    formato = 8 
    if version_mayor >= 21: formato = 34 
    elif version_mayor == 20: formato = 22
    elif version_mayor >= 13: formato = 8 
    elif version_mayor >= 9: formato = 2  
    else: formato = 1                      

    mcmeta = {
        "pack": {
            "pack_format": formato, 
            "supported_formats": {"min_inclusive": 8, "max_inclusive": 99},
            "description": "Marca Oficial Paraguacraft"
        }
    }
    with open(os.path.join(pack_dir, "pack.mcmeta"), "w") as f: json.dump(mcmeta, f)

    if getattr(sys, 'frozen', False): base_path = sys._MEIPASS
    else: base_path = os.path.dirname(os.path.abspath(__file__))
    
    if version_mayor >= 20:
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
    
    if version_mayor < 13:
        packs = []
        if graficos_minimos: packs.append('"Pack_Graficos_Minimos.zip"')
        if len(packs) > 0: lineas.append(f'resourcePacks:[{",".join(packs)}]\n')
    else:
        packs = ['"vanilla"']
        if graficos_minimos: packs.append('"file/Pack_Graficos_Minimos.zip"')
        if version_mayor >= 20: packs.append(f'"file/{pack_name}"')
        lineas.append(f'resourcePacks:[{",".join(packs)}]\n')
    
    with open(options_path, "w") as f: f.writelines(lineas)

def optimizar_graficos(game_dir, graficos_minimos, version):
    try: version_mayor = int(version.split(".")[1])
    except: version_mayor = 20

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

def instalar_mods_optimode(game_dir, version, progress_callback):
    mods_dir = os.path.join(game_dir, "mods")
    os.makedirs(mods_dir, exist_ok=True)
    
    if len([f for f in os.listdir(mods_dir) if f.endswith('.jar')]) >= 5:
        if progress_callback: progress_callback("✅ Mods detectados. Arranque rápido...")
        return

    slugs_mods = ["fabric-api", "sodium", "lithium", "ferrite-core", "modernfix", "iris", "modmenu", "entityculling", "immediatelyfast"]
    headers = {"User-Agent": "ParaguacraftLauncher/1.0"}
    try:
        if int(version.split(".")[1]) < 14: return
    except: pass
    if progress_callback: progress_callback("Descargando mods de optimización (Solo primera vez)...")
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


# =========================================================================
# EL CEREBRO DEL LAUNCHER (CÓDIGO OFICIAL Y LIMPIO)
# =========================================================================

def lanzar_minecraft(version="1.20.4", username="Player", max_ram="4G", gc_type="G1GC", optimizar=False, tipo_cliente="Fabric", papa_mode=False, usar_mesa=False, mostrar_consola=False, progress_callback=None, uuid_real=None, token_real=None):
    minecraft_directory = minecraft_launcher_lib.utils.get_minecraft_directory()

    version_base = version.strip()
    if version_base.startswith("fabric-loader-"):
        version_base = version_base.split("-")[-1]
        tipo_cliente = "Fabric"
    elif "forge" in version_base.lower():
        version_base = version_base.split("-")[0]
        tipo_cliente = "Forge"

    # 1. PURGA DEL MANIFIESTO (Evita errores de "Not Found" por caché roto)
    manifest_path = os.path.join(minecraft_directory, "versions", "version_manifest_v2.json")
    if os.path.exists(manifest_path):
        try: os.remove(manifest_path)
        except: pass

    # 2. INSTALACIÓN SILENCIOSA (Desconectamos la barra para que el hilo no explote)
    if progress_callback: progress_callback(f"Descargando base {version_base} (Puede tardar, no toques nada)...")

    try:
        # Al no tener "callback=", Python no se ahoga actualizando la interfaz
        minecraft_launcher_lib.install.install_minecraft_version(version_base, minecraft_directory)
    except Exception as e:
        raise Exception(f"Mojang rechazó la descarga: {e}")

    # Verificación estricta: ¿De verdad se instaló en el disco duro?
    ruta_json_base = os.path.join(minecraft_directory, "versions", version_base, f"{version_base}.json")
    if not os.path.exists(ruta_json_base):
        raise Exception("El juego no se pudo descargar. Revisá tu conexión a internet.")

    version_a_lanzar = version_base

    # 3. MOTOR DE INSTALACIÓN
    if tipo_cliente == "Fabric":
        if progress_callback: progress_callback("Inyectando motor Fabric (Instalación silenciosa)...")
        loader_nuevo = minecraft_launcher_lib.fabric.get_latest_loader_version()
        minecraft_launcher_lib.fabric.install_fabric(version_base, minecraft_directory, loader_nuevo)
        version_a_lanzar = f"fabric-loader-{loader_nuevo}-{version_base}"

    elif tipo_cliente == "Forge":
        if progress_callback: progress_callback("Inyectando motor Forge...")
        forge_version = minecraft_launcher_lib.forge.find_forge_version(version_base)
        if forge_version:
            minecraft_launcher_lib.forge.install_forge_version(forge_version, minecraft_directory)
            version_a_lanzar = forge_version

    # 4. LIBRERÍAS DEL MOTOR
    if version_a_lanzar != version_base:
        if progress_callback: progress_callback(f"Descargando librerías de {tipo_cliente}...")
        minecraft_launcher_lib.install.install_minecraft_version(version_a_lanzar, minecraft_directory)

    # --- VERIFICACIÓN BLINDADA DEL MOTOR ---
    ruta_json_motor = os.path.join(minecraft_directory, "versions", version_a_lanzar, f"{version_a_lanzar}.json")
    if not os.path.exists(ruta_json_motor):
        raise Exception(f"Fallo crítico instalando {tipo_cliente}. Se abortó la descarga.")

    # 5. AISLAMIENTO DE INSTANCIAS
    folder_name = f"Paraguacraft_{version_base}_{tipo_cliente}".replace(".", "_")
    game_dir = os.path.join(minecraft_directory, "instancias", folder_name)
    os.makedirs(game_dir, exist_ok=True)

    # 6. MODS Y GRÁFICOS
    if progress_callback: progress_callback("Configurando optimizaciones y mods...")
    if tipo_cliente == "Fabric": instalar_mods_optimode(game_dir, version_base, progress_callback)
    instalar_extras_graficos(game_dir, version_base, progress_callback, optimizar)
    inyectar_logos_paraguacraft(game_dir, version_base, optimizar)
    optimizar_graficos(game_dir, optimizar, version_base)

    # 7. GESTOR AUTOMÁTICO DE JAVA FINAL
    if progress_callback: progress_callback("Descargando Java y abriendo juego...")
    minecraft_launcher_lib.runtime.install_jvm_runtime(version_a_lanzar, minecraft_directory)
    java_path_final = minecraft_launcher_lib.runtime.get_executable_path(version_a_lanzar, minecraft_directory)

    # 8. ARGUMENTOS DE JAVA Y LANZAMIENTO
    jvm_arguments = [f"-Xmx{max_ram}", f"-Xms{max_ram}", "-XX:+UnlockExperimentalVMOptions", "-XX:+DisableExplicitGC", "-XX:+AlwaysPreTouch"]
    if gc_type == "G1GC":
        jvm_arguments.extend(["-XX:+UseG1GC", "-XX:G1NewSizePercent=30", "-XX:G1MaxNewSizePercent=40", "-XX:G1HeapRegionSize=8M", "-XX:G1ReservePercent=20", "-XX:G1HeapWastePercent=5", "-XX:G1MixedGCCountTarget=4", "-XX:InitiatingHeapOccupancyPercent=15", "-XX:G1MixedGCLiveThresholdPercent=90", "-XX:G1RSetUpdatingPauseTimePercent=5", "-XX:SurvivorRatio=32", "-XX:+PerfDisableSharedMem", "-XX:MaxTenuringThreshold=1", "-Dusing.aikars.flags=https://mcflags.emc.gs", "-Daikars.new.flags=true"])
    elif gc_type == "ZGC": jvm_arguments.append("-XX:+UseZGC")
    elif gc_type == "Shenandoah": jvm_arguments.append("-XX:+UseShenandoahGC")
    elif gc_type == "CMS": jvm_arguments.append("-XX:+UseConcMarkSweepGC")

    options = {
        "username": username, "uuid": uuid_real if uuid_real else str(uuid.uuid4()),
        "token": token_real if token_real else "", "jvmArguments": jvm_arguments, "gameDirectory": game_dir,
        "executablePath": java_path_final 
    }
    
    if papa_mode: options["customResolution"] = True; options["resolutionWidth"] = "800"; options["resolutionHeight"] = "600"
    command = minecraft_launcher_lib.command.get_minecraft_command(version_a_lanzar, minecraft_directory, options)
    
    entorno = os.environ.copy()
    if usar_mesa: entorno["MESA_GL_VERSION_OVERRIDE"] = "3.3"; entorno["MESA_GLSL_VERSION_OVERRIDE"] = "330"

    if mostrar_consola and platform.system() == "Windows": flags_creacion = subprocess.CREATE_NEW_CONSOLE; salida = None
    else: flags_creacion = subprocess.CREATE_NO_WINDOW if platform.system() == "Windows" else 0; salida = subprocess.DEVNULL 

    if progress_callback: progress_callback("¡Lanzando proceso...")
    proceso = subprocess.Popen(command, env=entorno, creationflags=flags_creacion, stdout=salida, stderr=salida, stdin=subprocess.DEVNULL)
    
    try:
        import psutil
        p = psutil.Process(proceso.pid)
        if sys.platform == "win32": p.nice(psutil.HIGH_PRIORITY_CLASS)
    except: pass
    
    proceso.wait()

if __name__ == "__main__": pass