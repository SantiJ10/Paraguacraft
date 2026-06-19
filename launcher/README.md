# Paraguacraft Launcher (Tauri v2 + Vue 3)

Reescritura del launcher de Paraguacraft a un cliente de escritorio multiplataforma de grado comercial.

- **Frontend**: Vue 3 (`<script setup>`) + Pinia + vue-router + Tailwind CSS + TypeScript.
- **Backend**: Rust + Tauri v2, modular (`commands/` -> `core/`).
- Los archivos Python/HTML originales del repo raiz se mantienen como **referencia de logica** y no se tocan.

## Estado: FASE 1 (UI/UX y navegacion)

Implementado en esta fase:

- Wizard de bienvenida (bienvenida -> hardware/perfil -> cuenta -> tema/fin).
- Layout principal estilo Modrinth/Lunar: barra de titulo propia, sidebar, status bar y panel de IA.
- Vistas con datos **mock**: Inicio, Instancias, Tienda, Versiones (selector version + loader + version exacta), Ajustes.
- Capa IPC (`src/lib/ipc.ts`) que hoy devuelve mocks y delega en Tauri cuando hay backend.
- Comando Rust real `get_hardware_info` (deteccion con `sysinfo`) como prueba del puente.

Las fases 2-4 (cuentas, instancias, descargas, tienda real, servidores, IA, auto-update) tienen sus modulos Rust ya esbozados en `src-tauri/src/core/`.

## Requisitos

- Node.js 18+ y npm.
- Rust (https://rustup.rs) + toolchain estable.
- Dependencias de sistema de Tauri v2 (WebView2 en Windows ya viene con Win10/11).

## Puesta en marcha

```bash
cd launcher
npm install
```

### Opcion A - Previsualizar solo la UI (sin Rust, en el navegador)

```bash
npm run dev
# abrir http://localhost:1420  (badge "WEB DEMO" en la status bar)
```

### Opcion B - App de escritorio completa (Tauri)

1. Generar los iconos una sola vez desde un PNG cuadrado existente:

```bash
npx tauri icon ../web/assets/iconomc.png
```

2. Lanzar en modo desarrollo:

```bash
npm run tauri:dev
```

> El badge de la status bar mostrara "TAURI" y `get_hardware_info` devolvera tu hardware real.

## Build de produccion

```bash
npm run tauri:build
```

## Estructura

```
launcher/
  src/                 # Frontend Vue
    components/         # common/, layout/, wizard/
    composables/        # useFormat, useHardware
    layouts/            # MainLayout
    lib/                # ipc.ts (abstraccion) + types.ts + mock/
    router/  stores/  views/
  src-tauri/           # Backend Rust
    src/commands/       # handlers IPC finos
    src/core/           # logica por dominio (hardware listo; resto Fase 2-4)
    src/models/         # structs serde (camelCase)
```
