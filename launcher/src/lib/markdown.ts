/** Subconjunto seguro de Markdown → HTML (sin dependencias npm). */

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function safeUrl(url: string): string | null {
  const u = url.trim();
  if (u.startsWith("https://") || u.startsWith("http://")) return u;
  return null;
}

/** Si el cuerpo ya es HTML de CurseForge, se usa tal cual con sanitizado básico. */
export function looksLikeHtml(src: string): boolean {
  return /<\/?(p|div|h[1-6]|ul|ol|li|img|br|a|span)\b/i.test(src);
}

export function renderMarkdown(src: string): string {
  if (!src.trim()) return "";
  if (looksLikeHtml(src)) {
    return src
      .replace(/<script[\s\S]*?>[\s\S]*?<\/script>/gi, "")
      .replace(/\son\w+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi, "");
  }

  const lines = src.replace(/\r\n/g, "\n").split("\n");
  const out: string[] = [];
  let inList = false;

  const flushList = () => {
    if (inList) {
      out.push("</ul>");
      inList = false;
    }
  };

  const inline = (s: string): string => {
    let t = escapeHtml(s);
    t = t.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, (_, alt, url) => {
      const u = safeUrl(String(url));
      return u ? `<img alt="${alt}" src="${u}" class="max-h-80 rounded-lg" />` : alt;
    });
    t = t.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_, text, url) => {
      const u = safeUrl(String(url));
      return u
        ? `<a href="${u}" class="text-sky-400 underline" target="_blank" rel="noreferrer">${text}</a>`
        : text;
    });
    t = t.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
    t = t.replace(/`([^`]+)`/g, '<code class="rounded bg-black/40 px-1">$1</code>');
    return t;
  };

  for (const line of lines) {
    if (/^\s*[-*]\s+/.test(line)) {
      if (!inList) {
        out.push("<ul class=\"list-disc pl-5 space-y-1\">");
        inList = true;
      }
      out.push(`<li>${inline(line.replace(/^\s*[-*]\s+/, ""))}</li>`);
      continue;
    }
    flushList();
    if (/^###\s+/.test(line)) {
      out.push(`<h3 class="mt-4 text-lg font-bold">${inline(line.slice(4))}</h3>`);
    } else if (/^##\s+/.test(line)) {
      out.push(`<h2 class="mt-5 text-xl font-bold">${inline(line.slice(3))}</h2>`);
    } else if (/^#\s+/.test(line)) {
      out.push(`<h1 class="mt-5 text-2xl font-bold">${inline(line.slice(2))}</h1>`);
    } else if (line.trim() === "") {
      out.push("");
    } else {
      out.push(`<p class="mt-2 leading-relaxed text-gray-300">${inline(line)}</p>`);
    }
  }
  flushList();
  return out.join("\n");
}
