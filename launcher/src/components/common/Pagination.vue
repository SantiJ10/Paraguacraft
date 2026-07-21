<script setup lang="ts">
import { computed } from "vue";

/**
 * Paginación por número de página (estilo Modrinth: « 1 2 … 3000 »), calculada
 * a partir de `offset`/`limit`/`totalHits` que devuelve la tienda.
 */
const props = defineProps<{
  offset: number;
  limit: number;
  totalHits: number;
  disabled?: boolean;
}>();

const emit = defineEmits<{ (e: "update:offset", offset: number): void }>();

const currentPage = computed(() => Math.floor(props.offset / Math.max(1, props.limit)) + 1);
const totalPages = computed(() => Math.max(1, Math.ceil(props.totalHits / Math.max(1, props.limit))));

/** Páginas visibles: siempre 1 y la última, + una ventana alrededor de la actual. */
const pages = computed<Array<number | "gap">>(() => {
  const total = totalPages.value;
  const current = currentPage.value;
  if (total <= 7) {
    return Array.from({ length: total }, (_, i) => i + 1);
  }
  const result = new Set<number>([1, 2, total - 1, total]);
  for (let p = current - 1; p <= current + 1; p++) {
    if (p >= 1 && p <= total) result.add(p);
  }
  const sorted = [...result].sort((a, b) => a - b);
  const out: Array<number | "gap"> = [];
  let prev = 0;
  for (const p of sorted) {
    if (prev && p - prev > 1) out.push("gap");
    out.push(p);
    prev = p;
  }
  return out;
});

function goTo(page: number) {
  if (props.disabled || page < 1 || page > totalPages.value || page === currentPage.value) return;
  emit("update:offset", (page - 1) * props.limit);
}
</script>

<template>
  <nav v-if="totalPages > 1" class="flex flex-wrap items-center justify-center gap-1.5 py-4" aria-label="Paginación">
    <button
      type="button"
      class="flex h-8 min-w-8 items-center justify-center rounded-lg px-2 text-sm font-semibold text-gray-400 transition-colors hover:bg-surface-4 hover:text-white disabled:pointer-events-none disabled:opacity-30"
      :disabled="disabled || currentPage === 1"
      @click="goTo(currentPage - 1)"
    >
      &lsaquo;
    </button>

    <template v-for="(p, i) in pages" :key="`${p}-${i}`">
      <span v-if="p === 'gap'" class="px-1 text-sm text-gray-500">…</span>
      <button
        v-else
        type="button"
        class="flex h-8 min-w-8 items-center justify-center rounded-lg px-2 text-sm font-semibold transition-colors"
        :class="
          p === currentPage
            ? 'bg-pc-green text-black'
            : 'text-gray-400 hover:bg-surface-4 hover:text-white'
        "
        :disabled="disabled"
        @click="goTo(p)"
      >
        {{ p }}
      </button>
    </template>

    <button
      type="button"
      class="flex h-8 min-w-8 items-center justify-center rounded-lg px-2 text-sm font-semibold text-gray-400 transition-colors hover:bg-surface-4 hover:text-white disabled:pointer-events-none disabled:opacity-30"
      :disabled="disabled || currentPage === totalPages"
      @click="goTo(currentPage + 1)"
    >
      &rsaquo;
    </button>
  </nav>
</template>
