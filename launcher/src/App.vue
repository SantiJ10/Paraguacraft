<script setup lang="ts">
import { onMounted, watch } from "vue";
import TitleBar from "@/components/layout/TitleBar.vue";
import PostCrashBanner from "@/components/layout/PostCrashBanner.vue";
import GameConsolePanel from "@/components/layout/GameConsolePanel.vue";
import { useSettingsStore } from "@/stores/settings";
import { applyAccentTheme } from "@/composables/useAccent";
import { applyTheme, applyIconStyle, applyCustomCss } from "@/composables/useAppearance";
import { setLocale, type Locale } from "@/i18n";

const settings = useSettingsStore();

function applyLanguage(lang: string) {
  setLocale((lang as Locale) || "es");
}

onMounted(() => {
  // Tema lo antes posible sin bloquear el mount del shell.
  void settings.load().then(() => {
    applyAccentTheme(settings.settings?.accent ?? "green");
    applyTheme(settings.settings?.theme ?? "dark");
    applyIconStyle(settings.settings?.iconStyle ?? "filled");
    applyLanguage(settings.settings?.language ?? "es");
    void applyCustomCss("theme", settings.settings?.customTheme);
    void applyCustomCss("icons", settings.settings?.customIconTheme);
  });
});

watch(
  () => settings.settings?.accent,
  (accent) => {
    if (accent) applyAccentTheme(accent);
  },
);

watch(
  () => settings.settings?.theme,
  (theme) => {
    if (theme) applyTheme(theme);
  },
);

watch(
  () => settings.settings?.iconStyle,
  (style) => {
    applyIconStyle(style ?? "filled");
  },
);

watch(
  () => settings.settings?.customTheme,
  (name) => {
    void applyCustomCss("theme", name);
  },
);

watch(
  () => settings.settings?.customIconTheme,
  (name) => {
    void applyCustomCss("icons", name);
  },
);

watch(
  () => settings.settings?.language,
  (lang) => {
    if (lang) applyLanguage(lang);
  },
);
</script>

<template>
  <div class="flex h-screen flex-col bg-surface-1">
    <TitleBar />
    <PostCrashBanner />
    <RouterView />
    <GameConsolePanel />
  </div>
</template>
