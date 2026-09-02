<script setup lang="ts">
defineOptions({ name: "skins" });
import { computed, onMounted, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { api, isTauri, openUrl } from "@/lib/ipc";
import { minotarBody, minotarSkin } from "@/lib/skins";
import BaseButton from "@/components/common/BaseButton.vue";
import SkinPreview3D from "@/components/skins/SkinPreview3D.vue";
import { useAccountsStore } from "@/stores/accounts";
import { useSkinsStore } from "@/stores/skins";
import type { ApplySkinResult, SkinCatalogEntry, SkinCatalogPage, SkinHistoryEntry, SkinLookup } from "@/lib/types";

type Tab = "library" | "store" | "import";

const ELY_BY_URL = "https://ely.by";
const OFFLINE_GUIDE_KEY = "paraguacraft.skins.offlineGuideOpen";

const STEVE_SKIN = "https://minotar.net/skin/Steve";
const ALEX_SKIN = "https://minotar.net/skin/Alex";

const RECOMMENDED: Array<{ id: string; label: string; skinUrl: string; model: "classic" | "slim" }> = [
  { id: "rec-jeb", label: "jeb_", skinUrl: "https://minotar.net/skin/jeb_", model: "classic" },
  { id: "rec-notch", label: "Notch", skinUrl: "https://minotar.net/skin/Notch", model: "classic" },
  { id: "rec-dinnerbone", label: "Dinnerbone", skinUrl: "https://minotar.net/skin/Dinnerbone", model: "classic" },
  { id: "rec-dream", label: "Dream", skinUrl: "https://minotar.net/skin/Dream", model: "classic" },
  { id: "rec-technoblade", label: "Technoblade", skinUrl: "https://minotar.net/skin/Technoblade", model: "classic" },
  { id: "rec-capy", label: "Capybara", skinUrl: "https://minotar.net/skin/Capybara", model: "slim" },
];

const accounts = useAccountsStore();
const skins = useSkinsStore();
const tab = ref<Tab>("library");

const premiumMode = ref(false);
const message = ref<string | null>(null);
const busy = ref(false);
const showCape = ref(true);

/** Mini guía solo cuenta offline / no-premium. */
const isOfflineAccount = computed(() => {
  const a = accounts.active;
  if (!a) return false;
  if (a.type === "offline") return true;
  return a.premium === false;
});

const offlineGuideOpen = ref(true);
try {
  const saved = localStorage.getItem(OFFLINE_GUIDE_KEY);
  if (saved === "0") offlineGuideOpen.value = false;
  if (saved === "1") offlineGuideOpen.value = true;
} catch {
  /* ignore */
}

function toggleOfflineGuide() {
  offlineGuideOpen.value = !offlineGuideOpen.value;
  try {
    localStorage.setItem(OFFLINE_GUIDE_KEY, offlineGuideOpen.value ? "1" : "0");
  } catch {
    /* ignore */
  }
}

async function openElyBy() {
  await openUrl(ELY_BY_URL);
}

const catalogQuery = ref("");
const catalogPage = ref<SkinCatalogPage | null>(null);
const catalogPreviews = ref<Record<string, string>>({});

const playerQuery = ref("");
const playerLookup = ref<SkinLookup | null>(null);
const playerVariant = ref<"classic" | "slim">("classic");

const importPath = ref<string | null>(null);
const importVariant = ref<"classic" | "slim">("classic");
const importUrl = ref("");
const importUrlVariant = ref<"classic" | "slim">("classic");

const history = ref<SkinHistoryEntry[]>([]);

const selectedPreview = ref<{
  skinUrl: string | null;
  model: "classic" | "slim";
  capeUrl: string | null;
  username: string | null;
}>({
  skinUrl: null,
  model: "classic",
  capeUrl: null,
  username: null,
});

const tabs: { id: Tab; label: string }[] = [
  { id: "library", label: "Biblioteca" },
  { id: "store", label: "Tienda" },
  { id: "import", label: "Importar" },
];

const previewCape = computed(() => (showCape.value ? selectedPreview.value.capeUrl : null));

const accountSkinUrl = computed(() => {
  const a = skins.activeSkin;
  return a?.skinUrl ?? (accounts.active ? minotarSkin(accounts.active.username) : null);
});

function setPreview(opts: {
  skinUrl: string | null;
  model?: "classic" | "slim";
  capeUrl?: string | null;
  username?: string | null;
}) {
  selectedPreview.value = {
    skinUrl: opts.skinUrl,
    model: opts.model ?? "classic",
    capeUrl: opts.capeUrl ?? null,
    username: opts.username ?? null,
  };
}

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
      setPreview({
        skinUrl: playerLookup.value.skinUrl ?? minotarSkin(playerLookup.value.username),
        model: playerVariant.value,
        capeUrl: playerLookup.value.capeUrl,
        username: playerLookup.value.username,
      });
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
    await searchPlayer();
    return;
  }
  setPreview({
    skinUrl: entry.skinUrl,
    model: entry.model === "slim" ? "slim" : "classic",
    username: entry.label,
  });
  playerLookup.value = {
    ok: true,
    username: entry.label,
    uuid: "",
    skinUrl: entry.skinUrl,
    capeUrl: null,
    model: entry.model ?? "classic",
  };
  playerVariant.value = entry.model === "slim" ? "slim" : "classic";
}

async function applyCatalogEntry(entry: SkinCatalogEntry) {
  const variant = entry.model === "slim" ? "slim" : "classic";
  if (entry.kind === "player") {
    await runApply(() => api.applySkinFromUsername(entry.label, variant));
    return;
  }
  await runApply(() => api.applySkinFromUrl(entry.skinUrl, variant, entry.label));
}

async function applyDefault(kind: "steve" | "alex") {
  const url = kind === "steve" ? STEVE_SKIN : ALEX_SKIN;
  const model = kind === "steve" ? "classic" : "slim";
  setPreview({ skinUrl: url, model, username: kind === "steve" ? "Steve" : "Alex" });
  await runApply(() => api.applySkinFromUrl(url, model, kind === "steve" ? "Steve" : "Alex"));
}

async function applyRecommended(rec: (typeof RECOMMENDED)[number]) {
  setPreview({ skinUrl: rec.skinUrl, model: rec.model, username: rec.label });
  await runApply(() => api.applySkinFromUrl(rec.skinUrl, rec.model, rec.label));
}

async function pickImportFile() {
  if (!isTauri()) return;
  const path = await api.pickSkinFileForPreview();
  if (!path) return;
  importPath.value = path;
  try {
    setPreview({ skinUrl: convertFileSrc(path), model: importVariant.value, username: "import" });
  } catch {
    /* ignore */
  }
}

async function applyImportFile() {
  if (!importPath.value) return;
  await runApply(() => api.applySkinFileWithVariant(importPath.value!, importVariant.value));
}

async function applyImportUrl() {
  const url = importUrl.value.trim();
  if (!url) return;
  const name = url.split("/").pop()?.replace(/\?.*$/, "") ?? "skin";
  setPreview({ skinUrl: url, model: importUrlVariant.value, username: name });
  await runApply(() => api.applySkinFromUrl(url, importUrlVariant.value, name));
}

async function applyHistory(entry: SkinHistoryEntry) {
  setPreview({
    skinUrl: entry.url,
    model: entry.tipo === "slim" ? "slim" : "classic",
    username: entry.nombre,
  });
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
    await skins.refresh(true);
    if (skins.activeSkin?.skinUrl) {
      setPreview({
        skinUrl: skins.activeSkin.skinUrl,
        model: skins.activeSkin.model === "slim" ? "slim" : "classic",
        capeUrl: null,
        username: skins.activeSkin.username ?? accounts.active?.username ?? null,
      });
    }
  } catch (e) {
    message.value = String(e);
  } finally {
    busy.value = false;
  }
}

watch(tab, (t) => {
  if (t === "store" && !catalogPage.value) void loadCatalog();
  if (t === "library") void loadHistory();
});

watch(importVariant, (m) => {
  if (importPath.value && isTauri()) {
    try {
      setPreview({ skinUrl: convertFileSrc(importPath.value), model: m, username: "import" });
    } catch {
      /* ignore */
    }
  }
});

onMounted(async () => {
  await accounts.load();
  await skins.refresh();
  await refreshPremiumMode();
  await loadHistory();
  setPreview({
    skinUrl: accountSkinUrl.value,
    model: skins.activeSkin?.model === "slim" ? "slim" : "classic",
    capeUrl: null,
    username: accounts.active?.username ?? skins.activeSkin?.username ?? null,
  });
});
</script>

<template>
  <div class="flex h-full min-h-0 flex-col p-6">
    <header class="mb-4 flex flex-wrap items-end justify-between gap-3">
      <div>
        <h1 class="text-2xl font-black">Selector de skins</h1>
        <p class="mt-1 text-sm text-gray-400">
          <span v-if="premiumMode" class="text-pc-green">Premium → Mojang</span>
          <span v-else-if="isOfflineAccount">Offline → pack local + Ely.by / CustomSkinLoader</span>
          <span v-else>Offline → resource pack local</span>
          · Jugando como {{ accounts.active?.username ?? "—" }}
        </p>
      </div>
      <div class="flex gap-2">
        <button
          v-for="t in tabs"
          :key="t.id"
          type="button"
          class="rounded-full px-4 py-1.5 text-sm font-bold transition"
          :class="tab === t.id ? 'bg-pc-green text-black' : 'bg-surface-3 text-gray-300 hover:bg-surface-4'"
          @click="tab = t.id"
        >
          {{ t.label }}
        </button>
      </div>
    </header>

    <!-- Guía skins multiplayer (solo no-premium) -->
    <section
      v-if="isOfflineAccount"
      class="mb-4 overflow-hidden rounded-2xl border border-sky-500/30 bg-gradient-to-br from-sky-500/10 via-surface-2 to-surface-2"
    >
      <button
        type="button"
        class="flex w-full items-center justify-between gap-3 px-4 py-3 text-left transition hover:bg-white/5"
        @click="toggleOfflineGuide"
      >
        <div class="min-w-0">
          <p class="text-sm font-bold text-sky-200">Skin en servidores · cuenta offline</p>
          <p class="mt-0.5 truncate text-xs text-gray-400">
            Cómo verse bien vos y que otros con la config correcta también te vean
          </p>
        </div>
        <span class="shrink-0 text-xs font-bold text-sky-300">{{ offlineGuideOpen ? "Ocultar" : "Ver guía" }}</span>
      </button>
      <div v-if="offlineGuideOpen" class="border-t border-sky-500/20 px-4 py-4 text-sm text-gray-300">
        <ol class="list-decimal space-y-3 pl-5">
          <li>
            <span class="font-semibold text-white">Elegí y aplicá tu skin acá</span>
            — biblioteca, tienda o Importar PNG. Se guarda para tu cuenta
            <span class="font-mono text-pc-green">{{ accounts.active?.username }}</span>.
          </li>
          <li>
            <span class="font-semibold text-white">Jugá con esta misma cuenta offline</span>
            (el nick del launcher debe coincidir con el de Ely.by). Al lanzar un loader con mods (PvP, Optimized, Fabric, Forge),
            Paraguacraft instala <span class="text-white">CustomSkinLoader</span> solo.
          </li>
          <li>
            <span class="font-semibold text-white">Multiplayer: registrá el mismo nick en Ely.by</span>
            y subí la misma skin en
            <button type="button" class="font-bold text-sky-300 underline decoration-sky-500/50 hover:text-sky-200" @click="openElyBy">
              ely.by
            </button>.
            Si tu nick del launcher es distinto, andá a
            <RouterLink to="/settings#accounts" class="font-bold text-sky-300 underline decoration-sky-500/50 hover:text-sky-200">
              Ajustes → Cuentas → Renombrar
            </RouterLink>
            y poné el mismo nombre que en Ely.by. Así otros clientes con CustomSkinLoader pueden cargarte.
          </li>
          <li>
            <span class="font-semibold text-white">Quién te ve</span>
            —
            <span class="text-white">vos</span> (LocalSkin / pack local del launcher, solo tu nick);
            <span class="text-white">otros con CSL / Ely.by</span> (TLauncher y launchers parecidos);
            en servers de Paraguacraft, <span class="text-white">SkinsRestorer pide a Ely.by</span>
            y vanilla / PvP también te ven. Nunca se reutiliza tu historial de skins en otros jugadores.
          </li>
        </ol>
        <div class="mt-4 flex flex-wrap gap-2">
          <BaseButton size="sm" variant="secondary" @click="openElyBy">Abrir Ely.by</BaseButton>
          <BaseButton size="sm" variant="ghost" @click="tab = 'import'">Importar PNG</BaseButton>
        </div>
        <p class="mt-3 text-xs leading-relaxed text-gray-500">
          Tip: el nick del launcher y el de Ely.by deben ser idénticos. PNG 64×64 (clásico o slim).
          Primera partida con internet para bajar CustomSkinLoader.
        </p>
      </div>
    </section>

    <p
      v-if="message"
      class="mb-3 rounded-lg border px-4 py-2 text-sm"
      :class="
        message.startsWith('Skin') || message.includes('subida') || message.includes('aplicada') || message.includes('Guardada')
          ? 'border-pc-green/40 bg-pc-green/10 text-pc-green'
          : 'border-red-500/40 bg-red-500/10 text-red-300'
      "
    >
      {{ message }}
    </p>

    <div class="grid min-h-0 flex-1 gap-5 lg:grid-cols-[320px_1fr]">
      <!-- Preview 3D -->
      <aside class="flex min-h-0 flex-col gap-3">
        <div class="rounded-2xl border border-surface-4 bg-surface-2 p-3">
          <p class="mb-2 text-center text-sm font-bold">
            {{ selectedPreview.username || accounts.active?.username || "Tu skin" }}
          </p>
          <SkinPreview3D
            :skin-url="selectedPreview.skinUrl"
            :model="selectedPreview.model"
            :cape-url="previewCape"
            :username="selectedPreview.username"
            :height="420"
          />
          <div class="mt-3 flex items-center justify-between gap-2">
            <label class="flex items-center gap-2 text-xs text-gray-400">
              <input v-model="showCape" type="checkbox" class="accent-pc-green" />
              Mostrar capa
            </label>
            <BaseButton
              size="sm"
              :disabled="busy || !selectedPreview.skinUrl"
              @click="
                runApply(() =>
                  api.applySkinFromUrl(
                    selectedPreview.skinUrl!,
                    selectedPreview.model,
                    selectedPreview.username ?? 'skin',
                  ),
                )
              "
            >
              Aplicar
            </BaseButton>
          </div>
        </div>
      </aside>

      <!-- Biblioteca / Tienda / Import -->
      <section class="min-h-0 overflow-y-auto pr-1">
        <template v-if="tab === 'library'">
          <div class="mb-6">
            <h2 class="mb-3 text-sm font-bold uppercase tracking-wider text-gray-400">Skins guardadas</h2>
            <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-4">
              <button
                type="button"
                class="flex aspect-[3/4] flex-col items-center justify-center rounded-2xl border border-dashed border-surface-5 bg-surface-2/60 text-gray-400 transition hover:border-pc-green hover:text-pc-green"
                @click="tab = 'import'"
              >
                <span class="text-3xl font-light">+</span>
                <span class="mt-1 text-xs font-bold">Añadir skin</span>
                <span class="text-[10px]">Importar PNG</span>
              </button>
              <button
                v-for="(entry, i) in history"
                :key="entry.url + i"
                type="button"
                class="overflow-hidden rounded-2xl border bg-surface-2 text-left transition hover:border-pc-green"
                :class="
                  selectedPreview.skinUrl === entry.url
                    ? 'border-pc-green bg-pc-green/10'
                    : 'border-surface-4'
                "
                @click="applyHistory(entry)"
              >
                <div class="flex h-36 items-end justify-center bg-gradient-to-b from-surface-3 to-surface-1">
                  <img :src="minotarBody(entry.nombre, 100)" :alt="entry.nombre" class="h-[130px] object-contain" />
                </div>
                <p class="truncate px-2 py-1.5 text-center text-xs font-bold">{{ entry.nombre }}</p>
              </button>
            </div>
            <div v-if="history.length" class="mt-2 flex justify-end">
              <BaseButton size="sm" variant="ghost" @click="clearHistory">Limpiar historial</BaseButton>
            </div>
          </div>

          <div class="mb-6">
            <h2 class="mb-3 text-sm font-bold uppercase tracking-wider text-gray-400">Steve y Alex</h2>
            <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
              <button
                type="button"
                class="overflow-hidden rounded-2xl border border-surface-4 bg-surface-2 transition hover:border-pc-green"
                @click="applyDefault('steve')"
              >
                <div class="flex h-36 items-end justify-center bg-surface-1">
                  <img :src="minotarBody('Steve', 100)" alt="Steve" class="h-[130px] object-contain" />
                </div>
                <p class="py-1.5 text-center text-xs font-bold">Steve</p>
              </button>
              <button
                type="button"
                class="overflow-hidden rounded-2xl border border-surface-4 bg-surface-2 transition hover:border-pc-green"
                @click="applyDefault('alex')"
              >
                <div class="flex h-36 items-end justify-center bg-surface-1">
                  <img :src="minotarBody('Alex', 100)" alt="Alex" class="h-[130px] object-contain" />
                </div>
                <p class="py-1.5 text-center text-xs font-bold">Alex</p>
              </button>
            </div>
          </div>

          <div>
            <h2 class="mb-3 text-sm font-bold uppercase tracking-wider text-gray-400">Recomendadas</h2>
            <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-4">
              <button
                v-for="rec in RECOMMENDED"
                :key="rec.id"
                type="button"
                class="overflow-hidden rounded-2xl border border-surface-4 bg-surface-2 transition hover:border-pc-green"
                @click="applyRecommended(rec)"
              >
                <div class="flex h-36 items-end justify-center bg-surface-1">
                  <img :src="minotarBody(rec.label, 100)" :alt="rec.label" class="h-[130px] object-contain" />
                </div>
                <p class="truncate px-2 py-1.5 text-center text-xs font-bold">{{ rec.label }}</p>
              </button>
            </div>
          </div>
        </template>

        <template v-else-if="tab === 'store'">
          <div class="mb-4 flex flex-wrap gap-2">
            <input
              v-model="catalogQuery"
              type="text"
              placeholder="Buscar skins o jugadores…"
              class="min-w-[200px] flex-1 rounded-xl border border-surface-5 bg-surface-2 px-4 py-2.5 text-sm outline-none focus:border-pc-green"
              @keyup.enter="catalogSearch()"
            />
            <BaseButton @click="catalogSearch()">Buscar</BaseButton>
            <BaseButton variant="secondary" :disabled="busy" @click="loadCatalog(true)">Aleatorio</BaseButton>
          </div>

          <div class="mb-4 flex flex-wrap gap-2">
            <input
              v-model="playerQuery"
              type="text"
              placeholder="Buscar jugador exacto…"
              class="min-w-[180px] flex-1 rounded-xl border border-surface-5 bg-surface-2 px-4 py-2.5 text-sm outline-none focus:border-pc-green"
              @keyup.enter="searchPlayer"
            />
            <BaseButton variant="secondary" :disabled="busy" @click="searchPlayer">Ver jugador</BaseButton>
            <BaseButton :disabled="busy || !playerLookup?.ok" @click="applyPlayer">Aplicar jugador</BaseButton>
          </div>

          <p v-if="catalogPage && !catalogPage.query" class="mb-3 text-xs text-gray-500">
            MineSkin · {{ formatSkinTotal(catalogPage.totalSkins) }} skins
          </p>

          <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-4">
            <div
              v-for="entry in catalogPage?.entries ?? []"
              :key="entry.id"
              class="group relative overflow-hidden rounded-2xl border border-surface-4 bg-surface-2"
            >
              <div class="flex h-40 items-end justify-center bg-surface-1">
                <img
                  :src="catalogPreviews[entry.id] ?? entry.previewUrl"
                  :alt="entry.label"
                  class="h-[148px] object-contain"
                  loading="lazy"
                />
              </div>
              <div
                class="absolute inset-0 flex flex-col items-center justify-center gap-2 bg-black/80 opacity-0 transition group-hover:opacity-100"
              >
                <span class="px-2 text-center text-sm font-black">{{ entry.label }}</span>
                <div class="flex gap-2">
                  <BaseButton size="sm" @click="viewCatalogEntry(entry)">Ver 3D</BaseButton>
                  <BaseButton size="sm" variant="secondary" :disabled="busy" @click="applyCatalogEntry(entry)">
                    Aplicar
                  </BaseButton>
                </div>
              </div>
              <p class="truncate border-t border-surface-4 px-2 py-1.5 text-center text-xs font-bold">
                {{ entry.label }}
              </p>
            </div>
          </div>

          <div class="mt-4 flex items-center justify-between gap-2">
            <BaseButton variant="secondary" :disabled="busy || !catalogPage || catalogPage.page <= 1" @click="catalogPrev">
              ← Anterior
            </BaseButton>
            <span class="text-xs text-gray-500">
              Página {{ catalogPage?.page ?? 1 }} / {{ catalogPage?.totalPages ?? 1 }}
            </span>
            <BaseButton
              variant="secondary"
              :disabled="busy || !catalogPage || catalogPage.page >= catalogPage.totalPages"
              @click="catalogNext"
            >
              Siguiente →
            </BaseButton>
          </div>
        </template>

        <template v-else>
          <div class="grid gap-4 lg:grid-cols-2">
            <div class="rounded-2xl border border-surface-4 bg-surface-2 p-5">
              <h2 class="mb-2 font-bold">Archivo local (.png)</h2>
              <p class="mb-3 text-sm text-gray-400">64×64 o 64×32. Arrastrá o elegí un archivo.</p>
              <p v-if="importPath" class="mb-2 truncate text-xs text-gray-500">{{ importPath }}</p>
              <div class="mb-3 flex gap-2">
                <button
                  type="button"
                  class="rounded-full px-3 py-1 text-xs font-bold"
                  :class="importVariant === 'classic' ? 'bg-pc-green text-black' : 'bg-surface-3 text-gray-300'"
                  @click="importVariant = 'classic'"
                >
                  Ancho
                </button>
                <button
                  type="button"
                  class="rounded-full px-3 py-1 text-xs font-bold"
                  :class="importVariant === 'slim' ? 'bg-pc-green text-black' : 'bg-surface-3 text-gray-300'"
                  @click="importVariant = 'slim'"
                >
                  Delgado
                </button>
              </div>
              <div class="flex gap-2">
                <BaseButton variant="secondary" @click="pickImportFile">Elegir archivo</BaseButton>
                <BaseButton :disabled="!importPath || busy" @click="applyImportFile">Guardar skin</BaseButton>
              </div>
            </div>

            <div class="rounded-2xl border border-surface-4 bg-surface-2 p-5">
              <h2 class="mb-2 font-bold">Desde URL</h2>
              <p class="mb-3 text-sm text-gray-400">Pegá un enlace directo a un PNG.</p>
              <input
                v-model="importUrl"
                type="url"
                placeholder="https://.../skin.png"
                class="mb-3 w-full rounded-xl border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
                @change="
                  importUrl.trim() &&
                    setPreview({
                      skinUrl: importUrl.trim(),
                      model: importUrlVariant,
                      username: 'url',
                    })
                "
              />
              <div class="mb-3 flex gap-2">
                <button
                  type="button"
                  class="rounded-full px-3 py-1 text-xs font-bold"
                  :class="importUrlVariant === 'classic' ? 'bg-pc-green text-black' : 'bg-surface-3 text-gray-300'"
                  @click="importUrlVariant = 'classic'"
                >
                  Ancho
                </button>
                <button
                  type="button"
                  class="rounded-full px-3 py-1 text-xs font-bold"
                  :class="importUrlVariant === 'slim' ? 'bg-pc-green text-black' : 'bg-surface-3 text-gray-300'"
                  @click="importUrlVariant = 'slim'"
                >
                  Delgado
                </button>
              </div>
              <BaseButton block :disabled="!importUrl.trim() || busy" @click="applyImportUrl">
                Descargar y aplicar
              </BaseButton>
            </div>
          </div>
        </template>
      </section>
    </div>
  </div>
</template>
