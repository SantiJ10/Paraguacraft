<script setup lang="ts">
import { onMounted, watch } from "vue";
import TitleBar from "@/components/layout/TitleBar.vue";
import PostCrashBanner from "@/components/layout/PostCrashBanner.vue";
import { useSettingsStore } from "@/stores/settings";
import { applyAccentTheme } from "@/composables/useAccent";

const settings = useSettingsStore();

onMounted(async () => {
  await settings.load();
  applyAccentTheme(settings.settings?.accent ?? "green");
});

watch(
  () => settings.settings?.accent,
  (accent) => {
    if (accent) applyAccentTheme(accent);
  },
);
</script>

<template>
  <div class="flex h-screen flex-col bg-surface-1">
    <TitleBar />
    <PostCrashBanner />
    <RouterView />
  </div>
</template>
