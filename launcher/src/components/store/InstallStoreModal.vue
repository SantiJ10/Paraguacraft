<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { api, openUrl, parseInvokeError } from "@/lib/ipc";
import { loadersCompatible, storeLoader } from "@/lib/loaders";
import type {
  ContentType,
  LoaderId,
  LoaderInfo,
  MinecraftVersion,
  StoreInstallDestination,
  StoreItem,
  StoreVersion,
  VersionChannel,
  WorldInfo,
} from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";
import { useInstancesStore } from "@/stores/instances";
import { useServersStore } from "@/stores/servers";
import { useDownloadsStore } from "@/stores/downloads";

const props = defineProps<{ item: StoreItem | null }>();
const open = defineModel<boolean>("open", { default: false });
const emit = defineEmits<{ (e: "installed"): void }>();

const instances = useInstancesStore();
const servers = useServersStore();
const downloads = useDownloadsStore();

const step = ref(0);
const mcVersions = ref<MinecraftVersion[]>([]);
const mcVersion = ref("");
const mcChannel = ref<VersionChannel | "all">("release");
const loaders = ref<LoaderInfo[]>([]);
const loaderId = ref<LoaderId>("fabric");
const instanceId = ref("");
const serverId = ref("");
const worldName = ref("");
const destination = ref<StoreInstallDestination>("instance");
/** Destino de instalación para mods/shaders/RP: local, externo o servidor. */
const installTarget = ref<"local" | "external" | "server">("local");
/** Destino de modpack: instancia cliente o servidor local Fabric/Forge. */
const modpackTarget = ref<"instance" | "server">("instance");
const instanceWorlds = ref<WorldInfo[]>([]);
const serverWorlds = ref<WorldInfo[]>([]);
const modVersions = ref<StoreVersion[]>([]);
const projectVersions = ref<StoreVersion[]>([]);
const selectedVersionId = ref("");

const loadingLoaders = ref(false);
const loadingProjectVersions = ref(false);
const loadingWorlds = ref(false);
const loadingVersions = ref(false);
const busy = ref(false);
const error = ref<string | null>(null);
const distributionUrl = ref<string | null>(null);
const installedModpackName = ref<string | null>(null);

const LOADER_TYPES: ContentType[] = ["mod", "modpack"];

const MODPACK_LOADER_LABELS: Record<string, string> = {
  fabric: "Fabric",
  forge: "Forge",
  neoforge: "NeoForge",
  quilt: "Quilt",
  iris: "Iris (Fabric)",
};

const MODPACK_LOADERS = new Set(["fabric", "forge", "neoforge", "quilt", "iris"]);

function normalizeModrinthLoader(raw: string): string {
  return storeLoader(raw);
}

function versionHasLoader(v: StoreVersion, loader: string): boolean {
  const want = normalizeModrinthLoader(loader);
  return v.loaders.some((l) => normalizeModrinthLoader(l) === want);
}

const MC_CHANNELS: Array<{ id: VersionChannel | "all"; label: string }> = [
  { id: "release", label: "Oficiales" },
  { id: "snapshot", label: "Snapshots" },
  { id: "old_beta", label: "Beta" },
  { id: "old_alpha", label: "Alpha" },
  { id: "all", label: "Todas" },
];

const projectType = computed(() => props.item?.projectType ?? "mod");
const isPlugin = computed(() => projectType.value === "plugin");
const isDatapack = computed(() => projectType.value === "datapack");
const isModpack = computed(() => projectType.value === "modpack");
const isInstanceFlow = computed(
  () => !isPlugin.value && !isDatapack.value && !isModpack.value,
);

const loaderRequired = computed(
  () => !!props.item && LOADER_TYPES.includes(props.item.projectType),
);

const stepLabels = computed(() => {
  if (isPlugin.value) return ["Versión MC", "Servidor local", "Versión del plugin"];
  if (isModpack.value) return ["Versión MC", "Plataforma", "Destino", "Versión del modpack"];
  if (isDatapack.value) {
    return ["Versión MC", "Destino", "Mundo", "Versión del datapack"];
  }
  if (loaderRequired.value) {
    return ["Versión MC", "Plataforma", "Destino", "Ubicación", "Versión del mod"];
  }
  if (isInstanceFlow.value) {
    return ["Versión MC", "Destino", "Ubicación", "Versión"];
  }
  return ["Versión MC", "Instancia", "Versión del mod"];
});

const stepCount = computed(() => stepLabels.value.length);
const versionsStep = computed(() => stepCount.value - 1);

const visibleMcVersions = computed(() =>
  mcVersions.value.filter((v) => mcChannel.value === "all" || v.channel === mcChannel.value),
);

/** MC disponibles según las versiones publicadas del modpack. */
const modpackMcOptions = computed(() => {
  const published = new Set<string>();
  for (const v of projectVersions.value) {
    for (const gv of v.gameVersions) published.add(gv);
  }
  return visibleMcVersions.value.filter((v) => published.has(v.id));
});

const wizardMcVersions = computed(() => (isModpack.value ? modpackMcOptions.value : visibleMcVersions.value));

/** Loaders reales del modpack para la MC elegida (p. ej. solo Fabric o solo Forge). */
const modpackLoadersForMc = computed(() => {
  if (!mcVersion.value) return [];
  const found = new Set<string>();
  for (const v of projectVersions.value) {
    if (!v.gameVersions.includes(mcVersion.value)) continue;
    for (const l of v.loaders) {
      const n = normalizeModrinthLoader(l);
      if (MODPACK_LOADERS.has(n)) found.add(n);
    }
  }
  return [...found].sort();
});

const installLoaders = computed(() => {
  if (isModpack.value) {
    return modpackLoadersForMc.value.map((id) => ({
      id: id as LoaderId,
      name: MODPACK_LOADER_LABELS[id] ?? id,
      description: `Disponible para ${mcVersion.value}`,
      versions: [""],
    }));
  }
  return loaders.value.filter((l) => l.id !== "vanilla" && l.versions.length > 0);
});

function isExternal(id: string): boolean {
  return id.startsWith("ext::");
}

const compatibleInstances = computed(() => {
  if (!mcVersion.value) return [];
  return instances.instances.filter((i) => {
    if (i.mcVersion !== mcVersion.value) return false;
    if (!loaderRequired.value) return true;
    return loadersCompatible(i.loader, loaderId.value);
  });
});

const compatibleServers = computed(() => {
  if (!mcVersion.value) return servers.servers;
  return servers.servers.filter(
    (s) => s.mcVersion === mcVersion.value || s.mcVersion === "?" || !s.mcVersion,
  );
});

const localCompatible = computed(() => compatibleInstances.value.filter((i) => !isExternal(i.id)));
const externalCompatible = computed(() => compatibleInstances.value.filter((i) => isExternal(i.id)));

const modCompatibleServers = computed(() => {
  if (!mcVersion.value || projectType.value !== "mod") return [];
  return servers.servers.filter((s) => {
    if (s.mcVersion !== mcVersion.value && s.mcVersion !== "?" && s.mcVersion) return false;
    if (s.serverType.startsWith("fabric")) return loadersCompatible("fabric", loaderId.value);
    if (s.serverType.startsWith("forge")) return loadersCompatible("forge", loaderId.value);
    return false;
  });
});

const serverDestinationAvailable = computed(
  () => projectType.value === "mod" && modCompatibleServers.value.length > 0,
);

const effectiveLoader = computed(() => {
  if (isPlugin.value) {
    const srv = servers.servers.find((s) => s.id === serverId.value);
    if (!srv) return "paper";
    return srv.serverType.startsWith("fabric") ? "fabric" : "paper";
  }
  if (isDatapack.value) return "minecraft";
  return loaderRequired.value ? loaderId.value : "";
});

const modpackServerAvailable = computed(() => {
  const l = loaderId.value;
  return l === "fabric" || l === "forge" || l === "neoforge" || l === "quilt";
});

const targetStep = computed(() => {
  if (isPlugin.value) return 1;
  if (isDatapack.value) return 2;
  if (loaderRequired.value && isInstanceFlow.value) return 3;
  if (isInstanceFlow.value) return 2;
  return -1;
});

const destinationStep = computed(() => {
  if (isModpack.value) return 2;
  if (isDatapack.value) return 1;
  if (loaderRequired.value && isInstanceFlow.value) return 2;
  if (isInstanceFlow.value) return 1;
  return -1;
});

const canNext = computed(() => {
  if (step.value === 0) return !!mcVersion.value;
  if (isDatapack.value && step.value === destinationStep.value) {
    return destination.value === "instance" || destination.value === "server";
  }
  if (isInstanceFlow.value && step.value === destinationStep.value) {
    if (installTarget.value === "local") return localCompatible.value.length > 0;
    if (installTarget.value === "external") return externalCompatible.value.length > 0;
    return serverDestinationAvailable.value;
  }
  if (loaderRequired.value && isModpack.value && step.value === 1) {
    return modpackLoadersForMc.value.includes(loaderId.value);
  }
  if (isModpack.value && step.value === destinationStep.value) {
    if (modpackTarget.value === "server") return modpackServerAvailable.value;
    return true;
  }
  if (step.value === targetStep.value) {
    if (isPlugin.value) return !!serverId.value;
    if (isDatapack.value) {
      if (destination.value === "server") return !!serverId.value && !!worldName.value;
      return !!instanceId.value && !!worldName.value;
    }
    if (installTarget.value === "server") return !!serverId.value;
    return !!instanceId.value;
  }
  return false;
});

const canInstall = computed(() => !!selectedVersionId.value && !busy.value);

watch(mcChannel, () => {
  const list = wizardMcVersions.value;
  if (!list.some((v) => v.id === mcVersion.value)) {
    mcVersion.value = list[0]?.id ?? "";
  }
});

async function loadProjectVersions() {
  if (!props.item || !isModpack.value) {
    projectVersions.value = [];
    return;
  }
  loadingProjectVersions.value = true;
  error.value = null;
  try {
    projectVersions.value = await api.listStoreProjectVersions({
      provider: props.item.provider,
      projectId: props.item.id,
      projectType: props.item.projectType,
    });
  } catch (e) {
    error.value = String(e);
    projectVersions.value = [];
  } finally {
    loadingProjectVersions.value = false;
  }
}

function syncModpackLoader() {
  const available = modpackLoadersForMc.value;
  if (!available.length) {
    loaderId.value = "fabric";
    return;
  }
  if (!available.includes(loaderId.value)) {
    loaderId.value = available[0] as LoaderId;
  }
}

async function loadInstanceWorlds() {
  if (!instanceId.value) {
    instanceWorlds.value = [];
    return;
  }
  loadingWorlds.value = true;
  try {
    instanceWorlds.value = await api.listInstanceWorlds(instanceId.value);
    const active = instanceWorlds.value.find((w) => w.active);
    worldName.value = active?.name ?? instanceWorlds.value[0]?.name ?? "world";
  } catch (e) {
    error.value = String(e);
    instanceWorlds.value = [];
    worldName.value = "world";
  } finally {
    loadingWorlds.value = false;
  }
}

async function loadServerWorlds() {
  if (!serverId.value) {
    serverWorlds.value = [];
    return;
  }
  loadingWorlds.value = true;
  try {
    const res = await api.listServerWorlds(serverId.value);
    serverWorlds.value = res.worlds;
    const active = res.worlds.find((w) => w.active);
    worldName.value = active?.name ?? res.defaultWorld ?? res.worlds[0]?.name ?? "world";
  } catch (e) {
    error.value = String(e);
    serverWorlds.value = [];
    worldName.value = "world";
  } finally {
    loadingWorlds.value = false;
  }
}

async function resetWizard() {
  step.value = 0;
  error.value = null;
  distributionUrl.value = null;
  installedModpackName.value = null;
  instanceId.value = "";
  serverId.value = "";
  worldName.value = "";
  destination.value = "instance";
  installTarget.value = "local";
  modpackTarget.value = "instance";
  instanceWorlds.value = [];
  serverWorlds.value = [];
  modVersions.value = [];
  selectedVersionId.value = "";
  projectVersions.value = [];
  loaders.value = [];
  loaderId.value = "fabric";
  mcChannel.value = "release";

  if (!mcVersions.value.length) {
    mcVersions.value = await api.getVersions();
  }

  await Promise.all([instances.scan(), servers.load(true)]);

  if (props.item && isModpack.value) {
    await loadProjectVersions();
    const opts = modpackMcOptions.value;
    mcVersion.value =
      opts.find((v) => v.channel === "release")?.id ??
      opts[0]?.id ??
      "";
    syncModpackLoader();
  } else {
    mcVersion.value =
      mcVersions.value.find((v) => v.channel === "release")?.id ??
      mcVersions.value[0]?.id ??
      "";
  }

  if (props.item && loaderRequired.value && mcVersion.value && !isModpack.value) {
    loadingLoaders.value = true;
    try {
      loaders.value = await api.getLoaders(mcVersion.value);
      const first = loaders.value.find((l) => l.id !== "vanilla" && l.versions.length > 0);
      if (first) loaderId.value = first.id;
    } catch (e) {
      error.value = String(e);
    } finally {
      loadingLoaders.value = false;
    }
  }

  if (isPlugin.value) {
    serverId.value = compatibleServers.value[0]?.id ?? "";
  }
}

watch(
  () => open.value,
  (v) => {
    if (v && props.item) void resetWizard();
  },
);

watch(mcVersion, async (mc) => {
  if (!mc) return;
  if (isModpack.value) {
    syncModpackLoader();
    return;
  }
  if (!loaderRequired.value) return;
  loadingLoaders.value = true;
  loaders.value = [];
  try {
    loaders.value = await api.getLoaders(mc);
    const first = installLoaders.value[0];
    if (first) loaderId.value = first.id;
  } catch (e) {
    error.value = String(e);
  } finally {
    loadingLoaders.value = false;
  }
});

watch([compatibleInstances, () => step.value, installTarget], () => {
  if (step.value === targetStep.value && isInstanceFlow.value) {
    if (installTarget.value === "server") {
      const ok = modCompatibleServers.value.some((s) => s.id === serverId.value);
      if (!ok) serverId.value = modCompatibleServers.value[0]?.id ?? "";
      return;
    }
    const pool =
      installTarget.value === "external" ? externalCompatible.value : localCompatible.value;
    const ok = pool.some((i) => i.id === instanceId.value);
    if (!ok) instanceId.value = pool[0]?.id ?? "";
  }
});

watch([compatibleServers, () => step.value], () => {
  if (isPlugin.value && step.value === targetStep.value) {
    const ok = compatibleServers.value.some((s) => s.id === serverId.value);
    if (!ok) serverId.value = compatibleServers.value[0]?.id ?? "";
  }
});

watch(instanceId, () => {
  if (isDatapack.value && destination.value === "instance") void loadInstanceWorlds();
});

watch(serverId, () => {
  if (isPlugin.value || (isDatapack.value && destination.value === "server")) {
    void loadServerWorlds();
  }
});

watch(installTarget, () => {
  if (installTarget.value === "server") {
    serverId.value = modCompatibleServers.value[0]?.id ?? "";
  } else {
    const pool =
      installTarget.value === "external" ? externalCompatible.value : localCompatible.value;
    instanceId.value = pool[0]?.id ?? "";
  }
});

function syncInstallTargetDefault() {
  if (localCompatible.value.length) installTarget.value = "local";
  else if (externalCompatible.value.length) installTarget.value = "external";
  else if (serverDestinationAvailable.value) installTarget.value = "server";
}
watch(destination, () => {
  worldName.value = "";
  if (destination.value === "server") {
    serverId.value = compatibleServers.value[0]?.id ?? "";
    void loadServerWorlds();
  } else {
    instanceId.value = compatibleInstances.value[0]?.id ?? "";
    void loadInstanceWorlds();
  }
});

watch([localCompatible, externalCompatible, modCompatibleServers], () => {
  if (open.value && isInstanceFlow.value) syncInstallTargetDefault();
});

async function loadModVersions() {
  if (!props.item || !mcVersion.value) return;
  loadingVersions.value = true;
  error.value = null;
  modVersions.value = [];
  selectedVersionId.value = "";
  try {
    if (isModpack.value) {
      const ext = props.item.provider === "curseforge" ? ".zip" : ".mrpack";
      modVersions.value = projectVersions.value.filter(
        (v) =>
          v.gameVersions.includes(mcVersion.value) &&
          versionHasLoader(v, loaderId.value) &&
          v.filename.toLowerCase().endsWith(ext),
      );
    } else {
      modVersions.value = await api.listStoreVersions({
        provider: props.item.provider,
        projectId: props.item.id,
        projectType: props.item.projectType,
        mc: mcVersion.value,
        loader: effectiveLoader.value,
      });
    }
    selectedVersionId.value = modVersions.value[0]?.id ?? "";
  } catch (e) {
    error.value = String(e);
  } finally {
    loadingVersions.value = false;
  }
}

async function nextStep() {
  error.value = null;
  if (step.value >= versionsStep.value) return;

  const next = step.value + 1;
  if (next === versionsStep.value) {
    step.value = next;
    await loadModVersions();
    return;
  }

  step.value = next;
  if (isInstanceFlow.value && next === destinationStep.value) {
    syncInstallTargetDefault();
  }
  if (isDatapack.value && next === targetStep.value) {
    if (destination.value === "server") {
      serverId.value = compatibleServers.value[0]?.id ?? "";
      await loadServerWorlds();
    } else {
      instanceId.value = compatibleInstances.value[0]?.id ?? "";
      await loadInstanceWorlds();
    }
  }
}

function prevStep() {
  error.value = null;
  if (step.value > 0) step.value -= 1;
}

async function install() {
  if (!props.item || !selectedVersionId.value) return;
  busy.value = true;
  error.value = null;
  distributionUrl.value = null;
  installedModpackName.value = null;
  try {
    await downloads.initEvents();

    if (isModpack.value) {
      if (modpackTarget.value === "server") {
        const srv =
          props.item.provider === "curseforge"
            ? await api.importCfpackVersionToServer(props.item.id, selectedVersionId.value)
            : await api.importMrpackVersionToServer(selectedVersionId.value);
        installedModpackName.value = srv.name;
        await servers.load(true);
      } else {
        let inst;
        if (props.item.provider === "curseforge") {
          inst = await api.importCfpackVersion(props.item.id, selectedVersionId.value);
        } else {
          inst = await api.importMrpackVersion(selectedVersionId.value);
        }
        installedModpackName.value = inst.name;
        await instances.load(true);
      }
      emit("installed");
      open.value = false;
      return;
    }

    await api.installStoreVersion({
      provider: props.item.provider,
      projectId: props.item.id,
      projectType: props.item.projectType,
      versionId: selectedVersionId.value,
      mc: mcVersion.value,
      loader: effectiveLoader.value,
      instanceId:
        isInstanceFlow.value && installTarget.value !== "server"
          ? instanceId.value
          : isDatapack.value && destination.value === "instance"
            ? instanceId.value
            : undefined,
      destination: isDatapack.value
        ? destination.value
        : isPlugin.value || (isInstanceFlow.value && installTarget.value === "server")
          ? "server"
          : "instance",
      serverId:
        isPlugin.value ||
        (isDatapack.value && destination.value === "server") ||
        (isInstanceFlow.value && installTarget.value === "server")
          ? serverId.value
          : undefined,
      worldName: isDatapack.value ? worldName.value : undefined,
    });

    await instances.load(true);
    emit("installed");
    open.value = false;
  } catch (e) {
    const parsed = parseInvokeError(e);
    if (parsed.code === "cf_distribution_blocked") {
      distributionUrl.value = parsed.projectUrl ?? props.item.projectUrl ?? null;
    }
    error.value = parsed.message;
  } finally {
    busy.value = false;
  }
}

function formatDate(iso: string) {
  if (!iso) return "";
  try {
    return new Date(iso).toLocaleDateString();
  } catch {
    return iso.slice(0, 10);
  }
}

function sourceLabel(source: string): string {
  const labels: Record<string, string> = {
    paraguacraft: "Paraguacraft",
    vanilla: "Vanilla",
    prism: "Prism",
    lunar: "Lunar",
    tlauncher: "TLauncher",
    sklauncher: "SKLauncher",
  };
  return labels[source] ?? source;
}

function serverTypeLabel(t: string): string {
  if (t.startsWith("fabric")) return "Fabric";
  if (t.startsWith("paper")) return "Paper";
  if (t.startsWith("forge")) return "Forge";
  return t;
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="open && item"
        class="fixed inset-0 z-[300] flex items-center justify-center bg-black/70 p-4"
        @click.self="open = false"
      >
        <div class="flex max-h-[90vh] w-full max-w-xl flex-col rounded-2xl border border-surface-5 bg-surface-2 shadow-2xl">
          <header class="flex shrink-0 items-start justify-between gap-4 border-b border-surface-4 px-6 py-4">
            <div class="flex min-w-0 gap-3">
              <div
                class="flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-surface-4 text-lg"
              >
                <img v-if="item.iconUrl" :src="item.iconUrl" :alt="item.title" class="h-full w-full object-cover" />
                <span v-else>{{ item.title[0] }}</span>
              </div>
              <div class="min-w-0">
                <h3 class="truncate text-lg font-bold">Instalar {{ item.title }}</h3>
                <p class="text-xs text-gray-500">por {{ item.author }}</p>
                <p v-if="isModpack" class="mt-0.5 text-[11px] text-pc-green">
                  {{
                    modpackTarget === "server"
                      ? "Se creará un servidor local Fabric/Forge con el modpack"
                      : "Se creará una instancia nueva"
                  }}
                  {{ item.provider === 'curseforge' ? '(.zip CurseForge)' : '(.mrpack Modrinth)' }}
                </p>
                <p v-else-if="isPlugin" class="mt-0.5 text-[11px] text-gray-500">
                  Se instala en plugins/ o mods/ del servidor local
                </p>
                <p v-else-if="isDatapack" class="mt-0.5 text-[11px] text-gray-500">
                  Elegí instancia (saves/mundo) o servidor local (world/datapacks)
                </p>
              </div>
            </div>
            <button class="text-xl text-gray-500 hover:text-white" @click="open = false">&times;</button>
          </header>

          <div class="flex shrink-0 gap-1 border-b border-surface-4 px-6 py-3">
            <div
              v-for="(label, i) in stepLabels"
              :key="label"
              class="flex flex-1 flex-col items-center gap-1"
            >
              <div
                class="flex h-7 w-7 items-center justify-center rounded-full text-xs font-bold"
                :class="
                  i === step
                    ? 'bg-pc-green text-black'
                    : i < step
                      ? 'bg-pc-green/30 text-pc-green'
                      : 'bg-surface-4 text-gray-500'
                "
              >
                {{ i + 1 }}
              </div>
              <span class="hidden text-[10px] text-gray-500 sm:block">{{ label }}</span>
            </div>
          </div>

          <div class="flex-1 overflow-y-auto px-6 py-4">
            <!-- Paso: Versión MC -->
            <div v-if="step === 0" class="space-y-3">
              <p class="text-sm text-gray-400">
                <template v-if="isModpack">
                  Elegí la versión de Minecraft en la que este modpack está publicado.
                </template>
                <template v-else>Elegí el canal y la versión de Minecraft.</template>
              </p>
              <p v-if="isModpack && loadingProjectVersions" class="text-sm text-gray-500">
                Consultando versiones del modpack…
              </p>
              <div class="flex flex-wrap gap-1.5">
                <button
                  v-for="c in MC_CHANNELS"
                  :key="c.id"
                  type="button"
                  class="rounded-lg px-2.5 py-1 text-xs font-semibold transition"
                  :class="
                    mcChannel === c.id
                      ? 'bg-pc-green text-black'
                      : 'bg-surface-3 text-gray-400 hover:bg-surface-4'
                  "
                  @click="mcChannel = c.id"
                >
                  {{ c.label }}
                </button>
              </div>
              <select
                v-model="mcVersion"
                class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
                :disabled="isModpack && loadingProjectVersions"
              >
                <option v-for="v in wizardMcVersions" :key="v.id" :value="v.id" class="bg-surface-2">
                  {{ v.id }}
                  <template v-if="v.channel !== 'release'"> ({{ v.channel.replace("_", " ") }})</template>
                </option>
              </select>
              <p v-if="!wizardMcVersions.length" class="text-sm text-amber-400">
                <template v-if="isModpack">
                  Este modpack no tiene releases compatibles en el canal seleccionado.
                </template>
                <template v-else>No hay versiones en este canal.</template>
              </p>
            </div>

            <!-- Paso: Plataforma / Loader (mod / modpack) -->
            <div v-else-if="loaderRequired && step === 1 && !isDatapack && !isPlugin" class="space-y-3">
              <p v-if="isModpack" class="text-sm text-gray-400">
                Plataformas disponibles para {{ mcVersion }} en este modpack
                (p. ej. solo Fabric o solo Forge).
              </p>
              <p v-else class="text-sm text-gray-400">Elegí la plataforma (loader) compatible con {{ mcVersion }}.</p>
              <p v-if="!isModpack && loadingLoaders" class="text-sm text-gray-500">Consultando loaders…</p>
              <div v-else-if="!installLoaders.length" class="text-sm text-amber-400">
                <template v-if="isModpack">
                  No hay plataforma publicada para {{ mcVersion }} en este modpack.
                </template>
                <template v-else>No hay loaders modded para {{ mcVersion }}.</template>
              </div>
              <div v-else class="grid gap-2">
                <button
                  v-for="l in installLoaders"
                  :key="l.id"
                  type="button"
                  class="rounded-xl border-2 p-3 text-left transition-all"
                  :class="
                    loaderId === l.id
                      ? 'border-pc-green bg-pc-green/10'
                      : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                  "
                  @click="loaderId = l.id"
                >
                  <p class="font-semibold">{{ l.name }}</p>
                  <p class="text-xs text-gray-500">{{ l.description }}</p>
                </button>
              </div>
            </div>

            <!-- Paso: Destino modpack -->
            <div v-else-if="isModpack && step === destinationStep" class="space-y-3">
              <p class="text-sm text-gray-400">¿Dónde querés instalar el modpack?</p>
              <div class="grid gap-2">
                <button
                  type="button"
                  class="rounded-xl border-2 p-4 text-left transition-all"
                  :class="
                    modpackTarget === 'instance'
                      ? 'border-pc-green bg-pc-green/10'
                      : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                  "
                  @click="modpackTarget = 'instance'"
                >
                  <p class="font-semibold">Instancia de juego (cliente)</p>
                  <p class="text-xs text-gray-500">Nueva instancia Paraguacraft con mods y loader</p>
                </button>
                <button
                  type="button"
                  class="rounded-xl border-2 p-4 text-left transition-all"
                  :class="
                    modpackTarget === 'server'
                      ? 'border-pc-green bg-pc-green/10'
                      : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                  "
                  :disabled="!modpackServerAvailable"
                  @click="modpackTarget = 'server'"
                >
                  <p class="font-semibold">Servidor local</p>
                  <p class="text-xs text-gray-500">
                    {{
                      modpackServerAvailable
                        ? `Fabric/Forge · mods del lado servidor · ${loaderId}`
                        : "Solo modpacks Fabric o Forge pueden ir a servidor"
                    }}
                  </p>
                </button>
              </div>
            </div>

            <!-- Paso: Destino (mod / shader / resourcepack) -->
            <div v-else-if="isInstanceFlow && step === destinationStep" class="space-y-3">
              <p class="text-sm text-gray-400">¿Dónde querés instalar este contenido?</p>
              <div class="grid gap-2">
                <button
                  type="button"
                  class="rounded-xl border-2 p-4 text-left transition-all"
                  :class="
                    installTarget === 'local'
                      ? 'border-pc-green bg-pc-green/10'
                      : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                  "
                  :disabled="!localCompatible.length"
                  @click="installTarget = 'local'"
                >
                  <p class="font-semibold">Instancia Paraguacraft</p>
                  <p class="text-xs text-gray-500">
                    {{ localCompatible.length ? `${localCompatible.length} compatible(s)` : "Sin instancias compatibles" }}
                    · mods/
                  </p>
                </button>
                <button
                  type="button"
                  class="rounded-xl border-2 p-4 text-left transition-all"
                  :class="
                    installTarget === 'external'
                      ? 'border-pc-green bg-pc-green/10'
                      : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                  "
                  :disabled="!externalCompatible.length"
                  @click="installTarget = 'external'"
                >
                  <p class="font-semibold">Otro launcher (Prism, Lunar, .minecraft)</p>
                  <p class="text-xs text-gray-500">
                    {{
                      externalCompatible.length
                        ? `${externalCompatible.length} detectada(s)`
                        : "Usá «Escanear» en Instancias si no aparece ninguna"
                    }}
                  </p>
                </button>
                <button
                  v-if="projectType === 'mod'"
                  type="button"
                  class="rounded-xl border-2 p-4 text-left transition-all"
                  :class="
                    installTarget === 'server'
                      ? 'border-pc-green bg-pc-green/10'
                      : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                  "
                  :disabled="!serverDestinationAvailable"
                  @click="installTarget = 'server'"
                >
                  <p class="font-semibold">Servidor local</p>
                  <p class="text-xs text-gray-500">
                    {{
                      serverDestinationAvailable
                        ? `Fabric/Forge · ${modCompatibleServers.length} servidor(es)`
                        : "Requiere un servidor Fabric o Forge compatible"
                    }}
                    · mods/
                  </p>
                </button>
              </div>
            </div>

            <!-- Paso: Destino datapack -->
            <div v-else-if="isDatapack && step === destinationStep" class="space-y-3">
              <p class="text-sm text-gray-400">¿Dónde querés instalar el datapack?</p>
              <div class="grid gap-2">
                <button
                  type="button"
                  class="rounded-xl border-2 p-4 text-left transition-all"
                  :class="
                    destination === 'instance'
                      ? 'border-pc-green bg-pc-green/10'
                      : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                  "
                  @click="destination = 'instance'"
                >
                  <p class="font-semibold">Instancia / mundo local</p>
                  <p class="text-xs text-gray-500">saves/&lt;mundo&gt;/datapacks/</p>
                </button>
                <button
                  type="button"
                  class="rounded-xl border-2 p-4 text-left transition-all"
                  :class="
                    destination === 'server'
                      ? 'border-pc-green bg-pc-green/10'
                      : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                  "
                  @click="destination = 'server'"
                >
                  <p class="font-semibold">Servidor local</p>
                  <p class="text-xs text-gray-500">&lt;mundo&gt;/datapacks/ del servidor (cmd + playit)</p>
                </button>
              </div>
            </div>

            <!-- Paso: Servidor (plugin) -->
            <div v-else-if="isPlugin && step === targetStep" class="space-y-3">
              <p class="text-sm text-gray-400">
                Elegí el servidor local destino ({{ mcVersion }}).
              </p>
              <p v-if="!compatibleServers.length" class="text-sm text-amber-400">
                No hay servidores locales. Creá uno en Servidores o importá una carpeta existente.
              </p>
              <div v-else class="space-y-2">
                <button
                  v-for="srv in compatibleServers"
                  :key="srv.id"
                  type="button"
                  class="flex w-full items-center gap-3 rounded-xl border-2 p-3 text-left transition-all"
                  :class="
                    serverId === srv.id
                      ? 'border-pc-green bg-pc-green/10'
                      : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                  "
                  @click="serverId = srv.id"
                >
                  <span class="text-xl">🖥️</span>
                  <div class="min-w-0">
                    <p class="truncate font-semibold">{{ srv.name }}</p>
                    <p class="text-xs text-gray-500">
                      {{ srv.mcVersion }} · {{ serverTypeLabel(srv.serverType) }}
                      · {{ srv.serverType.startsWith('fabric') ? 'mods/' : 'plugins/' }}
                    </p>
                  </div>
                </button>
              </div>
            </div>

            <!-- Paso: Ubicación (mods, shaders, rp) -->
            <div v-else-if="isInstanceFlow && step === targetStep" class="space-y-3">
              <template v-if="installTarget === 'server'">
                <p class="text-sm text-gray-400">
                  Elegí el servidor local ({{ mcVersion }} · {{ loaderId }})
                </p>
                <p v-if="!modCompatibleServers.length" class="text-sm text-amber-400">
                  No hay servidores Fabric/Forge compatibles. Creá uno en Servidores.
                </p>
                <div v-else class="space-y-2">
                  <button
                    v-for="srv in modCompatibleServers"
                    :key="srv.id"
                    type="button"
                    class="flex w-full items-center gap-3 rounded-xl border-2 p-3 text-left transition-all"
                    :class="
                      serverId === srv.id
                        ? 'border-pc-green bg-pc-green/10'
                        : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                    "
                    @click="serverId = srv.id"
                  >
                    <span class="text-xl">🖥️</span>
                    <div class="min-w-0">
                      <p class="truncate font-semibold">{{ srv.name }}</p>
                      <p class="text-xs text-gray-500">
                        {{ srv.mcVersion }} · {{ serverTypeLabel(srv.serverType) }} · mods/
                      </p>
                    </div>
                  </button>
                </div>
              </template>

              <template v-else>
                <p class="text-sm text-gray-400">
                  Elegí la instancia destino
                  <template v-if="loaderRequired"> ({{ mcVersion }} · {{ loaderId }})</template>
                  <template v-else> ({{ mcVersion }})</template>
                  · {{ installTarget === "external" ? "otro launcher" : "Paraguacraft" }}.
                </p>
                <p
                  v-if="!(installTarget === 'external' ? externalCompatible : localCompatible).length"
                  class="text-sm text-amber-400"
                >
                  No hay instancias compatibles. Creá una en Versiones/Instancias o escaneá otros launchers.
                </p>
                <div class="space-y-2">
                  <button
                    v-for="inst in installTarget === 'external' ? externalCompatible : localCompatible"
                    :key="inst.id"
                    type="button"
                    class="flex w-full items-center gap-3 rounded-xl border-2 p-3 text-left transition-all"
                    :class="
                      instanceId === inst.id
                        ? 'border-pc-green bg-pc-green/10'
                        : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                    "
                    @click="instanceId = inst.id"
                  >
                    <span class="text-xl">{{ inst.icon || (installTarget === 'external' ? '🔗' : '📦') }}</span>
                    <div class="min-w-0">
                      <p class="truncate font-semibold">{{ inst.name }}</p>
                      <p class="text-xs text-gray-500">
                        {{ inst.mcVersion }} · {{ inst.loader }}
                        <template v-if="installTarget === 'external'">
                          · {{ sourceLabel(inst.source) }}
                        </template>
                      </p>
                    </div>
                  </button>
                </div>
              </template>
            </div>

            <!-- Paso: Mundo (datapack) -->
            <div v-else-if="isDatapack && step === targetStep" class="space-y-4">
              <template v-if="destination === 'server'">
                <div class="space-y-2">
                  <p class="text-sm text-gray-400">Servidor local</p>
                  <p v-if="!compatibleServers.length" class="text-sm text-amber-400">No hay servidores compatibles.</p>
                  <div v-else class="space-y-2">
                    <button
                      v-for="srv in compatibleServers"
                      :key="srv.id"
                      type="button"
                      class="flex w-full items-center gap-3 rounded-xl border-2 p-3 text-left transition-all"
                      :class="
                        serverId === srv.id
                          ? 'border-pc-green bg-pc-green/10'
                          : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                      "
                      @click="serverId = srv.id"
                    >
                      <span class="text-xl">🖥️</span>
                      <div class="min-w-0">
                        <p class="truncate font-semibold">{{ srv.name }}</p>
                        <p class="text-xs text-gray-500">{{ srv.mcVersion }} · {{ serverTypeLabel(srv.serverType) }}</p>
                      </div>
                    </button>
                  </div>
                </div>
              </template>

              <template v-else>
                <div class="space-y-2">
                  <p class="text-sm text-gray-400">Instancia ({{ mcVersion }})</p>
                  <p v-if="!compatibleInstances.length" class="text-sm text-amber-400">No hay instancias compatibles.</p>
                  <select
                    v-model="instanceId"
                    class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
                  >
                    <option v-for="inst in compatibleInstances" :key="inst.id" :value="inst.id" class="bg-surface-2">
                      {{ inst.name }} ({{ inst.mcVersion }})
                    </option>
                  </select>
                </div>
              </template>

              <div class="space-y-2">
                <p class="text-sm text-gray-400">Mundo destino</p>
                <p v-if="loadingWorlds" class="text-sm text-gray-500">Detectando mundos…</p>
                <select
                  v-else
                  v-model="worldName"
                  class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
                >
                  <option
                    v-for="w in destination === 'server' ? serverWorlds : instanceWorlds"
                    :key="w.name"
                    :value="w.name"
                    class="bg-surface-2"
                  >
                    {{ w.name }}{{ w.active ? " (activo)" : "" }}
                  </option>
                  <option
                    v-if="!(destination === 'server' ? serverWorlds : instanceWorlds).length"
                    value="world"
                    class="bg-surface-2"
                  >
                    world
                  </option>
                </select>
                <p class="text-[11px] text-gray-600">
                  Se instalará en
                  <code class="text-gray-500">
                    {{ destination === 'server' ? '<servidor>/<mundo>/datapacks/' : 'saves/<mundo>/datapacks/' }}
                  </code>
                </p>
              </div>
            </div>

            <!-- Paso: Versión -->
            <div v-else-if="step === versionsStep" class="space-y-3">
              <p class="text-sm text-gray-400">Versiones compatibles con tu selección.</p>
              <p v-if="loadingVersions" class="text-sm text-gray-500">Cargando versiones…</p>
              <p v-else-if="!modVersions.length" class="text-sm text-amber-400">
                No hay versiones de este proyecto para {{ mcVersion }}
                <template v-if="loaderRequired || isPlugin"> / {{ effectiveLoader }}</template>.
              </p>
              <div v-else class="max-h-64 space-y-2 overflow-y-auto">
                <button
                  v-for="ver in modVersions"
                  :key="ver.id"
                  type="button"
                  class="flex w-full flex-col rounded-lg border-2 p-3 text-left transition-all"
                  :class="
                    selectedVersionId === ver.id
                      ? 'border-pc-green bg-pc-green/10'
                      : 'border-surface-4 bg-surface-3 hover:border-surface-6'
                  "
                  @click="selectedVersionId = ver.id"
                >
                  <span class="font-semibold">{{ ver.name || ver.versionNumber }}</span>
                  <span class="text-xs text-gray-500">{{ ver.filename }}</span>
                  <span v-if="ver.publishedAt" class="text-xs text-gray-600">{{ formatDate(ver.publishedAt) }}</span>
                </button>
              </div>
            </div>

            <p v-if="error" class="mt-4 rounded-lg bg-red-500/10 px-3 py-2 text-sm text-red-400">{{ error }}</p>
            <div v-if="distributionUrl" class="mt-2">
              <BaseButton size="sm" variant="secondary" @click="openUrl(distributionUrl!)">
                Descargar manualmente en CurseForge
              </BaseButton>
            </div>
          </div>

          <footer class="flex shrink-0 items-center justify-between gap-2 border-t border-surface-4 px-6 py-4">
            <BaseButton v-if="step > 0" variant="ghost" :disabled="busy" @click="prevStep">Atrás</BaseButton>
            <div v-else />

            <div class="flex gap-2">
              <BaseButton variant="ghost" @click="open = false">Cancelar</BaseButton>
              <BaseButton
                v-if="step < versionsStep"
                :disabled="!canNext || busy"
                @click="nextStep"
              >
                Siguiente
              </BaseButton>
              <BaseButton
                v-else
                :disabled="!canInstall"
                @click="install"
              >
                {{ busy ? "Instalando…" : isModpack ? (modpackTarget === "server" ? "Crear servidor" : "Crear instancia") : "Instalar" }}
              </BaseButton>
            </div>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
