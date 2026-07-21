# Changelog Paraguacraft Launcher

Todos los cambios notables del launcher se documentan acá.
Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [7.9.0] - 2026-07-21

### Added
- **Tienda paginada**: paginación real (no solo destacados) en Mods, Resource Packs, Plugins, Shaders y Modpacks, con componente `<Pagination>` reutilizable.
- **Filtrado dinámico de compatibilidad**: los selectores de versión/loader en el modal de instalación solo muestran combinaciones que el proyecto realmente publica en Modrinth/CurseForge.
- **Descarga inteligente de dependencias**: modal de confirmación con checkboxes al instalar un mod con dependencias requeridas/embebidas; se puede omitir sin frenar la instalación.
- **Motor de optimización dinámica**: antes de lanzar cualquier instancia, el launcher detecta RAM/CPU y aplica argumentos JVM y perfil de `options.txt` diferenciados por gama de PC (Baja/Media/Alta) y por loader (1.8.9 Forge+OptiFine, 1.21.11 Fabric+Sodium+Iris, o genérico), sin pisar los presets PvP ya afinados.

### Changed
- **Rendimiento de la UI**: Tienda, Ajustes, Skins, Instancias, Versiones y Servidores quedan en caché (`KeepAlive` + Pinia) al cambiar de pestaña, sin repetir pantallas de carga.
- **Backend sin bloqueos**: descompresión de `.mrpack`/CurseForge y hashing de auto-update de mods ahora corren en `tokio::spawn_blocking`, para que la ventana nunca aparezca como "No responde" durante instalaciones o auto-updates pesados.

## Cliente PvP Modern 0.8.0 - 2026-07-21

### Added
- **Mod Menu reorganizado**: tarjetas "Musica", "Insignias y Ping" y "Sprint" agrupan mods relacionados en una sola pantalla en vez de tarjetas sueltas.
- **Paridad 1.8.9 en Rendimiento**: tarjeta "Armadura %", ciclo de "Particulas" (Minimas/Reducidas/Todas), botones "Limpiar memoria" y "Aplicar preset de hardware".
- **Estadisticas de combate por sesion** (nuevo, ninguno de Lunar/Badlion lo tiene): golpes, mejor combo y muertes 100% confiables (cliente-only) + "posibles bajas" heuristico marcado siempre como estimado, con panel HUD opcional y reset automatico al unirse a un mundo/servidor.

### Fixed
- **HUD armadura + contador de bloques**: se eliminó el clamp que forzaba la posición de vuelta al valor viejo en cada carga, impidiendo reposicionar el HUD de forma permanente. Nuevo default junto al hotbar (esquina inferior derecha), con migración automática para quien nunca lo movió a mano.
- **Carátula de música**: si el registro de la textura fallaba con una excepción, el flag de carga quedaba pegado y esa canción se quedaba con el disco genérico para siempre. Ahora cualquier error libera el flag y se reintenta en el siguiente poll (con log para diagnóstico).

### Changed
- "FPS bajo minimizado" ahora se llama "FPS bajo sin foco" (la lógica ya usaba el foco de ventana, no solo minimizado).

## [7.8.0] - 2026-07-20

### Added
- **Join inteligente desde favoritos**: infiere perfil PvP 1.8.9 vs Modern por hint o Server List Ping; botón Unirse sin depender de la instancia seleccionada.
- **Compete mode PvP Modern**: presets agresivos al lanzar Hypixel/favoritos (culling, HUD mínimo, toggles).
- **Asistente Playit primera vez**: checklist con claim link, IP Java/Bedrock copiables y guardar favorito.
- **Consola de servidor mejorada**: botones rápidos (/whitelist, /op, /gamemode, /time, historial).
- **Tienda servidor**: recomendados Paraguacraft (ViaVersion, Geyser, SkinsRestorer) en Paper.
- **Plugin badges Paper 1.21+**: auto-instalación según versión del servidor.
- **Auto-update CurseForge**: fingerprint murmur2 + badge "Instalada" en tienda.
- **Bedrock ↔ Java**: favoritos con puerto Geyser, botón Bedrock y copiar instrucciones para amigos.
- **Paraguabot mejorado**: contexto de instancia/favoritos/servidor + acciones clicables (Reparar, Sincronizar PvP, Abrir consola).

### Changed
- Cache de carátula musical en disco (`file://`) para cliente Modern más robusto.
- Skins offline unificadas launcher → `paraguacraftpvp-modern.properties`.

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
- **Freelook**: camara en 3ª persona con mixin en `Camera.update`.
- **Toggle sprint** desactivado por defecto (sin simular tecla virtual).
- **Caratula musica**: conversion ARGB correcta y panel compacto con hardware arriba.
- **Paraguabot Groq**: modelos `openai/gpt-oss-*` como fallback.
- Servidores **Regorland** (`regorland.net`) y **Hylex** (`original.hylex.net`).
- Launcher ya no resetea `options.txt` ni toggles HUD en cada inicio.

## [7.7.5] - 2026-07-19

### Added
- **Discord RPC Bedrock**: `{usuario} - Bedrock Edition` con estado dinámico (menú, mundo) leyendo el título de ventana antes del rename.
- **Discord RPC Ajustes**: `Explorando Ajustes · {usuario}` al entrar a la pantalla de configuración.

### Changed
- RPC del launcher respeta sesión activa (Java o Bedrock) y no se pisa al navegar mientras jugás.

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
- Brújula detallada, ping con fallback, coordenadas con posicion propia en editor HUD.

## Cliente PvP Modern 0.6.5 - 2026-07-19

### Added
- **Discord RPC in-game** (usuario - versión - loader + servidor/mundo/menú).

## Cliente PvP Modern 0.6.4 - 2026-07-19

### Added
- **Pantalla sin bordes** (borderless LWJGL3) en Mod Menu; alternativa a fullscreen exclusivo.
- **Mod Menu** con tarjetas estilo 1.8.9 (negro + borde azul, ON/OFF/ABRIR).

### Changed
- Submenús (Multijugador, Hypixel Quick Play, Mod Menu, Tema, Packs, Skin) usan botones/tarjetas Paraguacraft.
- Launcher: ya no pisa `options.txt` ni configs en cada launch (solo merge de claves nuevas).

### Fixed
- **ESC / volver al menú**: redirige siempre al menú Paraguacraft, no al TitleScreen vanilla.
- **Logo duplicado** en submenús (fondo sin branding en pantallas hijas).
- **Opciones que se reseteaban**: volumen, chunks, pantalla completa, Sodium — el launcher y `PerformanceBootstrap` ya no las sobrescriben al actualizar.

## Cliente PvP Modern 0.6.3 - 2026-07-19

### Added
- **Contador de bloques** en HUD (BedWars, SkyWars, Lucky Islands, Pillars).
- **MusicArtCache**: carátula de Spotify/YouTube en el HUD de música.

### Changed
- **Recursos BedWars**: columna vertical con fondo transparente (estilo 1.8.9).
- **HUD pociones y objeto en mano**: nombre, duración y encantamientos como 1.8.9.
- **Armadura**: columna vertical solo iconos.
- **Menú**: botones negros con borde azul al hover; fondo Paraguacraft en subpantallas vanilla.
- **Toggle sprint**: restaurado modo virtual + legacy (W) como 1.8.9.
- **CPS**: calculado en tick, no en render (menos micro-lag).

### Fixed
- **Boost FPS** ya no aparece fijo en el HUD (libera espacio para FPS).
- **Logo duplicado** y pantalla negra en menú/multijugador/conexión.
- **IPC música**: launcher lee config modern (`paraguacraftpvp-modern.properties`).

## [7.7.3] - 2026-07-18

### Fixed
- **Mods Fabric 1.21.11**: instala dependencias de Controlling/Zoomify (`searchables`, `fabric-language-kotlin`, `yacl`) y resuelve `required` de Modrinth en cadena.
- **Tienda Modrinth (503)**: hasta 6 reintentos; si la API falla, descarga directo desde CDN con la URL ya cargada en el asistente.
- **Cliente PvP Modern 0.6.1**: sincroniza el JAR embebido y remoto; corrige crash al iniciar por mixin de hitbox en 1.21.11.
- **Dynamic FPS**: repara `dynamic_fps.json` con estado obsoleto `minimized` (ahora `invisible` en 3.11.6).

## [7.7.2] - 2026-07-18

### Fixed
- **Inicio y Ajustes lentos (5–7 s)**: bootstrap único en paralelo, sin cargas duplicadas de cuentas/skins/instancias; Ajustes ya no bloquea la UI esperando Mojang.
- **503 Mojang al jugar**: si `api.minecraftservices.com` está caído, reutiliza el token guardado en lugar de bloquear el inicio.
- **Mods Fabric 1.21.11**: el launcher instala dependencias de Controlling/Zoomify (`searchables`, `fabric-language-kotlin`, `yacl`) vía Modrinth.
- **Tienda Modrinth (503)**: reintentos más largos y descarga directa desde CDN si la API falla al instalar.

### Changed
- Cache de hardware (5 min) y avatar activo (2 min); avatar local instantáneo antes del enrich de Mojang.
- Escaneo de launchers externos (Prism/Lunar) diferido 3 s para no competir con el arranque.

## [7.7.1] - 2026-06-30

### Added
- **Ajustes → Cliente PvP 1.21.11**: verificar y sincronizar el mod modern sin reinstalar el launcher (igual que 1.8.9).

### Fixed
- **Sodium corrupto** en instancias 1.21.11: el launcher ya no parchea `sodium-options.json` con esquema inválido; borra configs rotas al lanzar.

## [7.7.0] - 2026-06-30

### Added
- **Perfil unificado PvP 1.8.9** con selector de destino: Hypixel, favorito, solo menú o **Práctica PvP** (mundo flat).
- **Paraguacraft PvP 1.21.11**: instancia Fabric+Iris auto-creada, mods HUD por tier de PC (baja/media/alta).
- **Fase 2 cliente 1.21.11**: mod `ParaguacraftPvP-Modern` (FPS, ping, keystrokes) vía manifest remoto.
- Presets de servidores en 1.21.11: Hypixel, CubeCraft, MineLatino y favoritos.

### Changed
- Perfil entrenamiento 1.8.9: Boost FPS sin turbo Competir; sprint toggle **M** por defecto en Competir.
- **Loaders 1.21.11 separados**: `fabric-iris` (solo optimización) vs `paraguacraft-pvp-modern` (cliente PvP dedicado).

### Fase 3 — Cliente PvP 1.21.11
- Loader propio **`paraguacraft-pvp-modern`** en el selector de versiones (como `paraguacraft-pvp` en 1.8.9).
- Perfil Inicio **Fabric + Iris** aparte del perfil **Paraguacraft PvP 1.21.11**.
- Mod modern lee `paraguacraft_modern.properties` del launcher (tier + HUD).

### Fase 4 — Optimizaciones 1.21.11 (superiores a 1.8.9)
- **JVM Java 21** dedicada: hasta **8 GB** heap (vs tope 4 GB en 1.8.9), **ZGC generacional** en PCs 16+ GB.
- **options.txt PvP** por tier: distancias, partículas mínimas, nubes off, entity distance scaling.
- Patch automático de **Lithium**, **Sodium** y **Dynamic FPS** al lanzar.
- Mod **0.2.0**: Boost FPS (preset vanilla + limpieza de memoria al cambiar mundo) e indicador en HUD.

### Pre-fase 5 — Menú PvP 1.21.11
- **Menú personalizado** estilo Paraguacraft/Lunar: constelación, logo, botones centrales.
- Barra inferior: **Skin**, **Tema** (5 presets), **Packs**, **Fabric** (Mod Menu).
- **Multijugador PvP** con Hypixel, CubeCraft, LibreCraft, Hylex y MineLatino.
- **Texture packs 1.21.11** descargados al instalar (launcher + catálogo GitHub `pvp-packs-modern-1.0`).
- Mod **0.3.0** incluye selector de packs in-game.

### Fase 5 — Paridad PvP 1.21.11
- **Mod Menu** (Right Shift): toggles HUD, Boost FPS, toggle sprint, acceso a Quick Play y packs.
- **Hypixel Quick Play** con reconexión al último modo.
- **Práctica PvP**: botón en menú + destino launcher con mundo local y HUD de entrenamiento.
- HUD ampliado: **coordenadas**, **armadura** y **CPS**.
- Mod **0.4.0** compilado y empaquetado en `bundled/pvp-modern/`.

## Cliente PvP Modern 0.6.2 - 2026-07-18

### Fixed
- **Pantalla negra en menu**: corrige firma del mixin `MixinItemEntityRenderer` para el pipeline de render 1.21.11.

## Cliente PvP Modern 0.6.1 - 2026-07-18

### Fixed
- **Crash al iniciar (MixinWorldRendererOutline)**: el mixin de grosor del hitbox apuntaba al ordinal incorrecto del parámetro `lineWidth` en `drawBlockOutline` de 1.21.11.

## Cliente PvP Modern 0.6.0 - 2026-07-18

### Added
- Mods de prioridad alta portados: No Hurt Cam, Fullbright, FOV dinámico, Hide titles, Scoreboard toggle, Low fire, Item physics, Old swing, Combo, TNT HUD, Chat triggers, Freelook (Alt), Pociones HUD, Brújula.
- Mod Menu ampliado con toggles para todos los módulos nuevos.
- Atajos: G fullbright, Alt freelook, RControl editar HUD, ` quick play.
- Launcher instala mods Fabric extra: Controlling, Smooth Scrolling, Zoomify (+ Mod Menu, AppleSkin, Better Ping, Shulker Tooltip, Dynamic FPS).

### Notes
- Fondos de tema PNG embebidos si están en `assets/paraguacraftpvp-modern/textures/gui/`.
- Filtro avanzado del scoreboard (ocultar stats/números rojos) pendiente de paridad total 1.8.9.

## Cliente PvP Modern 0.5.0 - 2026-07-18

### Added
- **Mod Menu** estilo Lunar con categorías, cards ON/OFF y botón **Editar HUD** (arrastrar módulos).
- **Skin Changer** por URL o nick (minotar) en lugar del menú vanilla de capas.
- HUD ampliado: iconos de armadura, objeto en mano, recursos BedWars, overlay música (IPC launcher).
- **Hitbox azul** cyan al apuntar bloques; **FPS/Ping/CPS** con etiquetas estilo 1.8.9.
- Keystrokes 20×20 (WASD + LMB/RMB) como 1.8.9.

### Fixed
- **Toggle sprint legacy**: W activa sprint automáticamente.
- **Resource packs**: aplicación real vía perfiles habilitados + refresh.
- **Hypixel Quick Play**: conecta a Hypixel y ejecuta el comando al entrar.
- **Práctica PvP flat**: borra mundo corrupto sin `level.dat` y crea flat automático.
- **Menú alternante** vanilla/custom: `CustomTitleScreen` extiende `TitleScreen`.
- **Layout responsive** del menú principal según escala GUI.
- **Tier hardware**: PCs con 16–32 GB RAM ya no reciben preset “baja” (R7 5700G + 32 GB → alta).

### Changed
- Preset gráfico **media** menos agresivo (render 12, sim 10, ImmediatelyFast activo).

## Cliente PvP Modern 0.4.2 - 2026-06-30

### Fixed
- **Texto invisible en el menú**: colores del tema sin canal alpha (1.21.11 ignora drawText si alpha = 0).
- **Logo rosa/negro**: textura del mod dibujada con `drawTexture` en lugar de `drawGuiTexture` (solo sprites del atlas).
- HUD: colores de FPS, ping, CPS, coords y armadura con alpha completo.

## Cliente PvP Modern 0.4.1 - 2026-06-30

### Fixed
- **Logo Paraguacraft** cargaba mal (namespace de assets incorrecto).
- **Menú compacto**: botones planos estilo Lunar, sin bloques de piedra gigantes ni solapamiento.
- Layout en 3 filas: juego principal, utilidades (Mod Menu/Opciones/Salir) y barra Skin/Tema/Packs/Fabric.

## Cliente PvP Modern 0.4.0 - 2026-06-30

### Added
- Mod Menu estilo Lunar (Right Shift).
- Hypixel Quick Play con estado persistente.
- Mundo de práctica PvP (flat) desde menú o launcher.
- HUD: coordenadas, armadura, CPS y toggle sprint configurable.

## Cliente PvP Modern 0.3.0 - 2026-06-30

### Added
- Menú principal y pausa personalizados (mixin TitleScreen / GameMenuScreen).
- Pantallas: tema, multijugador PvP, texture packs.
- Catálogo embebido + remoto de 5 packs PvP 1.21.11.

## Cliente PvP Modern 0.2.0 - 2026-06-30

### Added
- **Boost FPS**: aplica gráficos rápidos, partículas mínimas y distancias PvP al iniciar.
- Limpieza de memoria al entrar a mundo/servidor.
- Badge **Boost** en el HUD (media/alta).

### Changed
- Launcher sincroniza flags de rendimiento en `paraguacraft_modern.properties`.

## Cliente PvP Modern 0.1.0 - 2026-06-30

### Added
- Mod Fabric **Paraguacraft PvP Modern** para 1.21.11: HUD FPS, ping y keystrokes básicos.

## Cliente PvP 2.1.29 - 2026-06-30

### Added
- **Hypixel Quick Play**: botón reconectar al último modo jugado.
- **Práctica PvP**: mundo flat con reglas PvP (keepInventory, sin mobs/regeneración).
- Destino launcher «Práctica PvP» abre el mundo automáticamente.

### Changed
- Filtro de scoreboard Hypixel ampliado (quests, rank, daily reward, etc.).
- Perfil Competir fuerza scoreboard limpio y toggle sprint modo **M** (legacy **N** off).

## [7.6.0] - 2026-06-30

### Added
- **Contenido estilo Modrinth**: mods, shaders y resource packs muestran nombre, autor, descripción e icono original (API Modrinth + `pack.png` local).
- **Compatibilidad de mods** en chequeo pre-lanzamiento según loader y versión MC.

### Fixed
- **Resource packs** ya no se borran al aplicar skin offline ni al togglear desde el launcher (`options.txt` merge correcto).
- **Resource packs en PvP 1.8.9 y 1.21.x**: activación/desactivación sincronizada con `options.txt`.
- **Servidores Fabric**: preparación detecta `fabric-server-launch.jar`; playit.gg se auto-inicia al arrancar el servidor.
- **Playit.gg**: no devuelve dirección obsoleta como si fuera la actual.

## [7.5.0] - 2026-06-30

### Added
- **Diagnóstico post-crash liviano**: banner con causa, hints y enlace a instancia/Paraguabot (sin abrir el bot automáticamente).
- **Conflictos de mods**: avisos on-demand (duplicados, OptiFine+Iris, Essential/Patcher).
- **Bandeja ultra-lite**: icono en systray mientras jugás; restaurar launcher con un clic.

## [7.4.0] - 2026-06-30

### Added
- **Chequeo pre-lanzamiento** on-demand: Java, cliente PvP, espacio en disco y tips de antivirus.
- **Peso de instancia** (liviano / medio / pesado) según mods, shaders y RAM.
- **Perfiles de juego** 1 clic en Inicio: Hypixel PvP, PvP práctica, Vanilla e Iris/Modpack.

## [7.3.0] - 2026-06-30

### Added
- **Modo Competir** (PvP): un clic orquesta cierre del launcher al jugar, Game Mode,
  prioridad Java alta, RAM/GC por hardware, perfil cliente Boost FPS y actualizaciones
  PvP diferidas hasta cerrar Minecraft.
- **Presupuesto de recursos** en detalle de instancia: launcher, Java y RAM libre.
- **Música smart**: defaults por gama baja/media; IPC overlay solo si HUD música/hardware ON.
- **Actualizaciones diferidas**: no sync PvP ni chequeo de launcher mientras hay partida.

## [7.2.19] - 2026-07-01

### Fixed
- **Paraguabot**: carga `.env` desde `launcher/.env` aunque el cwd sea `src-tauri`; panel en
  Ajustes para guardar Groq API key; conocimiento embebido del launcher y cliente PvP 2.1.28.
- **Paraguabot**: consultas generales ya no quedan bloqueadas por un diagnostico de crash previo.

### Changed
- **Web**: modo claro en [paraguacraft.pages.dev](https://paraguacraft.pages.dev) — deploy desde repo [paraguacraft-web](https://github.com/SantiJ10/paraguacraft-web) (separado del launcher).

## Cliente PvP 2.1.28 - 2026-06-30

### Added
- **Dos modos de Correr toggle** para testeo A/B: nuevo (teclas virtuales, **M**) y legacy
  (`setSprinting`, **N**), cada uno con tarjeta en Mod Menu y atajo configurable.

## Cliente PvP 2.1.27 - 2026-06-30

### Fixed
- **Toggle Sprint / Toggle Sneak**: sin lag de 1 tick; las teclas virtuales se aplican al
  inicio de `onLivingUpdate` (patrón Lunar) para velocidad de sprint vanilla y sneak instantáneo.

## Cliente PvP 2.1.26 - 2026-06-30

### Fixed
- **Toggle Sprint / Toggle Sneak rotos** en 2.1.24–2.1.25: el mixin no se aplicaba (punto de
  inyección inválido). Ahora sneak/sprint se aplican al final de `onLivingUpdate`.
- **Detección de actualización**: el launcher consulta todos los mirrors del manifest y elige
  la versión más nueva (evita CDN de raw.githubusercontent con manifest viejo).

## Cliente PvP 2.1.25 - 2026-06-30

### Added
- **Toggle Sprint en Mod Menu** (categoría PvP): activar/desactivar sin depender de la tecla V.

## Cliente PvP 2.1.24 - 2026-06-30

### Fixed
- **Toggle Sneak**: respuesta instantánea (como Lunar); sneak/sprint se aplican después de
  `updatePlayerMoveState` vía mixin, no antes donde vanilla los pisaba cada tick.
- **Auditoría de input**: eliminados usos rotos de `setKeyBindState` para sprint; sneak solo
  bloquea Shift físico de forma intencional.

## Cliente PvP 2.1.23 - 2026-06-30

### Fixed
- **Toggle Sprint**: ya no simula la tecla Ctrl (se pisaba cada frame); ahora activa
  el sprint al moverse hacia adelante con W cuando el toggle está ON (tecla V).

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
- **Ajustes → Cliente PvP**: versión publicada vs instalada, sincronización manual y nota
  de que el cliente se actualiza sin recompilar el launcher.
- **Modo offline mejorado**: jugar con instancias y mods ya instalados sin internet;
  skins Premium se aplican localmente y se encolan para Mojang al reconectar; tokens
  Microsoft en caché si no hay red.

### Changed
- Launcher **7.2.17**.

## [7.2.16] - 2026-06-30

### Fixed
- **Crash al entrar a Hypixel (cliente 2.1.19)**: los sprites de cama coloreados ya no
  apuntan a PNG inexistentes en `paraguacraft:textures/beds/`; cargan desde la textura
  vanilla y se recolorean después del stitch del atlas.
- **HUD de hardware — GPU como CPU/RAM**: el overlay muestra **% de uso** de la GPU (y
  temperatura cuando está disponible), no el nombre de la placa. En Windows se lee con
  contadores de rendimiento / `nvidia-smi`.
- **Skins Steve corruptas en lobby de Hypixel**: se restaura el estado OpenGL tras dibujar
  logos/ping en nametags (la textura quedaba mal enlazada y rompía el siguiente jugador
  renderizado; Alex no se veía afectado).
- **Camas siempre rojas en BedWars**: detección de equipo en Hypixel (sidebar con ✓, lana en
  inventario, bloques cercanos) y sprites de cama recoloreados por equipo, estilo
  Lunar/Badlion.

### Changed
- **Cliente PvP 2.1.19** (hotfix de camas sobre 2.1.18).

## [7.2.15] - 2026-06-30

### Fixed
- **Pantalla renderizada "en la esquina"** (cuadrante con el resto en negro): self-heal de
  viewport: si el tamaño real de la ventana no coincide con el framebuffer, se fuerza el
  resize. Corrige el problema al usar Windowed Fullscreen y también con escalado DPI de
  Windows (típico en laptops con pantalla a 125/150%).
- **HUD de hardware con datos correctos**:
  - **CPU/RAM ya no marcan "demasiada carga"**: el % de CPU se medía con un `System` nuevo
    en cada lectura (sysinfo necesita dos muestras para el delta). Ahora se reutiliza una
    instancia persistente, así el % es real.
  - **GPU identificada de verdad**: se muestra el nombre real de la placa leído por OpenGL
    (`GL_RENDERER`), que funciona en cualquier PC (antes la GPU iba fija en "-").

### Changed
- **Cliente PvP 2.1.17**.

## [7.2.14] - 2026-06-30

### Added
- **Limitador de FPS en segundo plano** (estilo Lunar/Badlion): cuando la ventana está
  **minimizada** se baja el tope de FPS (default 5). Reduce uso de CPU/GPU y, sobre todo
  en laptops, evita el *thermal throttling* que después tira los FPS en partida. Opción
  extra para limitar también **sin foco** (default off, para no molestar al borderless con
  el juego visible en otro monitor). Configurable desde el Mod Menu (Rendimiento).

### Fixed
- **Windowed Fullscreen**: al activarlo desde el menú ya no se ve la ventana "achicada"
  hasta tocar F11; ahora aplica el resize del framebuffer al instante.

### Changed
- **Cliente PvP 2.1.16**.

## [7.2.13] - 2026-06-30

### Added
- **Windowed Fullscreen (pantalla completa en ventana)** estilo Patcher/Lunar en el
  cliente PvP. Reemplaza el fullscreen exclusivo (F11) por una ventana sin bordes del
  tamaño del escritorio: permite **alt-tab instantáneo**, enfocar otras ventanas y que
  **OBS/Discord** sigan capturando como ventana de juego, sin el parpadeo del modo
  exclusivo. Implementado con LWJGL2 (propiedad `undecorated` + `setFullscreen(false)`),
  no usa hacks nativos frágiles. Se activa desde el Mod Menu (categoría Mecánicas) y se
  aplica al instante; también cambia el comportamiento de F11.

### Changed
- **Limpieza automática de restos de Essential/Patcher**: el launcher borra el JAR de
  Essential y sus carpetas de datos (`essential/`, `ModCoreOSS/`, config de Patcher) de
  las instancias existentes, evitando el login y los reinicios que dejaba.
- **Cliente PvP 2.1.15**.

### Performance
- Optimizaciones propias revisadas y mantenidas: culling de entidades **sin parpadeo de
  jugadores** (nunca se cullan), nametags/armorstands/itemframes/tile-entities, límite de
  partículas, skip de FX de combate y limpieza de memoria. Preset automático por gama de
  hardware (LOW/MEDIUM/HIGH) para buen rendimiento desde laptops 8 GB hasta PCs de gama alta.

## [7.2.12] - 2026-06-30

### Removed
- **Patcher (Sk1er) / Essential** eliminado del cliente PvP. El Patcher actual depende de
  Essential (login, cosméticos y un reinicio del juego al arrancar) y no existe una versión
  standalone viable. Para un cliente PvP limpio se quita por completo; el launcher purga el
  JAR de Patcher de las instancias existentes automáticamente.

### Changed
- El cliente queda con **OptiFine + optimizaciones propias pulidas**: culling de
  entidades (jugadores nunca se cullan), nametags/armorstands/itemframes/tile-entities,
  límite de partículas, skip de FX de combate y limpieza de memoria al cambiar de mundo
  (GC solo al descargar mundo, sin freeze al entrar a la partida).
- **Cliente PvP 2.1.14**.

## [7.2.11] - 2026-06-30

### Fixed
- **Crash al iniciar** (no llegaba al menú principal): el preset Boost FPS ponía
  `mipmapLevels = 0`, lo que provoca `ArrayIndexOutOfBoundsException` al generar
  mipmaps del atlas de texturas en 1.8.9. Ahora se usa mínimo 1 y se corrige
  automáticamente si el perfil quedó en 0.

### Changed
- **Cliente PvP 2.1.13**.

## [7.2.10] - 2026-06-30

### Performance
- **Boost FPS reforzado** (preset estilo Lunar/Badlion):
  - OptiFine **Fast Render** + Render Regions + Smart Animations + Fast Math.
  - Apagado de nubes, lluvia, clima, estrellas, sky custom, partículas de agua/void/portal.
  - Gráficos en **Fast**, Smooth Lighting (AO) **off**, mipmaps **off**, AA/AF al mínimo.
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
- **Compañeros que desaparecian y volvian en partida**: el culling de entidades hacia un chequeo de frustum sobre jugadores lejanos y parpadeaban. Ahora los jugadores NUNCA se cullan (el culling solo aplica a mobs/objetos).
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
- Todos los mods del cliente son **solo cosmeticos/HUD** — sin reach hack, xray, autoclicker ni macros.
- Reach Display y Combo Counter solo **muestran** datos de tus golpes; no alteran hitboxes ni paquetes.

## [7.1.8] - 2026-06-29

### Added
- **Traducciones vanilla es_ES y es_AR** embebidas en el cliente PvP.

## [7.1.2] - 2026-06-25

### Fixed
- **JVM PvP 1.8.9**: flags G1 solo compatibles con Java 8 (evita «Could not create the Java Virtual Machine» en PCs con mucha RAM).
- Sin `-Xmx`/`-Xms` duplicados del perfil Forge cuando aplica preset PvP.
- Verificación SHA del mod PvP: solo acepta el hash del manifest (fuerza actualización del JAR viejo).

## [7.1.1] - 2026-06-19

### Added
- **Cliente PvP 2.0.0** recompilado: HUD/GUI Lunar, perfiles, keybinds, resource packs, badge sync, optimizaciones Fase C/D.

### Fixed
- **Auto-update PvP**: el SHA del manifest remoto manda sobre el JAR embebido; Play descarga el cliente nuevo aunque el instalador sea viejo.
- JAR embebido actualizado (`04aee52f…`) para instalaciones offline.

## [7.1.0] - 2026-06-25

### Added
- **Cliente PvP dinámico**: versión, release y mods desde `manifest.json` remoto; elimina JARs viejos al actualizar.
- **Perfil JVM PvP 1.8.9**: RAM/G1GC por gama de hardware (Java 8).

### Changed
- README orientado al jugador (intro, features, instalación) como la web.
- Manifest PvP con `client_version` y `release_tag`.

### Fixed
- SHA-1 del manifest remoto vs JAR embebido; prioridad local + fallback.

## [7.0.5] - 2026-06-24

### Fixed
- **Lanzamiento 1.8.9 en Windows**: Java 8 ya no usa `@args.txt` (solo Java 9+); corrige cierre instantáneo sin logs.
- **Java por versión**: el override global en Ajustes no bloquea Java 8 en instancias 1.8.9 si tenés Java 21 para 1.21.
- **Título de ventana**: solo «Paraguacraft PvP» en 1.8.9 PvP; otras versiones muestran «Paraguacraft X.X.X».

### Changed
- **PvP Client 2.0.0**: texture packs desde GitHub Release y Google Drive (sin Modrinth en el gestor del mod).
- Instancias PvP oficiales (`Paraguacraft_1.8.9_PvP`) separadas de carpetas de prueba.

## [7.0.4] - 2026-06-24

### Fixed
- **Paraguabot / crash falso**: ignora el "Crash Report" de la pantalla de carga de Forge 1.8.9 (no es un error real).
- No usa crash-reports viejos de sesiones anteriores al diagnosticar.
- Forge 1.8.9 que cierra con código 1 pero log `Stopping!` ya no marca "El juego terminó con error".

## [7.0.3] - 2026-06-24

### Added
- **Paraguacraft PvP Client 2.0.0**: descarga automática desde release `pvp-client-2.0.0` (Forge 1.8.9 + OptiFine + mod 2.0.0).
- **Texture packs PvP**: catálogo remoto + release `pvp-packs-1.0` en GitHub (9 packs); Faithful y Tightfault vía Modrinth.
- **Fallback local** de mods PvP: `%APPDATA%/ParaguacraftLauncher/bundled/pvp` si GitHub no responde.
- Scripts `publish-pvp-client.ps1` y `publish-pvp-packs.ps1` (GitHub CLI, detecta `gh` sin PATH).

### Changed
- Manifest y catálogo PvP actualizados a 2.0.0 (`clientes/paraguacraft-pvp/`).
- Reparación de instancias: corrige meta PvP inferida incorrectamente desde nombre de carpeta.

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
- CI release: genera `latest.json` con hash real y lo sube al release automáticamente.

## [7.0.0] - 2026-06-19

### Fixed
- PvP 1.8.9: OptiFine y cliente desde `bundled/pvp` en GitHub (sin depender de optifine.net).
- Lanzamiento Java 8: flags JVM compatibles; resolución de Java para instaladores Forge/OptiFine.
- Modpacks Modrinth: reintentos en red, User-Agent correcto, loader instalado antes de bajar mods.
- CurseForge: API key embebida en builds de release (CI + compile-time).

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
