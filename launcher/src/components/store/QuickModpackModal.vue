<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { api, openUrl, parseInvokeError } from "@/lib/ipc";
import type { StoreItem, StoreVersion } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";
import { useInstancesStore } from "@/stores/instances";
import { useDownloadsStore } from "@/stores/downloads";
import { useSettingsStore } from "@/stores/settings";
import { useAppStore } from "@/stores/app";

const props = defineProps<{ item: StoreItem | null }>();
const open = defineModel<boolean>("open", { default: false });
const emit = defineEmits<{
  (e: "installed"): void;
  (e: "advanced"): void;
}>();

const instances = useInstancesStore();
const downloads = useDownloadsStore();
const settings = useSettingsStore();
const app = useAppStore();

const versions = ref<StoreVersion[]>([]);
const versionId = ref("");
const instanceName = ref("");
const ramMb = ref(4096);
const loading = ref(false);
const busy = ref(false);
const error = ref<string | null>(null);
const distributionUrl = ref<string | null>(null);
const installedId = ref<string | null>(null);
const installedName = ref<string | null>(null);

const selected = computed(() => versions.value.find((v) => v.id === versionId.value) ?? null);
const maxRam = computed(() => {
  const gb = app.hardware?.ramGb ?? 16;
  return Math.max(4096, Math.floor(gb * 1024 * 0.7));
});

const RETRY_IDS = [
  "mrpack-import",
  "cfpack-0",
  "cfpack-1",
  "cfpack-2",
  "cfpack-final",
  "ensure-libraries",
  "install-loader",
];

watch(
  () => [open.value, props.item?.id],
  async () => {
    if (!open.value || !props.item) return;
    instanceName.value = props.item.title;
    ramMb.value = settings.settings?.ramMb || app.hardware?.recommendedRamMb || 4096;
    versions.value = [];
    versionId.value = "";
    error.value = null;
    distributionUrl.value = null;
    installedId.value = null;
    installedName.value = null;
    loading.value = true;
    try {
      versions.value = await api.listStoreProjectVersions({
        provider: props.item.provider,
        projectId: props.item.id,
        projectType: props.item.projectType,
      });
      versionId.value = versions.value[0]?.id ?? "";
    } catch (e) {
      const parsed = parseInvokeError(e);
      error.value = parsed.message;
      if (parsed.code === "cf_distribution_blocked") {
        distributionUrl.value = parsed.projectUrl ?? props.item.projectUrl ?? null;
      }
    } finally {
      loading.value = false;
    }
  },
);

function formatVersion(v: StoreVersion): string {
  const mc = v.gameVersions[0] ? ` · MC ${v.gameVersions[0]}` : "";
  const loader = v.loaders[0] ? ` · ${v.loaders[0]}` : "";
  return `${v.versionNumber || v.name}${mc}${loader}`;
}

async function install() {
  if (!props.item || !versionId.value) return;
  busy.value = true;
  error.value = null;
  distributionUrl.value = null;
  await downloads.initEvents();
  downloads.bindRetry(RETRY_IDS, () => install());
  try {
    let inst;
    if (props.item.provider === "curseforge") {
      inst = await api.importCfpackVersion(props.item.id, versionId.value);
    } else {
      inst = await api.importMrpackVersion(versionId.value);
    }
    const name = instanceName.value.trim() || inst.name;
    if (name && name !== inst.name) {
      try {
        await instances.rename(inst.id, name, inst.icon);
      } catch {
        /* el pack ya está; el nombre se puede cambiar después */
      }
    }
    if (ramMb.value && ramMb.value !== inst.ramMb) {
      try {
        await instances.setRam(inst.id, ramMb.value);
      } catch {
        /* igual */
      }
    }
    await instances.load(true);
    instances.select(inst.id);
    installedId.value = inst.id;
    installedName.value = name;
    emit("installed");
  } catch (e) {
    const parsed = parseInvokeError(e);
    if (parsed.code === "cf_distribution_blocked") {
      distributionUrl.value = parsed.projectUrl ?? props.item.projectUrl ?? null;
    }
    const msg = parsed.message;
    error.value = /cancelad/i.test(msg)
      ? "Instalación cancelada. Podés reintentar cuando quieras."
      : msg;
  } finally {
    busy.value = false;
  }
}

async function play() {
  if (!installedId.value || !installedName.value) return;
  await app.launch(installedId.value, installedName.value);
  open.value = false;
}

async function cancelInstall() {
  for (const t of downloads.active) {
    try {
      await downloads.cancel(t.id);
    } catch {
      /* el grupo pudo haber terminado */
    }
  }
}

function advanced() {
  open.value = false;
  emit("advanced");
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="open && item"
        class="fixed inset-x-0 top-0 bottom-9 z-[300] flex items-center justify-center bg-black/70 p-4"
        @click.self="!busy && (open = false)"
      >
        <div class="flex w-full max-w-md flex-col overflow-hidden rounded-2xl border border-surface-5 bg-surface-2 shadow-2xl">
          <header class="flex items-start justify-between gap-4 border-b border-surface-4 px-5 py-4">
            <div class="flex min-w-0 gap-3">
              <div class="flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-surface-4 text-lg">
                <img v-if="item.iconUrl" :src="item.iconUrl" :alt="item.title" class="h-full w-full object-cover" />
                <span v-else>{{ item.title[0] }}</span>
              </div>
              <div class="min-w-0">
                <h3 class="truncate text-lg font-bold">{{ item.title }}</h3>
                <p class="text-xs text-gray-500">por {{ item.author }}</p>
                <p class="mt-0.5 text-[11px] text-pc-green">Instancia nueva · 2 clics</p>
              </div>
            </div>
            <button class="text-xl text-gray-500 hover:text-white" :disabled="busy" @click="open = false">&times;</button>
          </header>

          <div class="space-y-4 px-5 py-4">
            <p v-if="loading" class="text-sm text-gray-500">Cargando versiones…</p>

            <template v-else-if="!installedId">
              <label class="block">
                <span class="mb-1 block text-sm text-gray-300">Nombre de la instancia</span>
                <input
                  v-model="instanceName"
                  class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
                  :disabled="busy"
                />
              </label>

              <label class="block">
                <span class="mb-1 block text-sm text-gray-300">Versión del pack</span>
                <select
                  v-model="versionId"
                  class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
                  :disabled="busy || !versions.length"
                >
                  <option v-for="v in versions" :key="v.id" :value="v.id">{{ formatVersion(v) }}</option>
                </select>
                <span v-if="selected" class="mt-1 block text-[11px] text-gray-500">
                  Última publicada al tope. Podés elegir otra si hace falta.
                </span>
              </label>

              <label class="block">
                <span class="mb-1 flex items-center justify-between text-sm text-gray-300">
                  RAM
                  <span class="font-semibold text-pc-green">{{ (ramMb / 1024).toFixed(1) }} GB</span>
                </span>
                <input
                  v-model.number="ramMb"
                  type="range"
                  min="2048"
                  :max="maxRam"
                  step="512"
                  class="w-full accent-pc-green"
                  :disabled="busy"
                />
              </label>
            </template>

            <div v-else class="rounded-xl border border-pc-green/30 bg-pc-green/10 px-4 py-3">
              <p class="text-sm font-semibold text-pc-green">{{ installedName }} listo</p>
              <p class="mt-1 text-xs text-gray-400">El pack ya está instalado. Podés jugar ahora.</p>
            </div>

            <p v-if="error" class="rounded-lg bg-red-500/10 px-3 py-2 text-sm text-red-400">{{ error }}</p>
            <p v-if="distributionUrl" class="text-xs text-amber-300">
              Este archivo no se puede bajar desde el launcher.
              <button class="underline" @click="openUrl(distributionUrl!)">Abrir en CurseForge</button>
            </p>
          </div>

          <footer class="flex flex-wrap items-center justify-between gap-2 border-t border-surface-4 px-5 py-3">
            <button
              class="text-xs text-gray-500 underline hover:text-white"
              :disabled="busy"
              @click="advanced"
            >
              Más opciones (servidor)
            </button>
            <div class="flex gap-2">
              <BaseButton v-if="!installedId" variant="secondary" size="sm" :disabled="busy" @click="open = false">
                Cerrar
              </BaseButton>
              <BaseButton
                v-if="busy && !installedId"
                size="sm"
                variant="danger"
                @click="cancelInstall"
              >
                Cancelar
              </BaseButton>
              <BaseButton
                v-if="!installedId"
                size="sm"
                :disabled="busy || loading || !versionId"
                @click="install"
              >
                {{ busy ? "Instalando…" : "Instalar" }}
              </BaseButton>
              <BaseButton v-else size="sm" @click="play">Jugar</BaseButton>
            </div>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
