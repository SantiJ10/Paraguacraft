<script setup lang="ts">
const model = defineModel<boolean>({ default: false });
defineProps<{ label?: string; hint?: string }>();
</script>

<template>
  <label class="flex cursor-pointer items-center justify-between gap-4">
    <span v-if="label || hint" class="flex flex-col">
      <span v-if="label" class="text-sm font-medium text-white">{{ label }}</span>
      <span v-if="hint" class="text-xs text-gray-500">{{ hint }}</span>
    </span>
    <button
      type="button"
      role="switch"
      :aria-checked="model"
      class="relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition-colors"
      :class="model ? 'bg-pc-green' : 'bg-surface-5'"
      @click="model = !model"
    >
      <span
        class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-sm transition-transform duration-200"
        :class="model ? 'translate-x-[1.375rem]' : 'translate-x-0.5'"
      />
    </button>
  </label>
</template>
