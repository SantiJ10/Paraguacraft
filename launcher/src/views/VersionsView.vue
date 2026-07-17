<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import SearchInput from "@/components/common/SearchInput.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import VersionCard from "@/components/versions/VersionCard.vue";
import VersionDetailPanel from "@/components/versions/VersionDetailPanel.vue";
import VersionSettingsModal from "@/components/versions/VersionSettingsModal.vue";
import { api, isTauri } from "@/lib/ipc";
import type { BedrockStatus, InstanceContentItem, LoaderInfo, MinecraftVersion } from "@/lib/types";
import {
  buildVersionCards,
  findInstanceForVersion,
  type VersionCardModel,
} from "@/lib/versionCatalog";
import { normalizeLoaderId } from "@/lib/loaders";
import { useAccountsStore } from "@/stores/accounts";
import { useAppStore } from "@/stores/app";
import { useDownloadsStore } from "@/stores/downloads";
import { useInstancesStore } from "@/stores/instances";

const router = useRouter();
const downloads = useDownloadsStore();
const instances = useInstancesStore();
const accounts = useAccountsStore();
const app = useAppStore();

const versions = ref<MinecraftVersion[]>([]);
const cards = ref<VersionCardModel[]>([]);
const selectedCard = ref<VersionCardModel | null>(null);
const mcVersion = ref("");
const loaders = ref<LoaderInfo[]>([]);
const loadingLoaders = ref(false);
const selectedLoaderId = ref("vanilla");
const selectedLoaderVersion = ref("");
const content = ref<InstanceContentItem[]>([]);
const query = ref("");
const filter = ref<"popular" | "all">("popular");
const busy = ref(false);
const message = ref<string | null>(null);
const error = ref<string | null>(null);

const showSettings = ref(false);
const settingsTab = ref<"content" | "files" | "java" | "versions">("content");

const bedrockStatus = ref<BedrockStatus | null>(null);
const bedrockMessage = ref("");

let unlistenBedrockStatus: (() => void) | null = null;
let unlistenBedrockStarted: (() => void) | null = null;
let unlistenBedrockExited: (() => void) | null = null;

const filteredCards = computed(() => {
  const q = query.value.trim().toLowerCase();
  return cards.value.filter((c) => {
    if (filter.value === "popular" && c.kind === "other") return false;
    if (!q) return true;
    return (
      c.title.toLowerCase().includes(q) ||
      c.id.toLowerCase().includes(q) ||
      c.subs.some((s) => s.toLowerCase().includes(q))
    );
  });
});

const linkedInstance = computed((): import("@/lib/types").Instance | null => {
  if (!selectedCard.value || selectedCard.value.kind === "bedrock") return null;
  if (selectedCard.value.kind === "installed") {
    const want = normalizeLoaderId(selectedLoaderId.value);
    return (
      selectedCard.value.instances?.find(
        (i) => i.mcVersion === mcVersion.value && normalizeLoaderId(i.loader) === want,
      ) ?? null
    );
  }
  return findInstanceForVersion(instances.instances, mcVersion.value, selectedLoaderId.value) ?? null;
});

const premiumLocked = computed(
  () => bedrockStatus.value != null && bedrockStatus.value.platformSupported && !bedrockStatus.value.premiumAllowed,
);
const canLaunchBedrock = computed(
  () =>
    isTauri() &&
    !!bedrockStatus.value?.platformSupported &&
    !!bedrockStatus.value?.premiumAllowed &&
    !!bedrockStatus.value?.installed,
);

async function refreshCards() {
  cards.value = buildVersionCards(versions.value, instances.instances);
  if (!selectedCard.value && cards.value.length) {
    await selectCard(cards.value.find((c) => c.id === "1.21") ?? cards.value[0]!);
  }
}

async function refreshBedrock() {
  if (!isTauri()) {
    bedrockStatus.value = { platformSupported: false, installed: false, premiumAllowed: false, username: null };
    return;
  }
  bedrockStatus.value = await api.getBedrockStatus();
}

async function bindBedrockEvents() {
  if (!isTauri()) return;
  const { listen } = await import("@tauri-apps/api/event");
  unlistenBedrockStatus = await listen<{ message: string }>("bedrock://status", (ev) => {
    bedrockMessage.value = ev.payload.message ?? "";
  });
  unlistenBedrockStarted = await listen("bedrock://started", () => {
    app.setLaunch("running", "Bedrock activo");
  });
  unlistenBedrockExited = await listen("bedrock://exited", () => {
    app.setLaunch("idle", "Listo para jugar");
    bedrockMessage.value = "";
    busy.value = false;
  });
}

async function loadContentForInstance(id: string | null) {
  if (!id) {
    content.value = [];
    return;
  }
  try {
    content.value = await api.listInstanceContent(id);
  } catch {
    content.value = [];
  }
}

async function selectCard(card: VersionCardModel) {
  selectedCard.value = card;
  error.value = null;
  message.value = null;

  if (card.kind === "bedrock") {
    loaders.value = [];
    content.value = [];
    return;
  }

  if (card.kind === "installed" && card.instances?.length) {
    mcVersion.value = card.instances[0]!.mcVersion;
    const inst = card.instances[0]!;
    selectedLoaderId.value = inst.loader;
    selectedLoaderVersion.value = inst.loaderVersion;
    await loadContentForInstance(inst.id);
    return;
  }

  mcVersion.value = card.subs[0] ?? card.id;
  selectedLoaderId.value = "vanilla";
  loadingLoaders.value = true;
  try {
    loaders.value = await api.getLoaders(mcVersion.value);
    selectedLoaderVersion.value =
      loaders.value.find((l) => l.id === "vanilla")?.recommended ??
      loaders.value[0]?.recommended ??
      loaders.value[0]?.versions[0] ??
      "";
  } catch (e) {
    error.value = String(e);
  } finally {
    loadingLoaders.value = false;
  }
  await loadContentForInstance(linkedInstance.value?.id ?? null);
}

watch(mcVersion, async (v) => {
  if (!selectedCard.value || selectedCard.value.kind === "bedrock" || selectedCard.value.kind === "installed") return;
  loadingLoaders.value = true;
  try {
    loaders.value = await api.getLoaders(v);
    if (!loaders.value.some((l) => l.id === selectedLoaderId.value)) {
      selectedLoaderId.value = loaders.value[0]?.id ?? "vanilla";
    }
    const loader = loaders.value.find((l) => l.id === selectedLoaderId.value);
    selectedLoaderVersion.value = loader?.recommended ?? loader?.versions[0] ?? "";
  } catch (e) {
    error.value = String(e);
  } finally {
    loadingLoaders.value = false;
  }
  await loadContentForInstance(linkedInstance.value?.id ?? null);
});

watch(selectedLoaderId, async () => {
  const loader = loaders.value.find((l) => l.id === selectedLoaderId.value);
  selectedLoaderVersion.value = loader?.recommended ?? loader?.versions[0] ?? "";
  await loadContentForInstance(linkedInstance.value?.id ?? null);
});

watch(
  () => linkedInstance.value?.id,
  (id) => {
    void loadContentForInstance(id ?? null);
  },
);

onMounted(async () => {
  await downloads.initEvents();
  await accounts.load();
  await instances.load();
  versions.value = await api.getVersions();
  await refreshBedrock();
  await bindBedrockEvents();
  await refreshCards();
});

onUnmounted(() => {
  unlistenBedrockStatus?.();
  unlistenBedrockStarted?.();
  unlistenBedrockExited?.();
});

watch(() => instances.instances.length, () => {
  void refreshCards();
});

async function install() {
  if (!selectedCard.value || busy.value || selectedCard.value.kind === "bedrock") return;
  busy.value = true;
  error.value = null;
  message.value = null;
  const loaderId = selectedLoaderId.value;
  const lv = selectedLoaderVersion.value;
  const loaderName = loaders.value.find((l) => l.id === loaderId)?.name ?? "Vanilla";
  try {
    const name = loaderId === "vanilla" ? `Minecraft ${mcVersion.value}` : `${mcVersion.value} ${loaderName}`;
    const inst = await instances.create({
      name,
      mcVersion: mcVersion.value,
      loader: loaderId,
      loaderVersion: loaderId === "vanilla" ? "" : lv,
      icon: "",
      ramMb: 0,
    });
    message.value = `Instalando ${name}…`;
    if (loaderId === "vanilla") {
      await api.installVersion(mcVersion.value);
    } else {
      await api.installLoader(mcVersion.value, loaderId, lv);
      if (loaderId === "fabric-iris") await api.installFabricIrisBundle(inst.id);
      if (loaderId === "paraguacraft-pvp") await api.installPvpBundle(inst.id);
      if (loaderId === "paraguacraft-pvp-modern") await api.installPvpModernBundle(inst.id);
    }
    message.value = `${name} lista.`;
    await instances.load(true);
    await refreshCards();
    await loadContentForInstance(inst.id);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function play() {
  const inst = linkedInstance.value;
  if (!inst || busy.value) return;
  busy.value = true;
  error.value = null;
  try {
    await app.launch(inst.id, inst.name);
    message.value = `Lanzando ${inst.name}…`;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function launchBedrock() {
  if (!canLaunchBedrock.value || busy.value) return;
  busy.value = true;
  error.value = null;
  bedrockMessage.value = "Abriendo Bedrock…";
  try {
    await api.launchBedrock();
  } catch (e) {
    error.value = String(e);
    bedrockMessage.value = "";
    busy.value = false;
  }
}

function openSettings() {
  if (!linkedInstance.value) return;
  settingsTab.value = "content";
  showSettings.value = true;
}

function goAccounts() {
  router.push({ name: "settings", hash: "#accounts" });
}
</script>

<template>
  <div class="flex h-full min-h-0">
    <!-- Grid central estilo Lunar -->
    <div class="flex min-w-0 flex-1 flex-col">
      <div class="flex flex-wrap items-center gap-3 border-b border-surface-3 px-6 py-4">
        <SearchInput v-model="query" placeholder="Buscar tus versiones PARAGUA…" class="w-72" />
        <select
          v-model="filter"
          class="rounded-lg border border-surface-5 bg-surface-2 px-3 py-2 text-sm outline-none focus:border-pc-green"
        >
          <option value="popular">Popular</option>
          <option value="all">Todas</option>
        </select>
        <div class="flex-1" />
        <BaseButton variant="secondary" size="sm" @click="router.push({ name: 'instances' })">
          + Nuevo perfil
        </BaseButton>
      </div>

      <div class="flex-1 overflow-y-auto p-6">
        <div class="grid grid-cols-2 gap-4 xl:grid-cols-3 2xl:grid-cols-4">
          <VersionCard
            v-for="card in filteredCards"
            :key="card.id"
            :card="card"
            :selected="selectedCard?.id === card.id"
            @select="selectCard"
          />
        </div>
        <p v-if="!filteredCards.length" class="py-20 text-center text-gray-500">No hay versiones que coincidan.</p>
      </div>
    </div>

    <!-- Panel derecho -->
    <VersionDetailPanel
      :card="selectedCard"
      :mc-version="mcVersion"
      :loaders="loaders"
      :loading-loaders="loadingLoaders"
      :selected-loader-id="selectedLoaderId"
      :selected-loader-version="selectedLoaderVersion"
      :linked-instance="linkedInstance"
      :content="content"
      :bedrock-status="bedrockStatus"
      :bedrock-message="bedrockMessage"
      :premium-locked="premiumLocked"
      :can-launch-bedrock="canLaunchBedrock"
      :busy="busy"
      :error="error"
      :message="message"
      @update:selected-loader-id="selectedLoaderId = $event"
      @update:selected-loader-version="selectedLoaderVersion = $event"
      @update:mc-version="mcVersion = $event"
      @install="install"
      @play="play"
      @launch-bedrock="launchBedrock"
      @open-settings="openSettings"
      @go-accounts="goAccounts"
    />

    <VersionSettingsModal
      :open="showSettings"
      :instance-id="linkedInstance?.id ?? null"
      :initial-tab="settingsTab"
      :mc-version="mcVersion"
      @close="showSettings = false"
    />
  </div>
</template>
