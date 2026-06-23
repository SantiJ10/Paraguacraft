/** URL fija del flujo device-code de Microsoft (QR / código). */
export const MS_DEVICE_LINK = "https://www.microsoft.com/link";

/** Imagen QR que abre microsoft.com/link al escanear. */
export function msLoginQrImageUrl(size = 168): string {
  return `https://api.qrserver.com/v1/create-qr-code/?size=${size}x${size}&data=${encodeURIComponent(MS_DEVICE_LINK)}`;
}

/** Extrae el `code` OAuth de la URL de redirección del navegador. */
export function parseMsAuthCode(redirectUrl: string): string | null {
  const trimmed = redirectUrl.trim();
  if (!trimmed) return null;
  try {
    const url = new URL(trimmed);
    const code = url.searchParams.get("code");
    if (code) return decodeURIComponent(code);
  } catch {
    /* URL relativa o mal formada */
  }
  const match = trimmed.match(/[?&]code=([^&]+)/);
  return match ? decodeURIComponent(match[1]) : null;
}
