# Paraguacraft — Landing (Astro)

Sitio estático para [paraguacraft.pages.dev](https://paraguacraft.pages.dev).

## Desarrollo

```bash
cd web-site
npm install
npm run dev
```

## Build (Cloudflare Pages)

```bash
npm run build
```

- **Build command:** `npm run build`
- **Output directory:** `dist`
- Copiar `public/latest.json` actualizado en cada release (mismo manifest que la raíz del repo).

## Componentes

| Archivo | Rol |
|---------|-----|
| `Hero.astro` | Hero + nav + botón descarga (`fetch /latest.json`) |
| `Features.astro` | Tarjetas de features |
| `Stats.astro` | Métricas de rendimiento + perfiles hardware |
| `Footer.astro` | Links y copyright |

Colores: verde `#2ECC71`, púrpura IA `#9B59B6`, fondo `#0A0A0B`.
