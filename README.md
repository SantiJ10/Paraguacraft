<div align="center">

# 🟩 Paraguacraft Launcher

**Un launcher personalizado de Minecraft, hecho en Python con interfaz web moderna.**

[![Version](https://img.shields.io/badge/versión-3.0.0-2ECC71?style=flat-square)](https://github.com/SantiJ10/Paraguacraft/releases)
[![Python](https://img.shields.io/badge/Python-3.10%2B-3776AB?style=flat-square&logo=python&logoColor=white)](https://python.org)
[![License](https://img.shields.io/badge/licencia-MIT-blue?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/plataforma-Windows-0078D6?style=flat-square&logo=windows&logoColor=white)](https://github.com/SantiJ10/Paraguacraft/releases)

</div>

---

## ¿Qué es Paraguacraft?

Paraguacraft es un launcher de Minecraft completamente personalizado, desarrollado en Python con una interfaz web moderna construida sobre HTML, TailwindCSS y JavaScript. Permite gestionar versiones, mods, skins, servidores, mundos y mucho más desde un único lugar — sin depender del launcher oficial.

---

## ✨ Características principales

### 🎮 Juego
- Soporte para **Vanilla, Fabric, Forge, Quilt, NeoForge y Fabric + Iris**
- Múltiples instancias independientes por versión y loader
- Lanzamiento directo con Quick Play (conectar a servidor al iniciar)
- Discord Rich Presence (RPC) en tiempo real
- Soporte para cuentas **Premium (Mojang/Microsoft)** y modo **Offline**

### 🛒 Tienda de Mods
- Búsqueda e instalación desde **Modrinth** y **CurseForge**
- Soporte para mods, shaders, resource packs, datapacks y modpacks
- Visualización de versiones compatibles por loader y versión de MC
- Detección y aviso de mods conflictivos en una instancia
- Verificación de actualizaciones disponibles para mods instalados

### 🖼️ Skins
- Visor 3D de skins con animaciones (caminar, rotar, modelo Alex/Steve)
- Catálogo de skins de jugadores populares
- Importación de skins desde archivo local (`.png`)
- Subida directa a Mojang para cuentas Premium

### 🔧 Herramientas y rendimiento
- **Java Optimizer**: análisis de hardware y configuración automática de RAM y GC flags
- **Game Mode**: suspensión de procesos innecesarios para maximizar FPS
- **Turbo Mode**: optimización agresiva de recursos del sistema
- **Smart RAM**: ajuste dinámico de memoria asignada a Minecraft
- **Benchmark integrado**: métricas de rendimiento en tiempo real
- **Overlay en juego**: visualización de estadísticas mientras jugás

### 🤖 Inteligencia Artificial (Gemini)
- Chat de asistencia sobre Minecraft
- Generador de comandos por descripción en texto
- Generador de modpacks con IA (sugiere e instala mods automáticamente)
- Generador de semillas
- **Analizador de crash logs con IA**: detecta errores y sugiere soluciones

### 🎵 Música
- **Spotify integrado**: controles de reproducción, álbum, progreso
- **Radio YouTube**: búsqueda y reproducción de playlists y videos
- Música de MC con control de volumen
- Barra de música global en el launcher

### 🌐 Servidor local
- Descarga y configuración automática de Paper/Spigot
- Túnel público con **PlayIt** (sin necesidad de abrir puertos)
- Consola en tiempo real con colores por nivel de log
- Sistema de whitelist

### 💾 Gestión de instancias
- Backup y restauración de mundos
- Compartir instancias por código (exporta lista de mods, versión y config)
- Importar instancias de otros jugadores con un código
- Gestión de mods instalados (activar/desactivar sin borrar)
- Visualizador de screenshots

### 🎨 Personalización
- **Fondo personalizado** del launcher: imágenes, GIFs o videos (reemplaza el banner del inicio)
- Modo compacto
- Perfiles de hardware (baja/media/alta gama) con mods preconfigurados

### 🔄 Sistema
- Auto-actualización via GitHub Releases
- Notificaciones de Windows nativas
- Noticias de Minecraft en la pantalla de inicio
- Jugadores online globales en la barra de navegación
- Ping en vivo a servidores populares

---

## 🚀 Instalación

### Opción 1 — Ejecutable (recomendado)

Descargá el último `.exe` desde [Releases](https://github.com/SantiJ10/Paraguacraft/releases) y ejecutalo directamente. No requiere instalar Python.

### Opción 2 — Desde el código fuente

**Requisitos:**
- Python 3.10 o superior
- pip

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

El ejecutable se genera en la carpeta `dist/`.

---

## 📁 Estructura del proyecto

```
Paraguacraft/
├── paragua.py          # Backend principal (API pywebview + servidor HTTP)
├── core.py             # Lógica de lanzamiento y utilidades
├── web/
│   └── index.html      # Interfaz web completa (UI del launcher)
├── web/assets/         # Imágenes, íconos y recursos estáticos
├── src/                # Módulos auxiliares
├── Paraguacraft.spec   # Configuración de PyInstaller
└── requirements.txt    # Dependencias Python
```

---

## 🛠️ Stack tecnológico

| Capa | Tecnología |
|------|-----------|
| Backend | Python 3, pywebview, requests, psutil |
| Frontend | HTML5, TailwindCSS, JavaScript |
| Empaquetado | PyInstaller |
| APIs externas | Modrinth, CurseForge, Mojang, Gemini AI, Spotify, YouTube |
| Minecraft | minecraft-launcher-lib |

---

## 📝 Changelog reciente

### v3.0.0
- Soporte completo para **Fabric + Iris** en la tienda de mods
- Corrección del spinner infinito en instalación de mods (Modrinth y CurseForge)
- Fondo personalizado del launcher ahora reemplaza correctamente el banner de inicio
- Instalación de mods CurseForge completamente funcional
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

Este proyecto está bajo la licencia MIT. Ver el archivo [LICENSE](LICENSE) para más detalles.

---

<div align="center">
Hecho con ❤️ para la comunidad de Minecraft hispanohablante
</div>