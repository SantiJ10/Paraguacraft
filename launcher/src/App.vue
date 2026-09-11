<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import TitleBar from "@/components/layout/TitleBar.vue";
import PostCrashBanner from "@/components/layout/PostCrashBanner.vue";
import GameConsolePanel from "@/components/layout/GameConsolePanel.vue";
import { useSettingsStore } from "@/stores/settings";
import { applyAccentTheme } from "@/composables/useAccent";
import {
  applyTheme,
  applyIconStyle,
  applyCustomCss,
  applyWallpaper,
  wallpaperCss,
  wallpaperCustomSrc,
} from "@/composables/useAppearance";
import { setLocale, type Locale } from "@/i18n";

const settings = useSettingsStore();
const customWallpaperUrl = ref("");

const wallpaperStyle = computed(() => {
  const bg = wallpaperCss(settings.settings?.wallpaper, customWallpaperUrl.value);
  if (!bg) return {};
  return {
    backgroundImage: bg,
    backgroundSize: "cover",
    backgroundPosition: "center",
  };
});

function applyLanguage(lang: string) {
  setLocale((lang as Locale) || "es");
}

async function refreshWallpaper() {
  applyWallpaper(settings.settings?.wallpaper);
  if (settings.settings?.wallpaper === "custom") {
    customWallpaperUrl.value = await wallpaperCustomSrc(settings.settings.wallpaperCustomPath);
  } else {
    customWallpaperUrl.value = "";
  }
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
    void refreshWallpaper();
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

watch(
  () => [settings.settings?.wallpaper, settings.settings?.wallpaperCustomPath],
  () => {
    void refreshWallpaper();
  },
);
</script>

<template>
  <div class="relative flex h-screen flex-col bg-surface-1">
    <div
      class="pc-wallpaper pointer-events-none absolute inset-0 z-0"
      aria-hidden="true"
      :style="wallpaperStyle"
    />
    <div class="relative z-10 flex min-h-0 flex-1 flex-col">
      <TitleBar />
      <PostCrashBanner />
      <RouterView />
      <GameConsolePanel />
    </div>
  </div>
</template>
