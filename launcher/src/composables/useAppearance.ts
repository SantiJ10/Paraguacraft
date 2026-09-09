import { api } from "@/lib/ipc";

const THEME_STYLE_ID = "pc-custom-theme";
const ICON_STYLE_ID = "pc-custom-icons";

export function applyTheme(theme: string) {
  const t = theme === "darker" || theme === "light" ? theme : "dark";
  document.documentElement.dataset.theme = t;
}

export function applyIconStyle(style: string) {
  document.documentElement.dataset.iconStyle = style === "simple" ? "simple" : "filled";
}

function setInjectedCss(id: string, css: string) {
  let el = document.getElementById(id) as HTMLStyleElement | null;
  if (!css.trim()) {
    el?.remove();
    return;
  }
  if (!el) {
    el = document.createElement("style");
    el.id = id;
    document.head.appendChild(el);
  }
  el.textContent = css;
}

export async function applyCustomCss(kind: "theme" | "icons", name: string | undefined) {
  const id = kind === "icons" ? ICON_STYLE_ID : THEME_STYLE_ID;
  if (!name?.trim()) {
    setInjectedCss(id, "");
    return;
  }
  try {
    const css = await api.readCustomThemeCss(kind, name);
    setInjectedCss(id, css);
  } catch {
    setInjectedCss(id, "");
  }
}
