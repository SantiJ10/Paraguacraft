<script setup lang="ts">
import TitleBar from "@/components/layout/TitleBar.vue";
</script>

<template>
  <div class="flex h-screen flex-col bg-surface-1">
    <TitleBar />
    <RouterView />
  </div>
</template>
