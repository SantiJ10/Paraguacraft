<script setup lang="ts">
import ProfileChip from "@/components/layout/ProfileChip.vue";
import MusicWidget from "@/components/layout/MusicWidget.vue";
import MusicPanel from "@/components/layout/MusicPanel.vue";
</script>

<template>
  <header class="relative flex shrink-0 items-center justify-end gap-2 border-b border-surface-3/60 bg-surface-0/50 px-4 py-2">
    <div class="relative">
      <MusicWidget />
      <MusicPanel />
    </div>
    <ProfileChip />
  </header>
</template>
