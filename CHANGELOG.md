# Changelog Paraguacraft Launcher

Todos los cambios notables del launcher se documentan acÃ¡.
Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [1.1.26] - 2026-08-10

### Launcher
- Estilo de juego PvP **Full rendimiento / Casual** (options + mods por estilo).
- Catalogo multiserver (Hypixel, Minemen, Cube, UC, Mush, Regorland) + notas premium/offline.
- Destinos de perfiles y Modo Competir ampliados.
- Optimized: re-aplica options/configs de mods en cada launch (marker v9).
- Packs oficiales: SHA actualizados (HUD, armor, crit, bridge).

### Cliente PvP Modern (0.9.24)
- ServerContext (Minemen/Mush/UC/Regorland), freelook estricto Minemen.
- HUD por modo (BW/SW/Duels/HG), filtros scoreboard LATAM.
- PlayStyle competitive feel (bob/FOV, RD, animaciones).

### Resource packs (oficial 1.8.9 + modern)
- Oleadas 1–3: bloques BW/SW, armor en otros jugadores (+netherite modern),
  crits, bridge/arena, HUD hearts/hotbar modern, icons/widgets 1.8, fuente ASCII mas fina.

## [1.1.17] - 2026-08-08

### Launcher â€” Servers / Playit
- Secret e IP **compartidos** entre todos los servers (un claim = misma `*.tun.ply.gg`).
- Captura claim/IP del plugin Paper en consola y la muestra en la UI.
- No lanza `playit.exe` si hay plugin playit-gg (evita cuelgue IPC).
- Un solo server MC a la vez con el tÃºnel compartido.
- Â«Restaurar secret compartidoÂ» y Â«Reseteo total PlayitÂ»; avisos de cupo de agentes.
- Puerto local unificado al arrancar (`server-port`).

## [1.1.16] - 2026-08-06

### Launcher
- BotÃ³n **Cafecito** en title bar / top bar (abre el navegador del sistema).
- Tarjeta **Otras versiones** (tras PARAGUA 1.7) con releases sin tarjeta propia y loaders filtrados.
- BrandPack: logo wide **1024Ã—256** desde Minecraft **1.20.0** (corrige estirado en 1.20.x).
- Fallbacks de clientes: **2.1.45** / Modern **0.9.22**.

### Cliente PvP 1.8.9 (2.1.45)
- Recursos BW: opciÃ³n `showItemNames` (nombres o layout compacto).

### Cliente PvP Modern (0.9.22)
- Recursos BW: muestra Hierro/Oro/Esmeralda/Diamante + `showItemNames`.

## [1.1.15] - 2026-08-06

### Launcher
- Pack PvP 1.8.9: SHA actualizado y orden brand â†’ oficial en `options.txt`.
- Fallbacks de clientes: **2.1.44** / Modern **0.9.21**.

### Cliente PvP 1.8.9 (2.1.44)
- Pack oficial se mueve a **Selected** con `setRepositories` (se guarda al salir).
- Efecto romper Dewier (`destroy_stage`) en pack oficial; re-descarga si el SHA no coincide.
- HUD MÃºsica: transparencia total + carÃ¡tula **16Ã—16 / 32Ã—32**.

### Cliente PvP Modern (0.9.21)
- HUD MÃºsica: carÃ¡tula 16/32 y mÃ¡s pasos de transparencia del panel.

## [1.1.14] - 2026-08-06

### Fixed
- Skins multiplayer offline: ya no se pisa `steve.png`/`alex.png` del BrandPack (otros jugadores dejaban de verse con tu historial).
- Avatar offline: muestra la cara recortada, no el PNG UV completo de la skin.
- LocalSkin de CustomSkinLoader se escribe por nick; al lanzar se limpian texturas default envenenadas.

### Added
- Renombrar nick de cuentas offline (Ajustes â†’ Cuentas) para alinear con Ely.by.
- (clientes ya en main) Escala proporcional por mÃ³dulo en Editar HUD â€” PvP **2.1.41** / Modern **0.9.19**.

## [1.1.13] - 2026-08-06

### Fixed
- Pack oficial PvP 1.8.9 se aplica solo al lanzar (token con `.zip` + no sobrescribir con BrandPack offline).

### Added
- (clientes) Escala UI compartida HUD + mod menu en 1.8.9 y modern.

## [1.1.12] - 2026-08-06

### Added
- Mini guÃ­a de skins para cuentas offline en la vista Skins (pack local, CustomSkinLoader, Ely.by y quiÃ©n te ve).

## [1.1.10] - 2026-07-28

### Fixed
- Crash al iniciar **Optimized / Fabric** en varias PCs (fullscreen exclusivo + flag GL de Sodium).
- Mods crÃ­ticos de Optimized: si faltan Fabric API / Sodium / Iris / Lithium, error claro en vez de crash silencioso.
- LibrerÃ­as incompletas: el lanzamiento avisa y sugiere Reparar instancia.
- Discord: stream congelado al compartir pantalla (renombre de ventana menos agresivo + sin exclusive fullscreen).

### Changed
- Optimized: RAM por defecto segÃºn la gama de la PC (sin forzar 6 GB).

## [1.1.9] - 2026-07-27

### Fixed
- CurseForge tienda **403 Forbidden**: la API key con `$2a$â€¦` ya no se corrompe al cargar `.env`.
- Optimized **1.8.9 / 1.12.2**: OptiFine + mods Forge (FoamFix, VanillaFix, etc.) se instalan de verdad (Modrinth + BMCL fallback).
- Shaders Iris/modernos ya no se meten en packs OptiFine legacy; se purgan los incompatibles.

### Changed
- Shaders por backend: OptiFine â†’ Sildurâ€™s/BSL/hosted; Fabric/NeoForge â†’ Iris (Oculus en NeoForge 1.20.1).

## [1.1.3] - 2026-07-24

### Fixed
- Discord: menos renombres de tÃ­tulo de ventana (evita romper Overlay / compartir pantalla).
- Clientes embebidos Modern 0.9.12 / PvP 2.1.35 (borderless, iconos de servidores, Mod Menu agrupado).

## Cliente PvP Modern 0.9.12 / 1.8.9 2.1.35 - 2026-07-24


### Fixed
- Borderless mÃ¡s estable con Discord (menos pelea con Overlay / captura de ventana).
- Iconos de servidores en lista multiplayer: si el ping no trae favicon, se descarga desde mcsrvstat.us.

### Changed
- Mod Menu: Armadura (+%), FPS, Entity/cull, BedWars, Chat y Scoreboard agrupados en tarjetas con submods (estilo MÃºsica).

## Cliente PvP Modern 0.9.11 / 1.8.9 2.1.34 - 2026-07-24


### Fixed
- TNT countdown estilo Lunar (`1,35`) ahora se dibuja de forma fiable:
  - **1.8.9:** `RenderWorldLastEvent` (no depende del renderer que OptiFine reemplaza).
  - **1.21.11:** `WorldRenderEvents.AFTER_ENTITIES` (texto billboard sobre la TNT).

## [1.1.2] - 2026-07-24

### Fixed
- TNT countdown en clientes embebidos (Modern 0.9.11, PvP 2.1.34).

## Cliente PvP Modern 0.9.10 / 1.8.9 2.1.33 - 2026-07-24

### Fixed
- TNT countdown estilo Lunar: segundos con 2 decimales (ej. `1,35`) sobre la TNT encendida.
- Modern: el label usaba coords del mundo en vez de offset local (el texto no se veÃ­a).
- 1.8.9: render tipo nametag (blanco + sombra + fondo).

## Cliente PvP Modern 0.9.9 / 1.8.9 2.1.32 - 2026-07-24

### Fixed
- Portada de Spotify/YouTube en el HUD: se muestra la imagen original (antes cuadrado negro o disco amarillo).
- Modern: `drawTexture` escalaba mal (solo 16Ã—16 px de esquina de la carÃ¡tula).
- 1.8.9: lee la cache del launcher (`music-art/{sha1}.jpg`) ademÃ¡s de HTTP.
- Launcher: cuando la carÃ¡tula ya estÃ¡ en disco, el IPC manda `file://` para carga instantÃ¡nea.

## Cliente PvP Modern 0.9.8 - 2026-07-23

### Fixed
- TNT countdown visible en 1.21.11 (usa `displayName` del pipeline de labels vanilla).
- Resource packs: orden vanilla â†’ brand â†’ PvP oficial (mÃ¡xima prioridad); purge conserva brand.

## [1.1.1] - 2026-07-24

### Fixed
- Portada Spotify/YouTube en HUD in-game (imagen original, no disco/negro).
- Clientes embebidos: Modern **0.9.9**, PvP 1.8.9 **2.1.32**.

## [1.1.0] - 2026-07-23

### Fixed
- TNT countdown en cliente PvP Modern 0.9.8.
- Resource packs: orden vanilla â†’ brand â†’ PvP oficial (mÃ¡xima prioridad).

### Cliente PvP Modern 0.9.8
- Incluido embebido en el instalador (sin release separada del cliente).

## [1.0.1] - 2026-07-23

### Fixed
- Stack de resource packs al lanzar: PvP oficial arriba de vanilla + brand preservado.

## Cliente PvP Modern 0.9.7 - 2026-07-23

### Added
- TNT countdown: segundos restantes visibles sobre la entidad TNT (1.21.11 y 1.8.9).
- Discord Rich Presence desde el launcher para todas las instancias; el mod respeta `PARAGUACRAFT_DISABLE_RPC`.

### Changed
- Resource packs: solo `paraguacraft-pvp-modern` oficial; purge de packs extra al instalar/lanzar.
- Groq + CurseForge integrados en el launcher (keys embebidas en build, ocultas en Ajustes).

## [1.0.0] - 2026-07-23

### Added
- Instalador NSIS per-machine con branding Paraguacraft.
- Keys de Groq y CurseForge gestionadas por el launcher (sin pedirlas al usuario).

### Changed
- Reset de versiÃ³n del launcher a **1.0.0**.
- Discord RPC unificado vÃ­a launcher para vanilla, Fabric, Forge y servidores locales.

## Cliente PvP Modern 0.9.6 - 2026-07-22

### Fixed
- Crash en Hypixel BedWars: bucle infinito al mostrar alertas de chat (`sendMessage` re-disparaba el evento GAME).
- `ScoreboardFilter.strip` ya no usa regex sobre cÃ³digos de formato (mÃ¡s seguro con texto de Hypixel).
- Resource pack: `paraguacraft-pvp-modern.zip` queda siempre como principal si estÃ¡ instalado; packs cosmÃ©ticos (p. ej. Mr Blue Sky) pasan a secundario.

## Cliente PvP Modern 0.9.5 - 2026-07-22

### Fixed
- Crash al iniciar: mixin del scoreboard apuntaba a `fill(RenderPipeline)` inexistente; usa `fill(IIIII)` en el overload correcto.

## Cliente PvP Modern 0.9.4 - 2026-07-22

### Fixed
- Crash al iniciar: mixin del scoreboard actualizado para Minecraft 1.21.11 (`fill` con RenderPipeline).

## Cliente PvP Modern 0.9.3 - 2026-07-21

### Added
- **Cull real 1.21** (paridad 1.8.9): entity, nametag + LOD, block entity, anim freeze, armor stand e item frame.
- Mixins en render pipeline moderno (`EntityRenderManager`, `PlayerEntityRenderer`, `BlockEntityRenderer`, `LivingEntityRenderer`).
- **GameModeDetector por servidor** (Hypixel / Cubecraft) con **override manual** desde Mod Menu.
- **Editor in-game** de palabras y color en chat alerts.
- **BedWars resources**: inventario + fallback scoreboard (iron/gold/diamond/emerald).
- Filtros de scoreboard especificos para **Cubecraft** ademas de Hypixel.

### Changed
- Eliminado hack de `entityDistanceScaling` como sustituto de cull; toggles activos por defecto en instalaciones nuevas.
- Mod Menu categoria Rendimiento: 7 toggles de cull + export en perfiles.
- **Fullbright unificado** (`FullbrightManager`): Gamma Utils o integrado, misma tarjeta y tecla G.

## [7.9.18] - 2026-07-22

### Fixed
- Cliente PvP Modern **0.9.4**: crash por mixin de scoreboard incompatible con 1.21.11.

## [7.9.17] - 2026-07-22

### Fixed
- Stack PvP 1.21.11: Sodium **0.8.12** (compatible con Iris 1.10.7 y Reese's Sodium Options 2.2.3; evita conflicto 0.8.7 vs Reese's).

## [7.9.16] - 2026-07-22

### Fixed
- **Servidores locales**: MC 26.x+ usa Java 25 (`java-runtime-epsilon`) en lugar de Java 21; descarga automÃ¡tica si falta.
- **Playit.gg**: agente por servidor con `--secret-path` (`playit-agent.toml`); recuperaciÃ³n ante `InvalidAgentKey`.
- Forge/NeoForge: `JAVA_HOME` al iniciar `run.bat`.

## [7.9.15] - 2026-07-22

### Fixed
- Pin de **Cloth Config** con nombre de archivo correcto (`-fabric`, no `+fabric`): el mod se descargaba pero la sync fallaba.
- **Fabric + Iris** (todas las versiones MC): cache v5, par Iris+Sodium compatible obligatorio, validaciÃ³n de fabric-api/sodium/iris/lithium y purge de Sodium incorrecto.

## [7.9.14] - 2026-07-22

### Fixed
- InstalaciÃ³n PvP Modern: al crear/reparar/lanzar la instancia se descargan **todos** los mods pinneados (HUD + stack), no solo Iris + mod Paraguacraft. Errores explÃ­citos si falta alguno.

## [7.9.13] - 2026-07-22

### Fixed
- Stack PvP Modern 1.21.11 con versiones fijadas (Sodium 0.8.7 + Iris 1.10.7 + FLK/YACL/Fabric API/etc.) y purge de JARs viejos.

## [7.9.12] - 2026-07-22

### Fixed
- Bundle Iris: ya no copia Sodium duplicado del cache (manifest por slug); purge de Sodium 0.8.13 con Iris 1.10.7.

## [7.9.11] - 2026-07-22

### Fixed
- Cliente PvP 1.21.11: conflicto Sodium 0.8.13 + Iris 1.10.7 al instalar `sodium-extra` 0.9.x (pin 0.8.3, dedupe bundle, reparar HUD).

## [7.9.10] - 2026-07-22

### Changed
- Cliente PvP Modern **0.9.3** (cull real, GameMode override, chat alerts editor, BW scoreboard, Cubecraft filters).

## Cliente PvP Modern 0.9.2 - 2026-07-21

### Added
- **Animaciones 1.7 fase 2**: swing visual, comer/beber y blockhit (escudo) en primera persona.
- **Editor in-game** de keywords en chat triggers.
- **GameModeDetector** lee lineas del scoreboard, no solo el titulo.
- **Shaders auto-off**: notificacion y reload Iris al entrar/salir de partida.
- **Freelook**: aviso en action bar cuando esta bloqueado en ranked.

### Changed
- Animaciones 1.7 activadas por defecto; toggle renombrado en Mod Menu; incluido en perfiles.

## Cliente PvP Modern 0.9.1 - 2026-07-21

### Added
- **Pack dual UI**: principal + secundario en Texture packs.
- **Editor HUD**: arrastrar modo de juego, timer bridge y reach.
- **Quick Play (`)**: selector Hypixel / Cubecraft.
- **Pantallas** de config chat alerts y chat triggers.

### Fixed
- Scoreboard: lineas filtradas ya no dejan huecos vacios.
- Perfiles export/import incluyen toggles nuevos de 0.9.0.
- Gamma Utils: toggle desde Mod Menu y tecla G.

## [7.9.9] - 2026-07-21

### Changed
- Cliente PvP Modern **0.9.2** (animaciones 1.7, chat triggers editor, shaders reload).

## [7.9.8] - 2026-07-21

### Changed
- Pulidos UX 0.9.1; cliente embebido y `latest.json` actualizados.

## Cliente PvP Modern 0.9.0 - 2026-07-21

### Added
- **Perfiles auto por modo**: HUD y toggles segÃºn BedWars, SkyWars, Duels, etc. (scoreboard).
- **Quick Play Cubecraft**: reconexiÃ³n y modos EggWars, SkyWars, Lucky Islands, etc.
- **Pack dual**: oficial + pack secundario sin pisarlo al reconectar.
- **Shaders auto-off** en partida competitiva (Hypixel/Cubecraft).
- **Scoreboard limpio**: fondo transparente, ocultar stats y nÃºmeros rojos.
- **Chat triggers/alerts** configurables; **timer bridge**; **HUD modo juego**; **reach solo prÃ¡ctica**.
- **Freelook blacklist** en servidores ranked; **Discord RPC** con modo; **export stats** al desconectar.

## [7.9.7] - 2026-07-21

### Changed
- Cliente PvP Modern embebido y fallback actualizados a **0.9.0**.

## Cliente PvP Modern 0.8.4 - 2026-07-21

### Fixed
- Gamma Utils fullbright, freelook pitch, carÃ¡tula mÃºsica, pack forzado, mods auto-install.

## Cliente PvP Modern 0.8.3 - 2026-07-21

### Fixed
- **CarÃ¡tula de mÃºsica**: rutas `file://?/C:/...` de Windows (prefijo `\\?\`) y fallback por nombre en `music-art/`.
- **Fullbright**: ya no fuerza gamma ilegal (16/100); solo mixin de lightmap.

## [7.9.4] - 2026-07-21

### Fixed
- **IPC mÃºsica**: URLs `file://` sin prefijo `?/` invÃ¡lido en Windows (canonicalize).

## [7.9.3] - 2026-07-21

### Fixed
- **Cliente PvP 1.21.11 (0.8.2)**: mirrors extra de descarga (GitHub raw + `github.com/raw`); JAR publicado tambiÃ©n en `clientes/`; cliente embebido en el instalador para sincronizar sin depender solo de la red.
- **IPC mÃºsica**: el campo de imagen mantiene URL https (el mod lee la cache local del launcher).

## [7.9.2] - 2026-07-21

### Added
- **Pack PvP auto-aplicado**: al lanzar instancias `paraguacraft-pvp` (1.8.9) y `paraguacraft-pvp-modern` (1.21.11), el launcher descarga y activa `paraguacraft-pvp-189.zip` / `paraguacraft-pvp-modern.zip` en `options.txt` (como `ParaguacraftBrandPack`).

### Fixed
- **CarÃ¡tula de mÃºsica in-game**: URLs `file://` corregidas en Windows; el cliente lee la imagen cacheada del launcher y muestra la portada original de Spotify/YouTube (1.8.9 y 1.21.11).

## Cliente PvP 2.1.31 - 2026-07-21

### Fixed
- Soporte `file://` para carÃ¡tulas cacheadas por el launcher; dibujo con dimensiones reales de la textura.

## Cliente PvP Modern 0.8.2 - 2026-07-21

### Fixed
- **Fullbright**: mixin en lightmap + gamma alto; ya no queda todo negro en cuevas.
- **TNT countdown**: segundos flotando sobre la TNT (como 1.8.9).
- **CarÃ¡tula de mÃºsica**: lee cache local del launcher por SHA-1; disco solo si Â«Descargar portadaÂ» estÃ¡ OFF.

### Changed
- **Mod Menu interno**: icono Ãºnico por mod (items temÃ¡ticos); logo Paraguacraft solo en Mod Menu de Fabric (`icon.png`).

## Cliente PvP Modern 0.8.1 - 2026-07-21

### Fixed
- CarÃ¡tula de mÃºsica: decodificaciÃ³n directa sin reescalar a 64Ã—64; pack PvP aplicado por defecto si no hay otro guardado.

## [7.9.1] - 2026-07-21

### Fixed
- **Servidores Paper (todas las versiones)**: migraciÃ³n a Paper Fill API v3 (`fill.papermc.io`); la v2 devolvÃ­a **410 Gone** y bloqueaba Â«PrepararÂ»/Â«IniciarÂ» (ej. MC 26.2, 1.21.11, 1.20.x).
- **Paper/Fabric + Geyser**: descarga oficial de Geyser/Floodgate corregida (ya no usa `latest/latest` ni la clave inexistente `paper`; usa `spigot` + Ãºltimo build) con fallback Modrinth.
- **Fabric + Iris (instancias)**: Iris ya no se salta al instalar el bundle â€” se resuelve Iris primero y se fija la versiÃ³n exacta de Sodium que Iris exige (compatible con 1.21.11).
- **Servidores NeoForge**: loader propio (1.20.2+) en lugar de confundirse con Forge; modpacks NeoForge de Modrinth/CurseForge tambiÃ©n.
- **Plugins/mods opcionales**: si ViaVersion, Geyser, SkinsRestorer, etc. fallan al preparar, se registra un aviso en la consola del servidor sin abortar la descarga del `server.jar`.

## [7.9.0] - 2026-07-21

### Added
- **Tienda paginada**: paginaciÃ³n real (no solo destacados) en Mods, Resource Packs, Plugins, Shaders y Modpacks, con componente `<Pagination>` reutilizable.
- **Filtrado dinÃ¡mico de compatibilidad**: los selectores de versiÃ³n/loader en el modal de instalaciÃ³n solo muestran combinaciones que el proyecto realmente publica en Modrinth/CurseForge.
- **Descarga inteligente de dependencias**: modal de confirmaciÃ³n con checkboxes al instalar un mod con dependencias requeridas/embebidas; se puede omitir sin frenar la instalaciÃ³n.
- **Motor de optimizaciÃ³n dinÃ¡mica**: antes de lanzar cualquier instancia, el launcher detecta RAM/CPU y aplica argumentos JVM y perfil de `options.txt` diferenciados por gama de PC (Baja/Media/Alta) y por loader (1.8.9 Forge+OptiFine, 1.21.11 Fabric+Sodium+Iris, o genÃ©rico), sin pisar los presets PvP ya afinados.

### Changed
- **Rendimiento de la UI**: Tienda, Ajustes, Skins, Instancias, Versiones y Servidores quedan en cachÃ© (`KeepAlive` + Pinia) al cambiar de pestaÃ±a, sin repetir pantallas de carga.
- **Backend sin bloqueos**: descompresiÃ³n de `.mrpack`/CurseForge y hashing de auto-update de mods ahora corren en `tokio::spawn_blocking`, para que la ventana nunca aparezca como "No responde" durante instalaciones o auto-updates pesados.

## Cliente PvP Modern 0.8.0 - 2026-07-21

### Added
- **Mod Menu reorganizado**: tarjetas "Musica", "Insignias y Ping" y "Sprint" agrupan mods relacionados en una sola pantalla en vez de tarjetas sueltas.
- **Paridad 1.8.9 en Rendimiento**: tarjeta "Armadura %", ciclo de "Particulas" (Minimas/Reducidas/Todas), botones "Limpiar memoria" y "Aplicar preset de hardware".
- **Estadisticas de combate por sesion** (nuevo, ninguno de Lunar/Badlion lo tiene): golpes, mejor combo y muertes 100% confiables (cliente-only) + "posibles bajas" heuristico marcado siempre como estimado, con panel HUD opcional y reset automatico al unirse a un mundo/servidor.

### Fixed
- **HUD armadura + contador de bloques**: se eliminÃ³ el clamp que forzaba la posiciÃ³n de vuelta al valor viejo en cada carga, impidiendo reposicionar el HUD de forma permanente. Nuevo default junto al hotbar (esquina inferior derecha), con migraciÃ³n automÃ¡tica para quien nunca lo moviÃ³ a mano.
- **CarÃ¡tula de mÃºsica**: si el registro de la textura fallaba con una excepciÃ³n, el flag de carga quedaba pegado y esa canciÃ³n se quedaba con el disco genÃ©rico para siempre. Ahora cualquier error libera el flag y se reintenta en el siguiente poll (con log para diagnÃ³stico).

### Changed
- "FPS bajo minimizado" ahora se llama "FPS bajo sin foco" (la lÃ³gica ya usaba el foco de ventana, no solo minimizado).

## [7.8.0] - 2026-07-20

### Added
- **Join inteligente desde favoritos**: infiere perfil PvP 1.8.9 vs Modern por hint o Server List Ping; botÃ³n Unirse sin depender de la instancia seleccionada.
- **Compete mode PvP Modern**: presets agresivos al lanzar Hypixel/favoritos (culling, HUD mÃ­nimo, toggles).
- **Asistente Playit primera vez**: checklist con claim link, IP Java/Bedrock copiables y guardar favorito.
- **Consola de servidor mejorada**: botones rÃ¡pidos (/whitelist, /op, /gamemode, /time, historial).
- **Tienda servidor**: recomendados Paraguacraft (ViaVersion, Geyser, SkinsRestorer) en Paper.
- **Plugin badges Paper 1.21+**: auto-instalaciÃ³n segÃºn versiÃ³n del servidor.
- **Auto-update CurseForge**: fingerprint murmur2 + badge "Instalada" en tienda.
- **Bedrock â†” Java**: favoritos con puerto Geyser, botÃ³n Bedrock y copiar instrucciones para amigos.
- **Paraguabot mejorado**: contexto de instancia/favoritos/servidor + acciones clicables (Reparar, Sincronizar PvP, Abrir consola).

### Changed
- Cache de carÃ¡tula musical en disco (`file://`) para cliente Modern mÃ¡s robusto.
- Skins offline unificadas launcher â†’ `paraguacraftpvp-modern.properties`.

## Cliente PvP Modern 0.7.0 - 2026-07-20

### Added
- **Paridad PvP con 1.8.9**: toggle sneak, culling entidades/nametags, idle FPS, perfiles export/import.
- **Paridad social**: badges en nametag, ping del rival, HUD de servidor conectado.
- **Mod Menu**: buscador funcional, atajos vanilla, pantalla de perfiles.
- **Training world**: kit PvP + 3 cofres (vanilla/pot/UHC) al crear mundo nuevo.
- **BadgeProtocol** Fabric compatible con plugin Paper/Fabric del servidor.

### Fixed
- Caratula musica via cache local del launcher (`file://`); skins offline auto-aplicadas al iniciar.

## Cliente PvP 1.8.9 2.1.30 - 2026-07-20

### Added
- **Training world**: kit inicial (espada, arco, perlas, bloques) y cofres con loadouts vanilla/pot/UHC.

## Cliente PvP Modern 0.6.17 - 2026-07-20

### Fixed
- Caratula musica: sin header WebP/AVIF, escala bilineal 64px, IPC sin basura en URL.
- Multijugador: Regorland (`regorland.net`), CubeCraft (`play.cubecraft.net`), lista v2.

## Cliente PvP Modern 0.6.16 - 2026-07-20

### Fixed
- Caratula Spotify/YouTube: decode con ImageIO (como 1.8.9), prefetch al leer IPC y flush inmediato de URL en launcher.
- Opcion "Descargar portada OFF" muestra disco; ON descarga la portada real.

## Cliente PvP Modern 0.6.15 - 2026-07-20

### Fixed
- Caratula de musica: descarga real de Spotify/YouTube via `NativeImage.read` (sin conversion manual que corrompia colores).

## Cliente PvP Modern 0.6.14 - 2026-07-20

### Changed
- HUD **Hardware** y **Musica** con posiciones independientes (arrastrables por separado en editar HUD).
- Caratula Spotify: textura 64x64, conversion ARGB corregida y reintentos con cooldown.
- **Freelook** usa swap de rotacion en `Camera.update` como 1.8.9 (sin camara desacoplada).
- **Toggle sprint (M)**: tecla virtual al inicio del tick estilo Lunar; ON por defecto.

## Cliente PvP Modern 0.6.13 - 2026-07-19

### Fixed
- Crash al iniciar: NickFinder en tab usa `getPlayerName` en lugar de `drawTextWithShadow` (API 1.21.11).

## Cliente PvP Modern 0.6.12 - 2026-07-19

### Fixed
- Crash al iniciar: firma de `MixinCameraFreelook` corregida (`World` en lugar de `BlockView` en `Camera.update`).

## Cliente PvP Modern 0.6.11 - 2026-07-19

### Added
- **NickFinder en nametags 3D**: resaltado cian en negrita sobre entidades cuando coincide la busqueda.
- **Colores de equipo** en nametags cuando `teamColors` esta activo (tinte segun lana adyacente en BedWars).

## Cliente PvP Modern 0.6.10 - 2026-07-19

### Added
- **NickFinder**: tecla `N`, busqueda parcial en tab y resaltado cian.
- **Camas coloridas** BedWars segun lana adyacente (estilo Hytils).

### Changed
- HUD musica alineado a 1.8.9: metricas CPU/RAM/GPU cian+blanco, artista `#8899AA`, alpha 255, pos Y 260.
- Caratula 16x16 nearest-neighbor + Referer Spotify; ancho panel ajustado al texto.

## Cliente PvP Modern 0.6.9 - 2026-07-19

### Added
- **Hardware HUD** y **Reach display** portados de 1.8.9.
- **Crosshair custom** (5 modos) en Mod Menu.
- Persistencia de **texture pack** seleccionado y **Boost FPS** entre sesiones.

### Fixed
- **Freelook**: camara en 3Âª persona con mixin en `Camera.update`.
- **Toggle sprint** desactivado por defecto (sin simular tecla virtual).
- **Caratula musica**: conversion ARGB correcta y panel compacto con hardware arriba.
- **Paraguabot Groq**: modelos `openai/gpt-oss-*` como fallback.
- Servidores **Regorland** (`regorland.net`) y **Hylex** (`original.hylex.net`).
- Launcher ya no resetea `options.txt` ni toggles HUD en cada inicio.

## [7.7.5] - 2026-07-19

### Added
- **Discord RPC Bedrock**: `{usuario} - Bedrock Edition` con estado dinÃ¡mico (menÃº, mundo) leyendo el tÃ­tulo de ventana antes del rename.
- **Discord RPC Ajustes**: `Explorando Ajustes Â· {usuario}` al entrar a la pantalla de configuraciÃ³n.

### Changed
- RPC del launcher respeta sesiÃ³n activa (Java o Bedrock) y no se pisa al navegar mientras jugÃ¡s.

## [7.7.4] - 2026-07-18

### Fixed
- **Cliente PvP Modern 0.6.2**: sincroniza el JAR corregido (mixin ItemEntityRenderer); evita pantalla negra en el menu.
- **Dynamic FPS offline**: manifest embebido y fallback actualizados a 0.6.2.

## Cliente PvP Modern 0.6.6 - 2026-07-19

### Added
- **HUD Musica**: transparencia (100%/25%/transparente), tamano y descarga de portada Spotify/YouTube.
- **Editar HUD**: preview real de mods activos con cajas arrastrables.

### Fixed
- **Armadura HUD**: iconos de casco a botas visibles en partida.
- **Logo Paraguacraft** en menu principal (textura embebida, escala nitida).
- **Freelook**: camara libre sin girar el movimiento (como 1.8.9).

### Changed
- BrÃºjula detallada, ping con fallback, coordenadas con posicion propia en editor HUD.

## Cliente PvP Modern 0.6.5 - 2026-07-19

### Added
- **Discord RPC in-game** (usuario - versiÃ³n - loader + servidor/mundo/menÃº).

## Cliente PvP Modern 0.6.4 - 2026-07-19

### Added
- **Pantalla sin bordes** (borderless LWJGL3) en Mod Menu; alternativa a fullscreen exclusivo.
- **Mod Menu** con tarjetas estilo 1.8.9 (negro + borde azul, ON/OFF/ABRIR).

### Changed
- SubmenÃºs (Multijugador, Hypixel Quick Play, Mod Menu, Tema, Packs, Skin) usan botones/tarjetas Paraguacraft.
- Launcher: ya no pisa `options.txt` ni configs en cada launch (solo merge de claves nuevas).

### Fixed
- **ESC / volver al menÃº**: redirige siempre al menÃº Paraguacraft, no al TitleScreen vanilla.
- **Logo duplicado** en submenÃºs (fondo sin branding en pantallas hijas).
- **Opciones que se reseteaban**: volumen, chunks, pantalla completa, Sodium â€” el launcher y `PerformanceBootstrap` ya no las sobrescriben al actualizar.

## Cliente PvP Modern 0.6.3 - 2026-07-19

### Added
- **Contador de bloques** en HUD (BedWars, SkyWars, Lucky Islands, Pillars).
- **MusicArtCache**: carÃ¡tula de Spotify/YouTube en el HUD de mÃºsica.

### Changed
- **Recursos BedWars**: columna vertical con fondo transparente (estilo 1.8.9).
- **HUD pociones y objeto en mano**: nombre, duraciÃ³n y encantamientos como 1.8.9.
- **Armadura**: columna vertical solo iconos.
- **MenÃº**: botones negros con borde azul al hover; fondo Paraguacraft en subpantallas vanilla.
- **Toggle sprint**: restaurado modo virtual + legacy (W) como 1.8.9.
- **CPS**: calculado en tick, no en render (menos micro-lag).

### Fixed
- **Boost FPS** ya no aparece fijo en el HUD (libera espacio para FPS).
- **Logo duplicado** y pantalla negra en menÃº/multijugador/conexiÃ³n.
- **IPC mÃºsica**: launcher lee config modern (`paraguacraftpvp-modern.properties`).

## [7.7.3] - 2026-07-18

### Fixed
- **Mods Fabric 1.21.11**: instala dependencias de Controlling/Zoomify (`searchables`, `fabric-language-kotlin`, `yacl`) y resuelve `required` de Modrinth en cadena.
- **Tienda Modrinth (503)**: hasta 6 reintentos; si la API falla, descarga directo desde CDN con la URL ya cargada en el asistente.
- **Cliente PvP Modern 0.6.1**: sincroniza el JAR embebido y remoto; corrige crash al iniciar por mixin de hitbox en 1.21.11.
- **Dynamic FPS**: repara `dynamic_fps.json` con estado obsoleto `minimized` (ahora `invisible` en 3.11.6).

## [7.7.2] - 2026-07-18

### Fixed
- **Inicio y Ajustes lentos (5â€“7 s)**: bootstrap Ãºnico en paralelo, sin cargas duplicadas de cuentas/skins/instancias; Ajustes ya no bloquea la UI esperando Mojang.
- **503 Mojang al jugar**: si `api.minecraftservices.com` estÃ¡ caÃ­do, reutiliza el token guardado en lugar de bloquear el inicio.
- **Mods Fabric 1.21.11**: el launcher instala dependencias de Controlling/Zoomify (`searchables`, `fabric-language-kotlin`, `yacl`) vÃ­a Modrinth.
- **Tienda Modrinth (503)**: reintentos mÃ¡s largos y descarga directa desde CDN si la API falla al instalar.

### Changed
- Cache de hardware (5 min) y avatar activo (2 min); avatar local instantÃ¡neo antes del enrich de Mojang.
- Escaneo de launchers externos (Prism/Lunar) diferido 3 s para no competir con el arranque.

## [7.7.1] - 2026-06-30

### Added
- **Ajustes â†’ Cliente PvP 1.21.11**: verificar y sincronizar el mod modern sin reinstalar el launcher (igual que 1.8.9).

### Fixed
- **Sodium corrupto** en instancias 1.21.11: el launcher ya no parchea `sodium-options.json` con esquema invÃ¡lido; borra configs rotas al lanzar.

## [7.7.0] - 2026-06-30

### Added
- **Perfil unificado PvP 1.8.9** con selector de destino: Hypixel, favorito, solo menÃº o **PrÃ¡ctica PvP** (mundo flat).
- **Paraguacraft PvP 1.21.11**: instancia Fabric+Iris auto-creada, mods HUD por tier de PC (baja/media/alta).
- **Fase 2 cliente 1.21.11**: mod `ParaguacraftPvP-Modern` (FPS, ping, keystrokes) vÃ­a manifest remoto.
- Presets de servidores en 1.21.11: Hypixel, CubeCraft, MineLatino y favoritos.

### Changed
- Perfil entrenamiento 1.8.9: Boost FPS sin turbo Competir; sprint toggle **M** por defecto en Competir.
- **Loaders 1.21.11 separados**: `fabric-iris` (solo optimizaciÃ³n) vs `paraguacraft-pvp-modern` (cliente PvP dedicado).

### Fase 3 â€” Cliente PvP 1.21.11
- Loader propio **`paraguacraft-pvp-modern`** en el selector de versiones (como `paraguacraft-pvp` en 1.8.9).
- Perfil Inicio **Fabric + Iris** aparte del perfil **Paraguacraft PvP 1.21.11**.
- Mod modern lee `paraguacraft_modern.properties` del launcher (tier + HUD).

### Fase 4 â€” Optimizaciones 1.21.11 (superiores a 1.8.9)
- **JVM Java 21** dedicada: hasta **8 GB** heap (vs tope 4 GB en 1.8.9), **ZGC generacional** en PCs 16+ GB.
- **options.txt PvP** por tier: distancias, partÃ­culas mÃ­nimas, nubes off, entity distance scaling.
- Patch automÃ¡tico de **Lithium**, **Sodium** y **Dynamic FPS** al lanzar.
- Mod **0.2.0**: Boost FPS (preset vanilla + limpieza de memoria al cambiar mundo) e indicador en HUD.

### Pre-fase 5 â€” MenÃº PvP 1.21.11
- **MenÃº personalizado** estilo Paraguacraft/Lunar: constelaciÃ³n, logo, botones centrales.
- Barra inferior: **Skin**, **Tema** (5 presets), **Packs**, **Fabric** (Mod Menu).
- **Multijugador PvP** con Hypixel, CubeCraft, LibreCraft, Hylex y MineLatino.
- **Texture packs 1.21.11** descargados al instalar (launcher + catÃ¡logo GitHub `pvp-packs-modern-1.0`).
- Mod **0.3.0** incluye selector de packs in-game.

### Fase 5 â€” Paridad PvP 1.21.11
- **Mod Menu** (Right Shift): toggles HUD, Boost FPS, toggle sprint, acceso a Quick Play y packs.
- **Hypixel Quick Play** con reconexiÃ³n al Ãºltimo modo.
- **PrÃ¡ctica PvP**: botÃ³n en menÃº + destino launcher con mundo local y HUD de entrenamiento.
- HUD ampliado: **coordenadas**, **armadura** y **CPS**.
- Mod **0.4.0** compilado y empaquetado en `bundled/pvp-modern/`.

## Cliente PvP Modern 0.6.2 - 2026-07-18

### Fixed
- **Pantalla negra en menu**: corrige firma del mixin `MixinItemEntityRenderer` para el pipeline de render 1.21.11.

## Cliente PvP Modern 0.6.1 - 2026-07-18

### Fixed
- **Crash al iniciar (MixinWorldRendererOutline)**: el mixin de grosor del hitbox apuntaba al ordinal incorrecto del parÃ¡metro `lineWidth` en `drawBlockOutline` de 1.21.11.

## Cliente PvP Modern 0.6.0 - 2026-07-18

### Added
- Mods de prioridad alta portados: No Hurt Cam, Fullbright, FOV dinÃ¡mico, Hide titles, Scoreboard toggle, Low fire, Item physics, Old swing, Combo, TNT HUD, Chat triggers, Freelook (Alt), Pociones HUD, BrÃºjula.
- Mod Menu ampliado con toggles para todos los mÃ³dulos nuevos.
- Atajos: G fullbright, Alt freelook, RControl editar HUD, ` quick play.
- Launcher instala mods Fabric extra: Controlling, Smooth Scrolling, Zoomify (+ Mod Menu, AppleSkin, Better Ping, Shulker Tooltip, Dynamic FPS).

### Notes
- Fondos de tema PNG embebidos si estÃ¡n en `assets/paraguacraftpvp-modern/textures/gui/`.
- Filtro avanzado del scoreboard (ocultar stats/nÃºmeros rojos) pendiente de paridad total 1.8.9.

## Cliente PvP Modern 0.5.0 - 2026-07-18

### Added
- **Mod Menu** estilo Lunar con categorÃ­as, cards ON/OFF y botÃ³n **Editar HUD** (arrastrar mÃ³dulos).
- **Skin Changer** por URL o nick (minotar) en lugar del menÃº vanilla de capas.
- HUD ampliado: iconos de armadura, objeto en mano, recursos BedWars, overlay mÃºsica (IPC launcher).
- **Hitbox azul** cyan al apuntar bloques; **FPS/Ping/CPS** con etiquetas estilo 1.8.9.
- Keystrokes 20Ã—20 (WASD + LMB/RMB) como 1.8.9.

### Fixed
- **Toggle sprint legacy**: W activa sprint automÃ¡ticamente.
- **Resource packs**: aplicaciÃ³n real vÃ­a perfiles habilitados + refresh.
- **Hypixel Quick Play**: conecta a Hypixel y ejecuta el comando al entrar.
- **PrÃ¡ctica PvP flat**: borra mundo corrupto sin `level.dat` y crea flat automÃ¡tico.
- **MenÃº alternante** vanilla/custom: `CustomTitleScreen` extiende `TitleScreen`.
- **Layout responsive** del menÃº principal segÃºn escala GUI.
- **Tier hardware**: PCs con 16â€“32 GB RAM ya no reciben preset â€œbajaâ€ (R7 5700G + 32 GB â†’ alta).

### Changed
- Preset grÃ¡fico **media** menos agresivo (render 12, sim 10, ImmediatelyFast activo).

## Cliente PvP Modern 0.4.2 - 2026-06-30

### Fixed
- **Texto invisible en el menÃº**: colores del tema sin canal alpha (1.21.11 ignora drawText si alpha = 0).
- **Logo rosa/negro**: textura del mod dibujada con `drawTexture` en lugar de `drawGuiTexture` (solo sprites del atlas).
- HUD: colores de FPS, ping, CPS, coords y armadura con alpha completo.

## Cliente PvP Modern 0.4.1 - 2026-06-30

### Fixed
- **Logo Paraguacraft** cargaba mal (namespace de assets incorrecto).
- **MenÃº compacto**: botones planos estilo Lunar, sin bloques de piedra gigantes ni solapamiento.
- Layout en 3 filas: juego principal, utilidades (Mod Menu/Opciones/Salir) y barra Skin/Tema/Packs/Fabric.

## Cliente PvP Modern 0.4.0 - 2026-06-30

### Added
- Mod Menu estilo Lunar (Right Shift).
- Hypixel Quick Play con estado persistente.
- Mundo de prÃ¡ctica PvP (flat) desde menÃº o launcher.
- HUD: coordenadas, armadura, CPS y toggle sprint configurable.

## Cliente PvP Modern 0.3.0 - 2026-06-30

### Added
- MenÃº principal y pausa personalizados (mixin TitleScreen / GameMenuScreen).
- Pantallas: tema, multijugador PvP, texture packs.
- CatÃ¡logo embebido + remoto de 5 packs PvP 1.21.11.

## Cliente PvP Modern 0.2.0 - 2026-06-30

### Added
- **Boost FPS**: aplica grÃ¡ficos rÃ¡pidos, partÃ­culas mÃ­nimas y distancias PvP al iniciar.
- Limpieza de memoria al entrar a mundo/servidor.
- Badge **Boost** en el HUD (media/alta).

### Changed
- Launcher sincroniza flags de rendimiento en `paraguacraft_modern.properties`.

## Cliente PvP Modern 0.1.0 - 2026-06-30

### Added
- Mod Fabric **Paraguacraft PvP Modern** para 1.21.11: HUD FPS, ping y keystrokes bÃ¡sicos.

## Cliente PvP 2.1.29 - 2026-06-30

### Added
- **Hypixel Quick Play**: botÃ³n reconectar al Ãºltimo modo jugado.
- **PrÃ¡ctica PvP**: mundo flat con reglas PvP (keepInventory, sin mobs/regeneraciÃ³n).
- Destino launcher Â«PrÃ¡ctica PvPÂ» abre el mundo automÃ¡ticamente.

### Changed
- Filtro de scoreboard Hypixel ampliado (quests, rank, daily reward, etc.).
- Perfil Competir fuerza scoreboard limpio y toggle sprint modo **M** (legacy **N** off).

## [7.6.0] - 2026-06-30

### Added
- **Contenido estilo Modrinth**: mods, shaders y resource packs muestran nombre, autor, descripciÃ³n e icono original (API Modrinth + `pack.png` local).
- **Compatibilidad de mods** en chequeo pre-lanzamiento segÃºn loader y versiÃ³n MC.

### Fixed
- **Resource packs** ya no se borran al aplicar skin offline ni al togglear desde el launcher (`options.txt` merge correcto).
- **Resource packs en PvP 1.8.9 y 1.21.x**: activaciÃ³n/desactivaciÃ³n sincronizada con `options.txt`.
- **Servidores Fabric**: preparaciÃ³n detecta `fabric-server-launch.jar`; playit.gg se auto-inicia al arrancar el servidor.
- **Playit.gg**: no devuelve direcciÃ³n obsoleta como si fuera la actual.

## [7.5.0] - 2026-06-30

### Added
- **DiagnÃ³stico post-crash liviano**: banner con causa, hints y enlace a instancia/Paraguabot (sin abrir el bot automÃ¡ticamente).
- **Conflictos de mods**: avisos on-demand (duplicados, OptiFine+Iris, Essential/Patcher).
- **Bandeja ultra-lite**: icono en systray mientras jugÃ¡s; restaurar launcher con un clic.

## [7.4.0] - 2026-06-30

### Added
- **Chequeo pre-lanzamiento** on-demand: Java, cliente PvP, espacio en disco y tips de antivirus.
- **Peso de instancia** (liviano / medio / pesado) segÃºn mods, shaders y RAM.
- **Perfiles de juego** 1 clic en Inicio: Hypixel PvP, PvP prÃ¡ctica, Vanilla e Iris/Modpack.

## [7.3.0] - 2026-06-30

### Added
- **Modo Competir** (PvP): un clic orquesta cierre del launcher al jugar, Game Mode,
  prioridad Java alta, RAM/GC por hardware, perfil cliente Boost FPS y actualizaciones
  PvP diferidas hasta cerrar Minecraft.
- **Presupuesto de recursos** en detalle de instancia: launcher, Java y RAM libre.
- **MÃºsica smart**: defaults por gama baja/media; IPC overlay solo si HUD mÃºsica/hardware ON.
- **Actualizaciones diferidas**: no sync PvP ni chequeo de launcher mientras hay partida.

## [7.2.19] - 2026-07-01

### Fixed
- **Paraguabot**: carga `.env` desde `launcher/.env` aunque el cwd sea `src-tauri`; panel en
  Ajustes para guardar Groq API key; conocimiento embebido del launcher y cliente PvP 2.1.28.
- **Paraguabot**: consultas generales ya no quedan bloqueadas por un diagnostico de crash previo.

### Changed
- **Web**: modo claro en [paraguacraft.pages.dev](https://paraguacraft.pages.dev) â€” deploy desde repo [paraguacraft-web](https://github.com/SantiJ10/paraguacraft-web) (separado del launcher).

## Cliente PvP 2.1.28 - 2026-06-30

### Added
- **Dos modos de Correr toggle** para testeo A/B: nuevo (teclas virtuales, **M**) y legacy
  (`setSprinting`, **N**), cada uno con tarjeta en Mod Menu y atajo configurable.

## Cliente PvP 2.1.27 - 2026-06-30

### Fixed
- **Toggle Sprint / Toggle Sneak**: sin lag de 1 tick; las teclas virtuales se aplican al
  inicio de `onLivingUpdate` (patrÃ³n Lunar) para velocidad de sprint vanilla y sneak instantÃ¡neo.

## Cliente PvP 2.1.26 - 2026-06-30

### Fixed
- **Toggle Sprint / Toggle Sneak rotos** en 2.1.24â€“2.1.25: el mixin no se aplicaba (punto de
  inyecciÃ³n invÃ¡lido). Ahora sneak/sprint se aplican al final de `onLivingUpdate`.
- **DetecciÃ³n de actualizaciÃ³n**: el launcher consulta todos los mirrors del manifest y elige
  la versiÃ³n mÃ¡s nueva (evita CDN de raw.githubusercontent con manifest viejo).

## Cliente PvP 2.1.25 - 2026-06-30

### Added
- **Toggle Sprint en Mod Menu** (categorÃ­a PvP): activar/desactivar sin depender de la tecla V.

## Cliente PvP 2.1.24 - 2026-06-30

### Fixed
- **Toggle Sneak**: respuesta instantÃ¡nea (como Lunar); sneak/sprint se aplican despuÃ©s de
  `updatePlayerMoveState` vÃ­a mixin, no antes donde vanilla los pisaba cada tick.
- **AuditorÃ­a de input**: eliminados usos rotos de `setKeyBindState` para sprint; sneak solo
  bloquea Shift fÃ­sico de forma intencional.

## Cliente PvP 2.1.23 - 2026-06-30

### Fixed
- **Toggle Sprint**: ya no simula la tecla Ctrl (se pisaba cada frame); ahora activa
  el sprint al moverse hacia adelante con W cuando el toggle estÃ¡ ON (tecla V).

## Cliente PvP 2.1.22 - 2026-06-30

### Fixed
- **Skins Steve + nametags corruptos en lobby Hypixel**: el reset GL tras logos/ping
  enlazaba el atlas de bloques en vez de la fuente ASCII; el siguiente jugador heredaba
  textura/color sucios. Ahora se re-enlaza `ascii.png` y se resetea al terminar cada
  `RenderPlayer#doRender`.

## [7.2.18] - 2026-06-30

### Fixed
- **Cliente PvP**: manifest remoto sin BOM UTF-8; mirrors (GitHub, jsDelivr) y manifest
  embebido en `bundled/pvp` cuando falla la red. Detecta correctamente **2.1.21**.

### Changed
- **Cliente PvP 2.1.21**: Hytils preconfig (solo camas), OneConfig sin Right Shift.

## [7.2.17] - 2026-06-30

### Changed
- **Cliente PvP 2.1.20**: camas coloreadas en BedWars con **Hytils Reborn** (GPL);
  eliminado el sistema custom de sprites de camas que causaba crashes.

### Added
- **Ajustes â†’ Cliente PvP**: versiÃ³n publicada vs instalada, sincronizaciÃ³n manual y nota
  de que el cliente se actualiza sin recompilar el launcher.
- **Modo offline mejorado**: jugar con instancias y mods ya instalados sin internet;
  skins Premium se aplican localmente y se encolan para Mojang al reconectar; tokens
  Microsoft en cachÃ© si no hay red.

### Changed
- Launcher **7.2.17**.

## [7.2.16] - 2026-06-30

### Fixed
- **Crash al entrar a Hypixel (cliente 2.1.19)**: los sprites de cama coloreados ya no
  apuntan a PNG inexistentes en `paraguacraft:textures/beds/`; cargan desde la textura
  vanilla y se recolorean despuÃ©s del stitch del atlas.
- **HUD de hardware â€” GPU como CPU/RAM**: el overlay muestra **% de uso** de la GPU (y
  temperatura cuando estÃ¡ disponible), no el nombre de la placa. En Windows se lee con
  contadores de rendimiento / `nvidia-smi`.
- **Skins Steve corruptas en lobby de Hypixel**: se restaura el estado OpenGL tras dibujar
  logos/ping en nametags (la textura quedaba mal enlazada y rompÃ­a el siguiente jugador
  renderizado; Alex no se veÃ­a afectado).
- **Camas siempre rojas en BedWars**: detecciÃ³n de equipo en Hypixel (sidebar con âœ“, lana en
  inventario, bloques cercanos) y sprites de cama recoloreados por equipo, estilo
  Lunar/Badlion.

### Changed
- **Cliente PvP 2.1.19** (hotfix de camas sobre 2.1.18).

## [7.2.15] - 2026-06-30

### Fixed
- **Pantalla renderizada "en la esquina"** (cuadrante con el resto en negro): self-heal de
  viewport: si el tamaÃ±o real de la ventana no coincide con el framebuffer, se fuerza el
  resize. Corrige el problema al usar Windowed Fullscreen y tambiÃ©n con escalado DPI de
  Windows (tÃ­pico en laptops con pantalla a 125/150%).
- **HUD de hardware con datos correctos**:
  - **CPU/RAM ya no marcan "demasiada carga"**: el % de CPU se medÃ­a con un `System` nuevo
    en cada lectura (sysinfo necesita dos muestras para el delta). Ahora se reutiliza una
    instancia persistente, asÃ­ el % es real.
  - **GPU identificada de verdad**: se muestra el nombre real de la placa leÃ­do por OpenGL
    (`GL_RENDERER`), que funciona en cualquier PC (antes la GPU iba fija en "-").

### Changed
- **Cliente PvP 2.1.17**.

## [7.2.14] - 2026-06-30

### Added
- **Limitador de FPS en segundo plano** (estilo Lunar/Badlion): cuando la ventana estÃ¡
  **minimizada** se baja el tope de FPS (default 5). Reduce uso de CPU/GPU y, sobre todo
  en laptops, evita el *thermal throttling* que despuÃ©s tira los FPS en partida. OpciÃ³n
  extra para limitar tambiÃ©n **sin foco** (default off, para no molestar al borderless con
  el juego visible en otro monitor). Configurable desde el Mod Menu (Rendimiento).

### Fixed
- **Windowed Fullscreen**: al activarlo desde el menÃº ya no se ve la ventana "achicada"
  hasta tocar F11; ahora aplica el resize del framebuffer al instante.

### Changed
- **Cliente PvP 2.1.16**.

## [7.2.13] - 2026-06-30

### Added
- **Windowed Fullscreen (pantalla completa en ventana)** estilo Patcher/Lunar en el
  cliente PvP. Reemplaza el fullscreen exclusivo (F11) por una ventana sin bordes del
  tamaÃ±o del escritorio: permite **alt-tab instantÃ¡neo**, enfocar otras ventanas y que
  **OBS/Discord** sigan capturando como ventana de juego, sin el parpadeo del modo
  exclusivo. Implementado con LWJGL2 (propiedad `undecorated` + `setFullscreen(false)`),
  no usa hacks nativos frÃ¡giles. Se activa desde el Mod Menu (categorÃ­a MecÃ¡nicas) y se
  aplica al instante; tambiÃ©n cambia el comportamiento de F11.

### Changed
- **Limpieza automÃ¡tica de restos de Essential/Patcher**: el launcher borra el JAR de
  Essential y sus carpetas de datos (`essential/`, `ModCoreOSS/`, config de Patcher) de
  las instancias existentes, evitando el login y los reinicios que dejaba.
- **Cliente PvP 2.1.15**.

### Performance
- Optimizaciones propias revisadas y mantenidas: culling de entidades **sin parpadeo de
  jugadores** (nunca se cullan), nametags/armorstands/itemframes/tile-entities, lÃ­mite de
  partÃ­culas, skip de FX de combate y limpieza de memoria. Preset automÃ¡tico por gama de
  hardware (LOW/MEDIUM/HIGH) para buen rendimiento desde laptops 8 GB hasta PCs de gama alta.

## [7.2.12] - 2026-06-30

### Removed
- **Patcher (Sk1er) / Essential** eliminado del cliente PvP. El Patcher actual depende de
  Essential (login, cosmÃ©ticos y un reinicio del juego al arrancar) y no existe una versiÃ³n
  standalone viable. Para un cliente PvP limpio se quita por completo; el launcher purga el
  JAR de Patcher de las instancias existentes automÃ¡ticamente.

### Changed
- El cliente queda con **OptiFine + optimizaciones propias pulidas**: culling de
  entidades (jugadores nunca se cullan), nametags/armorstands/itemframes/tile-entities,
  lÃ­mite de partÃ­culas, skip de FX de combate y limpieza de memoria al cambiar de mundo
  (GC solo al descargar mundo, sin freeze al entrar a la partida).
- **Cliente PvP 2.1.14**.

## [7.2.11] - 2026-06-30

### Fixed
- **Crash al iniciar** (no llegaba al menÃº principal): el preset Boost FPS ponÃ­a
  `mipmapLevels = 0`, lo que provoca `ArrayIndexOutOfBoundsException` al generar
  mipmaps del atlas de texturas en 1.8.9. Ahora se usa mÃ­nimo 1 y se corrige
  automÃ¡ticamente si el perfil quedÃ³ en 0.

### Changed
- **Cliente PvP 2.1.13**.

## [7.2.10] - 2026-06-30

### Performance
- **Boost FPS reforzado** (preset estilo Lunar/Badlion):
  - OptiFine **Fast Render** + Render Regions + Smart Animations + Fast Math.
  - Apagado de nubes, lluvia, clima, estrellas, sky custom, partÃ­culas de agua/void/portal.
  - GrÃ¡ficos en **Fast**, Smooth Lighting (AO) **off**, mipmaps **off**, AA/AF al mÃ­nimo.
- Se quita el **System.gc() forzado** al cambiar de mundo: provocaba un freeze al
  entrar a la partida. La memoria ya la maneja Patcher (leaks + world swapping).
  Se conserva solo la limpieza barata de display lists.

### Changed
- **Cliente PvP 2.1.12**.

> Nota: las opciones de OptiFine se aplican al **siguiente reinicio** del juego.

## [7.2.9] - 2026-06-30

### Removed
- **Borderless** (Win32 propio) eliminado del cliente: lo maneja **Patcher** (Windowed Fullscreen), que lo hace mejor y evita conflictos de ventana/captura. Se borraron `BorderlessWindowManager` y `Win32Helper`.
- **Chat compacto** propio eliminado: lo maneja **Patcher** (Compact Chat). Evita doble procesamiento del chat. Se borraron `MixinGuiNewChat` y `CompactChatHandler`.

### Changed
- **Cliente PvP 2.1.11**.

## [7.2.8] - 2026-06-30

### Added
- **Patcher (Sk1er) integrado** al cliente PvP: coremod 1.8.9 que arregla varios bugs de render y mejora rendimiento. Se distribuye junto con OptiFine en `bundled/pvp` (no en Releases).

### Fixed
- **CompaÃ±eros que desaparecian y volvian en partida**: el culling de entidades hacia un chequeo de frustum sobre jugadores lejanos y parpadeaban. Ahora los jugadores NUNCA se cullan (el culling solo aplica a mobs/objetos).
- **Skins rotas en salas de espera**: se endurecio el estado GL al dibujar logos/ping en los nametags (push/pop + reset de color y textura) para que el modelo/nametag siguiente no herede estado sucio.

### Changed
- **Cliente PvP 2.1.10**.

## [7.2.7] - 2026-06-29

### Added
- **Titulos al chat** (mod nuevo): los carteles gigantes del centro ("FINAL KILL", "LA CAMA HA SIDO DESTRUIDA", "TRAMPA ACTIVADA", etc.) ya no tapan la pantalla; se muestran solo en el chat. Toggle en el menu (categoria PvP), activado por defecto.
- **Overlay de musica con YouTube**: el mismo overlay (launcher e in-game) ahora muestra titulo + caratula (miniatura) tambien para YouTube y YouTube Music, igual que Spotify. Los dos reproductores funcionan de forma independiente.

### Fixed
- **Texturas/cielo rotos al terminar la partida (Bedwars)**: se quito el forzado de hora del mundo (`setWorldTime(6000)` cada tick) que peleaba con las actualizaciones del servidor y la animacion de cielo del fin de partida (causa del parpadeo). Reset GL del HUD mas estricto (texture2d, alpha, blendFunc, depthMask).
- **YouTube dejaba de sonar si Spotify estaba conectado**: el audio de YouTube estaba bloqueado globalmente al conectar Spotify. Ahora son independientes.
- **Borderless no se podia activar**: al fallar un intento el toggle se apagaba solo. Ya no se auto-desactiva (conserva la intencion y reintenta al iniciar). Busqueda de ventana por proceso priorizada (mas fiable en LWJGL2).

### Changed
- **Cliente PvP 2.1.9**.

## [7.2.6] - 2026-06-29

### Fixed
- **Freelook**: ahora gira SOLO la camara y no el cuerpo del jugador. El bug venia de un `@Redirect` sobre `EntityPlayerSP.setAngles` (metodo heredado de `Entity`, no se mapeaba). Se intercepta `Entity.setAngles` directamente.
- **Freelook indetectable**: el cuerpo nunca rota, el servidor recibe la rotacion real congelada y los raytrace/interacciones usan la rotacion real (la camara solo se sobreescribe al renderizar). Camara interpolada para movimiento fluido.

### Changed
- **Cliente PvP 2.1.8**.

## [7.2.5] - 2026-06-29

### Added
- **Chat Alerts**: comando `/chat alerts add <palabra>` con sonido (ding) y resaltado de mensajes en chat.
- **Combo y Reach** arrastrables en el editor de HUD.

### Fixed
- **Borderless**: ventana compatible con captura por ventana y overlay de Discord (WS_EX_APPWINDOW, sin fullscreen exclusivo DWM).
- **Scoreboard**: filtro Unicode para barras de progreso de Hypixel (cuadrados cian).
- **HUD de musica**: texto recortado al ancho del panel; fondo opaco por defecto.
- **Freelook**: sensibilidad corregida; cuerpo del jugador no rota mientras la camara esta libre.
- **Sky flicker**: encapsulado GL en HUDOverlay para evitar fugas de estado OpenGL.

### Changed
- **Cliente PvP 2.1.7**.

## [7.2.4] - 2026-06-29

### Fixed
- **Crash en Hypixel/servidores (Batching chunks)**: `MixinBedColor` usaba `GlStateManager.color()` en hilos de chunk sin contexto OpenGL. Ahora tinta vertices via `WorldRenderer.color()` (seguro en ChunkRenderWorker).

### Changed
- **Cliente PvP 2.1.5**.

## [7.2.3] - 2026-06-29

### Fixed
- **Crash al cargar (no llegaba al menu)**: `MixinGuiNewChat` usaba el tipo accessor `IChatLineAccess` como variable local, lo que rompia el remapeo del mixin en `GuiNewChat`. Ahora usa los getters publicos de `ChatLine` y refresca el chat con `refreshChat()`. Eliminado el accessor `IChatLineAccess`.

### Changed
- **Cliente PvP 2.1.4**.

## [7.2.2] - 2026-06-29

### Fixed
- **Crash codigo 1 (definitivo)**: `MixinNametagLogo` ya no usa `@ModifyVariable` (firma invalida que abortaba el arranque). El ping rival ahora se dibuja de forma aditiva a la derecha del nombre.

### Changed
- **Cliente PvP 2.1.3**.
- El cliente PvP se sirve desde `bundled/pvp` (repo + bundle embebido), **no** desde GitHub Releases. Solo el instalador del launcher se publica como release.

## [7.2.1] - 2026-06-29

### Fixed
- **Crash codigo 1 al iniciar**: `MixinNametagLogo` fallaba al inyectar ping rival (`ModifyVariable` con firma invalida).
- Accessor `IChatLineAccess` registrado en mixin config (chat compacto).

### Changed
- Cliente PvP **2.1.2**.

## [7.2.0] - 2026-06-29

### Fixed
- **Crash al iniciar Minecraft (codigo 1)**: mixins de camas coloridas y freelook reescritos con firmas correctas para 1.8.9.
- **MixinBedColor**: ahora usa `renderBlock` con BlockPos (antes apuntaba a un metodo inexistente).
- **Freelook**: swap temporal de rotacion en `orientCamera` en lugar de redirects de campo inestables.
- **Ping rival**: fusionado en un solo mixin de nametag para evitar conflictos en `Render`.
- **Alertas chat**: corregido NPE al abrir el editor de reglas.

### Changed
- Cliente PvP **2.1.1** con manifest `pvp-client-2.1.1`.

## [7.1.9] - 2026-06-29

### Added
- **Cliente PvP 2.1.0**: chat compacto, ping rival, camas coloridas, freelook, reach/combo HUD, fisica de items, categoria Hypixel en Mod Menu.
- **Juego rapido Hypixel**: menu con iconos de items y comandos `/play` / `/lobby` (tecla `` ` ``).
- **Alertas de chat configurables**: reglas personalizables en `paraguacraft/chat_triggers.json` (estilo ChatTriggers simplificado).

### Changed
- Manifest PvP actualizado a `pvp-client-2.1.0` con SHA-1 verificado.
- Fallback embebido del launcher apunta a `ParaguacraftPvP-2.1.0.jar`.

### Security / Hypixel
- Todos los mods del cliente son **solo cosmeticos/HUD** â€” sin reach hack, xray, autoclicker ni macros.
- Reach Display y Combo Counter solo **muestran** datos de tus golpes; no alteran hitboxes ni paquetes.

## [7.1.8] - 2026-06-29

### Added
- **Traducciones vanilla es_ES y es_AR** embebidas en el cliente PvP.

## [7.1.2] - 2026-06-25

### Fixed
- **JVM PvP 1.8.9**: flags G1 solo compatibles con Java 8 (evita Â«Could not create the Java Virtual MachineÂ» en PCs con mucha RAM).
- Sin `-Xmx`/`-Xms` duplicados del perfil Forge cuando aplica preset PvP.
- VerificaciÃ³n SHA del mod PvP: solo acepta el hash del manifest (fuerza actualizaciÃ³n del JAR viejo).

## [7.1.1] - 2026-06-19

### Added
- **Cliente PvP 2.0.0** recompilado: HUD/GUI Lunar, perfiles, keybinds, resource packs, badge sync, optimizaciones Fase C/D.

### Fixed
- **Auto-update PvP**: el SHA del manifest remoto manda sobre el JAR embebido; Play descarga el cliente nuevo aunque el instalador sea viejo.
- JAR embebido actualizado (`04aee52fâ€¦`) para instalaciones offline.

## [7.1.0] - 2026-06-25

### Added
- **Cliente PvP dinÃ¡mico**: versiÃ³n, release y mods desde `manifest.json` remoto; elimina JARs viejos al actualizar.
- **Perfil JVM PvP 1.8.9**: RAM/G1GC por gama de hardware (Java 8).

### Changed
- README orientado al jugador (intro, features, instalaciÃ³n) como la web.
- Manifest PvP con `client_version` y `release_tag`.

### Fixed
- SHA-1 del manifest remoto vs JAR embebido; prioridad local + fallback.

## [7.0.5] - 2026-06-24

### Fixed
- **Lanzamiento 1.8.9 en Windows**: Java 8 ya no usa `@args.txt` (solo Java 9+); corrige cierre instantÃ¡neo sin logs.
- **Java por versiÃ³n**: el override global en Ajustes no bloquea Java 8 en instancias 1.8.9 si tenÃ©s Java 21 para 1.21.
- **TÃ­tulo de ventana**: solo Â«Paraguacraft PvPÂ» en 1.8.9 PvP; otras versiones muestran Â«Paraguacraft X.X.XÂ».

### Changed
- **PvP Client 2.0.0**: texture packs desde GitHub Release y Google Drive (sin Modrinth en el gestor del mod).
- Instancias PvP oficiales (`Paraguacraft_1.8.9_PvP`) separadas de carpetas de prueba.

## [7.0.4] - 2026-06-24

### Fixed
- **Paraguabot / crash falso**: ignora el "Crash Report" de la pantalla de carga de Forge 1.8.9 (no es un error real).
- No usa crash-reports viejos de sesiones anteriores al diagnosticar.
- Forge 1.8.9 que cierra con cÃ³digo 1 pero log `Stopping!` ya no marca "El juego terminÃ³ con error".

## [7.0.3] - 2026-06-24

### Added
- **Paraguacraft PvP Client 2.0.0**: descarga automÃ¡tica desde release `pvp-client-2.0.0` (Forge 1.8.9 + OptiFine + mod 2.0.0).
- **Texture packs PvP**: catÃ¡logo remoto + release `pvp-packs-1.0` en GitHub (9 packs); Faithful y Tightfault vÃ­a Modrinth.
- **Fallback local** de mods PvP: `%APPDATA%/ParaguacraftLauncher/bundled/pvp` si GitHub no responde.
- Scripts `publish-pvp-client.ps1` y `publish-pvp-packs.ps1` (GitHub CLI, detecta `gh` sin PATH).

### Changed
- Manifest y catÃ¡logo PvP actualizados a 2.0.0 (`clientes/paraguacraft-pvp/`).
- ReparaciÃ³n de instancias: corrige meta PvP inferida incorrectamente desde nombre de carpeta.

### Fixed
- Descarga del mod PvP fallaba con HTTP 404 en PCs nuevas (release inexistente en GitHub).

## [7.0.2] - 2026-06-23

### Added
- Reparar instancia, visor de logs y presets RAM.
- Gestor de mods por instancia, modpacks CurseForge `.zip`, exportar instancia.
- Servidores favoritos con join directo, feedback de descargas mejorado.
- CI release unificado y `latest.json` con firma opcional.

## [7.0.1] - 2026-06-19

### Fixed
- Auto-update: abre el instalador NSIS visible (ya no usa `relaunch` del plugin ni `/S` silencioso).
- Auto-update: cierra el launcher tras lanzar el setup para permitir reinstalar.
- CI release: genera `latest.json` con hash real y lo sube al release automÃ¡ticamente.

## [7.0.0] - 2026-06-19

### Fixed
- PvP 1.8.9: OptiFine y cliente desde `bundled/pvp` en GitHub (sin depender de optifine.net).
- Lanzamiento Java 8: flags JVM compatibles; resoluciÃ³n de Java para instaladores Forge/OptiFine.
- Modpacks Modrinth: reintentos en red, User-Agent correcto, loader instalado antes de bajar mods.
- CurseForge: API key embebida en builds de release (CI + compile-time).

## [6.9.0] - 2026-06-22

### ðŸš€ Lanzamiento mayor â€” Paraguacraft Launcher (Tauri v2)

Reescritura completa del launcher: **Rust + Tauri v2 + Vue 3**. Cliente multiplataforma de grado comercial.

#### Rendimiento
- **0% CPU en segundo plano** mientras jugÃ¡s: el runtime apaga red, caches y hilos al lanzar Minecraft.
- **Descargas async ultrarrÃ¡pidas** con Rust (`reqwest`), concurrencia acotada, SHA-1 y escritura atÃ³mica.
- **OptimizaciÃ³n automÃ¡tica** de RAM/JVM segÃºn hardware (gama baja, media y alta).

#### Tienda y contenido
- Tienda nativa **Modrinth + CurseForge** (mods, modpacks, shaders, resource packs, datapacks, plugins).
- **Modpacks `.mrpack`**: crea instancia completa (index, overrides, loader).
- **Plugins** â†’ servidores locales (`plugins/` o `mods/`); **datapacks** â†’ instancia o servidor + mundo.
- Modpacks filtran **versiÃ³n MC y loader reales** del proyecto (ej. Fabulously Optimized = Fabric, Zombie Invade = Forge).

#### Servidores y extras
- Servidores locales Paper/Fabric/Forge + **Playit.gg invisible**.
- **DiagnÃ³stico de crashes con IA** al salir del juego.
- Instancias aisladas (PvP 1.8.9, modpacks pesados).
- Auto-update integrado (`tauri-plugin-updater` + fallback GitHub Releases).

---

## [6.8.0] - 2026-06-17

### ðŸ”§ Mejoras

- **UI del panel Versiones**: eliminada la tarjeta fija "Cliente Paraguacraft PvP" (botÃ³n "Descargar y Jugar") que se mostraba en todas las versiones aunque ya existÃ­a el loader **PvP** en 1.8.9. Ahora el flujo es: versiÃ³n `1.8.9` â†’ loader `PvP` â†’ **JUGAR**.

---

## [6.7.0] - 2026-06-17

### âœ¨ Novedades

- **Loader PvP (solo 1.8.9)**: nuevo motor en el selector de versiones. Instala Minecraft 1.8.9 + Forge `11.15.1.2318` + OptiFine HD U M5 + mod cliente `ParaguacraftPvP-1.0.0.jar` en la instancia `Paraguacraft_1.8.9_PvP`.
- **Descarga remota del cliente PvP**: `ParaguacraftPvP-1.0.0.jar` se obtiene desde GitHub (`bundled/pvp/` en `main`, con fallback a release `pvp-client-1.0.0`). VerificaciÃ³n SHA-1 y cachÃ© global en `.minecraft/Paraguacraft_cache/pvp/`.
- **Panel Cliente Paraguacraft PvP**: en Versiones â†’ 1.8.9, tarjeta dedicada para preparar el perfil, jugar directo y reparar loader + mods.
- **Preset hardware PvP Solo**: asistente de rendimiento actualizado a `1.8.9 Â· PvP` con instalaciÃ³n automÃ¡tica del bundle completo.

### ðŸ”§ Mejoras

- **OptiFine HD U M5**: descarga desde BMCL API al preparar el perfil PvP (no hace falta copiar el JAR manualmente).
- **Reparar loader PvP**: reinstala Forge `11.15.1.2318` y vuelve a sincronizar los mods del cliente.
- **Motores unificados**: el launcher reconoce tanto `PvP` como `Paraguacraft PvP` para la misma instancia.
- **Auto-actualizaciÃ³n Windows**: siempre descarga y ejecuta `Instalar_Paraguacraft_vX.exe` desde `paraguacraft.pages.dev/latest.json` (manifest apunta al instalador Inno Setup, no al portable).

---

## [6.6.0] - 2026-05-28

### ðŸ¤– Bot Discord

- **RCON no bloquea mÃ¡s el event loop**: todas las llamadas a `MCRcon` (vigilar jugadores, `!partido`, `!estado`, `!cmd`, `!dia`, `!noche`, `!sol`, `!kick`, `!ban`, `!op`, `!dificultad`, `!gamemode`, `!tp`, `!tpall`, `!anuncio`, `!guardar`) ahora corren en `asyncio.to_thread()`. El heartbeat de Discord ya no se bloquea cuando el servidor MC estÃ¡ caÃ­do o tarda en responder.
- **`!partido` y `buscar_proximo_partido` con fallback web**: si la API de fÃºtbol no tiene registrado el partido (o devuelve vacÃ­o), el bot busca automÃ¡ticamente en DuckDuckGo y muestra los resultados con embed.
- **CachÃ© de API de fÃºtbol corregida**: respuestas vacÃ­as ya no se guardan por 60 minutos â€” expiran en 2 min para reintentarse pronto. Los errores de API (rate limit, acceso denegado) directamente no se cachean.
- **Aliases de equipos ampliados**: `boca juniors`, `paris saint germain`, `paris sg`, `paris saint-germain` agregados al diccionario de alias para bÃºsquedas mÃ¡s robustas.

---

## [6.5.0] - 2026-05-26

### ðŸ› Arreglos
- **Lanzamiento desde Biblioteca**: instancias y modpacks ahora inician correctamente al presionar â–¶ en la tarjeta o â–¶ Jugar en el detalle. El botÃ³n muestra spinner y se restaura automÃ¡ticamente al cerrar el juego.
- **Spinner persistente**: el spinner del botÃ³n de lanzamiento ahora dura hasta que Minecraft se cierra (no desaparece despuÃ©s de 10 min para modpacks pesados â€” nuevo timeout de 60 min).
- **Servidor Fabric**: `iniciar_servidor` detecta y usa `fabric-server-launch.jar` correctamente. Antes intentaba usar `server.jar` que no existe en servidores Fabric.
- **Plugins en servidor Fabric**: instalaciÃ³n de mods/plugins en servidores Fabric ahora apunta a `/mods` en vez de `/plugins`. Afectaba tambiÃ©n a Geyser, listado, eliminaciÃ³n, toggle y subida de plugins.
- **DetecciÃ³n de tipo servidor**: nuevo helper `_es_fabric_servidor` unifica la detecciÃ³n (por archivo JAR o por `_paragua_srv.json`) en lugar de 7 chequeos dispersos.
- **Badge "corriendo" en Biblioteca**: corregida comparaciÃ³n de motor que nunca matcheaba para motores con espacios (ej: `Fabric + Iris`). La tarjeta ahora se resalta correctamente mientras el juego estÃ¡ activo.
- **ProtecciÃ³n contra sobreescritura de instancias**: `crear_instancia_personalizada` ahora avisa si ya existe una instancia con distinto nombre para la misma versiÃ³n/motor, en vez de sobreescribir silenciosamente.

### âœ¨ Novedades
- **Selector de versiÃ³n del loader Fabric por instancia**: en Biblioteca â†’ âš™ ConfiguraciÃ³n â†’ InstalaciÃ³n, las instancias Fabric/Quilt muestran un selector con todas las versiones del loader. Ãštil para bajar de versiÃ³n cuando un modpack requiere una versiÃ³n especÃ­fica (ej: error "Incompatible mods").

---

## [6.4.0] - 2026-05-26

### ðŸ› Arreglos
- **Nombre de modpacks**: modpacks instalados desde Modrinth (ej: Fabulously Optimized) ahora muestran su tÃ­tulo de proyecto real en vez del string de versiÃ³n (ej: "6.3.0-beta.4" o "12.1.2 for 1.21.1"). El tÃ­tulo se obtiene de la API de Modrinth al instalar y se guarda en `_paragua_instance.json`.
- **Guardar nombre en Biblioteca**: editar el nombre de una instancia desde Biblioteca â†’ âš™ ConfiguraciÃ³n ahora persiste correctamente. Antes, el nombre del modpack en `_paragua_modpacks.json` sobreescribÃ­a siempre el nombre guardado en `_paragua_instance.json` al recargar la biblioteca.
- **BotÃ³n Iniciar en tarjeta de instancia**: al hacer clic en â–¶ en la biblioteca, ahora muestra un spinner de carga, valida Java con `preflightJava` antes de lanzar (igual que el botÃ³n principal), y restaura el estado del botÃ³n al terminar. Antes no daba ninguna seÃ±al visual de que el juego estaba descargando o iniciando.

---

## [6.3.0] - 2026-05-26

### âœ¨ Novedades
- **Noticias interactivas en Inicio**: cards con imagen oficial, categorÃ­a, fecha y enlace directo a las noticias de Minecraft. Datos en tiempo real desde la API de Mojang.
- **SecciÃ³n "DescubrÃ­ en Modrinth"**: row horizontal con los 8 mods mÃ¡s descargados de Modrinth (Ã­cono, nombre, descripciÃ³n, descargas). Clic abre la tienda de mods del launcher.
- **Ãconos reales de servidores**: todos los cards de Servidores PÃºblicos y Quick Play ahora usan `api.mcsrvstat.us/icon/{ip}` â€” el Ã­cono real del servidor Minecraft, no un favicon genÃ©rico.
- **Servidores nuevos**: RedPVP (`play.redpvp.com.ar`), Minebolt (`minebolt.net`) y RhoMC (`play.rhomc.com`) agregados con Ã­conos, descripciÃ³n, badge No-Premium, ping en tiempo real y Auto-Join.
- **Auto-refresh de sesiÃ³n Microsoft**: hilo daemon que refresca el `access_token` cada 50 minutos â€” evita la expiraciÃ³n frecuente (tokens expiran a los 60 min). Arranca al cargar sesiÃ³n guardada y tras cada login exitoso. Guard flag para prevenir hilos duplicados.

### ðŸ”§ Mejoras
- **Quick Play**: eliminada la posiciÃ³n `sticky` que causaba superposiciÃ³n visual con la navbar al scrollear.
- **Auto-Join**: ahora usa `get_ultima_version_jugada()` en vez de leer el selector de versiones (que podÃ­a estar vacÃ­o). Funciona desde Servidores PÃºblicos sin necesidad de ir a Versiones primero.
- **Login Microsoft UI**: fondo del overlay de login reemplazado por `main_banner.png` con gradiente oscuro. BotÃ³n de Microsoft con Ã­cono SVG oficial.
- **Ping tracker**: RedPVP, Minebolt y RhoMC agregados a `_SERVIDORES_GLOBAL` para ping en tiempo real.
- **Refresco de token al arrancar**: si hay sesiÃ³n guardada, se lanza un refresh inmediato en background al iniciar el launcher.

### ðŸ› Arreglos
- IPs incorrectas de RedPVP/Minebolt/RhoMC en `pingTodosServidores` y `_newSrvCards` corregidas.
- `_iniciar_auto_refresh_ms` idempotente: mÃºltiples llamadas no generan hilos duplicados.

---

## [6.2.0] - 2026-05-24

### âœ¨ Novedades
- **Versiones Mojang en "Crear instancia"**: el picker de versiones ahora carga la lista completa desde `piston-meta.mojang.com` con fallback a `minecraft-launcher-lib`. Ya no aparece vacÃ­o.

### ðŸ”§ Mejoras
- Fallback a mÃºltiples URLs de manifest de versiones para mayor robustez.

---

## [6.1.0] - 2026-05-23

### âœ¨ Novedades
- **Panel "Versiones" completamente rediseÃ±ado**: grid de instancias con imagen de versiÃ³n, Ãºltimo motor, Ãºltima vez jugada, botones Jugar/Gestionar/Eliminar por card.
- **Wizard de primera vez**: guÃ­a paso a paso al abrir el launcher por primera vez (bienvenida â†’ cuenta â†’ versiÃ³n recomendada â†’ listo).

### ðŸ”§ Mejoras
- Indicadores de sesiÃ³n mejorados en el header.
- Transiciones y animaciones refinadas en la UI.

---

## [6.0.0] - 2026-05-22

### âœ¨ Novedades
- **Login Microsoft unificado (estilo ATLauncher)**: un solo botÃ³n con logo de Microsoft abre un modal con **navegador** y **cÃ³digo QR** en el mismo lugar.
- **Inicio de sesiÃ³n por QR / microsoft.com/link**: flujo device code con cÃ³digo copiable al portapapeles (y al generarse). El QR abre solo `microsoft.com/link` (sin URLs que redirigen a `login.live.com` y fallan en el celular).
- **Tienda â€” Data packs en servidor local**: selector de servidor guardado y mundo; los datapacks se instalan en `world/datapacks` del servidor (ya no en la instancia del cliente ni con selector de loaders).
- **Tienda â€” Plugins en servidor local**: selector de servidor destino en la pestaÃ±a Plugins; instalaciÃ³n en `plugins/` del servidor activo (Hangar + Modrinth).
- **Importar desde otros launchers**: nueva pestaÃ±a Extras â†’ **Importar launcher** con detecciÃ³n de TLauncher, SKLauncher y `.minecraft`, tutorial integrado y copia de saves/mods/resource packs/shaders/options/servers a la instancia activa (premium y no premium).

### ðŸ”§ Mejoras
- Login QR usa client ID compatible con device code (el client oficial del Launcher de Minecraft devolvÃ­a 401).
- CÃ³digo de verificaciÃ³n en recuadro clicable + botones **Copiar cÃ³digo** y **Abrir microsoft.com/link**.
- Datapacks en Modrinth: sin paso de â€œplataformaâ€ incompatible (`minecraft` / sin loaders).
- SesiÃ³n MS guarda `ms_client_id` para refrescar tokens segÃºn el mÃ©todo de login (navegador vs QR).
- Multi-cuenta: una sola opciÃ³n â€œAgregar cuenta Microsoftâ€ que abre el modal unificado.

### ðŸ› Arreglos
- **Overlays**: errores reales en la UI en lugar de `ok: true` silencioso; overlay de teclas sin `bind_all` global (evita conflictos/crashes con el launcher); cierre limpio del listener `pynput`.
- **Overlays**: comprobaciÃ³n de que la ventana tk se abriÃ³ antes de reportar Ã©xito.

---

## [5.9.0] - 2026-05-21

### âœ¨ Novedades
- **Tienda de plugins**: pestaÃ±a Plugins con bÃºsqueda federada **Hangar + Modrinth**, trending y instalaciÃ³n en el servidor local.
- **Actualizaciones de plugins del servidor**: detecciÃ³n y aplicaciÃ³n de updates en `plugins/`.
- **Importar servidor existente**: botÃ³n para importar una carpeta con `server.jar` / Fabric, detectar tipo y versiÃ³n, regenerar `iniciar_server.bat` y agregarlo a la lista guardada.
- **Playit.gg**: persistencia de la direcciÃ³n Java del tÃºnel por servidor en `_paragua_srv.json`.

### ðŸ”§ Mejoras
- DetecciÃ³n automÃ¡tica de versiÃ³n MC del servidor (`version_history.json`, nombre del JAR, `_paragua_srv.json`).
- RegeneraciÃ³n de `iniciar_server.bat` usando Java local del launcher cuando estÃ¡ disponible.
- Plugins Modrinth filtrados por loaders Paper-compatibles y versiÃ³n del servidor.
- Ampliaciones en la UI de la tienda de mods y panel de servidores.

### ðŸ› Arreglos
- Correcciones en flujos de instalaciÃ³n de plugins y estado del servidor activo.

---

## [5.8.0] - 2026-05-20

### ðŸ”§ Mejoras
- **RAM del servidor local**: `-Xms` y `-Xmx` ahora usan el mismo valor en modo automÃ¡tico (mitad de la RAM del sistema, tope 8 GB), evitando mÃ­nimos incoherentes que podÃ­an causar comportamiento raro al arrancar Paper/Fabric server.

### ðŸ› Arreglos
- Ajuste menor en el arranque del proceso del servidor.

---

## [5.7.0] - 2026-05-18

### ðŸ› Fixes crÃ­ticos
- **Hypixel / CubeCraft con sesiÃ³n premium**: arreglado el bug por el que servidores anti-cheat te kickeaban con `Invalid session`. Ahora el launcher refresca el token de Microsoft de forma sincrÃ³nica antes de lanzar y manda el `name` real de Mojang (no el username cacheado del config).
- **BotÃ³n "ðŸ›‘ Cerrar Minecraft" colgado**: ya no queda visible tras un cierre normal del juego. `estado_minecraft()` ahora limpia automÃ¡ticamente el handle del proceso muerto.
- **Crash analyzer: falsos positivos**: el filtro `ERROR_MARKERS` ahora exige que `at` aparezca como prefijo de lÃ­nea de stacktrace (`    at com.foo.Bar`), no en frases naturales como "OpenGL initialized at version 3.3". Reduce ~80% los falsos positivos en logs de Forge.
- **`ops.json` / `whitelist.json` / `banned-players.json` corruptos**: la escritura ahora es **atÃ³mica** (`tempfile + os.replace + fsync`) y serializada con lock. El bot de Discord y la UI ya no pueden corromperlos si escriben concurrentes.

### âš¡ Rendimiento
- **Servidor Minecraft con Aikar G1GC**: PaperMC / Vanilla / Fabric server ahora arrancan con los 21 flags de Aikar tuneados. **TPS estable garantizado en sesiones de 4 hs+** sin lagazos progresivos. Perfil "lite" automÃ¡tico para heap <4 GB. Forge sigue usando `run.bat` (no se puede inyectar sin reescribir el .bat).
- **Descarga de mods con verificaciÃ³n SHA-1**: cada `.jar` de Modrinth se baja a `.part`, se valida con el hash anunciado por la API y reciÃ©n entonces se renombra al destino final. Si el hash no coincide (MITM, corrupciÃ³n de red), se borra el `.part` y avisa. Soporta tambiÃ©n `expected_size` para detectar truncados.

### ðŸ›¡ï¸ Robustez
- **`atexit` cleanup global**: si cerrÃ¡s el launcher con un servidor MC o playit corriendo, ahora se cierran graceful (servidor recibe `stop` con timeout 15s; playit `terminate`). No quedan procesos huÃ©rfanos comiendo RAM.
- **`detener_servidor` con timeout 45s**: antes era 8s, insuficiente para servers con 10+ jugadores guardando mundos. Ahora hace escalada graceful â†’ SIGTERM â†’ SIGKILL con logging por etapa.
- **ValidaciÃ³n de sesiÃ³n MS antes de lanzar**: si el token expirÃ³ y no se pudo refrescar, se aborta el launch con un mensaje claro en vez de lanzar y crashear contra el authservice de Mojang.

### ðŸ¤– Bot Discord
- 9 comandos nuevos para administrar el servidor desde Discord: `/server-start`, `/server-stop`, `/server-restart`, `/whitelist add|remove|list`, `/op add|remove`, `/ban add|remove`. Toda escritura de JSON pasa por el `_srv_json_write` atÃ³mico.

### ðŸ§ª Tests
- `test_smoke.py` ampliado a **34 secciones** que cubren: sesiÃ³n MS, descarga atÃ³mica, crash analyzer (falsos positivos + detecciÃ³n OOM), `estado_minecraft` cleanup, regresiÃ³n Hypixel premium. Ejecutable con `.venv/Scripts/python.exe test_smoke.py`.

---

## [5.5.0] - 2026-05-04

### âœ¨ Novedades
- **Atajos de teclado globales**: `Ctrl+1..6` para navegar, `Ctrl+J` para jugar, `Ctrl+L` tienda de mods, `Ctrl+,` ajustes, `Ctrl+/` para ver todos los atajos.
- **Reporte de bugs con un clic**: nuevo botÃ³n en Extras â†’ Crash Log que arma un ZIP con logs, crash reports e info del sistema listo para adjuntar en GitHub Issues.
- **Modo sin conexiÃ³n explicativo**: el badge "Sin conexiÃ³n" ahora es clickeable y abre un modal que te dice quÃ© funciona y quÃ© no sin internet.
- **Filtros en la consola del servidor**: chips `Todo / INFO / WARN / ERROR / Chat` + buscador libre en la consola de servidores locales.
- **Changelog visible**: al actualizar el launcher, ahora vas a ver un modal con las novedades (este mismo).
- **Log interno del launcher**: nuevo botÃ³n `ðŸ“‹ Log launcher` en Extras â†’ Crash Log para abrir `paraguacraft_debug.log` (rotado, mÃ¡x. 15 MB). El log ahora se incluye automÃ¡ticamente en el ZIP de reporte de bug.
- **Importar modpacks `.mrpack`**: desde el menÃº del logo â†’ `ðŸ“¦ Importar modpack .mrpack` podÃ©s elegir un archivo local de Modrinth. El launcher detecta automÃ¡ticamente la versiÃ³n de Minecraft y el loader (Fabric / Forge / NeoForge / Quilt), descarga todos los mods con verificaciÃ³n SHA-1, aplica los `overrides/` y arma la instancia lista para jugar.

### ðŸ”§ Mejoras
- Timeouts reforzados en descargas de red para evitar cuelgues silenciosos.
- Logging estructurado: migrados los `print()` dispersos a `logging` con archivo rotatorio, facilitando diagnÃ³stico remoto.

### ðŸ› Arreglos
- Correcciones menores de UI.

---

## [5.3.0] - 2026-04

### âœ¨ Novedades
- Soporte para Minecraft 1.21.x.
- Playit.gg integrado para abrir el servidor al mundo.
- Panel de servidor local con consola en vivo y RCON.
- Skins 3D con editor bÃ¡sico.
- Tienda de mods con Modrinth y CurseForge.

### ðŸ”§ Mejoras
- Auto-reparaciÃ³n de JARs corruptos.
- DetecciÃ³n y resoluciÃ³n automÃ¡tica de conflictos de mods.

---

## [5.2.0]

### âœ¨ Novedades
- Microsoft Authentication (cuentas premium).
- Multi-cuenta.
- DetecciÃ³n automÃ¡tica de Java.

---

## [5.1.0]

### âœ¨ Novedades
- Soporte Fabric, Forge, NeoForge, Quilt.
- Descarga automÃ¡tica de Java runtime.

---

## [5.0.0]

### ðŸŽ‰ Primera release pÃºblica
- Launcher base con versiones vanilla.
- Interfaz web con pywebview.
- GestiÃ³n de skins locales.

