<div align="center">

# Paraguacraft Launcher

Launcher de Minecraft — **6.9.0+** en Tauri + Rust + Vue 3.

[**Descargar**](https://github.com/SantiJ10/Paraguacraft/releases/latest) · [**Web**](https://paraguacraft.pages.dev) · [**Issues**](https://github.com/SantiJ10/Paraguacraft/issues)

</div>

---

## Repo

Este proyecto vive en [github.com/SantiJ10/Paraguacraft](https://github.com/SantiJ10/Paraguacraft).  
La landing está en [paraguacraft-web](https://github.com/SantiJ10/paraguacraft-web) (Cloudflare Pages).

## Desarrollo

```powershell
cd launcher
npm install
npm run tauri:dev
```

Solo la UI (sin Rust): `npm run dev` → http://127.0.0.1:1420

## Compilar en Windows (local)

```powershell
cd launcher
npm run tauri:build
```

Instalador: `launcher/src-tauri/target/release/bundle/nsis/Instalar_Paraguacraft_v6.9.0.exe`

## Compilar con GitHub Actions (Win + Mac + Linux)

El workflow está en `.github/workflows/release.yml`. Necesitás subir el código a GitHub primero.

**Opción A — desde un tag (recomendado para releases):**

```powershell
git add .
git commit -m "6.9.0"
git push origin main
git tag v6.9.0
git push origin v6.9.0
```

**Opción B — manual:** en GitHub → **Actions** → **Release** → **Run workflow**.  
Sube artefactos en la pestaña del run (no crea release; para publicar usá la opción A con tag).

Cuando termine (~15–25 min la primera vez), en **Releases** vas a ver:

| SO | Archivo |
|----|---------|
| Windows | `Instalar_Paraguacraft_v6.9.0.exe` |
| macOS | `.dmg` |
| Linux | `.AppImage`, `.deb` |

Después actualizá `latest.json` con el hash del `.exe`:

```powershell
certutil -hashfile "launcher\src-tauri\target\release\bundle\nsis\Instalar_Paraguacraft_v6.9.0.exe" SHA256
```

(Si solo compilaste en Actions, descargá el `.exe` del release y corré lo mismo.)

Campos: `version`, `download_url`, `sha256`, `size_bytes`. Mismo archivo en `web-site/public/latest.json` para la web.

## Qué subir a GitHub (launcher)

Solo lo que el launcher usa en runtime o en CI:

```
.github/workflows/release.yml
.gitignore
launcher/                 # código Tauri
bundled/pvp/              # jars del cliente PvP (descarga desde raw.githubusercontent)
latest.json
CHANGELOG.md
LICENSE
README.md
version.txt
```

**No hace falta** en el repo del launcher: `web-site/` (va a paraguacraft-web), `botamin/`, `OneConfig/`, builds viejos de Python, `launcher/dist/`, `node_modules/`, `target/`.

El mod PvP (`client/`) podés dejarlo en este repo o en otro; el launcher lo baja desde `bundled/pvp/` o releases.

## Actualizaciones

- **Web y usuarios 6.8.x:** leen `https://paraguacraft.pages.dev/latest.json`.
- **Launcher Tauri:** busca releases en GitHub; prioriza `Instalar_Paraguacraft_*.exe` en versiones ≥ 6.9.0.

## Más detalle

Ver [`launcher/README.md`](launcher/README.md).
