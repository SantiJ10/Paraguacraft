# Changelog web (copia resumida — fuente: repo Paraguacraft)

## [7.2.17] - 2026-06-30

### Added
- **Ajustes → Cliente PvP**: versión publicada vs instalada; el cliente se actualiza sin recompilar el launcher.
- **Modo offline**: jugar con mods instalados; skins Premium locales + subida a Mojang al reconectar.

### Changed
- Launcher **7.2.17** / Cliente PvP **2.1.19**.

## [7.2.16] - 2026-06-30

### Fixed
- **HUD GPU**: % de uso y temperatura (como CPU/RAM), no el nombre de la placa.
- **Skins Steve en lobby Hypixel**: fix de texturas corruptas al restaurar estado OpenGL tras nametags.
- **Camas BedWars**: color por equipo (Hypixel sidebar, lana, bloques cercanos) estilo Lunar.

### Changed
- Cliente PvP **2.1.18**.

## [7.2.15] - 2026-06-30

### Fixed
- **Pantalla renderizada "en la esquina"** (cuadrante con el resto en negro): se corrige con un self-heal de viewport, tanto con Windowed Fullscreen como con escalado DPI de Windows (laptops a 125/150%).
- **HUD de hardware**: CPU/RAM ya no marcan carga falsa (medición corregida) y la **GPU se identifica de verdad** por OpenGL (antes iba en "-").

### Changed
- Cliente PvP **2.1.17**.

## [7.2.14] - 2026-06-30

### Added
- **Limitador de FPS en segundo plano** (estilo Lunar/Badlion): al **minimizar** el juego baja los FPS para no calentar ni consumir, y evita el *thermal throttling* que tira los FPS en laptops. Opción extra para limitar también sin foco. Configurable desde el Mod Menu (Rendimiento).

### Fixed
- **Windowed Fullscreen**: al activarlo desde el menú ya no se ve la ventana "achicada" hasta tocar F11; aplica al instante.

### Changed
- Cliente PvP **2.1.16**.

## [7.2.13] - 2026-06-30

### Added
- **Windowed Fullscreen (pantalla completa en ventana)** estilo Patcher/Lunar en el cliente PvP: ventana sin bordes del tamaño del escritorio con **alt-tab instantáneo**, posibilidad de enfocar otras ventanas y **captura por OBS/Discord** sin el parpadeo del fullscreen exclusivo. Se activa desde el Mod Menu (Mecánicas) y también con F11.

### Changed
- **Limpieza automática de restos de Essential/Patcher** en instancias existentes (JAR y carpetas de datos).
- Optimizaciones propias mantenidas y revisadas: culling **sin parpadeo de jugadores**, partículas, skipCombatFx, limpieza de memoria y preset automático por gama de hardware.
- Cliente PvP **2.1.15**.

## [7.2.12] - 2026-06-30

### Removed
- **Patcher / Essential** eliminado del cliente PvP: el Patcher actual depende de Essential (login, cosméticos y reinicio del juego) y no hay versión standalone viable. Cliente más limpio y liviano; el launcher purga Patcher de las instancias existentes.

### Changed
- Queda **OptiFine + optimizaciones propias pulidas**: culling sin parpadeo, partículas y limpieza de memoria al cambiar de mundo (sin freeze al entrar a la partida).
- Cliente PvP **2.1.14**.

## [7.2.11] - 2026-06-30

### Fixed
- **Crash al iniciar el cliente PvP**: el preset Boost FPS dejaba los mipmaps en 0 y rompía el atlas de texturas en 1.8.9. Corregido (mínimo 1, auto-fix).

### Changed
- Cliente PvP **2.1.13**.

## [7.2.10] - 2026-06-30

### Performance
- **Boost FPS reforzado** en el cliente PvP: OptiFine Fast Render + Render Regions + Fast Math, nubes/lluvia/clima apagados, gráficos Fast y Smooth Lighting off.
- Sin `System.gc()` forzado al cambiar de mundo: menos freeze al entrar a la partida (lo maneja Patcher).

## [7.2.9] - 2026-06-30

### Changed
- Se quita el **Borderless** propio y el **Chat compacto** del cliente PvP: los maneja **Patcher** mejor y se evitan conflictos.

## [7.2.8] - 2026-06-30

### Added
- **Patcher (Sk1er)** integrado en el cliente PvP para mejor rendimiento y render.

### Fixed
- Los compañeros ya no desaparecen/reaparecen en partida (no se cullan jugadores).
- Skins rotas en salas de espera (estado GL de nametags endurecido).

## [7.0.2] - 2026-06-23

### Added
- **Reparar instancia**: JARs corruptos, vanilla, loader y bundles PvP/Iris.
- **Visor de logs**: últimas líneas de `latest.log` desde el detalle de instancia.
- **Presets RAM** en ajustes (Casual / Normal / Modpack / Competitivo).
- **Gestor de mods** por instancia: buscar, agregar, eliminar y revelar en carpeta.
- **Modpacks CurseForge `.zip`**: importación desde archivo o tienda.
- **Exportar instancia** a `.zip` portable.
- **Servidores favoritos** con join directo (`--server` / `--port`).
- **Feedback de descargas** mejorado con errores por archivo.

### Changed
- Auto-update abre el instalador NSIS visible y cierra el launcher correctamente.

## [7.0.1] - 2026-06-19

### Fixed
- Auto-update: abre el instalador NSIS visible (no `relaunch` silencioso).
- Auto-update: cierra el launcher tras lanzar el setup para permitir reinstalar.
- CI release: genera `latest.json` con hash real.

## [7.0.0] - 2026-06-19

### Fixed
- PvP 1.8.9: OptiFine y cliente desde `bundled/pvp` en GitHub.
- Lanzamiento Java 8: flags JVM compatibles.
- Modpacks Modrinth: reintentos en red y loader antes de mods.
- CurseForge: API key embebida en builds de release.

## [6.9.0] - 2026-06-22

### Added
- **Lanzamiento mayor** — Paraguacraft Launcher (Tauri v2 + Rust + Vue 3).
- 0% CPU en segundo plano mientras jugás.
- Tienda Modrinth + CurseForge, modpacks `.mrpack`, servidores Playit.gg.
- Diagnóstico de crashes con IA y auto-update integrado.
