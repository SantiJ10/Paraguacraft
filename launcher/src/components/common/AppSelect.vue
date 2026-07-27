<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from "vue";

export interface AppSelectOption {
  value: string;
  label: string;
  hint?: string;
}

const props = withDefaults(
  defineProps<{
    options: AppSelectOption[];
    placeholder?: string;
    disabled?: boolean;
    searchable?: boolean;
    maxPanelHeight?: number;
  }>(),
  {
    placeholder: "Seleccionar…",
    disabled: false,
    searchable: false,
    maxPanelHeight: 280,
  },
);

const model = defineModel<string>({ default: "" });

const open = ref(false);
const query = ref("");
const trigger = ref<HTMLElement | null>(null);
const panel = ref<HTMLElement | null>(null);
const placement = ref<"down" | "up">("down");
const panelStyle = ref<Record<string, string>>({});

const selectedLabel = computed(() => {
  const hit = props.options.find((o) => o.value === model.value);
  return hit?.label ?? props.placeholder;
});

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return props.options;
  return props.options.filter(
    (o) => o.label.toLowerCase().includes(q) || o.value.toLowerCase().includes(q),
  );
});

function updatePlacement() {
  const el = trigger.value;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  const spaceBelow = window.innerHeight - rect.bottom - 8;
  const spaceAbove = rect.top - 8;
  const want = props.maxPanelHeight;
  const openUp = spaceBelow < Math.min(want, 160) && spaceAbove > spaceBelow;
  placement.value = openUp ? "up" : "down";
  const maxH = Math.max(120, Math.min(want, openUp ? spaceAbove : spaceBelow));
  panelStyle.value = {
    left: `${rect.left}px`,
    width: `${rect.width}px`,
    maxHeight: `${maxH}px`,
    ...(openUp
      ? { bottom: `${window.innerHeight - rect.top + 4}px`, top: "auto" }
      : { top: `${rect.bottom + 4}px`, bottom: "auto" }),
  };
}

async function toggle() {
  if (props.disabled) return;
  open.value = !open.value;
  if (open.value) {
    query.value = "";
    await nextTick();
    updatePlacement();
  }
}

function select(value: string) {
  model.value = value;
  open.value = false;
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") open.value = false;
}

function onPointerDown(e: PointerEvent) {
  const t = e.target as Node;
  if (trigger.value?.contains(t) || panel.value?.contains(t)) return;
  open.value = false;
}

watch(open, (v) => {
  if (v) {
    window.addEventListener("keydown", onKey);
    window.addEventListener("pointerdown", onPointerDown, true);
    window.addEventListener("resize", updatePlacement);
    window.addEventListener("scroll", updatePlacement, true);
  } else {
    window.removeEventListener("keydown", onKey);
    window.removeEventListener("pointerdown", onPointerDown, true);
    window.removeEventListener("resize", updatePlacement);
    window.removeEventListener("scroll", updatePlacement, true);
  }
});

onUnmounted(() => {
  open.value = false;
});
</script>

<template>
  <div class="relative w-full">
    <button
      ref="trigger"
      type="button"
      class="flex w-full items-center justify-between gap-2 rounded-xl border bg-surface-3 px-3 py-2.5 text-left text-sm outline-none transition-colors"
      :class="
        open
          ? 'border-pc-green text-white'
          : 'border-surface-5 text-gray-200 hover:border-surface-6'
      "
      :disabled="disabled"
      @click="toggle"
    >
      <span class="truncate">{{ selectedLabel }}</span>
      <svg
        class="h-4 w-4 shrink-0 text-gray-400 transition-transform"
        :class="open ? 'rotate-180' : ''"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M6 9l6 6 6-6" />
      </svg>
    </button>

    <Teleport to="body">
      <div
        v-if="open"
        ref="panel"
        class="fixed z-[450] overflow-hidden rounded-xl border border-surface-5 bg-surface-2 shadow-2xl"
        :style="panelStyle"
        @contextmenu.prevent
      >
        <div v-if="searchable" class="border-b border-surface-4 p-2">
          <input
            v-model="query"
            type="search"
            placeholder="Buscar…"
            class="w-full rounded-lg border border-surface-5 bg-surface-3 px-2.5 py-1.5 text-sm outline-none focus:border-pc-green"
            @keydown.stop
          />
        </div>
        <div class="overflow-y-auto" :style="{ maxHeight: panelStyle.maxHeight }">
          <button
            v-for="opt in filtered"
            :key="opt.value"
            type="button"
            class="flex w-full items-center justify-between gap-2 px-3 py-2 text-left text-sm transition-colors"
            :class="
              opt.value === model
                ? 'bg-pc-green/15 text-pc-green'
                : 'text-gray-200 hover:bg-surface-4'
            "
            @click="select(opt.value)"
          >
            <span class="truncate">{{ opt.label }}</span>
            <span v-if="opt.hint" class="shrink-0 text-[10px] text-gray-500">{{ opt.hint }}</span>
          </button>
          <p v-if="!filtered.length" class="px-3 py-4 text-center text-xs text-gray-500">Sin resultados</p>
        </div>
      </div>
    </Teleport>
  </div>
</template>
