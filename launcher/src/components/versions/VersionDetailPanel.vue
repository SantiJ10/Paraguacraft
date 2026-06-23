<script setup lang="ts">
import { computed, ref } from "vue";
import BaseButton from "@/components/common/BaseButton.vue";
import { contentFolderIcon } from "@/lib/contentIcons";
import { loaderDisplayName, loaderIconSrc } from "@/lib/loaderIcons";
import type { BedrockStatus, Instance, InstanceContentItem, LoaderInfo } from "@/lib/types";
import type { VersionCardModel } from "@/lib/versionCatalog";
import { versionCardImageUrl } from "@/lib/versionCatalog";

const props = defineProps<{
  card: VersionCardModel | null;
  mcVersion: string;
  loaders: LoaderInfo[];
  loadingLoaders: boolean;
  selectedLoaderId: string;
  selectedLoaderVersion: string;
  linkedInstance: Instance | null;
  content: InstanceContentItem[];
  bedrockStatus: BedrockStatus | null;
  bedrockMessage: string;
  premiumLocked: boolean;
  canLaunchBedrock: boolean;
  busy: boolean;
  error: string | null;
  message: string | null;
}>();

const emit = defineEmits<{
  "update:selectedLoaderId": [v: string];
  "update:selectedLoaderVersion": [v: string];
  "update:mcVersion": [v: string];
  install: [];
  play: [];
  launchBedrock: [];
  openSettings: [];
  goAccounts: [];
}>();

const imgFailed = ref(false);
const previewUrl = computed(() =>
  props.card ? versionCardImageUrl(props.card.imageKey) : "",
);

const selectedLoader = computed(() =>
  props.loaders.find((l) => l.id === props.selectedLoaderId) ?? null,
);

const complementos = computed(() => {
  const folders = ["mods", "resourcepacks", "shaderpacks", "datapacks"] as const;
  return folders
    .map((folder) => {
      const items = props.content.filter((c) => c.folder === folder && c.enabled);
      if (!items.length) return null;
      return { folder, item: items[0]!, count: items.length };
    })
    .filter(Boolean) as Array<{ folder: string; item: InstanceContentItem; count: number }>;
});

const isBedrock = computed(() => props.card?.kind === "bedrock");
const isInstalledCard = computed(() => props.card?.kind === "installed");

const versionOptions = computed(() => props.card?.subs ?? []);

const panelTitle = computed(() => {
  if (!props.card) return "Selecciona una versión";
  if (isBedrock.value) return "Paraguacraft Bedrock";
  if (isInstalledCard.value && props.linkedInstance) return props.linkedInstance.name;
  return `Minecraft ${props.mcVersion}`;
});
</script>

<template>
  <aside class="flex w-[340px] shrink-0 flex-col border-l border-surface-3 bg-surface-0">
    <div class="border-b border-surface-3 px-5 py-4">
      <p class="text-[10px] font-black uppercase tracking-widest text-gray-500">Versión seleccionada</p>
    </div>

    <div v-if="!card" class="flex flex-1 items-center justify-center p-6 text-center text-sm text-gray-500">
      Elegí una tarjeta PARAGUA para ver detalles y jugar.
    </div>

    <template v-else>
      <div class="relative mx-5 mt-4 h-36 overflow-hidden rounded-xl">
        <img
          v-if="!imgFailed"
          :src="previewUrl"
          alt=""
          class="h-full w-full object-cover"
          @error="imgFailed = true"
        />
        <div v-else class="h-full w-full bg-gradient-to-br from-surface-3 to-surface-1" />
        <div class="absolute inset-0 bg-gradient-to-t from-black/80 to-transparent" />
      </div>

      <div class="flex-1 overflow-y-auto px-5 py-4">
        <h2 class="text-lg font-bold">{{ panelTitle }}</h2>
        <p class="mt-2 text-sm leading-relaxed text-gray-400">{{ card.description }}</p>

        <!-- Bedrock -->
        <template v-if="isBedrock">
          <div v-if="premiumLocked" class="mt-4 rounded-lg border border-[#F39C12]/30 bg-[#1A1A1A] p-3">
            <p class="text-sm font-semibold text-[#F39C12]">Cuenta Premium requerida</p>
            <p class="mt-1 text-xs text-gray-500">Iniciá sesión con Microsoft para jugar Bedrock.</p>
            <BaseButton size="sm" variant="secondary" class="mt-3 w-full" @click="emit('goAccounts')">
              Agregar cuenta Microsoft
            </BaseButton>
          </div>
          <p v-else-if="bedrockStatus && !bedrockStatus.installed" class="mt-4 text-sm text-gray-400">
            No se detectó Bedrock. Instalalo desde Xbox / Microsoft Store.
          </p>
        </template>

        <!-- Java version cards -->
        <template v-else>
          <div v-if="versionOptions.length > 1" class="mt-4">
            <label class="mb-1 block text-xs font-semibold uppercase tracking-wider text-gray-500">Versión</label>
            <select
              :value="mcVersion"
              class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
              @change="emit('update:mcVersion', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="v in versionOptions" :key="v" :value="v">{{ v }}</option>
            </select>
          </div>

          <div v-if="isInstalledCard && linkedInstance" class="mt-4 rounded-lg border border-surface-4 bg-surface-2 p-3">
            <p class="text-xs text-gray-500">Instancia vinculada</p>
            <p class="font-semibold">{{ linkedInstance.name }}</p>
            <p class="text-xs text-gray-400">
              {{ linkedInstance.mcVersion }} · {{ loaderDisplayName(linkedInstance.loader) }}
            </p>
          </div>

          <div v-if="!isInstalledCard" class="mt-4">
            <label class="mb-2 block text-xs font-semibold uppercase tracking-wider text-gray-500">Loader</label>
            <p v-if="loadingLoaders" class="text-sm text-gray-500">Consultando loaders…</p>
            <div v-else class="space-y-2">
              <button
                v-for="l in loaders"
                :key="l.id"
                type="button"
                class="flex w-full items-center gap-3 rounded-xl border px-3 py-2.5 text-left transition"
                :class="selectedLoaderId === l.id ? 'border-pc-green bg-pc-green/10' : 'border-surface-4 hover:border-surface-5'"
                @click="emit('update:selectedLoaderId', l.id)"
              >
                <img :src="loaderIconSrc(l.id)" :alt="l.name" class="h-8 w-8 object-contain" />
                <div class="min-w-0">
                  <p class="font-semibold">{{ l.name }}</p>
                  <p class="truncate text-xs text-gray-500">{{ l.description }}</p>
                </div>
              </button>
            </div>
            <select
              v-if="selectedLoader && selectedLoader.versions.length"
              :value="selectedLoaderVersion"
              class="mt-2 w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm"
              @change="emit('update:selectedLoaderVersion', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="ver in selectedLoader.versions" :key="ver" :value="ver">{{ ver }}</option>
            </select>
          </div>

          <div v-if="complementos.length" class="mt-5">
            <p class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500">Complementos</p>
            <div class="flex flex-wrap gap-2">
              <div
                v-for="c in complementos"
                :key="c.folder"
                class="relative flex h-11 w-11 items-center justify-center rounded-lg border border-surface-4 bg-surface-2"
                :title="c.item.name"
              >
                <img :src="contentFolderIcon(c.folder)" alt="" class="h-7 w-7 object-contain" />
                <span
                  v-if="c.count > 1"
                  class="absolute -bottom-1 -right-1 rounded bg-pc-green px-1 text-[9px] font-black text-black"
                >{{ c.count }}</span>
              </div>
            </div>
          </div>
        </template>

        <p v-if="message" class="mt-4 text-sm text-pc-green">{{ message }}</p>
        <p v-if="error" class="mt-4 text-sm text-red-400">{{ error }}</p>
        <p v-if="bedrockMessage" class="mt-4 text-center text-xs font-semibold text-[#3498DB]">{{ bedrockMessage }}</p>
      </div>

      <div class="border-t border-surface-3 p-4">
        <div v-if="isBedrock" class="flex gap-2">
          <BaseButton
            class="flex-1 !bg-[#0078D4] hover:!bg-[#106EBE]"
            size="lg"
            :disabled="busy || !canLaunchBedrock"
            @click="emit('launchBedrock')"
          >
            {{ busy ? "Abriendo…" : "Abrir Bedrock" }}
          </BaseButton>
        </div>
        <div v-else class="flex gap-2">
          <button
            type="button"
            class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-surface-4 bg-surface-2 text-gray-300 transition hover:border-surface-5 hover:text-white disabled:opacity-40"
            :disabled="!linkedInstance"
            title="Ajustes de la instancia"
            @click="emit('openSettings')"
          >
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 15a3 3 0 100-6 3 3 0 000 6z" />
              <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 01-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z" />
            </svg>
          </button>
          <BaseButton
            v-if="linkedInstance"
            class="flex-1"
            size="lg"
            :disabled="busy"
            @click="emit('play')"
          >
            {{ busy ? "Lanzando…" : "Iniciar el juego" }}
          </BaseButton>
          <BaseButton
            v-else
            class="flex-1"
            size="lg"
            :disabled="busy"
            @click="emit('install')"
          >
            {{ busy ? "Instalando…" : "Instalar y jugar" }}
          </BaseButton>
        </div>
      </div>
    </template>
  </aside>
</template>
