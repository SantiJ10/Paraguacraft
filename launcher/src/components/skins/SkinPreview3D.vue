<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { IdleAnimation, SkinViewer } from "skinview3d";

const props = withDefaults(
  defineProps<{
    skinUrl?: string | null;
    model?: "classic" | "slim" | "default";
    capeUrl?: string | null;
    username?: string | null;
    height?: number;
    showHint?: boolean;
  }>(),
  { model: "classic", height: 380, showHint: true },
);

const containerRef = ref<HTMLElement | null>(null);
const loading = ref(false);
const loadError = ref<string | null>(null);

let viewer: SkinViewer | null = null;
let resizeObs: ResizeObserver | null = null;

const hasSkin = computed(() => Boolean(props.skinUrl?.trim()));

function mapModel(m: string): "default" | "slim" {
  return m === "slim" ? "slim" : "default";
}

function disposeViewer() {
  resizeObs?.disconnect();
  resizeObs = null;
  if (viewer) {
    try {
      viewer.dispose();
    } catch {
      /* ignore */
    }
    viewer.canvas.remove();
    viewer = null;
  }
}

function syncSize() {
  if (!viewer || !containerRef.value) return;
  const w = Math.max(containerRef.value.clientWidth, 200);
  viewer.setSize(w, props.height);
}

async function applySkin() {
  if (!viewer) return;
  const url = props.skinUrl?.trim();
  if (!url) {
    viewer.loadSkin(null);
    viewer.loadCape(null);
    return;
  }

  loading.value = true;
  loadError.value = null;
  try {
    await viewer.loadSkin(url, { model: mapModel(props.model) });
    const cape = props.capeUrl?.trim();
    if (cape) {
      await viewer.loadCape(cape);
    } else {
      viewer.loadCape(null);
    }
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

function initViewer() {
  if (!containerRef.value) return;
  disposeViewer();

  const width = Math.max(containerRef.value.clientWidth, 200);
  viewer = new SkinViewer({
    width,
    height: props.height,
    enableControls: true,
  });

  const canvas = viewer.canvas;
  canvas.classList.add("skin-preview-canvas");
  canvas.style.touchAction = "none";
  containerRef.value.appendChild(canvas);

  viewer.controls.enablePan = false;
  viewer.controls.enableRotate = true;
  viewer.controls.enableZoom = true;
  viewer.controls.rotateSpeed = 0.55;
  viewer.controls.zoomSpeed = 0.7;
  viewer.controls.minPolarAngle = 0.15;
  viewer.controls.maxPolarAngle = Math.PI - 0.15;
  viewer.autoRotate = false;
  viewer.animation = new IdleAnimation();
  viewer.animation.speed = 0.6;

  resizeObs = new ResizeObserver(() => syncSize());
  resizeObs.observe(containerRef.value);

  void applySkin();
}

watch(
  () => [props.skinUrl, props.model, props.capeUrl] as const,
  () => {
    if (viewer) void applySkin();
  },
);

watch(
  () => props.height,
  () => syncSize(),
);

onMounted(() => {
  initViewer();
});

onUnmounted(disposeViewer);
</script>

<template>
  <div
    class="relative flex min-h-[280px] w-full flex-col overflow-hidden rounded-xl border border-surface-4 bg-surface-2"
    style="background: radial-gradient(circle at 50% 35%, rgba(46, 204, 113, 0.1), transparent 55%)"
  >
    <div
      ref="containerRef"
      class="relative min-h-0 flex-1 cursor-grab active:cursor-grabbing"
      :style="{ minHeight: `${height}px` }"
    />

    <div
      v-if="!hasSkin"
      class="pointer-events-none absolute inset-0 flex flex-col items-center justify-center gap-2 px-6 text-center"
    >
      <span class="text-3xl opacity-40">🧍</span>
      <p class="text-sm text-gray-500">Buscá o importá una skin para ver la vista 3D</p>
    </div>

    <div
      v-if="loading"
      class="pointer-events-none absolute inset-x-0 top-3 text-center text-xs text-gray-400"
    >
      Cargando textura…
    </div>

    <p v-if="loadError" class="absolute inset-x-0 bottom-10 px-4 text-center text-xs text-red-400">
      {{ loadError }}
    </p>

    <p
      v-if="hasSkin && showHint"
      class="pointer-events-none absolute inset-x-0 bottom-2 text-center text-[11px] text-gray-500"
    >
      Arrastrá para rotar · rueda del mouse para zoom
      <span v-if="username" class="text-gray-400"> · {{ username }}</span>
    </p>
  </div>
</template>

<style scoped>
:deep(.skin-preview-canvas) {
  display: block;
  width: 100% !important;
  height: 100% !important;
}
</style>
