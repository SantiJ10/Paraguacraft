"""Gestor de Java para Paraguacraft.

Detecta instalaciones de Java en el sistema, valida su versión, y permite
descargar Adoptium (Eclipse Temurin) automáticamente cuando el usuario no
tiene la versión requerida por Minecraft.

API pública (todas thread-safe, no usar webview directamente):
    detectar_javas() -> list[dict]
        [{path, version_major, version_full, vendor, fuente}]
    java_requerido_para_mc(mc_version) -> int (8 | 17 | 21)
    verificar_java(path) -> dict {ok, version_major, version_full, error?}
    elegir_mejor_java(majors_aceptados, javas?) -> dict | None
    descargar_adoptium(major, dest_dir, progress_cb=None) -> dict {ok, path|error}
"""
from __future__ import annotations

import os
import re
import sys
import json
import shutil
import zipfile
import tarfile
import platform
import subprocess
import tempfile
import urllib.request
import urllib.error


# ─── Versión de Java requerida por versión de Minecraft ────────────────────
def java_requerido_para_mc(mc_version: str) -> int:
    """Devuelve el major de Java requerido (8, 17, 21).

    Reglas oficiales de Mojang:
      - <= 1.16.x  → Java 8 (recomendado 8u51+)
      - 1.17.x     → Java 16
      - 1.18 - 1.20.4 → Java 17
      - 1.20.5+    → Java 21
      - 1.21.x+    → Java 21
      - "26.x.x" (snapshots experimentales Paragua) → Java 21
    """
    try:
        v = (mc_version or "").strip()
        if not v:
            return 17
        parts = v.split(".")
        # Snapshots con prefijo numérico no estándar (ej. 26.1.2)
        major = int(parts[0]) if parts[0].isdigit() else 1
        if major >= 23:
            return 21
        if major != 1:
            return 17
        minor = int(parts[1]) if len(parts) > 1 and parts[1].isdigit() else 0
        patch = int(parts[2]) if len(parts) > 2 and parts[2].isdigit() else 0
        if minor <= 16:
            return 8
        if minor == 17:
            return 16
        if minor < 20:
            return 17
        if minor == 20:
            return 21 if patch >= 5 else 17
        return 21  # 1.21+
    except Exception:
        return 17


# ─── Verificar una ruta de Java ────────────────────────────────────────────
_VER_RE = re.compile(r'version "?([0-9._]+)"?')


def verificar_java(path: str) -> dict:
    """Ejecuta `java -version` y parsea el major. No descarga nada."""
    try:
        if not path or not os.path.isfile(path):
            return {"ok": False, "error": "Ruta no existe"}
        # `java -version` escribe a stderr en todas las JDK conocidas
        startupinfo = None
        if os.name == "nt":
            startupinfo = subprocess.STARTUPINFO()
            startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        proc = subprocess.run(
            [path, "-version"],
            capture_output=True, text=True, timeout=8,
            startupinfo=startupinfo,
        )
        out = (proc.stderr or "") + (proc.stdout or "")
        m = _VER_RE.search(out)
        if not m:
            return {"ok": False, "error": "No se pudo leer la versión"}
        ver_str = m.group(1)
        # "1.8.0_281" → 8 ; "17.0.5" → 17 ; "21" → 21
        if ver_str.startswith("1."):
            major = int(ver_str.split(".")[1])
        else:
            major = int(ver_str.split(".")[0])
        vendor = ""
        for line in out.splitlines():
            low = line.lower()
            if "openjdk" in low or "temurin" in low or "adoptium" in low \
               or "graalvm" in low or "zulu" in low or "corretto" in low \
               or "liberica" in low or "hotspot" in low:
                vendor = line.strip()
                break
        return {"ok": True, "version_major": major, "version_full": ver_str,
                "vendor": vendor, "raw": out.strip()[:400]}
    except subprocess.TimeoutExpired:
        return {"ok": False, "error": "Timeout al verificar Java"}
    except Exception as e:
        return {"ok": False, "error": str(e)}


# ─── Detección de Javas instalados ─────────────────────────────────────────
_JAVA_BINS = ("javaw.exe", "java.exe") if os.name == "nt" else ("java",)


def _candidate_paths_windows():
    candidates = []
    pf_dirs = []
    for var in ("ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"):
        v = os.environ.get(var)
        if v and os.path.isdir(v):
            pf_dirs.append(v)
    pf_dirs = list(dict.fromkeys(pf_dirs))  # dedup preservando orden
    sub_brands = ["Java", "Eclipse Adoptium", "Eclipse Foundation",
                  "AdoptOpenJDK", "Microsoft", "Zulu", "Amazon Corretto",
                  "BellSoft", "GraalVM", "Semeru"]
    for pf in pf_dirs:
        for brand in sub_brands:
            root = os.path.join(pf, brand)
            if not os.path.isdir(root):
                continue
            try:
                for entry in os.listdir(root):
                    sub = os.path.join(root, entry, "bin")
                    if os.path.isdir(sub):
                        for b in _JAVA_BINS:
                            p = os.path.join(sub, b)
                            if os.path.isfile(p):
                                candidates.append((p, f"system:{brand}"))
            except Exception:
                continue
    # Mojang/Minecraft Launcher runtimes
    appdata = os.environ.get("APPDATA", "")
    if appdata:
        mojang_runtime = os.path.join(appdata, ".minecraft", "runtime")
        if os.path.isdir(mojang_runtime):
            for root, dirs, files in os.walk(mojang_runtime):
                for b in _JAVA_BINS:
                    if b in files:
                        candidates.append((os.path.join(root, b), "mojang"))
                        break
                # Limitar profundidad
                depth = root[len(mojang_runtime):].count(os.sep)
                if depth >= 5:
                    dirs[:] = []
    # JAVA_HOME
    jh = os.environ.get("JAVA_HOME", "")
    if jh:
        for b in _JAVA_BINS:
            p = os.path.join(jh, "bin", b)
            if os.path.isfile(p):
                candidates.append((p, "JAVA_HOME"))
    # Adoptium descargado por nosotros
    paragua_data = os.path.join(os.environ.get("APPDATA", ""), "ParaguacraftLauncher", "java")
    if os.path.isdir(paragua_data):
        for root, dirs, files in os.walk(paragua_data):
            for b in _JAVA_BINS:
                if b in files:
                    candidates.append((os.path.join(root, b), "paraguacraft"))
                    break
            if root[len(paragua_data):].count(os.sep) >= 4:
                dirs[:] = []
    return candidates


def _candidate_paths_unix():
    candidates = []
    for base in ("/usr/lib/jvm", "/usr/java", "/opt", "/Library/Java/JavaVirtualMachines"):
        if not os.path.isdir(base):
            continue
        try:
            for entry in os.listdir(base):
                for sub in (
                    os.path.join(base, entry, "bin", "java"),
                    os.path.join(base, entry, "Contents", "Home", "bin", "java"),
                    os.path.join(base, entry, "jre", "bin", "java"),
                ):
                    if os.path.isfile(sub):
                        candidates.append((sub, f"system:{base}"))
        except Exception:
            continue
    jh = os.environ.get("JAVA_HOME", "")
    if jh:
        p = os.path.join(jh, "bin", "java")
        if os.path.isfile(p):
            candidates.append((p, "JAVA_HOME"))
    paragua_data = os.path.expanduser("~/.config/ParaguacraftLauncher/java")
    if os.path.isdir(paragua_data):
        for root, dirs, files in os.walk(paragua_data):
            if "java" in files:
                candidates.append((os.path.join(root, "java"), "paraguacraft"))
            if root[len(paragua_data):].count(os.sep) >= 4:
                dirs[:] = []
    return candidates


def detectar_javas() -> list:
    """Lista todos los Javas detectables. No verifica versión todavía
    (eso es costoso). El consumidor puede llamar `verificar_java` después."""
    seen = set()
    raw = _candidate_paths_windows() if os.name == "nt" else _candidate_paths_unix()
    # Agregar PATH
    path_java = shutil.which("java")
    if path_java:
        raw.append((path_java, "PATH"))
    out = []
    for p, fuente in raw:
        try:
            real = os.path.normcase(os.path.realpath(p))
        except Exception:
            real = os.path.normcase(p)
        if real in seen:
            continue
        seen.add(real)
        info = verificar_java(p)
        if info.get("ok"):
            out.append({
                "path": p,
                "version_major": info["version_major"],
                "version_full": info["version_full"],
                "vendor": info.get("vendor", ""),
                "fuente": fuente,
            })
    # Ordenar por (fuente preferida, versión)
    fuente_rank = {"paraguacraft": 0, "mojang": 1, "JAVA_HOME": 2,
                   "PATH": 3}
    out.sort(key=lambda j: (fuente_rank.get(j["fuente"].split(":")[0], 9),
                            -j["version_major"]))
    return out


def elegir_mejor_java(majors_aceptados, javas=None):
    """Devuelve el primer Java cuya version_major esté en majors_aceptados.
    Prioriza coincidencia exacta, después la versión más alta compatible."""
    javas = javas if javas is not None else detectar_javas()
    if not javas:
        return None
    aceptados = set(majors_aceptados) if not isinstance(majors_aceptados, int) else {majors_aceptados}
    # Coincidencia exacta primero
    for j in javas:
        if j["version_major"] in aceptados:
            return j
    return None


# ─── Descarga Adoptium ─────────────────────────────────────────────────────
def _adoptium_os():
    s = platform.system().lower()
    if s == "windows":
        return "windows"
    if s == "darwin":
        return "mac"
    return "linux"


def _adoptium_arch():
    m = platform.machine().lower()
    if m in ("amd64", "x86_64"):
        return "x64"
    if m in ("i386", "i686", "x86"):
        return "x86"
    if m in ("arm64", "aarch64"):
        return "aarch64"
    return "x64"


def descargar_adoptium(major: int, dest_dir: str, progress_cb=None) -> dict:
    """Descarga el JRE de Adoptium Temurin para `major` y lo extrae en dest_dir.
    Devuelve {ok, path: <ruta a javaw|java>, error?}.
    `progress_cb(pct: int, etiqueta: str)` se invoca periódicamente."""
    def _pcb(pct, lbl):
        if progress_cb:
            try: progress_cb(int(pct), lbl)
            except Exception: pass

    try:
        os.makedirs(dest_dir, exist_ok=True)
        url = (f"https://api.adoptium.net/v3/binary/latest/{int(major)}/ga/"
               f"{_adoptium_os()}/{_adoptium_arch()}/jre/hotspot/normal/eclipse")
        _pcb(0, f"Conectando con Adoptium Java {major}...")
        # La API redirige al archivo final
        req = urllib.request.Request(url, headers={"User-Agent": "ParaguacraftLauncher/3.0"})
        # Detectar tipo de archivo desde la URL final
        with urllib.request.urlopen(req, timeout=30) as resp:
            final_url = resp.geturl()
            total = int(resp.headers.get("Content-Length", 0) or 0)
            ext = ".zip" if final_url.endswith(".zip") else ".tar.gz"
            tmp_path = os.path.join(tempfile.gettempdir(), f"paragua_jre_{major}{ext}")
            downloaded = 0
            chunk = 64 * 1024
            with open(tmp_path, "wb") as f:
                while True:
                    data = resp.read(chunk)
                    if not data:
                        break
                    f.write(data)
                    downloaded += len(data)
                    if total > 0:
                        _pcb(downloaded * 80 / total, f"Descargando Java {major}: {downloaded // (1024*1024)} MB")
        _pcb(85, f"Extrayendo Java {major}...")

        # Extraer al destino
        extract_root = os.path.join(dest_dir, f"jre-{major}")
        if os.path.isdir(extract_root):
            shutil.rmtree(extract_root, ignore_errors=True)
        os.makedirs(extract_root, exist_ok=True)
        try:
            if ext == ".zip":
                with zipfile.ZipFile(tmp_path) as z:
                    z.extractall(extract_root)
            else:
                with tarfile.open(tmp_path, "r:gz") as t:
                    t.extractall(extract_root)
        finally:
            try: os.remove(tmp_path)
            except Exception: pass

        # Buscar el binario java(w)
        bin_name = "javaw.exe" if os.name == "nt" else "java"
        java_path = ""
        for root, dirs, files in os.walk(extract_root):
            if bin_name in files:
                java_path = os.path.join(root, bin_name)
                break
        if not java_path:
            return {"ok": False, "error": "No se encontró el binario tras extraer"}
        _pcb(100, f"Java {major} listo")
        return {"ok": True, "path": java_path}
    except urllib.error.HTTPError as e:
        return {"ok": False, "error": f"HTTP {e.code} de Adoptium"}
    except urllib.error.URLError as e:
        return {"ok": False, "error": f"Sin conexión: {e.reason}"}
    except Exception as e:
        return {"ok": False, "error": str(e)}


# ─── Hardware info consolidado ─────────────────────────────────────────────
def get_hardware_info() -> dict:
    """Devuelve información de hardware útil para wizard / perfiles.
    Usa psutil si está disponible, con fallbacks."""
    info = {
        "ram_gb": 0.0,
        "cpu_cores": 0,
        "cpu_threads": 0,
        "cpu_name": "",
        "os": platform.system(),
        "os_version": platform.version(),
        "os_release": platform.release(),
        "arch": platform.machine(),
    }
    try:
        import psutil
        info["ram_gb"] = round(psutil.virtual_memory().total / (1024 ** 3), 1)
        info["cpu_cores"] = psutil.cpu_count(logical=False) or 0
        info["cpu_threads"] = psutil.cpu_count(logical=True) or 0
    except Exception:
        try:
            info["cpu_threads"] = os.cpu_count() or 0
        except Exception:
            pass
    # Nombre del CPU (Windows: registry; Linux: /proc/cpuinfo; Mac: sysctl)
    try:
        if os.name == "nt":
            import winreg
            key = winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE,
                                 r"HARDWARE\DESCRIPTION\System\CentralProcessor\0")
            info["cpu_name"] = winreg.QueryValueEx(key, "ProcessorNameString")[0].strip()
            winreg.CloseKey(key)
        elif sys.platform == "darwin":
            info["cpu_name"] = subprocess.check_output(
                ["sysctl", "-n", "machdep.cpu.brand_string"], timeout=2
            ).decode().strip()
        else:
            with open("/proc/cpuinfo") as f:
                for line in f:
                    if line.startswith("model name"):
                        info["cpu_name"] = line.split(":", 1)[1].strip()
                        break
    except Exception:
        pass
    # Sugerencia de perfil basada en RAM y cores
    if info["ram_gb"] <= 8 or info["cpu_cores"] <= 4:
        info["perfil_sugerido"] = "baja"
    elif info["ram_gb"] <= 16 or info["cpu_cores"] <= 8:
        info["perfil_sugerido"] = "media"
    else:
        info["perfil_sugerido"] = "alta"
    return info
