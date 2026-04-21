"""
test_smoke.py — Prueba rápida de sanidad para Paraguacraft Launcher.

Ejecutar con:
    py -3 test_smoke.py

NO abre la ventana del launcher. Prueba el backend puro.
Resultados también se escriben en el log de AppData.
"""

import sys
import os
import time
import traceback

# ── asegurar que los módulos del proyecto son encontrables ──────────────────
_HERE = os.path.dirname(os.path.abspath(__file__))
if _HERE not in sys.path:
    sys.path.insert(0, _HERE)

# ── activar el logger centralizado antes de todo ────────────────────────────
try:
    from src.logger import setup_logger
    setup_logger()
except Exception as _le:
    import logging
    logging.basicConfig(level=logging.DEBUG)

import logging
log = logging.getLogger("paraguacraft.test_smoke")

# ── helpers de reporte ───────────────────────────────────────────────────────
_RESULTADOS = []

def _ok(nombre):
    msg = f"  [PASS] {nombre}"
    print(msg); log.info("[SMOKE] PASS — %s", nombre)
    _RESULTADOS.append(("PASS", nombre))

def _fail(nombre, detalle=""):
    msg = f"  [FAIL] {nombre}"
    if detalle:
        msg += f"\n         {detalle}"
    print(msg); log.error("[SMOKE] FAIL — %s — %s", nombre, detalle)
    _RESULTADOS.append(("FAIL", nombre, detalle))

def _seccion(titulo):
    sep = "─" * 60
    print(f"\n{sep}\n  {titulo}\n{sep}")
    log.info("[SMOKE] ── %s ──", titulo)

def _cronometrar(fn):
    t0 = time.monotonic()
    try:
        resultado = fn()
        return True, resultado, time.monotonic() - t0
    except Exception as e:
        return False, traceback.format_exc(), time.monotonic() - t0


# ════════════════════════════════════════════════════════════════════════════
# 1. IMPORTACIONES
# ════════════════════════════════════════════════════════════════════════════
_seccion("1 · Importaciones de módulos")

for _mod in ["requests", "psutil", "minecraft_launcher_lib", "webview"]:
    try:
        __import__(_mod)
        _ok(f"import {_mod}")
    except ImportError as e:
        _fail(f"import {_mod}", str(e))

try:
    import core
    _ok("import core")
except Exception as e:
    _fail("import core", str(e))

try:
    from src.modelo import GestorMods, TiendaAPI, CreadorServidor
    _ok("import src.modelo (GestorMods, TiendaAPI, CreadorServidor)")
except Exception as e:
    _fail("import src.modelo", str(e))

try:
    from src.logger import setup_logger, LOG_PATH
    _ok(f"import src.logger → log en: {LOG_PATH}")
except Exception as e:
    _fail("import src.logger", str(e))


# ════════════════════════════════════════════════════════════════════════════
# 2. INICIALIZACIÓN DE Api()
# ════════════════════════════════════════════════════════════════════════════
_seccion("2 · Inicialización de Api()")

# Monkey-patch webview para que Api() no explote sin ventana
import types
_fake_wv = types.ModuleType("webview")
_fake_wv.windows = []
_fake_wv.create_window = lambda *a, **kw: None
_fake_wv.start = lambda *a, **kw: None
sys.modules.setdefault("webview", _fake_wv)

try:
    # Importar sin arrancar la ventana
    import importlib, paragua as _pg
    api = _pg.Api()
    _ok("Api() instanciada")
except Exception as e:
    _fail("Api() instanciada", traceback.format_exc())
    api = None

if api:
    try:
        assert isinstance(api.config_actual, dict)
        _ok("config_actual cargado como dict")
    except Exception as e:
        _fail("config_actual", str(e))

    try:
        from src.logger import LOG_PATH as _lp
        assert os.path.exists(os.path.dirname(_lp)), "AppData dir no existe"
        _ok(f"Directorio AppData accesible: {os.path.dirname(_lp)}")
    except Exception as e:
        _fail("Directorio AppData", str(e))


# ════════════════════════════════════════════════════════════════════════════
# 3. ARCHIVOS DE CONFIGURACIÓN
# ════════════════════════════════════════════════════════════════════════════
_seccion("3 · Archivos de configuración")

if api:
    try:
        api._guardar()
        import json
        with open(api.ruta_config) as f:
            data = json.load(f)
        assert "usuario" in data
        _ok("_guardar() y relectura de config.json")
    except Exception as e:
        _fail("_guardar() config", str(e))

    try:
        assert hasattr(api, "_guardar_lock"), "Falta _guardar_lock"
        assert hasattr(api, "_launch_lock"), "Falta _launch_lock"
        _ok("Locks de concurrencia presentes (_guardar_lock, _launch_lock)")
    except AssertionError as e:
        _fail("Locks de concurrencia", str(e))


# ════════════════════════════════════════════════════════════════════════════
# 4. RED — APIs externas
# ════════════════════════════════════════════════════════════════════════════
_seccion("4 · Red — APIs externas (timeout 10 s)")

# 4a. Lista de versiones de Mojang
def _test_mojang():
    import minecraft_launcher_lib
    vl = minecraft_launcher_lib.utils.get_version_list()
    assert len(vl) > 0, "Lista vacía"
    return len(vl)

ok, res, t = _cronometrar(_test_mojang)
if ok:
    _ok(f"Mojang version list ({res} versiones) — {t:.1f}s")
else:
    _fail("Mojang version list", res[:300])

# 4b. FabricMC loader versions
def _test_fabric():
    import requests as _r
    resp = _r.get("https://meta.fabricmc.net/v2/versions/loader", timeout=10)
    resp.raise_for_status()
    data = resp.json()
    assert len(data) > 0, "Lista vacía"
    return data[0]["version"]

ok, res, t = _cronometrar(_test_fabric)
if ok:
    _ok(f"FabricMC loaders (último: {res}) — {t:.1f}s")
else:
    _fail("FabricMC loaders", str(res)[:300])

# 4c. Modrinth búsqueda
def _test_modrinth():
    from src.modelo import TiendaAPI
    ok2, hits = TiendaAPI.buscar_mods("sodium", "1.21.1", "Fabric", "mod")
    assert ok2, f"Búsqueda falló: {hits}"
    assert isinstance(hits, list) and len(hits) > 0, "Sin resultados"
    return hits[0].get("title", "?")

ok, res, t = _cronometrar(_test_modrinth)
if ok:
    _ok(f"Modrinth búsqueda 'sodium' → primer resultado: '{res}' — {t:.1f}s")
else:
    _fail("Modrinth búsqueda", str(res)[:300])

# 4d. PaperMC API
def _test_papermc():
    import requests as _r
    resp = _r.get("https://api.papermc.io/v2/projects/paper/versions/1.21.1/builds", timeout=10)
    resp.raise_for_status()
    builds = resp.json().get("builds", [])
    assert len(builds) > 0, "Sin builds"
    return builds[-1]["build"]

ok, res, t = _cronometrar(_test_papermc)
if ok:
    _ok(f"PaperMC API (build más reciente 1.21.1: #{res}) — {t:.1f}s")
else:
    _fail("PaperMC API", str(res)[:300])

# 4e. GitHub Releases (updater)
def _test_github():
    import requests as _r
    resp = _r.get(
        "https://api.github.com/repos/SantiJ10/Paraguacraft/releases/latest",
        timeout=10,
        headers={"User-Agent": "Paraguacraft-Smoke-Test"},
    )
    resp.raise_for_status()
    tag = resp.json().get("tag_name", "?")
    return tag

ok, res, t = _cronometrar(_test_github)
if ok:
    _ok(f"GitHub Releases API (última versión: {res}) — {t:.1f}s")
else:
    _fail("GitHub Releases API", str(res)[:300])


# ════════════════════════════════════════════════════════════════════════════
# 5. DIRECTORIOS DE MINECRAFT
# ════════════════════════════════════════════════════════════════════════════
_seccion("5 · Directorios de Minecraft")

try:
    import minecraft_launcher_lib
    mc_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
    _ok(f"MC directory: {mc_dir}")
    inst_dir = os.path.join(mc_dir, "instancias")
    if os.path.isdir(inst_dir):
        instancias = os.listdir(inst_dir)
        _ok(f"Instancias encontradas: {len(instancias)} → {instancias[:5]}")
    else:
        _ok("Carpeta 'instancias' aún no existe (normal en primer uso)")
except Exception as e:
    _fail("MC directory", str(e))

try:
    from core import carpeta_instancia_paraguacraft
    nombre = carpeta_instancia_paraguacraft("1.21.1", "Fabric")
    assert nombre == "Paraguacraft_1.21.1_Fabric", f"Nombre inesperado: {nombre}"
    _ok(f"carpeta_instancia_paraguacraft → '{nombre}'")
except Exception as e:
    _fail("carpeta_instancia_paraguacraft", str(e))


# ════════════════════════════════════════════════════════════════════════════
# 6. JAVA — Verificar disponibilidad
# ════════════════════════════════════════════════════════════════════════════
_seccion("6 · Java disponible")

import subprocess as _sp

for _java in ["java", "javaw"]:
    try:
        r = _sp.run([_java, "-version"], capture_output=True, timeout=5)
        ver_line = (r.stderr or r.stdout or b"").decode(errors="replace").split("\n")[0]
        _ok(f"{_java} → {ver_line.strip()}")
        break
    except FileNotFoundError:
        _fail(f"{_java} no encontrado en PATH")
    except Exception as e:
        _fail(f"{_java} error", str(e))


# ════════════════════════════════════════════════════════════════════════════
# 7. ARCHIVO DE LOG
# ════════════════════════════════════════════════════════════════════════════
_seccion("7 · Archivo de log")

try:
    from src.logger import LOG_PATH
    log.info("[SMOKE] Prueba de escritura en log")
    # Forzar flush de todos los handlers
    for h in logging.getLogger("paraguacraft").handlers:
        h.flush()
    assert os.path.exists(LOG_PATH), "Archivo de log no creado"
    size = os.path.getsize(LOG_PATH)
    _ok(f"Log en disco: {LOG_PATH} ({size} bytes)")
except Exception as e:
    _fail("Archivo de log", str(e))


# ════════════════════════════════════════════════════════════════════════════
# 8. ESCRITURA ATÓMICA DE CONFIG
# ════════════════════════════════════════════════════════════════════════════
_seccion("8 · Escritura atómica de config (simular corrupción)")

if api:
    try:
        import json, tempfile
        original_usuario = api.config_actual.get("usuario", "Invitado")
        api.config_actual["usuario"] = "__smoke_test__"
        api._guardar()
        with open(api.ruta_config) as f:
            leido = json.load(f)
        assert leido.get("usuario") == "__smoke_test__", "Valor no guardado"
        # Restaurar
        api.config_actual["usuario"] = original_usuario
        api._guardar()
        _ok("Escritura atómica de config — round-trip OK")
    except Exception as e:
        _fail("Escritura atómica de config", str(e))


# ════════════════════════════════════════════════════════════════════════════
# RESUMEN FINAL
# ════════════════════════════════════════════════════════════════════════════
print("\n" + "═" * 60)
total = len(_RESULTADOS)
fallos = [r for r in _RESULTADOS if r[0] == "FAIL"]
pasados = total - len(fallos)
print(f"  RESULTADO: {pasados}/{total} pruebas pasaron")
if fallos:
    print(f"\n  FALLOS ({len(fallos)}):")
    for f in fallos:
        print(f"    ✗ {f[1]}")
        if len(f) > 2 and f[2]:
            for linea in f[2].strip().split("\n")[:3]:
                print(f"      {linea}")
else:
    print("  ✓ Todo en orden — backend OK para compilar")
print("═" * 60)

log.info("[SMOKE] Resultado final: %d/%d pasaron — %d fallos", pasados, total, len(fallos))

sys.exit(0 if not fallos else 1)
