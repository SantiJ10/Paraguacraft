import webview
import json
import os
import sys
if sys.stdout is None: sys.stdout = open(os.devnull, "w")
if sys.stderr is None: sys.stderr = open(os.devnull, "w")
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
import socket
try:
    import minecraft_launcher_lib
except ImportError:
    minecraft_launcher_lib = None
import struct
import zipfile
import datetime
import traceback
import socketserver
from http.server import BaseHTTPRequestHandler
try:
    from core import lanzar_minecraft as _lanzar_minecraft_ref  # noqa – forces PyInstaller to bundle core
except Exception:
    _lanzar_minecraft_ref = None

VERSION = "4.7.0"  # Actualizar en cada release
GITHUB_REPO = "SantiJ10/Paraguacraft"  # usuario/repo en GitHub

try:
    import credentials as _cred
    _GEMINI_API_KEY   = _cred.GEMINI_API_KEY
    _CF_API_KEY       = _cred.CF_API_KEY
    _SP_CLIENT_ID     = _cred.SP_CLIENT_ID
    _SP_CLIENT_SECRET = _cred.SP_CLIENT_SECRET
    _YOUTUBE_API_KEY  = getattr(_cred, "YOUTUBE_API_KEY", "")
except (ImportError, AttributeError):
    _GEMINI_API_KEY   = ""
    _CF_API_KEY       = ""
    _SP_CLIENT_ID     = ""
    _SP_CLIENT_SECRET = ""
    _YOUTUBE_API_KEY  = ""

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
        self._gpu_name = None
        self._gpu_cache = {}
        self._gpu_cache_ts = 0

        self._servidor_proc = None
        self._servidor_carpeta = None
        self._servidor_tipo = 'paper'
        self._servidor_log = []
        self._servidor_lock = threading.Lock()
        self._servidor_creando = False
        self._jugadores_online = set()
        self._playit_proc = None
        self._playit_address = ""
        self._playit_bedrock_address = ""
        self._playit_bedrock_custom = ""
        self._musica_proc = None
        self._stats_file = "paraguacraft_stats.json"
        self._cache_dir = os.path.join(os.path.expanduser("~"), ".paraguacraft_cache")
        os.makedirs(self._cache_dir, exist_ok=True)
        self._game_status = {"running": False, "status": "idle"}
        self._modo_offline = False
        self._mod_dl_progress = {"pct": 0, "nombre": ""}
        self._spotify_token = None
        self._spotify_refresh = None

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
            "backup_auto_horas": 0,
            "ultimo_backup_auto": "",
            "limpiador_deep_var": False,
            "lan_distancia": False,
            "fabric_loader_version": "",
            "java_custom_path": "",
            "launch_behavior": "minimize",
            "auto_update_check": True,
            "discord_rpc": True,
            "discord_rpc_version": True,
            "discord_rpc_time": True,
            "fondo_animado": "",
            "spotify_client_id": _SP_CLIENT_ID,
            "spotify_client_secret": _SP_CLIENT_SECRET,
        }

        if os.path.exists(self.ruta_config):
            with open(self.ruta_config, "r") as f:
                self.config_actual.update(json.load(f))
        saved_refresh = self.config_actual.get("spotify_refresh_token", "")
        if saved_refresh:
            self._spotify_refresh = saved_refresh
        saved_srv = self.config_actual.get("srv_carpeta", "")
        if saved_srv and os.path.exists(saved_srv):
            self._servidor_carpeta = saved_srv

        if os.path.exists(self.ruta_sesion):
            try:
                with open(self.ruta_sesion, "r") as f:
                    self.ms_data = json.load(f)
                    self.config_actual["usuario"] = f"{self.ms_data['name']} [PREMIUM]"
                    self.config_actual["is_premium"] = True
            except Exception:
                pass

        threading.Thread(target=self.iniciar_discord_rpc, daemon=True).start()
        threading.Thread(target=self._iniciar_check_internet, daemon=True).start()

    def iniciar_discord_rpc(self):
        if not self.config_actual.get('discord_rpc', True):
            return
        def _intentar_conectar():
            from pypresence import Presence
            delays = [2, 4, 8, 15, 30]  # reintentos con backoff si Discord tarda en abrir
            for i, delay in enumerate([0] + delays):
                if delay:
                    time.sleep(delay)
                if not self.config_actual.get('discord_rpc', True):
                    return
                try:
                    rpc = Presence(DISCORD_APP_ID)
                    rpc.connect()
                    self.rpc = rpc
                    usuario = self.config_actual.get("usuario", "Invitado").replace(" [PREMIUM]", "")
                    self.rpc.update(
                        state=f"Conectado como {usuario}",
                        details="En el launcher",
                        large_image="logo",
                        large_text="Paraguacraft",
                        small_image="logo",
                        small_text="Launcher Paraguacraft",
                        start=int(time.time()),
                    )
                    return  # conexión exitosa
                except Exception:
                    self.rpc = None
        threading.Thread(target=_intentar_conectar, daemon=True).start()

    def _rpc_menu(self):
        if self.rpc:
            try:
                usuario = self.config_actual.get("usuario", "Invitado").replace(" [PREMIUM]", "")
                self.rpc.update(
                    state=f"Conectado como {usuario}",
                    details="Explorando el launcher",
                    large_image="logo",
                    large_text="Paraguacraft",
                    start=int(time.time()),
                )
            except Exception:
                pass

    def _rpc_jugando(self, version, motor):
        if not self.rpc or not self.config_actual.get('discord_rpc', True):
            return
        try:
            usuario = self.config_actual.get("usuario", "Invitado").replace(" [PREMIUM]", "")
            show_ver = self.config_actual.get('discord_rpc_version', True)
            show_time = self.config_actual.get('discord_rpc_time', True)
            state = f"{version} · {motor}" if show_ver else "Jugando Minecraft"
            self.rpc.update(
                state=state,
                details=f"Jugando como {usuario}",
                large_image="logo",
                large_text=f"Paraguacraft {version}",
                small_image="logo",
                small_text=motor,
                start=int(time.time()) if show_time else None,
            )
        except Exception:
            pass

    # ── MODO OFFLINE ────────────────────────────────────────────────────────
    def _check_internet(self):
        import socket as _sock
        try:
            s = _sock.socket(_sock.AF_INET, _sock.SOCK_STREAM)
            s.settimeout(3)
            s.connect(("8.8.8.8", 53))
            s.close()
            return True
        except Exception:
            return False

    def _iniciar_check_internet(self):
        import time as _t
        while True:
            online = self._check_internet()
            was_offline = self._modo_offline
            self._modo_offline = not online
            if was_offline != self._modo_offline:
                try:
                    estado = 'false' if online else 'true'
                    webview.windows[0].evaluate_js(f'actualizarBadgeOffline({estado})')
                except Exception:
                    pass
            _t.sleep(30)

    def get_modo_offline(self):
        return {"offline": self._modo_offline}

    # ── NOTIFICACIONES WINDOWS ───────────────────────────────────────────────
    def _mostrar_notificacion(self, titulo, mensaje):
        try:
            import subprocess as _sp
            t = titulo.replace("'", "''").replace('"', '')
            m = mensaje.replace("'", "''").replace('"', '')
            ps = (
                "Add-Type -AssemblyName System.Windows.Forms;"
                "$n=New-Object System.Windows.Forms.NotifyIcon;"
                "$n.Icon=[System.Drawing.SystemIcons]::Information;"
                "$n.Visible=$true;"
                f"$n.ShowBalloonTip(4000,'{t}','{m}',"
                "[System.Windows.Forms.ToolTipIcon]::None);"
                "Start-Sleep -Seconds 5;$n.Dispose()"
            )
            _sp.Popen(
                ['powershell', '-WindowStyle', 'Hidden', '-ExecutionPolicy', 'Bypass', '-Command', ps],
                creationflags=0x08000000
            )
        except Exception:
            pass

    # ── ANÁLISIS DE CRASH ────────────────────────────────────────────────────
    def _analizar_crash_report(self, version, motor, since_ts=0):
        try:
            import minecraft_launcher_lib as _mcl
            import re as _re
            mc_dir = _mcl.utils.get_minecraft_directory()
            dirs_a_revisar = []
            try:
                from core import carpeta_instancia_paraguacraft as _cia
                dirs_a_revisar.append(os.path.join(mc_dir, "instancias", _cia(version, motor), "crash-reports"))
            except Exception:
                pass
            dirs_a_revisar.append(os.path.join(mc_dir, "crash-reports"))
            crash_path = None
            for d in dirs_a_revisar:
                if not os.path.isdir(d):
                    continue
                archivos = [
                    os.path.join(d, f) for f in os.listdir(d)
                    if f.endswith('.txt') and os.path.getmtime(os.path.join(d, f)) >= since_ts
                ]
                if archivos:
                    crash_path = max(archivos, key=os.path.getmtime)
                    break
            if not crash_path:
                return None
            with open(crash_path, 'r', encoding='utf-8', errors='ignore') as _f:
                content = _f.read()
            result = {
                "archivo": os.path.basename(crash_path),
                "descripcion": "",
                "causa": "Error desconocido",
                "mods_sospechosos": [],
                "java": "",
                "sugerencia": "Revisá el crash report completo en crash-reports/."
            }
            for line in content.splitlines():
                l = line.strip()
                if l.startswith("Description:") and not result["descripcion"]:
                    result["descripcion"] = l.replace("Description:", "").strip()
                elif "Java Version:" in l and not result["java"]:
                    result["java"] = l.split(":", 1)[-1].strip()
            if "OutOfMemoryError" in content:
                result["causa"] = "OutOfMemoryError — Memoria insuficiente"
                result["sugerencia"] = "Subí la RAM asignada en Ajustes → Juego & Rendimiento."
            elif "StackOverflowError" in content:
                result["causa"] = "StackOverflowError"
                result["sugerencia"] = "Posible mod con bucle infinito. Deshabilitalos uno a uno."
            elif "ClassNotFoundException" in content or "NoClassDefFoundError" in content:
                result["causa"] = "Clase Java no encontrada — mod incompatible"
                result["sugerencia"] = "Actualizá los mods a versiones compatibles con este loader."
            elif "Pixel format not accelerated" in content or "EXCEPTION_ACCESS_VIOLATION" in content:
                result["causa"] = "Fallo de GPU / OpenGL"
                result["sugerencia"] = "Actualizá los drivers de tu GPU o desactivá shaders."
            elif "Failed to launch" in content or "Could not find" in content:
                result["causa"] = "Archivos de juego faltantes o corruptos"
                result["sugerencia"] = "Reinstalá esta versión desde el gestor de instancias."
            elif "Login failed" in content or "Invalid session" in content:
                result["causa"] = "Sesión inválida"
                result["sugerencia"] = "Cerrá sesión y volvé a iniciar en el launcher."
            jars = list(dict.fromkeys(_re.findall(r'[\w\-\.]+\.jar', content)))[:6]
            result["mods_sospechosos"] = jars
            return result
        except Exception:
            return None

    # ── PERFIL DE JAVA ───────────────────────────────────────────────────────
    def elegir_java_path(self):
        try:
            result = webview.windows[0].create_file_dialog(
                webview.OPEN_DIALOG,
                allow_multiple=False,
                file_types=("Ejecutable Java (*.exe)", "*.exe")
            )
            if result and len(result) > 0:
                path = result[0]
                self.config_actual["java_custom_path"] = path
                threading.Thread(target=self._guardar, daemon=True).start()
                return {"ok": True, "path": path}
            return {"ok": False, "path": ""}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def actualizar_rpc(self, estado, detalle=""):
        if self.rpc:
            try:
                self.rpc.update(
                    state=estado,
                    details=detalle or "Paraguacraft",
                    large_image="logo",
                    large_text="Paraguacraft",
                    start=int(time.time()),
                )
            except Exception:
                pass
        return {"ok": True}

    def get_usuario(self):
        return {"nombre": self.config_actual.get("usuario", "Invitado"), "premium": self.config_actual.get("is_premium", False)}

    def _refresh_ms_token(self):
        if not self.ms_data:
            return
        try:
            from minecraft_launcher_lib.microsoft_account import refresh_authorization_token
            new_data = refresh_authorization_token(CLIENT_ID, None, REDIRECT_URI, self.ms_data["refresh_token"])
            if new_data and "access_token" in new_data:
                self.ms_data.update(new_data)
                with open(self.ruta_sesion, "w") as f:
                    json.dump(self.ms_data, f)
        except Exception as e:
            print("[MS Refresh] Error al refrescar token:", e)

    def login_microsoft_paso1(self):
        try:
            from minecraft_launcher_lib.microsoft_account import get_login_url
            url = get_login_url(CLIENT_ID, REDIRECT_URI)
            try:
                webbrowser.open(url)
            except Exception:
                import os, subprocess
                try:
                    os.startfile(url)
                except Exception:
                    subprocess.Popen(['cmd', '/c', 'start', '', url], shell=False)
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
                try:
                    _ss = []
                    if os.path.exists(self._SESIONES_PATH):
                        with open(self._SESIONES_PATH, "r") as _f: _ss = json.load(_f)
                    _idx = next((i for i,s in enumerate(_ss) if s.get("id")==self.ms_data.get("id")), None)
                    if _idx is not None: _ss[_idx] = self.ms_data
                    else: _ss.append(self.ms_data)
                    with open(self._SESIONES_PATH, "w") as _f: json.dump(_ss, _f)
                except Exception: pass
                self._guardar()
                return {"ok": True, "nombre": nombre_premium}
            return {"ok": False, "error": "URL inválida."}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def login_invitado(self, nombre):
        nombre = (nombre or "").strip()
        self.config_actual["usuario"] = nombre
        self.config_actual["is_premium"] = False
        invitados = self.config_actual.get("cuentas_invitado", [])
        if nombre and not any(g["nombre"] == nombre for g in invitados):
            invitados.append({"nombre": nombre})
            self.config_actual["cuentas_invitado"] = invitados
        self._guardar()
        return nombre

    def cerrar_sesion(self):
        self.ms_data = None
        self.config_actual["usuario"] = "Invitado"
        self.config_actual["is_premium"] = False
        if os.path.exists(self.ruta_sesion):
            os.remove(self.ruta_sesion)
        # _SESIONES_PATH se conserva intencionalmente para multi-cuenta
        self._guardar()
        return True

    def modo_invitado(self, nombre):
        """Cambia al modo invitado SIN cerrar ni eliminar sesiones premium guardadas."""
        self.ms_data = None
        nombre = nombre.strip() or "Invitado"
        self.config_actual["usuario"] = nombre
        self.config_actual["is_premium"] = False
        self._guardar()
        return {"ok": True, "nombre": nombre}

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

    def _get_gpu_info(self):
        """Returns cached GPU name + utilisation. Refresh every 2 s."""
        now = time.time()
        _NO_WIN = 0x08000000
        if self._gpu_name is None:
            try:
                r = subprocess.run(
                    ["powershell", "-Command",
                     "(Get-WmiObject Win32_VideoController | Select-Object -First 1).Name"],
                    capture_output=True, text=True, timeout=4, creationflags=_NO_WIN
                )
                self._gpu_name = r.stdout.strip() or "GPU"
            except Exception:
                self._gpu_name = "GPU"
        if now - self._gpu_cache_ts > 2:
            try:
                r = subprocess.run(
                    ["nvidia-smi",
                     "--query-gpu=utilization.gpu,memory.used,memory.total",
                     "--format=csv,noheader,nounits"],
                    capture_output=True, text=True, timeout=3, creationflags=_NO_WIN
                )
                parts = r.stdout.strip().split(",")
                if len(parts) >= 3:
                    self._gpu_cache = {
                        "gpu_usage": int(parts[0].strip()),
                        "gpu_vram_used": int(parts[1].strip()),
                        "gpu_vram_total": int(parts[2].strip()),
                    }
            except Exception:
                pass
            self._gpu_cache_ts = now
        return {"gpu_name": self._gpu_name, **self._gpu_cache}

    def get_telemetry(self):
        """CPU, RAM, GPU y disco."""
        ram = psutil.virtual_memory()
        disk = psutil.disk_usage(os.path.expanduser("~"))
        cpu = psutil.cpu_percent(interval=None)
        now = time.time()
        mc_bytes = self._telemetry_mc_bytes
        mc_dir = ""
        if now - self._telemetry_mc_ts > 8:
            try:
                import minecraft_launcher_lib
                mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                mc_bytes = _tamano_carpeta(mc_dir)
            except Exception:
                mc_bytes = 0
            self._telemetry_mc_bytes = mc_bytes
            self._telemetry_mc_ts = now
        gpu = self._get_gpu_info()
        return {
            "cpu_percent": round(cpu, 1),
            "ram_used_gb": round(ram.used / (1024**3), 2),
            "ram_total_gb": round(ram.total / (1024**3), 2),
            "ram_percent": ram.percent,
            "disk_free_gb": round(disk.free / (1024**3), 2),
            "disk_total_gb": round(disk.total / (1024**3), 2),
            "disk_percent": disk.percent,
            "minecraft_folder_mb": round((mc_bytes or 0) / (1024**2), 1),
            "mc_dir": mc_dir,
            **gpu,
        }

    def get_storage_info(self):
        """Tamaño de subcarpetas de .minecraft para el panel de Almacenamiento."""
        try:
            import minecraft_launcher_lib
            mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            keys = ["versions", "assets", "mods", "resourcepacks",
                    "logs", "crash-reports", "screenshots"]
            result = {"mc_dir": mc_dir}
            total = 0
            for key in keys:
                p = os.path.join(mc_dir, key)
                size = _tamano_carpeta(p) if os.path.exists(p) else 0
                result[key] = size
                total += size
            result["total"] = total
            return {"ok": True, "storage": result}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def abrir_minecraft_folder(self):
        try:
            import minecraft_launcher_lib
            mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            subprocess.Popen(["explorer", mc_dir])
            return {"ok": True, "path": mc_dir}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def limpiar_logs_minecraft(self):
        try:
            import minecraft_launcher_lib
            mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            logs_dir = os.path.join(mc_dir, "logs")
            count, freed = 0, 0
            if os.path.exists(logs_dir):
                for fname in os.listdir(logs_dir):
                    fp = os.path.join(logs_dir, fname)
                    try:
                        if os.path.isfile(fp):
                            freed += os.path.getsize(fp)
                            os.remove(fp)
                            count += 1
                    except Exception:
                        pass
            return {"ok": True, "count": count, "freed_mb": round(freed / (1024 * 1024), 2)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def limpiar_crash_reports(self):
        try:
            import minecraft_launcher_lib
            mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            crash_dir = os.path.join(mc_dir, "crash-reports")
            count, freed = 0, 0
            if os.path.exists(crash_dir):
                for fname in os.listdir(crash_dir):
                    fp = os.path.join(crash_dir, fname)
                    try:
                        if os.path.isfile(fp):
                            freed += os.path.getsize(fp)
                            os.remove(fp)
                            count += 1
                    except Exception:
                        pass
            return {"ok": True, "count": count, "freed_mb": round(freed / (1024 * 1024), 2)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_loaders_for_version(self, version_id):
        """Add-ons permitidos según la versión concreta (parche)."""
        loaders = ["Vanilla"]
        v = (version_id or "").strip()
        if not v:
            return loaders

        if v.startswith("26"):
            loaders.extend(["OptiFine", "Forge", "NeoForge", "Fabric", "Fabric + Iris"])
            return loaders

        parts = v.split(".")
        try:
            major = int(parts[0])
            minor = int(parts[1]) if len(parts) > 1 and parts[1].isdigit() else 0
            patch = int(parts[2]) if len(parts) > 2 and parts[2].isdigit() else 0
        except (ValueError, IndexError):
            return loaders

        if major != 1:
            return loaders

        if minor >= 7:
            loaders.append("OptiFine")
            loaders.append("Forge")
        if minor > 20 or (minor == 20 and patch >= 2):
            loaders.append("NeoForge")
        if minor >= 14:
            loaders.append("Fabric")
        if minor >= 16:
            loaders.append("Fabric + Iris")

        # orden estable: Vanilla, OptiFine, Forge, Fabric, Fabric + Iris
        orden = ["Vanilla", "OptiFine", "Forge", "NeoForge", "Fabric", "Fabric + Iris"]
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

    def instalar_mod_archivo(self, ruta, tipo='mods'):
        """Copia un JAR/ZIP suelto al directorio de la instancia activa."""
        import shutil
        try:
            if not ruta or not os.path.isfile(ruta):
                return {'ok': False, 'error': 'Archivo no encontrado: ' + str(ruta)}
            v = self.config_actual.get('ultima_version', '')
            m = self.config_actual.get('ultimo_motor', '')
            if not v or not m:
                return {'ok': False, 'error': 'No hay versi\u00f3n activa seleccionada'}
            sub_map = {'mods': 'mods', 'resourcepacks': 'resourcepacks', 'shaders': 'shaderpacks', 'datapacks': 'datapacks'}
            carpeta = os.path.join(self._dir_instancia(v, m), sub_map.get(tipo, 'mods'))
            os.makedirs(carpeta, exist_ok=True)
            dest = os.path.join(carpeta, os.path.basename(ruta))
            shutil.copy2(ruta, dest)
            return {'ok': True, 'nombre': os.path.basename(ruta)}
        except Exception as e:
            return {'ok': False, 'error': str(e)}

    def get_instance_content(self, version, motor, categoria):
        try:
            from src.modelo import GestorContenidoInstancia, GestorLocalMods

            base = self._dir_instancia(version, motor)
            cat = (categoria or "mods").lower()
            if cat == "mods":
                mods_dir = os.path.join(base, "mods")
                lista = GestorLocalMods.obtener_lista_mods(mods_dir)
                return [
                    {
                        "archivo": m["archivo"],
                        "nombre_visible": m.get("nombre_visible") or m["archivo"],
                        "estado": m["estado"],
                        "mundo": "",
                        "version_mod": m.get("version_mod", ""),
                        "autor": m.get("autor", ""),
                        "icono_b64": m.get("icono_b64", ""),
                    }
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

            if "26.1" not in grupos:
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

    def get_versiones_instaladas(self):
        """Escanea .minecraft/versions/ y detecta versiones instaladas con su loader."""
        try:
            import minecraft_launcher_lib, re as _re
            mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            versions_dir = os.path.join(mc_dir, "versions")
            if not os.path.isdir(versions_dir):
                return {"ok": True, "instaladas": []}
            result = []
            extras = []
            seen = set()
            for v in sorted(os.listdir(versions_dir)):
                if not os.path.exists(os.path.join(versions_dir, v, v + ".json")):
                    continue
                version_base = None
                motor = None
                if v.startswith("fabric-loader-"):
                    parts = v.split("-")
                    version_base = parts[-1]
                    motor = "Fabric"
                elif "OptiFine" in v:
                    if "_OptiFine" in v:
                        version_base = v.split("_OptiFine")[0]
                    else:
                        version_base = v.split("-OptiFine")[0]
                    motor = "OptiFine"
                elif _re.search(r"-forge", v, _re.IGNORECASE):
                    m = _re.match(r"^(\d+\.\d+(?:\.\d+)?)-forge", v, _re.IGNORECASE)
                    if m:
                        version_base = m.group(1)
                        motor = "Forge"
                if version_base and motor:
                    parts = version_base.split(".")
                    group_key = f"{parts[0]}.{parts[1]}" if len(parts) >= 2 else ""
                    key = (version_base, motor)
                    if key in seen:
                        continue
                    seen.add(key)
                    if group_key in FEATURED_VERSION_KEYS:
                        extras.append({"version": version_base, "motor": motor})
                    else:
                        result.append({"version": version_base, "motor": motor})
            return {"ok": True, "instaladas": result, "extras": extras}
        except Exception as e:
            return {"ok": False, "error": str(e), "instaladas": []}

    def _hilo_ninja_renombrar(self, version_jugada):
        if sys.platform != "win32":
            return
        try:
            import win32gui, win32process
        except ImportError:
            return
        nuevo_titulo = f"Paraguacraft {version_jugada}"
        while self.hilo_juego_activo:
            time.sleep(0.3)

            def callback(hwnd, _):
                if not win32gui.IsWindowVisible(hwnd):
                    return
                titulo = win32gui.GetWindowText(hwnd)
                if "Minecraft" in titulo and "Paraguacraft" not in titulo and titulo != nuevo_titulo:
                    try:
                        _, pid = win32process.GetWindowThreadProcessId(hwnd)
                        pname = psutil.Process(pid).name().lower()
                        if pname in ('javaw.exe', 'java.exe'):
                            win32gui.SetWindowText(hwnd, nuevo_titulo)
                    except Exception:
                        pass

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
            session_start = int(time.time())
            en_mundo = False

            def _rpc_update():
                try:
                    self.rpc.update(
                        state=estado_actual,
                        details=f"👤 {usuario} · {version_jugada} · {motor_elegido}",
                        large_image="logo",
                        large_text=f"Paraguacraft {version_jugada}",
                        small_image="logo" if en_mundo else None,
                        small_text=motor_elegido if en_mundo else None,
                        start=session_start,
                    )
                except Exception:
                    pass

            _rpc_update()

            while not os.path.exists(log_path) and self.hilo_juego_activo:
                time.sleep(1)

            if not os.path.exists(log_path):
                return

            with open(log_path, "r", encoding="utf-8", errors="ignore") as f:
                while self.hilo_juego_activo:
                    linea = f.readline()
                    if not linea:
                        time.sleep(0.45)
                        continue
                    changed = True
                    if "Local game hosted on" in linea:
                        m = re.search(r"\[(.*?)\]", linea)
                        if m:
                            estado_actual = f"🏠 LAN: {m.group(1)}"
                        en_mundo = True
                        session_start = int(time.time())
                    elif "Connecting to" in linea:
                        ip = linea.split("Connecting to ")[1].split(",")[0].split(":")[0].strip()
                        estado_actual = f"🌐 Multijugador: {ip}"
                        en_mundo = True
                        session_start = int(time.time())
                    elif "Starting integrated minecraft server" in linea:
                        if "LAN" not in estado_actual:
                            estado_actual = "🌍 Mundo local"
                        en_mundo = True
                        session_start = int(time.time())
                    elif "Disconnecting from" in linea or "Stopping server" in linea:
                        estado_actual = "En el menú principal"
                        en_mundo = False
                        session_start = int(time.time())
                    else:
                        changed = False
                    if changed:
                        try:
                            self.rpc.update(
                                state=estado_actual,
                                details=f"👤 {usuario} · {version_jugada} · {motor_elegido}",
                                large_image="logo",
                                large_text=f"Paraguacraft {version_jugada}",
                                small_image="logo" if en_mundo else None,
                                small_text=motor_elegido if en_mundo else None,
                                start=session_start,
                            )
                        except Exception:
                            break
        except Exception as e:
            print("Error RPC dinámico:", e)

    def lanzar_juego(self, version, motor, server_ip=""):
        if self.hilo_juego_activo:
            print(f"[GUARD] Ya hay un juego activo, ignorando lanzamiento de {version}")
            return "ya_activo"
        print(f"Lanzando {version} con {motor}. AutoJoin: {server_ip}")
        self.hilo_juego_activo = True
        self.config_actual["ultima_version"] = version
        self.config_actual["ultimo_motor"] = motor
        threading.Thread(target=self._guardar, daemon=True).start()

        username_limpio = self.config_actual.get("usuario", "Invitado").replace(" [PREMIUM]", "")

        threading.Thread(target=self._hilo_ninja_renombrar, args=(version,), daemon=True).start()
        if self.config_actual.get('discord_rpc', True):
            threading.Thread(
                target=self._hilo_discord_rpc_dinamico,
                args=(version, motor, username_limpio),
                daemon=True,
            ).start()

        self._game_status = {"running": True, "status": "Iniciando..."}
        _last_ui_update = [0.0]

        def _set_status(msg):
            import time as _time
            self._game_status["status"] = str(msg)
            now = _time.monotonic()
            if now - _last_ui_update[0] >= 0.4:
                _last_ui_update[0] = now
                _js = f"actualizarEstadoPanel({json.dumps(str(msg))})"
                threading.Thread(
                    target=lambda: webview.windows[0].evaluate_js(_js),
                    daemon=True
                ).start()

        def _log_launch(msg):
            for _ld in [
                os.path.join(os.environ.get("APPDATA", ""), "ParaguacraftLauncher"),
                os.path.join(os.path.expanduser("~"), "AppData", "Roaming", "ParaguacraftLauncher"),
            ]:
                try:
                    os.makedirs(_ld, exist_ok=True)
                    with open(os.path.join(_ld, "launch_debug.log"), "a") as _f:
                        _f.write("[" + str(datetime.datetime.now()) + "] " + str(msg)[:2000] + "\n")
                    break
                except Exception:
                    pass

        def hilo_lanzar():
            try:
                from core import lanzar_minecraft
                _log_launch(f"START version={version} motor={motor}")

                _set_status("Iniciando motor...")

                threading.Thread(target=self._refresh_ms_token, daemon=True).start()
                if self.config_actual.get('discord_rpc', True):
                    threading.Thread(target=self._rpc_jugando, args=(version, motor), daemon=True).start()

                uuid_real = self.ms_data["id"] if self.ms_data else None
                token_real = self.ms_data["access_token"] if self.ms_data else None

                fl_raw = ""
                if "fabric" in motor.lower():
                    try:
                        from core import carpeta_instancia_paraguacraft as _cia_fl
                        import minecraft_launcher_lib as _mcl_fl
                        _icp_fl = os.path.join(_mcl_fl.utils.get_minecraft_directory(), "instancias",
                                               _cia_fl(version, motor), "_paragua_instance.json")
                        if os.path.exists(_icp_fl):
                            with open(_icp_fl) as _ffl:
                                fl_raw = json.load(_ffl).get("fabric_loader_version", "") or ""
                    except Exception:
                        pass
                    if not fl_raw:
                        fl_raw = (self.config_actual.get("fabric_loader_version") or "").strip()
                fabric_override = fl_raw.strip() or None

                _t0 = time.time()
                try:
                    from core import carpeta_instancia_paraguacraft as _cia
                    import minecraft_launcher_lib as _mcl2
                    _icp = os.path.join(_mcl2.utils.get_minecraft_directory(), "instancias", _cia(version, motor), "_paragua_instance.json")
                    with open(_icp) as _f2:
                        _ram_gb = json.load(_f2).get("ram_gb") or self.config_actual.get("ram_asignada", 4)
                except Exception:
                    _ram_gb = self.config_actual.get("ram_asignada", 4)
                # Auto-backup de mundos si corresponde
                try:
                    import datetime as _dt
                    _bk_h = self.config_actual.get("backup_auto_horas", 0)
                    if _bk_h and _bk_h > 0:
                        _ult = self.config_actual.get("ultimo_backup_auto", "")
                        _now = _dt.datetime.now()
                        _do_bk = True
                        if _ult:
                            _do_bk = (_now - _dt.datetime.fromisoformat(_ult)).total_seconds() / 3600 >= _bk_h
                        if _do_bk:
                            _set_status("Realizando backup autom\u00e1tico...")
                            from core import carpeta_instancia_paraguacraft as _cia_bk
                            import minecraft_launcher_lib as _mcl_bk
                            _bk_dir = os.path.join(_mcl_bk.utils.get_minecraft_directory(), "instancias", _cia_bk(version, motor))
                            threading.Thread(target=self.backup_mundos, args=(_bk_dir,), daemon=True).start()
                            self.config_actual["ultimo_backup_auto"] = _now.isoformat()
                            self._guardar()
                except Exception as _bk_e:
                    print(f"[BackupAuto] {_bk_e}")
                # Dynamic RAM: +1 GB si >20 mods, +2 GB si >50 mods
                try:
                    import minecraft_launcher_lib as _mcl3
                    _mdir = os.path.join(_mcl3.utils.get_minecraft_directory(), "instancias", _cia(version, motor), "mods")
                    _mc = len([x for x in os.listdir(_mdir) if x.endswith('.jar')]) if os.path.isdir(_mdir) else 0
                    if _mc > 50: _ram_gb = min(int(_ram_gb) + 2, 16)
                    elif _mc > 20: _ram_gb = min(int(_ram_gb) + 1, 16)
                except Exception: pass
                # limpieza profunda antes de lanzar
                if self.config_actual.get('limpiador_deep_var', False):
                    try:
                        self.limpiar_logs_minecraft()
                        self.limpiar_crash_reports()
                    except Exception:
                        pass
                # comportamiento del launcher al lanzar
                _lb = self.config_actual.get('launch_behavior', 'minimize')
                try:
                    if _lb == 'minimize':
                        webview.windows[0].minimize()
                    elif _lb == 'close':
                        webview.windows[0].hide()
                except Exception:
                    pass
                # notificación: juego iniciando
                threading.Thread(
                    target=lambda: self._mostrar_notificacion(
                        "Paraguacraft", f"Iniciando Minecraft {version} · {motor}"
                    ), daemon=True
                ).start()
                lanzar_minecraft(
                    version=version,
                    username=username_limpio,
                    max_ram=f"{_ram_gb}G",
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
                    progress_callback=_set_status,
                    server_ip=server_ip or "",
                    java_path=self.config_actual.get("java_custom_path") or None,
                )
                # restaurar ventana si se minimizó o ocultó
                try:
                    if _lb == 'minimize':
                        webview.windows[0].restore()
                    elif _lb == 'close':
                        webview.windows[0].show()
                except Exception:
                    pass
                # análisis de crash + notificación de cierre
                _crash = self._analizar_crash_report(version, motor, since_ts=_t0)
                if _crash:
                    threading.Thread(
                        target=lambda c=_crash: self._mostrar_notificacion(
                            "💥 Minecraft Crasheó", c.get("causa", "Error desconocido")
                        ), daemon=True
                    ).start()
                    try:
                        webview.windows[0].evaluate_js(f"mostrarCrashAnalisis({json.dumps(_crash)})")
                    except Exception:
                        pass
                else:
                    threading.Thread(
                        target=lambda: self._mostrar_notificacion("Paraguacraft", "Minecraft cerrado."),
                        daemon=True
                    ).start()
                _seg = int(time.time() - _t0)
                if _seg > 10:
                    try:
                        from core import carpeta_instancia_paraguacraft as _cia2
                        _inst_key = _cia2(version, motor)
                    except Exception:
                        _inst_key = f"{version}_{motor}"
                    threading.Thread(target=lambda s=_seg, k=_inst_key: self.guardar_sesion_estadistica(version, motor, s, instancia=k), daemon=True).start()

                _log_launch("DONE - juego cerrado")
                self.hilo_juego_activo = False
                self._game_status = {"running": False, "status": "Juego cerrado."}
                try:
                    webview.windows[0].evaluate_js("resetLaunchButton()")
                except Exception:
                    pass
                self._rpc_menu()
            except Exception as e:
                tb = traceback.format_exc()
                err = f"Error: {str(e)[:120]}"
                _log_launch(f"ERROR: {tb}")
                print(f"Error critico lanzar: {tb}")
                self.hilo_juego_activo = False
                self._game_status = {"running": False, "status": err}
                _crash_err = self._analizar_crash_report(version, motor, since_ts=time.time() - 300)
                threading.Thread(
                    target=lambda c=_crash_err, e2=str(e): self._mostrar_notificacion(
                        "💥 Error en Minecraft",
                        (c.get("causa") if c else None) or e2[:80]
                    ), daemon=True
                ).start()
                if _crash_err:
                    try:
                        webview.windows[0].evaluate_js(f"mostrarCrashAnalisis({json.dumps(_crash_err)})")
                    except Exception:
                        pass
                try:
                    webview.windows[0].evaluate_js(f"actualizarEstadoPanel({json.dumps(err)})")
                    webview.windows[0].evaluate_js("resetLaunchButton()")
                except Exception:
                    pass
                self._rpc_menu()
            finally:
                self.hilo_juego_activo = False
                self._game_status["running"] = False

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

    # ── Server properties editor ──────────────────────────────────────────────
    def leer_server_properties(self, carpeta):
        try:
            path = os.path.join(carpeta, "server.properties")
            if not os.path.exists(path):
                return {"ok": False, "error": "server.properties no existe. Iniciá el servidor al menos una vez."}
            props = {}
            with open(path, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith("#") and "=" in line:
                        k, _, v = line.partition("=")
                        props[k.strip()] = v.strip()
            return {"ok": True, "props": props}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def guardar_server_properties(self, carpeta, props):
        try:
            path = os.path.join(carpeta, "server.properties")
            if not os.path.exists(path):
                return {"ok": False, "error": "server.properties no existe. Iniciá el servidor al menos una vez."}
            with open(path, "r", encoding="utf-8") as f:
                lines = f.readlines()
            updated = set()
            new_lines = []
            for line in lines:
                stripped = line.strip()
                if stripped.startswith("#") or "=" not in stripped:
                    new_lines.append(line)
                    continue
                k = stripped.split("=", 1)[0].strip()
                if k in props:
                    new_lines.append(f"{k}={props[k]}\n")
                    updated.add(k)
                else:
                    new_lines.append(line)
            for k, v in props.items():
                if k not in updated:
                    new_lines.append(f"{k}={v}\n")
            with open(path, "w", encoding="utf-8") as f:
                f.writelines(new_lines)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── RAM per instance ──────────────────────────────────────────────────────
    def get_instancia_config(self, folder):
        try:
            import minecraft_launcher_lib as _mcl
            mine_dir = _mcl.utils.get_minecraft_directory()
            cfg_path = os.path.join(mine_dir, "instancias", folder, "_paragua_instance.json")
            if os.path.exists(cfg_path):
                with open(cfg_path, "r") as f:
                    return {"ok": True, "config": json.load(f)}
            return {"ok": True, "config": {}}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def set_instancia_config(self, folder, config):
        try:
            import minecraft_launcher_lib as _mcl
            mine_dir = _mcl.utils.get_minecraft_directory()
            inst_dir = os.path.join(mine_dir, "instancias", folder)
            os.makedirs(inst_dir, exist_ok=True)
            cfg_path = os.path.join(inst_dir, "_paragua_instance.json")
            existing = {}
            if os.path.exists(cfg_path):
                with open(cfg_path, "r") as f:
                    existing = json.load(f)
            existing.update(config)
            with open(cfg_path, "w") as f:
                json.dump(existing, f)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── Modpack importer (Modrinth) ───────────────────────────────────────────
    def importar_modpack_modrinth(self, slug_o_url, mc_version_filter=""):
        import re, zipfile, shutil, tempfile
        try:
            slug = slug_o_url.strip()
            m = re.search(r'modrinth\.com/modpack/([^/?#]+)', slug)
            if m:
                slug = m.group(1)
            proj_r = requests.get(f"https://api.modrinth.com/v2/project/{slug}", timeout=10)
            if proj_r.status_code != 200:
                return {"ok": False, "error": f"Modpack '{slug}' no encontrado en Modrinth"}
            proj = proj_r.json()
            vers_r = requests.get(f"https://api.modrinth.com/v2/project/{slug}/version", timeout=10)
            vers_r.raise_for_status()
            versions = vers_r.json()
            if not versions:
                return {"ok": False, "error": "No hay versiones disponibles"}
            ver = next((v for v in versions if mc_version_filter in v.get("game_versions", [])), versions[0]) if mc_version_filter else versions[0]
            mrpack = next((f for f in ver.get("files", []) if f["filename"].endswith(".mrpack")), None)
            if not mrpack:
                return {"ok": False, "error": "No se encontró archivo .mrpack"}
            with tempfile.NamedTemporaryFile(suffix=".mrpack", delete=False) as tmp:
                tmp_path = tmp.name
            r = requests.get(mrpack["url"], stream=True, timeout=(15, 300))
            r.raise_for_status()
            with open(tmp_path, "wb") as f:
                shutil.copyfileobj(r.raw, f, length=65536)
            import minecraft_launcher_lib as _mcl
            mine_dir = _mcl.utils.get_minecraft_directory()
            with zipfile.ZipFile(tmp_path, "r") as z:
                with z.open("modrinth.index.json") as f:
                    index = json.load(f)
            deps = index.get("dependencies", {})
            mc_ver = deps.get("minecraft", ver.get("game_versions", ["1.20.1"])[0])
            loader = "fabric" if "fabric-loader" in deps else "forge" if "forge" in deps else "vanilla"
            inst_name = re.sub(r'[^\w\-]', '_', proj.get("title", slug))[:28]
            inst_dir = os.path.join(mine_dir, "instancias", inst_name)
            os.makedirs(inst_dir, exist_ok=True)
            files = index.get("files", [])
            for mod_file in files:
                dest = os.path.join(inst_dir, mod_file["path"].lstrip("/").replace("/", os.sep))
                os.makedirs(os.path.dirname(dest), exist_ok=True)
                if os.path.exists(dest):
                    continue
                for dl_url in mod_file.get("downloads", []):
                    try:
                        r2 = requests.get(dl_url, stream=True, timeout=(10, 120))
                        if r2.status_code == 200:
                            with open(dest, "wb") as f:
                                shutil.copyfileobj(r2.raw, f, length=65536)
                            break
                    except Exception:
                        continue
            with zipfile.ZipFile(tmp_path, "r") as z:
                for zname in z.namelist():
                    for prefix in ("overrides/", "client-overrides/"):
                        if zname.startswith(prefix) and not zname.endswith("/"):
                            rel = zname[len(prefix):]
                            dest = os.path.join(inst_dir, rel.replace("/", os.sep))
                            os.makedirs(os.path.dirname(dest), exist_ok=True)
                            with z.open(zname) as src, open(dest, "wb") as dst:
                                shutil.copyfileobj(src, dst)
            try:
                os.remove(tmp_path)
            except Exception:
                pass
            with open(os.path.join(inst_dir, "_paragua_instance.json"), "w") as f:
                json.dump({"mc_version": mc_ver, "loader": loader, "name": inst_name, "modpack_slug": slug}, f)
            return {"ok": True, "nombre": inst_name, "version": mc_ver, "loader": loader, "mods": len(files)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── Multi-cuenta ──────────────────────────────────────────────────────────
    _SESIONES_PATH = "paraguacraft_sessions.json"

    def get_cuentas(self):
        try:
            cuentas = []
            # Premium accounts
            if os.path.exists(self._SESIONES_PATH):
                with open(self._SESIONES_PATH, "r") as f:
                    sessions = json.load(f)
                for i, s in enumerate(sessions):
                    cuentas.append({"nombre": s.get("name", "?"), "tipo": "premium", "idx": i})
            elif os.path.exists(self.ruta_sesion):
                with open(self.ruta_sesion, "r") as f:
                    s = json.load(f)
                cuentas.append({"nombre": s.get("name", "?"), "tipo": "premium", "idx": 0})
            # Guest accounts
            for i, g in enumerate(self.config_actual.get("cuentas_invitado", [])):
                cuentas.append({"nombre": g.get("nombre", "?"), "tipo": "invitado", "idx": i})
            activo = self.config_actual.get("usuario", "Invitado")
            es_premium = self.config_actual.get("is_premium", False)
            return {"ok": True, "cuentas": cuentas, "activo": activo, "es_premium": es_premium}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def cambiar_cuenta(self, tipo, idx):
        try:
            if tipo == "premium":
                with open(self._SESIONES_PATH, "r") as f:
                    sessions = json.load(f)
                if not (0 <= idx < len(sessions)):
                    return {"ok": False, "error": "Índice inválido"}
                self.ms_data = sessions[idx]
                nombre = f"{self.ms_data['name']} [PREMIUM]"
                self.config_actual["usuario"] = nombre
                self.config_actual["is_premium"] = True
                with open(self.ruta_sesion, "w") as f:
                    json.dump(self.ms_data, f)
                self._guardar()
                return {"ok": True, "usuario": nombre, "es_premium": True}
            elif tipo == "invitado":
                invitados = self.config_actual.get("cuentas_invitado", [])
                if not (0 <= idx < len(invitados)):
                    return {"ok": False, "error": "Índice inválido"}
                nombre = invitados[idx]["nombre"]
                self.ms_data = None
                self.config_actual["usuario"] = nombre
                self.config_actual["is_premium"] = False
                self._guardar()
                return {"ok": True, "usuario": nombre, "es_premium": False}
            return {"ok": False, "error": "Tipo desconocido"}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def agregar_cuenta_invitado(self, nombre):
        nombre = (nombre or "").strip()
        if not nombre:
            return {"ok": False, "error": "Nombre vacío"}
        invitados = self.config_actual.get("cuentas_invitado", [])
        if any(g["nombre"] == nombre for g in invitados):
            return {"ok": False, "error": "Ya existe una cuenta con ese nombre"}
        invitados.append({"nombre": nombre})
        self.config_actual["cuentas_invitado"] = invitados
        self._guardar()
        return {"ok": True}

    def eliminar_cuenta(self, tipo, idx):
        try:
            if tipo == "premium":
                if not os.path.exists(self._SESIONES_PATH):
                    return {"ok": False, "error": "Sin sesiones guardadas"}
                with open(self._SESIONES_PATH, "r") as f:
                    sessions = json.load(f)
                if not (0 <= idx < len(sessions)):
                    return {"ok": False, "error": "Índice inválido"}
                sessions.pop(idx)
                with open(self._SESIONES_PATH, "w") as f:
                    json.dump(sessions, f)
                # If active account was this one, reset to first available or guest
                activo = self.config_actual.get("usuario", "")
                if activo.endswith("[PREMIUM]"):
                    if sessions:
                        self.ms_data = sessions[0]
                        self.config_actual["usuario"] = f"{sessions[0]['name']} [PREMIUM]"
                        with open(self.ruta_sesion, "w") as f:
                            json.dump(self.ms_data, f)
                    else:
                        self.ms_data = None
                        self.config_actual["usuario"] = "Invitado"
                        self.config_actual["is_premium"] = False
                self._guardar()
                return {"ok": True}
            elif tipo == "invitado":
                invitados = self.config_actual.get("cuentas_invitado", [])
                if not (0 <= idx < len(invitados)):
                    return {"ok": False, "error": "Índice inválido"}
                nombre_eliminado = invitados[idx]["nombre"]
                invitados.pop(idx)
                self.config_actual["cuentas_invitado"] = invitados
                if self.config_actual.get("usuario") == nombre_eliminado:
                    self.config_actual["usuario"] = "Invitado"
                self._guardar()
                return {"ok": True}
            return {"ok": False, "error": "Tipo desconocido"}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def _hilo_bedrock_renombrar(self):
        if sys.platform != "win32":
            return
        try:
            import win32gui, win32process
        except ImportError:
            return

        nuevo_titulo = "Paraguacraft Bedrock"
        PROC_NAMES = {"minecraft.windows.exe", "minecraftuwp.exe", "minecraftpe.exe"}
        usuario = self.config_actual.get("usuario", "Jugador").replace(" [PREMIUM]", "")

        def _js(code):
            try:
                webview.windows[0].evaluate_js(code)
            except Exception:
                pass

        def _set_estado(msg):
            _js(f"actualizarEstadoBedrock({json.dumps(msg)})")

        def _renombrar(hwnd, _):
            if not win32gui.IsWindowVisible(hwnd):
                return
            titulo = win32gui.GetWindowText(hwnd)
            if "Minecraft" in titulo and "Paraguacraft" not in titulo and titulo != nuevo_titulo:
                try:
                    _, pid = win32process.GetWindowThreadProcessId(hwnd)
                    pname = psutil.Process(pid).name().lower()
                    if pname in PROC_NAMES:
                        win32gui.SetWindowText(hwnd, nuevo_titulo)
                except Exception:
                    pass

        # ── Fase 1: esperar a que arranque el proceso (hasta 2 min) ──────────
        _set_estado("Detectando Bedrock...")
        MAX_WAIT = 120
        elapsed = 0
        found = False
        while elapsed < MAX_WAIT:
            time.sleep(0.5)
            elapsed += 0.5
            try:
                procs = [p.info["name"].lower() for p in psutil.process_iter(["name"])]
                if any(n in PROC_NAMES for n in procs):
                    found = True
                    break
            except Exception:
                pass

        if not found:
            _set_estado("")
            return

        # ── Fase 2: Bedrock corriendo — esperar ventana visible antes de minimizar ──
        _set_estado("🎮 Bedrock activo")
        try:
            if self.rpc and self.config_actual.get("discord_rpc", True):
                show_time = self.config_actual.get("discord_rpc_time", True)
                self.rpc.update(
                    state="Paraguacraft Bedrock",
                    details=f"Jugando como {usuario}",
                    large_image="logo",
                    large_text="Paraguacraft Bedrock",
                    start=int(time.time()) if show_time else None,
                )
        except Exception:
            pass

        # Minimizar el launcher ANTES de que Bedrock renderice su ventana.
        # Si minimizamos después, Bedrock calcula su fullscreen con el launcher
        # visible y deja un borde negro al quedarse solo.
        _lb = self.config_actual.get("launch_behavior", "minimize")
        try:
            if _lb == "minimize":
                webview.windows[0].minimize()
            elif _lb == "close":
                webview.windows[0].hide()
        except Exception:
            pass

        MAX_RUN = 600
        ran = 0
        while ran < MAX_RUN:
            time.sleep(0.15)
            ran += 0.15
            try:
                win32gui.EnumWindows(_renombrar, None)
            except Exception:
                pass
            try:
                procs = [p.info["name"].lower() for p in psutil.process_iter(["name"])]
                if not any(n in PROC_NAMES for n in procs):
                    break
            except Exception:
                pass

        # ── Fase 3: Bedrock cerrado — restaurar launcher + RPC ───────────────
        try:
            if _lb == "minimize":
                webview.windows[0].restore()
            elif _lb == "close":
                webview.windows[0].show()
        except Exception:
            pass
        self._rpc_menu()
        _set_estado("Bedrock cerrado")

    def _bedrock_exe_paths(self):
        paths = [
            r"C:\XboxGames\Minecraft for Windows\Content\Minecraft.Windows.exe",
            r"C:\XboxGames\Minecraft for Windows\Content\gamelaunchhelper.exe",
        ]
        try:
            import winreg
            with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE,
                                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall") as k:
                pass
        except Exception:
            pass
        try:
            prog = os.environ.get("PROGRAMFILES", r"C:\Program Files")
            base = os.path.join(prog, "WindowsApps")
            if os.path.isdir(base):
                for entry in os.listdir(base):
                    if "Minecraft" in entry and "Windows" in entry:
                        candidate = os.path.join(base, entry, "Minecraft.Windows.exe")
                        if os.path.exists(candidate):
                            paths.insert(0, candidate)
        except Exception:
            pass
        return paths

    def _bedrock_mojang_dir(self):
        import glob
        localappdata = os.environ.get("LOCALAPPDATA", "")
        if not localappdata:
            return None
        pattern = os.path.join(
            localappdata, "Packages", "Microsoft.MinecraftUWP_*",
            "LocalState", "games", "com.mojang"
        )
        matches = glob.glob(pattern)
        if not matches:
            pattern2 = os.path.join(
                localappdata, "Packages", "Microsoft.MinecraftWindowsBeta_*",
                "LocalState", "games", "com.mojang"
            )
            matches = glob.glob(pattern2)
        return matches[0] if matches else None

    def instalar_pack_paraguacraft(self):
        PACK_UUID   = "a1b2c3d4-e5f6-7890-abcd-ef1234567891"
        MODULE_UUID = "b2c3d4e5-f6a7-8901-bcde-f12345678902"
        PACK_VER    = [1, 0, 0]
        try:
            mojang_dir = self._bedrock_mojang_dir()
            if not mojang_dir:
                return {"ok": False, "error": "Bedrock no encontrado"}

            pack_dir = os.path.join(mojang_dir, "resource_packs", "ParaguacraftBranding")
            tex_dir  = os.path.join(pack_dir, "textures", "ui")
            os.makedirs(tex_dir, exist_ok=True)

            logo_src = os.path.join(os.path.dirname(os.path.abspath(__file__)), "web", "assets", "paraguacraft_logo.png")

            # Resize logo → 512×128 with transparent padding
            try:
                from PIL import Image
                img = Image.open(logo_src).convert("RGBA")
                tw, th = 512, 128
                ir = img.width / img.height
                tr = tw / th
                if ir > tr:
                    nw, nh = tw, int(tw / ir)
                else:
                    nw, nh = int(th * ir), th
                img = img.resize((nw, nh), Image.LANCZOS)
                canvas = Image.new("RGBA", (tw, th), (0, 0, 0, 0))
                canvas.paste(img, ((tw - nw) // 2, (th - nh) // 2))
                canvas.save(os.path.join(tex_dir, "title.png"))
                # Algunas versiones de Bedrock usan rutas adicionales
                for _extra in ["title2.png", "mojanglogo.png"]:
                    try:
                        canvas.save(os.path.join(tex_dir, _extra))
                    except Exception:
                        pass
            except ImportError:
                import shutil
                for _fn in ["title.png", "title2.png", "mojanglogo.png"]:
                    shutil.copy2(logo_src, os.path.join(tex_dir, _fn))

            # Pack icon
            try:
                import shutil
                shutil.copy2(logo_src, os.path.join(pack_dir, "pack_icon.png"))
            except Exception:
                pass

            # manifest.json
            manifest = {
                "format_version": 2,
                "header": {
                    "name": "Paraguacraft Branding",
                    "description": "Logo personalizado — Paraguacraft Launcher",
                    "uuid": PACK_UUID,
                    "version": PACK_VER,
                    "min_engine_version": [1, 16, 0]
                },
                "modules": [{"type": "resources", "uuid": MODULE_UUID, "version": PACK_VER}]
            }
            with open(os.path.join(pack_dir, "manifest.json"), "w") as f:
                json.dump(manifest, f, indent=2)

            # global_resource_packs.json — activar globalmente
            minecraftpe_dir = os.path.join(mojang_dir, "minecraftpe")
            os.makedirs(minecraftpe_dir, exist_ok=True)
            grp_path = os.path.join(minecraftpe_dir, "global_resource_packs.json")
            existing = []
            if os.path.exists(grp_path):
                try:
                    with open(grp_path) as f:
                        existing = json.load(f)
                except Exception:
                    existing = []
            if not any(p.get("pack_id") == PACK_UUID for p in existing):
                existing.insert(0, {"pack_id": PACK_UUID, "version": PACK_VER})
                with open(grp_path, "w") as f:
                    json.dump(existing, f, indent=2)

            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def lanzar_bedrock(self):
        try:
            if not self.config_actual.get("is_premium", False):
                return {"ok": False, "error": "Se necesita cuenta Premium para jugar Minecraft: Bedrock Edition"}
            try:
                _pack_r = self.instalar_pack_paraguacraft()
                print("[PACK]", _pack_r)
                with open(os.path.join(os.path.expanduser("~"), "paraguacraft_pack_debug.txt"), "w") as _dbg:
                    _dbg.write(str(_pack_r))
            except Exception as _pe:
                print("[PACK] error:", _pe)
                try:
                    with open(os.path.join(os.path.expanduser("~"), "paraguacraft_pack_debug.txt"), "w") as _dbg:
                        _dbg.write("ERROR: " + str(_pe))
                except Exception:
                    pass
            import ctypes
            _NO_WIN = subprocess.CREATE_NO_WINDOW if sys.platform == "win32" else 0

            # --- 1. Descubrir AUMID dinámicamente via PowerShell ---
            aumids = [
                r"shell:AppsFolder\Microsoft.MinecraftUWP_8wekyb3d8bbwe!App",
                r"shell:AppsFolder\Microsoft.MinecraftWindowsBeta_8wekyb3d8bbwe!App",
            ]
            try:
                r = subprocess.run(
                    ["powershell", "-NoProfile", "-Command",
                     "Get-AppxPackage *Minecraft* | Select-Object -ExpandProperty PackageFamilyName"],
                    capture_output=True, text=True, timeout=8, creationflags=_NO_WIN
                )
                for line in r.stdout.splitlines():
                    pfn = line.strip()
                    if pfn:
                        dyn = fr"shell:AppsFolder\{pfn}!App"
                        if dyn not in aumids:
                            aumids.insert(0, dyn)
            except Exception:
                pass

            # --- 2. Intentar AUMIDs conocidos via ShellExecuteW ---
            for aumid in aumids:
                ret = ctypes.windll.shell32.ShellExecuteW(None, "open", aumid, None, None, 1)
                if ret > 32:
                    threading.Thread(target=self._hilo_bedrock_renombrar, daemon=True).start()
                    return {"ok": True}

            # --- 3. Fallback: lanzar el .exe directo (Xbox Game Pass path) ---
            for path in self._bedrock_exe_paths():
                if os.path.exists(path):
                    subprocess.Popen([path], cwd=os.path.dirname(path),
                                     creationflags=_NO_WIN)
                    threading.Thread(target=self._hilo_bedrock_renombrar, daemon=True).start()
                    return {"ok": True}

            return {"ok": False, "error": "No se encontró Minecraft: Bedrock Edition. Instalalo desde Xbox / Microsoft Store."}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── Optimizaciones (Sodium / Iris / OptiFine / Lithium) ───────────────────
    def instalar_optimizacion(self, tipo, carpeta):
        try:
            cfg_path = os.path.join(carpeta, "_paragua_instance.json")
            version_mc, loader = "1.20.1", "fabric"
            if os.path.exists(cfg_path):
                with open(cfg_path) as f:
                    cfg = json.load(f)
                version_mc = cfg.get("mc_version") or cfg.get("version", "1.20.1")
                loader = cfg.get("loader", "fabric")
            mods_dir = os.path.join(carpeta, "mods")
            os.makedirs(mods_dir, exist_ok=True)
            slug_map = {"sodium": "sodium", "iris": "iris", "lithium": "lithium"}
            if tipo in slug_map:
                slug = slug_map[tipo]
                params = f"?game_versions=%5B%22{version_mc}%22%5D&loaders=%5B%22{loader}%22%5D"
                r = requests.get(f"https://api.modrinth.com/v2/project/{slug}/version{params}", timeout=10)
                versions = r.json()
                if not versions:
                    return {"ok": False, "error": f"No hay versión de {tipo} para {version_mc} ({loader})"}
                file_info = versions[0]["files"][0]
                r2 = requests.get(file_info["url"], timeout=30)
                dest = os.path.join(mods_dir, file_info["filename"])
                data_bytes = r2.content
                sha1_exp = (file_info.get("hashes") or {}).get("sha1", "")
                if sha1_exp:
                    import hashlib as _hl
                    if _hl.sha1(data_bytes).hexdigest() != sha1_exp:
                        return {"ok": False, "error": f"{tipo}: checksum SHA1 inválido (archivo corrupto)"}
                with open(dest, "wb") as f:
                    f.write(data_bytes)
                return {"ok": True, "archivo": file_info["filename"]}
            elif tipo == "optifine":
                r = requests.get(f"https://bmclapi2.bangbang93.com/optifine/{version_mc}", timeout=10)
                builds = r.json()
                if not builds:
                    return {"ok": False, "error": f"No hay OptiFine disponible para {version_mc}"}
                latest = next((b for b in builds if "HD_U" in b.get("patch", "")), builds[0])
                dl_url = f"https://bmclapi2.bangbang93.com/optifine/{version_mc}/{latest['type']}/{latest['patch']}"
                fname = f"OptiFine_{version_mc}_{latest['type']}_{latest['patch']}.jar"
                r2 = requests.get(dl_url, timeout=60)
                with open(os.path.join(mods_dir, fname), "wb") as f:
                    f.write(r2.content)
                return {"ok": True, "archivo": fname}
            return {"ok": False, "error": f"Tipo desconocido: {tipo}"}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── Java auto-selector ────────────────────────────────────────────────────
    def detectar_java_para_version(self, version):
        try:
            import minecraft_launcher_lib as _mcl
            from core import _java_exe_para_version
            mc_dir = _mcl.utils.get_minecraft_directory()
            java_exe = _java_exe_para_version(mc_dir, version)
            if "jre-legacy" in java_exe:
                label = "Java 8 (bundled · jre-legacy)"
            elif "java-runtime-delta" in java_exe or "java-runtime-loom" in java_exe:
                label = "Java 21 (bundled)"
            elif "java-runtime-gamma" in java_exe or "java-runtime-beta" in java_exe:
                label = "Java 17 (bundled)"
            elif java_exe == "java":
                label = "Java (sistema)"
            else:
                label = "Java (bundled)"
            return {"ok": True, "java_exe": java_exe, "label": label, "bundled": java_exe != "java"}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── OptiFine automation + version preset completo ─────────────────────────
    def preparar_version_preset_completo(self, tipo):
        import minecraft_launcher_lib as _mcl
        from core import carpeta_instancia_paraguacraft as _cia, _asegurar_java_runtime, _java_exe_para_mc
        presets = {
            "pvp":        {"ver": "1.8.9",  "motor": "Forge",  "estilo": "pvp",      "optifine": True},
            "survival":   {"ver": "1.20.1", "motor": "Fabric", "estilo": "survival", "optifine": False},
            "servidores": {"ver": "1.16.5", "motor": "Fabric", "estilo": "pvp",      "optifine": False},
            "modded":     {"ver": "1.12.2", "motor": "Forge",  "estilo": "modded",   "optifine": False},
        }
        if tipo not in presets:
            return {"ok": False, "error": f"Preset desconocido: {tipo}"}
        p = presets[tipo]
        ver, motor, estilo = p["ver"], p["motor"], p["estilo"]
        mc_dir = _mcl.utils.get_minecraft_directory()
        resultados = []
        tier = "media"
        try:
            motor_lower = motor.lower()
            if "fabric" in motor_lower:
                loader_ver = _mcl.fabric.get_latest_loader_version()
                _mcl.fabric.install_fabric(ver, mc_dir, loader_ver, callback={"setStatus": lambda s: None})
                resultados.append(f"✅ Fabric {ver} instalado")
            elif "forge" in motor_lower:
                _mcl.install.install_minecraft_version(ver, mc_dir, callback={"setStatus": lambda s: None})
                _asegurar_java_runtime(ver, mc_dir, None)
                java_exe = _java_exe_para_mc(mc_dir)
                java_bin = os.path.dirname(java_exe) if java_exe != "java" else ""
                orig_path = os.environ.get("PATH", "")
                if java_bin and java_bin not in orig_path:
                    os.environ["PATH"] = java_bin + os.pathsep + orig_path
                try:
                    forge_ver = _mcl.forge.find_forge_version(ver)
                    if forge_ver:
                        try:
                            _mcl.forge.install_forge_version(forge_ver, mc_dir, java=java_exe, callback={"setStatus": lambda s: None})
                        except TypeError:
                            _mcl.forge.install_forge_version(forge_ver, mc_dir, callback={"setStatus": lambda s: None})
                        resultados.append(f"✅ Forge {ver} instalado")
                    else:
                        resultados.append(f"⚠️ Forge no disponible para {ver}")
                finally:
                    os.environ["PATH"] = orig_path
            hw = self.detectar_perfil_hardware_sugerido()
            tier = hw.get("perfil_id", "media")
            r_mods = self.aplicar_perfil_hardware(tier, ver, motor, estilo)
            if r_mods.get("ok"):
                resultados.append(f"✅ {len(r_mods.get('mods_instalados', []))} mods instalados (tier: {tier})")
            else:
                resultados.append(f"⚠️ Mods: {r_mods.get('error', 'sin mods')}")
            if p.get("optifine"):
                carpeta = os.path.join(mc_dir, "instancias", _cia(ver, motor))
                os.makedirs(os.path.join(carpeta, "mods"), exist_ok=True)
                cfg_path = os.path.join(carpeta, "_paragua_instance.json")
                if not os.path.exists(cfg_path):
                    with open(cfg_path, "w") as _f:
                        json.dump({"mc_version": ver, "loader": motor.lower()}, _f)
                r_of = self.instalar_optimizacion("optifine", carpeta)
                if r_of.get("ok"):
                    resultados.append(f"✅ OptiFine instalado ({r_of.get('archivo', '')})")
                else:
                    resultados.append(f"⚠️ OptiFine: {r_of.get('error', '')}")
            return {"ok": True, "version": ver, "motor": motor, "tier": tier, "resultados": resultados}
        except Exception as e:
            return {"ok": False, "error": str(e), "resultados": resultados}

    # ── Exportar instancia ────────────────────────────────────────────────────
    def exportar_instancia(self, carpeta):
        import zipfile, datetime
        try:
            if not os.path.exists(carpeta):
                return {"ok": False, "error": "Carpeta no encontrada"}
            nombre = os.path.basename(carpeta)
            ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
            out_dir = os.path.join(os.path.expanduser("~"), "Desktop")
            os.makedirs(out_dir, exist_ok=True)
            out_path = os.path.join(out_dir, f"{nombre}_{ts}.zip")
            skip = {"saves", "logs", "crash-reports", "crash_reports", ".git", "shaderpacks"}
            with zipfile.ZipFile(out_path, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as zf:
                for root, dirs, files in os.walk(carpeta):
                    dirs[:] = [d for d in dirs if d not in skip]
                    for file in files:
                        fp = os.path.join(root, file)
                        zf.write(fp, os.path.relpath(fp, carpeta))
            size_mb = round(os.path.getsize(out_path) / (1024 * 1024), 1)
            return {"ok": True, "ruta": out_path, "size_mb": size_mb}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── Importar instancia Prism/MultiMC ──────────────────────────────────────
    def importar_instancia_prism(self, ruta_zip):
        import zipfile
        try:
            import minecraft_launcher_lib as _mcl3
            inst_base = os.path.join(_mcl3.utils.get_minecraft_directory(), "instancias")
            os.makedirs(inst_base, exist_ok=True)
            with zipfile.ZipFile(ruta_zip, "r") as zf:
                names = zf.namelist()
                cfg_file = next((n for n in names if n.endswith("instance.cfg") or n.endswith("mmc-pack.json")), None)
                if not cfg_file:
                    return {"ok": False, "error": "No es una instancia válida de Prism/MultiMC"}
                base = cfg_file.split("/")[0]
                dest_dir = os.path.join(inst_base, base)
                for prefix in (f"{base}/.minecraft/", f"{base}/minecraft/"):
                    if any(n.startswith(prefix) for n in names):
                        for member in names:
                            if member.startswith(prefix) and not member.endswith("/"):
                                rel = member[len(prefix):]
                                target = os.path.join(dest_dir, rel.replace("/", os.sep))
                                os.makedirs(os.path.dirname(target), exist_ok=True)
                                with zf.open(member) as src, open(target, "wb") as dst:
                                    dst.write(src.read())
                        break
            return {"ok": True, "nombre": base, "carpeta": dest_dir}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── Backup de mundos ──────────────────────────────────────────────────────
    def backup_mundos(self, carpeta):
        import zipfile, datetime
        try:
            saves_dir = os.path.join(carpeta, "saves")
            if not os.path.exists(saves_dir) or not os.listdir(saves_dir):
                return {"ok": False, "error": "No hay mundos en esta instancia (saves vacía o inexistente)"}
            ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
            nombre = os.path.basename(carpeta)
            out_dir = os.path.join(os.path.expanduser("~"), "Desktop", "paraguacraft_backups")
            os.makedirs(out_dir, exist_ok=True)
            out = os.path.join(out_dir, f"{nombre}_backup_{ts}.zip")
            with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as zf:
                for root, dirs, files in os.walk(saves_dir):
                    for file in files:
                        fp = os.path.join(root, file)
                        zf.write(fp, os.path.relpath(fp, saves_dir))
            size_mb = round(os.path.getsize(out) / (1024 * 1024), 1)
            return {"ok": True, "ruta": out, "size_mb": size_mb}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def set_backup_auto(self, horas):
        try:
            self.config_actual["backup_auto_horas"] = int(horas)
            self._guardar()
            return {"ok": True, "horas": int(horas)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_backup_config(self):
        return {
            "ok": True,
            "horas": self.config_actual.get("backup_auto_horas", 0),
            "ultimo": self.config_actual.get("ultimo_backup_auto", ""),
        }

    # ── Screenshots viewer ────────────────────────────────────────────────────
    def get_screenshots_instancia(self, carpeta):
        try:
            ss_dir = os.path.join(carpeta, "screenshots")
            if not os.path.exists(ss_dir):
                return {"ok": True, "screenshots": []}
            files = []
            for fname in sorted(os.listdir(ss_dir), reverse=True):
                if fname.lower().endswith((".png", ".jpg", ".jpeg")):
                    fp = os.path.join(ss_dir, fname)
                    files.append({"nombre": fname, "ruta": fp, "size": os.path.getsize(fp)})
            return {"ok": True, "screenshots": files[:60]}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── Favoritos servidores ──────────────────────────────────────────────────
    def get_favoritos_srv(self):
        return {"ok": True, "favoritos": self.config_actual.get("srv_favoritos", [])}

    def toggle_favorito_srv(self, ip):
        favs = list(self.config_actual.get("srv_favoritos", []))
        if ip in favs:
            favs.remove(ip)
            added = False
        else:
            favs.append(ip)
            added = True
        self.config_actual["srv_favoritos"] = favs
        self._guardar()
        return {"ok": True, "favorito": added, "favoritos": favs}

    def crear_servidor(self, version, carpeta, tipo='paper'):
        try:
            from src.modelo import CreadorServidor
            os.makedirs(carpeta, exist_ok=True)
            self._servidor_carpeta = carpeta
            self._servidor_tipo = tipo or 'paper'
            self._servidor_creando = True
            tipo = tipo or 'paper'
            try:
                with open(os.path.join(carpeta, '_paragua_srv.json'), 'w') as _f:
                    json.dump({'tipo': self._servidor_tipo}, _f)
            except Exception:
                pass
            _nombre_tipo = {'paper': 'PaperMC', 'paper-geyser': 'PaperMC + Geyser', 'fabric': 'Fabric'}.get(tipo, tipo)
            with self._servidor_lock:
                self._servidor_log.clear()
                self._servidor_log.append(f"[SETUP] Creando servidor {_nombre_tipo} {version} en {carpeta}...")
                self._servidor_log.append(f"[SETUP] Servidor multi-versión: compatible con clientes 1.8 → última versión")
                self._servidor_log.append(f"[SETUP] Conectando a API...")

            def cb(msg):
                with self._servidor_lock:
                    self._servidor_log.append(f"[SETUP] {msg}")
                    if len(self._servidor_log) > 200:
                        self._servidor_log = self._servidor_log[-200:]

            try:
                exito, msg_result = CreadorServidor.descargar_y_preparar(carpeta, version, cb, tipo)
            finally:
                self._servidor_creando = False

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
                self.config_actual["srv_carpeta"] = carpeta
                _lista = self.config_actual.get("srv_lista", [])
                if not any(s.get("carpeta") == carpeta for s in _lista):
                    _lista.append({"carpeta": carpeta, "tipo": tipo, "version": version})
                    self.config_actual["srv_lista"] = _lista
                threading.Thread(target=self._guardar, daemon=True).start()
                cb("online-mode=false → premium y no-premium pueden conectarse")
                cb("✔️ Servidor listo. ¡Ya puedes iniciarlo desde el launcher!")
            else:
                with self._servidor_lock:
                    self._servidor_log.append(f"[ERROR] {msg_result}")

            with self._servidor_lock:
                log_final = list(self._servidor_log)
            servidor_existe = bool(os.path.exists(os.path.join(carpeta, "server.jar")))
            return {"ok": True, "log": log_final, "servidor_existe": servidor_existe}
        except Exception as e:
            self._servidor_creando = False
            return {"ok": False, "error": str(e)}

    def get_servidores_guardados(self):
        srv_lista = list(self.config_actual.get("srv_lista", []))
        if self._servidor_carpeta and not any(s.get("carpeta") == self._servidor_carpeta for s in srv_lista):
            srv_lista.insert(0, {"carpeta": self._servidor_carpeta, "tipo": "paper", "version": "?"})
        result = []
        for s in srv_lista:
            carpeta = s.get("carpeta", "")
            corriendo = (carpeta == self._servidor_carpeta and
                         self._servidor_proc is not None and self._servidor_proc.poll() is None)
            result.append({
                **s,
                "nombre": os.path.basename(carpeta.rstrip("/\\")) or carpeta,
                "existe": os.path.exists(os.path.join(carpeta, "server.jar")),
                "activo": carpeta == self._servidor_carpeta,
                "corriendo": corriendo,
            })
        return {"ok": True, "servidores": result}

    def activar_servidor(self, carpeta):
        if not os.path.exists(os.path.join(carpeta, "server.jar")):
            return {"ok": False, "error": "No se encontr\u00f3 server.jar en esa carpeta"}
        self._servidor_carpeta = carpeta
        self.config_actual["srv_carpeta"] = carpeta
        threading.Thread(target=self._guardar, daemon=True).start()
        return {"ok": True, "carpeta": carpeta}

    def eliminar_servidor_guardado(self, carpeta):
        srv_lista = [s for s in self.config_actual.get("srv_lista", []) if s.get("carpeta") != carpeta]
        self.config_actual["srv_lista"] = srv_lista
        if self._servidor_carpeta == carpeta:
            self._servidor_carpeta = srv_lista[0]["carpeta"] if srv_lista else ""
            self.config_actual["srv_carpeta"] = self._servidor_carpeta
        threading.Thread(target=self._guardar, daemon=True).start()
        return {"ok": True}

    def _detectar_tipo_servidor(self, carpeta):
        """Detecta el tipo de servidor inspeccionando la carpeta (fallback sin _paragua_srv.json)."""
        if not carpeta:
            return 'paper'
        has_geyser = False
        plugins_dir = os.path.join(carpeta, 'plugins')
        if os.path.isdir(plugins_dir):
            for entry in os.listdir(plugins_dir):
                if 'geyser' in entry.lower():
                    has_geyser = True
                    break
        has_fabric = os.path.exists(os.path.join(carpeta, 'fabric-server-launch.jar'))
        if has_fabric and has_geyser:
            return 'fabric-geyser'
        if has_fabric:
            return 'fabric'
        if has_geyser:
            return 'paper-geyser'
        return 'paper'

    def iniciar_servidor(self, carpeta=None):
        if self._servidor_proc and self._servidor_proc.poll() is None:
            return {"ok": False, "error": "El servidor ya está corriendo."}
        carpeta = carpeta or self._servidor_carpeta
        try:
            _srv_info = os.path.join(carpeta or '', '_paragua_srv.json')
            if os.path.exists(_srv_info):
                with open(_srv_info) as _f:
                    _d = json.load(_f)
                    self._servidor_tipo = _d.get('tipo', 'paper')
                    self._playit_bedrock_custom = _d.get('bedrock_address', '')
            else:
                self._servidor_tipo = self._detectar_tipo_servidor(carpeta)
                self._playit_bedrock_custom = ''
        except Exception:
            self._servidor_tipo = self._detectar_tipo_servidor(carpeta)
            self._playit_bedrock_custom = ''
        if not carpeta or not os.path.exists(os.path.join(carpeta, "server.jar")):
            return {"ok": False, "error": "No se encontró server.jar. Priméro creá el servidor."}
        # Kill any stale Java process running in this folder (releases session.lock)
        try:
            import psutil as _psu
            _norm = os.path.normcase(os.path.normpath(carpeta))
            for _p in _psu.process_iter(['pid', 'name', 'cwd']):
                try:
                    if _p.info['name'] in ('java.exe', 'java') and _p.info['cwd'] and \
                            os.path.normcase(os.path.normpath(_p.info['cwd'])) == _norm:
                        _p.kill()
                except Exception:
                    pass
        except Exception:
            pass
        try:
            import minecraft_launcher_lib
            mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            runtime_dir = os.path.join(mc_dir, "runtime")
            java_cmd = "java"
            if os.path.exists(runtime_dir):
                javas = [os.path.join(r, "java.exe") for r, _, fs in os.walk(runtime_dir) if "java.exe" in fs]
                if javas:
                    # Prefer: delta/lts (Java 21) > gamma/beta (Java 17) > alpha (Java 16) > skip jre-legacy
                    _prio = ["delta", "lts", "gamma", "beta", "alpha"]
                    java_ideal = next(
                        (j for p in _prio for j in javas if p in j.lower()),
                        next((j for j in javas if "legacy" not in j.lower()), javas[0])
                    )
                    java_cmd = java_ideal
            self._servidor_carpeta = carpeta
            with self._servidor_lock:
                self._servidor_log.clear()
                self._servidor_log.append(f"[SERVER] Iniciando servidor en {carpeta}...")
                self._servidor_log.append(f"[SERVER] Java: {java_cmd}")
                self._servidor_log.append("[SERVER] Esperando output de Minecraft (puede tardar 30-60 seg)...")
            try:
                import psutil as _ps
                total_gb = _ps.virtual_memory().total / (1024 ** 3)
            except Exception:
                try:
                    import ctypes as _ct
                    _kb = _ct.c_ulonglong(0)
                    _ct.windll.kernel32.GetPhysicallyInstalledSystemMemory(_ct.byref(_kb))
                    total_gb = _kb.value / (1024 ** 2)
                except Exception:
                    total_gb = 4.0
            xmx = max(1, min(8, int(total_gb * 0.5)))
            xms = max(512, xmx * 256)
            xmx_flag = f"-Xmx{xmx}G"
            xms_flag = f"-Xms{xms}M" if xms < 1024 else f"-Xms{xms // 1024}G"
            with self._servidor_lock:
                self._servidor_log.append(f"[SERVER] RAM asignada: {xms_flag} min / {xmx_flag} max (sistema: {total_gb:.1f} GB)")
            flags = subprocess.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
            self._jugadores_online.clear()
            run_bat = os.path.join(carpeta, "run.bat")
            if os.path.exists(run_bat):
                startup_cmd = ["cmd", "/c", "run.bat", "nogui"]
                with self._servidor_lock:
                    self._servidor_log.append("[SERVER] Servidor Forge detectado, usando run.bat")
            else:
                startup_cmd = [java_cmd, xmx_flag, xms_flag, "-jar", "server.jar", "nogui"]
            self._servidor_proc = subprocess.Popen(
                startup_cmd,
                cwd=carpeta, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT, text=True, encoding="utf-8",
                errors="replace", bufsize=1, creationflags=flags,
            )

            _proc_ref = self._servidor_proc

            def _drain_stdout():
                try:
                    for _ in iter(_proc_ref.stdout.readline, ''):
                        pass
                except Exception:
                    pass

            def _stream():
                import time as _time
                log_file = os.path.join(carpeta, "logs", "latest.log")
                # Record the mtime before Java starts so we can detect when Minecraft
                # rotates the log and creates a fresh latest.log
                old_mtime = os.path.getmtime(log_file) if os.path.exists(log_file) else 0
                pos = 0
                # Wait up to 60s for Minecraft to create a FRESH latest.log
                deadline = _time.time() + 60
                while _time.time() < deadline and _proc_ref.poll() is None:
                    if os.path.exists(log_file):
                        try:
                            cur_mtime = os.path.getmtime(log_file)
                            if cur_mtime != old_mtime:
                                break  # fresh file detected
                        except Exception:
                            pass
                    _time.sleep(0.5)
                # Read the fresh log file until the process exits
                while _proc_ref.poll() is None:
                    try:
                        if os.path.exists(log_file):
                            with open(log_file, "r", encoding="utf-8", errors="replace") as _f:
                                _f.seek(pos)
                                chunk = _f.read()
                                if chunk:
                                    import re as _re
                                    for line in chunk.splitlines():
                                        line = line.strip()
                                        if line:
                                            with self._servidor_lock:
                                                self._servidor_log.append(line)
                                                if len(self._servidor_log) > 300:
                                                    self._servidor_log = self._servidor_log[-300:]
                                            _join = _re.search(r': (\w+) joined the game', line)
                                            _left = _re.search(r': (\w+) (?:left the game|lost connection)', line)
                                            if _join:
                                                self._jugadores_online.add(_join.group(1))
                                            elif _left:
                                                self._jugadores_online.discard(_left.group(1))
                                    pos = _f.tell()
                    except Exception:
                        pass
                    _time.sleep(0.5)
                with self._servidor_lock:
                    self._servidor_log.append("[SERVER] Servidor detenido.")
                self._jugadores_online.clear()

            threading.Thread(target=_drain_stdout, daemon=True).start()
            threading.Thread(target=_stream, daemon=True).start()
            try:
                self.actualizar_rpc("Servidor activo", f"🖥 {os.path.basename(carpeta)}")
            except Exception:
                pass
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
            try:
                self._rpc_menu()
            except Exception:
                pass
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def buscar_plugins(self, query):
        try:
            url = "https://hangar.papermc.io/api/v1/projects"
            params = {"q": query, "limit": 12, "sort": "-stars"}
            r = requests.get(url, params=params, timeout=10,
                             headers={"User-Agent": "ParaguacraftLauncher/2.0"})
            r.raise_for_status()
            data = r.json()
            plugins = []
            for p in data.get("result", []):
                ns = p.get("namespace", {})
                plugins.append({
                    "slug": ns.get("slug", ""),
                    "owner": ns.get("owner", ""),
                    "name": p.get("name", ""),
                    "description": (p.get("description") or "")[:120],
                    "downloads": p.get("stats", {}).get("downloads", 0),
                    "stars": p.get("stats", {}).get("stars", 0),
                })
            return {"ok": True, "plugins": plugins}
        except Exception as e:
            return {"ok": False, "error": str(e), "plugins": []}

    def guardar_bedrock_address(self, address, carpeta=None):
        """Guarda la dirección Bedrock personalizada (ej: mailing-theories.gl.at.ply.gg:4698)."""
        carpeta = carpeta or self._servidor_carpeta
        self._playit_bedrock_custom = address or ''
        if carpeta:
            _srv_info = os.path.join(carpeta, '_paragua_srv.json')
            try:
                _d = {}
                if os.path.exists(_srv_info):
                    with open(_srv_info) as _f:
                        _d = json.load(_f)
                _d['bedrock_address'] = address or ''
                with open(_srv_info, 'w') as _f:
                    json.dump(_d, _f)
            except Exception:
                pass
        return {'ok': True}

    def instalar_geyser_servidor(self, carpeta=None):
        carpeta = carpeta or self._servidor_carpeta
        if not carpeta:
            return {"ok": False, "error": "No hay carpeta de servidor seleccionada"}
        try:
            is_fabric = os.path.exists(os.path.join(carpeta, "fabric-server-launch.jar"))
            dest_dir  = os.path.join(carpeta, "mods" if is_fabric else "plugins")
            os.makedirs(dest_dir, exist_ok=True)
            if is_fabric:
                targets = [
                    ("Geyser-Fabric",    "geyser",    "fabric"),
                    ("Floodgate-Fabric", "floodgate", "fabric"),
                ]
            else:
                targets = [
                    ("Geyser-Spigot",    "geyser",    "spigot"),
                    ("Floodgate-Spigot", "floodgate", "spigot"),
                ]
            resultados = []
            for name, slug, platform in targets:
                dest = os.path.join(dest_dir, name + ".jar")
                if os.path.exists(dest) and os.path.getsize(dest) > 100_000:
                    resultados.append(f"✅ {name} ya estaba instalado")
                    continue
                ok_dl = False
                for plat in (platform, "spigot", "paper"):
                    if ok_dl:
                        break
                    try:
                        url = f"https://download.geysermc.org/v2/projects/{slug}/versions/latest/builds/latest/downloads/{plat}"
                        r = requests.get(url, stream=True, timeout=(20, 180), allow_redirects=True)
                        r.raise_for_status()
                        import shutil as _sh
                        with open(dest, "wb") as f:
                            _sh.copyfileobj(r.raw, f, length=65536)
                        if os.path.getsize(dest) > 100_000:
                            ok_dl = True
                        else:
                            os.remove(dest)
                    except Exception:
                        pass
                # Fallback Hangar
                if not ok_dl:
                    try:
                        hangar_slug = "Geyser-Spigot" if slug == "geyser" else "Floodgate"
                        ver_r = requests.get(
                            f"https://hangar.papermc.io/api/v1/projects/GeyserMC/{hangar_slug}/latestrelease",
                            timeout=15, headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                        ver_name = ver_r.text.strip().strip('"')
                        dl_url = f"https://hangar.papermc.io/api/v1/projects/GeyserMC/{hangar_slug}/versions/{ver_name}/PAPER/download"
                        r2 = requests.get(dl_url, stream=True, timeout=(20, 180),
                                          allow_redirects=True, headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                        r2.raise_for_status()
                        import shutil as _sh2
                        with open(dest, "wb") as f:
                            _sh2.copyfileobj(r2.raw, f, length=65536)
                        if os.path.getsize(dest) > 100_000:
                            ok_dl = True
                        else:
                            os.remove(dest)
                    except Exception:
                        pass
                resultados.append(f"✅ {name} instalado" if ok_dl else f"⚠️ {name}: descarga fallida")
            # Guardar tipo en _paragua_srv.json
            tipo_guardado = "fabric-geyser" if is_fabric else "paper-geyser"
            self._servidor_tipo = tipo_guardado
            try:
                with open(os.path.join(carpeta, "_paragua_srv.json"), "w") as f:
                    json.dump({"tipo": tipo_guardado}, f)
            except Exception:
                pass
            return {"ok": True, "msg": " | ".join(resultados)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def instalar_plugin(self, owner, slug, carpeta_server=None):
        try:
            carpeta = carpeta_server or self._servidor_carpeta
            if not carpeta:
                return {"ok": False, "error": "No hay servidor activo seleccionado"}
            plugins_dir = os.path.join(carpeta, "plugins")
            os.makedirs(plugins_dir, exist_ok=True)
            url = f"https://hangar.papermc.io/api/v1/projects/{owner}/{slug}/latestrelease"
            r = requests.get(url, timeout=10, headers={"User-Agent": "ParaguacraftLauncher/2.0"})
            r.raise_for_status()
            ver_name = r.text.strip().strip('"')
            dl_url = f"https://hangar.papermc.io/api/v1/projects/{owner}/{slug}/versions/{ver_name}/PAPER/download"
            r2 = requests.get(dl_url, timeout=90, headers={"User-Agent": "ParaguacraftLauncher/2.0"}, stream=True)
            r2.raise_for_status()
            fname = f"{slug}.jar"
            cd = r2.headers.get("Content-Disposition", "")
            m = re.search(r'filename[^;=\n]*=([\'"]?)([^\'"\;\n]+)\1', cd)
            if m:
                fname = m.group(2).strip()
            out_path = os.path.join(plugins_dir, fname)
            with open(out_path, "wb") as f:
                for chunk in r2.iter_content(65536):
                    if chunk:
                        f.write(chunk)
            size_kb = round(os.path.getsize(out_path) / 1024)
            return {"ok": True, "nombre": fname, "size_kb": size_kb}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_local_plugins_servidor(self, carpeta=None):
        carpeta = carpeta or self._servidor_carpeta
        if not carpeta:
            return {"ok": True, "plugins": [], "dir_tipo": "plugins"}
        is_fabric = os.path.exists(os.path.join(carpeta, "fabric-server-launch.jar"))
        dir_tipo = "mods" if is_fabric else "plugins"
        target_dir = os.path.join(carpeta, dir_tipo)
        if not os.path.isdir(target_dir):
            return {"ok": True, "plugins": [], "dir_tipo": dir_tipo}
        plugins = []
        for f in os.listdir(target_dir):
            if not (f.endswith(".jar") or f.endswith(".jar.disabled")):
                continue
            estado = "Activo" if f.endswith(".jar") else "Desactivado"
            nombre = f.replace(".jar.disabled", "").replace(".jar", "")
            size_kb = 0
            try:
                size_kb = round(os.path.getsize(os.path.join(target_dir, f)) / 1024)
            except Exception:
                pass
            plugins.append({"archivo": f, "nombre": nombre, "estado": estado, "size_kb": size_kb})
        plugins.sort(key=lambda x: x["nombre"].lower())
        return {"ok": True, "plugins": plugins, "dir_tipo": dir_tipo}

    def toggle_local_plugin_servidor(self, archivo, carpeta=None):
        carpeta = carpeta or self._servidor_carpeta
        if not carpeta:
            return False
        is_fabric = os.path.exists(os.path.join(carpeta, "fabric-server-launch.jar"))
        target_dir = os.path.join(carpeta, "mods" if is_fabric else "plugins")
        ruta = os.path.join(target_dir, archivo)
        if not os.path.exists(ruta):
            return False
        try:
            if archivo.endswith(".jar"):
                os.rename(ruta, ruta + ".disabled")
            elif archivo.endswith(".jar.disabled"):
                os.rename(ruta, ruta[:-9])
            return True
        except Exception:
            return False

    def eliminar_plugin_servidor(self, archivo, carpeta=None):
        carpeta = carpeta or self._servidor_carpeta
        if not carpeta:
            return False
        is_fabric = os.path.exists(os.path.join(carpeta, "fabric-server-launch.jar"))
        target_dir = os.path.join(carpeta, "mods" if is_fabric else "plugins")
        ruta = os.path.join(target_dir, archivo)
        try:
            if os.path.exists(ruta):
                os.remove(ruta)
                return True
        except Exception:
            pass
        return False

    def instalar_plugin_b64_servidor(self, nombre, b64, carpeta=None):
        import base64 as _b64
        carpeta = carpeta or self._servidor_carpeta
        if not carpeta:
            return {"ok": False, "error": "No hay servidor seleccionado"}
        if not nombre.lower().endswith(".jar"):
            return {"ok": False, "error": "Solo se aceptan archivos .jar"}
        is_fabric = os.path.exists(os.path.join(carpeta, "fabric-server-launch.jar"))
        target_dir = os.path.join(carpeta, "mods" if is_fabric else "plugins")
        os.makedirs(target_dir, exist_ok=True)
        try:
            data = _b64.b64decode(b64)
            dest = os.path.join(target_dir, nombre)
            with open(dest, "wb") as f:
                f.write(data)
            return {"ok": True, "nombre": nombre, "size_kb": round(len(data) / 1024)}
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

    # ── Whitelist y ban manager ───────────────────────────────────────────────
    def _srv_json_list(self, filename):
        if not self._servidor_carpeta:
            return []
        path = os.path.join(self._servidor_carpeta, filename)
        if not os.path.exists(path):
            return []
        try:
            with open(path, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception:
            return []

    def _srv_json_write(self, filename, data):
        if not self._servidor_carpeta:
            return
        path = os.path.join(self._servidor_carpeta, filename)
        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)

    def whitelist_list(self):
        entries = self._srv_json_list("whitelist.json")
        return {"ok": True, "lista": [e.get("name", "") for e in entries]}

    def whitelist_add(self, nombre):
        nombre = nombre.strip()
        if not nombre:
            return {"ok": False, "error": "Nombre vacío"}
        if self._servidor_proc and self._servidor_proc.poll() is None:
            self.enviar_comando_servidor(f"whitelist add {nombre}")
            self.enviar_comando_servidor("whitelist reload")
        else:
            if not self._servidor_carpeta:
                return {"ok": False, "error": "No hay servidor configurado"}
            entries = self._srv_json_list("whitelist.json")
            if not any(e.get("name", "").lower() == nombre.lower() for e in entries):
                import uuid as _uuid
                entries.append({"uuid": str(_uuid.uuid4()), "name": nombre})
                self._srv_json_write("whitelist.json", entries)
        return {"ok": True}

    def whitelist_remove(self, nombre):
        nombre = nombre.strip()
        if self._servidor_proc and self._servidor_proc.poll() is None:
            self.enviar_comando_servidor(f"whitelist remove {nombre}")
            self.enviar_comando_servidor("whitelist reload")
        else:
            if not self._servidor_carpeta:
                return {"ok": False, "error": "No hay servidor configurado"}
            entries = [e for e in self._srv_json_list("whitelist.json") if e.get("name", "").lower() != nombre.lower()]
            self._srv_json_write("whitelist.json", entries)
        return {"ok": True}

    def ban_list(self):
        entries = self._srv_json_list("banned-players.json")
        return {"ok": True, "lista": [{"name": e.get("name", ""), "razon": e.get("reason", "")} for e in entries]}

    def ban_add(self, nombre, razon=""):
        nombre = nombre.strip()
        if not nombre:
            return {"ok": False, "error": "Nombre vacío"}
        if self._servidor_proc and self._servidor_proc.poll() is None:
            cmd = f"ban {nombre}" + (f" {razon.strip()}" if razon.strip() else "")
            self.enviar_comando_servidor(cmd)
        else:
            if not self._servidor_carpeta:
                return {"ok": False, "error": "No hay servidor configurado"}
            import datetime as _dt, uuid as _uuid3
            entries = self._srv_json_list("banned-players.json")
            if not any(e.get("name", "").lower() == nombre.lower() for e in entries):
                entries.append({"uuid": str(_uuid3.uuid4()), "name": nombre,
                                "reason": razon.strip() or "Banned by admin",
                                "created": _dt.datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S +0000"),
                                "expires": "forever", "source": "Server"})
                self._srv_json_write("banned-players.json", entries)
        return {"ok": True}

    def ban_remove(self, nombre):
        nombre = nombre.strip()
        if self._servidor_proc and self._servidor_proc.poll() is None:
            self.enviar_comando_servidor(f"pardon {nombre}")
        else:
            if not self._servidor_carpeta:
                return {"ok": False, "error": "No hay servidor configurado"}
            entries = [e for e in self._srv_json_list("banned-players.json") if e.get("name", "").lower() != nombre.lower()]
            self._srv_json_write("banned-players.json", entries)
        return {"ok": True}

    # ── Reinicio programado del servidor ──────────────────────────────────────
    _reinicio_timers = []
    _reinicio_horas = 0

    def set_reinicio_programado(self, horas):
        for t in self._reinicio_timers:
            t.cancel()
        self._reinicio_timers = []
        self._reinicio_horas = int(horas)
        if self._reinicio_horas <= 0:
            return {"ok": True, "msg": "Reinicio automático desactivado"}
        self._programar_reinicio_ciclo()
        return {"ok": True, "horas": self._reinicio_horas}

    def _programar_reinicio_ciclo(self):
        secs = self._reinicio_horas * 3600
        avisos = []
        if secs > 600:
            avisos.append((secs - 600, "say §c[SERVIDOR] Reinicio automatico en 10 minutos"))
        if secs > 300:
            avisos.append((secs - 300, "say §c[SERVIDOR] Reinicio automatico en 5 minutos"))
        if secs > 60:
            avisos.append((secs - 60,  "say §4[SERVIDOR] REINICIO EN 1 MINUTO"))
        avisos.append((secs, None))
        def _accion(cmd):
            if cmd:
                self.enviar_comando_servidor(cmd)
            else:
                self._reiniciar_servidor_auto()
        self._reinicio_timers = []
        for delay, cmd in avisos:
            t = threading.Timer(delay, _accion, args=[cmd])
            t.daemon = True
            t.start()
            self._reinicio_timers.append(t)

    def _reiniciar_servidor_auto(self):
        carpeta = self._servidor_carpeta
        self.detener_servidor()
        import time as _t2
        _t2.sleep(8)
        if carpeta:
            self.iniciar_servidor(carpeta)
        if self._reinicio_horas > 0:
            self._programar_reinicio_ciclo()

    def iniciar_playitgg(self, carpeta=None):
        if self._playit_proc and self._playit_proc.poll() is None:
            return {"ok": False, "error": "playit.gg ya está corriendo."}
        carpeta = carpeta or self._servidor_carpeta
        playit_exe = os.path.join(carpeta or "", "playit.exe")
        if not os.path.exists(playit_exe):
            return {"ok": False, "error": "No se encontró playit.exe. Priméro creá el servidor."}
        try:
            self._playit_address = ""
            self._playit_bedrock_address = ""
            if not self._playit_bedrock_custom and carpeta:
                _srv_info = os.path.join(carpeta, '_paragua_srv.json')
                if os.path.exists(_srv_info):
                    try:
                        with open(_srv_info) as _f:
                            _d = json.load(_f)
                            self._playit_bedrock_custom = _d.get('bedrock_address', '')
                    except Exception:
                        pass
            import tempfile as _tmp
            _log_fd, _log_path = _tmp.mkstemp(prefix="playit_", suffix=".log", text=True)
            os.close(_log_fd)
            with self._servidor_lock:
                self._servidor_log.append("[PLAYIT] Iniciando playit.gg...")
                self._servidor_log.append("[PLAYIT] Esperando conexión con el servidor de playit...")
            # Redirect stdout+stderr to a temp file (avoids pipe buffering).
            # File I/O is unbuffered at OS level so output is visible immediately.
            _log_fh = open(_log_path, "w", encoding="utf-8", errors="replace")
            flags = subprocess.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
            self._playit_proc = subprocess.Popen(
                [playit_exe], cwd=carpeta,
                stdout=_log_fh, stderr=_log_fh,
                stdin=subprocess.PIPE,
                creationflags=flags,
            )
            _log_fh.close()  # close our write handle; process keeps its own

            def _tail_log():
                import time as _t, re as _re
                _ansi = _re.compile(r'\x1b(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
                _seen = set()
                pos = 0
                while self._playit_proc.poll() is None:
                    try:
                        with open(_log_path, "r", encoding="utf-8", errors="replace") as _f:
                            _f.seek(pos)
                            chunk = _f.read()
                            if chunk:
                                # Process every \r-separated segment (spinner may hide addresses in earlier parts)
                                _all_lines = []
                                for raw in chunk.split('\n'):
                                    for part in raw.split('\r'):
                                        _l = _ansi.sub('', part).strip()
                                        if _l:
                                            _all_lines.append(_l)
                                for line in _all_lines:
                                    if not line or line in _seen:
                                        continue
                                    _seen.add(line)
                                    if len(_seen) > 400:
                                        _seen.clear()
                                    with self._servidor_lock:
                                        self._servidor_log.append("[PLAYIT] " + line)
                                        if len(self._servidor_log) > 300:
                                            self._servidor_log = self._servidor_log[-300:]
                                    # Auto-open claim URL in browser
                                    claim = _re.search(r"(https://playit\.gg/claim/\S+)", line)
                                    if claim:
                                        import webbrowser as _wb
                                        _wb.open(claim.group(1))
                                    # Detect both Java (TCP) and Bedrock (UDP) addresses
                                    _addr_m = _re.search(
                                        r"((?:[\w\-]+\.)+(?:ply\.gg|joinmc\.link|auto\.playit\.gg|playit\.gg)(?::\d+)?)",
                                        line)
                                    if not _addr_m:
                                        _addr_m = _re.search(
                                            r"address[=:\s]+([\w\-.]+(?::\d+)?)", line, _re.IGNORECASE)
                                    if _addr_m:
                                        _addr = _addr_m.group(1)
                                        _port_in_addr = _re.search(r':(\d+)$', _addr)
                                        _addr_port    = int(_port_in_addr.group(1)) if _port_in_addr else None
                                        # Domain-based classification (highest priority)
                                        # joinmc.link → always Java regardless of keywords
                                        _domain_java    = bool(_re.search(r'joinmc\.link', _addr))
                                        # ply.gg with non-25565 port → always Bedrock
                                        _domain_bedrock = (bool(_re.search(r'ply\.gg', _addr))
                                                           and _addr_port is not None and _addr_port != 25565)
                                        if _domain_java:
                                            _is_udp = False
                                        elif _domain_bedrock:
                                            _is_udp = True
                                        else:
                                            # Fallback: port/keyword heuristics
                                            _line_lo = line.lower()
                                            _has_tcp = bool(_re.search(r'\btcp\b', _line_lo))
                                            _has_udp = bool(_re.search(r'\budp\b|\bbedrock\b', _line_lo))
                                            _port_bedrock = bool(_re.search(r'\b19132\b', line))
                                            _port_java    = bool(_re.search(r'\b25565\b', line))
                                            _force_java    = _port_java or _addr_port == 25565
                                            _force_bedrock = _port_bedrock or _addr_port == 19132
                                            _proto_udp     = _has_udp and not _has_tcp and not _force_java
                                            _fallback_udp  = (_addr_port is not None and _addr_port != 25565
                                                              and not _force_java and not _has_tcp)
                                            _is_udp = _force_bedrock or (_proto_udp and not _force_java) or _fallback_udp
                                        _es_geyser = self._servidor_tipo in ('paper-geyser', 'fabric-geyser')
                                        if _is_udp and not self._playit_bedrock_address and _es_geyser:
                                            self._playit_bedrock_address = _addr
                                            with self._servidor_lock:
                                                self._servidor_log.append(
                                                    f"[PLAYIT] 🎮 Bedrock (Geyser) detectado: {_addr} — compartí esta dirección y puerto con tus jugadores Bedrock")
                                        elif not _is_udp and not self._playit_address:
                                            self._playit_address = _addr
                                            with self._servidor_lock:
                                                self._servidor_log.append(
                                                    f"[PLAYIT] ☕ Java detectado: {_addr} — compartí esta IP con tus jugadores Java")
                                            if _es_geyser and self._playit_bedrock_address:
                                                with self._servidor_lock:
                                                    self._servidor_log.append(
                                                        f"[PLAYIT] ✅ Ambos túneles activos → Java: {self._playit_address} | Bedrock: {self._playit_bedrock_address}")
                                pos = _f.tell()
                    except Exception:
                        pass
                    _t.sleep(1)
                try:
                    os.remove(_log_path)
                except Exception:
                    pass

            threading.Thread(target=_tail_log, daemon=True).start()

            def _probe_tuneles():
                pass  # log-based detection handled in _tail_log

            threading.Thread(target=_probe_tuneles, daemon=True).start()
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def detener_playitgg(self):
        try:
            if self._playit_proc and self._playit_proc.poll() is None:
                self._playit_proc.kill()
            self._playit_proc = None
            self._playit_address = ""
            self._playit_bedrock_address = ""
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_estado_servidor(self):
        corriendo = self._servidor_proc is not None and self._servidor_proc.poll() is None
        playit_corriendo = self._playit_proc is not None and self._playit_proc.poll() is None
        with self._servidor_lock:
            log_reciente = list(self._servidor_log)
        carpeta = self._servidor_carpeta or ""
        servidor_existe = bool(carpeta and os.path.exists(os.path.join(carpeta, "server.jar")))
        return {
            "corriendo": corriendo,
            "carpeta": carpeta,
            "servidor_existe": servidor_existe,
            "playit_corriendo": playit_corriendo,
            "playit_address": self._playit_address,
            "playit_bedrock_address": self._playit_bedrock_custom or self._playit_bedrock_address,
            "servidor_tipo": self._servidor_tipo if self._servidor_tipo != 'paper' else self._detectar_tipo_servidor(self._servidor_carpeta or ''),
            "log": log_reciente,
            "creando": self._servidor_creando,
        }


    def guardar_skin(self, skin_url, nombre="skin"):
        import re
        try:
            if not skin_url:
                return {"ok": False, "error": "URL vacía"}
            r = requests.get(skin_url, timeout=10)
            r.raise_for_status()
            skins_dir = os.path.join(os.path.expanduser("~"), "Paraguacraft_Skins")
            os.makedirs(skins_dir, exist_ok=True)
            safe_name = re.sub(r"[^a-zA-Z0-9_\-]", "_", nombre)[:40] or "skin"
            out_path = os.path.join(skins_dir, safe_name + ".png")
            with open(out_path, "wb") as f:
                f.write(r.content)
            self._guardar_historial_skin(nombre, skin_url)
            return {"ok": True, "path": out_path}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def aplicar_skin_offline(self, skin_path):
        """Copia la skin a todas las instancias Paraguacraft para usuarios offline."""
        try:
            if not os.path.isfile(skin_path):
                return {"ok": False, "error": "Archivo no encontrado"}
            import minecraft_launcher_lib as _mcl_sk
            mc_dir = _mcl_sk.utils.get_minecraft_directory()
            pack_name = "ParaguacraftBrandPack"
            applied = 0
            instancias_root = os.path.join(mc_dir, "instancias")
            if os.path.isdir(instancias_root):
                for inst in os.listdir(instancias_root):
                    pack_path = os.path.join(instancias_root, inst, "resourcepacks", pack_name)
                    if not os.path.isdir(pack_path):
                        continue
                    wide = os.path.join(pack_path, "assets", "minecraft", "textures", "entity", "player", "wide")
                    slim = os.path.join(pack_path, "assets", "minecraft", "textures", "entity", "player", "slim")
                    old  = os.path.join(pack_path, "assets", "minecraft", "textures", "entity")
                    for d in [wide, slim, old]:
                        os.makedirs(d, exist_ok=True)
                    shutil.copy2(skin_path, os.path.join(wide, "steve.png"))
                    shutil.copy2(skin_path, os.path.join(slim, "alex.png"))
                    shutil.copy2(skin_path, os.path.join(old,  "steve.png"))
                    applied += 1
            skin_store = os.path.join(mc_dir, "paraguacraft_offline_skin.png")
            shutil.copy2(skin_path, skin_store)
            msg = f"Skin aplicada a {applied} instancia(s)." if applied else "Skin guardada. Se aplicará al iniciar el juego."
            return {"ok": True, "msg": msg}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def aplicar_skin_url(self, url, variante="classic"):
        """Descarga una skin desde URL y la aplica para el usuario actual."""
        try:
            import re, tempfile
            r = requests.get(url, timeout=10)
            r.raise_for_status()
            with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as tmp:
                tmp.write(r.content)
                tmp_path = tmp.name
            if self.ms_data:
                result = self.subir_skin_premium(tmp_path, variante)
            else:
                result = self.aplicar_skin_offline(tmp_path)
            try: os.remove(tmp_path)
            except Exception: pass
            return result
        except Exception as e:
            return {"ok": False, "error": str(e)}

    _HISTORIAL_FILE = os.path.join(os.path.expanduser("~"), "Paraguacraft_Skins", "historial.json")

    def get_skin_historial(self):
        try:
            if os.path.exists(self._HISTORIAL_FILE):
                with open(self._HISTORIAL_FILE, "r", encoding="utf-8") as f:
                    return {"historial": json.load(f)}
        except Exception:
            pass
        return {"historial": []}

    def _guardar_historial_skin(self, nombre, url, tipo="classic"):
        try:
            os.makedirs(os.path.dirname(self._HISTORIAL_FILE), exist_ok=True)
            hist = self.get_skin_historial().get("historial", [])
            hist = [h for h in hist if h.get("url") != url]
            hist.insert(0, {"nombre": nombre, "url": url, "tipo": tipo})
            with open(self._HISTORIAL_FILE, "w", encoding="utf-8") as f:
                json.dump(hist[:50], f, ensure_ascii=False)
        except Exception:
            pass

    def limpiar_skin_historial(self):
        try:
            if os.path.exists(self._HISTORIAL_FILE):
                os.remove(self._HISTORIAL_FILE)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def subir_skin_premium(self, skin_path, variante="classic"):
        if not self.ms_data:
            return {"ok": False, "error": "Se necesita cuenta Premium conectada"}
        try:
            token = self.ms_data.get("access_token", "")
            url = "https://api.minecraftservices.com/minecraft/profile/skins"
            headers = {"Authorization": f"Bearer {token}"}
            with open(skin_path, "rb") as f:
                skin_bytes = f.read()
            files = [("file", ("skin.png", skin_bytes, "image/png"))]
            data = {"variant": variante}
            r = requests.post(url, headers=headers, data=data, files=files, timeout=10)
            if r.status_code == 200:
                return {"ok": True}
            return {"ok": False, "error": f"HTTP {r.status_code}: {r.text[:120]}"}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def seleccionar_archivo_generico(self, extension='*.jar;*.zip', titulo='Seleccionar archivo'):
        try:
            result = webview.windows[0].create_file_dialog(
                webview.OPEN_DIALOG,
                allow_multiple=False,
                file_types=(f'Archivos ({extension})', 'Todos los archivos (*.*)')
            )
            return result[0] if result else ''
        except Exception:
            return ''

    def seleccionar_skin_local(self):
        try:
            import base64
            result = webview.windows[0].create_file_dialog(
                webview.OPEN_DIALOG,
                allow_multiple=False,
                file_types=('Imágenes PNG (*.png)', 'Todos los archivos (*.*)')
            )
            path = result[0] if result else None
            if not path:
                return {"ok": False, "cancelled": True}
            with open(path, "rb") as f:
                raw = f.read()
            b64 = base64.b64encode(raw).decode()
            return {
                "ok": True,
                "path": path.replace("\\", "/"),
                "data_url": f"data:image/png;base64,{b64}"
            }
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_ultima_version_jugada(self):
        return {
            "version": self.config_actual.get("ultima_version", ""),
            "motor": self.config_actual.get("ultimo_motor", "Vanilla"),
        }

    def get_versiones_mod(self, project_id, version_mc, tipo):
        try:
            params = {}
            if version_mc:
                params["game_versions"] = json.dumps([version_mc])
            url = f"https://api.modrinth.com/v2/project/{project_id}/version"
            r = requests.get(url, timeout=10, params=params,
                             headers={"User-Agent": "Paraguacraft-Launcher"})
            data = r.json()
            versions = []
            for v in (data if isinstance(data, list) else [])[:50]:
                f0 = v.get("files", [{}])[0]
                versions.append({
                    "id": v["id"],
                    "name": v.get("name", v["id"]),
                    "game_versions": v.get("game_versions", []),
                    "loaders": v.get("loaders", []),
                    "downloads": v.get("downloads", 0),
                    "date_published": v.get("date_published", "")[:10],
                    "size_mb": round(f0.get("size", 0) / (1024 * 1024), 2),
                    "filename": f0.get("filename", ""),
                })
            return {"ok": True, "versions": versions}
        except Exception as e:
            return {"ok": False, "error": str(e), "versions": []}

    def get_mod_details(self, project_id):
        try:
            r = requests.get(
                f"https://api.modrinth.com/v2/project/{project_id}",
                timeout=10, headers={"User-Agent": "Paraguacraft-Launcher"})
            d = r.json()
            return {
                "ok": True,
                "title": d.get("title", ""),
                "description": d.get("description", ""),
                "icon_url": d.get("icon_url", ""),
                "gallery": [g.get("url", "") for g in d.get("gallery", [])[:6]],
                "categories": d.get("categories", []),
                "loaders": d.get("loaders", []),
                "game_versions": d.get("game_versions", []),
                "downloads": d.get("downloads", 0),
                "followers": d.get("followers", 0),
                "project_type": d.get("project_type", "mod"),
                "client_side": d.get("client_side", ""),
                "server_side": d.get("server_side", ""),
                "slug": d.get("slug", project_id),
            }
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def instalar_mod_desde_modrinth(self, project_id, version_id, tipo, version_mc, motor):
        try:
            url = f"https://api.modrinth.com/v2/version/{version_id}"
            r = requests.get(url, timeout=10, headers={"User-Agent": "Paraguacraft-Launcher"})
            data = r.json()
            files = data.get("files", [])
            primary = next((f for f in files if f.get("primary")), files[0] if files else None)
            if not primary:
                return {"ok": False, "error": "No se encontró archivo primario"}
            download_url = primary["url"]
            filename = primary["filename"]
            from core import carpeta_instancia_paraguacraft
            import minecraft_launcher_lib
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            folder = carpeta_instancia_paraguacraft(version_mc.strip(), motor)
            base = os.path.join(mine_dir, "instancias", folder)
            tipo_mapa = {"mod": "mods", "resourcepack": "resourcepacks",
                         "shader": "shaderpacks", "datapack": "datapacks",
                         "modpack": "mods", "plugin": "plugins"}
            dest_dir = os.path.join(base, tipo_mapa.get(tipo, "mods"))
            os.makedirs(dest_dir, exist_ok=True)
            dest_path = os.path.join(dest_dir, filename)
            self._mod_dl_progress = {"pct": 0, "nombre": filename}
            r2 = requests.get(download_url, stream=True, timeout=120,
                              headers={"User-Agent": "Paraguacraft-Launcher"})
            r2.raise_for_status()
            total = int(r2.headers.get("content-length", 0))
            downloaded = 0
            with open(dest_path, "wb") as f:
                for chunk in r2.iter_content(65536):
                    if chunk:
                        f.write(chunk)
                        downloaded += len(chunk)
                        if total > 0:
                            self._mod_dl_progress["pct"] = int(downloaded * 100 / total)
            self._mod_dl_progress = {"pct": 100, "nombre": filename}
            return {"ok": True, "nombre": filename}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_mod_dl_progress(self):
        return dict(self._mod_dl_progress)

    def instalar_mod_modrinth(self, slug, version, motor, extra=""):
        try:
            loader = motor.lower().replace(" + iris", "").replace("optifine", "").strip().replace(" ", "-") or "fabric"
            sr = requests.get(
                f"https://api.modrinth.com/v2/search?query={requests.utils.quote(slug)}&limit=1",
                headers={"User-Agent": "Paraguacraft-Launcher"}, timeout=8).json()
            if not sr.get("hits"):
                return {"ok": False, "error": f"No se encontró '{slug}' en Modrinth"}
            pid = sr["hits"][0]["project_id"]
            params = {"loaders": f'["{loader}"]'}
            if version:
                params["game_versions"] = f'["{version}"]'
            vr = requests.get(
                f"https://api.modrinth.com/v2/project/{pid}/version",
                params=params, headers={"User-Agent": "Paraguacraft-Launcher"}, timeout=8).json()
            if not vr or not isinstance(vr, list):
                params_retry = {k: v for k, v in params.items() if k != "loaders"}
                vr = requests.get(
                    f"https://api.modrinth.com/v2/project/{pid}/version",
                    params=params_retry, headers={"User-Agent": "Paraguacraft-Launcher"}, timeout=8).json()
            if not vr or not isinstance(vr, list):
                return {"ok": False, "error": f"Sin versiones compatibles para '{slug}'"}
            return self.instalar_mod_desde_modrinth(pid, vr[0]["id"], "mod", version, motor)
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_limpieza_info(self):
        try:
            import minecraft_launcher_lib
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            instancias_dir = os.path.join(mine_dir, "instancias")
            logs_bytes = _tamano_carpeta(os.path.join(mine_dir, "logs"))
            crash_bytes = _tamano_carpeta(os.path.join(mine_dir, "crash-reports"))
            if os.path.isdir(instancias_dir):
                for inst in os.listdir(instancias_dir):
                    inst_path = os.path.join(instancias_dir, inst)
                    logs_bytes += _tamano_carpeta(os.path.join(inst_path, "logs"))
                    crash_bytes += _tamano_carpeta(os.path.join(inst_path, "crash-reports"))
            mc_ram_mb = 0
            for proc in psutil.process_iter(["name", "memory_info"]):
                try:
                    if "java" in (proc.info["name"] or "").lower():
                        mc_ram_mb += proc.info["memory_info"].rss // (1024 ** 2)
                except Exception:
                    pass
            return {"ok": True, "logs_mb": round(logs_bytes / (1024 ** 2), 1),
                    "crash_mb": round(crash_bytes / (1024 ** 2), 1), "mc_ram_mb": mc_ram_mb}
        except Exception as e:
            return {"ok": False, "error": str(e), "logs_mb": 0, "crash_mb": 0, "mc_ram_mb": 0}

    def ejecutar_limpieza(self, tipo):
        try:
            import minecraft_launcher_lib
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            instancias_dir = os.path.join(mine_dir, "instancias")
            borrados = 0
            _dirs_logs = [os.path.join(mine_dir, "logs")]
            _dirs_crash = [os.path.join(mine_dir, "crash-reports")]
            if os.path.isdir(instancias_dir):
                for inst in os.listdir(instancias_dir):
                    inst_path = os.path.join(instancias_dir, inst)
                    _dirs_logs.append(os.path.join(inst_path, "logs"))
                    _dirs_crash.append(os.path.join(inst_path, "crash-reports"))
            if tipo in ("logs", "ambos"):
                for logs_dir in _dirs_logs:
                    if os.path.isdir(logs_dir):
                        for f in os.listdir(logs_dir):
                            if f != "latest.log":
                                try:
                                    os.remove(os.path.join(logs_dir, f))
                                    borrados += 1
                                except Exception:
                                    pass
            if tipo in ("crash", "ambos"):
                for crash_dir in _dirs_crash:
                    if os.path.isdir(crash_dir):
                        for f in os.listdir(crash_dir):
                            try:
                                os.remove(os.path.join(crash_dir, f))
                                borrados += 1
                            except Exception:
                                pass
            return {"ok": True, "borrados": borrados}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def verificar_actualizacion(self):
        try:
            url = f"https://api.github.com/repos/{GITHUB_REPO}/releases/latest"
            r = requests.get(url, timeout=8, headers={"User-Agent": "Paraguacraft-Launcher"})
            data = r.json()
            tag = data.get("tag_name", "").strip().lstrip("v").lstrip(".")
            if not tag:
                return {"actualizar": False, "version_actual": VERSION}
            def _ver_tuple(v):
                try: return tuple(int(x) for x in v.strip().lstrip("v").lstrip(".").split("."))
                except: return (0,)
            if _ver_tuple(tag) > _ver_tuple(VERSION):
                exe_url = None
                for asset in data.get("assets", []):
                    name = asset.get("name", "").lower()
                    if name.endswith(".exe") and not any(kw in name for kw in ("setup", "install", "instalar")):
                        exe_url = asset.get("browser_download_url")
                        break
                html_url = data.get("html_url", f"https://github.com/{GITHUB_REPO}/releases/latest")
                return {
                    "actualizar": True,
                    "version_actual": VERSION,
                    "version_nueva": tag,
                    "url": exe_url,
                    "html_url": html_url,
                    "notas": data.get("body", "")[:600],
                }
            return {"actualizar": False, "version_actual": VERSION}
        except Exception as e:
            return {"actualizar": False, "version_actual": VERSION, "error": str(e)}

    def aplicar_actualizacion(self, download_url):
        import tempfile, shutil as _sh
        if not getattr(sys, "frozen", False):
            return {"ok": False, "error": "Solo funciona en el ejecutable compilado (.exe)."}
        if not download_url:
            return {"ok": False, "error": "Sin URL de descarga directa."}
        def _notify_err(msg):
            try:
                webview.windows[0].evaluate_js(f"_updateFailed({json.dumps(str(msg))})")
            except Exception: pass
        def _hilo():
            tmp = os.path.join(tempfile.gettempdir(), "Paraguacraft_update.exe")
            exe_actual = sys.executable
            old_exe = exe_actual + ".old"
            marker = os.path.join(tempfile.gettempdir(), "paraguacraft_updated.flag")
            try:
                # ── Descarga ─────────────────────────────────────────────────
                r = requests.get(download_url, stream=True, timeout=180,
                                 headers={"User-Agent": "Paraguacraft-Launcher"})
                if r.status_code == 404:
                    _notify_err(f"Archivo no encontrado en GitHub (404). Verific\u00e1 que el release '{VERSION}' tenga Paraguacraft.exe adjunto como asset.")
                    return
                r.raise_for_status()
                with open(tmp, "wb") as f:
                    for chunk in r.iter_content(65536):
                        if chunk: f.write(chunk)

                # ── Verificar descarga ────────────────────────────────────────
                dl_size = os.path.getsize(tmp) if os.path.exists(tmp) else 0
                if dl_size < 500_000:
                    _notify_err(f"Descarga incompleta o corrupta ({dl_size} bytes). Intent\u00e1 de nuevo.")
                    try: os.remove(tmp)
                    except Exception: pass
                    return

                # ── Estrategia 1: rename en proceso (sin PowerShell) ──────────
                # Windows permite renombrar un .exe en ejecuci\u00f3n. Al renombrar
                # el ejecutable actual, su path queda libre y podemos copiar el
                # nuevo sin que ning\u00fan antivirus lo vea como overwrite de proceso.
                try:
                    if os.path.exists(old_exe):
                        os.remove(old_exe)
                    os.rename(exe_actual, old_exe)   # renombrar exe en uso ✓
                    try:
                        _sh.copy2(tmp, exe_actual)   # copiar nuevo al path libre
                        try: os.remove(tmp)
                        except Exception: pass
                    except Exception:
                        # rollback: restaurar exe original si la copia fall\u00f3
                        try: os.rename(old_exe, exe_actual)
                        except Exception: pass
                        raise
                    try: open(marker, "w").write("updated")
                    except Exception: pass
                    subprocess.Popen(
                        [exe_actual],
                        creationflags=subprocess.DETACHED_PROCESS | subprocess.CREATE_NEW_PROCESS_GROUP,
                    )
                    os._exit(0)
                except OSError as _oe:
                    print(f"[Updater] Rename in-process fall\u00f3 ({_oe}), usando PowerShell...")

                # ── Estrategia 2: PowerShell (fallback para carpetas UAC) ─────
                ps_path = os.path.join(tempfile.gettempdir(), "paragua_updater.ps1")
                tmp_esc    = tmp.replace("'", "''")
                exe_esc    = exe_actual.replace("'", "''")
                old_esc    = old_exe.replace("'", "''")
                marker_esc = marker.replace("'", "''")
                ps = [
                    "Start-Sleep -Seconds 4",
                    "$ok = $false",
                    "for ($i = 0; $i -lt 10; $i++) {",
                    "    try {",
                    f"        if (Test-Path '{old_esc}') {{ Remove-Item -Force '{old_esc}' -EA Stop }}",
                    f"        Rename-Item '{exe_esc}' '{old_esc}' -EA Stop",
                    f"        Copy-Item -Force '{tmp_esc}' '{exe_esc}' -EA Stop",
                    f"        Remove-Item -Force '{tmp_esc}' -EA SilentlyContinue",
                    "        $ok = $true; break",
                    "    } catch { Start-Sleep -Seconds 2 }",
                    "}",
                    "if ($ok) {",
                    f"    Set-Content '{marker_esc}' 'updated' -EA SilentlyContinue",
                    f"    Start-Process '{exe_esc}'",
                    f"    Start-Sleep -Seconds 2",
                    f"    Remove-Item -Force '{old_esc}' -EA SilentlyContinue",
                    "}",
                    "Remove-Item -Force $PSCommandPath -EA SilentlyContinue",
                ]
                with open(ps_path, "w", encoding="utf-8-sig") as f:
                    f.write("\n".join(ps))
                subprocess.Popen(
                    ["powershell", "-NonInteractive", "-NoProfile",
                     "-ExecutionPolicy", "Bypass", "-WindowStyle", "Hidden",
                     "-File", ps_path],
                    creationflags=subprocess.DETACHED_PROCESS | subprocess.CREATE_NEW_PROCESS_GROUP,
                    close_fds=True,
                )
                os._exit(0)
            except Exception as e:
                print(f"[Updater] Error: {e}")
                _notify_err(str(e)[:300])
        threading.Thread(target=_hilo, daemon=True).start()
        return {"ok": True}


    # ── VARINT HELPERS ──────────────────────────────────────────────────
    def _pack_varint(self, val):
        out = bytearray()
        while True:
            b = val & 0x7F; val >>= 7
            out.append(b | 0x80 if val else b)
            if not val: break
        return bytes(out)

    def _read_varint_bytes(self, data, offset):
        n = shift = 0
        while offset < len(data):
            b = data[offset]; offset += 1
            n |= (b & 0x7F) << shift; shift += 7
            if not (b & 0x80): break
        return n, offset

    # ── PING SERVIDOR MC ────────────────────────────────────────────────
    def ping_servidor(self, ip, puerto=25565):
        try:
            puerto = int(puerto)
            t0 = time.time()
            with socket.create_connection((ip, puerto), timeout=4) as s:
                host_b = ip.encode("utf-8")
                data = bytearray()
                data += b"\x00"
                data += b"\x00"
                data += self._pack_varint(len(host_b)) + host_b
                data += struct.pack(">H", puerto)
                data += b"\x01"
                s.sendall(self._pack_varint(len(data)) + bytes(data))
                s.sendall(b"\x01\x00")
                raw = b""
                while len(raw) < 5:
                    chunk = s.recv(4096)
                    if not chunk: break
                    raw += chunk
                pkt_len, off = self._read_varint_bytes(raw, 0)
                while len(raw) < off + pkt_len:
                    chunk = s.recv(4096)
                    if not chunk: break
                    raw += chunk
                _, off = self._read_varint_bytes(raw, off)
                str_len, off = self._read_varint_bytes(raw, off)
                srv = json.loads(raw[off:off + str_len].decode("utf-8"))
            latency = int((time.time() - t0) * 1000)
            desc = srv.get("description", "")
            motd = desc if isinstance(desc, str) else desc.get("text", "")
            return {
                "ok": True, "latency_ms": latency,
                "players_online": srv.get("players", {}).get("online", 0),
                "players_max": srv.get("players", {}).get("max", 0),
                "version": srv.get("version", {}).get("name", "?"),
                "motd": motd,
            }
        except Exception as e:
            return {"ok": False, "error": str(e), "latency_ms": -1}

    # ── JUGADORES ONLINE GLOBAL ──────────────────────────────────────────
    _SERVIDORES_GLOBAL = [
        ("mc.hypixel.net",          25565),
        ("play.cubecraft.net",       25565),
        ("play.wynncraft.com",       25565),
        ("minemen.club",             25565),
        ("play.jartexnetwork.com",   25565),
        ("extremecraft.net",         25565),
        ("play.fadecloud.com",       25565),
        ("play.minesuperior.com",    25565),
    ]

    def get_jugadores_online_global(self):
        resultados = []
        lock = threading.Lock()

        def _ping_one(ip, puerto):
            r = self.ping_servidor(ip, puerto)
            if r.get("ok") and r.get("players_online", 0) > 0:
                with lock:
                    resultados.append({
                        "ip": ip,
                        "online": r["players_online"],
                        "max": r["players_max"],
                    })

        hilos = [
            threading.Thread(target=_ping_one, args=(ip, puerto), daemon=True)
            for ip, puerto in self._SERVIDORES_GLOBAL
        ]
        for h in hilos:
            h.start()
        for h in hilos:
            h.join(timeout=5)

        total = sum(r["online"] for r in resultados)
        return {"ok": True, "total": total, "servidores": resultados,
                "respondieron": len(resultados)}

    # ── DETECTOR MODS CONFLICTIVOS ───────────────────────────────────────
    def get_mods_conflictivos(self, version, motor):
        try:
            mods_dir = os.path.join(self._dir_instancia(version, motor), "mods")
            if not os.path.isdir(mods_dir):
                return {"ok": True, "conflictos": [], "total": 0}
            ids_vistos = {}
            for archivo in os.listdir(mods_dir):
                if not archivo.endswith((".jar", ".jar.disabled")):
                    continue
                path = os.path.join(mods_dir, archivo)
                try:
                    with zipfile.ZipFile(path, "r") as z:
                        mod_id = None
                        names = z.namelist()
                        if "fabric.mod.json" in names:
                            mod_id = json.loads(z.read("fabric.mod.json")).get("id")
                        elif "quilt.mod.json" in names:
                            mod_id = json.loads(z.read("quilt.mod.json")).get("quilt_loader", {}).get("id")
                        elif "META-INF/mods.toml" in names:
                            content = z.read("META-INF/mods.toml").decode("utf-8", errors="ignore")
                            m = re.search(r'modId\s*=\s*"([^"]+)"', content)
                            if m: mod_id = m.group(1)
                        if mod_id:
                            ids_vistos.setdefault(mod_id, []).append(archivo)
                except Exception:
                    pass
            conflictos = [{"mod_id": k, "archivos": v} for k, v in ids_vistos.items() if len(v) > 1]
            return {"ok": True, "conflictos": conflictos, "total": len(conflictos)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── OPTIMIZADOR JAVA ─────────────────────────────────────────────────
    def get_java_recomendado(self):
        try:
            ram_gb = psutil.virtual_memory().total / (1024 ** 3)
            cores = psutil.cpu_count(logical=False) or 2
            ram_rec = max(2, min(8, int(ram_gb * 0.35)))
            gc = "ZGC" if ram_gb >= 16 else "G1GC"
            gc_why = ("Tenés mucha RAM — ZGC elimina lag spikes." if ram_gb >= 16
                      else "G1GC es el mejor equilibrio para tu RAM.")
            flags = [f"-Xmx{ram_rec}G", f"-Xms{max(1, ram_rec//2)}G",
                     "-XX:+UnlockExperimentalVMOptions", f"-XX:+Use{gc}GC",
                     "-XX:+AlwaysPreTouch"]
            if gc == "G1GC":
                flags += ["-XX:MaxGCPauseMillis=200", "-XX:G1NewSizePercent=20",
                          "-XX:G1MaxNewSizePercent=40", "-XX:G1HeapRegionSize=16M"]
            rating = "Alto rendimiento" if ram_gb >= 16 else ("Buen rendimiento" if ram_gb >= 8 else "Rendimiento básico")
            return {"ok": True, "ram_total_gb": round(ram_gb, 1), "ram_rec_gb": ram_rec,
                    "cpu_cores": cores, "gc_rec": gc, "gc_why": gc_why,
                    "flags": flags, "rating": rating}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def aplicar_java_recomendado(self):
        try:
            rec = self.get_java_recomendado()
            if rec["ok"]:
                self.config_actual["ram_asignada"] = rec["ram_rec_gb"]
                self.config_actual["gc_type"] = rec["gc_rec"]
                self._guardar()
                return {"ok": True, "ram": rec["ram_rec_gb"], "gc": rec["gc_rec"]}
            return rec
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── BACKUP DE MUNDOS ─────────────────────────────────────────────────
    def get_mundos(self):
        try:
            import minecraft_launcher_lib
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            instancias_dir = os.path.join(mine_dir, "instancias")
            saves_dirs = []
            if os.path.isdir(instancias_dir):
                for inst in os.listdir(instancias_dir):
                    s = os.path.join(instancias_dir, inst, "saves")
                    if os.path.isdir(s):
                        saves_dirs.append(s)
            global_saves = os.path.join(mine_dir, "saves")
            if os.path.isdir(global_saves):
                saves_dirs.append(global_saves)
            mundos = []
            vistos = set()
            for saves in saves_dirs:
                for nombre in os.listdir(saves):
                    ruta = os.path.join(saves, nombre)
                    if os.path.isdir(ruta) and nombre not in vistos:
                        vistos.add(nombre)
                        stat = os.stat(ruta)
                        mundos.append({"nombre": nombre, "ruta_saves": saves,
                                       "modificado": time.strftime("%d/%m/%Y %H:%M", time.localtime(stat.st_mtime)),
                                       "tamano_mb": round(_tamano_carpeta(ruta) / (1024 * 1024), 1)})
            mundos.sort(key=lambda x: x["modificado"], reverse=True)
            return {"ok": True, "mundos": mundos}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def backup_mundo(self, nombre_mundo):
        try:
            import minecraft_launcher_lib
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            instancias_dir = os.path.join(mine_dir, "instancias")
            mundo_dir = None
            if os.path.isdir(instancias_dir):
                for inst in os.listdir(instancias_dir):
                    candidate = os.path.join(instancias_dir, inst, "saves", nombre_mundo)
                    if os.path.isdir(candidate):
                        mundo_dir = candidate
                        break
            if mundo_dir is None:
                candidate = os.path.join(mine_dir, "saves", nombre_mundo)
                if os.path.isdir(candidate):
                    mundo_dir = candidate
            if mundo_dir is None:
                return {"ok": False, "error": "Mundo no encontrado."}
            backup_dir = os.path.join(os.path.dirname(os.path.abspath(self.ruta_config)), "paraguacraft_backups")
            os.makedirs(backup_dir, exist_ok=True)
            ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
            nombre_zip = f"{nombre_mundo}_{ts}.zip"
            ruta_zip = os.path.join(backup_dir, nombre_zip)
            with zipfile.ZipFile(ruta_zip, "w", zipfile.ZIP_DEFLATED) as zf:
                for root, _, files in os.walk(mundo_dir):
                    for f in files:
                        fp = os.path.join(root, f)
                        zf.write(fp, os.path.relpath(fp, os.path.dirname(mundo_dir)))
            return {"ok": True, "archivo": nombre_zip, "tamano_mb": round(os.path.getsize(ruta_zip) / (1024 * 1024), 1)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def listar_backups(self):
        try:
            backup_dir = os.path.join(os.path.dirname(os.path.abspath(self.ruta_config)), "paraguacraft_backups")
            if not os.path.isdir(backup_dir):
                return {"ok": True, "backups": []}
            backups = []
            for f in os.listdir(backup_dir):
                if not f.endswith(".zip"): continue
                ruta = os.path.join(backup_dir, f)
                stat = os.stat(ruta)
                backups.append({"archivo": f,
                                "fecha": time.strftime("%d/%m/%Y %H:%M", time.localtime(stat.st_mtime)),
                                "tamano_mb": round(stat.st_size / (1024 * 1024), 1),
                                "mundo": "_".join(f.replace(".zip", "").split("_")[:-2])})
            backups.sort(key=lambda x: x["fecha"], reverse=True)
            return {"ok": True, "backups": backups}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def restaurar_mundo(self, archivo_backup):
        try:
            import minecraft_launcher_lib
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            backup_dir = os.path.join(os.path.dirname(os.path.abspath(self.ruta_config)), "paraguacraft_backups")
            ruta_zip = os.path.join(backup_dir, archivo_backup)
            if not os.path.exists(ruta_zip):
                return {"ok": False, "error": "Backup no encontrado."}
            nombre_mundo = "_".join(archivo_backup.replace(".zip", "").split("_")[:-2])
            instancias_dir = os.path.join(mine_dir, "instancias")
            dest_saves = None
            if nombre_mundo and os.path.isdir(instancias_dir):
                for inst in os.listdir(instancias_dir):
                    candidate = os.path.join(instancias_dir, inst, "saves", nombre_mundo)
                    if os.path.isdir(candidate):
                        dest_saves = os.path.join(instancias_dir, inst, "saves")
                        break
            if dest_saves is None:
                ult_ver = self.config_actual.get("ultima_version", "")
                ult_mot = self.config_actual.get("ultimo_motor", "")
                if ult_ver and ult_mot:
                    try:
                        from core import carpeta_instancia_paraguacraft
                        dest_saves = os.path.join(instancias_dir, carpeta_instancia_paraguacraft(ult_ver, ult_mot), "saves")
                    except Exception:
                        pass
            if dest_saves is None:
                dest_saves = os.path.join(mine_dir, "saves")
            os.makedirs(dest_saves, exist_ok=True)
            with zipfile.ZipFile(ruta_zip, "r") as zf:
                zf.extractall(dest_saves)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def eliminar_backup(self, archivo_backup):
        try:
            backup_dir = os.path.join(os.path.dirname(os.path.abspath(self.ruta_config)), "paraguacraft_backups")
            ruta = os.path.join(backup_dir, archivo_backup)
            if os.path.exists(ruta): os.remove(ruta)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── ESTADÍSTICAS DE SESIONES ─────────────────────────────────────────
    def get_estadisticas(self):
        try:
            if not os.path.exists(self._stats_file):
                return {"ok": True, "sesiones": [], "total_segundos": 0, "total_dias": 0}
            with open(self._stats_file, "r") as f:
                data = json.load(f)
            return {"ok": True, **data}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def guardar_sesion_estadistica(self, version, motor, segundos, instancia=""):
        try:
            data = {"sesiones": [], "total_segundos": 0, "total_dias": 0}
            if os.path.exists(self._stats_file):
                with open(self._stats_file, "r") as f:
                    data = json.load(f)
            sesion = {"fecha": datetime.date.today().isoformat(), "version": version,
                      "motor": motor, "segundos": int(segundos), "instancia": instancia}
            data.setdefault("sesiones", []).append(sesion)
            data["total_segundos"] = data.get("total_segundos", 0) + int(segundos)
            data["total_dias"] = len(set(s["fecha"] for s in data["sesiones"]))
            with open(self._stats_file, "w") as f:
                json.dump(data, f, indent=2)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_stats_instancia(self, carpeta):
        try:
            inst_key = os.path.basename(carpeta.rstrip("/\\"))
            data = {"sesiones": [], "total_segundos": 0, "total_dias": 0}
            if os.path.exists(self._stats_file):
                with open(self._stats_file, "r") as f:
                    all_data = json.load(f)
                seses = [s for s in all_data.get("sesiones", [])
                         if inst_key and (s.get("instancia", "") == inst_key
                         or inst_key in s.get("version", ""))]
                data["sesiones"] = seses
                data["total_segundos"] = sum(s.get("segundos", 0) for s in seses)
                data["total_dias"] = len(set(s.get("fecha", "") for s in seses))
            return {"ok": True, **data}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def _cached_get(self, url, cache_key, ttl=3600):
        """GET con fallback a caché local para modo offline."""
        cache_file = os.path.join(self._cache_dir, cache_key + ".json")
        try:
            r = requests.get(url, timeout=8)
            r.raise_for_status()
            data = r.json()
            with open(cache_file, "w", encoding="utf-8") as f:
                json.dump({"ts": time.time(), "data": data}, f)
            return data
        except Exception:
            if os.path.exists(cache_file):
                with open(cache_file, "r", encoding="utf-8") as f:
                    return json.load(f)["data"]
            raise


    def instalar_shader_preset(self, shader_id, version, motor):
        SHADERS = {
            'bsl':           {'slug': 'bsl-shaders',               'nombre': 'BSL Shaders'},
            'complementary': {'slug': 'complementary-reimagined',  'nombre': 'Complementary Reimagined'},
            'sildurs':       {'slug': 'sildurs-vibrant-shaders',   'nombre': "Sildur's Vibrant Shaders"},
            'makeup':        {'slug': 'makeup-ultra-fast-shaders', 'nombre': 'MakeUp Ultra Fast'},
        }
        if shader_id not in SHADERS:
            return {'ok': False, 'error': 'Shader desconocido.'}
        info = SHADERS[shader_id]
        try:
            import minecraft_launcher_lib as _mcl
            from core import carpeta_instancia_paraguacraft as _cia
            mc_dir   = _mcl.utils.get_minecraft_directory()
            shdr_dir = os.path.join(mc_dir, 'instancias', _cia(version, motor), 'shaderpacks')
            os.makedirs(shdr_dir, exist_ok=True)
            headers = {'User-Agent': 'ParaguacraftLauncher/2.0'}
            r = requests.get(f'https://api.modrinth.com/v2/project/{info["slug"]}/version', headers=headers, timeout=10)
            if r.status_code != 200:
                return {'ok': False, 'error': f'{info["nombre"]} no encontrado en Modrinth.'}
            versions = r.json()
            if not versions:
                return {'ok': False, 'error': f'{info["nombre"]} sin versiones disponibles.'}
            files = versions[0].get('files', [])
            if not files:
                return {'ok': False, 'error': 'Sin archivos de descarga.'}
            archivo = files[0]
            fn   = archivo['filename']
            dest = os.path.join(shdr_dir, fn)
            if os.path.exists(dest):
                return {'ok': True, 'nombre': info['nombre'], 'tamano_mb': round(os.path.getsize(dest)/1048576, 1)}
            dl = requests.get(archivo['url'], stream=True, headers=headers, timeout=60)
            if dl.status_code != 200:
                return {'ok': False, 'error': 'Error al descargar el shader.'}
            with open(dest, 'wb') as f:
                shutil.copyfileobj(dl.raw, f)
            return {'ok': True, 'nombre': info['nombre'], 'tamano_mb': round(os.path.getsize(dest)/1048576, 1)}
        except Exception as e:
            return {'ok': False, 'error': str(e)}

    def instalar_mod_b64(self, nombre, carpeta, b64, tipo="mods"):
        import base64, hashlib
        try:
            ext = os.path.splitext(nombre)[1].lower()
            if ext not in (".jar", ".zip"):
                return {"ok": False, "error": "Solo se permiten .jar y .zip"}
            dest_dir = os.path.join(carpeta, tipo)
            os.makedirs(dest_dir, exist_ok=True)
            data_bytes = base64.b64decode(b64)
            dest = os.path.join(dest_dir, nombre)
            with open(dest, "wb") as f:
                f.write(data_bytes)
            size_kb = round(len(data_bytes) / 1024, 1)
            return {"ok": True, "archivo": nombre, "size_kb": size_kb}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── PERFILES EXPORTABLES ─────────────────────────────────────────────
    def exportar_perfil(self, version, motor):
        try:
            perfil = {"version": version, "motor": motor,
                      "ram_asignada": self.config_actual.get("ram_asignada", 4),
                      "gc_type": self.config_actual.get("gc_type", "G1GC"),
                      "opt_minimos": self.config_actual.get("opt_minimos", False),
                      "papa_mode": self.config_actual.get("papa_mode", False),
                      "exportado_por": self.config_actual.get("usuario", "Jugador"),
                      "launcher": "Paraguacraft", "version_launcher": VERSION}
            try:
                mods_dir = os.path.join(self._dir_instancia(version, motor), "mods")
                if os.path.isdir(mods_dir):
                    perfil["mods"] = [f for f in os.listdir(mods_dir) if f.endswith(".jar")]
            except Exception:
                pass
            return {"ok": True, "json": json.dumps(perfil, indent=2, ensure_ascii=False)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def importar_perfil(self, json_str):
        try:
            perfil = json.loads(json_str)
            for k in ("ram_asignada", "gc_type", "opt_minimos", "papa_mode"):
                if k in perfil:
                    self.config_actual[k] = perfil[k]
            self._guardar()
            return {"ok": True, "version": perfil.get("version", "?"), "motor": perfil.get("motor", "?")}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── NOTICIAS MINECRAFT ───────────────────────────────────────────────
    def get_noticias_minecraft(self):
        try:
            r = requests.get("https://launchercontent.mojang.com/v2/news.json",
                             timeout=8, headers={"User-Agent": "Paraguacraft-Launcher"})
            entries = r.json().get("entries", [])[:6]
            noticias = []
            for e in entries:
                img = e.get("newsPageImage", {})
                noticias.append({"titulo": e.get("title", ""),
                                 "subtitulo": e.get("subTitle", ""),
                                 "fecha": e.get("date", ""),
                                 "imagen": img.get("url", "") if isinstance(img, dict) else "",
                                 "url": e.get("readMoreLink", ""),
                                 "categoria": e.get("category", "")})
            return {"ok": True, "noticias": noticias}
        except Exception as e:
            return {"ok": False, "error": str(e), "noticias": []}

    def abrir_url_navegador(self, url):
        try:
            import webbrowser
            webbrowser.open(url)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── NOTIFICACIONES WINDOWS ───────────────────────────────────────────
    def notificar_windows(self, titulo, mensaje):
        def _notif():
            try:
                from win10toast import ToastNotifier
                ToastNotifier().show_toast(titulo, mensaje, duration=5, threaded=False)
                return
            except ImportError:
                pass
            ps = (
                "Add-Type -AssemblyName System.Windows.Forms;"
                "$n=New-Object System.Windows.Forms.NotifyIcon;"
                "$n.Icon=[System.Drawing.SystemIcons]::Information;"
                "$n.Visible=$true;"
                f"$n.ShowBalloonTip(5000,'{titulo}','{mensaje}',"
                "[System.Windows.Forms.ToolTipIcon]::Info);"
                "Start-Sleep 6;$n.Dispose()"
            )
            subprocess.Popen(["powershell", "-WindowStyle", "Hidden", "-Command", ps],
                             creationflags=subprocess.CREATE_NO_WINDOW)
        threading.Thread(target=_notif, daemon=True).start()
        return {"ok": True}

    # ── MÚSICA DE MINECRAFT ──────────────────────────────────────────────
    def get_musica_mc_lista(self):
        try:
            import minecraft_launcher_lib
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            indexes_dir = os.path.join(mine_dir, "assets", "indexes")
            if not os.path.isdir(indexes_dir):
                return {"ok": False, "error": "Assets no descargados aún.", "tracks": []}
            indices = sorted(os.listdir(indexes_dir), reverse=True)
            if not indices:
                return {"ok": False, "error": "Sin índices de assets.", "tracks": []}
            with open(os.path.join(indexes_dir, indices[0]), "r") as f:
                objects = json.load(f).get("objects", {})
            tracks = []
            for key, val in objects.items():
                if "sounds/music" not in key: continue
                h = val["hash"]
                path = os.path.join(mine_dir, "assets", "objects", h[:2], h)
                nombre = key.replace("minecraft/sounds/music/", "").replace(".ogg", "")
                nombre = nombre.replace("/", " – ").replace("_", " ").title()
                tracks.append({"nombre": nombre, "path": path,
                               "disponible": os.path.exists(path), "key": key})
            tracks.sort(key=lambda x: x["nombre"])
            return {"ok": True, "tracks": tracks}
        except Exception as e:
            return {"ok": False, "error": str(e), "tracks": []}

    def tocar_musica_mc(self, path):
        try:
            self.detener_musica_mc()
            if not os.path.exists(path):
                return {"ok": False, "error": "Archivo no encontrado."}
            try:
                import pygame
                if not pygame.mixer.get_init():
                    pygame.mixer.init(frequency=44100)
                pygame.mixer.music.load(path)
                pygame.mixer.music.play()
                return {"ok": True, "motor": "pygame"}
            except ImportError:
                pass
            self._musica_proc = subprocess.Popen(
                ["powershell", "-WindowStyle", "Hidden", "-Command",
                 f'Add-Type -AssemblyName presentationcore; $m=New-Object System.Windows.Media.MediaPlayer; $m.Open([uri]"{path}"); $m.Play(); Start-Sleep 600'],
                creationflags=subprocess.CREATE_NO_WINDOW)
            return {"ok": True, "motor": "powershell"}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def detener_musica_mc(self):
        try:
            try:
                import pygame
                if pygame.mixer.get_init(): pygame.mixer.music.stop()
            except Exception:
                pass
            if self._musica_proc:
                try: self._musica_proc.terminate()
                except Exception: pass
                self._musica_proc = None
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def set_musica_mc_volumen(self, volumen):
        try:
            import pygame
            if pygame.mixer.get_init():
                pygame.mixer.music.set_volume(max(0.0, min(1.0, float(volumen))))
                return {"ok": True}
            return {"ok": False, "error": "No hay música reproduciéndose."}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_musica_mc_estado(self):
        try:
            import pygame
            if pygame.mixer.get_init() and pygame.mixer.music.get_busy():
                pos_ms = pygame.mixer.music.get_pos()
                vol = pygame.mixer.music.get_volume()
                return {"ok": True, "reproduciendo": True, "posicion_ms": pos_ms, "volumen": round(vol, 2)}
            return {"ok": True, "reproduciendo": False, "posicion_ms": 0, "volumen": 1.0}
        except Exception:
            return {"ok": True, "reproduciendo": False, "posicion_ms": 0, "volumen": 1.0}

    def buscar_youtube(self, query):
        try:
            if not _YOUTUBE_API_KEY:
                return {"ok": False, "error": "API key de YouTube no configurada.", "resultados": []}
            base_url = "https://www.googleapis.com/youtube/v3/search"
            def _parse_items(items, tipo_forzado=None):
                out = []
                for item in items:
                    kind = item["id"].get("kind", "")
                    vid_id = item["id"].get("videoId") or item["id"].get("playlistId")
                    snip = item.get("snippet", {})
                    es_playlist = "playlist" in kind
                    out.append({
                        "id": vid_id,
                        "tipo": tipo_forzado or ("playlist" if es_playlist else "video"),
                        "titulo": snip.get("title", ""),
                        "canal": snip.get("channelTitle", ""),
                        "thumb": (snip.get("thumbnails", {}).get("medium") or {}).get("url", ""),
                    })
                return out
            resultados = []
            r_vid = requests.get(base_url, params={
                "part": "snippet", "q": query, "type": "video",
                "videoEmbeddable": "true", "maxResults": 9, "key": _YOUTUBE_API_KEY
            }, timeout=8)
            data_vid = r_vid.json()
            if "error" not in data_vid:
                resultados.extend(_parse_items(data_vid.get("items", []), "video"))
            r_pl = requests.get(base_url, params={
                "part": "snippet", "q": query, "type": "playlist",
                "maxResults": 3, "key": _YOUTUBE_API_KEY
            }, timeout=8)
            data_pl = r_pl.json()
            if "error" not in data_pl:
                resultados.extend(_parse_items(data_pl.get("items", []), "playlist"))
            if not resultados and "error" in data_vid:
                return {"ok": False, "error": data_vid["error"].get("message", "Error API"), "resultados": []}
            return {"ok": True, "resultados": resultados}
        except Exception as e:
            return {"ok": False, "error": str(e), "resultados": []}

    # ── FONDO ANIMADO ────────────────────────────────────────────────────
    def check_post_update(self):
        import tempfile
        # Limpiar el .old que dejó la actualización anterior
        if getattr(sys, "frozen", False):
            try:
                old_exe = sys.executable + ".old"
                if os.path.exists(old_exe):
                    os.remove(old_exe)
            except Exception:
                pass
        marker = os.path.join(tempfile.gettempdir(), "paraguacraft_updated.flag")
        if os.path.isfile(marker):
            try: os.remove(marker)
            except Exception: pass
            return {"updated": True, "version": VERSION}
        return {"updated": False}

    def get_fondo_actual(self):
        fondo = self.config_actual.get("fondo_animado", "")
        if fondo and not os.path.isfile(fondo):
            self.config_actual["fondo_animado"] = ""
            self._guardar()
            fondo = ""
        return {"ok": True, "fondo": fondo}

    def elegir_fondo_animado(self):
        try:
            import base64
            result = webview.windows[0].create_file_dialog(
                webview.OPEN_DIALOG,
                allow_multiple=False,
                file_types=(
                    'Imágenes y videos (*.png;*.jpg;*.jpeg;*.gif;*.mp4;*.webm)',
                    'Todos los archivos (*.*)',
                )
            )
            if not result:
                return {"ok": True, "fondo": None}
            path = result[0]
            norm = path.replace("\\", "/")
            ext = path.rsplit('.', 1)[-1].lower()
            self.config_actual["fondo_animado"] = norm
            self._guardar()
            if ext in ('mp4', 'webm'):
                return {"ok": True, "fondo": norm, "tipo": "video"}
            with open(path, "rb") as f:
                raw = f.read()
            mime = {"png": "image/png", "jpg": "image/jpeg", "jpeg": "image/jpeg",
                    "gif": "image/gif"}.get(ext, "image/png")
            b64 = base64.b64encode(raw).decode()
            return {"ok": True, "fondo": norm, "tipo": "imagen",
                    "data_url": f"data:{mime};base64,{b64}"}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def eliminar_fondo_animado(self):
        self.config_actual["fondo_animado"] = ""
        self._guardar()
        return {"ok": True}

    # ── MODO COMPACTO ────────────────────────────────────────────────────
    def set_modo_compacto(self, activar):
        try:
            ventana = webview.windows[0] if webview.windows else None
            if ventana:
                if activar:
                    ventana.resize(340, 110)
                else:
                    ventana.resize(1150, 700)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── SPOTIFY ──────────────────────────────────────────────────────────
    def get_spotify_auth_url(self, client_id):
        import urllib.parse
        scopes = "user-read-playback-state user-modify-playback-state user-read-currently-playing"
        params = {"client_id": client_id, "response_type": "code",
                  "redirect_uri": "http://localhost:8888/callback", "scope": scopes}
        url = "https://accounts.spotify.com/authorize?" + urllib.parse.urlencode(params)
        webbrowser.open(url)
        return {"ok": True, "url": url}

    def conectar_spotify(self, client_id, client_secret, code):
        return self._spotify_exchange(client_id, client_secret, code, "http://localhost:8888/callback")

    def _spotify_exchange(self, client_id, client_secret, code, redirect_uri):
        import base64
        try:
            creds = base64.b64encode(f"{client_id}:{client_secret}".encode()).decode()
            r = requests.post("https://accounts.spotify.com/api/token",
                              headers={"Authorization": f"Basic {creds}",
                                       "Content-Type": "application/x-www-form-urlencoded"},
                              data={"grant_type": "authorization_code", "code": code,
                                    "redirect_uri": redirect_uri}, timeout=10)
            data = r.json()
            if "access_token" in data:
                self._spotify_token = data["access_token"]
                self._spotify_refresh = data.get("refresh_token")
                self.config_actual["spotify_client_id"] = client_id
                self.config_actual["spotify_client_secret"] = client_secret
                if self._spotify_refresh:
                    self.config_actual["spotify_refresh_token"] = self._spotify_refresh
                self._guardar()
                return {"ok": True}
            return {"ok": False, "error": data.get("error_description", "Error de autenticación")}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def _spotify_do_refresh(self):
        if not self._spotify_refresh:
            return False
        try:
            import base64
            cid = self.config_actual.get("spotify_client_id", "")
            sec = self.config_actual.get("spotify_client_secret", "")
            if not cid or not sec:
                return False
            creds = base64.b64encode(f"{cid}:{sec}".encode()).decode()
            r = requests.post("https://accounts.spotify.com/api/token",
                              headers={"Authorization": f"Basic {creds}",
                                       "Content-Type": "application/x-www-form-urlencoded"},
                              data={"grant_type": "refresh_token",
                                    "refresh_token": self._spotify_refresh}, timeout=10)
            data = r.json()
            if "access_token" in data:
                self._spotify_token = data["access_token"]
                if "refresh_token" in data:
                    self._spotify_refresh = data["refresh_token"]
                    self.config_actual["spotify_refresh_token"] = self._spotify_refresh
                    self._guardar()
                return True
            return False
        except Exception:
            return False

    def spotify_try_autoconnect(self):
        if self._spotify_token:
            return {"ok": True}
        saved = self.config_actual.get("spotify_refresh_token", "")
        if not saved:
            return {"ok": False, "error": "Sin sesión guardada."}
        self._spotify_refresh = saved
        if self._spotify_do_refresh():
            return {"ok": True}
        self.config_actual.pop("spotify_refresh_token", None)
        self._guardar()
        return {"ok": False, "error": "Sesión expirada, autorizá nuevamente."}

    def _spotify_headers(self):
        return {"Authorization": f"Bearer {self._spotify_token}"}

    def get_spotify_nowplaying(self):
        if not self._spotify_token:
            return {"ok": False, "error": "No conectado.", "reproduciendo": False}
        try:
            r = requests.get("https://api.spotify.com/v1/me/player/currently-playing",
                             headers=self._spotify_headers(), timeout=5)
            if r.status_code == 204:
                return {"ok": True, "reproduciendo": False}
            if r.status_code == 401:
                if self._spotify_do_refresh():
                    r = requests.get("https://api.spotify.com/v1/me/player/currently-playing",
                                     headers=self._spotify_headers(), timeout=5)
                    if r.status_code not in (200, 204):
                        self._spotify_token = None
                        return {"ok": False, "error": "Sesión expirada, reconectá.", "reproduciendo": False}
                    if r.status_code == 204:
                        return {"ok": True, "reproduciendo": False}
                else:
                    self._spotify_token = None
                    return {"ok": False, "error": "Sesión expirada, reconectá.", "reproduciendo": False}
            data = r.json()
            item = data.get("item", {}) or {}
            return {"ok": True, "reproduciendo": data.get("is_playing", False),
                    "titulo": item.get("name", ""),
                    "artista": ", ".join(a["name"] for a in item.get("artists", [])),
                    "album": item.get("album", {}).get("name", ""),
                    "imagen": (item.get("album", {}).get("images") or [{}])[0].get("url", ""),
                    "progreso_ms": data.get("progress_ms", 0),
                    "duracion_ms": item.get("duration_ms", 1),
                    "shuffle": data.get("shuffle_state", False),
                    "repeat": data.get("repeat_state", "off")}
        except Exception as e:
            return {"ok": False, "error": str(e), "reproduciendo": False}

    def spotify_control(self, accion):
        if not self._spotify_token:
            return {"ok": False, "error": "No conectado."}
        try:
            base = "https://api.spotify.com/v1/me/player"
            urls = {"play": (requests.put, f"{base}/play"), "pause": (requests.put, f"{base}/pause"),
                    "next": (requests.post, f"{base}/next"), "prev": (requests.post, f"{base}/previous")}
            if accion not in urls:
                return {"ok": False, "error": "Acción desconocida."}
            fn, url = urls[accion]
            r = fn(url, headers=self._spotify_headers(), timeout=5)
            if r.status_code == 401 and self._spotify_do_refresh():
                r = fn(url, headers=self._spotify_headers(), timeout=5)
            return {"ok": r.status_code in (200, 204)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def spotify_shuffle(self, state):
        if not self._spotify_token:
            return {"ok": False, "error": "No conectado."}
        try:
            r = requests.put(f"https://api.spotify.com/v1/me/player/shuffle?state={'true' if state else 'false'}",
                             headers=self._spotify_headers(), timeout=5)
            if r.status_code == 401 and self._spotify_do_refresh():
                r = requests.put(f"https://api.spotify.com/v1/me/player/shuffle?state={'true' if state else 'false'}",
                                 headers=self._spotify_headers(), timeout=5)
            return {"ok": r.status_code in (200, 204)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def spotify_repeat(self, state):
        if not self._spotify_token:
            return {"ok": False, "error": "No conectado."}
        try:
            r = requests.put(f"https://api.spotify.com/v1/me/player/repeat?state={state}",
                             headers=self._spotify_headers(), timeout=5)
            if r.status_code == 401 and self._spotify_do_refresh():
                r = requests.put(f"https://api.spotify.com/v1/me/player/repeat?state={state}",
                                 headers=self._spotify_headers(), timeout=5)
            return {"ok": r.status_code in (200, 204)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── CURSEFORGE STORE ─────────────────────────────────────────────────
    _CF_API_KEY = _CF_API_KEY
    _GEMINI_API_KEY = _GEMINI_API_KEY
    _SP_CLIENT_ID = _SP_CLIENT_ID
    _SP_CLIENT_SECRET = _SP_CLIENT_SECRET

    def buscar_curseforge(self, query, tipo="mods", mc_version=""):
        try:
            game_id = 432  # Minecraft
            class_map = {"mods": 6, "resourcepacks": 12, "shaders": 6552, "modpacks": 4471}
            class_id = class_map.get(tipo, 6)
            params = {"gameId": game_id, "classId": class_id, "searchFilter": query,
                      "sortField": 2, "sortOrder": "desc", "pageSize": 16}
            if mc_version:
                params["gameVersion"] = mc_version
            r = requests.get("https://api.curseforge.com/v1/mods/search",
                             headers={"x-api-key": self._CF_API_KEY, "Accept": "application/json"},
                             params=params, timeout=10)
            if r.status_code != 200:
                return {"error": f"HTTP {r.status_code} — API CurseForge no disponible", "data": []}
            data = r.json().get("data", [])
            return {"data": data}
        except Exception as e:
            return {"error": str(e), "data": []}

    def get_versiones_curseforge(self, project_id, mc_version=""):
        try:
            r = requests.get(f"https://api.curseforge.com/v1/mods/{project_id}/files",
                             headers={"x-api-key": self._CF_API_KEY, "Accept": "application/json"},
                             params={"gameVersion": mc_version, "pageSize": 10}, timeout=10)
            if r.status_code != 200:
                return {"ok": False, "error": f"HTTP {r.status_code}", "versions": []}
            data = r.json().get("data", [])
            vers = []
            for f in data:
                gvs = f.get("gameVersions", [])
                vers.append({
                    "id": str(f.get("id", "")),
                    "name": f.get("displayName", ""),
                    "game_versions": gvs,
                    "loaders": [v for v in gvs if v.lower() in ("forge","fabric","quilt","neoforge")],
                    "size_mb": round(f.get("fileLength", 0) / 1048576, 2),
                    "date_published": (f.get("fileDate") or "")[:10],
                    "download_url": f.get("downloadUrl", "")
                })
            return {"ok": True, "versions": vers}
        except Exception as e:
            return {"ok": False, "error": str(e), "versions": []}

    def instalar_mod_curseforge(self, project_id, file_id, tipo, version, motor):
        try:
            r = requests.get(f"https://api.curseforge.com/v1/mods/{project_id}/files/{file_id}/download-url",
                             headers={"x-api-key": self._CF_API_KEY}, timeout=10)
            if r.status_code != 200:
                return {"ok": False, "error": "No se pudo obtener URL de descarga"}
            url = r.json().get("data", "")
            if not url:
                return {"ok": False, "error": "URL vacía"}
            import minecraft_launcher_lib as _mcl
            from core import carpeta_instancia_paraguacraft
            carpeta_tipo = {"mod": "mods", "resourcepack": "resourcepacks",
                            "shader": "shaderpacks", "modpack": "mods"}.get(tipo, "mods")
            dest = os.path.join(_mcl.utils.get_minecraft_directory(), "instancias",
                                carpeta_instancia_paraguacraft(version, motor), carpeta_tipo)
            os.makedirs(dest, exist_ok=True)
            nombre = url.split("/")[-1].split("?")[0] or f"cf_{file_id}.jar"
            ruta = os.path.join(dest, nombre)
            with requests.get(url, stream=True, timeout=60) as dl:
                with open(ruta, "wb") as f:
                    for chunk in dl.iter_content(65536):
                        f.write(chunk)
            return {"ok": True, "nombre": nombre}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── COMPARTIR INSTANCIA POR CÓDIGO ────────────────────────────────────
    def generar_codigo_instancia(self, version, motor):
        try:
            from core import carpeta_instancia_paraguacraft
            inst_dir = carpeta_instancia_paraguacraft(version, motor)
            mods_dir = os.path.join(inst_dir, "mods")
            mods = []
            if os.path.isdir(mods_dir):
                for jar in os.listdir(mods_dir):
                    if jar.endswith((".jar", ".jar.disabled")):
                        mods.append(jar)
            cfg = self.config_actual
            manifiesto = {
                "paraguacraft": True,
                "version": version,
                "motor": motor,
                "ram": cfg.get("ram_asignada", 4),
                "gc": cfg.get("gc_type", "G1GC"),
                "mods": mods,
                "creado": datetime.datetime.now().strftime("%Y-%m-%d %H:%M"),
                "autor": cfg.get("usuario", "Jugador")
            }
            json_str = json.dumps(manifiesto, ensure_ascii=False, indent=2)
            r = requests.post("https://dpaste.org/api/",
                              data={"content": json_str, "syntax": "json", "expiry_days": 30},
                              timeout=10)
            if r.status_code == 200:
                url = r.text.strip().strip('"')
                codigo = url.rstrip("/").split("/")[-1]
                return {"ok": True, "codigo": codigo, "url": url,
                        "mods": len(mods), "version": version, "motor": motor}
            return {"ok": False, "error": f"dpaste error {r.status_code}"}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def importar_instancia_por_codigo(self, codigo):
        try:
            codigo = codigo.strip().upper()
            r = requests.get(f"https://dpaste.org/{codigo}/raw", timeout=10)
            if r.status_code != 200:
                return {"ok": False, "error": "Código inválido o expirado."}
            manifiesto = json.loads(r.text)
            if not manifiesto.get("paraguacraft"):
                return {"ok": False, "error": "El código no es un perfil Paraguacraft."}
            return {"ok": True, "manifiesto": manifiesto}
        except json.JSONDecodeError:
            return {"ok": False, "error": "Contenido inválido."}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def descargar_mods_manifiesto(self, manifiesto):
        try:
            version = manifiesto.get("version", "")
            motor = manifiesto.get("motor", "")
            mods_lista = manifiesto.get("mods", [])
            if not version or not motor:
                return {"ok": False, "error": "Manifiesto sin versión/motor."}
            self.config_actual["ram_asignada"] = manifiesto.get("ram", self.config_actual.get("ram_asignada", 4))
            self.config_actual["gc_type"] = manifiesto.get("gc", self.config_actual.get("gc_type", "G1GC"))
            self._guardar()
            aplicados = 0
            errores = []
            for mod_nombre in mods_lista:
                slug = os.path.splitext(mod_nombre)[0].rstrip("-disabled").lower()
                try:
                    sr = requests.get(f"https://api.modrinth.com/v2/search?query={requests.utils.quote(slug)}&limit=1",
                                      timeout=8).json()
                    if sr.get("hits"):
                        pid = sr["hits"][0]["project_id"]
                        vr = requests.get(f"https://api.modrinth.com/v2/project/{pid}/version",
                                          params={"game_versions": f'["{version}"]'}, timeout=8).json()
                        if vr:
                            self.instalar_mod_desde_modrinth(pid, vr[0]["id"], "mod", version, motor)
                            aplicados += 1
                except Exception:
                    errores.append(slug)
            return {"ok": True, "aplicados": aplicados, "errores": errores, "total": len(mods_lista)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── GAME MODE / OS OPTIMIZER ──────────────────────────────────────────
    _game_mode_activo = False

    def activar_game_mode(self):
        try:
            procesos_suspendidos = []
            blacklist = {
                "SearchIndexer.exe", "OneDrive.exe", "Teams.exe", "MicrosoftTeams.exe",
                "Widgets.exe", "SearchApp.exe", "YourPhone.exe", "PhoneExperienceHost.exe",
                "GameBarFTServer.exe", "XboxPCApp.exe", "SkypeApp.exe", "slack.exe",
                "chrome.exe", "msedge.exe", "firefox.exe",
                "AdobeUpdateService.exe", "CCXProcess.exe", "NortonSecurity.exe",
                "MsMpEng.exe", "SgrmBroker.exe", "SpeechRuntime.exe", "Cortana.exe",
                "WMIProviderHost.exe", "SysMain.exe", "SearchUI.exe"
            }
            for proc in psutil.process_iter(["pid", "name", "status"]):
                try:
                    name = proc.info["name"]
                    if name in blacklist and proc.info["status"] != psutil.STATUS_ZOMBIE:
                        proc.suspend()
                        procesos_suspendidos.append(name)
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    pass
            # Limpiar standby RAM (Windows)
            if platform.system() == "Windows":
                try:
                    subprocess.run(
                        ["powershell", "-Command",
                         "[System.GC]::Collect(); [System.GC]::WaitForPendingFinalizers()"],
                        capture_output=True, timeout=5)
                    import ctypes
                    ctypes.windll.psapi.EmptyWorkingSet(ctypes.windll.kernel32.GetCurrentProcess())
                except Exception:
                    pass
            # Prioridad alta a javaw.exe si ya corre
            javaw_count = 0
            for proc in psutil.process_iter(["name", "pid"]):
                try:
                    if proc.info["name"].lower() in ("javaw.exe", "java.exe"):
                        p = psutil.Process(proc.info["pid"])
                        p.nice(psutil.HIGH_PRIORITY_CLASS if platform.system() == "Windows" else -15)
                        javaw_count += 1
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    pass
            self._game_mode_activo = True
            self._procesos_suspendidos = procesos_suspendidos
            return {"ok": True, "suspendidos": len(procesos_suspendidos), "javaw": javaw_count,
                    "nombres": procesos_suspendidos[:10]}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def desactivar_game_mode(self):
        try:
            reanudados = 0
            suspendidos = getattr(self, "_procesos_suspendidos", [])
            for proc in psutil.process_iter(["name", "status"]):
                try:
                    if proc.info["name"] in suspendidos and proc.info["status"] == psutil.STATUS_STOPPED:
                        proc.resume()
                        reanudados += 1
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    pass
            # Restaurar prioridad javaw
            for proc in psutil.process_iter(["name", "pid"]):
                try:
                    if proc.info["name"].lower() in ("javaw.exe", "java.exe"):
                        p = psutil.Process(proc.info["pid"])
                        p.nice(psutil.NORMAL_PRIORITY_CLASS if platform.system() == "Windows" else 0)
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    pass
            self._game_mode_activo = False
            return {"ok": True, "reanudados": reanudados}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_game_mode_status(self):
        javaw_corriendo = any(
            p.info["name"].lower() in ("javaw.exe", "java.exe")
            for p in psutil.process_iter(["name"]) if p.info.get("name")
        )
        return {"activo": self._game_mode_activo, "javaw_corriendo": javaw_corriendo}

    def set_javaw_prioridad(self, nivel="alta"):
        try:
            prio_map = {
                "alta": (psutil.HIGH_PRIORITY_CLASS, -15),
                "realtime": (psutil.REALTIME_PRIORITY_CLASS, -20),
                "normal": (psutil.NORMAL_PRIORITY_CLASS, 0),
                "baja": (psutil.BELOW_NORMAL_PRIORITY_CLASS, 5)
            }
            prio_win, prio_unix = prio_map.get(nivel, prio_map["alta"])
            cambiados = 0
            for proc in psutil.process_iter(["name", "pid"]):
                try:
                    if proc.info["name"].lower() in ("javaw.exe", "java.exe"):
                        p = psutil.Process(proc.info["pid"])
                        p.nice(prio_win if platform.system() == "Windows" else prio_unix)
                        cambiados += 1
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    pass
            return {"ok": True, "cambiados": cambiados, "nivel": nivel}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── PERFILES DE HARDWARE ──────────────────────────────────────────────
    ESTILOS_JUEGO = {
        "pvp": {
            "nombre": "PvP", "icono": "\u2694\ufe0f",
            "descripcion": "M\u00e1ximo FPS, sin distracciones. Reducir chunks y visuales.",
            "mods_extra": {
                "baja":  [("minihud","MiniHUD"),("no-chat-reports","NoChatReports")],
                "media": [("minihud","MiniHUD"),("no-chat-reports","NoChatReports"),("appleskin","AppleSkin")],
                "alta":  [("minihud","MiniHUD"),("no-chat-reports","NoChatReports"),("appleskin","AppleSkin")],
            },
            "config_override": {
                "baja":  {"chunks": 6,  "gc": "ZGC"},
                "media": {"chunks": 8,  "gc": "ZGC"},
                "alta":  {"chunks": 10, "gc": "ZGC"},
            },
        },
        "survival": {
            "nombre": "Survival", "icono": "\U0001f332",
            "descripcion": "Mapas, crafting, exploraci\u00f3n c\u00f3moda.",
            "mods_extra": {
                "baja":  [("xaeros-minimap","Xaero's Minimap"),("jei","JEI")],
                "media": [("xaeros-minimap","Xaero's Minimap"),("xaeros-world-map","Xaero's World Map"),("jei","JEI"),("appleskin","AppleSkin")],
                "alta":  [("xaeros-minimap","Xaero's Minimap"),("xaeros-world-map","Xaero's World Map"),("jei","JEI"),("appleskin","AppleSkin"),("waystones","Waystones")],
            },
            "config_override": {},
        },
        "competitivo": {
            "nombre": "Competitivo", "icono": "\U0001f3c6",
            "descripcion": "Speedrun / ranked. M\u00e1xima precisi\u00f3n y estabilidad.",
            "mods_extra": {
                "baja":  [("minihud","MiniHUD")],
                "media": [("minihud","MiniHUD"),("no-chat-reports","NoChatReports")],
                "alta":  [("minihud","MiniHUD"),("no-chat-reports","NoChatReports"),("replaymod","ReplayMod")],
            },
            "config_override": {
                "baja":  {"chunks": 6, "gc": "ZGC"},
                "media": {"chunks": 8, "gc": "ZGC"},
            },
        },
        "streaming": {
            "nombre": "Streaming", "icono": "\U0001f4fa",
            "descripcion": "Calidad visual alta, replay y screenshots.",
            "mods_extra": {
                "baja":  [],
                "media": [("replaymod","ReplayMod")],
                "alta":  [("replaymod","ReplayMod"),("appleskin","AppleSkin")],
            },
            "config_override": {
                "media": {"chunks": 12},
                "alta":  {"chunks": 16, "ram_fraction": 0.65},
            },
        },
        "modded": {
            "nombre": "Modded", "icono": "\U0001f9e9",
            "descripcion": "M\u00e1xima compatibilidad para packs de mods.",
            "mods_extra": {
                "baja":  [("memoryleakfix","MemoryLeakFix")],
                "media": [("memoryleakfix","MemoryLeakFix"),("jei","JEI")],
                "alta":  [("memoryleakfix","MemoryLeakFix"),("jei","JEI"),("appleskin","AppleSkin")],
            },
            "config_override": {
                "baja":  {"ram_fraction": 0.55},
                "media": {"ram_fraction": 0.60},
                "alta":  {"ram_fraction": 0.70, "chunks": 12},
            },
        },
    }

    MODS_LOADER_MAP = {
        "forge": {
            "sodium":        ("embeddium",    "Embeddium"),
            "lithium":       ("canary",       "Canary"),
            "iris":          ("oculus",       "Oculus"),
            "indium":        None,            # Fabric-only Sodium addon
            "entityculling": None,            # Port uses SpongeMixin, crashes Forge LaunchWrapper
            "starlight":     None,            # Discontinued, no Forge build exists
            "itemmodel-fix": None,            # Fabric only
            "minihud":       None,            # Fabric only (masa mod)
        },
        "neoforge": {
            "sodium":        ("embeddium",    "Embeddium"),
            "lithium":       ("canary",       "Canary"),
            "iris":          ("oculus",       "Oculus"),
            "indium":        None,
            "entityculling": None,
            "starlight":     None,
            "itemmodel-fix": None,
            "minihud":       None,
        },
    }

    PERFILES_HARDWARE = {
        "baja": {
            "nombre": "Gama Baja",
            "descripcion": "Máximo rendimiento, mínimos visuales",
            "ram_max_gb": 3,
            "chunks": 8,
            "mods_modrinth": [
                ("sodium", "Sodium"),
                ("lithium", "Lithium"),
                ("ferrite-core", "FerriteCore"),
                ("starlight", "Starlight"),
                ("entityculling", "EntityCulling"),
                ("memoryleakfix", "MemoryLeakFix"),
                ("no-chat-reports", "No Chat Reports"),
            ],
            "gc": "ZGC",
            "ram_fraction": 0.45
        },
        "media": {
            "nombre": "Gama Media",
            "descripcion": "Balance visual / rendimiento + shaders livianos",
            "ram_max_gb": 5,
            "chunks": 10,
            "mods_modrinth": [
                ("sodium", "Sodium"),
                ("lithium", "Lithium"),
                ("iris", "Iris Shaders"),
                ("ferrite-core", "FerriteCore"),
                ("entityculling", "EntityCulling"),
                ("memoryleakfix", "MemoryLeakFix"),
                ("indium", "Indium"),
            ],
            "shaders_modrinth": [("makeup-ultra-fast-shaders", "Makeup Ultra Fast Shaders")],
            "gc": "G1GC",
            "ram_fraction": 0.5
        },
        "alta": {
            "nombre": "Gama Alta",
            "descripcion": "Calidad visual alta + optimizaciones",
            "ram_max_gb": 8,
            "chunks": 14,
            "mods_modrinth": [
                ("sodium", "Sodium"),
                ("lithium", "Lithium"),
                ("iris", "Iris Shaders"),
                ("indium", "Indium"),
                ("ferrite-core", "FerriteCore"),
                ("entityculling", "EntityCulling"),
                ("memoryleakfix", "MemoryLeakFix"),
                ("replaymod", "ReplayMod"),
                ("itemmodel-fix", "ItemModel Fix"),
            ],
            "shaders_modrinth": [("complementary-reimagined", "Complementary Reimagined")],
            "gc": "G1GC",
            "ram_fraction": 0.6
        }
    }

    def detectar_perfil_hardware_sugerido(self):
        try:
            ram_gb = round(psutil.virtual_memory().total / 1073741824, 1)
            cpu_cores = psutil.cpu_count(logical=False) or 2
            if ram_gb <= 8 or cpu_cores <= 4:
                sugerido = "baja"
            elif ram_gb <= 16 or cpu_cores <= 8:
                sugerido = "media"
            else:
                sugerido = "alta"
            perfil = self.PERFILES_HARDWARE[sugerido]
            ram_rec = min(int(ram_gb * perfil["ram_fraction"]), perfil["ram_max_gb"])
            return {"ok": True, "ram_gb": ram_gb, "cpu_cores": cpu_cores,
                    "perfil_id": sugerido, "sugerido": sugerido, "nombre": perfil["nombre"],
                    "descripcion": perfil["descripcion"],
                    "ram_rec": ram_rec, "chunks": perfil["chunks"],
                    "mods": [m[1] for m in perfil["mods_modrinth"]],
                    "shaders": [s[1] for s in perfil.get("shaders_modrinth", [])]}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def aplicar_perfil_hardware(self, perfil_id, version, motor, estilo="pvp"):
        try:
            if perfil_id not in self.PERFILES_HARDWARE:
                return {"ok": False, "error": "Perfil desconocido."}
            perfil = dict(self.PERFILES_HARDWARE[perfil_id])
            # Merge estilo overrides
            estilo_data = self.ESTILOS_JUEGO.get(estilo, self.ESTILOS_JUEGO.get("pvp", {}))
            cfg_ov = estilo_data.get("config_override", {}).get(perfil_id, {})
            if "chunks" in cfg_ov:
                perfil["chunks"] = cfg_ov["chunks"]
            if "gc" in cfg_ov:
                perfil["gc"] = cfg_ov["gc"]
            if "ram_fraction" in cfg_ov:
                perfil["ram_fraction"] = cfg_ov["ram_fraction"]
            mods_extra = estilo_data.get("mods_extra", {}).get(perfil_id, [])
            # Merge without duplicates
            base_slugs = {s for s, _ in perfil["mods_modrinth"]}
            mods_merged = list(perfil["mods_modrinth"]) + [(s, n) for s, n in mods_extra if s not in base_slugs]
            # Apply loader-specific substitutions (e.g. Forge: Embeddium instead of Sodium)
            loader_key = motor.lower().split()[0]
            loader_subs = self.MODS_LOADER_MAP.get(loader_key, {})
            mods_final = []
            for slug, nombre in mods_merged:
                if slug in loader_subs:
                    sub = loader_subs[slug]
                    if sub is None:
                        continue  # skip Fabric-only mod (e.g. Indium on Forge)
                    slug, nombre = sub
                mods_final.append((slug, nombre))
            ram_gb = round(psutil.virtual_memory().total / 1073741824, 1)
            ram_rec = min(int(ram_gb * perfil["ram_fraction"]), perfil["ram_max_gb"])
            self.config_actual["ram_asignada"] = ram_rec
            self.config_actual["gc_type"] = perfil["gc"]
            self._guardar()
            instalados, errores = [], []
            for slug, nombre in mods_final:
                try:
                    sr = requests.get(f"https://api.modrinth.com/v2/search?query={requests.utils.quote(slug)}&limit=1",
                                      timeout=8).json()
                    if sr.get("hits"):
                        pid = sr["hits"][0]["project_id"]
                        params = {"game_versions": f'["{version}"]', "loaders": f'["{motor.lower()}"]'}
                        vr = requests.get(f"https://api.modrinth.com/v2/project/{pid}/version",
                                          params=params, timeout=8).json()
                        if not vr:
                            params.pop("loaders")
                            vr = requests.get(f"https://api.modrinth.com/v2/project/{pid}/version",
                                              params=params, timeout=8).json()
                        if vr:
                            res = self.instalar_mod_desde_modrinth(pid, vr[0]["id"], "mod", version, motor)
                            if res.get("ok"):
                                instalados.append(nombre)
                            else:
                                errores.append(nombre)
                        else:
                            errores.append(nombre)
                    else:
                        errores.append(nombre)
                except Exception:
                    errores.append(nombre)
            for slug, nombre in perfil.get("shaders_modrinth", []):
                try:
                    sr = requests.get(f"https://api.modrinth.com/v2/search?query={requests.utils.quote(slug)}&limit=1",
                                      timeout=8).json()
                    if sr.get("hits"):
                        pid = sr["hits"][0]["project_id"]
                        vr = requests.get(f"https://api.modrinth.com/v2/project/{pid}/version",
                                          params={"game_versions": f'["{version}"]'}, timeout=8).json()
                        if vr:
                            res = self.instalar_mod_desde_modrinth(pid, vr[0]["id"], "shader", version, motor)
                            if res.get("ok"):
                                instalados.append(nombre)
                            else:
                                errores.append(nombre)
                except Exception:
                    errores.append(nombre)
            self._guardar()
            return {"ok": True, "ya_aplicado": False, "mods_instalados": instalados,
                    "errores": errores, "ram_asignada": ram_rec, "gc": perfil["gc"],
                    "chunks": perfil["chunks"]}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── ANALIZADOR CRASH LOG ──────────────────────────────────────────────
    _CRASH_PATTERNS = [
        (r"net\.fabricmc\.loader.*incompatible", "Incompatibilidad de mods Fabric detectada."),
        (r"optifine.*sodium|sodium.*optifine", "OptiFine y Sodium son incompatibles. Desactivá OptiFine o usá Iris+Sodium."),
        (r"java\.lang\.OutOfMemoryError", "Sin memoria RAM suficiente. Aumentá la RAM asignada al juego."),
        (r"java\.lang\.OutOfMemoryError.*PermGen", "PermGen agotado. Actualizá a Java 8+ o aumentá PermGen."),
        (r"StackOverflowError", "Stack overflow. Puede ser un mod buggy o recursión infinita."),
        (r"UnsatisfiedLinkError.*lwjgl", "Error de LWJGL nativo. Reinstalá el juego o actualizá drivers gráficos."),
        (r"GLFW error.*65543|GLFWException", "Error de ventana GLFW. Actualizá drivers de GPU."),
        (r"class.*has been loaded from.*different.*loader", "Conflicto de classloader entre mods. Revisá versiones duplicadas."),
        (r"Duplicate mod.*same mod ID", "Mods duplicados detectados. Eliminá uno de los JARs conflictivos."),
        (r"mixin.*failed|MixinException", "Un mod con Mixin falló. Actualizá los mods a versiones compatibles."),
        (r"forge.*requires.*minecraft.*but", "Versión de Forge incompatible con esta versión de Minecraft."),
        (r"fabric.*requires.*minecraft.*but", "Versión de Fabric incompatible con esta versión de Minecraft."),
        (r"id_of_mod.*already registered|Registry.*already.*registered", "Dos mods registran el mismo item/bloque ID. Incompatibilidad de contenido."),
        (r"Driver crashed.*shutting down|GL_OUT_OF_MEMORY", "La GPU se quedó sin VRAM. Bajá la calidad gráfica o los chunks renderizados."),
        (r"Unable to make.*accessible|InaccessibleObjectException", "Error de reflexión Java. Actualizá Java 17+ o verificá los flags JVM."),
        (r"CrashReport.*caused by.*incompatible.*receiving channel", "Incompatibilidad de protocolo de red con el servidor."),
        (r"Missing required.*dependency.*mod", "Falta una dependencia de mod. Revisá el README de los mods instalados."),
    ]

    def analizar_crash_log(self, version="", motor=""):
        try:
            import minecraft_launcher_lib
            mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            _v = (version or "").strip()
            if _v and _v != "latest" and motor:
                from core import carpeta_instancia_paraguacraft
                inst_path = os.path.join(mc_dir, "instancias", carpeta_instancia_paraguacraft(_v, motor))
                logs_dir = os.path.join(inst_path, "logs")
                crash_dir = os.path.join(inst_path, "crash-reports")
            else:
                logs_dir = os.path.join(mc_dir, "logs")
                crash_dir = os.path.join(mc_dir, "crash-reports")
            contenido = ""
            fuente = ""
            latest = os.path.join(logs_dir, "latest.log")
            if os.path.isfile(latest):
                with open(latest, "r", encoding="utf-8", errors="replace") as f:
                    contenido = f.read()[-60000:]
                fuente = "latest.log"
            if os.path.isdir(crash_dir):
                crashfiles = sorted(
                    [os.path.join(crash_dir, x) for x in os.listdir(crash_dir)
                     if x.endswith(".txt")],
                    key=os.path.getmtime, reverse=True
                )
                if crashfiles:
                    with open(crashfiles[0], "r", encoding="utf-8", errors="replace") as f:
                        contenido += "\n" + f.read()[-30000:]
                    fuente += (" + " if fuente else "") + os.path.basename(crashfiles[0])
            if not contenido:
                return {"ok": False, "error": "No se encontraron logs de Minecraft."}
            diagnosticos = []
            lower = contenido.lower()
            for pat, msg in self._CRASH_PATTERNS:
                if re.search(pat, contenido, re.IGNORECASE):
                    diagnosticos.append(msg)
            mod_match = re.findall(r'Loaded mods:\s*(.*?)(?:\n\n|\Z)', contenido, re.DOTALL)
            mods_cargados = []
            if mod_match:
                mods_cargados = [m.strip() for m in mod_match[0].split(",") if m.strip()][:20]
            # Map diagnostics strings → structured issues for frontend
            _sev_map = [
                ("OutOfMemory", "error"), ("StackOverflow", "error"), ("Driver crashed", "error"),
                ("GL_OUT_OF_MEMORY", "error"), ("UnsatisfiedLink", "error"), ("GLFW", "error"),
            ]
            issues = []
            for msg in (diagnosticos if diagnosticos else ["No se detectaron patrones conocidos. Revisá el log manualmente."]):
                sev = "advertencia"
                for kw, s in _sev_map:
                    if kw.lower() in msg.lower():
                        sev = s; break
                if any(w in msg.lower() for w in ["incompatible", "conflicto", "falta", "duplicad", "mixin", "driver", "memory", "overflow", "glfw", "vram"]):
                    sev = "error"
                titulo = msg.split(".")[0].strip()[:60]
                desc = msg
                sug = ""
                if "RAM" in msg or "memoria" in msg.lower():
                    sug = "Aumentá la RAM en Configuración → RAM asignada."
                elif "Sodium" in msg and "OptiFine" in msg:
                    sug = "Desactivá OptiFine y usá Iris en su lugar."
                elif "duplicad" in msg.lower():
                    sug = "Borrá el JAR duplicado de la carpeta mods."
                elif "driver" in msg.lower() or "VRAM" in msg:
                    sug = "Actualizá los drivers de tu GPU desde el sitio del fabricante."
                issues.append({"severidad": sev, "titulo": titulo, "descripcion": desc, "sugerencia": sug})
            return {"ok": True, "fuente": fuente, "issues": issues,
                    "mods_cargados": mods_cargados,
                    "fragmento": contenido[-3000:]}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def analizar_crash_con_ia(self, fragmento, _unused_key=""):
        try:
            if not fragmento:
                return {"ok": False, "error": "Fragmento vacío."}
            prompt = ("Sos un experto en Minecraft y modding. Analizá este log de crash y explicá:"
                      " 1) Cuál fue la causa principal, 2) Qué mods o configuración lo generó,"
                      " 3) Cómo solucionarlo paso a paso. Respondé en español, claro y conciso.\n\nLOG:\n"
                      + fragmento[:4000])
            texto, err = self._gemini(prompt, max_tokens=600, temperature=0.2)
            if texto:
                return {"ok": True, "analisis": texto}
            return {"ok": False, "error": err or "Sin respuesta de Gemini."}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def _gemini(self, prompt, max_tokens=800, temperature=0.7):
        if not self._GEMINI_API_KEY:
            return None, "API key de Gemini no configurada."
        try:
            import re as _re
            payload = {"contents": [{"parts": [{"text": prompt}]}],
                       "generationConfig": {"temperature": temperature, "maxOutputTokens": max_tokens}}
            _models = (
                "gemma-3-27b-it",        # 14 400 RPD · 30 RPM
                "gemma-3-12b-it",        # 14 400 RPD · 30 RPM
                "gemma-3-4b-it",         # 14 400 RPD · 30 RPM
                "gemma-3-1b-it",         # 14 400 RPD · 30 RPM
                "gemini-2.0-flash-lite", # 500 RPD  · 15 RPM
                "gemini-2.0-flash",      # 20 RPD   · 5 RPM
                "gemini-1.5-flash",      # fallback
                "gemini-1.5-flash-8b",   # fallback
            )
            last_err = "Sin respuesta de Gemini."
            for model in _models:
                url = (f"https://generativelanguage.googleapis.com/v1beta/models/"
                       f"{model}:generateContent?key={self._GEMINI_API_KEY}")
                r = requests.post(url, json=payload, headers={"Content-Type": "application/json"}, timeout=25)
                if r.status_code == 200:
                    return r.json()["candidates"][0]["content"]["parts"][0]["text"], None
                if r.status_code in (404, 400):
                    continue
                if r.status_code == 429:
                    msg = r.json().get("error", {}).get("message", "")
                    retry = _re.search(r'retry in ([\d.]+)s', msg)
                    secs = int(float(retry.group(1))) if retry else None
                    last_err = (f"⏳ Cuota de Gemini agotada."
                                f"{f' Reintentá en {secs}s.' if secs else ' Reintentá en unos minutos.'}")
                    continue
                last_err = f"Gemini {r.status_code}: " + r.json().get("error", {}).get("message", r.text[:120])
                break
            return None, last_err
        except Exception as e:
            return None, str(e)

    # ── GEMINI CHAT ───────────────────────────────────────────────────────
    def gemini_chat(self, mensaje, historial=None):
        try:
            ctx = ("Sos un asistente experto en Minecraft integrado en el launcher Paraguacraft. "
                   "Respondé siempre en español rioplatense, de forma concisa y amigable. "
                   "Podés responder sobre recetas, comandos, mods, seeds, versiones, estrategias, builds, etc.")
            conv = ""
            for h in (historial or [])[-6:]:
                rol = "Usuario" if h["rol"] == "user" else "Asistente"
                conv += f"\n{rol}: {h['texto']}"
            prompt = f"{ctx}\n\nConversación:{conv}\n\nUsuario: {mensaje}\nAsistente:"
            resp, err = self._gemini(prompt, max_tokens=500)
            if resp:
                return {"ok": True, "respuesta": resp.strip()}
            return {"ok": False, "error": err or "Sin respuesta de Gemini."}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── GEMINI COMANDOS ───────────────────────────────────────────────────
    def gemini_generar_comando(self, descripcion, version="1.21"):
        try:
            prompt = (f"Sos un experto en comandos de Minecraft. Versión: {version}.\n"
                      f"Generá el/los comandos exactos para: {descripcion}\n"
                      "Respondé SOLO con los comandos (uno por línea), sin explicación. "
                      "Si necesitás más de uno, ponelos en orden de ejecución.")
            resp, err = self._gemini(prompt, max_tokens=300, temperature=0.1)
            if resp:
                cmds = [l.strip() for l in resp.strip().splitlines() if l.strip()]
                return {"ok": True, "comandos": cmds}
            return {"ok": False, "error": err or "Sin respuesta."}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── GEMINI MODPACK ────────────────────────────────────────────────────
    def gemini_generar_modpack(self, descripcion, version, motor):
        try:
            prompt = (f"Sos un experto en modding de Minecraft. El usuario quiere: \"{descripcion}\".\n"
                      f"Versión: {version}, Loader: {motor}.\n"
                      "Devolvé una lista JSON de exactamente 8 slugs de mods de Modrinth compatibles "
                      "con esa versión y loader, ordenados por importancia. Solo el array JSON, sin texto extra. "
                      "Ejemplo: [\"sodium\",\"lithium\",\"fabric-api\",\"jei\"]")
            resp, err = self._gemini(prompt, max_tokens=200, temperature=0.2)
            if not resp:
                return {"ok": False, "error": err or "Sin respuesta de Gemini."}
            import re
            match = re.search(r'\[.*?\]', resp, re.DOTALL)
            if not match:
                return {"ok": False, "error": "Formato de respuesta inválido."}
            slugs = json.loads(match.group())
            return {"ok": True, "slugs": slugs}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── GEMINI SEMILLAS ────────────────────────────────────────────────────
    def gemini_generar_semilla(self, descripcion):
        try:
            prompt = (
                "Sos un experto en Minecraft. El jugador describe el tipo de mundo que quiere. "
                "Generá 3 seeds de Minecraft (números o texto) con una breve explicación de cada una "
                "(biomas cercanos al spawn, estructuras, etc.). "
                "Respondé en español, formato claro con numeración. "
                f"\nDescripción del jugador: {descripcion[:300]}"
            )
            resp, err = self._gemini(prompt, max_tokens=400, temperature=0.7)
            if resp:
                return {"ok": True, "respuesta": resp}
            return {"ok": False, "error": err or "Sin respuesta de Gemini."}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── GEMINI SURVIVAL ASSIST ─────────────────────────────────────────────
    def gemini_survival_assist(self, pregunta):
        try:
            prompt = (
                "Sos un asistente experto en Minecraft Survival. "
                "Respondé de forma ULTRA CONCISA y directa en español. "
                "Máximo 3-4 líneas por respuesta. Priorizá pasos claros y recetas exactas. "
                f"\nPregunta: {pregunta[:400]}"
            )
            resp, err = self._gemini(prompt, max_tokens=300, temperature=0.3)
            if resp:
                return {"ok": True, "respuesta": resp}
            return {"ok": False, "error": err or "Sin respuesta de Gemini."}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── SCREENSHOTS ───────────────────────────────────────────────────────
    def get_screenshots(self):
        try:
            import minecraft_launcher_lib
            mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            shot_dirs = [os.path.join(mc_dir, "screenshots")]
            instancias_dir = os.path.join(mc_dir, "instancias")
            if os.path.isdir(instancias_dir):
                for inst in os.listdir(instancias_dir):
                    s = os.path.join(instancias_dir, inst, "screenshots")
                    if os.path.isdir(s):
                        shot_dirs.append(s)
            items = []
            seen = set()
            for shots_dir in shot_dirs:
                if not os.path.isdir(shots_dir):
                    continue
                for f in os.listdir(shots_dir):
                    if f.lower().endswith((".png", ".jpg", ".jpeg")) and f not in seen:
                        seen.add(f)
                        full = os.path.join(shots_dir, f)
                        items.append({"nombre": f, "ruta": full.replace("\\", "/"),
                                      "fecha": datetime.datetime.fromtimestamp(os.path.getmtime(full)).strftime("%d/%m/%Y %H:%M")})
            items.sort(key=lambda x: x["fecha"], reverse=True)
            return {"ok": True, "screenshots": items[:50]}
        except Exception as e:
            return {"ok": False, "error": str(e), "screenshots": []}

    # ── MOD UPDATES ───────────────────────────────────────────────────────
    def get_mod_updates(self, version, motor):
        try:
            import minecraft_launcher_lib
            mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            from core import carpeta_instancia_paraguacraft
            mods_dir = os.path.join(mc_dir, "instancias",
                                    carpeta_instancia_paraguacraft(version.strip(), motor), "mods")
            if not os.path.isdir(mods_dir):
                return {"ok": True, "mods": []}
            loader = "fabric" if "fabric" in motor.lower() else "forge" if "forge" in motor.lower() else "quilt"
            archivos = [f for f in os.listdir(mods_dir) if f.endswith(".jar")]
            resultados = []
            for jar in archivos[:20]:
                slug_guess = jar.replace(".jar","").split("-")[0].lower()
                try:
                    url = f"https://api.modrinth.com/v2/project/{slug_guess}/version"
                    params = {"game_versions": f'["{version}"]', "loaders": f'["{loader}"]'}
                    r = requests.get(url, params=params, headers={"User-Agent": "ParaguacraftLauncher/2.0"}, timeout=5)
                    if r.status_code == 200 and r.json():
                        latest = r.json()[0]
                        latest_files = latest.get("files", [])
                        latest_fn = latest_files[0]["filename"] if latest_files else ""
                        update_disp = bool(latest_fn) and latest_fn != jar
                        resultados.append({"archivo": jar, "slug": slug_guess,
                                           "version_nueva": latest.get("version_number","?"),
                                           "filename_nuevo": latest_fn,
                                           "fecha": latest.get("date_published","")[:10],
                                           "update_disponible": update_disp})
                    else:
                        resultados.append({"archivo": jar, "slug": slug_guess,
                                           "update_disponible": False})
                except Exception:
                    resultados.append({"archivo": jar, "slug": slug_guess, "update_disponible": False})
            return {"ok": True, "mods": resultados}
        except Exception as e:
            return {"ok": False, "error": str(e), "mods": []}

    # ── TURBO MODE ────────────────────────────────────────────────────────
    _turbo_activo = False
    _turbo_dns_backup = []

    def turbo_mode_toggle(self, activar):
        try:
            import subprocess
            self._turbo_activo = activar
            msgs = []
            if activar:
                result = subprocess.run(
                    ['netsh', 'interface', 'ip', 'set', 'dns', 'name=Wi-Fi', 'static', '1.1.1.1'],
                    capture_output=True, text=True, timeout=5)
                msgs.append("DNS → Cloudflare 1.1.1.1 ✅" if result.returncode == 0 else "DNS: requiere admin")
                kill_list = ["Teams.exe", "OneDrive.exe"]
                killed = []
                for proc_name in kill_list:
                    k = subprocess.run(['taskkill', '/F', '/IM', proc_name], capture_output=True, timeout=3)
                    if k.returncode == 0:
                        killed.append(proc_name.replace(".exe",""))
                if killed:
                    msgs.append(f"Cerrados: {', '.join(killed)}")
                msgs.append("Modo Turbo activado 🚀")
            else:
                msgs.append("Modo Turbo desactivado")
            return {"ok": True, "activo": activar, "mensajes": msgs}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── OVERLAY ───────────────────────────────────────────────────────────
    _overlay_window = None

    def abrir_overlay(self):
        try:
            import tkinter as tk
            if self.__class__._overlay_window is not None:
                try:
                    self.__class__._overlay_window.destroy()
                except Exception:
                    pass
                self.__class__._overlay_window = None

            def _run():
                try:
                    import time as _time
                    import subprocess as _sp
                    import ctypes, re as _re

                    C_BG  = '#111111'
                    C_WIN = '#000000'

                    root = tk.Tk()
                    root.overrideredirect(True)
                    root.attributes('-topmost', True)
                    root.attributes('-alpha', 0.80)
                    root.configure(bg=C_WIN)
                    try:
                        root.wm_attributes('-transparentcolor', C_WIN)
                    except Exception:
                        pass
                    sw = root.winfo_screenwidth()
                    root.geometry(f'188x148+{sw - 198}+20')

                    def _drag_start(e): root._dx = e.x; root._dy = e.y
                    def _drag_move(e):
                        root.geometry(f'+{root.winfo_x()+e.x-root._dx}+{root.winfo_y()+e.y-root._dy}')

                    outer = tk.Frame(root, bg=C_BG, highlightbackground='#2ECC71',
                                     highlightthickness=1)
                    outer.pack(fill='both', expand=True)
                    outer.bind('<Button-1>', _drag_start)
                    outer.bind('<B1-Motion>', _drag_move)

                    header = tk.Frame(outer, bg=C_BG)
                    header.pack(fill='x', padx=4, pady=(3, 1))
                    header.bind('<Button-1>', _drag_start)
                    header.bind('<B1-Motion>', _drag_move)
                    tk.Label(header, text='Paraguacraft', bg=C_BG, fg='#2ECC71',
                             font=('Segoe UI', 7, 'bold')).pack(side='left')
                    tk.Button(header, text='✕', bg=C_BG, fg='#555',
                              relief='flat', cursor='hand2', font=('Segoe UI', 7),
                              activebackground='#E74C3C', activeforeground='white',
                              command=root.destroy).pack(side='right')

                    _ROW_DEFS = [
                        ('FPS',   '#F1C40F'),
                        ('CPU',   '#2ECC71'),
                        ('CPU°',  '#E67E22'),
                        ('GPU',   '#3498DB'),
                        ('GPU°',  '#E74C3C'),
                        ('RAM',   '#9B59B6'),
                        ('VRAM',  '#1ABC9C'),
                    ]
                    _labels = {}
                    for key, color in _ROW_DEFS:
                        fr = tk.Frame(outer, bg=C_BG)
                        fr.pack(fill='x', padx=6, pady=0)
                        fr.bind('<Button-1>', _drag_start)
                        fr.bind('<B1-Motion>', _drag_move)
                        lbl_key = tk.Label(fr, text=key, bg=C_BG,
                                           fg='#666666', font=('Consolas', 7), width=5, anchor='w')
                        lbl_key.pack(side='left')
                        lbl_key.bind('<Button-1>', _drag_start)
                        lbl_key.bind('<B1-Motion>', _drag_move)
                        lbl_val = tk.Label(fr, text='—', bg=C_BG, fg=color,
                                           font=('Consolas', 7, 'bold'), anchor='w')
                        lbl_val.pack(side='left', fill='x', expand=True)
                        lbl_val.bind('<Button-1>', _drag_start)
                        lbl_val.bind('<B1-Motion>', _drag_move)
                        _labels[key] = (lbl_val, color)

                    _cache = {'cpu_temp': '—', 'gpu_pct': '—', 'gpu_temp': '—', 'vram': '—'}
                    _alive = [True]

                    def _bg_gpu():
                        _flags = _sp.CREATE_NO_WINDOW
                        while _alive[0]:
                            nvidia_ok = False
                            # 1. NVIDIA via nvidia-smi
                            try:
                                r = _sp.run(
                                    ['nvidia-smi',
                                     '--query-gpu=utilization.gpu,temperature.gpu,memory.used,memory.total',
                                     '--format=csv,noheader,nounits'],
                                    capture_output=True, text=True, timeout=4,
                                    creationflags=_flags)
                                if r.returncode == 0:
                                    p = [x.strip() for x in r.stdout.strip().split(',')]
                                    if len(p) >= 4:
                                        _cache['gpu_pct'] = p[0] + '%'
                                        _cache['gpu_temp'] = p[1] + '°C'
                                        _cache['vram'] = f'{p[2]}MB/{p[3]}MB'
                                        nvidia_ok = True
                            except Exception:
                                pass

                            if not nvidia_ok:
                                # 2. AMD/Intel: Windows Perf Counter GPU usage (Win10+)
                                try:
                                    r2 = _sp.run(
                                        ['powershell', '-NoProfile', '-Command',
                                         '$s=(Get-Counter "\\GPU Engine(*engtype_3D*)\\Utilization Percentage"'
                                         ' -ErrorAction SilentlyContinue).CounterSamples |'
                                         ' Where-Object {$_.CookedValue -gt 0} |'
                                         ' Measure-Object -Property CookedValue -Sum;'
                                         ' if($s){[Math]::Min(100,[Math]::Round($s.Sum))}else{0}'],
                                        capture_output=True, text=True, timeout=6,
                                        creationflags=_flags)
                                    val = r2.stdout.strip()
                                    if val and val.replace('.', '').isdigit():
                                        _cache['gpu_pct'] = f'{round(float(val))}%'
                                except Exception:
                                    pass

                                # 3. VRAM used: Windows Perf Counter dedicated memory (Win10+)
                                try:
                                    r3 = _sp.run(
                                        ['powershell', '-NoProfile', '-Command',
                                         '$s=(Get-Counter "\\GPU Process Memory(*)\\Dedicated Usage"'
                                         ' -ErrorAction SilentlyContinue).CounterSamples |'
                                         ' Measure-Object -Property CookedValue -Sum;'
                                         ' if($s){[Math]::Round($s.Sum/1MB)}else{0}'],
                                        capture_output=True, text=True, timeout=6,
                                        creationflags=_flags)
                                    val3 = r3.stdout.strip()
                                    # Total VRAM from WMI
                                    r4 = _sp.run(
                                        ['powershell', '-NoProfile', '-Command',
                                         '[Math]::Round((Get-WmiObject Win32_VideoController |'
                                         ' Select-Object -First 1).AdapterRAM/1MB)'],
                                        capture_output=True, text=True, timeout=4,
                                        creationflags=_flags)
                                    val4 = r4.stdout.strip()
                                    if val3 and val3.isdigit() and val4 and val4.isdigit():
                                        _cache['vram'] = f'{val3}MB/{val4}MB'
                                    elif val3 and val3.isdigit():
                                        _cache['vram'] = f'{val3}MB'
                                except Exception:
                                    pass

                            _time.sleep(2)

                    def _bg_cpu_temp():
                        _flags = _sp.CREATE_NO_WINDOW
                        while _alive[0]:
                            found = False
                            # 1. MSAcpi_ThermalZoneTemperature (returns tenths of Kelvin)
                            try:
                                r = _sp.run(
                                    ['powershell', '-NoProfile', '-Command',
                                     '(Get-WmiObject -Namespace "root/wmi" MSAcpi_ThermalZoneTemperature'
                                     ' -ErrorAction SilentlyContinue |'
                                     ' Select-Object -First 1).CurrentTemperature'],
                                    capture_output=True, text=True, timeout=4,
                                    creationflags=_flags)
                                val = r.stdout.strip()
                                if val and val.replace('.', '').isdigit():
                                    temp_c = round((float(val) - 2732) / 10)
                                    if 0 < temp_c < 120:
                                        _cache['cpu_temp'] = f'{temp_c}°C'
                                        found = True
                            except Exception:
                                pass
                            # 2. Win32_PerfFormattedData_Counters_ThermalZoneInformation (Kelvin)
                            if not found:
                                try:
                                    r2 = _sp.run(
                                        ['powershell', '-NoProfile', '-Command',
                                         '(Get-WmiObject Win32_PerfFormattedData_Counters_ThermalZoneInformation'
                                         ' -ErrorAction SilentlyContinue |'
                                         ' Select-Object -First 1).Temperature'],
                                        capture_output=True, text=True, timeout=4,
                                        creationflags=_flags)
                                    val2 = r2.stdout.strip()
                                    if val2 and val2.replace('.', '').isdigit():
                                        temp_c = round(float(val2) - 273.15)
                                        if 0 < temp_c < 120:
                                            _cache['cpu_temp'] = f'{temp_c}°C'
                                except Exception:
                                    pass
                            _time.sleep(3)

                    def _get_mc_fps():
                        try:
                            EnumWP = ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_long, ctypes.c_long)
                            found = []
                            def _cb(hwnd, _):
                                n = ctypes.windll.user32.GetWindowTextLengthW(hwnd)
                                if n > 0:
                                    buf = ctypes.create_unicode_buffer(n + 1)
                                    ctypes.windll.user32.GetWindowTextW(hwnd, buf, n + 1)
                                    m = _re.search(r'(\d+)\s*fps', buf.value, _re.IGNORECASE)
                                    if m:
                                        found.append(m.group(1) + ' fps')
                                return True
                            ctypes.windll.user32.EnumWindows(EnumWP(_cb), 0)
                            return found[0] if found else '—'
                        except Exception:
                            return '—'

                    threading.Thread(target=_bg_gpu, daemon=True).start()
                    threading.Thread(target=_bg_cpu_temp, daemon=True).start()

                    def _set(key, text, color=None):
                        lbl, default_color = _labels[key]
                        lbl.config(text=text, fg=color or default_color)

                    def _update():
                        try:
                            _set('FPS',  _get_mc_fps())
                            cpu = psutil.cpu_percent(interval=None)
                            cpu_color = '#2ECC71' if cpu < 70 else '#F39C12' if cpu < 90 else '#E74C3C'
                            _set('CPU',  f'{cpu:.0f}%', cpu_color)
                            _set('CPU°', _cache['cpu_temp'])
                            _set('GPU',  _cache['gpu_pct'])
                            _set('GPU°', _cache['gpu_temp'])
                            vm = psutil.virtual_memory()
                            _set('RAM',  f'{vm.used/(1024**3):.1f}/{vm.total/(1024**3):.0f}GB')
                            _set('VRAM', _cache['vram'])
                            root.after(1000, _update)
                        except Exception:
                            pass

                    _update()
                    self.__class__._overlay_window = root
                    root.mainloop()
                except Exception:
                    pass
                finally:
                    _alive[0] = False
                    self.__class__._overlay_window = None

            threading.Thread(target=_run, daemon=True).start()
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def cerrar_overlay(self):
        try:
            w = self.__class__._overlay_window
            if w is not None:
                try:
                    w.destroy()
                except Exception:
                    pass
                self.__class__._overlay_window = None
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── KEYSTROKE OVERLAY ─────────────────────────────────────────────────
    _keystroke_window = None

    def abrir_keystroke_overlay(self, cfg=None):
        if cfg is None:
            cfg = {}
        _show_wasd    = bool(cfg.get('show_wasd',    True))
        _show_extra   = bool(cfg.get('show_extra',   True))
        _show_mouse   = bool(cfg.get('show_mouse',   True))
        _show_compass = bool(cfg.get('show_compass', True))
        try:
            import tkinter as tk
            import time
            from collections import deque
            if self.__class__._keystroke_window is not None:
                try: self.__class__._keystroke_window.destroy()
                except Exception: pass
                self.__class__._keystroke_window = None

            def _run():
                _pynput_listener = None
                try:
                    import math as _math
                    C_TRANS   = '#010203'
                    C_KEY_BG  = '#151515'
                    C_KEY_OFF = '#2A2A2A'
                    C_TEAL    = '#00CCAA'
                    C_BLUE    = '#5599FF'
                    C_RED     = '#FF4466'
                    C_WASD_ON = '#003D30'
                    C_MB_L_ON = '#002F28'
                    C_MB_R_ON = '#001235'

                    KS = 52
                    PAD = 3
                    CW = 260
                    WASD_H  = 120 if _show_wasd else 0
                    EXTRA_H = 32  if _show_extra else 0
                    MB_H    = 80  if _show_mouse else 0
                    COMP_H  = 110 if _show_compass else 0
                    H = 8 + WASD_H + EXTRA_H + MB_H + COMP_H + 8

                    root = tk.Tk()
                    root.overrideredirect(True)
                    root.attributes('-topmost', True)
                    root.configure(bg=C_TRANS)
                    try:
                        root.wm_attributes('-transparentcolor', C_TRANS)
                    except Exception:
                        pass
                    sw = root.winfo_screenwidth()
                    sh = root.winfo_screenheight()
                    root.geometry(f'{CW}x{H}+{sw//2 - CW//2}+{sh - H - 50}')

                    cv = tk.Canvas(root, width=CW, height=H, bg=C_TRANS,
                                   highlightthickness=0)
                    cv.pack()

                    def _drag_start(e):
                        root._ox = root.winfo_x(); root._oy = root.winfo_y()
                        root._mx = e.x_root; root._my = e.y_root
                    def _drag_move(e):
                        root.geometry(f'+{root._ox+e.x_root-root._mx}+{root._oy+e.y_root-root._my}')
                    cv.bind('<Button-1>', _drag_start)
                    cv.bind('<B1-Motion>', _drag_move)

                    def _key_rect(x, y, w, h, text, fsz=13):
                        rid = cv.create_rectangle(x, y, x+w, y+h,
                                                   fill=C_KEY_BG, outline=C_KEY_OFF,
                                                   width=1)
                        cv.create_line(x+4, y+1, x+w-4, y+1, fill='#222')
                        tid = cv.create_text(x+w//2, y+h//2, text=text,
                                             fill=C_KEY_OFF,
                                             font=('Consolas', fsz, 'bold'))
                        return [rid, tid]

                    yy = 8
                    _key_ids   = {}
                    _extra_ids = {}

                    if _show_wasd:
                        cx_w = CW // 2
                        _key_ids['w'] = _key_rect(cx_w - KS//2, yy, KS, KS, 'W')
                        row_y = yy + KS + PAD
                        _key_ids['a'] = _key_rect(cx_w - KS//2 - KS - PAD, row_y, KS, KS, 'A')
                        _key_ids['s'] = _key_rect(cx_w - KS//2,             row_y, KS, KS, 'S')
                        _key_ids['d'] = _key_rect(cx_w - KS//2 + KS + PAD, row_y, KS, KS, 'D')
                        yy += WASD_H

                    if _show_extra:
                        ex = (CW - (4*(38+PAD)-PAD)) // 2
                        for kid, ktxt in [('space','SPC'),('shift_l','SHF'),
                                          ('control_l','CTL'),('e','INV')]:
                            _extra_ids[kid] = _key_rect(ex, yy, 38, 22, ktxt, fsz=7)
                            ex += 38 + PAD
                        yy += EXTRA_H

                    _lmb_ids = _rmb_ids = None
                    _lmb_cps_id = _rmb_cps_id = None
                    MBW, MBH = 80, 50
                    if _show_mouse:
                        mx_l = CW//2 - MBW - 6
                        mx_r = CW//2 + 6
                        _lmb_ids = _key_rect(mx_l, yy, MBW, MBH, 'LMB', fsz=11)
                        _lmb_cps_id = cv.create_text(mx_l+MBW//2, yy+MBH+10,
                                                      text='0 CPS', fill='#555',
                                                      font=('Consolas', 7))
                        _rmb_ids = _key_rect(mx_r, yy, MBW, MBH, 'RMB', fsz=11)
                        _rmb_cps_id = cv.create_text(mx_r+MBW//2, yy+MBH+10,
                                                      text='0 CPS', fill='#555',
                                                      font=('Consolas', 7))
                        yy += MB_H

                    canvas = cv
                    _arrow = None
                    _dir_dots = {}
                    _spd_txt = None
                    cx_comp = CW // 2
                    cy_comp = yy + COMP_H // 2 - 4
                    R = 36
                    if _show_compass:
                        cv.create_oval(cx_comp-R, cy_comp-R, cx_comp+R, cy_comp+R,
                                       fill='#0D0D0D', outline='#2A2A2A', width=1)

                    _dir_angles = {
                        'N': -90, 'NE': -45, 'E': 0,  'SE': 45,
                        'S':  90, 'SW': 135, 'W': 180, 'NW': -135
                    }
                    if _show_compass:
                        for dname, ang in _dir_angles.items():
                            rad = _math.radians(ang)
                            px = cx_comp + int(_math.cos(rad) * (R - 8))
                            py = cy_comp + int(_math.sin(rad) * (R - 8))
                            _dir_dots[dname] = cv.create_oval(
                                px-3, py-3, px+3, py+3,
                                fill='#1E1E1E', outline='#303030')
                            if dname in ('N', 'S', 'E', 'W'):
                                lx = cx_comp + int(_math.cos(rad) * (R + 13))
                                ly = cy_comp + int(_math.sin(rad) * (R + 13))
                                c_lbl = '#E74C3C' if dname == 'N' else '#555'
                                cv.create_text(lx, ly, text=dname, fill=c_lbl,
                                               font=('Consolas', 7, 'bold'))
                        _arrow = cv.create_line(cx_comp, cy_comp, cx_comp, cy_comp-1,
                                                fill=C_TEAL, width=2, arrow=tk.LAST,
                                                arrowshape=(8, 10, 4))
                        cv.create_oval(cx_comp-3, cy_comp-3, cx_comp+3, cy_comp+3,
                                       fill='#444', outline='')
                        _spd_txt = cv.create_text(CW-24, cy_comp, text='0\npx/s',
                                                   fill='#555', font=('Consolas', 7),
                                                   justify='center')

                    _lmb_ts   = deque()
                    _rmb_ts   = deque()
                    _mpos     = [0, 0]
                    _mlast    = [0, 0]
                    _last_tick = [time.time()]
                    _cur_dir  = [None]

                    def _calc_cps(dq):
                        now = time.time()
                        while dq and dq[0] < now - 1.0:
                            dq.popleft()
                        return len(dq)

                    def _get_dir8(dx, dy):
                        if abs(dx) < 3 and abs(dy) < 3:
                            return None
                        ang = (_math.degrees(_math.atan2(dy, dx)) + 360) % 360
                        return ['E','SE','S','SW','W','NW','N','NE'][
                            int((ang + 22.5) / 45) % 8]

                    def _tick():
                        if not root.winfo_exists():
                            return
                        lc = _calc_cps(_lmb_ts)
                        rc = _calc_cps(_rmb_ts)
                        if _lmb_cps_id:
                            cv.itemconfig(_lmb_cps_id,
                                          text=f'{lc} CPS',
                                          fill=C_TEAL if lc > 0 else '#555')
                        if _rmb_cps_id:
                            cv.itemconfig(_rmb_cps_id,
                                          text=f'{rc} CPS',
                                          fill=C_BLUE if rc > 0 else '#555')
                        now = time.time()
                        dt = max(now - _last_tick[0], 0.001)
                        _last_tick[0] = now
                        dx = _mpos[0] - _mlast[0]
                        dy = _mpos[1] - _mlast[1]
                        _mlast[0] = _mpos[0]; _mlast[1] = _mpos[1]
                        spd = int(((dx*dx + dy*dy) ** 0.5) / dt)
                        new_dir = _get_dir8(dx, dy)
                        if _show_compass and _arrow is not None:
                            if new_dir != _cur_dir[0]:
                                if _cur_dir[0] and _cur_dir[0] in _dir_dots:
                                    cv.itemconfig(_dir_dots[_cur_dir[0]],
                                                  fill='#1E1E1E', outline='#303030')
                                if new_dir and new_dir in _dir_dots:
                                    cv.itemconfig(_dir_dots[new_dir],
                                                  fill=C_TEAL, outline=C_TEAL)
                                _cur_dir[0] = new_dir
                            if new_dir:
                                ang_r = _math.radians(_dir_angles[new_dir])
                                ax = cx_comp + int(_math.cos(ang_r) * 24)
                                ay = cy_comp + int(_math.sin(ang_r) * 24)
                                sc = C_RED if spd > 800 else C_TEAL if spd > 100 else '#666'
                                cv.coords(_arrow, cx_comp, cy_comp, ax, ay)
                                cv.itemconfig(_arrow, fill=sc)
                            else:
                                cv.coords(_arrow, cx_comp, cy_comp, cx_comp, cy_comp)
                            sc = C_RED if spd > 800 else C_TEAL if spd > 100 else '#555'
                            if _spd_txt:
                                cv.itemconfig(_spd_txt, text=f'{spd}\npx/s', fill=sc)
                        root.after(80, _tick)

                    root.after(80, _tick)

                    def _on_key(event, pressed):
                        name = event.keysym.lower()
                        if name in _key_ids:
                            rid, tid = _key_ids[name]
                            cv.itemconfig(rid,
                                          fill=C_WASD_ON if pressed else C_KEY_BG,
                                          outline=C_TEAL if pressed else C_KEY_OFF)
                            cv.itemconfig(tid,
                                          fill=C_TEAL if pressed else C_KEY_OFF)
                        elif name in _extra_ids:
                            rid, tid = _extra_ids[name]
                            cv.itemconfig(rid,
                                          fill='#1E1E1E' if pressed else C_KEY_BG,
                                          outline=C_TEAL if pressed else C_KEY_OFF)
                            cv.itemconfig(tid,
                                          fill=C_TEAL if pressed else C_KEY_OFF)

                    root.bind_all('<KeyPress>', lambda e: _on_key(e, True))
                    root.bind_all('<KeyRelease>', lambda e: _on_key(e, False))
                    root.focus_force()

                    _pm = None
                    try:
                        from pynput import mouse as _pm
                    except ImportError:
                        try:
                            import subprocess as _sp2, sys as _sys2
                            _sp2.check_call(
                                [_sys2.executable, '-m', 'pip', 'install', '--quiet', 'pynput'],
                                stdout=_sp2.DEVNULL, stderr=_sp2.DEVNULL)
                            from pynput import mouse as _pm
                        except Exception:
                            pass

                    if _pm is None and _spd_txt:
                        cv.itemconfig(_spd_txt, text='sin\npynput', fill='#F39C12')
                    else:
                        def _on_click(x, y, button, pressed_):
                            if not root.winfo_exists():
                                return False
                            try:
                                if button == _pm.Button.left:
                                    if pressed_: _lmb_ts.append(time.time())
                                    if _lmb_ids:
                                        def _upd_lmb(p=pressed_):
                                            cv.itemconfig(_lmb_ids[0],
                                                          fill=C_MB_L_ON if p else C_KEY_BG,
                                                          outline=C_TEAL if p else C_KEY_OFF)
                                            cv.itemconfig(_lmb_ids[1],
                                                          fill=C_TEAL if p else C_KEY_OFF)
                                        root.after(0, _upd_lmb)
                                elif button == _pm.Button.right:
                                    if pressed_: _rmb_ts.append(time.time())
                                    if _rmb_ids:
                                        def _upd_rmb(p=pressed_):
                                            cv.itemconfig(_rmb_ids[0],
                                                          fill=C_MB_R_ON if p else C_KEY_BG,
                                                          outline=C_BLUE if p else C_KEY_OFF)
                                            cv.itemconfig(_rmb_ids[1],
                                                          fill=C_BLUE if p else C_KEY_OFF)
                                        root.after(0, _upd_rmb)
                            except Exception:
                                pass

                        def _on_move(x, y):
                            _mpos[0] = x; _mpos[1] = y

                        _pynput_listener = _pm.Listener(on_click=_on_click, on_move=_on_move)
                        _pynput_listener.daemon = True
                        _pynput_listener.start()

                    self.__class__._keystroke_window = root
                    root.mainloop()
                except Exception:
                    pass
                finally:
                    self.__class__._keystroke_window = None
                    if _pynput_listener:
                        try: _pynput_listener.stop()
                        except Exception: pass

            threading.Thread(target=_run, daemon=True).start()
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def get_carpeta_mods(self, version, motor):
        try:
            import minecraft_launcher_lib
            mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            from core import carpeta_instancia_paraguacraft
            mods_dir = os.path.join(mc_dir, "instancias",
                                    carpeta_instancia_paraguacraft(version.strip(), motor), "mods")
            return mods_dir
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def instalar_mods_pvp_189(self, carpeta_mods):
        import urllib.request, json as _json
        os.makedirs(carpeta_mods, exist_ok=True)
        results = []
        MODS = [
            {"slug": "keystrokes",   "name": "Keystrokes"},
            {"slug": "evergreenhud", "name": "EvergreenHUD"},
        ]
        for mod in MODS:
            try:
                url = (f"https://api.modrinth.com/v2/project/{mod['slug']}/version"
                       f"?game_versions=[%221.8.9%22]&loaders=[%22forge%22]")
                req = urllib.request.Request(
                    url, headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                with urllib.request.urlopen(req, timeout=10) as resp:
                    versions = _json.loads(resp.read())
                if not versions:
                    results.append({"mod": mod["name"], "ok": False,
                                    "error": "No hay versi\u00f3n para 1.8.9 Forge en Modrinth"})
                    continue
                files = versions[0].get("files", [])
                jar = next((f for f in files if f.get("primary")), files[0] if files else None)
                if not jar:
                    results.append({"mod": mod["name"], "ok": False, "error": "Sin JAR"})
                    continue
                out = os.path.join(carpeta_mods, jar["filename"])
                if os.path.exists(out):
                    results.append({"mod": mod["name"], "ok": True, "ya_existia": True,
                                    "nombre": jar["filename"]})
                    continue
                req2 = urllib.request.Request(
                    jar["url"], headers={"User-Agent": "ParaguacraftLauncher/2.0"})
                with urllib.request.urlopen(req2, timeout=60) as r2:
                    data = r2.read()
                with open(out, "wb") as f:
                    f.write(data)
                results.append({"mod": mod["name"], "ok": True, "ya_existia": False,
                                "nombre": jar["filename"],
                                "size_kb": round(len(data) / 1024)})
            except Exception as e:
                results.append({"mod": mod["name"], "ok": False, "error": str(e)})
        ok_count = sum(1 for r in results if r.get("ok"))
        return {"ok": ok_count > 0, "results": results}

    def set_overlay_opacity(self, valor):
        try:
            alpha = max(0.2, min(1.0, int(valor) / 100))
            w = self.__class__._overlay_window
            if w:
                w.attributes('-alpha', alpha)
            kw = self.__class__._keystroke_window
            if kw:
                kw.attributes('-alpha', alpha)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── IA OVERLAY ────────────────────────────────────────────────────────
    _ia_overlay_window = None

    def abrir_ia_overlay(self):
        try:
            import tkinter as tk
            from tkinter import scrolledtext
            if self.__class__._ia_overlay_window is not None:
                try:
                    self.__class__._ia_overlay_window.destroy()
                except Exception:
                    pass
                self.__class__._ia_overlay_window = None

            def _run():
                try:
                    C_BG    = '#0E0E0E'
                    C_PANEL = '#161616'
                    C_BORD  = '#9B59B6'
                    C_IN    = '#1A1A1A'
                    C_BTN   = '#9B59B6'
                    C_BTN_H = '#7D3C98'
                    C_TEXT  = '#E0E0E0'
                    C_HINT  = '#555555'
                    C_USER  = '#2ECC71'
                    C_AI    = '#9B59B6'

                    root = tk.Tk()
                    root.overrideredirect(True)
                    root.attributes('-topmost', True)
                    root.attributes('-alpha', 0.93)
                    root.configure(bg=C_BG)
                    sw = root.winfo_screenwidth()
                    root.geometry(f'360x480+{sw - 375}+60')

                    def _drag_start(e): root._dx = e.x; root._dy = e.y
                    def _drag_move(e):
                        root.geometry(f'+{root.winfo_x()+e.x-root._dx}+{root.winfo_y()+e.y-root._dy}')

                    outer = tk.Frame(root, bg=C_BG, highlightbackground=C_BORD, highlightthickness=1)
                    outer.pack(fill='both', expand=True)

                    header = tk.Frame(outer, bg=C_PANEL)
                    header.pack(fill='x')
                    header.bind('<Button-1>', _drag_start)
                    header.bind('<B1-Motion>', _drag_move)
                    tk.Label(header, text='🤖 IA Gemini', bg=C_PANEL, fg=C_AI,
                             font=('Segoe UI', 8, 'bold')).pack(side='left', padx=8, pady=5)
                    tk.Label(header, text='F9 = ocultar/mostrar', bg=C_PANEL, fg=C_HINT,
                             font=('Segoe UI', 7)).pack(side='left')
                    tk.Button(header, text='✕', bg=C_PANEL, fg='#555', relief='flat',
                              cursor='hand2', font=('Segoe UI', 8),
                              activebackground='#E74C3C', activeforeground='white',
                              command=root.destroy).pack(side='right', padx=4)

                    chat = scrolledtext.ScrolledText(
                        outer, bg=C_IN, fg=C_TEXT, font=('Segoe UI', 8),
                        relief='flat', wrap='word', state='disabled',
                        bd=0, insertbackground=C_TEXT, height=20)
                    chat.pack(fill='both', expand=True, padx=6, pady=(4, 0))
                    chat.tag_config('user', foreground=C_USER, font=('Segoe UI', 8, 'bold'))
                    chat.tag_config('ai',   foreground=C_AI,   font=('Segoe UI', 8))
                    chat.tag_config('err',  foreground='#E74C3C', font=('Segoe UI', 8))
                    chat.tag_config('hint', foreground=C_HINT,  font=('Segoe UI', 7, 'italic'))

                    def _append(tag, text):
                        chat.config(state='normal')
                        chat.insert('end', text + '\n', tag)
                        chat.config(state='disabled')
                        chat.see('end')

                    _append('hint', 'Preguntale cualquier cosa sobre Minecraft.\n')

                    bottom = tk.Frame(outer, bg=C_BG)
                    bottom.pack(fill='x', padx=6, pady=6)

                    inp = tk.Entry(bottom, bg=C_IN, fg=C_TEXT, insertbackground=C_TEXT,
                                  relief='flat', font=('Segoe UI', 8), bd=0)
                    inp.pack(side='left', fill='x', expand=True, ipady=5, padx=(0, 4))
                    inp.configure(highlightbackground=C_BORD, highlightthickness=1)

                    send_btn = tk.Button(bottom, text='Enviar', bg=C_BTN, fg='white',
                                        relief='flat', font=('Segoe UI', 8, 'bold'),
                                        cursor='hand2', padx=8,
                                        activebackground=C_BTN_H, activeforeground='white')
                    send_btn.pack(side='right', ipady=4)

                    def _send(_event=None):
                        msg = inp.get().strip()
                        if not msg:
                            return
                        inp.delete(0, 'end')
                        send_btn.config(state='disabled', text='...')
                        _append('user', f'Vos: {msg}')

                        def _ask():
                            try:
                                resp, err = self._gemini(
                                    f"Sos un asistente experto en Minecraft. Respondé en español rioplatense, "
                                    f"de forma concisa y directa. Máximo 4 oraciones.\n\nPregunta: {msg}",
                                    max_tokens=350, temperature=0.5
                                )
                                root.after(0, lambda: _append('ai', f'IA: {resp.strip()}' if resp else f'Error: {err}'))
                                root.after(0, lambda: (send_btn.config(state='normal', text='Enviar'),))
                            except Exception as ex:
                                root.after(0, lambda: _append('err', f'Error: {ex}'))
                                root.after(0, lambda: send_btn.config(state='normal', text='Enviar'))

                        threading.Thread(target=_ask, daemon=True).start()

                    send_btn.config(command=_send)
                    inp.bind('<Return>', _send)

                    _hotkey_listener = [None]
                    _visible = [True]

                    def _toggle_window():
                        if _visible[0]:
                            root.withdraw()
                            _visible[0] = False
                        else:
                            root.deiconify()
                            root.attributes('-topmost', True)
                            _visible[0] = True

                    def _hotkey_thread():
                        try:
                            from pynput import keyboard as _kb
                            def _on_press(key):
                                try:
                                    if key == _kb.Key.f9:
                                        root.after(0, _toggle_window)
                                except Exception:
                                    pass
                            with _kb.Listener(on_press=_on_press) as lst:
                                _hotkey_listener[0] = lst
                                lst.join()
                        except Exception:
                            pass

                    threading.Thread(target=_hotkey_thread, daemon=True).start()

                    self.__class__._ia_overlay_window = root
                    root.mainloop()
                except Exception:
                    pass
                finally:
                    self.__class__._ia_overlay_window = None

            threading.Thread(target=_run, daemon=True).start()
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def cerrar_ia_overlay(self):
        try:
            w = self.__class__._ia_overlay_window
            if w:
                try:
                    w.destroy()
                except Exception:
                    pass
                self.__class__._ia_overlay_window = None
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── BENCHMARK DE PC COMPLETO ───────────────────────────────────────────
    def run_benchmark(self):
        try:
            import math as _math
            import subprocess as _sp
            import tempfile as _tf

            # 1. CPU benchmark (test matemático)
            t0 = time.perf_counter()
            acc = 0
            for i in range(1, 2_500_001):
                acc += _math.sqrt(i) * _math.log(i)
            elapsed = time.perf_counter() - t0
            cpu_score = max(1, round(1_000_000 / elapsed))

            # 2. Info básica
            cpu_name = platform.processor() or platform.machine() or "CPU desconocida"
            ram_gb = round(psutil.virtual_memory().total / (1024 ** 3), 1)

            # 3. GPU (via wmic)
            try:
                _flags = _sp.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
                r_gpu = _sp.run(
                    ['wmic', 'path', 'win32_VideoController', 'get', 'name'],
                    capture_output=True, text=True, timeout=5, creationflags=_flags)
                lines = [l.strip() for l in r_gpu.stdout.split('\n')
                         if l.strip() and l.strip().lower() != 'name']
                gpu_name = lines[0] if lines else "Desconocida"
            except Exception:
                gpu_name = "Desconocida"

            # 4. Tipo de almacenamiento (PowerShell)
            storage_type = "Desconocido"
            try:
                _flags = _sp.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
                r_disk = _sp.run(
                    ['powershell', '-NoProfile', '-Command',
                     'Get-PhysicalDisk | Select-Object -First 1 -ExpandProperty MediaType'],
                    capture_output=True, text=True, timeout=6, creationflags=_flags)
                media = r_disk.stdout.strip()
                if 'SSD' in media or media == '4':
                    storage_type = "SSD"
                elif 'HDD' in media or media in ('3', 'Unspecified'):
                    storage_type = "HDD"
                else:
                    storage_type = "SSD"
            except Exception:
                pass

            # 5. Velocidad de disco (escritura 10 MB)
            disk_speed_mb = 0
            try:
                test_data = os.urandom(10 * 1024 * 1024)
                with _tf.NamedTemporaryFile(delete=False, suffix='.tmp') as tmp:
                    tmp_path = tmp.name
                t_d = time.perf_counter()
                with open(tmp_path, 'wb') as f:
                    f.write(test_data)
                disk_speed_mb = round(10.0 / max(0.001, time.perf_counter() - t_d))
                try:
                    os.unlink(tmp_path)
                except Exception:
                    pass
            except Exception:
                pass

            # 6. Puntuación total (0-100)
            pts = 0
            # CPU (0-40)
            pts += (40 if cpu_score >= 3_000_000 else
                    30 if cpu_score >= 2_000_000 else
                    20 if cpu_score >= 1_000_000 else
                    10 if cpu_score >= 500_000 else 5)
            # RAM (0-25)
            pts += (25 if ram_gb >= 16 else
                    18 if ram_gb >= 8 else
                    12 if ram_gb >= 6 else
                    6 if ram_gb >= 4 else 2)
            # GPU (0-25)
            gn = gpu_name.lower()
            if any(x in gn for x in ['rtx', 'gtx', 'rx ', 'geforce', 'arc a', 'radeon rx']):
                pts += 25
            elif any(x in gn for x in ['nvidia', 'amd radeon', 'radeon']):
                pts += 18
            elif any(x in gn for x in ['iris xe', 'iris plus', 'radeon vega', '680m', '780m']):
                pts += 14
            else:
                pts += 6
            # Almacenamiento (0-10)
            pts += 10 if storage_type == "SSD" else 4 if storage_type == "HDD" else 7

            # 7. Tier y recomendaciones
            if pts >= 70:
                tier = "alta"
                tier_label = "🏆 PC Gamer"
                tier_color = "#2ECC71"
                tier_desc = "Excelente PC para gaming"
                mc_version = "1.21.x"
                mc_motor = "Fabric + Iris Shaders"
                mc_ram = "8G"
                mc_chunks = "16"
                mc_fps = "100+"
                mc_config = "Ultra — máxima calidad"
                mc_shaders = "Sí, recomendados"
            elif pts >= 40:
                tier = "media"
                tier_label = "⚡ PC Casual"
                tier_color = "#F39C12"
                tier_desc = "Buena para la mayoría de versiones"
                mc_version = "1.20.4"
                mc_motor = "Fabric"
                mc_ram = "4G"
                mc_chunks = "12"
                mc_fps = "60+"
                mc_config = "Media"
                mc_shaders = "No recomendados"
            else:
                tier = "baja"
                tier_label = "🔧 PC Básica"
                tier_color = "#E74C3C"
                tier_desc = "Funcional en versiones optimizadas"
                mc_version = "1.16.5"
                mc_motor = "Vanilla / OptiFine"
                mc_ram = "2G"
                mc_chunks = "8"
                mc_fps = "30+"
                mc_config = "Mínima / Optimizada"
                mc_shaders = "No"

            return {
                "ok": True,
                "cpu_score": cpu_score,
                "cpu_name": cpu_name[:60],
                "cpu_tiempo_s": round(elapsed, 2),
                "ram_gb": ram_gb,
                "gpu_name": gpu_name[:60],
                "storage_type": storage_type,
                "disk_speed_mb": disk_speed_mb,
                "total_pts": pts,
                "tier": tier,
                "tier_label": tier_label,
                "tier_color": tier_color,
                "tier_desc": tier_desc,
                "mc_version": mc_version,
                "mc_motor": mc_motor,
                "mc_ram": mc_ram,
                "mc_chunks": mc_chunks,
                "mc_fps": mc_fps,
                "mc_config": mc_config,
                "mc_shaders": mc_shaders,
            }
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def _benchmark_submit(self, entrada):
        try:
            from src.nube import MONGO_URI, MONGO_USER, MONGO_PASS
            import pymongo
            client = pymongo.MongoClient(
                MONGO_URI, username=MONGO_USER, password=MONGO_PASS,
                authSource="admin", serverSelectionTimeoutMS=4000)
            db = client["paraguacraft_db"]
            col = db["benchmarks"]
            usuario = entrada.get("usuario", "Invitado")
            col.update_one({"usuario": usuario}, {"$set": entrada}, upsert=True)
            client.close()
        except Exception:
            pass

    def get_benchmark_leaderboard(self):
        try:
            from src.nube import MONGO_URI, MONGO_USER, MONGO_PASS
            import pymongo
            client = pymongo.MongoClient(
                MONGO_URI, username=MONGO_USER, password=MONGO_PASS,
                authSource="admin", serverSelectionTimeoutMS=4000)
            db = client["paraguacraft_db"]
            col = db["benchmarks"]
            docs = list(col.find({}, {"_id": 0}).sort("score", -1).limit(15))
            client.close()
            return {"ok": True, "rankings": docs}
        except Exception as e:
            return {"ok": False, "error": str(e), "rankings": []}

    # ── GENERADOR DE SKINS CON IA ─────────────────────────────────────────
    def gemini_generar_skin(self, descripcion):
        try:
            prompt = (
                "Sos un diseñador de skins de Minecraft. El usuario quiere una skin con esta descripción: "
                f'"{descripcion}"\n'
                "Devolvé SOLO un JSON con exactamente estas 6 claves y valores en formato hex (#RRGGBB):\n"
                '{"skin": "#color_piel", "cabello": "#color_pelo", "camiseta": "#color_ropa_superior", '
                '"pantalon": "#color_pantalon", "zapatos": "#color_zapatos", "ojos": "#color_ojos"}\n'
                "Sin texto extra, solo el JSON."
            )
            resp, err = self._gemini(prompt, max_tokens=80, temperature=0.3)
            if not resp:
                return {"ok": False, "error": err or "Sin respuesta de Gemini."}
            match = re.search(r'\{[^}]+\}', resp, re.DOTALL)
            if not match:
                return {"ok": False, "error": "Gemini no devolvió JSON válido."}
            colores = json.loads(match.group())
            keys = {"skin", "cabello", "camiseta", "pantalon", "zapatos", "ojos"}
            if not keys.issubset(colores.keys()):
                return {"ok": False, "error": "JSON incompleto de Gemini."}
            resultado = self._generar_skin_pil(colores)
            resultado["colores"] = colores
            return resultado
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def _hex_to_rgb(self, hex_str):
        h = hex_str.lstrip("#")
        if len(h) == 3:
            h = "".join(c*2 for c in h)
        return tuple(int(h[i:i+2], 16) for i in (0, 2, 4)) + (255,)

    def _generar_skin_pil(self, colores):
        try:
            from PIL import Image, ImageDraw
            img = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
            d = ImageDraw.Draw(img)

            def fill(x0, y0, x1, y1, hex_c):
                d.rectangle([x0, y0, x1 - 1, y1 - 1], fill=self._hex_to_rgb(hex_c))

            c = colores
            # ── HEAD ──────────────────────────────────────────────────────
            fill(8, 0, 16, 8,   c["cabello"])   # top
            fill(0, 8, 8, 16,   c["skin"])       # right
            fill(8, 8, 16, 16,  c["skin"])       # front (face)
            fill(16, 8, 24, 16, c["skin"])       # left
            fill(24, 8, 32, 16, c["cabello"])    # back
            fill(8, 16, 16, 24, c["skin"])       # bottom
            # Eyes (2 pixels on front face)
            fill(9, 10, 11, 12,  c["ojos"])
            fill(13, 10, 15, 12, c["ojos"])
            # ── BODY ──────────────────────────────────────────────────────
            fill(16, 20, 20, 28, c["camiseta"])  # right
            fill(20, 20, 28, 28, c["camiseta"])  # front
            fill(28, 20, 32, 28, c["camiseta"])  # left
            fill(32, 20, 40, 28, c["camiseta"])  # back
            fill(20, 16, 28, 20, c["camiseta"])  # top
            fill(28, 16, 36, 20, c["camiseta"])  # bottom
            # ── RIGHT ARM ─────────────────────────────────────────────────
            fill(40, 16, 44, 20, c["camiseta"])  # top
            fill(40, 20, 44, 28, c["skin"])      # right
            fill(44, 20, 48, 28, c["camiseta"])  # front
            fill(48, 20, 52, 28, c["skin"])      # left
            fill(52, 20, 56, 28, c["camiseta"])  # back
            fill(44, 28, 48, 32, c["skin"])      # bottom
            # ── LEFT ARM (64x64 format) ────────────────────────────────────
            fill(32, 48, 36, 52, c["camiseta"])  # top
            fill(32, 52, 36, 60, c["skin"])      # right
            fill(36, 52, 40, 60, c["camiseta"])  # front
            fill(40, 52, 44, 60, c["skin"])      # left
            fill(44, 52, 48, 60, c["camiseta"])  # back
            fill(36, 60, 40, 64, c["skin"])      # bottom
            # ── RIGHT LEG ─────────────────────────────────────────────────
            fill(0, 16, 4, 20,  c["pantalon"])   # top
            fill(0, 20, 4, 28,  c["pantalon"])   # right
            fill(4, 20, 8, 28,  c["pantalon"])   # front
            fill(8, 20, 12, 28, c["pantalon"])   # left
            fill(12, 20, 16, 28, c["pantalon"])  # back
            # Shoes on lower half of leg
            fill(4, 25, 8, 28,  c["zapatos"])    # front shoe
            fill(0, 25, 4, 28,  c["zapatos"])    # right shoe
            fill(8, 25, 12, 28, c["zapatos"])    # left shoe
            fill(0, 16, 8, 20,  c["pantalon"])   # top
            fill(4, 28, 8, 32,  c["zapatos"])    # bottom
            # ── LEFT LEG (64x64 format) ────────────────────────────────────
            fill(16, 48, 20, 52, c["pantalon"])  # top
            fill(16, 52, 20, 60, c["pantalon"])  # right
            fill(20, 52, 24, 60, c["pantalon"])  # front
            fill(24, 52, 28, 60, c["pantalon"])  # left
            fill(28, 52, 32, 60, c["pantalon"])  # back
            # Shoes on lower half
            fill(20, 57, 24, 60, c["zapatos"])   # front shoe
            fill(16, 57, 20, 60, c["zapatos"])   # right shoe
            fill(24, 57, 28, 60, c["zapatos"])   # left shoe
            fill(20, 60, 24, 64, c["zapatos"])   # bottom

            out_path = os.path.join(os.path.expanduser("~"), "AppData", "Roaming",
                                    ".minecraft", "paraguacraft_skin_gen.png")
            img.save(out_path, "PNG")
            return {"ok": True, "ruta": out_path.replace("\\", "/")}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def aplicar_skin_generada(self):
        try:
            out_path = os.path.join(os.path.expanduser("~"), "AppData", "Roaming",
                                    ".minecraft", "paraguacraft_skin_gen.png")
            if not os.path.exists(out_path):
                return {"ok": False, "error": "Skin no generada aún."}
            dest = os.path.join(os.path.expanduser("~"), "AppData", "Roaming",
                                ".minecraft", "paraguacraft_skin_aplicada.png")
            shutil.copy2(out_path, dest)
            return {"ok": True, "ruta": dest.replace("\\", "/")}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── SMART RAM ─────────────────────────────────────────────────────────
    def get_smart_ram(self):
        try:
            vm = psutil.virtual_memory()
            total_gb = round(vm.total / (1024 ** 3), 1)
            disponible_gb = round(vm.available / (1024 ** 3), 1)
            reserva_mb = 2048
            usable_mb = max(1024, int(vm.total / (1024 ** 2)) - reserva_mb)
            if total_gb >= 32:
                recomendada_mb = min(8192, usable_mb)
                nota = "PC de alta gama — 8 GB para Minecraft es ideal."
            elif total_gb >= 16:
                recomendada_mb = min(6144, usable_mb)
                nota = "16 GB RAM — 6 GB es un buen balance."
            elif total_gb >= 8:
                recomendada_mb = min(3072, usable_mb)
                nota = "8 GB RAM — 3 GB evita problemas de estabilidad."
            else:
                recomendada_mb = min(2048, usable_mb)
                nota = "Poca RAM disponible — 2 GB es lo recomendado."
            return {
                "ok": True,
                "ram_total_gb": total_gb,
                "ram_disponible_gb": disponible_gb,
                "ram_recomendada_mb": recomendada_mb,
                "nota": nota
            }
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def set_ram(self, mb):
        try:
            mb = int(mb)
            self.config_actual["ram_mb"] = mb
            self._guardar()
            return {"ok": True, "ram_mb": mb}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── DETECTOR DE CONFLICTOS DE MODS ────────────────────────────────────
    def detectar_conflictos_mods(self):
        try:
            if minecraft_launcher_lib:
                mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            else:
                mc_dir = os.path.join(os.environ.get('APPDATA', os.path.expanduser('~')), '.minecraft')
            mods_dir = os.path.join(mc_dir, "mods")
            if not os.path.isdir(mods_dir):
                return {"ok": False, "error": "No se encontró la carpeta mods."}
            archivos = [f for f in os.listdir(mods_dir) if f.endswith('.jar')]
            conflictos = []
            CONFLICTOS_CONOCIDOS = [
                (["optifine", "sodium"], "OptiFine y Sodium son incompatibles — usá Iris en lugar de OptiFine."),
                (["optifine", "iris"], "OptiFine e Iris son incompatibles — elegí uno."),
                (["sodium", "embeddium"], "Sodium y Embeddium son el mismo mod con distinto nombre."),
                (["rubidium", "sodium"], "Rubidium y Sodium son incompatibles (mismo propósito)."),
                (["jei", "rei"], "JEI y REI son incompatibles — usá solo uno."),
            ]
            archivos_lower = [f.lower() for f in archivos]
            for mods_pair, descripcion in CONFLICTOS_CONOCIDOS:
                found = [m for m in mods_pair if any(m in a for a in archivos_lower)]
                if len(found) >= 2:
                    archivos_match = [a for a in archivos if any(m in a.lower() for m in found)]
                    conflictos.append({"tipo": "incompatible", "descripcion": descripcion,
                                       "archivos": archivos_match[:3]})
            seen_ids = {}
            for archivo in archivos:
                import re as _re2
                base = _re2.sub(r'[\d\.\-\_v]+.*', '',
                                archivo.lower().replace('-','').replace('_','').replace(' ',''))
                if base in seen_ids:
                    conflictos.append({"tipo": "duplicado",
                                       "descripcion": f"Posible duplicado: {seen_ids[base]} y {archivo}",
                                       "archivos": [seen_ids[base], archivo]})
                else:
                    seen_ids[base] = archivo
            return {"ok": True, "total_mods": len(archivos), "conflictos": conflictos}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── AUTO-OPTIMIZER OPTIONS.TXT ────────────────────────────────────────
    def optimizar_opciones_mc(self):
        try:
            if minecraft_launcher_lib:
                _mc_dir_opt = minecraft_launcher_lib.utils.get_minecraft_directory()
            else:
                _mc_dir_opt = os.path.join(os.environ.get('APPDATA', os.path.expanduser('~')), '.minecraft')
            vm = psutil.virtual_memory()
            total_gb = vm.total / (1024 ** 3)
            import subprocess as _sp
            _flags = _sp.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
            try:
                r = _sp.run(['wmic', 'cpu', 'get', 'NumberOfCores', '/value'],
                            capture_output=True, text=True, timeout=4, creationflags=_flags)
                cores = int([l for l in r.stdout.splitlines() if 'NumberOfCores' in l][0].split('=')[1])
            except Exception:
                cores = 4
            if total_gb >= 16 and cores >= 8:
                tier = "alta"
                opciones = {"renderDistance": "14", "simulationDistance": "12",
                            "particles": "0", "fboEnable": "true", "ao": "2",
                            "biomeBlendRadius": "4", "maxFps": "260", "fullscreen": "false"}
            elif total_gb >= 8 and cores >= 4:
                tier = "media"
                opciones = {"renderDistance": "10", "simulationDistance": "8",
                            "particles": "1", "fboEnable": "true", "ao": "1",
                            "biomeBlendRadius": "2", "maxFps": "120", "fullscreen": "false"}
            else:
                tier = "baja"
                opciones = {"renderDistance": "6", "simulationDistance": "6",
                            "particles": "2", "fboEnable": "false", "ao": "0",
                            "biomeBlendRadius": "0", "maxFps": "60", "fullscreen": "false"}
            mc_dir = _mc_dir_opt
            opts_path = os.path.join(mc_dir, "options.txt")
            if os.path.isfile(opts_path):
                with open(opts_path, 'r', encoding='utf-8', errors='ignore') as f:
                    lines = f.readlines()
                updated = {}
                new_lines = []
                for line in lines:
                    key = line.split(':')[0].strip() if ':' in line else None
                    if key and key in opciones:
                        new_lines.append(f"{key}:{opciones[key]}\n")
                        updated[key] = opciones[key]
                    else:
                        new_lines.append(line)
                for k, v in opciones.items():
                    if k not in updated:
                        new_lines.append(f"{k}:{v}\n")
                        updated[k] = v
                with open(opts_path, 'w', encoding='utf-8') as f:
                    f.writelines(new_lines)
                return {"ok": True, "tier": tier, "opciones_aplicadas": updated}
            else:
                with open(opts_path, 'w', encoding='utf-8') as f:
                    for k, v in opciones.items():
                        f.write(f"{k}:{v}\n")
                return {"ok": True, "tier": tier, "opciones_aplicadas": opciones}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── COMPARADOR DE RENDIMIENTO POR VERSION ─────────────────────────────
    def comparar_rendimiento_versiones(self):
        try:
            vm = psutil.virtual_memory()
            total_gb = vm.total / (1024 ** 3)
            import subprocess as _sp
            _flags = _sp.CREATE_NO_WINDOW if platform.system() == "Windows" else 0
            try:
                r = _sp.run(['wmic', 'cpu', 'get', 'NumberOfCores', '/value'],
                            capture_output=True, text=True, timeout=4, creationflags=_flags)
                cores = int([l for l in r.stdout.splitlines() if 'NumberOfCores' in l][0].split('=')[1])
            except Exception:
                cores = 4
            base = 60 if total_gb < 8 else (90 if total_gb < 16 else 130)
            core_bonus = min(40, (cores - 4) * 5)
            VERSIONES = [
                ("1.8.9",  "Vanilla",  2.5,  "FPS ultra alto, sin features modernas"),
                ("1.12.2", "Fabric",   2.0,  "Clásico modding, buen rendimiento"),
                ("1.16.5", "Fabric",   1.5,  "Nether update, buen balance"),
                ("1.18.2", "Fabric",   1.1,  "Caves & Cliffs, más pesado"),
                ("1.20.1", "Fabric",   1.0,  "Versión estable recomendada"),
                ("1.21.1", "Fabric",   0.9,  "Última version, más optimización requerida"),
                ("1.21.4", "Fabric",   0.8,  "Muy nueva, puede ser inestable"),
            ]
            versiones_out = []
            for ver, loader, mult, nota in VERSIONES:
                fps = max(20, round((base + core_bonus) * mult))
                versiones_out.append({"version": ver, "loader": loader,
                                      "fps_estimado": fps, "nota": nota})
            return {"ok": True, "versiones": versiones_out}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # ── WHITELIST VISUAL ──────────────────────────────────────────────────
    def get_whitelist(self):
        try:
            if minecraft_launcher_lib:
                mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            else:
                mc_dir = os.path.join(os.environ.get('APPDATA', os.path.expanduser('~')), '.minecraft')
            wl_path = os.path.join(mc_dir, "whitelist.json")
            if not os.path.isfile(wl_path):
                srv_dir = self.config_actual.get("srv_carpeta", "")
                wl_path = os.path.join(srv_dir, "whitelist.json") if srv_dir else wl_path
            if not os.path.isfile(wl_path):
                return {"ok": True, "jugadores": []}
            with open(wl_path, 'r', encoding='utf-8') as f:
                jugadores = json.load(f)
            return {"ok": True, "jugadores": jugadores if isinstance(jugadores, list) else []}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def agregar_whitelist(self, nombre):
        try:
            srv_dir = self.config_actual.get("srv_carpeta", "")
            wl_path = os.path.join(srv_dir, "whitelist.json") if srv_dir else None
            if not wl_path or not os.path.isdir(srv_dir):
                return {"ok": False, "error": "Configurá primero la carpeta del servidor."}
            jugadores = []
            if os.path.isfile(wl_path):
                with open(wl_path, 'r', encoding='utf-8') as f:
                    jugadores = json.load(f)
            if any(j.get("name", j) == nombre for j in jugadores if isinstance(j, dict)):
                return {"ok": False, "error": f"{nombre} ya está en la whitelist."}
            jugadores.append({"uuid": "00000000-0000-0000-0000-000000000000", "name": nombre})
            with open(wl_path, 'w', encoding='utf-8') as f:
                json.dump(jugadores, f, indent=2)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def eliminar_whitelist(self, nombre):
        try:
            srv_dir = self.config_actual.get("srv_carpeta", "")
            wl_path = os.path.join(srv_dir, "whitelist.json") if srv_dir else None
            if not wl_path or not os.path.isfile(wl_path):
                return {"ok": False, "error": "whitelist.json no encontrado."}
            with open(wl_path, 'r', encoding='utf-8') as f:
                jugadores = json.load(f)
            jugadores = [j for j in jugadores
                         if (j.get("name", j) if isinstance(j, dict) else j) != nombre]
            with open(wl_path, 'w', encoding='utf-8') as f:
                json.dump(jugadores, f, indent=2)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def elegir_archivo_zip(self):
        try:
            result = webview.windows[0].create_file_dialog(
                webview.OPEN_DIALOG,
                allow_multiple=False,
                file_types=('ZIP files (*.zip)', 'All files (*.*)')
            )
            if result:
                return {"ok": True, "ruta": result[0]}
            return {"ok": False, "error": "Cancelado"}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def elegir_carpeta_servidor(self):
        try:
            result = webview.windows[0].create_file_dialog(webview.FOLDER_DIALOG)
            carpeta = result[0] if result else None
            if carpeta:
                self.config_actual["srv_carpeta"] = carpeta
                self._guardar()
                return {"ok": True, "carpeta": carpeta}
            return {"ok": False, "error": "Cancelado"}
        except Exception as e:
            return {"ok": False, "error": str(e)}


# ─── LOCAL HTTP API SERVER (bypasses pywebview bridge) ───────────────────────
_api_http_port = 0
_api_http_ref = None
_spotify_pending_code = None

class _LocalAPIHandler(BaseHTTPRequestHandler):
    def log_message(self, *a): pass

    def do_OPTIONS(self):
        self.send_response(200)
        self._cors()
        self.end_headers()

    def do_GET(self):
        global _spotify_pending_code
        import urllib.parse as _up
        parsed = _up.urlparse(self.path)
        qs = _up.parse_qs(parsed.query)

        if parsed.path == '/api/ms_start':
            r = _api_http_ref.login_microsoft_paso1()
            self._json({'ok': r is True, 'error': None if r is True else str(r)})

        elif parsed.path == '/callback':  # Spotify OAuth redirect
            code = qs.get('code', [None])[0]
            if code:
                _spotify_pending_code = code
                html = '<html><body style="background:#121212;color:#1DB954;font-family:sans-serif;text-align:center;padding:60px"><h2>\u2705 Spotify autorizado</h2><p>Ya pod\u00e9s cerrar esta ventana y volver al launcher.</p></body></html>'.encode('utf-8')
            else:
                html = '<html><body style="background:#121212;color:#e74c3c;font-family:sans-serif;text-align:center;padding:60px"><h2>\u274c Error</h2><p>No se recibio el codigo de autorizacion.</p></body></html>'.encode('utf-8')
            self.send_response(200)
            self.send_header('Content-Type', 'text/html; charset=utf-8')
            self.send_header('Content-Length', str(len(html)))
            self.end_headers()
            self.wfile.write(html)

        elif parsed.path == '/api/spotify_auth_start':
            cid = qs.get('cid', [''])[0]
            sec = qs.get('sec', [''])[0]
            if not cid:
                self._json({'ok': False, 'error': 'Falta Client ID'}); return
            _spotify_pending_code = None
            # Save credentials for later
            _api_http_ref.config_actual['spotify_client_id'] = cid
            if sec:
                _api_http_ref.config_actual['spotify_client_secret'] = sec
            _api_http_ref._guardar()
            import urllib.parse as _up2
            scopes = 'user-read-playback-state user-modify-playback-state user-read-currently-playing'
            redirect_uri = f'http://127.0.0.1:{_api_http_port}/callback'
            params = {'client_id': cid, 'response_type': 'code', 'redirect_uri': redirect_uri, 'scope': scopes}
            url = 'https://accounts.spotify.com/authorize?' + _up2.urlencode(params)
            webbrowser.open(url)
            self._json({'ok': True})

        elif parsed.path == '/api/spotify_code_ready':
            if _spotify_pending_code:
                self._json({'ok': True, 'code': _spotify_pending_code})
            else:
                self._json({'ok': False})

        elif parsed.path == '/api/spotify_connect':
            code = qs.get('code', [''])[0]
            cid = _api_http_ref.config_actual.get('spotify_client_id', '')
            sec = _api_http_ref.config_actual.get('spotify_client_secret', '')
            redirect_uri = f'http://127.0.0.1:{_api_http_port}/callback'
            self._json(_api_http_ref._spotify_exchange(cid, sec, code, redirect_uri))

        elif parsed.path == '/api/spotify_nowplaying':
            self._json(_api_http_ref.get_spotify_nowplaying())

        elif parsed.path == '/api/spotify_control':
            accion = qs.get('accion', [''])[0]
            self._json(_api_http_ref.spotify_control(accion))

        elif parsed.path == '/api/get_cuentas':
            self._json(_api_http_ref.get_cuentas() if _api_http_ref else {'ok': False})

        elif parsed.path == '/api/instancia_config':
            folder = qs.get('folder', [''])[0]
            self._json(_api_http_ref.get_instancia_config(folder) if _api_http_ref and folder else {'ok': False})

        elif parsed.path == '/api/leer_server_properties':
            carpeta = qs.get('carpeta', [''])[0]
            self._json(_api_http_ref.leer_server_properties(carpeta) if _api_http_ref and carpeta else {'ok': False})

        elif parsed.path == '/api/whitelist':
            self._json(_api_http_ref.whitelist_list() if _api_http_ref else {'ok': False})

        elif parsed.path == '/api/banlist':
            self._json(_api_http_ref.ban_list() if _api_http_ref else {'ok': False})

        elif parsed.path == '/api/favoritos_srv':
            self._json(_api_http_ref.get_favoritos_srv() if _api_http_ref else {'ok': False})

        elif parsed.path == '/api/screenshots':
            carpeta = qs.get('carpeta', [''])[0]
            self._json(_api_http_ref.get_screenshots_instancia(carpeta) if _api_http_ref and carpeta else {'ok': False})

        elif parsed.path == '/api/stats_instancia':
            carpeta = qs.get('carpeta', [''])[0]
            self._json(_api_http_ref.get_stats_instancia(carpeta) if _api_http_ref and carpeta else {'ok': False})

        elif parsed.path == '/api/get_usuario':
            self._json(_api_http_ref.get_usuario())

        elif parsed.path == '/api/get_settings':
            self._json(_api_http_ref.get_settings())

        elif parsed.path == '/api/game_status':
            self._json(_api_http_ref._game_status if _api_http_ref else {"running": False, "status": "idle"})

        elif parsed.path == '/api/servidor_status':
            if _api_http_ref:
                _api = _api_http_ref
                with _api._servidor_lock:
                    _log = list(_api._servidor_log)
                _carpeta = _api._servidor_carpeta or ""
                self._json({
                    "log": _log,
                    "creando": _api._servidor_creando,
                    "servidor_existe": bool(_carpeta and (
                        os.path.exists(os.path.join(_carpeta, "server.jar")) or
                        os.path.exists(os.path.join(_carpeta, "run.bat"))
                    )),
                    "carpeta": _carpeta,
                    "jugadores": list(_api._jugadores_online),
                    "corriendo": bool(_api._servidor_proc and _api._servidor_proc.poll() is None),
                })
            else:
                self._json({"log": [], "creando": False, "servidor_existe": False, "carpeta": ""})

        elif parsed.path == '/api/skin_proxy':
            name = qs.get('name', ['Steve'])[0].strip() or 'Steve'
            try:
                _sr = requests.get(f'https://minotar.net/skin/{name}', timeout=6)
                if _sr.status_code == 200 and _sr.content:
                    try:
                        from PIL import Image as _PILImg
                        from io import BytesIO as _BIO
                        _img = _PILImg.open(_BIO(_sr.content)).convert('RGBA')
                        _w, _h = _img.size
                        _base_h = min(32, _h)
                        _base = _img.crop((0, 0, _w, _base_h))
                        _white = _PILImg.new('RGBA', (_w, _base_h), (255, 255, 255, 255))
                        _white.alpha_composite(_base)
                        _img.paste(_white, (0, 0))
                        _buf = _BIO()
                        _img.save(_buf, format='PNG')
                        payload = _buf.getvalue()
                    except Exception:
                        payload = _sr.content
                    self.send_response(200)
                    self.send_header('Content-Type', 'image/png')
                    self.send_header('Access-Control-Allow-Origin', '*')
                    self.send_header('Cache-Control', 'public, max-age=300')
                    self.send_header('Content-Length', str(len(payload)))
                    self.end_headers()
                    self.wfile.write(payload)
                else:
                    self.send_response(404); self.end_headers()
            except Exception:
                self.send_response(503); self.end_headers()

        elif parsed.path == '/api/bg_file':
            path = qs.get('path', [''])[0].strip()
            try:
                if not path or not os.path.isfile(path):
                    self.send_response(404); self.end_headers(); return
                ext = path.rsplit('.', 1)[-1].lower()
                mime_map = {'mp4': 'video/mp4', 'webm': 'video/webm',
                            'gif': 'image/gif', 'png': 'image/png',
                            'jpg': 'image/jpeg', 'jpeg': 'image/jpeg'}
                mime = mime_map.get(ext, 'application/octet-stream')
                file_size = os.path.getsize(path)
                range_hdr = self.headers.get('Range', '')
                self.send_response(206 if range_hdr else 200)
                self.send_header('Content-Type', mime)
                self.send_header('Access-Control-Allow-Origin', '*')
                self.send_header('Accept-Ranges', 'bytes')
                self.send_header('Cache-Control', 'no-store')
                if range_hdr and range_hdr.startswith('bytes='):
                    parts = range_hdr[6:].split('-')
                    start = int(parts[0]) if parts[0] else 0
                    end = int(parts[1]) if parts[1] else file_size - 1
                    length = end - start + 1
                    self.send_header('Content-Range', f'bytes {start}-{end}/{file_size}')
                    self.send_header('Content-Length', str(length))
                    self.end_headers()
                    with open(path, 'rb') as fv:
                        fv.seek(start)
                        self.wfile.write(fv.read(length))
                else:
                    self.send_header('Content-Length', str(file_size))
                    self.end_headers()
                    with open(path, 'rb') as fv:
                        self.wfile.write(fv.read())
            except Exception:
                self.send_response(503); self.end_headers()

        elif parsed.path == '/api/skin_local':
            path = qs.get('path', [''])[0].strip()
            try:
                if not path or not os.path.isfile(path):
                    self.send_response(404); self.end_headers(); return
                from PIL import Image as _PILImg
                from io import BytesIO as _BIO
                with open(path, 'rb') as _f:
                    _raw = _f.read()
                try:
                    _img = _PILImg.open(_BIO(_raw)).convert('RGBA')
                    _w, _h = _img.size
                    _base_h = min(32, _h)
                    _base = _img.crop((0, 0, _w, _base_h))
                    _white = _PILImg.new('RGBA', (_w, _base_h), (255, 255, 255, 255))
                    _white.alpha_composite(_base)
                    _img.paste(_white, (0, 0))
                    _buf = _BIO()
                    _img.save(_buf, format='PNG')
                    payload = _buf.getvalue()
                except Exception:
                    payload = _raw
                self.send_response(200)
                self.send_header('Content-Type', 'image/png')
                self.send_header('Access-Control-Allow-Origin', '*')
                self.send_header('Cache-Control', 'no-store')
                self.send_header('Content-Length', str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            except Exception:
                self.send_response(503); self.end_headers()

        else:
            self.send_response(404); self.end_headers()

    def do_POST(self):
        length = int(self.headers.get('Content-Length', 0))
        body = json.loads(self.rfile.read(length) or b'{}')
        if self.path == '/api/cambiar_cuenta':
            self._json(_api_http_ref.cambiar_cuenta(body.get('tipo', 'premium'), int(body.get('idx', 0))) if _api_http_ref else {'ok': False})
        elif self.path == '/api/set_instancia_config':
            self._json(_api_http_ref.set_instancia_config(body.get('folder', ''), body.get('config', {})) if _api_http_ref else {'ok': False})
        elif self.path == '/api/guardar_server_properties':
            self._json(_api_http_ref.guardar_server_properties(body.get('carpeta', ''), body.get('props', {})) if _api_http_ref else {'ok': False})
        elif self.path == '/api/importar_modpack':
            self._json(_api_http_ref.importar_modpack_modrinth(body.get('slug', ''), body.get('version', '')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/modo_invitado':
            self._json(_api_http_ref.modo_invitado(body.get('nombre', '')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/ms_complete':
            self._json(_api_http_ref.login_microsoft_paso2(body.get('url', '')))
        elif self.path == '/api/login_invitado':
            nombre = _api_http_ref.login_invitado(body.get('nombre', ''))
            self._json({'ok': True, 'nombre': nombre})
        elif self.path == '/api/lanzar_juego':
            version = body.get('version', '')
            motor = body.get('motor', '')
            server_ip = body.get('server_ip', '')
            if _api_http_ref and version:
                try:
                    result = _api_http_ref.lanzar_juego(version, motor, server_ip)
                    if result == 'ya_activo':
                        self._json({'ok': False, 'error': 'ya_activo'})
                    else:
                        self._json({'ok': True})
                except Exception as _e:
                    self._json({'ok': False, 'error': str(_e)})
            else:
                self._json({'ok': False, 'error': 'version requerida'})
        elif self.path == '/api/crear_servidor':
            version = body.get('version', '')
            carpeta = body.get('carpeta', '')
            tipo    = body.get('tipo', 'paper')
            if not _api_http_ref or not version or not carpeta:
                self._json({'ok': False, 'error': 'version y carpeta son requeridas'})
            else:
                try:
                    result = _api_http_ref.crear_servidor(version, carpeta, tipo)
                    self._json(result)
                except Exception as _e:
                    self._json({'ok': False, 'error': str(_e)})
        elif self.path == '/api/whitelist_add':
            self._json(_api_http_ref.whitelist_add(body.get('nombre', '')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/whitelist_remove':
            self._json(_api_http_ref.whitelist_remove(body.get('nombre', '')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/ban_add':
            self._json(_api_http_ref.ban_add(body.get('nombre', ''), body.get('razon', '')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/ban_remove':
            self._json(_api_http_ref.ban_remove(body.get('nombre', '')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/toggle_favorito_srv':
            self._json(_api_http_ref.toggle_favorito_srv(body.get('ip', '')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/exportar_instancia':
            self._json(_api_http_ref.exportar_instancia(body.get('carpeta', '')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/importar_instancia_prism':
            self._json(_api_http_ref.importar_instancia_prism(body.get('ruta', '')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/instalar_optimizacion':
            self._json(_api_http_ref.instalar_optimizacion(body.get('tipo', ''), body.get('carpeta', '')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/backup_mundos':
            self._json(_api_http_ref.backup_mundos(body.get('carpeta', '')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/set_reinicio_srv':
            self._json(_api_http_ref.set_reinicio_programado(int(body.get('horas', 0))) if _api_http_ref else {'ok': False})
        elif self.path == '/api/instalar_mod_b64':
            self._json(_api_http_ref.instalar_mod_b64(body.get('nombre', ''), body.get('carpeta', ''), body.get('b64', ''), body.get('tipo', 'mods')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/instalar_shader_preset':
            self._json(_api_http_ref.instalar_shader_preset(body.get('shader_id',''), body.get('version',''), body.get('motor','Fabric')) if _api_http_ref else {'ok':False})
        elif self.path == '/api/aplicar_perfil_hardware':
            self._json(_api_http_ref.aplicar_perfil_hardware(body.get('perfil_id', ''), body.get('version', ''), body.get('motor', 'Fabric'), body.get('estilo', 'base')) if _api_http_ref else {'ok': False})
        elif self.path == '/api/call':
            method = body.get('method', '')
            args = body.get('args', [])
            if not method or not _api_http_ref or not hasattr(_api_http_ref, method) or method.startswith('_'):
                self._json({'ok': False, 'error': f'Method not found: {method}'}); return
            try:
                result = getattr(_api_http_ref, method)(*args)
                if result is None:
                    result = {'ok': True}
                self._json(result)
            except Exception as _ce:
                self._json({'ok': False, 'error': str(_ce)})
        else:
            self.send_response(404); self.end_headers()

    def _cors(self):
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')

    def _json(self, data):
        payload = json.dumps(data).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self._cors()
        self.send_header('Content-Length', str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)


def _start_local_api(api):
    global _api_http_port, _api_http_ref
    _api_http_ref = api
    for candidate in range(9875, 9885):
        try:
            socketserver.ThreadingTCPServer.allow_reuse_address = True
            srv = socketserver.ThreadingTCPServer(('127.0.0.1', candidate), _LocalAPIHandler)
            srv.daemon_threads = True
            _api_http_port = candidate
            threading.Thread(target=srv.serve_forever, daemon=True).start()
            return candidate
        except OSError:
            continue
    raise RuntimeError("No se pudo iniciar el servidor de API local (puertos 9875-9884 ocupados)")
# ──────────────────────────────────────────────────────────────────────────────


if __name__ == "__main__":
    api = Api()
    _base = sys._MEIPASS if getattr(sys, "frozen", False) else os.path.dirname(os.path.abspath(__file__))
    html_path = os.path.join(_base, "web", "index.html")

    try:
        import shutil as _shutil
        _logo_src = os.path.join(_base, "paraguacraft_main_menu.png")
        _logo_dst = os.path.join(_base, "web", "assets", "paraguacraft_logo.png")
        if os.path.exists(_logo_src) and not os.path.exists(_logo_dst):
            _shutil.copy2(_logo_src, _logo_dst)
    except Exception:
        pass

    port = _start_local_api(api)

    ventana = webview.create_window(
        "Paraguacraft Launcher",
        url=html_path,
        js_api=api,
        width=1150,
        height=700,
        min_size=(950, 600),
        background_color="#101010",
    )

    def _on_loaded():
        try:
            ventana.evaluate_js(f'window._API_PORT = {port};')
        except Exception:
            pass
        # Auto-maximize: pywebview native first, then win32gui fallback in a thread
        import threading as _th
        def _do_maximize():
            try:
                ventana.maximize()
            except Exception:
                pass
            import time as _t
            _t.sleep(0.5)
            try:
                import ctypes as _ct
                hwnd = _ct.windll.user32.FindWindowW(None, "Paraguacraft Launcher")
                if hwnd:
                    _ct.windll.user32.ShowWindow(hwnd, 3)
                    return
            except Exception:
                pass
            try:
                import win32gui as _w32g, win32con as _w32c
                maximized = [False]
                def _cb_max(hwnd, _):
                    if maximized[0]:
                        return
                    try:
                        t = _w32g.GetWindowText(hwnd)
                        if "Paraguacraft" in t and _w32g.IsWindowVisible(hwnd):
                            _w32g.ShowWindow(hwnd, _w32c.SW_MAXIMIZE)
                            maximized[0] = True
                    except Exception:
                        pass
                _w32g.EnumWindows(_cb_max, None)
            except Exception:
                pass
        _th.Thread(target=_do_maximize, daemon=True).start()

    ventana.events.loaded += _on_loaded
    _icon_path = os.path.join(_base, "iconomc.ico")
    webview.start(debug=False, http_server=True,
                  icon=_icon_path if os.path.exists(_icon_path) else None,
                  user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
