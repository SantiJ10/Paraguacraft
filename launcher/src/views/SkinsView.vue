<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { api, isTauri } from "@/lib/ipc";
import { minotarBody, minotarHelm, minotarSkin } from "@/lib/skins";
import BaseButton from "@/components/common/BaseButton.vue";
import SkinAvatar from "@/components/account/SkinAvatar.vue";
import SkinPreview3D from "@/components/skins/SkinPreview3D.vue";
import { useAccountsStore } from "@/stores/accounts";
import { useSkinsStore } from "@/stores/skins";
import type { ApplySkinResult, SkinCatalogEntry, SkinCatalogPage, SkinHistoryEntry, SkinLookup } from "@/lib/types";

type Tab = "catalog" | "player" | "import" | "history";

const accounts = useAccountsStore();
const skins = useSkinsStore();
const tab = ref<Tab>("catalog");

const premiumMode = ref(false);
const message = ref<string | null>(null);
const busy = ref(false);

// Catálogo
const catalogQuery = ref("");
const catalogPage = ref<SkinCatalogPage | null>(null);
const catalogPreviews = ref<Record<string, string>>({});

// Jugador
const playerQuery = ref("");
const playerLookup = ref<SkinLookup | null>(null);
const playerVariant = ref<"classic" | "slim">("classic");

// Importar
const importPath = ref<string | null>(null);
const importVariant = ref<"classic" | "slim">("classic");
const importUrl = ref("");
const importUrlVariant = ref<"classic" | "slim">("classic");

// Historial
const history = ref<SkinHistoryEntry[]>([]);

const tabs: { id: Tab; label: string; icon: string }[] = [
  { id: "catalog", label: "Catálogo", icon: "🎨" },
  { id: "player", label: "Buscar jugador", icon: "🔍" },
  { id: "import", label: "Importar", icon: "📁" },
  { id: "history", label: "Historial", icon: "📋" },
];

const playerPreviewUrl = computed(() => {
  if (!playerLookup.value?.ok) return null;
  return playerLookup.value.skinUrl ?? minotarSkin(playerLookup.value.username);
});

const playerPreviewModel = computed((): "classic" | "slim" => {
  if (!playerLookup.value?.ok) return playerVariant.value;
  const m = playerLookup.value.model || playerVariant.value;
  return m === "slim" ? "slim" : "classic";
});

const importFilePreviewUrl = computed(() => {
  if (!importPath.value || !isTauri()) return null;
  try {
    return convertFileSrc(importPath.value);
  } catch {
    return null;
  }
});

const importUrlPreviewUrl = computed(() => {
  const url = importUrl.value.trim();
  if (!url || !/^https?:\/\//i.test(url)) return null;
  return url;
});

async function refreshPremiumMode() {
  try {
    premiumMode.value = await api.canUploadPremiumSkin();
  } catch {
    premiumMode.value = false;
  }
}

async function loadCatalog(random = false) {
  busy.value = true;
  message.value = null;
  try {
    const requestedPage = catalogPage.value?.page ?? 1;
    catalogPage.value = await api.skinCatalogSearch(catalogQuery.value, requestedPage, random);
    catalogPreviews.value = {};
    for (const entry of catalogPage.value.entries) {
      if (entry.kind === "player") {
        const img = await api.skinPreviewImage(entry.id, "body", 160);
        catalogPreviews.value[entry.id] = img ?? entry.previewUrl;
      } else {
        catalogPreviews.value[entry.id] = entry.previewUrl;
      }
    }
  } catch (e) {
    message.value = String(e);
  } finally {
    busy.value = false;
  }
}

function catalogSearch() {
  catalogPage.value = {
    entries: [],
    page: 1,
    totalPages: 1,
    totalSkins: 0,
    query: catalogQuery.value,
  };
  void loadCatalog();
}

function formatSkinTotal(n: number) {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

function catalogPrev() {
  if (!catalogPage.value || catalogPage.value.page <= 1) return;
  catalogPage.value = { ...catalogPage.value, page: catalogPage.value.page - 1 };
  void loadCatalog();
}

function catalogNext() {
  if (!catalogPage.value || catalogPage.value.page >= catalogPage.value.totalPages) return;
  catalogPage.value = { ...catalogPage.value, page: catalogPage.value.page + 1 };
  void loadCatalog();
}

async function searchPlayer() {
  const q = playerQuery.value.trim();
  if (!q) return;
  busy.value = true;
  message.value = null;
  playerLookup.value = null;
  try {
    playerLookup.value = await api.lookupSkinPlayer(q);
    if (playerLookup.value?.ok) {
      playerVariant.value = playerLookup.value.model === "slim" ? "slim" : "classic";
    } else {
      message.value = playerLookup.value?.error ?? "Jugador no encontrado";
    }
  } catch (e) {
    message.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function applyPlayer() {
  if (!playerLookup.value?.ok) return;
  await runApply(() => api.applySkinFromUsername(playerLookup.value!.username, playerVariant.value));
}

async function viewCatalogEntry(entry: SkinCatalogEntry) {
  if (entry.kind === "player") {
    playerQuery.value = entry.label;
    tab.value = "player";
    await searchPlayer();
    return;
  }
  playerLookup.value = {
    ok: true,
    username: entry.label,
    uuid: "",
    skinUrl: entry.skinUrl,
    capeUrl: null,
    model: entry.model ?? "classic",
  };
  playerVariant.value = entry.model === "slim" ? "slim" : "classic";
  tab.value = "player";
}

async function applyCatalogEntry(entry: SkinCatalogEntry) {
  const variant = entry.model === "slim" ? "slim" : "classic";
  if (entry.kind === "player") {
    await runApply(() => api.applySkinFromUsername(entry.label, variant));
    return;
  }
  await runApply(() => api.applySkinFromUrl(entry.skinUrl, variant, entry.label));
}

async function pickImportFile() {
  if (!isTauri()) return;
  const path = await api.pickSkinFileForPreview();
  if (!path) return;
  importPath.value = path;
}

async function applyImportFile() {
  if (!importPath.value) return;
  await runApply(() => api.applySkinFileWithVariant(importPath.value!, importVariant.value));
}

async function applyImportUrl() {
  const url = importUrl.value.trim();
  if (!url) return;
  const name = url.split("/").pop()?.replace(/\?.*$/, "") ?? "skin";
  await runApply(() => api.applySkinFromUrl(url, importUrlVariant.value, name));
}

async function downloadCurrentSkin() {
  if (!playerLookup.value?.skinUrl) return;
  busy.value = true;
  try {
    const path = await api.downloadSkinFile(playerLookup.value.skinUrl, playerLookup.value.username);
    message.value = `Guardada en ${path}`;
    await loadHistory();
  } catch (e) {
    message.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function applyHistory(entry: SkinHistoryEntry) {
  await runApply(() => api.applySkinFromUrl(entry.url, entry.tipo, entry.nombre));
}

async function loadHistory() {
  history.value = await api.getSkinHistory();
}

async function clearHistory() {
  await api.clearSkinHistory();
  history.value = [];
}

async function runApply(fn: () => Promise<ApplySkinResult>) {
  busy.value = true;
  message.value = null;
  try {
    const result = await fn();
    message.value = result.message;
    await refreshPremiumMode();
    await loadHistory();
    await skins.refresh();
  } catch (e) {
    message.value = String(e);
  } finally {
    busy.value = false;
  }
}

watch(tab, (t) => {
  if (t === "history") void loadHistory();
  if (t === "catalog" && !catalogPage.value) void loadCatalog();
});

onMounted(async () => {
  await accounts.load();
  await refreshPremiumMode();
  await loadCatalog();
});
</script>

<template>
  <div class="mx-auto flex h-full max-w-6xl flex-col p-6">
    <header class="mb-5">
      <h1 class="text-2xl font-black">Skins</h1>
      <p class="mt-1 text-sm text-gray-400">
        Explorá, descargá e importá skins.
        <span v-if="premiumMode" class="text-pc-green">Cuenta Premium → se suben a Mojang.</span>
        <span v-else>Modo offline → resource pack local + SkinsRestorer.</span>
      </p>
    </header>

    <div class="mb-4 flex flex-wrap gap-2">
      <button
        v-for="t in tabs"
        :key="t.id"
        class="rounded-lg px-4 py-2 text-sm font-bold transition"
        :class="tab === t.id ? 'bg-pc-green text-black' : 'bg-surface-3 text-gray-300 hover:bg-surface-4'"
        @click="tab = t.id"
      >
        {{ t.icon }} {{ t.label }}
      </button>
    </div>

    <p
      v-if="message"
      class="mb-4 rounded-lg border px-4 py-2 text-sm"
      :class="message.startsWith('Skin') || message.includes('subida') || message.includes('aplicada') || message.includes('Guardada')
        ? 'border-pc-green/40 bg-pc-green/10 text-pc-green'
        : 'border-red-500/40 bg-red-500/10 text-red-300'"
    >
      {{ message }}
    </p>

    <!-- Catálogo -->
    <section v-if="tab === 'catalog'" class="flex min-h-0 flex-1 flex-col">
      <div class="mb-4 flex gap-2">
        <input
          v-model="catalogQuery"
          type="text"
          placeholder="Buscar skin por nombre de jugador..."
          class="flex-1 rounded-lg border border-surface-5 bg-surface-2 px-4 py-2.5 text-sm outline-none focus:border-pc-green"
          @keyup.enter="catalogSearch()"
        />
        <BaseButton @click="catalogSearch()">Buscar</BaseButton>
        <BaseButton variant="secondary" :disabled="busy" @click="loadCatalog(true)">🔀 Aleatorio</BaseButton>
      </div>

      <p v-if="catalogPage && !catalogPage.query" class="mb-3 text-xs text-gray-500">
        Catálogo MineSkin · {{ formatSkinTotal(catalogPage.totalSkins) }} skins disponibles
      </p>

      <div v-if="busy && !catalogPage?.entries.length" class="flex flex-1 items-center justify-center py-16">
        <div class="h-8 w-8 animate-spin rounded-full border-2 border-pc-green border-t-transparent" />
      </div>

      <div class="grid flex-1 grid-cols-2 gap-3 overflow-y-auto pr-1 md:grid-cols-3 lg:grid-cols-5">
        <div
          v-for="(entry, idx) in catalogPage?.entries ?? []"
          :key="entry.id"
          class="group relative overflow-hidden rounded-xl border border-surface-4 bg-surface-2 transition hover:border-pc-green"
        >
          <span class="absolute left-2 top-2 z-10 text-[10px] font-bold text-gray-600">#{{ idx + 1 }}</span>
          <div
            class="flex h-40 items-end justify-center"
            style="background: linear-gradient(180deg, #0d1520 0%, #111 100%)"
          >
            <img
              :src="catalogPreviews[entry.id] ?? entry.previewUrl"
              :alt="entry.label"
              class="h-[148px] object-contain object-bottom"
              loading="lazy"
            />
          </div>
          <div class="absolute inset-0 flex flex-col items-center justify-center gap-2 bg-black/80 opacity-0 transition group-hover:opacity-100">
            <span class="px-2 text-center text-sm font-black">{{ entry.label }}</span>
            <div class="flex gap-2">
              <BaseButton size="sm" @click="viewCatalogEntry(entry)">Ver 3D</BaseButton>
              <BaseButton size="sm" variant="secondary" :disabled="busy" @click="applyCatalogEntry(entry)">
                Aplicar
              </BaseButton>
            </div>
          </div>
          <div class="border-t border-surface-4 px-2 py-1.5">
            <p class="truncate text-center text-xs font-bold text-gray-300">{{ entry.label }}</p>
          </div>
        </div>
      </div>

      <div class="mt-4 flex items-center justify-between gap-2">
        <BaseButton variant="secondary" :disabled="busy || !catalogPage || catalogPage.page <= 1" @click="catalogPrev">
          ← Anterior
        </BaseButton>
        <span class="text-center text-xs text-gray-500">
          Página {{ catalogPage?.page ?? 1 }} / {{ catalogPage?.totalPages ?? 1 }}
          <template v-if="catalogPage && !catalogPage.query">
            · {{ formatSkinTotal(catalogPage.totalSkins) }} skins
          </template>
          <template v-else-if="catalogPage?.query">
            · «{{ catalogPage.query }}»
          </template>
        </span>
        <BaseButton
          variant="secondary"
          :disabled="busy || !catalogPage || catalogPage.page >= catalogPage.totalPages"
          @click="catalogNext"
        >
          Siguiente →
        </BaseButton>
      </div>
    </section>

    <!-- Buscar jugador -->
    <section v-else-if="tab === 'player'" class="flex flex-1 flex-col gap-4 lg:flex-row">
      <div class="flex flex-1 flex-col gap-4">
        <div class="flex flex-wrap gap-2">
          <input
            v-model="playerQuery"
            type="text"
            placeholder="Nombre de jugador (ej: San_Jlf)"
            class="min-w-[200px] flex-1 rounded-lg border border-surface-5 bg-surface-2 px-4 py-2.5 text-sm outline-none focus:border-pc-green"
            @keyup.enter="searchPlayer"
          />
          <select
            v-model="playerVariant"
            class="rounded-lg border border-surface-5 bg-surface-2 px-3 py-2 text-sm outline-none"
          >
            <option value="classic">Steve (classic)</option>
            <option value="slim">Alex (slim)</option>
          </select>
          <BaseButton :disabled="busy" @click="searchPlayer">Buscar</BaseButton>
          <BaseButton variant="secondary" :disabled="busy || !playerLookup?.ok" @click="applyPlayer">
            Aplicar
          </BaseButton>
        </div>

        <SkinPreview3D
          :skin-url="playerPreviewUrl"
          :model="playerPreviewModel"
          :cape-url="playerLookup?.capeUrl"
          :username="playerLookup?.username"
          :height="400"
        />
      </div>

      <aside v-if="playerLookup?.ok" class="w-full shrink-0 space-y-3 lg:w-72">
        <div class="rounded-xl border border-surface-4 bg-surface-2 p-4">
          <div class="mb-3 flex items-center gap-3">
            <SkinAvatar
              :username="playerLookup.username"
              :avatar-url="minotarHelm(playerLookup.username, 64)"
            />
            <div>
              <p class="font-bold">{{ playerLookup.username }}</p>
              <p class="text-xs text-gray-500">{{ playerLookup.model === "slim" ? "Alex (slim)" : "Steve (classic)" }}</p>
            </div>
          </div>
          <p v-if="playerLookup.skinUrl" class="mb-3 break-all font-mono text-[10px] text-pc-green">
            {{ playerLookup.skinUrl }}
          </p>
          <div class="flex flex-col gap-2">
            <BaseButton size="sm" :disabled="busy" @click="applyPlayer">✓ Aplicar skin</BaseButton>
            <BaseButton size="sm" variant="secondary" :disabled="busy || !playerLookup.skinUrl" @click="downloadCurrentSkin">
              ⬇ Descargar PNG
            </BaseButton>
          </div>
        </div>
      </aside>
    </section>

    <!-- Importar -->
    <section v-else-if="tab === 'import'" class="grid flex-1 gap-6 lg:grid-cols-2">
      <div class="rounded-xl border border-surface-4 bg-surface-2 p-5">
        <h2 class="mb-3 font-bold">Archivo local (.png)</h2>
        <p class="mb-4 text-sm text-gray-400">64×64 o 64×32. Premium → Mojang. Offline → resource pack.</p>
        <SkinPreview3D
          class="mb-4"
          :skin-url="importFilePreviewUrl"
          :model="importVariant"
          :height="280"
        />
        <p v-if="importPath" class="mb-3 truncate text-xs text-gray-500">{{ importPath }}</p>
        <select v-model="importVariant" class="mb-3 w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm">
          <option value="classic">Steve (classic)</option>
          <option value="slim">Alex (slim)</option>
        </select>
        <div class="flex gap-2">
          <BaseButton variant="secondary" @click="pickImportFile">Elegir archivo</BaseButton>
          <BaseButton :disabled="!importPath || busy" @click="applyImportFile">Aplicar</BaseButton>
        </div>
      </div>

      <div class="rounded-xl border border-surface-4 bg-surface-2 p-5">
        <h2 class="mb-3 font-bold">Desde URL</h2>
        <p class="mb-4 text-sm text-gray-400">Pegá un enlace directo a un PNG de skin (NameMC, etc.).</p>
        <SkinPreview3D
          class="mb-4"
          :skin-url="importUrlPreviewUrl"
          :model="importUrlVariant"
          :height="280"
        />
        <input
          v-model="importUrl"
          type="url"
          placeholder="https://.../skin.png"
          class="mb-3 w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
        />
        <select v-model="importUrlVariant" class="mb-4 w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm">
          <option value="classic">Steve (classic)</option>
          <option value="slim">Alex (slim)</option>
        </select>
        <BaseButton block :disabled="!importUrl.trim() || busy" @click="applyImportUrl">
          Descargar y aplicar
        </BaseButton>
      </div>
    </section>

    <!-- Historial -->
    <section v-else class="flex-1 overflow-y-auto">
      <div class="mb-4 flex justify-between">
        <h2 class="font-bold">Skins recientes</h2>
        <BaseButton v-if="history.length" size="sm" variant="ghost" @click="clearHistory">Limpiar</BaseButton>
      </div>
      <div v-if="!history.length" class="text-sm text-gray-500">Aún no aplicaste skins desde el launcher.</div>
      <div class="grid grid-cols-3 gap-3 md:grid-cols-5 lg:grid-cols-6">
        <button
          v-for="(entry, i) in history"
          :key="entry.url + i"
          class="rounded-xl border border-surface-4 bg-surface-2 p-2 text-left transition hover:border-pc-green"
          @click="applyHistory(entry)"
        >
          <img :src="minotarBody(entry.nombre, 80)" :alt="entry.nombre" class="mx-auto mb-2 h-20 object-contain" />
          <p class="truncate text-center text-xs font-bold">{{ entry.nombre }}</p>
          <p class="text-center text-[10px] text-gray-500">{{ entry.tipo }}</p>
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.image-rendering-pixelated {
  image-rendering: pixelated;
}
</style>
