<script setup lang="ts">
const model = defineModel<string>({ default: "" });
defineProps<{ placeholder?: string }>();
</script>

<template>
  <div class="relative">
    <svg
      class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-500"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <circle cx="11" cy="11" r="7" />
      <path d="m21 21-4.3-4.3" />
    </svg>
    <input
      v-model="model"
      type="text"
      :placeholder="placeholder ?? 'Buscar...'"
      class="w-full rounded-lg border border-surface-5 bg-surface-3 py-2.5 pl-9 pr-3 text-sm outline-none transition-colors placeholder:text-gray-500 focus:border-pc-green"
    />
  </div>
</template>
