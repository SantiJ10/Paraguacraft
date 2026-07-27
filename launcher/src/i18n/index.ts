/** i18n mínimo (es / en / pt). Ampliar catálogo progresivamente. */

export type Locale = "es" | "en" | "pt";

const messages = {
  es: {
    nav_home: "Inicio",
    nav_instances: "Instancias",
    nav_store: "Tienda",
    nav_skins: "Skins",
    nav_versions: "Versiones",
    nav_servers: "Servidores",
    nav_settings: "Ajustes",
    nav_bot: "Paraguabot",
    ready: "Listo para jugar",
    settings_appearance: "Apariencia",
    settings_theme: "Tema",
    settings_theme_dark: "Oscuro",
    settings_theme_darker: "Más oscuro",
    settings_language: "Idioma",
    settings_accent: "Color de acento",
    play_now: "Jugar ahora",
    manage: "Gestionar",
  },
  en: {
    nav_home: "Home",
    nav_instances: "Instances",
    nav_store: "Store",
    nav_skins: "Skins",
    nav_versions: "Versions",
    nav_servers: "Servers",
    nav_settings: "Settings",
    nav_bot: "Paraguabot",
    ready: "Ready to play",
    settings_appearance: "Appearance",
    settings_theme: "Theme",
    settings_theme_dark: "Dark",
    settings_theme_darker: "Darker",
    settings_language: "Language",
    settings_accent: "Accent color",
    play_now: "Play now",
    manage: "Manage",
  },
  pt: {
    nav_home: "Início",
    nav_instances: "Instâncias",
    nav_store: "Loja",
    nav_skins: "Skins",
    nav_versions: "Versões",
    nav_servers: "Servidores",
    nav_settings: "Ajustes",
    nav_bot: "Paraguabot",
    ready: "Pronto para jogar",
    settings_appearance: "Aparência",
    settings_theme: "Tema",
    settings_theme_dark: "Escuro",
    settings_theme_darker: "Mais escuro",
    settings_language: "Idioma",
    settings_accent: "Cor de destaque",
    play_now: "Jogar agora",
    manage: "Gerenciar",
  },
} as const;

export type MessageKey = keyof typeof messages.es;

let current: Locale = "es";

export function getLocale(): Locale {
  return current;
}

export function setLocale(locale: Locale) {
  current = locale;
  if (typeof document !== "undefined") {
    document.documentElement.lang = locale;
  }
}

export function t(key: MessageKey): string {
  return messages[current][key] ?? messages.es[key] ?? key;
}
