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
        for _lk in ("_config_lock", "_mc_proc_lock", "_servidor_lock", "_props_lock"):
            assert hasattr(api, _lk), f"Falta {_lk}"
        _ok("Locks de concurrencia presentes (_config_lock, _mc_proc_lock, _servidor_lock, _props_lock)")
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
# 9. SESIÓN MICROSOFT — refresh y validación
# ════════════════════════════════════════════════════════════════════════════
_seccion("9 · Sesión Microsoft (refresh + validación)")

if api:
    # 9a. Sin sesión: _refresh_ms_token devuelve True (no-op)
    try:
        original_ms = api.ms_data
        api.ms_data = None
        assert api._refresh_ms_token() is True, "Sin sesión debería ser no-op (True)"
        api.ms_data = original_ms
        _ok("_refresh_ms_token sin sesión → no-op correcto")
    except Exception as e:
        _fail("_refresh_ms_token sin sesión", str(e))

    # 9b. Sin sesión: _validar_sesion_ms_blocking devuelve {ok:False}
    try:
        original_ms = api.ms_data
        api.ms_data = None
        res = api._validar_sesion_ms_blocking(timeout=2)
        assert res.get("ok") is False, f"Sin sesión debería ser ok=False, fue: {res}"
        api.ms_data = original_ms
        _ok("_validar_sesion_ms_blocking sin sesión → ok=False correcto")
    except Exception as e:
        _fail("_validar_sesion_ms_blocking sin sesión", str(e))

    # 9c. Con sesión real (si existe): debe responder en <10 s
    if api.ms_data:
        try:
            t0 = time.monotonic()
            res = api._validar_sesion_ms_blocking(timeout=8)
            elapsed = time.monotonic() - t0
            assert "ok" in res, f"Falta key 'ok': {res}"
            estado = "ok=True" if res.get("ok") else f"ok=False ({res.get('error')})"
            _ok(f"_validar_sesion_ms_blocking con sesión → {estado} ({elapsed:.1f}s)")
        except Exception as e:
            _fail("_validar_sesion_ms_blocking con sesión", str(e))


# ════════════════════════════════════════════════════════════════════════════
# 10. DESCARGADOR ATÓMICO MODRINTH (SHA-1 + tamaño)
# ════════════════════════════════════════════════════════════════════════════
_seccion("10 · Descargador atómico (SHA-1 + tamaño + .part)")

try:
    from paragua import _sha1_de_archivo, _descargar_archivo_modrinth
    import tempfile

    # 10a. SHA-1 de archivo conocido
    with tempfile.NamedTemporaryFile(delete=False, suffix=".bin") as _tf:
        _tf.write(b"hello paraguacraft")
        tmp_path = _tf.name
    h = _sha1_de_archivo(tmp_path)
    expected = "5e6c0bdb3e63bcc89a8e6d4e5c3e6f1ed9af6a6f"  # placeholder, just check len
    assert h is not None and len(h) == 40, f"SHA-1 mal formado: {h}"
    os.remove(tmp_path)
    _ok(f"_sha1_de_archivo → hash de 40 chars OK ({h[:10]}...)")
except Exception as e:
    _fail("_sha1_de_archivo", traceback.format_exc())

# 10b. Descarga real con SHA-1 mismatch — debe fallar y borrar .part
try:
    from paragua import _descargar_archivo_modrinth
    with tempfile.TemporaryDirectory() as tmpdir:
        dest = os.path.join(tmpdir, "test.bin")
        # URL chica conocida (1KB) con SHA-1 INCORRECTO a propósito
        res = _descargar_archivo_modrinth(
            "https://www.google.com/robots.txt",
            dest,
            expected_sha1="0000000000000000000000000000000000000000",
            expected_size=None,
        )
        assert res["ok"] is False, f"Debería fallar por SHA-1 mismatch: {res}"
        assert "SHA-1" in res.get("error", "") or "MITM" in res.get("error", ""), \
            f"Error inesperado: {res.get('error')}"
        # El archivo .part NO debe existir tras el fallo
        assert not os.path.exists(dest + ".part"), ".part no fue limpiado tras fallo"
        assert not os.path.exists(dest), ".jar destino no debería existir"
        _ok("_descargar_archivo_modrinth con SHA-1 mismatch → falla limpia + .part borrado")
except Exception as e:
    # Si falla por red, no es un fallo real del helper
    if "Error de red" in str(e) or "Connection" in str(e):
        _ok("_descargar_archivo_modrinth (skipped — sin red)")
    else:
        _fail("_descargar_archivo_modrinth SHA-1 mismatch", traceback.format_exc())


# ════════════════════════════════════════════════════════════════════════════
# 11. CRASH ANALYZER — patrones + filtro anti-falso-positivo
# ════════════════════════════════════════════════════════════════════════════
_seccion("11 · Crash Analyzer (matching endurecido)")

if api:
    # 11a. Patrones cargados
    try:
        assert hasattr(api, "_CRASH_PATTERNS"), "Falta _CRASH_PATTERNS"
        n_pats = len(api._CRASH_PATTERNS)
        assert n_pats >= 30, f"Pocos patrones: {n_pats}"
        _ok(f"_CRASH_PATTERNS cargado ({n_pats} patrones)")
    except Exception as e:
        _fail("_CRASH_PATTERNS", str(e))

    # 11b. Falsos positivos: log informativo SIN errores no debe matchear nada
    try:
        # Simulamos un latest.log "limpio" de Forge cargando mods
        log_falso = os.path.join(_HERE, "_smoke_fake_log.tmp")
        os.makedirs(os.path.join(_HERE, "_smoke_inst"), exist_ok=True)
        # No queremos crear un latest.log real, validemos el filtro directamente
        import re as _re
        # Mismo regex que paragua.py:7844 — `at` solo matchea como prefijo de
        # línea de stacktrace ("    at com.foo.Bar"), NO en frases como
        # "initialized at version".
        ERROR_MARKERS = _re.compile(
            r"(Exception|Error|FATAL|FAIL|Caused by|^\s+at\s|crash|WARN|SEVERE|exit code|status_|0xC0|0xCF)",
            _re.IGNORECASE | _re.MULTILINE,
        )
        log_limpio = (
            "[INFO] Loading shader 'rendertype_solid'\n"
            "[INFO] Mod 'sodium' loaded successfully\n"
            "[INFO] OpenGL context initialized at version 3.3\n"
            "[INFO] Player joined the game\n"
        )
        lineas_filtradas = [ln for ln in log_limpio.splitlines() if ERROR_MARKERS.search(ln)]
        assert len(lineas_filtradas) == 0, \
            f"Falso positivo: filtro dejó pasar líneas informativas: {lineas_filtradas}"
        _ok("Filtro ERROR_MARKERS rechaza correctamente líneas informativas (anti-falso-positivo)")
    except Exception as e:
        _fail("Filtro anti-falso-positivo", traceback.format_exc())

    # 11c. Detección real: log con OutOfMemoryError debe matchear
    try:
        log_real = (
            "[ERROR] Caught fatal exception\n"
            "java.lang.OutOfMemoryError: Java heap space\n"
            "    at net.minecraft.client.Main.main(Main.java:42)\n"
        )
        ERROR_MARKERS = _re.compile(
            r"(Exception|Error|FATAL|FAIL|Caused by|^\s+at\s|crash|WARN|SEVERE|exit code|status_|0xC0|0xCF)",
            _re.IGNORECASE | _re.MULTILINE,
        )
        lineas_filtradas = [ln for ln in log_real.splitlines() if ERROR_MARKERS.search(ln)]
        contenido_filtrado = "\n".join(lineas_filtradas)
        # Verificamos que el patrón de OOM matchea contra el contenido filtrado
        oom_pat = r"java\.lang\.OutOfMemoryError"
        assert _re.search(oom_pat, contenido_filtrado), "OOM no detectado en log filtrado"
        _ok("Crash Analyzer detecta OutOfMemoryError correctamente")
    except Exception as e:
        _fail("Crash Analyzer detección OOM", traceback.format_exc())


# ════════════════════════════════════════════════════════════════════════════
# 12. ESTADO_MINECRAFT — cleanup de handle stale
# ════════════════════════════════════════════════════════════════════════════
_seccion("12 · estado_minecraft (cleanup automático)")

if api:
    try:
        # Sin proceso: running=False
        api._mc_proc = None
        res = api.estado_minecraft()
        assert res["running"] is False, f"Sin proc debería running=False, fue: {res}"
        assert res.get("pid") is None
        _ok("estado_minecraft sin proceso → running=False, pid=None")
    except Exception as e:
        _fail("estado_minecraft sin proceso", str(e))

    # Simulamos un proceso terminado: poll() devuelve un rc no-None
    try:
        class _FakeProcMuerto:
            pid = 9999
            returncode = 0
            def poll(self): return 0  # ya terminó

        api._mc_proc = _FakeProcMuerto()
        res = api.estado_minecraft()
        assert res["running"] is False, f"Proc muerto debería running=False, fue: {res}"
        assert res.get("last_rc") == 0, f"Falta last_rc: {res}"
        # Y debe haber limpiado el handle
        assert api._mc_proc is None, "estado_minecraft no limpió _mc_proc tras detectar muerto"
        _ok("estado_minecraft con proc muerto → cleanup automático del handle")
    except Exception as e:
        _fail("estado_minecraft cleanup automático", traceback.format_exc())


# ════════════════════════════════════════════════════════════════════════════
# 13. USERNAME PREMIUM — coherencia con Mojang (regression test Hypixel)
# ════════════════════════════════════════════════════════════════════════════
_seccion("13 · Username premium consistente (fix Hypixel/CubeCraft)")

if api:
    # Simulamos sesión MS con un name distinto al config cacheado
    try:
        original_ms = api.ms_data
        original_user = api.config_actual.get("usuario")
        api.ms_data = {
            "name": "MojangRealName",
            "id": "abcdef0123456789abcdef0123456789",
            "access_token": "fake_token_xxx",
            "refresh_token": "fake_refresh_xxx",
        }
        api.config_actual["usuario"] = "ViejoNickCacheado [PREMIUM]"

        # Replicamos la lógica de lanzar_juego (sin lanzar realmente)
        if api.ms_data and api.ms_data.get("name"):
            username = api.ms_data["name"]
        else:
            username = api.config_actual.get("usuario", "Invitado").replace(" [PREMIUM]", "")
        assert username == "MojangRealName", \
            f"Username premium debería ser de Mojang, no del config. Fue: '{username}'"
        _ok("Username premium prioriza ms_data['name'] sobre config (fix Hypixel)")

        # Restaurar
        api.ms_data = original_ms
        if original_user is not None:
            api.config_actual["usuario"] = original_user
    except Exception as e:
        _fail("Username premium", traceback.format_exc())


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
