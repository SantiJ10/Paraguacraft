<script setup lang="ts">
withDefaults(defineProps<{ value: number; variant?: "green" | "ai" }>(), {
  variant: "green",
});
</script>

<template>
  <div class="h-2 w-full overflow-hidden rounded-full bg-surface-4">
    <div
      class="h-full rounded-full transition-all duration-300"
      :class="variant === 'ai' ? 'bg-pc-ai' : 'bg-pc-green'"
      :style="{ width: `${Math.max(0, Math.min(100, value))}%` }"
    />
  </div>
</template>
