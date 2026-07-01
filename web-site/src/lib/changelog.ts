import fs from "node:fs";
import path from "node:path";

export type ChangelogEntry = {
  version: string;
  date: string;
  sections: { heading: string; items: string[] }[];
};

export function parseChangelog(md: string): ChangelogEntry[] {
  if (!md) return [];
  const entries: ChangelogEntry[] = [];
  const lines = md.split(/\r?\n/);
  let current: ChangelogEntry | null = null;
  let currentSection: { heading: string; items: string[] } | null = null;

  for (const ln of lines) {
    const mVer = ln.match(/^##\s+\[?([\d.]+)\]?\s*[-–—]\s*(.+)$/);
    if (mVer) {
      if (current) entries.push(current);
      current = { version: mVer[1], date: mVer[2].trim(), sections: [] };
      currentSection = null;
      continue;
    }
    if (!current) continue;

    const mSection = ln.match(/^###\s+(.+)$/);
    if (mSection) {
      currentSection = { heading: mSection[1].trim(), items: [] };
      current.sections.push(currentSection);
      continue;
    }

    const mItem = ln.match(/^[-*]\s+(.+)$/);
    if (mItem && currentSection) {
      currentSection.items.push(mItem[1].trim());
    }
  }
  if (current) entries.push(current);
  return entries;
}

export function loadChangelog(limit = 4): ChangelogEntry[] {
  const candidates = [
    path.resolve("./data/changelog.md"),
    path.resolve("../CHANGELOG.md"),
  ];
  for (const p of candidates) {
    try {
      const all = parseChangelog(fs.readFileSync(p, "utf8"));
      if (all.length) return all.slice(0, limit);
    } catch {
      /* try next */
    }
  }
  return [
    {
      version: "7.0.2",
      date: "2026-06-23",
      sections: [
        {
          heading: "Novedades",
          items: [
            "Reparar instancia, visor de logs y presets RAM",
            "Gestor de mods, modpacks CurseForge .zip, exportar instancia",
            "Servidores favoritos con join directo",
          ],
        },
      ],
    },
  ];
}

export function inlineMd(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/\*\*(.+?)\*\*/g, '<b class="text-fg">$1</b>')
    .replace(
      /`([^`]+)`/g,
      '<code class="font-mono text-[0.85em] px-1.5 py-0.5 rounded bg-bg-soft border border-bg-border text-brand-300">$1</code>',
    )
    .replace(
      /\[(.+?)\]\((.+?)\)/g,
      '<a href="$2" target="_blank" rel="noopener" class="text-brand-300 hover:text-brand-200 underline underline-offset-2">$1</a>',
    );
}
