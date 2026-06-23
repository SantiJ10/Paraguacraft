# Paraguacraft Launcher

Stack: **Tauri 2**, **Vue 3**, **Rust**.

## Requisitos

- Node 20+
- [Rust](https://rustup.rs)
- Windows 10/11 (WebView2 ya viene instalado)

## Comandos

```bash
npm install
npm run tauri:dev      # app de escritorio
npm run dev            # solo frontend
npm run tauri:build    # instalador Windows
```

Build firmado (auto-update Tauri, opcional):

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = "src-tauri/updater.key"
npm run tauri:build:signed
```

## CurseForge

Copiá `.env.example` → `.env` con tu API key. No commitear `.env`.

## Estructura

```
src/           Vue (views, components, stores)
src-tauri/     Rust (commands/, core/)
public/        assets estáticos (iconos de versiones MC)
```

## Release

Los builds multiplataforma los hace GitHub Actions al pushear un tag `v*` (ver README en la raíz del repo).
