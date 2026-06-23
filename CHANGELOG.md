# Changelog Paraguacraft Launcher

Todos los cambios notables del launcher se documentan acá.
Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [6.9.0] - 2026-06-22

### 🚀 Lanzamiento mayor — Paraguacraft Launcher (Tauri v2)

Reescritura completa del launcher: **Rust + Tauri v2 + Vue 3**. Cliente multiplataforma de grado comercial.

#### Rendimiento
- **0% CPU en segundo plano** mientras jugás: el runtime apaga red, caches y hilos al lanzar Minecraft.
- **Descargas async ultrarrápidas** con Rust (`reqwest`), concurrencia acotada, SHA-1 y escritura atómica.
- **Optimización automática** de RAM/JVM según hardware (gama baja, media y alta).

#### Tienda y contenido
- Tienda nativa **Modrinth + CurseForge** (mods, modpacks, shaders, resource packs, datapacks, plugins).
- **Modpacks `.mrpack`**: crea instancia completa (index, overrides, loader).
- **Plugins** → servidores locales (`plugins/` o `mods/`); **datapacks** → instancia o servidor + mundo.
- Modpacks filtran **versión MC y loader reales** del proyecto (ej. Fabulously Optimized = Fabric, Zombie Invade = Forge).

#### Servidores y extras
- Servidores locales Paper/Fabric/Forge + **Playit.gg invisible**.
- **Diagnóstico de crashes con IA** al salir del juego.
- Instancias aisladas (PvP 1.8.9, modpacks pesados).
- Auto-update integrado (`tauri-plugin-updater` + fallback GitHub Releases).

---

## [6.8.0] - 2026-06-17

### 🔧 Mejoras

- **UI del panel Versiones**: eliminada la tarjeta fija "Cliente Paraguacraft PvP" (botón "Descargar y Jugar") que se mostraba en todas las versiones aunque ya existía el loader **PvP** en 1.8.9. Ahora el flujo es: versión `1.8.9` → loader `PvP` → **JUGAR**.

---

## [6.7.0] - 2026-06-17

### ✨ Novedades

- **Loader PvP (solo 1.8.9)**: nuevo motor en el selector de versiones. Instala Minecraft 1.8.9 + Forge `11.15.1.2318` + OptiFine HD U M5 + mod cliente `ParaguacraftPvP-1.0.0.jar` en la instancia `Paraguacraft_1.8.9_PvP`.
- **Descarga remota del cliente PvP**: `ParaguacraftPvP-1.0.0.jar` se obtiene desde GitHub (`bundled/pvp/` en `main`, con fallback a release `pvp-client-1.0.0`). Verificación SHA-1 y caché global en `.minecraft/Paraguacraft_cache/pvp/`.
- **Panel Cliente Paraguacraft PvP**: en Versiones → 1.8.9, tarjeta dedicada para preparar el perfil, jugar directo y reparar loader + mods.
- **Preset hardware PvP Solo**: asistente de rendimiento actualizado a `1.8.9 · PvP` con instalación automática del bundle completo.

### 🔧 Mejoras

- **OptiFine HD U M5**: descarga desde BMCL API al preparar el perfil PvP (no hace falta copiar el JAR manualmente).
- **Reparar loader PvP**: reinstala Forge `11.15.1.2318` y vuelve a sincronizar los mods del cliente.
- **Motores unificados**: el launcher reconoce tanto `PvP` como `Paraguacraft PvP` para la misma instancia.
- **Auto-actualización Windows**: siempre descarga y ejecuta `Instalar_Paraguacraft_vX.exe` desde `paraguacraft.pages.dev/latest.json` (manifest apunta al instalador Inno Setup, no al portable).

---

## [6.6.0] - 2026-05-28

### 🤖 Bot Discord

- **RCON no bloquea más el event loop**: todas las llamadas a `MCRcon` (vigilar jugadores, `!partido`, `!estado`, `!cmd`, `!dia`, `!noche`, `!sol`, `!kick`, `!ban`, `!op`, `!dificultad`, `!gamemode`, `!tp`, `!tpall`, `!anuncio`, `!guardar`) ahora corren en `asyncio.to_thread()`. El heartbeat de Discord ya no se bloquea cuando el servidor MC está caído o tarda en responder.
- **`!partido` y `buscar_proximo_partido` con fallback web**: si la API de fútbol no tiene registrado el partido (o devuelve vacío), el bot busca automáticamente en DuckDuckGo y muestra los resultados con embed.
- **Caché de API de fútbol corregida**: respuestas vacías ya no se guardan por 60 minutos — expiran en 2 min para reintentarse pronto. Los errores de API (rate limit, acceso denegado) directamente no se cachean.
- **Aliases de equipos ampliados**: `boca juniors`, `paris saint germain`, `paris sg`, `paris saint-germain` agregados al diccionario de alias para búsquedas más robustas.

---

## [6.5.0] - 2026-05-26

### 🐛 Arreglos
- **Lanzamiento desde Biblioteca**: instancias y modpacks ahora inician correctamente al presionar ▶ en la tarjeta o ▶ Jugar en el detalle. El botón muestra spinner y se restaura automáticamente al cerrar el juego.
- **Spinner persistente**: el spinner del botón de lanzamiento ahora dura hasta que Minecraft se cierra (no desaparece después de 10 min para modpacks pesados — nuevo timeout de 60 min).
- **Servidor Fabric**: `iniciar_servidor` detecta y usa `fabric-server-launch.jar` correctamente. Antes intentaba usar `server.jar` que no existe en servidores Fabric.
- **Plugins en servidor Fabric**: instalación de mods/plugins en servidores Fabric ahora apunta a `/mods` en vez de `/plugins`. Afectaba también a Geyser, listado, eliminación, toggle y subida de plugins.
- **Detección de tipo servidor**: nuevo helper `_es_fabric_servidor` unifica la detección (por archivo JAR o por `_paragua_srv.json`) en lugar de 7 chequeos dispersos.
- **Badge "corriendo" en Biblioteca**: corregida comparación de motor que nunca matcheaba para motores con espacios (ej: `Fabric + Iris`). La tarjeta ahora se resalta correctamente mientras el juego está activo.
- **Protección contra sobreescritura de instancias**: `crear_instancia_personalizada` ahora avisa si ya existe una instancia con distinto nombre para la misma versión/motor, en vez de sobreescribir silenciosamente.

### ✨ Novedades
- **Selector de versión del loader Fabric por instancia**: en Biblioteca → ⚙ Configuración → Instalación, las instancias Fabric/Quilt muestran un selector con todas las versiones del loader. Útil para bajar de versión cuando un modpack requiere una versión específica (ej: error "Incompatible mods").

---

## [6.4.0] - 2026-05-26

### 🐛 Arreglos
- **Nombre de modpacks**: modpacks instalados desde Modrinth (ej: Fabulously Optimized) ahora muestran su título de proyecto real en vez del string de versión (ej: "6.3.0-beta.4" o "12.1.2 for 1.21.1"). El título se obtiene de la API de Modrinth al instalar y se guarda en `_paragua_instance.json`.
- **Guardar nombre en Biblioteca**: editar el nombre de una instancia desde Biblioteca → ⚙ Configuración ahora persiste correctamente. Antes, el nombre del modpack en `_paragua_modpacks.json` sobreescribía siempre el nombre guardado en `_paragua_instance.json` al recargar la biblioteca.
- **Botón Iniciar en tarjeta de instancia**: al hacer clic en ▶ en la biblioteca, ahora muestra un spinner de carga, valida Java con `preflightJava` antes de lanzar (igual que el botón principal), y restaura el estado del botón al terminar. Antes no daba ninguna señal visual de que el juego estaba descargando o iniciando.

---

## [6.3.0] - 2026-05-26

### ✨ Novedades
- **Noticias interactivas en Inicio**: cards con imagen oficial, categoría, fecha y enlace directo a las noticias de Minecraft. Datos en tiempo real desde la API de Mojang.
- **Sección "Descubrí en Modrinth"**: row horizontal con los 8 mods más descargados de Modrinth (ícono, nombre, descripción, descargas). Clic abre la tienda de mods del launcher.
- **Íconos reales de servidores**: todos los cards de Servidores Públicos y Quick Play ahora usan `api.mcsrvstat.us/icon/{ip}` — el ícono real del servidor Minecraft, no un favicon genérico.
- **Servidores nuevos**: RedPVP (`play.redpvp.com.ar`), Minebolt (`minebolt.net`) y RhoMC (`play.rhomc.com`) agregados con íconos, descripción, badge No-Premium, ping en tiempo real y Auto-Join.
- **Auto-refresh de sesión Microsoft**: hilo daemon que refresca el `access_token` cada 50 minutos — evita la expiración frecuente (tokens expiran a los 60 min). Arranca al cargar sesión guardada y tras cada login exitoso. Guard flag para prevenir hilos duplicados.

### 🔧 Mejoras
- **Quick Play**: eliminada la posición `sticky` que causaba superposición visual con la navbar al scrollear.
- **Auto-Join**: ahora usa `get_ultima_version_jugada()` en vez de leer el selector de versiones (que podía estar vacío). Funciona desde Servidores Públicos sin necesidad de ir a Versiones primero.
- **Login Microsoft UI**: fondo del overlay de login reemplazado por `main_banner.png` con gradiente oscuro. Botón de Microsoft con ícono SVG oficial.
- **Ping tracker**: RedPVP, Minebolt y RhoMC agregados a `_SERVIDORES_GLOBAL` para ping en tiempo real.
- **Refresco de token al arrancar**: si hay sesión guardada, se lanza un refresh inmediato en background al iniciar el launcher.

### 🐛 Arreglos
- IPs incorrectas de RedPVP/Minebolt/RhoMC en `pingTodosServidores` y `_newSrvCards` corregidas.
- `_iniciar_auto_refresh_ms` idempotente: múltiples llamadas no generan hilos duplicados.

---

## [6.2.0] - 2026-05-24

### ✨ Novedades
- **Versiones Mojang en "Crear instancia"**: el picker de versiones ahora carga la lista completa desde `piston-meta.mojang.com` con fallback a `minecraft-launcher-lib`. Ya no aparece vacío.

### 🔧 Mejoras
- Fallback a múltiples URLs de manifest de versiones para mayor robustez.

---

## [6.1.0] - 2026-05-23

### ✨ Novedades
- **Panel "Versiones" completamente rediseñado**: grid de instancias con imagen de versión, último motor, última vez jugada, botones Jugar/Gestionar/Eliminar por card.
- **Wizard de primera vez**: guía paso a paso al abrir el launcher por primera vez (bienvenida → cuenta → versión recomendada → listo).

### 🔧 Mejoras
- Indicadores de sesión mejorados en el header.
- Transiciones y animaciones refinadas en la UI.

---

## [6.0.0] - 2026-05-22

### ✨ Novedades
- **Login Microsoft unificado (estilo ATLauncher)**: un solo botón con logo de Microsoft abre un modal con **navegador** y **código QR** en el mismo lugar.
- **Inicio de sesión por QR / microsoft.com/link**: flujo device code con código copiable al portapapeles (y al generarse). El QR abre solo `microsoft.com/link` (sin URLs que redirigen a `login.live.com` y fallan en el celular).
- **Tienda — Data packs en servidor local**: selector de servidor guardado y mundo; los datapacks se instalan en `world/datapacks` del servidor (ya no en la instancia del cliente ni con selector de loaders).
- **Tienda — Plugins en servidor local**: selector de servidor destino en la pestaña Plugins; instalación en `plugins/` del servidor activo (Hangar + Modrinth).
- **Importar desde otros launchers**: nueva pestaña Extras → **Importar launcher** con detección de TLauncher, SKLauncher y `.minecraft`, tutorial integrado y copia de saves/mods/resource packs/shaders/options/servers a la instancia activa (premium y no premium).

### 🔧 Mejoras
- Login QR usa client ID compatible con device code (el client oficial del Launcher de Minecraft devolvía 401).
- Código de verificación en recuadro clicable + botones **Copiar código** y **Abrir microsoft.com/link**.
- Datapacks en Modrinth: sin paso de “plataforma” incompatible (`minecraft` / sin loaders).
- Sesión MS guarda `ms_client_id` para refrescar tokens según el método de login (navegador vs QR).
- Multi-cuenta: una sola opción “Agregar cuenta Microsoft” que abre el modal unificado.

### 🐛 Arreglos
- **Overlays**: errores reales en la UI en lugar de `ok: true` silencioso; overlay de teclas sin `bind_all` global (evita conflictos/crashes con el launcher); cierre limpio del listener `pynput`.
- **Overlays**: comprobación de que la ventana tk se abrió antes de reportar éxito.

---

## [5.9.0] - 2026-05-21

### ✨ Novedades
- **Tienda de plugins**: pestaña Plugins con búsqueda federada **Hangar + Modrinth**, trending y instalación en el servidor local.
- **Actualizaciones de plugins del servidor**: detección y aplicación de updates en `plugins/`.
- **Importar servidor existente**: botón para importar una carpeta con `server.jar` / Fabric, detectar tipo y versión, regenerar `iniciar_server.bat` y agregarlo a la lista guardada.
- **Playit.gg**: persistencia de la dirección Java del túnel por servidor en `_paragua_srv.json`.

### 🔧 Mejoras
- Detección automática de versión MC del servidor (`version_history.json`, nombre del JAR, `_paragua_srv.json`).
- Regeneración de `iniciar_server.bat` usando Java local del launcher cuando está disponible.
- Plugins Modrinth filtrados por loaders Paper-compatibles y versión del servidor.
- Ampliaciones en la UI de la tienda de mods y panel de servidores.

### 🐛 Arreglos
- Correcciones en flujos de instalación de plugins y estado del servidor activo.

---

## [5.8.0] - 2026-05-20

### 🔧 Mejoras
- **RAM del servidor local**: `-Xms` y `-Xmx` ahora usan el mismo valor en modo automático (mitad de la RAM del sistema, tope 8 GB), evitando mínimos incoherentes que podían causar comportamiento raro al arrancar Paper/Fabric server.

### 🐛 Arreglos
- Ajuste menor en el arranque del proceso del servidor.

---

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
