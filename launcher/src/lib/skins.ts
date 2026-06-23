/** UUID de Steve (fallback visual cuando no hay cuenta o falla Crafatar). */
export const STEVE_UUID = "8667ba71-b85a-4004-af54-4576b382cd64";

/** URL de cabeza 2D vía Minotar (más fiable en WebView que Crafatar). */
export function minotarHelm(id: string, size = 64): string {
  const clean = id.replace(/-/g, "");
  return `https://minotar.net/helm/${encodeURIComponent(clean || id)}/${size}.png`;
}

export function minotarBody(id: string, size = 160): string {
  const clean = id.replace(/-/g, "");
  return `https://minotar.net/armor/body/${encodeURIComponent(clean || id)}/${size}.png`;
}

/** PNG de skin completa (64×64) para visor 3D. */
export function minotarSkin(username: string): string {
  return `https://minotar.net/skin/${encodeURIComponent(username.trim())}`;
}

/** @deprecated usar minotarHelm */
export function crafatarAvatar(uuid: string, size = 128): string {
  return minotarHelm(uuid, size);
}

export const STEVE_AVATAR_URL = minotarHelm(STEVE_UUID, 64);

export function crafatarBody(uuid: string): string {
  return minotarBody(uuid, 160);
}
