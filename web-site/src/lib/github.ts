export type ReleaseStatus = {
  tag: string;
  name: string;
  publishedAt: string;
  htmlUrl: string;
  downloadUrl: string | null;
  ok: boolean;
};

export async function fetchReleaseStatus(): Promise<ReleaseStatus | null> {
  try {
    const res = await fetch(
      "https://api.github.com/repos/SantiJ10/Paraguacraft/releases/latest",
      {
        headers: { Accept: "application/vnd.github+json", "User-Agent": "paraguacraft-web" },
      },
    );
    if (!res.ok) return null;
    const data = await res.json();
    const asset =
      (data.assets as { name: string; browser_download_url: string }[] | undefined)?.find(
        (a) => a.name.startsWith("Instalar_Paraguacraft_") && a.name.endsWith(".exe"),
      ) ?? null;
    return {
      tag: String(data.tag_name ?? ""),
      name: String(data.name ?? data.tag_name ?? "Release"),
      publishedAt: String(data.published_at ?? ""),
      htmlUrl: String(data.html_url ?? ""),
      downloadUrl: asset?.browser_download_url ?? null,
      ok: true,
    };
  } catch {
    return null;
  }
}
