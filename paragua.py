import webview
import json
import os
import sys
import threading
import time
import re
import psutil
import math
import platform
import webbrowser
import shutil
import subprocess
import requests

VERSION = "2.0.0"  # Actualizar en cada release
GITHUB_REPO = "SantiJ10/Paraguacraft"  # usuario/repo en GitHub

# Credenciales oficiales
CLIENT_ID = "72fb7c48-c2f5-4d13-b0e7-9835b3b906c0"
REDIRECT_URI = "https://login.live.com/oauth20_desktop.srf"
DISCORD_APP_ID = "1487516329631154206"

FEATURED_VERSION_KEYS = ("26.1", "1.21", "1.20", "1.19", "1.18", "1.17", "1.16", "1.12", "1.8", "1.7")


def _tamano_carpeta(ruta):
    n = 0
    if not os.path.isdir(ruta):
        return 0
    for root, _, files in os.walk(ruta):
        for f in files:
            try:
                n += os.path.getsize(os.path.join(root, f))
            except OSError:
                pass
    return n


class Api:
    def __init__(self):
        self.ruta_config = "paraguacraft_config.json"
        self.ruta_sesion = "paraguacraft_session.json"
        self.ms_data = None
        self.rpc = None
        self.hilo_juego_activo = False
        self._telemetry_mc_bytes = None
        self._telemetry_mc_ts = 0

        self._servidor_proc = None
        self._servidor_carpeta = None
        self._servidor_log = []
        self._servidor_lock = threading.Lock()
        self._playit_proc = None
        self._playit_address = ""

        self.config_actual = {
            "usuario": "Invitado",
            "is_premium": False,
            "ram_asignada": 4,
            "gc_type": "G1GC",
            "opt_minimos": False,
            "papa_mode": False,
            "cerrar_al_jugar": True,
            "mostrar_consola": False,
            "backup_var": True,
            "limpiador_deep_var": False,
            "lan_distancia": False,
            "fabric_loader_version": "",
        }

        if os.path.exists(self.ruta_config):
            with open(self.ruta_config, "r") as f:
                self.config_actual.update(json.load(f))

        if os.path.exists(self.ruta_sesion):
            try:
                with open(self.ruta_sesion, "r") as f:
                    self.ms_data = json.load(f)
                    self.config_actual["usuario"] = f"{self.ms_data['name']} [PREMIUM]"
                    self.config_actual["is_premium"] = True
            except Exception:
                pass

        threading.Thread(target=self.iniciar_discord_rpc, daemon=True).start()

    def iniciar_discord_rpc(self):
        try:
            from pypresence import Presence

            self.rpc = Presence(DISCORD_APP_ID)
            self.rpc.connect()
            self.rpc.update(
                state="Navegando por el Launcher",
                details="Preparándose para jugar",
                large_image="logo",
                large_text="Paraguacraft",
            )
        except Exception:
            self.rpc = None

    def _rpc_menu(self):
        if self.rpc:
            try:
                self.rpc.update(
                    state="En el launcher",
                    details="Paraguacraft",
                    large_image="logo",
                    large_text="Paraguacraft",
                )
            except Exception:
                pass

    def get_usuario(self):
        return {"nombre": self.config_actual.get("usuario", "Invitado"), "premium": self.config_actual.get("is_premium", False)}

    def login_microsoft_paso1(self):
        try:
            from minecraft_launcher_lib.microsoft_account import get_login_url

            url = get_login_url(CLIENT_ID, REDIRECT_URI)
            webbrowser.open(url)
            return True
        except Exception as e:
            return str(e)

    def login_microsoft_paso2(self, url_respuesta):
        try:
            from minecraft_launcher_lib.microsoft_account import complete_login, url_contains_auth_code, get_auth_code_from_url

            if url_contains_auth_code(url_respuesta):
                auth_code = get_auth_code_from_url(url_respuesta)
                self.ms_data = complete_login(CLIENT_ID, None, REDIRECT_URI, auth_code)
                nombre_premium = f"{self.ms_data['name']} [PREMIUM]"
                self.config_actual["usuario"] = nombre_premium
                self.config_actual["is_premium"] = True
                with open(self.ruta_sesion, "w") as f:
                    json.dump(self.ms_data, f)
                self._guardar()
                webview.windows[0].evaluate_js(f"actualizarUiUsuario('{nombre_premium}', true)")
                return "EXITO"
            return "URL inválida."
        except Exception as e:
            return f"Error: {e}"

    def login_invitado(self, nombre):
        self.config_actual["usuario"] = nombre
        self.config_actual["is_premium"] = False
        self._guardar()
        webview.windows[0].evaluate_js(f"actualizarUiUsuario('{nombre}', false)")
        return nombre

    def cerrar_sesion(self):
        self.ms_data = None
        self.config_actual["usuario"] = "Invitado"
        self.config_actual["is_premium"] = False
        if os.path.exists(self.ruta_sesion):
            os.remove(self.ruta_sesion)
        self._guardar()
        return True

    def _guardar(self):
        with open(self.ruta_config, "w") as f:
            json.dump(self.config_actual, f)

    def get_settings(self):
        ram_total = max(1, math.floor(psutil.virtual_memory().total / (1024**3)))
        return {"config": self.config_actual, "ram_max": ram_total}

    def save_setting(self, key, value):
        self.config_actual[key] = value
        self._guardar()
        return True

    def get_telemetry(self):
        """CPU, RAM y disco (como panel de rendimiento / almacenamiento)."""
        ram = psutil.virtual_memory()
        disk = psutil.disk_usage(os.path.expanduser("~"))
        cpu = psutil.cpu_percent(interval=None)
        now = time.time()
        mc_bytes = self._telemetry_mc_bytes
        if now - self._telemetry_mc_ts > 8:
            try:
                import minecraft_launcher_lib

                mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                mc_bytes = _tamano_carpeta(mine_dir)
            except Exception:
                mc_bytes = 0
            self._telemetry_mc_bytes = mc_bytes
            self._telemetry_mc_ts = now
        return {
            "cpu_percent": round(cpu, 1),
            "ram_used_gb": round(ram.used / (1024**3), 2),
            "ram_total_gb": round(ram.total / (1024**3), 2),
            "ram_percent": ram.percent,
            "disk_free_gb": round(disk.free / (1024**3), 2),
            "disk_total_gb": round(disk.total / (1024**3), 2),
            "disk_percent": disk.percent,
            "minecraft_folder_mb": round((mc_bytes or 0) / (1024**2), 1),
        }

    def get_loaders_for_version(self, version_id):
        """Add-ons permitidos según la versión concreta (parche)."""
        loaders = ["Vanilla"]
        v = (version_id or "").strip()
        if not v:
            return loaders

        if v.startswith("26"):
            loaders.extend(["Fabric", "Fabric + Iris"])
            return loaders

        parts = v.split(".")
        try:
            major = int(parts[0])
            minor = int(parts[1]) if len(parts) > 1 and parts[1].isdigit() else 0
        except (ValueError, IndexError):
            return loaders

        if major != 1:
            return loaders

        if minor in (7, 8, 12) or 16 <= minor <= 20:
            loaders.append("OptiFine")
        if minor in (8, 12):
            loaders.append("Forge")
        if 16 <= minor <= 21:
            loaders.append("Fabric")
            loaders.append("Fabric + Iris")

        # orden estable: Vanilla, OptiFine, Forge, Fabric, Fabric + Iris
        orden = ["Vanilla", "OptiFine", "Forge", "Fabric", "Fabric + Iris"]
        vistos = set()
        out = []
        for o in orden:
            if o in loaders and o not in vistos:
                out.append(o)
                vistos.add(o)
        for x in loaders:
            if x not in vistos:
                out.append(x)
                vistos.add(x)
        return out

    def get_local_mods(self, version, motor):
        try:
            from core import carpeta_instancia_paraguacraft
            from src.modelo import GestorLocalMods
            import minecraft_launcher_lib

            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            folder = carpeta_instancia_paraguacraft(version.strip(), motor)
            mods_dir = os.path.join(mine_dir, "instancias", folder, "mods")
            lista = GestorLocalMods.obtener_lista_mods(mods_dir)
            return [{"archivo": m["archivo"], "estado": m["estado"]} for m in lista]
        except Exception as e:
            print("get_local_mods:", e)
            return []

    def toggle_local_mod(self, version, motor, nombre_archivo):
        try:
            from core import carpeta_instancia_paraguacraft
            from src.modelo import GestorLocalMods
            import minecraft_launcher_lib

            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            folder = carpeta_instancia_paraguacraft(version.strip(), motor)
            mods_dir = os.path.join(mine_dir, "instancias", folder, "mods")
            GestorLocalMods.alternar_estado_mod(mods_dir, nombre_archivo)
            return True
        except Exception as e:
            print("toggle_local_mod:", e)
            return False

    def _dir_instancia(self, version, motor):
        from core import carpeta_instancia_paraguacraft
        import minecraft_launcher_lib

        mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
        folder = carpeta_instancia_paraguacraft(version.strip(), motor)
        return os.path.join(mine_dir, "instancias", folder)

    def get_instance_content(self, version, motor, categoria):
        try:
            from src.modelo import GestorContenidoInstancia, GestorLocalMods

            base = self._dir_instancia(version, motor)
            cat = (categoria or "mods").lower()
            if cat == "mods":
                mods_dir = os.path.join(base, "mods")
                lista = GestorLocalMods.obtener_lista_mods(mods_dir)
                return [
                    {"archivo": m["archivo"], "nombre_visible": m["archivo"], "estado": m["estado"], "mundo": ""}
                    for m in lista
                ]
            if cat == "resourcepacks":
                return [
                    {**x, "mundo": ""}
                    for x in GestorContenidoInstancia.listar_resource_packs(os.path.join(base, "resourcepacks"))
                ]
            if cat == "shaders":
                return [
                    {**x, "mundo": ""}
                    for x in GestorContenidoInstancia.listar_shaderpacks(os.path.join(base, "shaderpacks"))
                ]
            if cat == "datapacks":
                return GestorContenidoInstancia.listar_datapacks(base)
            return []
        except Exception as e:
            print("get_instance_content:", e)
            return []

    def toggle_instance_content(self, version, motor, categoria, nombre_archivo, mundo=None):
        try:
            from src.modelo import GestorContenidoInstancia, GestorLocalMods

            base = self._dir_instancia(version, motor)
            cat = (categoria or "mods").lower()
            if cat == "mods":
                mods_dir = os.path.join(base, "mods")
                GestorLocalMods.alternar_estado_mod(mods_dir, nombre_archivo)
                return True
            if cat == "resourcepacks":
                return GestorContenidoInstancia.alternar_resourcepack(os.path.join(base, "resourcepacks"), nombre_archivo)
            if cat == "shaders":
                return GestorContenidoInstancia.alternar_shaderpack(os.path.join(base, "shaderpacks"), nombre_archivo)
            if cat == "datapacks" and mundo:
                return GestorContenidoInstancia.alternar_datapack(base, mundo, nombre_archivo)
            return False
        except Exception as e:
            print("toggle_instance_content:", e)
            return False

    def open_instance_folder(self, version, motor, subcarpeta):
        try:
            base = self._dir_instancia(version, motor)
            sub = (subcarpeta or "mods").lower()
            if sub == "mods":
                path = os.path.join(base, "mods")
            elif sub == "resourcepacks":
                path = os.path.join(base, "resourcepacks")
            elif sub == "shaders" or sub == "shaderpacks":
                path = os.path.join(base, "shaderpacks")
            elif sub == "datapacks":
                path = os.path.join(base, "saves")
            else:
                path = os.path.join(base, "mods")
            os.makedirs(path, exist_ok=True)
            if platform.system() == "Windows":
                os.startfile(path)
            elif sys.platform == "darwin":
                subprocess.Popen(["open", path])
            else:
                subprocess.Popen(["xdg-open", path])
            return True
        except Exception as e:
            print("open_instance_folder:", e)
            return False

    def get_fabric_loader_versions(self):
        try:
            r = requests.get("https://meta.fabricmc.net/v2/versions/loader", timeout=12)
            if r.status_code != 200:
                return []
            return [x["version"] for x in r.json()[:60]]
        except Exception:
            return []

    def get_assets_path(self):
        ruta_assets = os.path.join(os.getcwd(), "assets")
        return f"file:///{ruta_assets.replace(chr(92), '/')}"

    def get_versiones_oficiales(self):
        try:
            import minecraft_launcher_lib

            versiones = minecraft_launcher_lib.utils.get_version_list()
            lista_oficiales = [v["id"] for v in versiones if v["type"] == "release"]

            grupos = {}
            for vid in lista_oficiales:
                partes = vid.split(".")
                if len(partes) >= 2:
                    mayor = f"{partes[0]}.{partes[1]}"
                    grupos.setdefault(mayor, []).append(vid)

            grupos["26.1"] = ["26.1.1"]

            featured = set()
            for k in FEATURED_VERSION_KEYS:
                for vid in grupos.get(k, []):
                    featured.add(vid)

            otras = [vid for vid in lista_oficiales if vid not in featured]
            grupos["otras"] = otras
            return grupos
        except Exception as e:
            print("Error obteniendo versiones de Mojang:", e)
            return {}

    def _hilo_ninja_renombrar(self, version_jugada):
        if sys.platform != "win32":
            return
        try:
            import win32gui
        except ImportError:
            return
        while self.hilo_juego_activo:
            time.sleep(1.1)

            def callback(hwnd, _):
                if not win32gui.IsWindowVisible(hwnd):
                    return
                titulo = win32gui.GetWindowText(hwnd)
                if "Minecraft" in titulo and "Paraguacraft Launcher" not in titulo:
                    win32gui.SetWindowText(hwnd, f"Paraguacraft {version_jugada}")

            try:
                win32gui.EnumWindows(callback, None)
            except Exception:
                pass

    def _hilo_discord_rpc_dinamico(self, version_jugada, motor_elegido, usuario):
        if not self.rpc:
            return
        try:
            import minecraft_launcher_lib
            from core import carpeta_instancia_paraguacraft

            version_base = version_jugada.strip()
            folder = carpeta_instancia_paraguacraft(version_base, motor_elegido)
            log_path = os.path.join(
                minecraft_launcher_lib.utils.get_minecraft_directory(),
                "instancias",
                folder,
                "logs",
                "latest.log",
            )
            try:
                if os.path.exists(log_path):
                    os.remove(log_path)
            except OSError:
                pass

            estado_actual = "En el menú principal"
            try:
                self.rpc.update(
                    state=estado_actual,
                    details=f"👤 {usuario} | 🎮 {version_jugada} | {motor_elegido}",
                    large_image="logo",
                    large_text="Paraguacraft",
                )
            except Exception:
                pass

            while not os.path.exists(log_path) and self.hilo_juego_activo:
                time.sleep(1)

            if not os.path.exists(log_path):
                return

            with open(log_path, "r", encoding="utf-8", errors="ignore") as f:
                while self.hilo_juego_activo:
                    linea = f.readline()
                    if not linea:
                        time.sleep(0.45)
                        try:
                            f.seek(f.tell())
                        except OSError:
                            pass
                        continue
                    if "Local game hosted on" in linea:
                        m = re.search(r"\[(.*?)\]", linea)
                        if m:
                            estado_actual = f"🏠 LAN: {m.group(1)}"
                    elif "Connecting to" in linea:
                        ip = linea.split("Connecting to ")[1].split(",")[0].split(":")[0].strip()
                        estado_actual = f"🌐 Multijugador: {ip}"
                    elif "Starting integrated minecraft server" in linea:
                        if "LAN" not in estado_actual:
                            estado_actual = "🌍 Mundo local"
                    elif "Disconnecting from" in linea or "Stopping server" in linea:
                        estado_actual = "En el menú principal"
                    try:
                        self.rpc.update(
                            state=estado_actual,
                            details=f"👤 {usuario} | 🎮 {version_jugada}",
                            large_image="logo",
                            large_text="Paraguacraft",
                        )
                    except Exception:
                        break
        except Exception as e:
            print("Error RPC dinámico:", e)

    def lanzar_juego(self, version, motor, server_ip=""):
        print(f"Lanzando {version} con {motor}. AutoJoin: {server_ip}")
        self.hilo_juego_activo = True

        if self.rpc:
            try:
                self.rpc.update(
                    state=f"Jugando {version}",
                    details=f"{motor}",
                    large_image="logo",
                    large_text="Paraguacraft",
                )
            except Exception:
                pass

        username_limpio = self.config_actual.get("usuario", "Invitado").replace(" [PREMIUM]", "")

        threading.Thread(target=self._hilo_ninja_renombrar, args=(version,), daemon=True).start()
        threading.Thread(
            target=self._hilo_discord_rpc_dinamico,
            args=(version, motor, username_limpio),
            daemon=True,
        ).start()

        def hilo_lanzar():
            try:
                from core import lanzar_minecraft

                webview.windows[0].evaluate_js(f"actualizarEstadoPanel({json.dumps('Iniciando motor...')})")

                uuid_real = self.ms_data["id"] if self.ms_data else None
                token_real = self.ms_data["access_token"] if self.ms_data else None

                fl_raw = (self.config_actual.get("fabric_loader_version") or "").strip()
                fabric_override = fl_raw if fl_raw and "fabric" in motor.lower() else None

                lanzar_minecraft(
                    version=version,
                    username=username_limpio,
                    max_ram=f"{self.config_actual.get('ram_asignada', 4)}G",
                    gc_type=self.config_actual.get("gc_type", "G1GC"),
                    optimizar=self.config_actual.get("opt_minimos", False),
                    motor_elegido=motor,
                    papa_mode=self.config_actual.get("papa_mode", False),
                    usar_mesa=False,
                    mostrar_consola=self.config_actual.get("mostrar_consola", False),
                    uuid_real=uuid_real,
                    token_real=token_real,
                    lan_distancia=self.config_actual.get("lan_distancia", False),
                    fabric_loader_override=fabric_override,
                    progress_callback=lambda m: webview.windows[0].evaluate_js(f"actualizarEstadoPanel({json.dumps(m)})"),
                )

                webview.windows[0].evaluate_js(f"actualizarEstadoPanel({json.dumps('¡Juego cerrado!')})")
                webview.windows[0].evaluate_js("resetLaunchButton()")
                self._rpc_menu()
            except Exception as e:
                print(f"Error crítico: {e}")
                webview.windows[0].evaluate_js(f"actualizarEstadoPanel({json.dumps('Error: ' + str(e)[:80])})")
                webview.windows[0].evaluate_js("resetLaunchButton()")
                self._rpc_menu()
            finally:
                self.hilo_juego_activo = False

        threading.Thread(target=hilo_lanzar, daemon=True).start()
        return "Iniciando"


    def seleccionar_carpeta_servidor(self):
        try:
            carpeta = webview.windows[0].create_file_dialog(webview.FOLDER_DIALOG)
            if carpeta and len(carpeta) > 0:
                return carpeta[0]
            return ""
        except Exception:
            return ""

    def crear_servidor(self, version, carpeta):
        try:
            from src.modelo import CreadorServidor
            os.makedirs(carpeta, exist_ok=True)
            self._servidor_carpeta = carpeta

            def cb(msg):
                with self._servidor_lock:
                    self._servidor_log.append(f"[SETUP] {msg}")
                    if len(self._servidor_log) > 200:
                        self._servidor_log = self._servidor_log[-200:]
                try:
                    webview.windows[0].evaluate_js(f"servidorAppendLog({json.dumps('[SETUP] ' + msg)})")
                except Exception:
                    pass

            def _hilo():
                exito, _ = CreadorServidor.descargar_y_preparar(carpeta, version, cb)
                if exito:
                    props_path = os.path.join(carpeta, "server.properties")
                    if os.path.exists(props_path):
                        with open(props_path, "r") as f:
                            content = f.read()
                        content = re.sub(r"online-mode=.*", "online-mode=false", content)
                        with open(props_path, "w") as f:
                            f.write(content)
                    else:
                        with open(props_path, "w") as f:
                            f.write("online-mode=false\nserver-port=25565\nmax-players=20\n")
                    cb("online-mode=false → premium y no-premium pueden conectarse")
                    try:
                        webview.windows[0].evaluate_js("actualizarEstadoServidor()")
                    except Exception:
                        pass
                else:
                    try:
                        webview.windows[0].evaluate_js(f"servidorAppendLog({json.dumps('[ERROR] Fallo al crear servidor')})")
                    except Exception:
                        pass

            threading.Thread(target=_hilo, daemon=True).start()
            return {"ok": True, "carpeta": carpeta}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def iniciar_servidor(self, carpeta=None):
        if self._servidor_proc and self._servidor_proc.poll() is None:
            return {"ok": False, "error": "El servidor ya est\u00e1 corriendo."}
        carpeta = carpeta or self._servidor_carpeta
        if not carpeta or not os.path.exists(os.path.join(carpeta, "server.jar")):
            return {"ok": False, "error": "No se encontr\u00f3 server.jar. Prim\u00e9ro cre\u00e1 el servidor."}
        try:
            import minecraft_launcher_lib
            mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            runtime_dir = os.path.join(mc_dir, "runtime")
            java_cmd = "java"
            if os.path.exists(runtime_dir):
                javas = [os.path.join(r, "java.exe") for r, _, fs in os.walk(runtime_dir) if "java.exe" in fs]
                if javas:
                    java_ideal = next((j for j in javas if "21" in j or "delta" in j.lower() or "gamma" in j.lower()), javas[0])
                    java_cmd = java_ideal
            self._servidor_carpeta = carpeta
            with self._servidor_lock:
                self._servidor_log.clear()
            flags = subprocess.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
            self._servidor_proc = subprocess.Popen(
                [java_cmd, "-Xmx4G", "-Xms1G", "-jar", "server.jar", "nogui"],
                cwd=carpeta, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT, text=True, encoding="utf-8",
                errors="replace", creationflags=flags,
            )

            def _stream():
                for line in iter(self._servidor_proc.stdout.readline, ""):
                    line = line.rstrip()
                    if not line:
                        continue
                    with self._servidor_lock:
                        self._servidor_log.append(line)
                        if len(self._servidor_log) > 200:
                            self._servidor_log = self._servidor_log[-200:]
                    try:
                        webview.windows[0].evaluate_js(f"servidorAppendLog({json.dumps(line)})")
                    except Exception:
                        pass
                try:
                    webview.windows[0].evaluate_js("actualizarEstadoServidor()")
                    webview.windows[0].evaluate_js(f"servidorAppendLog({json.dumps('[SERVER] Servidor detenido.')})")
                except Exception:
                    pass

            threading.Thread(target=_stream, daemon=True).start()
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def detener_servidor(self):
        try:
            if self._servidor_proc and self._servidor_proc.poll() is None:
                try:
                    self._servidor_proc.stdin.write("stop\n")
                    self._servidor_proc.stdin.flush()
                except Exception:
                    pass
                try:
                    self._servidor_proc.wait(timeout=8)
                except Exception:
                    self._servidor_proc.kill()
            self._servidor_proc = None
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def enviar_comando_servidor(self, cmd):
        try:
            if self._servidor_proc and self._servidor_proc.poll() is None:
                self._servidor_proc.stdin.write(cmd + "\n")
                self._servidor_proc.stdin.flush()
                return True
            return False
        except Exception:
            return False

    def iniciar_playitgg(self, carpeta=None):
        if self._playit_proc and self._playit_proc.poll() is None:
            return {"ok": False, "error": "playit.gg ya est\u00e1 corriendo."}
        carpeta = carpeta or self._servidor_carpeta
        playit_exe = os.path.join(carpeta or "", "playit.exe")
        if not os.path.exists(playit_exe):
            return {"ok": False, "error": "No se encontr\u00f3 playit.exe. Prim\u00e9ro cre\u00e1 el servidor."}
        try:
            self._playit_address = ""
            flags = subprocess.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
            self._playit_proc = subprocess.Popen(
                [playit_exe], cwd=carpeta, stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT, text=True, encoding="utf-8",
                errors="replace", creationflags=flags,
            )

            def _stream():
                import re as _re
                for line in iter(self._playit_proc.stdout.readline, ""):
                    line = line.rstrip()
                    if not line:
                        continue
                    m = _re.search(r"([\w\-.]+\.ply\.gg:\d+)", line)
                    if not m:
                        m = _re.search(r"address[:\s]+([\w\-.]+:\d+)", line, _re.IGNORECASE)
                    if m and not self._playit_address:
                        self._playit_address = m.group(1)
                        try:
                            webview.windows[0].evaluate_js(f"actualizarDireccionPlayit({json.dumps(self._playit_address)})")
                        except Exception:
                            pass
                    try:
                        webview.windows[0].evaluate_js(f"servidorAppendLog({json.dumps('[PLAYIT] ' + line)})")
                    except Exception:
                        pass
                try:
                    webview.windows[0].evaluate_js("actualizarEstadoServidor()")
                except Exception:
                    pass

            threading.Thread(target=_stream, daemon=True).start()
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def detener_playitgg(self):
        try:
            if self._playit_proc and self._playit_proc.poll() is None:
                self._playit_proc.kill()
            self._playit_proc = None
            self._playit_address = ""
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_estado_servidor(self):
        corriendo = self._servidor_proc is not None and self._servidor_proc.poll() is None
        playit_corriendo = self._playit_proc is not None and self._playit_proc.poll() is None
        with self._servidor_lock:
            log_reciente = list(self._servidor_log[-60:])
        return {
            "corriendo": corriendo,
            "carpeta": self._servidor_carpeta or "",
            "playit_corriendo": playit_corriendo,
            "playit_address": self._playit_address,
            "log": log_reciente,
        }


    def verificar_actualizacion(self):
        try:
            url = f"https://api.github.com/repos/{GITHUB_REPO}/releases/latest"
            r = requests.get(url, timeout=8, headers={"User-Agent": "Paraguacraft-Launcher"})
            data = r.json()
            tag = data.get("tag_name", "").lstrip("v")
            if not tag:
                return {"actualizar": False, "version_actual": VERSION}
            def _ver_tuple(v):
                try: return tuple(int(x) for x in v.split("."))
                except: return (0,)
            if _ver_tuple(tag) > _ver_tuple(VERSION):
                exe_url = None
                for asset in data.get("assets", []):
                    if asset.get("name", "").lower().endswith(".exe"):
                        exe_url = asset.get("browser_download_url")
                        break
                return {
                    "actualizar": True,
                    "version_actual": VERSION,
                    "version_nueva": tag,
                    "url": exe_url,
                    "notas": data.get("body", "")[:600],
                }
            return {"actualizar": False, "version_actual": VERSION}
        except Exception as e:
            return {"actualizar": False, "version_actual": VERSION, "error": str(e)}

    def aplicar_actualizacion(self, download_url):
        import tempfile
        if not getattr(sys, "frozen", False):
            return {"ok": False, "error": "Solo funciona en el ejecutable compilado (.exe)."}
        if not download_url:
            return {"ok": False, "error": "URL de descarga no disponible."}
        def _hilo():
            try:
                tmp = os.path.join(tempfile.gettempdir(), "Paraguacraft_update.exe")
                exe_actual = sys.executable
                r = requests.get(download_url, stream=True, timeout=120)
                r.raise_for_status()
                with open(tmp, "wb") as f:
                    for chunk in r.iter_content(65536):
                        if chunk:
                            f.write(chunk)
                bat = os.path.join(tempfile.gettempdir(), "paragua_updater.bat")
                with open(bat, "w") as f:
                    f.write("@echo off\n")
                    f.write("timeout /t 2 /nobreak > nul\n")
                    f.write(f'move /y "{tmp}" "{exe_actual}"\n')
                    f.write(f'start "" "{exe_actual}"\n')
                    f.write('del "%~f0"\n')
                subprocess.Popen(
                    ["cmd", "/c", bat],
                    creationflags=subprocess.CREATE_NO_WINDOW,
                    close_fds=True,
                )
                os._exit(0)
            except Exception as e:
                print(f"[Updater] Error: {e}")
        threading.Thread(target=_hilo, daemon=True).start()
        return {"ok": True}


if __name__ == "__main__":
    api = Api()
    _base = sys._MEIPASS if getattr(sys, "frozen", False) else os.path.dirname(os.path.abspath(__file__))
    html_path = os.path.join(_base, "web", "index.html")

    ventana = webview.create_window(
        "Paraguacraft Launcher",
        url=html_path,
        js_api=api,
        width=1150,
        height=700,
        min_size=(950, 600),
        background_color="#101010",
    )
    webview.start(debug=False)
