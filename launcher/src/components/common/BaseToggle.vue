<script setup lang="ts">
const model = defineModel<boolean>({ default: false });
defineProps<{ label?: string; hint?: string }>();
</script>

<template>
  <label class="flex items-center justify-between gap-4 cursor-pointer">
    <span v-if="label || hint" class="flex flex-col">
      <span v-if="label" class="text-sm font-medium text-white">{{ label }}</span>
      <span v-if="hint" class="text-xs text-gray-500">{{ hint }}</span>
    </span>
    <button
      type="button"
      role="switch"
      :aria-checked="model"
      class="relative h-6 w-11 shrink-0 rounded-full transition-colors"
      :class="model ? 'bg-pc-green' : 'bg-surface-5'"
      @click="model = !model"
    >
      <span
        class="absolute top-0.5 h-5 w-5 rounded-full bg-white transition-transform"
        :class="model ? 'translate-x-5' : 'translate-x-0.5'"
      />
    </button>
  </label>
</template>
