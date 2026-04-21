import os
import requests
import hashlib
import shutil
import minecraft_launcher_lib

class GestorMods:
    @staticmethod
    def calcular_sha1(ruta):
        """Calcula la huella digital binaria de un archivo (SHA-1)"""
        sha1 = hashlib.sha1()
        try:
            with open(ruta, "rb") as f:
                for bloque in iter(lambda: f.read(8192), b""):
                    sha1.update(bloque)
            return sha1.hexdigest()
        except Exception:
            return None

    @staticmethod
    def verificar_y_actualizar(mods_dir, version_actual, callback_progreso):
        """
        Escanea la carpeta de mods, calcula hashes y los compara con la API.
        Descarga las versiones íntegras si hay corrupción o actualizaciones.
        """
        callback_progreso("Escaneando integridad de mods (Calculando Hashes)...")
        mods_locales = [f for f in os.listdir(mods_dir) if f.endswith(".jar")]
        actualizados = 0
        corruptos_reparados = 0

        for mod_file in mods_locales:
            ruta_local = os.path.join(mods_dir, mod_file)
            hash_local = GestorMods.calcular_sha1(ruta_local)
            nombre_busqueda = mod_file.split("-")[0].split("_")[0]
            
            url_api = f"https://api.modrinth.com/v2/search?query={nombre_busqueda}&facets=[[\"versions:{version_actual}\"],[\"project_type:mod\"]]"
            
            try:
                r = requests.get(url_api, timeout=10).json()
                if r.get("hits"):
                    slug = r["hits"][0]["slug"]
                    
                    v_r = requests.get(f"https://api.modrinth.com/v2/project/{slug}/version", params={"game_versions": f'["{version_actual}"]', "loaders": '["fabric"]'}).json()
                    if v_r:
                        datos_archivo_remoto = v_r[0]["files"][0]
                        hash_remoto = datos_archivo_remoto["hashes"]["sha1"]
                        nuevo_archivo = datos_archivo_remoto["filename"]
                        
                        # LOGICA CRÍTICA DE INTEGRIDAD (HASHING)
                        if hash_local != hash_remoto:
                            motivo = "Actualizando" if nuevo_archivo != mod_file else "Reparando"
                            callback_progreso(f"{motivo} {slug} (Hash desactualizado)...")
                            
                            # Descargamos el archivo íntegro
                            r_dl = requests.get(datos_archivo_remoto["url"], stream=True, timeout=20)
                            with open(os.path.join(mods_dir, nuevo_archivo), "wb") as f:
                                shutil.copyfileobj(r_dl.raw, f)
                            
                            # Borramos el viejo si es distinto o estaba corrupto
                            if nuevo_archivo != mod_file or motivo == "Reparando":
                                try: os.remove(ruta_local)
                                except: pass
                            
                            if nuevo_archivo != mod_file: actualizados += 1
                            else: corruptos_reparados += 1
            except Exception as e:
                print(f"Omitiendo {mod_file} por error de red: {e}")
                continue

        callback_progreso("¡Integridad verificada y mods al día!")
        return actualizados, corruptos_reparados
    
class TiendaAPI:
    # Mapeo de nombres de motor del launcher → categoría Modrinth
    _LOADER_MODRINTH = {
        "fabric + iris": "fabric", "fabric+iris": "fabric",
        "fabric": "fabric", "forge": "forge", "neoforge": "neoforge",
        "quilt": "quilt", "optifine": "forge", "vanilla": "",
    }

    @staticmethod
    def _normalizar_loader(tipo_motor):
        _key = tipo_motor.lower().replace(" ", "").replace("+", "+")
        for k, v in TiendaAPI._LOADER_MODRINTH.items():
            if k.replace(" ", "") == _key.replace(" ", ""):
                return v
        return tipo_motor.lower().split(" ")[0]

    @staticmethod
    def buscar_mods(query, version_actual, tipo_motor, tipo_proyecto):
        """Se conecta a Modrinth, hace la búsqueda y devuelve los resultados crudos."""
        import json as _json
        _loader = TiendaAPI._normalizar_loader(tipo_motor)
        facets = [
            [f"versions:{version_actual}"],
            [f"project_type:{tipo_proyecto}"],
        ]
        if tipo_proyecto in ["mod", "modpack"] and _loader:
            facets.append([f"categories:{_loader}"])

        try:
            r = requests.get(
                "https://api.modrinth.com/v2/search",
                params={
                    "query": query,
                    "limit": 20,
                    "index": "downloads",
                    "facets": _json.dumps(facets),
                },
                headers={"User-Agent": "ParaguacraftLauncher/1.0"},
                timeout=10,
            )
            if r.status_code == 200:
                return True, r.json().get("hits", [])
            else:
                return False, f"Error de la API: {r.status_code}"
        except Exception:
            return False, "No hay conexión a internet o la API está caída."

    @staticmethod
    def descargar_mod(slug, version, tipo_loader, carpeta_destino, callback_progreso):
        """Descarga un mod y, recursivamente, busca y descarga sus dependencias requeridas."""
        
        def _bajar_recursivo(project_slug_or_id, descargados_historico):
            url_api = f"https://api.modrinth.com/v2/project/{project_slug_or_id}/version"
            params = {"game_versions": f'["{version}"]'}
            
            # Exigir compatibilidad de motor si va a la carpeta mods
            if "mods" in carpeta_destino:
                loader = "fabric" if tipo_loader == "vanilla" else tipo_loader
                params["loaders"] = f'["{loader}"]'
                
            try:
                r = requests.get(url_api, params=params, headers={"User-Agent": "ParaguacraftLauncher/1.1"}, timeout=10)
                if r.status_code == 200:
                    data = r.json()
                    if len(data) > 0:
                        version_data = data[0]
                        archivo = version_data["files"][0]
                        nombre_archivo = archivo["filename"]
                        url_descarga = archivo["url"]
                        
                        # Evitar bucles infinitos si dos mods se requieren mutuamente
                        if nombre_archivo in descargados_historico:
                            return True, "exito"
                            
                        ruta_final = os.path.join(carpeta_destino, nombre_archivo)
                        
                        # Si ya lo tiene físicamente, lo salteamos
                        if not os.path.exists(ruta_final):
                            callback_progreso(f"Descargando {nombre_archivo}...")
                            r_dl = requests.get(url_descarga, stream=True, timeout=20)
                            with open(ruta_final, "wb") as f:
                                import shutil
                                shutil.copyfileobj(r_dl.raw, f)
                                
                        descargados_historico.add(nombre_archivo)
                        
                        # === MAGIA PURA: LECTOR DE DEPENDENCIAS ===
                        if "dependencies" in version_data:
                            for dep in version_data["dependencies"]:
                                # Si es requerida obligatoriamente, la bajamos
                                if dep.get("dependency_type") == "required":
                                    dep_id = dep.get("project_id")
                                    if dep_id:
                                        callback_progreso(f"Resolviendo dependencia...")
                                        _bajar_recursivo(dep_id, descargados_historico)
                        
                        return True, "exito"
                    else:
                        return False, "incompatible"
                else:
                    return False, "error_api"
            except Exception as e:
                return False, str(e)

        # Iniciamos el ciclo recursivo con un Set vacío
        try:
            exito, msj = _bajar_recursivo(slug, set())
            return exito, msj
        except Exception as e:
            return False, str(e)
        
class CreadorServidor:
    @staticmethod
    def descargar_y_preparar(carpeta_server, ver_server, callback_progreso, tipo='paper'):
        if tipo == 'fabric':
            return CreadorServidor._setup_fabric(carpeta_server, ver_server, callback_progreso)
        if tipo == 'fabric-geyser':
            return CreadorServidor._setup_fabric_geyser(carpeta_server, ver_server, callback_progreso)
        if tipo == 'forge':
            return CreadorServidor._setup_forge(carpeta_server, ver_server, callback_progreso)
        try:
            callback_progreso(f"[1/7] Buscando PaperMC para la versión {ver_server}...")
            callback_progreso("[INFO] Este servidor es compatible con múltiples versiones de cliente (1.8 → última versión).")

            # 1. PaperMC API — buscar el último build estable para la versión dada
            try:
                builds_resp = requests.get(
                    f"https://api.papermc.io/v2/projects/paper/versions/{ver_server}/builds",
                    timeout=15)
            except Exception as e:
                return False, f"No se pudo conectar con la API de PaperMC: {e}"

            if builds_resp.status_code != 200:
                return False, (
                    f"PaperMC no soporta la versión '{ver_server}'. "
                    f"Usá una versión soportada (ej: 26.1.1, 1.21.4, 1.20.4, 1.19.4, 1.18.2)."
                )

            builds_data = builds_resp.json()
            builds = builds_data.get("builds", [])
            if not builds:
                return False, f"No hay builds de PaperMC disponibles para la versión {ver_server}."

            latest_build = next(
                (b for b in reversed(builds) if b.get("channel") == "default"),
                builds[-1]
            )
            build_number = latest_build["build"]
            jar_name = latest_build["downloads"]["application"]["name"]
            callback_progreso(f"[1/7] PaperMC {ver_server} build #{build_number} encontrado.")

            # 2. Descarga server.jar (PaperMC)
            paper_url = (
                f"https://api.papermc.io/v2/projects/paper/versions/{ver_server}"
                f"/builds/{build_number}/downloads/{jar_name}"
            )
            server_jar_path = os.path.join(carpeta_server, "server.jar")
            if os.path.exists(server_jar_path) and os.path.getsize(server_jar_path) > 1_000_000:
                callback_progreso("[2/7] server.jar ya existe, omitiendo descarga...")
            else:
                callback_progreso(f"[2/7] Descargando PaperMC {ver_server} (build #{build_number})...")
                try:
                    r_jar = requests.get(paper_url, stream=True, timeout=(15, 300))
                    r_jar.raise_for_status()
                except Exception as e:
                    return False, f"Error descargando PaperMC: {e}"
                total_bytes = int(r_jar.headers.get("content-length", 0))
                downloaded = 0
                _CHUNK = 65536
                _last_pct = -1
                with open(server_jar_path, "wb") as f:
                    for chunk in r_jar.iter_content(chunk_size=_CHUNK):
                        if chunk:
                            f.write(chunk)
                            downloaded += len(chunk)
                            if total_bytes > 0:
                                pct = downloaded * 100 // total_bytes
                                if pct >= _last_pct + 5:
                                    _last_pct = pct
                                    callback_progreso(
                                        f"[2/7] Descargando PaperMC... "
                                        f"{downloaded/1048576:.1f}/{total_bytes/1048576:.1f} MB ({pct}%)")
                callback_progreso(f"[2/7] PaperMC descargado ({downloaded/1048576:.1f} MB).")

            # 3. Descargar plugins ViaVersion + ViaBackwards (multi-versión de clientes)
            plugins_dir = os.path.join(carpeta_server, "plugins")
            os.makedirs(plugins_dir, exist_ok=True)

            for plugin_name, repo in [
                ("ViaVersion",   "ViaVersion/ViaVersion"),
                ("ViaBackwards", "ViaVersion/ViaBackwards"),
            ]:
                plugin_path = os.path.join(plugins_dir, f"{plugin_name}.jar")
                if os.path.exists(plugin_path):
                    callback_progreso(f"[3/7] {plugin_name}.jar ya existe.")
                    continue
                callback_progreso(f"[3/7] Descargando {plugin_name}...")
                try:
                    api_r = requests.get(
                        f"https://api.github.com/repos/{repo}/releases/latest",
                        timeout=15, headers={"Accept": "application/vnd.github+json"})
                    api_r.raise_for_status()
                    assets = api_r.json().get("assets", [])
                    jar_url = next(
                        (a["browser_download_url"] for a in assets
                         if a["name"].lower().endswith(".jar")
                         and "sources" not in a["name"].lower()
                         and "javadoc" not in a["name"].lower()),
                        None
                    )
                    if jar_url:
                        r_plugin = requests.get(jar_url, stream=True, timeout=(15, 120))
                        r_plugin.raise_for_status()
                        with open(plugin_path, "wb") as f:
                            import shutil
                            shutil.copyfileobj(r_plugin.raw, f)
                        callback_progreso(f"[3/7] {plugin_name} instalado en plugins/")
                    else:
                        callback_progreso(f"⚠️ No se encontró .jar para {plugin_name} en GitHub.")
                except Exception as e:
                    callback_progreso(f"⚠️ Error descargando {plugin_name}: {e}")

            # 3b. SkinsRestorer plugin (para todos los tipos de server Paper)
            sr_plugin_path = os.path.join(plugins_dir, "SkinsRestorer.jar")
            if not (os.path.exists(sr_plugin_path) and os.path.getsize(sr_plugin_path) > 100_000):
                callback_progreso("[3b/7] Descargando SkinsRestorer...")
                try:
                    sr_r = requests.get(
                        "https://api.modrinth.com/v2/project/skinsrestorer/version",
                        params={"loaders": '["paper","spigot","bukkit"]',
                                "game_versions": f'["{ver_server}"]'},
                        timeout=10, headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                    sr_vers = sr_r.json() if sr_r.status_code == 200 else []
                    if not sr_vers:  # sin filtro de versión si no hay match exacto
                        sr_r2 = requests.get(
                            "https://api.modrinth.com/v2/project/skinsrestorer/version",
                            params={"loaders": '["paper","spigot","bukkit"]'},
                            timeout=10, headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                        sr_vers = sr_r2.json() if sr_r2.status_code == 200 else []
                    if sr_vers:
                        sr_file = next((f for f in sr_vers[0]["files"] if f.get("primary")), sr_vers[0]["files"][0])
                        r_sr = requests.get(sr_file["url"], stream=True, timeout=(15, 120), allow_redirects=True)
                        r_sr.raise_for_status()
                        with open(sr_plugin_path, "wb") as f:
                            shutil.copyfileobj(r_sr.raw, f, length=65536)
                        callback_progreso("✅ SkinsRestorer instalado en plugins/")
                    else:
                        callback_progreso("⚠️ SkinsRestorer no disponible para esta versión en Modrinth.")
                except Exception as e_sr:
                    callback_progreso(f"⚠️ SkinsRestorer no descargado: {e_sr}")

            # 3c. Geyser + Floodgate (soporte Bedrock)
            if tipo == 'paper-geyser':
                for plugin_name, slug, hangar_owner, hangar_slug in [
                    ("Geyser-Spigot",    "geyser",    "GeyserMC", "Geyser-Spigot"),
                    ("Floodgate-Spigot", "floodgate", "GeyserMC", "Floodgate"),
                ]:
                    plugin_path = os.path.join(plugins_dir, f"{plugin_name}.jar")
                    if os.path.exists(plugin_path) and os.path.getsize(plugin_path) > 100_000:
                        callback_progreso(f"[3c/7] {plugin_name}.jar ya existe.")
                        continue
                    callback_progreso(f"[3c/7] Descargando {plugin_name} (soporte Bedrock)...")
                    downloaded = False
                    # Intento 1: download.geysermc.org
                    for platform in ("spigot", "paper"):
                        if downloaded:
                            break
                        try:
                            dl_url = f"https://download.geysermc.org/v2/projects/{slug}/versions/latest/builds/latest/downloads/{platform}"
                            r_g = requests.get(dl_url, stream=True, timeout=(20, 180), allow_redirects=True)
                            r_g.raise_for_status()
                            with open(plugin_path, "wb") as f:
                                shutil.copyfileobj(r_g.raw, f, length=65536)
                            if os.path.getsize(plugin_path) > 100_000:
                                downloaded = True
                                callback_progreso(f"[3b/7] {plugin_name} instalado (geysermc.org).")
                            else:
                                os.remove(plugin_path)
                        except Exception:
                            pass
                    # Intento 2: Hangar API (papermc.io)
                    if not downloaded:
                        try:
                            callback_progreso(f"[3b/7] Fallback Hangar para {plugin_name}...")
                            ver_r = requests.get(
                                f"https://hangar.papermc.io/api/v1/projects/{hangar_owner}/{hangar_slug}/latestrelease",
                                timeout=15, headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                            ver_name = ver_r.text.strip().strip('"')
                            dl_url2 = f"https://hangar.papermc.io/api/v1/projects/{hangar_owner}/{hangar_slug}/versions/{ver_name}/PAPER/download"
                            r_h = requests.get(dl_url2, stream=True, timeout=(20, 180),
                                               allow_redirects=True, headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                            r_h.raise_for_status()
                            with open(plugin_path, "wb") as f:
                                shutil.copyfileobj(r_h.raw, f, length=65536)
                            if os.path.getsize(plugin_path) > 100_000:
                                downloaded = True
                                callback_progreso(f"[3b/7] {plugin_name} instalado (Hangar).")
                            else:
                                os.remove(plugin_path)
                        except Exception as e2:
                            callback_progreso(f"[3b/7] Fallback Hangar falló: {e2}")
                    if not downloaded:
                        callback_progreso(f"⚠️ No se pudo descargar {plugin_name}. Instalalo manualmente desde geysermc.org")

            # 4. EULA
            callback_progreso("[4/7] Aceptando EULA...")
            with open(os.path.join(carpeta_server, "eula.txt"), "w") as f:
                f.write("eula=true\n")

            # 5. Buscar Java
            callback_progreso("[5/7] Buscando Java instalado...")
            java_cmd = "java"
            try:
                mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                runtime_dir = os.path.join(mine_dir, "runtime")
                if os.path.exists(runtime_dir):
                    javas = [os.path.join(r, "java.exe")
                             for r, _, fs in os.walk(runtime_dir) if "java.exe" in fs]
                    if javas:
                        best = next((j for j in javas if "21" in j or "delta" in j.lower() or "gamma" in j.lower()), javas[0])
                        java_cmd = f'"{best}"'
                        callback_progreso(f"[5/7] Java encontrado: {best}")
                    else:
                        callback_progreso("[5/7] Java del launcher no encontrado, usando 'java' del sistema.")
                else:
                    callback_progreso("[5/7] Runtime dir no existe, usando 'java' del sistema.")
            except Exception as e:
                callback_progreso(f"[5/7] Error buscando Java ({e}), usando 'java' del sistema.")

            # 6. Crear iniciar_server.bat
            callback_progreso("[6/7] Creando iniciar_server.bat...")
            bat_content = (
                "@echo off\n"
                "echo ========================================\n"
                f"echo  Servidor PaperMC {ver_server} (Multi-Version)\n"
                "echo  Compatible: clientes 1.8 hasta ultima version\n"
                "echo  Premium y no-premium: online-mode=false\n"
                "echo ========================================\n"
                "echo Iniciando Tunel Playit...\n"
                "if exist playit.exe (start cmd /k playit.exe) else "
                "(echo playit.exe no encontrado, descargalo de playit.gg)\n"
                "echo Iniciando Servidor PaperMC...\n"
                f"{java_cmd} -Xmx4G -Xms1G -jar server.jar nogui\n"
                "pause\n"
            )
            with open(os.path.join(carpeta_server, "iniciar_server.bat"), "w", encoding="utf-8") as f:
                f.write(bat_content)

            # 7. Descarga playit.exe (sincrónico, busca URL correcta via GitHub API)
            playit_path = os.path.join(carpeta_server, "playit.exe")
            if not os.path.exists(playit_path):
                callback_progreso("[7/7] Buscando última versión de playit.exe...")
                try:
                    api_r = requests.get(
                        "https://api.github.com/repos/playit-cloud/playit-agent/releases/latest",
                        timeout=15, headers={"Accept": "application/vnd.github+json"})
                    api_r.raise_for_status()
                    assets = api_r.json().get("assets", [])
                    exe_url = None
                    for asset in assets:
                        n = asset["name"].lower()
                        if n.endswith(".exe") and ("win" in n or "windows" in n):
                            exe_url = asset["browser_download_url"]
                            break
                    if not exe_url:
                        raise Exception("No se encontró un .exe de Windows en los assets del release.")
                    tag = api_r.json().get("tag_name", "")
                    callback_progreso(f"[7/7] Descargando playit.exe {tag}...")
                    r_p = requests.get(exe_url, stream=True, timeout=(20, 300))
                    r_p.raise_for_status()
                    with open(playit_path, "wb") as _f:
                        shutil.copyfileobj(r_p.raw, _f, length=65536)
                    callback_progreso("[7/7] playit.exe descargado.")
                except Exception as _e:
                    callback_progreso(f"⚠️ playit.exe no descargado: {_e}. Descargalo manualmente desde playit.gg")
            else:
                callback_progreso("[7/7] playit.exe ya existe.")

            callback_progreso("✅ Servidor PaperMC listo. Compatible con clientes 1.8 - última versión.")
            callback_progreso("✅ Plugins: ViaVersion + ViaBackwards instalados en plugins/")
            if tipo == 'paper-geyser':
                callback_progreso("✅ Geyser + Floodgate instalados. ¡Jugadores Bedrock pueden conectarse!")
                callback_progreso("ℹ️ Bedrock: En playit.gg creá un túnel UDP en puerto 19132 para jugadores Bedrock.")
            return True, f"¡Todo listo en {carpeta_server}!"

        except Exception as e:
            return False, str(e)

    @staticmethod
    def _setup_fabric(carpeta_server, ver_server, callback_progreso):
        try:
            callback_progreso(f"[1/5] Obteniendo versiones de Fabric compatibles con {ver_server}...")
            loader_ver = None
            inst_ver = None
            try:
                # API version-específica: devuelve loader+installer compatibles con la versión del juego
                ver_resp = requests.get(
                    f"https://meta.fabricmc.net/v2/versions/loader/{ver_server}",
                    timeout=10)
                if ver_resp.status_code == 200 and ver_resp.json():
                    entries = ver_resp.json()
                    best = next((e for e in entries if e.get("loader", {}).get("stable")), entries[0])
                    loader_ver = best["loader"]["version"]
                    inst_ver   = best.get("installer", {}).get("version")
            except Exception:
                pass
            # Fallback: API global si la versión específica no respondió
            if not loader_ver or not inst_ver:
                try:
                    loader_resp = requests.get("https://meta.fabricmc.net/v2/versions/loader",    timeout=10)
                    inst_resp   = requests.get("https://meta.fabricmc.net/v2/versions/installer", timeout=10)
                    loader_resp.raise_for_status(); inst_resp.raise_for_status()
                    loaders = loader_resp.json(); installers = inst_resp.json()
                    loader_ver = loader_ver or next((l['version'] for l in loaders    if l.get('stable')), loaders[0]['version'])
                    inst_ver   = inst_ver   or next((i['version'] for i in installers if i.get('stable')), installers[0]['version'])
                except Exception as e:
                    return False, f"No se pudo conectar con la API de Fabric: {e}"
            callback_progreso(f"[1/5] Fabric loader {loader_ver} | installer {inst_ver}")

            server_jar_path = os.path.join(carpeta_server, "server.jar")
            if os.path.exists(server_jar_path) and os.path.getsize(server_jar_path) > 100_000:
                callback_progreso("[2/5] server.jar ya existe, omitiendo descarga...")
            else:
                jar_url = (f"https://meta.fabricmc.net/v2/versions/loader/{ver_server}"
                           f"/{loader_ver}/{inst_ver}/server/jar")
                callback_progreso(f"[2/5] Descargando Fabric server {ver_server}...")
                try:
                    r_jar = requests.get(jar_url, stream=True, timeout=(15, 300), allow_redirects=True)
                    if r_jar.status_code != 200:
                        return False, (f"Fabric no encontró servidor para '{ver_server}' "
                                       f"(HTTP {r_jar.status_code}). Verificá que la versión sea soportada.")
                    r_jar.raise_for_status()
                except Exception as e:
                    return False, f"Error descargando Fabric server: {e}"
                downloaded = 0
                with open(server_jar_path, "wb") as f:
                    for chunk in r_jar.iter_content(chunk_size=65536):
                        if chunk:
                            f.write(chunk)
                            downloaded += len(chunk)
                            if downloaded % (5 * 1024 * 1024) < 65536:
                                callback_progreso(f"[2/5] Descargando... {downloaded/1048576:.1f} MB")
                callback_progreso(f"[2/5] Fabric server descargado ({downloaded/1048576:.1f} MB).")

            # 2b. SkinsRestorer mod (skinrestorer para Fabric)
            mods_dir = os.path.join(carpeta_server, "mods")
            os.makedirs(mods_dir, exist_ok=True)
            sr_path = os.path.join(mods_dir, "SkinsRestorer.jar")
            if not (os.path.exists(sr_path) and os.path.getsize(sr_path) > 100_000):
                callback_progreso("[2b/5] Descargando SkinsRestorer...")
                try:
                    sr_r = requests.get(
                        "https://api.modrinth.com/v2/project/skinsrestorer/version",
                        params={"loaders": '["fabric"]', "game_versions": f'["{ver_server}"]'},
                        timeout=10, headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                    sr_vers = sr_r.json() if sr_r.status_code == 200 else []
                    if not sr_vers:  # sin filtro de versión si no hay match exacto
                        sr_r2 = requests.get(
                            "https://api.modrinth.com/v2/project/skinsrestorer/version",
                            params={"loaders": '["fabric"]'}, timeout=10,
                            headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                        sr_vers = sr_r2.json() if sr_r2.status_code == 200 else []
                    if sr_vers:
                        sr_file = next((f for f in sr_vers[0]["files"] if f.get("primary")), sr_vers[0]["files"][0])
                        r_sr = requests.get(sr_file["url"], stream=True, timeout=(15, 120), allow_redirects=True)
                        r_sr.raise_for_status()
                        with open(sr_path, "wb") as f:
                            shutil.copyfileobj(r_sr.raw, f, length=65536)
                        callback_progreso("✅ SkinsRestorer instalado en mods/")
                    else:
                        callback_progreso("⚠️ SkinsRestorer no disponible para esta versión en Modrinth.")
                except Exception as e_sr:
                    callback_progreso(f"⚠️ SkinsRestorer no descargado: {e_sr}")

            # 2c. Fabric API (requerida por Geyser y la mayoría de mods Fabric server)
            fa_existing = [f for f in os.listdir(mods_dir)
                           if 'fabric-api' in f.lower() or 'fabricapi' in f.lower()]
            if not fa_existing:
                callback_progreso("[2c/5] Descargando Fabric API...")
                try:
                    fa_r = requests.get(
                        "https://api.modrinth.com/v2/project/fabric-api/version",
                        params={"loaders": '["fabric"]', "game_versions": f'["{ver_server}"]'},
                        timeout=10, headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                    fa_vers = fa_r.json() if fa_r.status_code == 200 else []
                    if not fa_vers:
                        fa_r2 = requests.get(
                            "https://api.modrinth.com/v2/project/fabric-api/version",
                            params={"loaders": '["fabric"]'}, timeout=10,
                            headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                        fa_vers = fa_r2.json() if fa_r2.status_code == 200 else []
                    if fa_vers:
                        fa_file = next((f for f in fa_vers[0]["files"] if f.get("primary")), fa_vers[0]["files"][0])
                        r_fa = requests.get(fa_file["url"], stream=True, timeout=(15, 120), allow_redirects=True)
                        r_fa.raise_for_status()
                        with open(os.path.join(mods_dir, fa_file["filename"]), "wb") as f:
                            shutil.copyfileobj(r_fa.raw, f, length=65536)
                        callback_progreso("✅ Fabric API instalada en mods/")
                    else:
                        callback_progreso("⚠️ Fabric API no disponible para esta versión en Modrinth.")
                except Exception as e_fa:
                    callback_progreso(f"⚠️ Fabric API no descargada: {e_fa}")

            callback_progreso("[3/5] Aceptando EULA...")
            with open(os.path.join(carpeta_server, "eula.txt"), "w") as f:
                f.write("eula=true\n")

            callback_progreso("[4/5] Buscando Java...")
            java_cmd = "java"
            try:
                mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                runtime_dir = os.path.join(mine_dir, "runtime")
                if os.path.exists(runtime_dir):
                    javas = [os.path.join(r, "java.exe") for r, _, fs in os.walk(runtime_dir) if "java.exe" in fs]
                    if javas:
                        best = next((j for j in javas if "21" in j or "delta" in j.lower() or "gamma" in j.lower()), javas[0])
                        java_cmd = f'"{best}"'
                        callback_progreso(f"[4/5] Java: {best}")
            except Exception:
                pass

            callback_progreso("[5/5] Creando iniciar_server.bat y descargando playit.exe...")
            bat = (
                "@echo off\n"
                "echo ========================================\n"
                f"echo  Servidor Fabric {ver_server}\n"
                "echo  Coloca los mods .jar en la carpeta mods/\n"
                "echo  Jugadores necesitan Fabric Client instalado\n"
                "echo ========================================\n"
                "if exist playit.exe (start cmd /k playit.exe) else "
                "(echo playit.exe no encontrado - descargalo de playit.gg)\n"
                f"{java_cmd} -Xmx4G -Xms1G -jar server.jar nogui\n"
                "pause\n"
            )
            with open(os.path.join(carpeta_server, "iniciar_server.bat"), "w", encoding="utf-8") as f:
                f.write(bat)

            playit_path = os.path.join(carpeta_server, "playit.exe")
            if not os.path.exists(playit_path):
                try:
                    api_r = requests.get(
                        "https://api.github.com/repos/playit-cloud/playit-agent/releases/latest",
                        timeout=15, headers={"Accept": "application/vnd.github+json"})
                    api_r.raise_for_status()
                    exe_url = next(
                        (a["browser_download_url"] for a in api_r.json().get("assets", [])
                         if a["name"].lower().endswith(".exe") and
                         ("win" in a["name"].lower() or "windows" in a["name"].lower())),
                        None)
                    if exe_url:
                        r_p = requests.get(exe_url, stream=True, timeout=(20, 300))
                        r_p.raise_for_status()
                        with open(playit_path, "wb") as _f:
                            shutil.copyfileobj(r_p.raw, _f, length=65536)
                        callback_progreso("playit.exe descargado.")
                    else:
                        callback_progreso("⚠️ playit.exe no encontrado, descargalo de playit.gg")
                except Exception as e:
                    callback_progreso(f"⚠️ playit.exe no descargado: {e}")
            else:
                callback_progreso("playit.exe ya existe.")

            os.makedirs(os.path.join(carpeta_server, "mods"), exist_ok=True)
            callback_progreso("✅ Servidor Fabric listo. Coloca tus mods .jar en la carpeta mods/")
            callback_progreso("⚠️ IMPORTANTE: todos los jugadores deben tener Fabric Client instalado.")
            return True, f"¡Fabric server listo en {carpeta_server}!"

        except Exception as e:
            return False, str(e)

    @staticmethod
    def _setup_fabric_geyser(carpeta_server, ver_server, callback_progreso):
        ok, msg = CreadorServidor._setup_fabric(carpeta_server, ver_server, callback_progreso)
        if not ok:
            return False, msg
        mods_dir = os.path.join(carpeta_server, "mods")
        os.makedirs(mods_dir, exist_ok=True)
        geyser_mods = [
            ("Geyser-Fabric",    "geyser",    "fabric"),
            ("Floodgate-Fabric", "floodgate", "fabric"),
        ]
        for mod_name, slug, platform in geyser_mods:
            dest = os.path.join(mods_dir, mod_name + ".jar")
            if os.path.exists(dest) and os.path.getsize(dest) > 100_000:
                callback_progreso(f"{mod_name} ya existe.")
                continue
            callback_progreso(f"Descargando {mod_name}...")
            downloaded = False
            for attempt_platform in (platform, "fabric-api"):
                if downloaded:
                    break
                try:
                    url = f"https://download.geysermc.org/v2/projects/{slug}/versions/latest/builds/latest/downloads/{attempt_platform}"
                    r = requests.get(url, stream=True, timeout=(20, 180), allow_redirects=True)
                    r.raise_for_status()
                    with open(dest, "wb") as f:
                        shutil.copyfileobj(r.raw, f, length=65536)
                    if os.path.getsize(dest) > 100_000:
                        downloaded = True
                        callback_progreso(f"✅ {mod_name} instalado en mods/.")
                    else:
                        os.remove(dest)
                except Exception:
                    pass
            if not downloaded:
                try:
                    url = f"https://api.modrinth.com/v2/project/{slug}/version"
                    r = requests.get(url, timeout=15)
                    r.raise_for_status()
                    versions = r.json()
                    if versions:
                        version = versions[0]
                        files = version.get("files", [])
                        if files:
                            file = files[0]
                            file_url = file.get("url")
                            if file_url:
                                r = requests.get(file_url, stream=True, timeout=(20, 180), allow_redirects=True)
                                r.raise_for_status()
                                with open(dest, "wb") as f:
                                    shutil.copyfileobj(r.raw, f, length=65536)
                                if os.path.getsize(dest) > 100_000:
                                    downloaded = True
                                    callback_progreso(f"✅ {mod_name} instalado en mods/.")
                                else:
                                    os.remove(dest)
                except Exception:
                    pass
            if not downloaded:
                callback_progreso(f"⚠️ No se pudo descargar {mod_name}. Instalalo manualmente desde geysermc.org")
        callback_progreso("✅ Geyser + Floodgate para Fabric instalados. Jugadores Bedrock pueden conectarse.")
        callback_progreso("ℹ️ Bedrock: En playit.gg creá un túnel UDP en puerto 19132 para jugadores Bedrock.")
        return True, f"¡Fabric + Geyser listo en {carpeta_server}!"

class GestorLocalMods:
    @staticmethod
    def _extraer_metadata_jar(ruta_jar):
        """Extrae nombre, version, autor e icono (base64) de un .jar de mod."""
        import zipfile, json as _json, base64
        nombre = version = autor = icono_b64 = None
        try:
            with zipfile.ZipFile(ruta_jar, 'r') as z:
                nombres_zip = set(z.namelist())
                if 'fabric.mod.json' in nombres_zip:
                    with z.open('fabric.mod.json') as f:
                        meta = _json.loads(f.read().decode('utf-8', errors='replace'))
                    nombre = meta.get('name') or meta.get('id')
                    version = meta.get('version', '')
                    autores = meta.get('authors', [])
                    if autores:
                        a0 = autores[0]
                        autor = a0 if isinstance(a0, str) else a0.get('name', '')
                    icon_path = meta.get('icon', '')
                    if icon_path and icon_path in nombres_zip:
                        with z.open(icon_path) as img:
                            icono_b64 = 'data:image/png;base64,' + base64.b64encode(img.read()).decode()
                elif 'META-INF/mods.toml' in nombres_zip:
                    with z.open('META-INF/mods.toml') as f:
                        content = f.read().decode('utf-8', errors='replace')
                    logo_path = None
                    for line in content.splitlines():
                        ls = line.strip()
                        if ls.startswith('displayName') and nombre is None:
                            nombre = ls.split('=', 1)[-1].strip().strip('"\'')
                        elif ls.startswith('version') and version is None and '=' in ls:
                            v = ls.split('=', 1)[-1].strip().strip('"\'')
                            if v and not v.startswith('$'):
                                version = v
                        elif ls.startswith('authors') and autor is None:
                            autor = ls.split('=', 1)[-1].strip().strip('"\'[]')
                        elif 'logoFile' in ls and logo_path is None:
                            logo_path = ls.split('=', 1)[-1].strip().strip('"\'')
                    if logo_path and logo_path in nombres_zip:
                        with z.open(logo_path) as img:
                            icono_b64 = 'data:image/png;base64,' + base64.b64encode(img.read()).decode()
                elif 'mcmod.info' in nombres_zip:
                    with z.open('mcmod.info') as f:
                        try:
                            mods_info = _json.loads(f.read().decode('utf-8', errors='replace'))
                        except Exception:
                            mods_info = []
                    if isinstance(mods_info, list) and mods_info:
                        m = mods_info[0]
                    elif isinstance(mods_info, dict) and mods_info.get('modList'):
                        m = mods_info['modList'][0]
                    else:
                        m = {}
                    nombre = m.get('name') or m.get('modid')
                    version = m.get('version', '')
                    autores = m.get('authorList', [])
                    autor = autores[0] if autores else None
                    logo = m.get('logoFile', '')
                    if logo and logo in nombres_zip:
                        with z.open(logo) as img:
                            icono_b64 = 'data:image/png;base64,' + base64.b64encode(img.read()).decode()
        except Exception:
            pass
        if not nombre:
            base = os.path.basename(ruta_jar).replace('.jar.disabled', '').replace('.jar', '')
            import re as _re2
            nombre = _re2.sub(r'[-_][\d].*$', '', base).replace('-', ' ').replace('_', ' ').strip()
            if not nombre:
                nombre = base
        return nombre, version or '', autor or '', icono_b64 or ''

    @staticmethod
    def obtener_lista_mods(mods_dir):
        """Devuelve una lista con el nombre de cada mod y su estado actual."""
        if not os.path.exists(mods_dir): return []
        mods = []
        for f in os.listdir(mods_dir):
            if f.endswith(".jar") or f.endswith(".jar.disabled"):
                estado = "Activo" if f.endswith(".jar") else "Desactivado"
                ruta = os.path.join(mods_dir, f)
                nombre, version_mod, autor, icono_b64 = GestorLocalMods._extraer_metadata_jar(ruta)
                mods.append({
                    "archivo": f,
                    "estado": estado,
                    "nombre_visible": nombre,
                    "version_mod": version_mod,
                    "autor": autor,
                    "icono_b64": icono_b64,
                })
        return mods

    @staticmethod
    def alternar_estado_mod(mods_dir, nombre_archivo):
        """Alterna la extensión entre .jar y .jar.disabled."""
        ruta_actual = os.path.join(mods_dir, nombre_archivo)
        if not os.path.exists(ruta_actual): return None

        if nombre_archivo.endswith(".jar"):
            nueva_ruta = ruta_actual + ".disabled"
            nuevo_nombre = nombre_archivo + ".disabled"
        elif nombre_archivo.endswith(".jar.disabled"):
            nueva_ruta = ruta_actual.replace(".disabled", "")
            nuevo_nombre = nombre_archivo.replace(".disabled", "")
        else:
            return None

        try:
            os.rename(ruta_actual, nueva_ruta)
            return nuevo_nombre
        except Exception:
            return None


class GestorContenidoInstancia:
    """Mods (.jar), resource packs, shaderpacks y datapacks por mundo (activar/desactivar con .disabled)."""
    PACK_SISTEMA = "ParaguacraftBrandPack"

    @staticmethod
    def _alternar_disabled_en_carpeta(carpeta, nombre_entrada):
        if not carpeta or not nombre_entrada:
            return False
        ruta = os.path.join(carpeta, nombre_entrada)
        if not os.path.exists(ruta):
            return False
        try:
            if nombre_entrada.endswith(".disabled"):
                destino = os.path.join(carpeta, nombre_entrada[:-9])
            else:
                destino = os.path.join(carpeta, nombre_entrada + ".disabled")
            os.rename(ruta, destino)
            return True
        except OSError:
            return False

    @staticmethod
    def listar_resource_packs(rp_dir):
        salida = []
        if not os.path.isdir(rp_dir):
            return salida
        for name in os.listdir(rp_dir):
            if name == GestorContenidoInstancia.PACK_SISTEMA:
                continue
            if name.startswith("."):
                continue
            full = os.path.join(rp_dir, name)
            if not (os.path.isfile(full) or os.path.isdir(full)):
                continue
            if name.endswith(".disabled"):
                estado = "Desactivado"
                visible = name[:-9]
            else:
                estado = "Activo"
                visible = name
            salida.append({"archivo": name, "nombre_visible": visible, "estado": estado})
        return salida

    @staticmethod
    def listar_shaderpacks(sp_dir):
        return GestorContenidoInstancia.listar_resource_packs(sp_dir)

    @staticmethod
    def listar_datapacks(game_dir):
        salida = []
        saves = os.path.join(game_dir, "saves")
        if not os.path.isdir(saves):
            return salida
        for mundo in os.listdir(saves):
            dp = os.path.join(saves, mundo, "datapacks")
            if not os.path.isdir(dp):
                continue
            for name in os.listdir(dp):
                if name.startswith("."):
                    continue
                full = os.path.join(dp, name)
                if not os.path.isfile(full) and not os.path.isdir(full):
                    continue
                if name.endswith(".disabled"):
                    estado = "Desactivado"
                    visible = name[:-9]
                else:
                    estado = "Activo"
                    visible = name
                salida.append(
                    {
                        "archivo": name,
                        "nombre_visible": visible,
                        "mundo": mundo,
                        "estado": estado,
                    }
                )
        return salida

    @staticmethod
    def alternar_resourcepack(rp_dir, nombre_archivo):
        return GestorContenidoInstancia._alternar_disabled_en_carpeta(rp_dir, nombre_archivo)

    @staticmethod
    def alternar_shaderpack(sp_dir, nombre_archivo):
        return GestorContenidoInstancia._alternar_disabled_en_carpeta(sp_dir, nombre_archivo)

    @staticmethod
    def alternar_datapack(game_dir, mundo, nombre_archivo):
        dp = os.path.join(game_dir, "saves", mundo, "datapacks")
        return GestorContenidoInstancia._alternar_disabled_en_carpeta(dp, nombre_archivo)