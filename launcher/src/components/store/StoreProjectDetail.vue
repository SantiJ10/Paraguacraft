<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { api, openUrl } from "@/lib/ipc";
import { formatNumber } from "@/composables/useFormat";
import { renderMarkdown } from "@/lib/markdown";
import type { StoreItem, StoreProjectDetail, SystemSpecs } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";

const props = defineProps<{
  item: StoreItem;
}>();

const emit = defineEmits<{
  close: [];
  install: [item: StoreItem];
}>();

const loading = ref(true);
const error = ref("");
const detail = ref<StoreProjectDetail | null>(null);
const specs = ref<SystemSpecs | null>(null);
const tab = ref<"description" | "gallery">("description");
const galleryIndex = ref(0);

onMounted(load);
watch(() => props.item.id, load);

async function load() {
  loading.value = true;
  error.value = "";
  galleryIndex.value = 0;
  try {
    const [d, s] = await Promise.all([
      api.getStoreProjectDetail({
        provider: props.item.provider,
        projectId: props.item.id,
        projectType: props.item.projectType,
      }),
      api.getSystemSpecs().catch(() => null),
    ]);
    detail.value = d;
    specs.value = s;
    if (!d.gallery.length && d.item.iconUrl) {
      d.gallery = [{ url: d.item.iconUrl, title: d.item.title }];
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

const bodyHtml = computed(() => renderMarkdown(detail.value?.body || props.item.description));

const ramWarn = computed(() => {
  const rec = detail.value?.recommendedRamGb;
  const ram = specs.value?.ramGb;
  if (!rec || ram == null) return false;
  return ram + 0.05 < rec;
});

const accent = computed(() =>
  props.item.provider === "modrinth" ? "bg-green-500 hover:bg-green-400" : "bg-orange-500 hover:bg-orange-400",
);

const gallery = computed(() => detail.value?.gallery ?? []);

function prevImg() {
  if (!gallery.value.length) return;
  galleryIndex.value = (galleryIndex.value + gallery.value.length - 1) % gallery.value.length;
}
function nextImg() {
  if (!gallery.value.length) return;
  galleryIndex.value = (galleryIndex.value + 1) % gallery.value.length;
}

async function openExternal() {
  const url = detail.value?.item.projectUrl ?? props.item.projectUrl;
  if (url) await openUrl(url);
}

function formatDate(iso?: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleDateString("es", { year: "numeric", month: "short", day: "numeric" });
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto bg-black/70 p-4 md:p-8" @click.self="emit('close')">
    <article class="relative w-full max-w-6xl overflow-hidden rounded-2xl border border-surface-3 bg-surface-1 shadow-2xl">
      <button
        class="absolute right-3 top-3 z-10 rounded-lg bg-black/50 px-3 py-1 text-sm text-white hover:bg-black/80"
        @click="emit('close')"
      >
        Cerrar
      </button>

      <header class="relative overflow-hidden border-b border-surface-3">
        <div
          class="absolute inset-0 bg-cover bg-center opacity-30 blur-md"
          :style="item.iconUrl ? { backgroundImage: `url(${item.iconUrl})` } : {}"
        />
        <div class="relative flex flex-wrap items-end gap-5 px-6 pb-5 pt-10">
          <div class="h-24 w-24 shrink-0 overflow-hidden rounded-xl border border-white/10 bg-surface-4 shadow-lg">
            <img v-if="item.iconUrl" :src="item.iconUrl" :alt="item.title" class="h-full w-full object-cover" />
          </div>
          <div class="min-w-0 flex-1">
            <p class="text-xs font-semibold uppercase tracking-wider text-gray-400">
              {{ item.projectType }} · {{ item.provider === "modrinth" ? "Modrinth" : "CurseForge" }}
            </p>
            <h2 class="text-3xl font-bold leading-tight">{{ item.title }}</h2>
            <p class="mt-1 text-sm text-gray-400">
              {{ formatNumber(item.downloads) }} descargas
              · {{ formatNumber(item.follows) }} seguidores
              <span v-if="detail?.updated"> · Actualizado {{ formatDate(detail.updated) }}</span>
            </p>
          </div>
          <BaseButton class="shrink-0" :class="accent" @click="emit('install', item)">Instalar</BaseButton>
        </div>
      </header>

      <nav class="flex gap-1 border-b border-surface-3 px-6">
        <button
          class="px-4 py-3 text-sm font-semibold"
          :class="tab === 'description' ? 'border-b-2 border-orange-400 text-white' : 'text-gray-400'"
          @click="tab = 'description'"
        >
          Descripción
        </button>
        <button
          class="px-4 py-3 text-sm font-semibold"
          :class="tab === 'gallery' ? 'border-b-2 border-orange-400 text-white' : 'text-gray-400'"
          @click="tab = 'gallery'"
        >
          Imágenes
        </button>
      </nav>

      <div v-if="loading" class="p-8 text-sm text-gray-500">Cargando ficha del proyecto…</div>
      <p v-else-if="error" class="m-6 rounded-lg bg-red-500/10 px-4 py-2 text-sm text-red-400">{{ error }}</p>

      <div v-else class="grid gap-6 p-6 lg:grid-cols-[minmax(0,1fr)_280px]">
        <section>
          <div
            v-if="ramWarn"
            class="mb-5 flex items-start gap-3 rounded-xl border border-amber-500/40 bg-amber-500/10 px-4 py-3"
          >
            <span class="mt-0.5 text-lg">⚠</span>
            <div>
              <p class="font-semibold text-amber-300">Tu PC podría tener problemas de rendimiento con este paquete</p>
              <p class="text-sm text-amber-200/80">
                Recomienda {{ detail?.recommendedRamGb }} GB de RAM y este equipo tiene
                {{ specs?.ramGb?.toFixed(0) }} GB ({{ specs?.cpuThreads }} hilos, {{ specs?.os }}).
              </p>
            </div>
          </div>

          <div v-if="tab === 'gallery' || gallery.length" class="mb-6">
            <div class="relative overflow-hidden rounded-xl bg-black">
              <img
                v-if="gallery[galleryIndex]"
                :src="gallery[galleryIndex].url"
                :alt="gallery[galleryIndex].title || item.title"
                class="max-h-[420px] w-full object-contain"
              />
              <button
                v-if="gallery.length > 1"
                class="absolute left-2 top-1/2 -translate-y-1/2 rounded-full bg-black/60 px-3 py-2 text-white"
                @click="prevImg"
              >
                ‹
              </button>
              <button
                v-if="gallery.length > 1"
                class="absolute right-2 top-1/2 -translate-y-1/2 rounded-full bg-black/60 px-3 py-2 text-white"
                @click="nextImg"
              >
                ›
              </button>
            </div>
            <div v-if="gallery.length > 1" class="mt-2 flex gap-2 overflow-x-auto">
              <button
                v-for="(g, i) in gallery"
                :key="g.url"
                class="h-14 w-20 shrink-0 overflow-hidden rounded border"
                :class="i === galleryIndex ? 'border-orange-400' : 'border-transparent opacity-70'"
                @click="galleryIndex = i"
              >
                <img :src="g.url" class="h-full w-full object-cover" alt="" />
              </button>
            </div>
          </div>

          <div v-show="tab === 'description'" class="prose-store max-w-none text-sm" v-html="bodyHtml" />
        </section>

        <aside class="space-y-4">
          <div class="rounded-xl border border-surface-3 bg-surface-2 p-4 text-sm">
            <h3 class="mb-3 font-bold">Acerca del proyecto</h3>
            <dl class="space-y-2 text-gray-300">
              <div class="flex justify-between gap-2"><dt class="text-gray-500">ID</dt><dd class="truncate">{{ item.id }}</dd></div>
              <div class="flex justify-between gap-2"><dt class="text-gray-500">Creado</dt><dd>{{ formatDate(detail?.created) }}</dd></div>
              <div class="flex justify-between gap-2"><dt class="text-gray-500">Actualizado</dt><dd>{{ formatDate(detail?.updated) }}</dd></div>
              <div class="flex justify-between gap-2"><dt class="text-gray-500">Descargas</dt><dd>{{ formatNumber(item.downloads) }}</dd></div>
              <div v-if="detail?.license" class="flex justify-between gap-2">
                <dt class="text-gray-500">Licencia</dt><dd class="truncate">{{ detail.license }}</dd>
              </div>
            </dl>
            <BaseButton class="mt-4 w-full" variant="secondary" @click="openExternal">Página oficial</BaseButton>
          </div>

          <div v-if="detail?.gameVersions?.length || detail?.loaders?.length" class="rounded-xl border border-surface-3 bg-surface-2 p-4 text-sm">
            <h3 class="mb-2 font-bold">Compatibilidad</h3>
            <p v-if="detail?.gameVersions?.length" class="text-gray-400">
              {{ detail.gameVersions.slice(0, 8).join(", ") }}
              <span v-if="detail.gameVersions.length > 8"> +{{ detail.gameVersions.length - 8 }}</span>
            </p>
            <div class="mt-2 flex flex-wrap gap-1">
              <span
                v-for="l in detail?.loaders ?? []"
                :key="l"
                class="rounded-full bg-green-500/15 px-2 py-0.5 text-xs font-semibold text-green-300"
              >
                {{ l }}
              </span>
            </div>
          </div>

          <div v-if="(detail?.item.categories.length || item.categories.length)" class="rounded-xl border border-surface-3 bg-surface-2 p-4">
            <h3 class="mb-2 text-sm font-bold">Tags</h3>
            <div class="flex flex-wrap gap-1.5">
              <span
                v-for="c in (detail?.item.categories.length ? detail.item.categories : item.categories)"
                :key="c"
                class="rounded-full bg-surface-4 px-2.5 py-0.5 text-xs text-gray-300"
              >
                {{ c }}
              </span>
            </div>
          </div>

          <div v-if="detail?.creators?.length" class="rounded-xl border border-surface-3 bg-surface-2 p-4">
            <h3 class="mb-3 text-sm font-bold">Creadores</h3>
            <ul class="space-y-2">
              <li v-for="c in detail.creators" :key="c.name" class="flex items-center gap-2">
                <img
                  v-if="c.avatarUrl"
                  :src="c.avatarUrl"
                  :alt="c.name"
                  class="h-8 w-8 rounded-full object-cover"
                />
                <div v-else class="flex h-8 w-8 items-center justify-center rounded-full bg-surface-4 text-xs">
                  {{ c.name[0] }}
                </div>
                <div>
                  <p class="text-sm font-semibold">{{ c.name }}</p>
                  <p class="text-xs text-gray-500">{{ c.role }}</p>
                </div>
              </li>
            </ul>
          </div>
        </aside>
      </div>
    </article>
  </div>
</template>
