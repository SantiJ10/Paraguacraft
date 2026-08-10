<script setup lang="ts">
import { onMounted, watch } from "vue";
import TitleBar from "@/components/layout/TitleBar.vue";
import PostCrashBanner from "@/components/layout/PostCrashBanner.vue";
import GameConsolePanel from "@/components/layout/GameConsolePanel.vue";
import { useSettingsStore } from "@/stores/settings";
import { applyAccentTheme } from "@/composables/useAccent";
import { setLocale, type Locale } from "@/i18n";

const settings = useSettingsStore();

function applyTheme(theme: string) {
  document.documentElement.dataset.theme = theme === "darker" ? "darker" : "dark";
}

function applyLanguage(lang: string) {
  setLocale((lang as Locale) || "es");
}

onMounted(() => {
  // Tema lo antes posible sin bloquear el mount del shell.
  void settings.load().then(() => {
    applyAccentTheme(settings.settings?.accent ?? "green");
    applyTheme(settings.settings?.theme ?? "dark");
    applyLanguage(settings.settings?.language ?? "es");
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
