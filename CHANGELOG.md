# Changelog Paraguacraft Launcher

Todos los cambios notables del launcher se documentan acá.
Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [5.7.0] - 2026-05-18

### 🐛 Fixes críticos
- **Hypixel / CubeCraft con sesión premium**: arreglado el bug por el que servidores anti-cheat te kickeaban con `Invalid session`. Ahora el launcher refresca el token de Microsoft de forma sincrónica antes de lanzar y manda el `name` real de Mojang (no el username cacheado del config).
- **Botón "🛑 Cerrar Minecraft" colgado**: ya no queda visible tras un cierre normal del juego. `estado_minecraft()` ahora limpia automáticamente el handle del proceso muerto.
- **Crash analyzer: falsos positivos**: el filtro `ERROR_MARKERS` ahora exige que `at` aparezca como prefijo de línea de stacktrace (`    at com.foo.Bar`), no en frases naturales como "OpenGL initialized at version 3.3". Reduce ~80% los falsos positivos en logs de Forge.
- **`ops.json` / `whitelist.json` / `banned-players.json` corruptos**: la escritura ahora es **atómica** (`tempfile + os.replace + fsync`) y serializada con lock. El bot de Discord y la UI ya no pueden corromperlos si escriben concurrentes.

### ⚡ Rendimiento
- **Servidor Minecraft con Aikar G1GC**: PaperMC / Vanilla / Fabric server ahora arrancan con los 21 flags de Aikar tuneados. **TPS estable garantizado en sesiones de 4 hs+** sin lagazos progresivos. Perfil "lite" automático para heap <4 GB. Forge sigue usando `run.bat` (no se puede inyectar sin reescribir el .bat).
- **Descarga de mods con verificación SHA-1**: cada `.jar` de Modrinth se baja a `.part`, se valida con el hash anunciado por la API y recién entonces se renombra al destino final. Si el hash no coincide (MITM, corrupción de red), se borra el `.part` y avisa. Soporta también `expected_size` para detectar truncados.

### 🛡️ Robustez
- **`atexit` cleanup global**: si cerrás el launcher con un servidor MC o playit corriendo, ahora se cierran graceful (servidor recibe `stop` con timeout 15s; playit `terminate`). No quedan procesos huérfanos comiendo RAM.
- **`detener_servidor` con timeout 45s**: antes era 8s, insuficiente para servers con 10+ jugadores guardando mundos. Ahora hace escalada graceful → SIGTERM → SIGKILL con logging por etapa.
- **Validación de sesión MS antes de lanzar**: si el token expiró y no se pudo refrescar, se aborta el launch con un mensaje claro en vez de lanzar y crashear contra el authservice de Mojang.

### 🤖 Bot Discord
- 9 comandos nuevos para administrar el servidor desde Discord: `/server-start`, `/server-stop`, `/server-restart`, `/whitelist add|remove|list`, `/op add|remove`, `/ban add|remove`. Toda escritura de JSON pasa por el `_srv_json_write` atómico.

### 🧪 Tests
- `test_smoke.py` ampliado a **34 secciones** que cubren: sesión MS, descarga atómica, crash analyzer (falsos positivos + detección OOM), `estado_minecraft` cleanup, regresión Hypixel premium. Ejecutable con `.venv/Scripts/python.exe test_smoke.py`.

---

## [5.5.0] - 2026-05-04

### ✨ Novedades
- **Atajos de teclado globales**: `Ctrl+1..6` para navegar, `Ctrl+J` para jugar, `Ctrl+L` tienda de mods, `Ctrl+,` ajustes, `Ctrl+/` para ver todos los atajos.
- **Reporte de bugs con un clic**: nuevo botón en Extras → Crash Log que arma un ZIP con logs, crash reports e info del sistema listo para adjuntar en GitHub Issues.
- **Modo sin conexión explicativo**: el badge "Sin conexión" ahora es clickeable y abre un modal que te dice qué funciona y qué no sin internet.
- **Filtros en la consola del servidor**: chips `Todo / INFO / WARN / ERROR / Chat` + buscador libre en la consola de servidores locales.
- **Changelog visible**: al actualizar el launcher, ahora vas a ver un modal con las novedades (este mismo).
- **Log interno del launcher**: nuevo botón `📋 Log launcher` en Extras → Crash Log para abrir `paraguacraft_debug.log` (rotado, máx. 15 MB). El log ahora se incluye automáticamente en el ZIP de reporte de bug.
- **Importar modpacks `.mrpack`**: desde el menú del logo → `📦 Importar modpack .mrpack` podés elegir un archivo local de Modrinth. El launcher detecta automáticamente la versión de Minecraft y el loader (Fabric / Forge / NeoForge / Quilt), descarga todos los mods con verificación SHA-1, aplica los `overrides/` y arma la instancia lista para jugar.

### 🔧 Mejoras
- Timeouts reforzados en descargas de red para evitar cuelgues silenciosos.
- Logging estructurado: migrados los `print()` dispersos a `logging` con archivo rotatorio, facilitando diagnóstico remoto.

### 🐛 Arreglos
- Correcciones menores de UI.

---

## [5.3.0] - 2026-04

### ✨ Novedades
- Soporte para Minecraft 1.21.x.
- Playit.gg integrado para abrir el servidor al mundo.
- Panel de servidor local con consola en vivo y RCON.
- Skins 3D con editor básico.
- Tienda de mods con Modrinth y CurseForge.

### 🔧 Mejoras
- Auto-reparación de JARs corruptos.
- Detección y resolución automática de conflictos de mods.

---

## [5.2.0]

### ✨ Novedades
- Microsoft Authentication (cuentas premium).
- Multi-cuenta.
- Detección automática de Java.

---

## [5.1.0]

### ✨ Novedades
- Soporte Fabric, Forge, NeoForge, Quilt.
- Descarga automática de Java runtime.

---

## [5.0.0]

### 🎉 Primera release pública
- Launcher base con versiones vanilla.
- Interfaz web con pywebview.
- Gestión de skins locales.
