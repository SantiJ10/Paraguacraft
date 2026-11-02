<script setup lang="ts">
defineOptions({ name: "servers" });
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useServersStore } from "@/stores/servers";
import { useAppStore } from "@/stores/app";
import { api, isTauri } from "@/lib/ipc";
import type { MinecraftVersion, ServerProfile, ServerType } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";
import AppSelect from "@/components/common/AppSelect.vue";
import ContextMenu, { type ContextMenuItem } from "@/components/common/ContextMenu.vue";

type EngineMode = "java-cmd" | "rust-horizon";

const JAVA_SERVER_TYPES: Array<{ id: ServerType; label: string; icon: string; desc: string }> = [
  { id: "paper", label: "Paper", icon: "📄", desc: "Plugins · más estable" },
  { id: "paper-geyser", label: "Paper + Geyser", icon: "🌐", desc: "Plugins + Bedrock" },
  { id: "fabric", label: "Fabric", icon: "🧵", desc: "Mods · server side" },
  { id: "fabric-geyser", label: "Fabric + Geyser", icon: "🧵🌐", desc: "Mods + Bedrock" },
  { id: "forge", label: "Forge", icon: "🔧", desc: "Modpacks CurseForge · MC ≤1.20.1" },
  { id: "neoforge", label: "NeoForge", icon: "⚙️", desc: "Modpacks modernos · MC 1.20.2+" },
];

const SERVER_TYPE_LABELS: Record<string, string> = {
  ...Object.fromEntries(JAVA_SERVER_TYPES.map((t) => [t.id, t.label])),
  "rust-native": "Motor Ultraligero Rust (Horizon / Pumpkin)",
};

function serverTypeLabel(t: string): string {
  return SERVER_TYPE_LABELS[t] ?? t;
}

const SERVER_RAM_PRESETS_MB = [2048, 4096, 6144, 8192, 12288, 16384] as const;

const GUIDE_KEY = "paraguacraft.servers.localGuideOpen";
const guideOpen = ref(true);
try {
  const saved = localStorage.getItem(GUIDE_KEY);
  if (saved === "0") guideOpen.value = false;
  if (saved === "1") guideOpen.value = true;
} catch {
  /* ignore */
}

function toggleGuide() {
  guideOpen.value = !guideOpen.value;
  try {
    localStorage.setItem(GUIDE_KEY, guideOpen.value ? "1" : "0");
  } catch {
    /* ignore */
  }
}

function formatRamLabel(s: ServerProfile): string {
  if (s.serverType === "rust-native") return "~80 MB RAM (Sin Java)";
  if (s.ramMb < 1024) return `${s.ramMb} MB`;
  return `${s.ramMb / 1024} GB`;
}

function formatRamGb(mb: number): string {
  return `${mb / 1024} GB`;
}

const router = useRouter();
const serversStore = useServersStore();
const app = useAppStore();

const loading = ref(!serversStore.loaded);
const busy = ref<string | null>(null);
const error = ref<string | null>(null);
const showCreate = ref(false);

const engineMode = ref<EngineMode>("java-cmd");
const newName = ref("Mi servidor");
const newMc = ref("1.21.1");
const newType = ref<ServerType>("paper");
const newRam = ref<number>(4096);
const newShowCmd = ref<boolean>(false);
const mcVersions = ref<MinecraftVersion[]>([]);
const versionsLoading = ref(false);

const releaseMcVersions = computed(() =>
  mcVersions.value.filter((v) => v.channel === "release"),
);

const ramOptions = computed(() => {
  const maxMb = app.hardware ? Math.floor(app.hardware.ramGb * 1024 * 0.75) : Infinity;
  return SERVER_RAM_PRESETS_MB.filter((mb) => mb <= maxMb);
});

const lastActiveServer = computed(() =>
  serversStore.servers.find((s) => s.id === serversStore.lastActiveId) ?? null,
);

onMounted(() => {
  void refresh(false);
});

watch(engineMode, (mode) => {
  if (mode === "rust-horizon") {
    newType.value = "rust-native";
    newMc.value = releaseMcVersions.value[0]?.id ?? "26.3";
  } else if (newType.value === "rust-native") {
    newType.value = "paper";
    newMc.value =
      releaseMcVersions.value.find((v) => v.id === "1.21.1")?.id ??
      releaseMcVersions.value[0]?.id ??
      "1.21.1";
  }
});

async function ensureCreateDeps() {
  if (!app.hardware) void app.loadHardware();
  if (mcVersions.value.length || versionsLoading.value) return;
  versionsLoading.value = true;
  try {
    mcVersions.value = await api.getVersions();
    if (engineMode.value === "rust-horizon") {
      newMc.value = releaseMcVersions.value[0]?.id ?? "26.3";
    } else {
      newMc.value =
        releaseMcVersions.value.find((v) => v.id === "1.21.1")?.id ??
        releaseMcVersions.value[0]?.id ??
        "1.21.1";
    }
    if (
      ramOptions.value.length &&
      !ramOptions.value.includes(newRam.value as (typeof SERVER_RAM_PRESETS_MB)[number])
    ) {
      newRam.value = ramOptions.value[ramOptions.value.length - 1] ?? 4096;
    }
  } finally {
    versionsLoading.value = false;
  }
}

watch(showCreate, (open) => {
  if (open) void ensureCreateDeps();
});

async function refresh(force = true) {
  if (!serversStore.loaded) loading.value = true;
  error.value = null;
  try {
    await serversStore.load(force);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function create() {
  busy.value = "create";
  error.value = null;
  try {
    await ensureCreateDeps();
    const isRust = engineMode.value === "rust-horizon";
    const s = await serversStore.create({
      name: newName.value.trim(),
      mcVersion: isRust ? (releaseMcVersions.value[0]?.id ?? "26.3") : newMc.value.trim(),
      serverType: isRust ? "rust-native" : newType.value,
      ramMb: isRust ? 256 : newRam.value,
      showCmdWindow: newShowCmd.value,
    });
    showCreate.value = false;
    router.push({ name: "server-detail", params: { id: s.id } });
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}

async function importFolder() {
  if (!isTauri()) return;
  busy.value = "import";
  error.value = null;
  try {
    const path = await api.pickServerFolder();
    if (!path) return;
    const s = await serversStore.importFolder(path);
    router.push({ name: "server-detail", params: { id: s.id } });
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}

function openServer(id: string) {
  serversStore.setLastActive(id);
  router.push({ name: "server-detail", params: { id } });
}

const menu = ref<{ x: number; y: number; server: ServerProfile } | null>(null);

const menuItems: ContextMenuItem[] = [
  { id: "open", label: "Abrir" },
  { id: "rename", label: "Renombrar" },
  { id: "sep", label: "", separator: true },
  { id: "delete", label: "Eliminar", danger: true },
];

function onServerContext(e: MouseEvent, s: ServerProfile) {
  e.preventDefault();
  e.stopPropagation();
  menu.value = { x: e.clientX, y: e.clientY, server: s };
}

async function onMenuSelect(id: string) {
  const ctx = menu.value;
  menu.value = null;
  if (!ctx) return;
  const s = ctx.server;
  if (id === "open") {
    openServer(s.id);
    return;
  }
  if (id === "rename") {
    const name = window.prompt("Nuevo nombre del servidor", s.name)?.trim();
    if (!name || name === s.name) return;
    busy.value = "rename";
    error.value = null;
    try {
      await serversStore.update({ id: s.id, name });
    } catch (e) {
      error.value = String(e);
    } finally {
      busy.value = null;
    }
    return;
  }
  if (id === "delete") {
    const ok = window.confirm(`¿Eliminar el servidor "${s.name}"?`);
    if (!ok) return;
    busy.value = "delete";
    error.value = null;
    try {
      await serversStore.remove(s.id);
    } catch (e) {
      error.value = String(e);
    } finally {
      busy.value = null;
    }
  }
}
</script>

<template>
  <div class="p-8">
    <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
      <div>
        <h1 class="text-2xl font-bold">Servidores para Amigos</h1>
        <p class="text-sm text-gray-500">
          Elegí entre el Modo Clásico (Java / CMD + Playit.gg) o el Motor Ultraligero Nativo Rust (estilo Horizon + Playit.gg).
        </p>
      </div>
      <div class="flex gap-2">
        <BaseButton variant="secondary" :disabled="!!busy || !isTauri()" @click="importFolder">
          Importar carpeta
        </BaseButton>
        <BaseButton @click="showCreate = !showCreate">
          {{ showCreate ? "Cancelar" : "Nuevo servidor" }}
        </BaseButton>
      </div>
    </div>

    <button
      v-if="lastActiveServer"
      type="button"
      class="mb-4 flex w-full items-center justify-between gap-3 rounded-xl border border-pc-green/35 bg-pc-green/10 px-4 py-3 text-left transition hover:bg-pc-green/15"
      @click="openServer(lastActiveServer.id)"
    >
      <div>
        <p class="text-xs font-semibold uppercase tracking-wide text-pc-green/80">Continuar</p>
        <p class="font-bold text-white">{{ lastActiveServer.name }}</p>
        <p class="text-xs text-gray-400">Consola y estado del server que estabas mirando</p>
      </div>
      <span class="text-sm text-pc-green">Abrir →</span>
    </button>

    <section
      class="mb-6 overflow-hidden rounded-2xl border border-sky-500/30 bg-gradient-to-br from-sky-500/10 via-surface-2 to-surface-2"
    >
      <button
        type="button"
        class="flex w-full items-center justify-between gap-3 px-4 py-3 text-left transition hover:bg-white/5"
        @click="toggleGuide"
      >
        <div class="min-w-0">
          <p class="text-sm font-bold text-sky-200">Dos formas de hostear para tus amigos + Playit.gg</p>
          <p class="mt-0.5 truncate text-xs text-gray-400">
            Clásico Java/CMD (plugins, mods y Geyser) o Motor Nativo Rust (~80 MB RAM, arranque en &lt;1s)
          </p>
        </div>
        <span class="shrink-0 text-xs font-bold text-sky-300">{{ guideOpen ? "Ocultar" : "Ver guía" }}</span>
      </button>
      <div v-if="guideOpen" class="border-t border-sky-500/20 px-4 py-4 text-sm text-gray-300">
        <ol class="list-decimal space-y-2.5 pl-5">
          <li>
            <span class="font-semibold text-white">Opción 1 · Clásico (Java / CMD + Playit.gg):</span>
            usa Paper, Fabric, Forge o NeoForge. Ideal para plugins, modpacks, redstone 100% vanilla, cualquier versión y entrada desde Bedrock con Geyser. Podés usar la consola integrada o abrir la ventana CMD clásica de Windows.
          </li>
          <li>
            <span class="font-semibold text-white">Opción 2 · Ultraligero Rust (Estilo Horizon / Pumpkin + Playit.gg):</span>
            binario nativo escrito en Rust sin máquina virtual Java. Consume ~80 MB de RAM y arranca en menos de 1 segundo, ideal para jugar en PCs de pocos recursos mientras tenés tu Minecraft abierto.
          </li>
          <li>
            <span class="font-semibold text-white">Invitá amigos sin abrir puertos:</span>
            ambos modos usan tu mismo túnel compartido de <span class="font-mono text-sky-300">Playit.gg (*.tun.ply.gg)</span> automáticamente al iniciar.
          </li>
        </ol>
      </div>
    </section>

    <div v-if="showCreate" class="mb-6 space-y-4 rounded-2xl border border-surface-3 bg-surface-2 p-5">
      <h2 class="text-sm font-semibold uppercase tracking-wider text-gray-400">Nuevo servidor · Elegí el motor</h2>

      <!-- Selector de las dos opciones principales -->
      <div class="grid gap-3 md:grid-cols-2">
        <button
          type="button"
          class="rounded-xl border-2 p-4 text-left transition"
          :class="
            engineMode === 'java-cmd'
              ? 'border-pc-green bg-pc-green/10'
              : 'border-surface-4 bg-surface-3/60 hover:border-surface-5'
          "
          @click="engineMode = 'java-cmd'"
        >
          <div class="flex items-center justify-between gap-2">
            <span class="text-sm font-bold text-white">🖥️ Opción 1 · Clásico (CMD / Java + Playit.gg)</span>
            <span
              class="rounded px-2 py-0.5 text-[10px] font-bold uppercase"
              :class="engineMode === 'java-cmd' ? 'bg-pc-green text-black' : 'bg-surface-4 text-gray-400'"
            >
              Completo
            </span>
          </div>
          <p class="mt-1.5 text-xs text-gray-400">
            Paper, Fabric, Forge, NeoForge y Geyser (Bedrock). Compatible con todas las versiones, mods, plugins y SkinsRestorer.
          </p>
        </button>

        <button
          type="button"
          class="rounded-xl border-2 p-4 text-left transition"
          :class="
            engineMode === 'rust-horizon'
              ? 'border-amber-400 bg-amber-500/10'
              : 'border-surface-4 bg-surface-3/60 hover:border-surface-5'
          "
          @click="engineMode = 'rust-horizon'"
        >
          <div class="flex items-center justify-between gap-2">
            <span class="text-sm font-bold text-white">⚡ Opción 2 · Ultraligero Rust (Horizon / Pumpkin + Playit.gg)</span>
            <span
              class="rounded px-2 py-0.5 text-[10px] font-bold uppercase"
              :class="engineMode === 'rust-horizon' ? 'bg-amber-400 text-black' : 'bg-amber-500/20 text-amber-300'"
            >
              ~80 MB RAM
            </span>
          </div>
          <p class="mt-1.5 text-xs text-gray-400">
            Cero Java · Arranca en &lt;1s · Motor nativo en Rust ideal para jugar con amigos sin consumir la RAM de tu PC.
          </p>
        </button>
      </div>

      <p v-if="versionsLoading" class="text-xs text-gray-500">Cargando versiones de Minecraft…</p>

      <!-- Subtipos solo para Modo Clásico Java -->
      <div v-if="engineMode === 'java-cmd'" class="flex flex-wrap gap-2">
        <button
          v-for="t in JAVA_SERVER_TYPES"
          :key="t.id"
          type="button"
          class="flex min-w-[105px] flex-col items-center gap-0.5 rounded-lg border-2 px-3 py-2 text-xs font-bold transition"
          :class="
            newType === t.id
              ? 'border-pc-green bg-pc-green/10 text-pc-green'
              : 'border-surface-4 bg-surface-3 text-gray-300 hover:border-surface-6'
          "
          @click="newType = t.id"
        >
          <span class="text-base">{{ t.icon }}</span>
          <span>{{ t.label }}</span>
        </button>
      </div>
      <div
        v-else
        class="rounded-xl border border-amber-500/30 bg-amber-950/20 px-4 py-3 text-xs text-amber-100"
      >
        <p class="font-semibold text-amber-300">⚡ Motor Nativo Rust (estilo Horizon / Pumpkin MC 26.3)</p>
        <p class="mt-0.5 text-amber-100/80">
          No necesita Java ni consume gigabytes de RAM. Usa el mismo túnel automático de Playit.gg para que tus amigos entren con la IP pública o localhost.
        </p>
      </div>

      <div class="flex flex-wrap items-end gap-3">
        <label class="text-sm">
          Nombre del servidor
          <input v-model="newName" class="mt-1 block rounded-lg border border-surface-5 bg-surface-3 px-3 py-2" />
        </label>
        <label v-if="engineMode === 'java-cmd'" class="text-sm">
          Versión MC
          <div class="mt-1 min-w-[10rem]">
            <AppSelect
              v-model="newMc"
              :options="
                releaseMcVersions.length
                  ? releaseMcVersions.map((v) => ({ value: v.id, label: v.id }))
                  : [{ value: newMc, label: newMc }]
              "
              searchable
              :max-panel-height="240"
            />
          </div>
        </label>
        <label v-if="engineMode === 'java-cmd'" class="text-sm">
          RAM asignada
          <div class="mt-1 min-w-[8rem]">
            <AppSelect
              :model-value="String(newRam)"
              :options="ramOptions.map((mb) => ({ value: String(mb), label: formatRamGb(mb) }))"
              @update:model-value="newRam = Number($event)"
            />
          </div>
        </label>
        <label class="flex cursor-pointer items-center gap-2 rounded-lg border border-surface-4 bg-surface-3 px-3 py-2.5 text-xs text-gray-300">
          <input v-model="newShowCmd" type="checkbox" class="accent-pc-green" />
          <span>Abrir ventana CMD de Windows al iniciar</span>
        </label>
        <BaseButton :disabled="busy === 'create'" @click="create">
          {{ busy === "create" ? "Creando…" : "Crear servidor" }}
        </BaseButton>
      </div>
    </div>

    <p v-if="error" class="mb-4 rounded-lg bg-red-500/10 px-4 py-2 text-sm text-red-400">{{ error }}</p>
    <p v-if="loading" class="text-sm text-gray-500">Cargando…</p>
    <p v-else-if="!serversStore.servers.length" class="text-sm text-gray-500">
      No hay servidores. Creá uno nuevo arriba o importá una carpeta existente.
    </p>

    <div class="grid gap-3 sm:grid-cols-2">
      <article
        v-for="s in serversStore.servers"
        :key="s.id"
        class="lunar-card cursor-pointer p-4 transition hover:border-pc-green/40"
        @click="openServer(s.id)"
        @contextmenu="onServerContext($event, s)"
      >
        <div class="flex items-center justify-between gap-2">
          <h3 class="truncate font-bold">{{ s.name }}</h3>
          <div class="flex shrink-0 items-center gap-1.5">
            <span
              v-if="s.serverType === 'rust-native'"
              class="rounded bg-amber-500/20 px-2 py-0.5 text-[10px] font-bold uppercase text-amber-300"
            >
              ⚡ Rust Nativo
            </span>
            <span
              v-if="s.showCmdWindow"
              class="rounded bg-surface-4 px-2 py-0.5 text-[10px] font-semibold text-gray-300"
            >
              🪟 CMD
            </span>
          </div>
        </div>
        <p class="mt-1 text-xs text-gray-500">
          MC {{ s.mcVersion }} · {{ serverTypeLabel(s.serverType) }} · {{ formatRamLabel(s) }}
          <span v-if="s.customFolder" class="text-amber-500/80"> · importado</span>
        </p>
        <p v-if="s.playitAddress" class="mt-1 truncate text-xs text-pc-green">Playit: {{ s.playitAddress }}</p>
      </article>
    </div>

    <ContextMenu
      v-if="menu"
      :x="menu.x"
      :y="menu.y"
      :items="menuItems"
      @close="menu = null"
      @select="onMenuSelect"
    />
  </div>
</template>
