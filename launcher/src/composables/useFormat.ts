// Helpers de formato reutilizables en toda la UI.

export function formatNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return `${n}`;
}

export function formatPlaytime(minutes: number): string {
  if (minutes <= 0) return "Nunca jugado";
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  if (h === 0) return `${m} min`;
  return `${h} h ${m} min`;
}

export function formatRelative(iso: string | null): string {
  if (!iso) return "Nunca";
  const diff = Date.now() - new Date(iso).getTime();
  const day = 86_400_000;
  if (diff < 3_600_000) return "Hace poco";
  if (diff < day) return "Hoy";
  if (diff < 2 * day) return "Ayer";
  return `Hace ${Math.floor(diff / day)} dias`;
}

export function formatRam(mb: number): string {
  return `${(mb / 1024).toFixed(mb % 1024 === 0 ? 0 : 1)} GB`;
}
