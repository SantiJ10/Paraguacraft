<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";

export interface ContextMenuItem {
  id: string;
  label: string;
  danger?: boolean;
  disabled?: boolean;
  separator?: boolean;
}

const props = defineProps<{
  x: number;
  y: number;
  items: ContextMenuItem[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "select", id: string): void;
}>();

const root = ref<HTMLElement | null>(null);
const pos = ref({ left: props.x, top: props.y });

const visibleItems = computed(() => props.items.filter((i) => i.separator || i.label));

function clampPosition() {
  const el = root.value;
  if (!el) return;
  const pad = 8;
  const rect = el.getBoundingClientRect();
  let left = props.x;
  let top = props.y;
  if (left + rect.width > window.innerWidth - pad) {
    left = Math.max(pad, window.innerWidth - rect.width - pad);
  }
  if (top + rect.height > window.innerHeight - pad) {
    top = Math.max(pad, window.innerHeight - rect.height - pad);
  }
  pos.value = { left, top };
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") emit("close");
}

function onPointerDown(e: PointerEvent) {
  if (root.value && !root.value.contains(e.target as Node)) {
    emit("close");
  }
}

watch(
  () => [props.x, props.y, props.items.length],
  async () => {
    await nextTick();
    clampPosition();
  },
);

onMounted(async () => {
  await nextTick();
  clampPosition();
  window.addEventListener("keydown", onKey);
  window.addEventListener("pointerdown", onPointerDown, true);
  window.addEventListener("resize", clampPosition);
  window.addEventListener("scroll", () => emit("close"), true);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKey);
  window.removeEventListener("pointerdown", onPointerDown, true);
  window.removeEventListener("resize", clampPosition);
});
</script>

<template>
  <Teleport to="body">
    <div
      ref="root"
      class="fixed z-[500] min-w-[180px] overflow-hidden rounded-xl border border-surface-5 bg-surface-2 py-1 shadow-2xl"
      :style="{ left: `${pos.left}px`, top: `${pos.top}px` }"
      role="menu"
      @contextmenu.prevent
    >
      <template v-for="(item, idx) in visibleItems" :key="`${item.id}-${idx}`">
        <div v-if="item.separator" class="my-1 border-t border-surface-4" />
        <button
          v-else
          type="button"
          role="menuitem"
          class="flex w-full items-center px-3 py-2 text-left text-sm transition-colors disabled:cursor-not-allowed disabled:opacity-40"
          :class="
            item.danger
              ? 'text-red-400 hover:bg-red-500/15'
              : 'text-gray-200 hover:bg-surface-4'
          "
          :disabled="item.disabled"
          @click="emit('select', item.id)"
        >
          {{ item.label }}
        </button>
      </template>
    </div>
  </Teleport>
</template>
