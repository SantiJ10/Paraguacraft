<div align="center">

# 🟩 Paraguacraft Launcher

**Launcher completo de Minecraft, desarrollado en Python con interfaz web moderna.**

[![Version](https://img.shields.io/badge/versión-5.0.0-2ECC71?style=flat-square)](https://github.com/SantiJ10/Paraguacraft/releases)
[![Python](https://img.shields.io/badge/Python-3.10%2B-3776AB?style=flat-square&logo=python&logoColor=white)](https://python.org)
[![License](https://img.shields.io/badge/licencia-MIT-blue?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/plataforma-Windows-0078D6?style=flat-square&logo=windows&logoColor=white)](https://github.com/SantiJ10/Paraguacraft/releases)

[**Descargar**](https://github.com/SantiJ10/Paraguacraft/releases/latest) · [**Reportar bug**](https://github.com/SantiJ10/Paraguacraft/issues) · [**Changelog**](#-changelog)

</div>

---

## ¿Qué es Paraguacraft?

Paraguacraft es un launcher de Minecraft completamente personalizado, desarrollado en Python con una interfaz web construida sobre HTML, TailwindCSS y JavaScript. Permite gestionar versiones, mods, skins, servidores y mucho más desde un único lugar, sin depender del launcher oficial de Mojang.

---

## ✨ Características

### 🎮 Juego
- Soporte para **Vanilla, Fabric, Forge, Quilt, NeoForge y Fabric + Iris**
- Múltiples instancias independientes por versión y loader
- Quick Play: conectar directamente a un servidor al iniciar
- Discord Rich Presence (RPC) en tiempo real
- Cuentas **Premium (Microsoft/Mojang)** y modo **Offline**
- Ventana **maximizada al iniciar** automáticamente
- Restauración del estado de ventana correcto al cerrar Minecraft

### 🛒 Tienda de Mods
- Búsqueda e instalación desde **Modrinth** y **CurseForge**
- Mods, shaders, resource packs, datapacks y modpacks
- Filtrado por loader y versión de Minecraft
- Detección de mods conflictivos en la instancia activa
- **Auto-check de actualizaciones** al abrir la sección (vía hash SHA-512 contra Modrinth)
- **Configuración de API Key de CurseForge** integrada en la UI (sin editar archivos)
- Acceso directo desde el botón 🧩 del panel de juego

### 🖼️ Skins
- Visor 3D con animaciones (caminar, rotar, modelos Alex/Steve)
- Catálogo de skins de jugadores populares
- Importación desde archivo local `.png`
- Subida directa a Mojang para cuentas Premium

### 🔧 Rendimiento y herramientas
- **Java Optimizer**: configura RAM y flags JVM según el hardware detectado
- **Game Mode**: suspende procesos del sistema para maximizar FPS
- **Turbo Mode**: optimización agresiva de recursos
- **Smart RAM**: ajuste dinámico de memoria asignada a Minecraft
- **Benchmark integrado** con métricas en tiempo real
- **Overlay en juego**: estadísticas de rendimiento sin salir de MC
- **Centro de Diagnóstico**: 45+ patrones de error, lectura de 5 archivos de log, auto-fixes con un clic
- **Auto-diagnóstico**: al crashear Minecraft, el launcher navega y escanea automáticamente

### 🤖 Inteligencia Artificial (Gemini)
- Asistente de chat sobre Minecraft
- Generador de comandos por descripción en lenguaje natural
- Generador de modpacks inteligente (sugiere e instala mods automáticamente)

### 🎵 Música
- **Spotify integrado**: controles, portada, progreso de reproducción
- **Radio YouTube**: búsqueda y reproducción de playlists y videos
- Música ambiental de Minecraft con control de volumen
- Barra de música persistente en toda la interfaz

### 🌐 Servidor local
- Creación y configuración automática de servidores **Paper**, **Spigot**, **Fabric**
- Soporte para **Geyser** (jugadores Bedrock en servidores Java)
- Túnel público con **playit.gg** — sin abrir puertos ni configurar router
  - Detección automática de la dirección Java desde el log del agente
  - Soporte de dirección Bedrock personalizada con **persistencia entre reinicios**
  - La dirección se guarda por servidor en `_paragua_srv.json`
- Consola en tiempo real con colores por nivel de log
- Reinicio automático programado por horas
- Sistema de **whitelist** y **bans** integrado
- Edición de `server.properties` desde la UI

### 💾 Instancias y mundos
- Backup y restauración de mundos
- Compartir instancias por código (mods, versión y config)
- Importar instancias de otros jugadores
- Activar/desactivar mods sin borrarlos
- Visualizador de screenshots

### 🎨 Personalización
- Fondo del launcher configurable: imágenes, GIFs o videos
- Modo compacto
- Perfiles de hardware (baja/media/alta gama) con mods preconfigurados

### 🔄 Sistema
- **Auto-actualización** vía GitHub Releases
- Notificaciones nativas de Windows
- Noticias de Minecraft en la pantalla de inicio
- Contador de jugadores online en tiempo real
- Ping en vivo a servidores populares

---

## 🚀 Instalación

### Opción 1 — Instalador (recomendado)

Descargá el instalador `.exe` desde [Releases](https://github.com/SantiJ10/Paraguacraft/releases/latest) y ejecutalo. Instala para todos los usuarios del equipo. No requiere Python.

### Opción 2 — Desde el código fuente

**Requisitos:** Python 3.10+, pip

```bash
git clone https://github.com/SantiJ10/Paraguacraft.git
cd Paraguacraft
pip install -r requirements.txt
python paragua.py
```

---

## 🏗️ Compilar el ejecutable

```bash
pip install pyinstaller
pyinstaller Paraguacraft.spec --noconfirm
```

El ejecutable queda en `dist/Paraguacraft.exe`.

---

## 📁 Estructura del proyecto

```
Paraguacraft/
├── paragua.py            # Backend principal (API pywebview)
├── core.py               # Lógica de lanzamiento de Minecraft
├── web/
│   └── index.html        # Interfaz web completa
├── web/assets/           # Imágenes, íconos y recursos estáticos
├── src/                  # Módulos auxiliares
├── Paraguacraft.spec     # Configuración de PyInstaller
└── requirements.txt      # Dependencias Python
```

---

## 🛠️ Stack tecnológico

| Capa | Tecnología |
|------|-----------|
| Backend | Python 3, pywebview, requests, psutil |
| Frontend | HTML5, TailwindCSS, JavaScript |
| Empaquetado | PyInstaller + Inno Setup |
| APIs externas | Modrinth, CurseForge, Mojang, Gemini AI, Spotify, YouTube |
| Minecraft | minecraft-launcher-lib |
| Túnel público | playit.gg |

---

## 📝 Changelog

### v5.0.0
- **Auto-maximize**: la ventana del launcher inicia maximizada via parámetro nativo de pywebview
- **Restaurar estado de ventana**: al cerrar Minecraft, la ventana vuelve al estado previo (maximizada si lo estaba)
- **Auto-diagnóstico de crashes**: al detectar un crash, navega automáticamente a Extras → Crash Log y escanea
- **CF API Key UI**: configuración de API Key de CurseForge desde dentro del launcher con guía paso a paso
- **Auto-check de actualizaciones de mods**: escanea los JARs por SHA-512 contra Modrinth al abrir la tienda
- **Botón 🧩 Mods** en la fila de lanzamiento para acceso rápido a la tienda
- **Botón "Buscar en tienda"** dentro del Gestor de Instancia → Contenido
- Corrección de bugs: `_cargar_config`/`_guardar_config` inexistentes en `aplicar_fix`
- Corrección: `inst_path` indefinido cuando `motor` era vacío en `analizar_crash_log`
- CF 403 devuelve mensaje explícito para detección correcta en el frontend

### v4.8.0
- **Centro de Diagnóstico** expandido: 45+ patrones de error de inicio y en juego
- Lectura de 5 fuentes de log: `latest.log`, `debug.log`, crash reports, `launch_debug.log`, `hs_err_pid*.log`
- Auto-fixes: aumentar RAM, eliminar duplicados/OptiFine/shaders/JARs corruptos, flags JVM
- Botón 📄 Log inicio para leer `launch_debug.log` directamente
- Patrones nuevos: `UnsupportedClassVersionError`, JVM crash, sesión expirada, OpenGL, exit codes

### v4.7.0
- Instalador Inno Setup configurado para instalación **global** (todos los usuarios, `Program Files`)
- Versión bumpeada para nueva distribución

### v4.6.0
- **Servidor → playit.gg**: detección automática de dirección Java desde el log del agente
- **Bedrock**: soporte de dirección personalizada con persistencia entre reinicios y reaperturas de la app
- La dirección Bedrock se carga desde `_paragua_srv.json` al iniciar el túnel, incluso antes que el servidor
- La fila Bedrock en la UI ahora se muestra también cuando hay una dirección guardada (aunque el server no sea Geyser)
- Eliminado el spam de debug `[PLAYIT][API]` de la consola
- Correcciones de indentación y sintaxis en la función `_probe_tuneles`

### v3.0.0
- Soporte completo para **Fabric + Iris** en la tienda de mods
- Corrección del spinner infinito en instalación de mods (Modrinth y CurseForge)
- Fondo personalizado ahora reemplaza correctamente el banner de inicio
- El modal de instalación pre-selecciona automáticamente el loader activo

### v2.9.0
- Sistema de fondo personalizado: imágenes, GIFs y videos
- Streaming de video vía endpoint HTTP local con soporte de Range requests

### v2.8.0
- Tienda dual Modrinth/CurseForge
- Analizador de crash logs con IA (Gemini)
- Compartir instancias por código
- Spotify y YouTube Radio integrados

---

## 📄 Licencia

Este proyecto está bajo la licencia MIT. Ver [LICENSE](LICENSE) para más detalles.

---

<div align="center">
Hecho con ❤️ para la comunidad de Minecraft hispanohablante
</div>