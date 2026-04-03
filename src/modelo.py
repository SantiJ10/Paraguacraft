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
    @staticmethod
    def buscar_mods(query, version_actual, tipo_motor, tipo_proyecto):
        """Se conecta a Modrinth, hace la búsqueda y devuelve los resultados crudos."""
        facets = f'[["versions:{version_actual}"],["project_type:{tipo_proyecto}"]'
        # Shaders y Texturas no dependen de Fabric/Forge en Modrinth
        if tipo_proyecto in ["mod", "modpack"]:
            facets += f',["categories:{tipo_motor}"]'
        facets += ']'
        
        url = f'https://api.modrinth.com/v2/search?query={query}&limit=8&facets={facets}'
        
        try:
            r = requests.get(url, headers={"User-Agent": "ParaguacraftLauncher/1.0"}, timeout=10)
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
    def descargar_y_preparar(carpeta_server, ver_server, callback_progreso):
        try:
            callback_progreso(f"Buscando servidor para la versión {ver_server}...")
            
            # 1. Busca en el manifiesto oficial de Mojang
            manifest = requests.get("https://launchermeta.mojang.com/mc/game/version_manifest.json", timeout=10).json()
            version_url = next((v["url"] for v in manifest["versions"] if v["id"] == ver_server), None)
            
            if not version_url:
                return False, f"La versión '{ver_server}' no existe en Mojang.\nAsegurate de poner una versión oficial válida."
            
            # 2. Entra a la versión y saca el link del server.jar
            v_data = requests.get(version_url, timeout=10).json()
            if "server" not in v_data.get("downloads", {}):
                return False, "Esta versión no tiene un servidor oficial disponible."
                
            server_jar_url = v_data["downloads"]["server"]["url"]
            
            # 3. Descarga el archivo pesado
            callback_progreso(f"Descargando server.jar ({ver_server})...")
            r_jar = requests.get(server_jar_url, stream=True, timeout=30)
            r_jar.raise_for_status()
            
            with open(os.path.join(carpeta_server, "server.jar"), "wb") as f:
                for chunk in r_jar.iter_content(chunk_size=8192):
                    if chunk: f.write(chunk)
            
            # 4. Acepta la EULA
            with open(os.path.join(carpeta_server, "eula.txt"), "w") as f:
                f.write("eula=true")
            
            # 5. Buscador de Java
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            runtime_dir = os.path.join(mine_dir, "runtime")
            java_cmd = "java"
            
            if os.path.exists(runtime_dir):
                javas_encontrados = [os.path.join(root, "java.exe") for root, _, files in os.walk(runtime_dir) if "java.exe" in files]
                if javas_encontrados:
                    java_ideal = next((j for j in javas_encontrados if "delta" in j.lower() or "gamma" in j.lower() or "21" in j.lower()), javas_encontrados[0])
                    java_cmd = f'"{java_ideal}"'

            # 6. Descarga de Playit.gg
            callback_progreso("Inyectando túnel Playit.gg...")
            r_playit = requests.get("https://github.com/playit-cloud/playit-agent/releases/latest/download/playit-win_64.exe", stream=True, timeout=20)
            with open(os.path.join(carpeta_server, "playit.exe"), "wb") as f:
                shutil.copyfileobj(r_playit.raw, f)

            # 7. Creación del BAT
            bat_content = f"@echo off\necho Iniciando Tunel Playit...\nstart cmd /k playit.exe\necho Iniciando Servidor de Minecraft...\n{java_cmd} -Xmx4G -Xms4G -jar server.jar nogui\npause"
            with open(os.path.join(carpeta_server, "iniciar_server.bat"), "w", encoding="utf-8") as f:
                f.write(bat_content)
            
            callback_progreso("¡Servidor y Túnel listos!")
            return True, f"¡Todo listo en {carpeta_server}!\n\nAl ejecutar el .bat, se abrirán dos consolas:\n1. El Servidor de Minecraft.\n2. El túnel de Playit (Ahí te va a generar un link para que tus amigos entren directo)."

        except Exception as e:
            return False, str(e)

class GestorLocalMods:
    @staticmethod
    def obtener_lista_mods(mods_dir):
        """Devuelve una lista con el nombre de cada mod y su estado actual."""
        if not os.path.exists(mods_dir): return []
        mods = []
        for f in os.listdir(mods_dir):
            if f.endswith(".jar") or f.endswith(".jar.disabled"):
                estado = "Activo" if f.endswith(".jar") else "Desactivado"
                mods.append({"archivo": f, "estado": estado})
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