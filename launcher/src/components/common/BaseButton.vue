<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "ghost" | "ai" | "danger";
    size?: "sm" | "md" | "lg";
    block?: boolean;
    disabled?: boolean;
  }>(),
  { variant: "primary", size: "md", block: false, disabled: false },
);

const classes = computed(() => {
  const base =
    "inline-flex items-center justify-center gap-2 rounded-lg font-semibold transition-all disabled:opacity-50 disabled:cursor-not-allowed select-none";
  const sizes = {
    sm: "px-3 py-1.5 text-xs",
    md: "px-4 py-2.5 text-sm",
    lg: "px-6 py-3.5 text-base",
  }[props.size];
  const variants = {
    primary: "bg-pc-green text-black hover:bg-pc-green-hover",
    secondary: "bg-surface-5 text-white hover:bg-surface-6",
    ghost: "bg-transparent text-gray-300 hover:bg-surface-3 hover:text-white",
    ai: "bg-pc-ai text-white hover:bg-pc-ai-dark shadow-glow-ai",
    danger: "bg-red-600/90 text-white hover:bg-red-600",
  }[props.variant];
  return [base, sizes, variants, props.block ? "w-full" : ""].join(" ");
});
</script>

<template>
  <button :class="classes" :disabled="disabled">
    <slot />
  </button>
</template>
