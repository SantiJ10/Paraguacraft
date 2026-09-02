<script setup lang="ts">
defineOptions({ name: "server-detail" });
import { computed, onActivated, onDeactivated, onMounted, onUnmounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useServersStore } from "@/stores/servers";
import { useFavoritesStore } from "@/stores/favorites";
import { useAppStore } from "@/stores/app";
import { api, isTauri } from "@/lib/ipc";
import type { HangarPlugin, ServerContentItem, ServerRepairReport, ServerStatus } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";
import { contentFolderIcon } from "@/lib/contentIcons";

type TabId = "console" | "properties" | "plugins" | "admin" | "files";

const SERVER_RAM_PRESETS_MB = [2048, 4096, 6144, 8192, 12288, 16384] as const;

const PROP_FIELDS: Array<{ key: string; label: string; type?: "select"; options?: string[] }> = [
  { key: "motd", label: "MOTD" },
  { key: "max-players", label: "Máx. jugadores" },
  { key: "difficulty", label: "Dificultad", type: "select", options: ["peaceful", "easy", "normal", "hard"] },
  { key: "gamemode", label: "Modo de juego", type: "select", options: ["survival", "creative", "adventure", "spectator"] },
  { key: "pvp", label: "PvP", type: "select", options: ["true", "false"] },
  { key: "white-list", label: "Whitelist activa", type: "select", options: ["true", "false"] },
  { key: "online-mode", label: "Online mode", type: "select", options: ["true", "false"] },
  { key: "server-port", label: "Puerto" },
  { key: "level-name", label: "Nombre del mundo" },
  { key: "view-distance", label: "View distance" },
  { key: "spawn-protection", label: "Spawn protection" },
];

const route = useRoute();
const router = useRouter();
const serversStore = useServersStore();
const favoritesStore = useFavoritesStore();
const appStore = useAppStore();

const tab = ref<TabId>("console");
const editRamMb = ref(4096);
const status = ref<ServerStatus | null>(null);
const logLines = ref<string[]>([]);
const command = ref("");
const props = ref<Record<string, string>>({});
const content = ref<ServerContentItem[]>([]);
const folderPath = ref("");
const whitelist = ref<string[]>([]);
const ops = ref<string[]>([]);
const bans = ref<string[]>([]);
const hangarQuery = ref("");
const hangarResults = ref<HangarPlugin[]>([]);
const playitAddr = ref("");
const repairReport = ref<ServerRepairReport | null>(null);

const loading = ref(true);
const busy = ref(false);
const error = ref<string | null>(null);
const message = ref<string | null>(null);

const newWhitelist = ref("");
const newOp = ref("");
const newBan = ref("");

// --- Consola: jugador objetivo + historial (Fase 4.1) ---
const targetPlayer = ref("");
const commandHistory = ref<string[]>([]);
const HISTORY_KEY = "pc_server_console_history";
const HISTORY_MAX = 12;

function loadHistory() {
  try {
    const raw = localStorage.getItem(HISTORY_KEY);
    commandHistory.value = raw ? JSON.parse(raw) : [];
  } catch {
    commandHistory.value = [];
  }
}

function pushHistory(cmd: string) {
  const trimmed = cmd.trim();
  if (!trimmed) return;
  commandHistory.value = [trimmed, ...commandHistory.value.filter((c) => c !== trimmed)].slice(0, HISTORY_MAX);
  try {
    localStorage.setItem(HISTORY_KEY, JSON.stringify(commandHistory.value));
  } catch {
    // ignorar (localStorage lleno o deshabilitado)
  }
}

interface QuickCommand {
  label: string;
  build: (player: string) => string;
  needsPlayer?: boolean;
}

const quickCommands: QuickCommand[] = [
  { label: "Día", build: () => "time set day" },
  { label: "Noche", build: () => "time set night" },
  { label: "Clima despejado", build: () => "weather clear" },
  { label: "Dificultad: fácil", build: () => "difficulty easy" },
  { label: "Dificultad: normal", build: () => "difficulty normal" },
  { label: "Dificultad: difícil", build: () => "difficulty hard" },
  { label: "PvP on", build: () => "pvp true" },
  { label: "PvP off", build: () => "pvp false" },
  { label: "Guardar mundo", build: () => "save-all" },
];

const playerCommands: QuickCommand[] = [
  { label: "Whitelist add", build: (p) => `whitelist add ${p}`, needsPlayer: true },
  { label: "OP", build: (p) => `op ${p}`, needsPlayer: true },
  { label: "Modo survival", build: (p) => `gamemode survival ${p}`, needsPlayer: true },
  { label: "Modo creative", build: (p) => `gamemode creative ${p}`, needsPlayer: true },
  { label: "Kick", build: (p) => `kick ${p}`, needsPlayer: true },
  { label: "Curar", build: (p) => `effect give ${p} minecraft:instant_health 1 10`, needsPlayer: true },
];

const serverId = computed(() => String(route.params.id ?? ""));
const server = computed(
  () => serversStore.servers.find((s) => s.id === serverId.value) ?? null,
);
const isFabric = computed(() => server.value?.serverType.startsWith("fabric") ?? false);
const isNeoForge = computed(() => (server.value?.serverType ?? "").includes("neoforge"));
const isForge = computed(() => {
  const t = (server.value?.serverType ?? "").toLowerCase();
  return t.startsWith("forge") && !t.includes("neoforge");
});
/** Fabric / Forge / NeoForge — pueden importar modpacks. */
const isModpackServer = computed(() => isFabric.value || isForge.value || isNeoForge.value);
const isPaper = computed(() => !isModpackServer.value);

const ramOptions = computed(() => {
  const maxMb = appStore.hardware
    ? Math.floor(appStore.hardware.ramGb * 1024 * 0.75)
    : Infinity;
  return SERVER_RAM_PRESETS_MB.filter((mb) => mb <= maxMb);
});

function formatRamGb(mb: number): string {
  return `${mb / 1024} GB`;
}

function contentTitle(item: ServerContentItem): string {
  return item.displayName?.trim() || item.name.replace(/\.jar$/i, "");
}

function contentIcon(item: ServerContentItem): string {
  if (item.iconUrl) return item.iconUrl;
  return contentFolderIcon(item.kind === "plugins" ? "mods" : item.kind || "mods");
}

let pollTimer: ReturnType<typeof setInterval> | null = null;
let fastPollTimer: ReturnType<typeof setInterval> | null = null;
const consoleRef = ref<HTMLElement | null>(null);

const tabs = computed(() => {
  const base: Array<{ id: TabId; label: string }> = [
    { id: "console", label: "Consola" },
    { id: "properties", label: "Propiedades" },
    { id: "plugins", label: isModpackServer.value ? "Mods" : "Plugins" },
    { id: "admin", label: "Admin" },
    { id: "files", label: "Archivos" },
  ];
  return base;
});

async function refreshStatus() {
  if (!serverId.value) return;
  status.value = await api.serverStatus(serverId.value);
}

async function refreshLog() {
  if (!serverId.value) return;
  logLines.value = await api.getServerLog(serverId.value, 500);
}

async function loadTabData() {
  if (!server.value) return;
  error.value = null;
  try {
    if (tab.value === "properties") {
      props.value = await api.readServerProperties(serverId.value);
    } else if (tab.value === "plugins") {
      content.value = await api.listServerContent(serverId.value);
    } else if (tab.value === "admin") {
      [whitelist.value, ops.value, bans.value] = await Promise.all([
        api.serverWhitelistList(serverId.value),
        api.serverOpList(serverId.value),
        api.serverBanList(serverId.value),
      ]);
    } else if (tab.value === "files") {
      folderPath.value = await api.getServerFolderPath(serverId.value);
    }
  } catch (e) {
    error.value = String(e);
  }
}

/** Carga completa (primera vez / cambio de id). */
async function loadAll(opts?: { soft?: boolean }) {
  const soft = opts?.soft === true;
  if (!soft) loading.value = true;
  error.value = null;
  try {
    await serversStore.load(!soft && !serversStore.loaded);
    if (!server.value) {
      error.value = "Servidor no encontrado.";
      return;
    }
    serversStore.setLastActive(serverId.value);
    playitAddr.value = server.value.playitAddress ?? "";
    editRamMb.value = server.value.ramMb || 4096;
    if (!appStore.hardware) void appStore.loadHardware();
    await refreshStatus();
    if (tab.value === "console" || !soft) {
      await refreshLog();
    }
    if (!soft) await loadTabData();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function startPolling() {
  stopPolling();
  pollTimer = setInterval(async () => {
    await refreshStatus();
    if (tab.value === "console") await refreshLog();
  }, 2000);
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
  stopFastPolling();
}

function startFastPolling(durationMs = 15000) {
  stopFastPolling();
  fastPollTimer = setInterval(async () => {
    await refreshStatus();
    await refreshLog();
    scrollConsoleToBottom();
  }, 400);
  setTimeout(stopFastPolling, durationMs);
}

function stopFastPolling() {
  if (fastPollTimer) {
    clearInterval(fastPollTimer);
    fastPollTimer = null;
  }
}

function scrollConsoleToBottom() {
  const el = consoleRef.value;
  if (el) el.scrollTop = el.scrollHeight;
}

const logText = computed(() => logLines.value.join("\n"));

async function copyLog() {
  const text = logText.value;
  if (!text) {
    message.value = "No hay líneas en la consola para copiar.";
    return;
  }
  try {
    await navigator.clipboard.writeText(text);
    message.value = "Log copiado al portapapeles.";
  } catch {
    error.value = "No se pudo copiar. Seleccioná el texto manualmente (Ctrl+A en la consola).";
  }
}

async function exportLog() {
  if (!isTauri()) return;
  try {
    const path = await api.exportServerLog(serverId.value);
    message.value = `Log exportado: ${path}`;
  } catch (e) {
    error.value = String(e);
  }
}

async function openLatestLog() {
  try {
    await api.openServerFolder(serverId.value);
    message.value = "Abrí la carpeta del servidor → logs/latest.log";
  } catch (e) {
    error.value = String(e);
  }
}

onMounted(() => {
  loadHistory();
  void loadAll();
  startPolling();
});

// KeepAlive: al volver de Inicio/Ajustes el poller y la consola siguen vivos.
onActivated(() => {
  if (serverId.value) {
    serversStore.setLastActive(serverId.value);
    startPolling();
    void loadAll({ soft: true });
  }
});

onDeactivated(() => {
  stopPolling();
});

onUnmounted(stopPolling);

watch(serverId, (id, prev) => {
  if (!id || id === prev) return;
  tab.value = "console";
  logLines.value = [];
  status.value = null;
  void loadAll();
  startPolling();
});

watch(tab, () => {
  void loadTabData();
});

watch(
  () => serversStore.servers.find((s) => s.id === serverId.value)?.playitAddress,
  (addr) => {
    if (addr) playitAddr.value = addr;
  },
);

async function startServer() {
  busy.value = true;
  error.value = null;
  try {
    await api.startServer(serverId.value);
    startFastPolling();
    await refreshStatus();
    await refreshLog();
    scrollConsoleToBottom();
    message.value = "Servidor iniciado — revisá la consola si hay errores.";
  } catch (e) {
    error.value = String(e);
    await refreshLog();
  } finally {
    busy.value = false;
  }
}

async function stopServer() {
  busy.value = true;
  try {
    await api.stopServer(serverId.value);
    await refreshStatus();
    message.value = "Servidor detenido.";
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function prepareJar() {
  busy.value = true;
  try {
    await api.prepareServerJar(serverId.value);
    message.value = "Servidor preparado.";
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function sendCmd() {
  const cmd = command.value.trim();
  if (!cmd) return;
  try {
    await api.sendServerCommand(serverId.value, cmd);
    pushHistory(cmd);
    command.value = "";
    await refreshLog();
  } catch (e) {
    error.value = String(e);
  }
}

async function runQuickCommand(qc: QuickCommand) {
  if (qc.needsPlayer && !targetPlayer.value.trim()) {
    error.value = "Escribí el nombre del jugador arriba primero.";
    return;
  }
  const cmd = qc.build(targetPlayer.value.trim());
  try {
    await api.sendServerCommand(serverId.value, cmd);
    pushHistory(cmd);
    message.value = `Enviado: /${cmd}`;
    await refreshLog();
  } catch (e) {
    error.value = String(e);
  }
}

async function runHistoryCommand(cmd: string) {
  command.value = cmd;
  await sendCmd();
}

async function saveProps() {
  busy.value = true;
  try {
    await api.writeServerProperties(serverId.value, props.value);
    message.value = "Propiedades guardadas.";
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function saveRam() {
  if (!server.value) return;
  if (status.value?.running) {
    error.value = "Detené el servidor antes de cambiar la RAM.";
    return;
  }
  busy.value = true;
  error.value = null;
  try {
    await serversStore.update({ id: serverId.value, ramMb: editRamMb.value });
    message.value = `RAM del servidor: ${formatRamGb(editRamMb.value)}. Se aplica al reiniciar (JVM -Xmx).`;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function searchHangar() {
  if (!hangarQuery.value.trim() || isModpackServer.value) return;
  busy.value = true;
  try {
    hangarResults.value = await api.hangarSearchPlugins(hangarQuery.value.trim());
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function importMrpackToThisServer() {
  if (!serverId.value || status.value?.running) {
    error.value = "Detené el servidor antes de importar un modpack.";
    return;
  }
  busy.value = true;
  error.value = null;
  message.value = null;
  try {
    const prof = await api.pickAndImportMrpackToServer(serverId.value);
    message.value = `Modpack importado en «${prof.name}». Revisá la lista de mods e iniciá el servidor.`;
    content.value = await api.listServerContent(serverId.value);
  } catch (e) {
    const msg = String(e);
    if (!msg.toLowerCase().includes("no se seleccion")) {
      error.value = msg;
    }
  } finally {
    busy.value = false;
  }
}

function goStoreModpacks() {
  router.push({ name: "store", query: { tab: "modpack" } });
}

async function installPlugin(p: HangarPlugin) {
  busy.value = true;
  try {
    const name = await api.hangarInstallPlugin(serverId.value, p.owner, p.slug);
    message.value = `Instalado: ${name}`;
    content.value = await api.listServerContent(serverId.value);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function addWhitelist() {
  if (!newWhitelist.value.trim()) return;
  await api.serverWhitelistAdd(serverId.value, newWhitelist.value.trim());
  newWhitelist.value = "";
  whitelist.value = await api.serverWhitelistList(serverId.value);
}

async function addOp() {
  if (!newOp.value.trim()) return;
  await api.serverOpAdd(serverId.value, newOp.value.trim());
  newOp.value = "";
  ops.value = await api.serverOpList(serverId.value);
}

async function addBan() {
  if (!newBan.value.trim()) return;
  await api.serverBanAdd(serverId.value, newBan.value.trim());
  newBan.value = "";
  bans.value = await api.serverBanList(serverId.value);
}

async function startPlayit() {
  busy.value = true;
  try {
    const msg = await api.startPlayit(serverId.value);
    message.value = msg;
    startFastPolling(30000);
    await refreshStatus();
    await refreshLog();
    scrollConsoleToBottom();
    await serversStore.load(true);
    playitAddr.value = serversStore.servers.find((s) => s.id === serverId.value)?.playitAddress ?? playitAddr.value;
  } catch (e) {
    error.value = String(e);
    await refreshLog();
  } finally {
    busy.value = false;
  }
}

async function stopPlayit() {
  await api.stopPlayit(serverId.value);
  await refreshStatus();
}

async function resetPlayit() {
  busy.value = true;
  error.value = null;
  try {
    const msg = await api.resetPlayit(serverId.value);
    message.value = msg;
    await refreshStatus();
    await refreshLog();
    scrollConsoleToBottom();
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function resetPlayitFull() {
  if (
    !confirm(
      "Esto borra el secret compartido de TODOS los servers y hay que hacer claim de nuevo (1 sola vez). ¿Seguro?",
    )
  ) {
    return;
  }
  busy.value = true;
  error.value = null;
  try {
    const msg = await api.resetPlayitFull(serverId.value);
    message.value = msg;
    await refreshStatus();
    await refreshLog();
    scrollConsoleToBottom();
    await serversStore.load(true);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function savePlayitAddr() {
  await api.setPlayitAddress(serverId.value, playitAddr.value);
  message.value = "Dirección Playit guardada.";
  await serversStore.load(true);
}

// --- Asistente Playit primera vez (Fase 1.3) ---
const isGeyser = computed(() => server.value?.serverType.includes("geyser") ?? false);
const effectiveAddress = computed(() => status.value?.playitAddress ?? playitAddr.value);
const addressHost = computed(() => effectiveAddress.value.split(":")[0] ?? "");
/** Dirección Bedrock real (puede traer puerto público distinto de 19132). */
const bedrockAddress = computed(() => {
  const fromApi = status.value?.playitBedrockAddress?.trim();
  if (fromApi) return fromApi;
  // Fallback legacy: host Java:19132 (solo si hay túnel UDP propio con mismo host — poco frecuente)
  return addressHost.value ? `${addressHost.value}:19132` : "";
});
const bedrockHostPort = computed(() => {
  const a = bedrockAddress.value;
  if (!a) return { host: "", port: "19132" };
  const idx = a.lastIndexOf(":");
  if (idx > 0 && a.slice(idx + 1).split("").every((c) => c >= "0" && c <= "9")) {
    return { host: a.slice(0, idx), port: a.slice(idx + 1) };
  }
  return { host: a, port: "19132" };
});

const claimLink = computed(() => {
  const hint = status.value?.playitClaimHint ?? "";
  const match = hint.match(/https?:\/\/\S*playit\.gg\/claim\S*/i);
  return match ? match[0] : null;
});

function bedrockInstructions(): string {
  const name = server.value?.name ?? "el servidor";
  const { host, port } = bedrockHostPort.value;
  return [
    `¡Unite a ${name}!`,
    `- Java (PC): agregá el servidor con la IP ${effectiveAddress.value}`,
    `- Bedrock: Dirección ${host} · Puerto ${port}`,
  ].join("\n");
}

async function copyBedrockAddress() {
  if (!bedrockAddress.value) return;
  await copyText(
    bedrockAddress.value,
    status.value?.playitBedrockAddress
      ? "IP Bedrock (túnel Playit) copiada."
      : "IP copiada. Si no entra, esperá a que el launcher cree el túnel Bedrock o reiniciá el server Geyser.",
  );
}

async function copyBedrockInstructions() {
  await copyText(bedrockInstructions(), "Instrucciones copiadas — pegalas en el chat con tu amigo.");
}

async function copyText(text: string, okMessage: string) {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    message.value = okMessage;
  } catch {
    error.value = "No se pudo copiar al portapapeles.";
  }
}

async function confirmPlayitClaimed() {
  try {
    await api.markPlayitClaimed(serverId.value);
    await refreshStatus();
    message.value = "¡Listo! Cuenta Playit vinculada.";
  } catch (e) {
    error.value = String(e);
  }
}

async function addThisServerToFavorites() {
  if (!server.value) return;
  try {
    await favoritesStore.addFromServer(serverId.value, isGeyser.value ? 19132 : undefined);
    message.value = `«${server.value.name}» agregado a favoritos.`;
  } catch (e) {
    error.value = String(e);
  }
}

async function repairServer() {
  if (!isTauri()) return;
  busy.value = true;
  error.value = null;
  repairReport.value = null;
  try {
    repairReport.value = await api.repairServer(serverId.value);
    message.value =
      repairReport.value.fixedCount > 0
        ? `Reparación completada: ${repairReport.value.fixedCount} corrección(es).`
        : "Análisis completado. Revisá los avisos abajo.";
    await loadTabData();
    tab.value = "files";
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

function repairSeverityClass(severity: string) {
  if (severity === "fixed") return "border-pc-green/40 bg-pc-green/10 text-pc-green";
  if (severity === "warning") return "border-amber-500/40 bg-amber-500/10 text-amber-200";
  if (severity === "error") return "border-red-500/40 bg-red-500/10 text-red-300";
  return "border-surface-4 bg-surface-3 text-gray-400";
}

async function backupWorld() {
  busy.value = true;
  try {
    const r = await api.serverBackupWorlds(serverId.value);
    message.value = `Backup creado (${r.sizeMb.toFixed(1)} MB): ${r.path}`;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function serverTypeLabel(t: string) {
  const map: Record<string, string> = {
    paper: "Paper",
    "paper-geyser": "Paper + Geyser",
    fabric: "Fabric",
    "fabric-geyser": "Fabric + Geyser",
    forge: "Forge",
    neoforge: "NeoForge",
  };
  return map[t] ?? t;
}
</script>

<template>
  <div class="mx-auto max-w-5xl p-8">
    <button
      class="mb-4 text-sm text-gray-500 transition hover:text-white"
      type="button"
      @click="router.push({ name: 'servers' })"
    >
      ← Todos los servidores
    </button>

    <div v-if="loading" class="py-20 text-center text-gray-500">Cargando servidor…</div>

    <template v-else-if="server">
      <header class="mb-6 flex flex-wrap items-start gap-4">
        <div class="flex h-16 w-16 items-center justify-center rounded-xl bg-surface-4 text-3xl">🖥️</div>
        <div class="min-w-0 flex-1">
          <h1 class="text-2xl font-bold">{{ server.name }}</h1>
          <p class="text-sm text-gray-400">
            Minecraft {{ server.mcVersion }} · {{ serverTypeLabel(server.serverType) }} ·
            {{ (server.ramMb / 1024).toFixed(0) }} GB · puerto {{ server.port }}
          </p>
          <p class="text-xs" :class="status?.running ? 'text-pc-green' : 'text-gray-500'">
            {{ status?.running ? `En ejecución (PID ${status?.pid ?? "?"})` : "Detenido" }}
            <span v-if="status?.playitRunning"> · túnel Playit activo</span>
          </p>
          <p v-if="playitAddr || status?.playitAddress" class="mt-1 text-xs text-pc-green">
            Playit: {{ status?.playitAddress ?? playitAddr }}
          </p>
        </div>
        <div class="flex flex-wrap gap-2">
          <BaseButton size="lg" :disabled="busy || status?.running" @click="startServer">
            {{ busy ? "…" : "Iniciar" }}
          </BaseButton>
          <BaseButton size="lg" variant="secondary" :disabled="busy || !status?.running" @click="stopServer">
            {{ busy && status?.running ? "Deteniendo…" : "Detener" }}
          </BaseButton>
          <BaseButton variant="secondary" :disabled="busy" @click="prepareJar">Preparar</BaseButton>
          <BaseButton
            variant="secondary"
            :disabled="busy || status?.running || !isTauri()"
            :title="status?.running ? 'Detené el servidor primero' : 'JARs corruptos, caché Paper, ViaVersion…'"
            @click="repairServer"
          >
            Reparar
          </BaseButton>
        </div>
      </header>

      <p v-if="error" class="mb-4 text-sm text-red-400">{{ error }}</p>
      <p v-if="message" class="mb-4 text-sm text-pc-green">{{ message }}</p>

      <div
        v-if="repairReport?.items.length"
        class="mb-4 space-y-2 rounded-xl border border-surface-4 bg-surface-2 p-4"
      >
        <h2 class="text-sm font-bold">
          Reparación del servidor
          <span class="ml-2 font-normal text-gray-500">
            {{ repairReport.fixedCount }} arreglado(s) · {{ repairReport.warningCount }} aviso(s)
          </span>
        </h2>
        <ul class="max-h-48 space-y-2 overflow-y-auto text-sm">
          <li
            v-for="(item, i) in repairReport.items"
            :key="i"
            class="rounded-lg border px-3 py-2"
            :class="repairSeverityClass(item.severity)"
          >
            <p class="font-semibold">{{ item.title }}</p>
            <p class="mt-0.5 text-xs opacity-90">{{ item.detail }}</p>
            <p v-if="item.path" class="mt-1 truncate font-mono text-[10px] opacity-70">{{ item.path }}</p>
          </li>
        </ul>
      </div>

      <div class="mb-6 flex gap-1 rounded-xl bg-surface-2 p-1">
        <button
          v-for="t in tabs"
          :key="t.id"
          class="flex-1 rounded-lg px-3 py-2 text-sm font-semibold transition-colors"
          :class="tab === t.id ? 'bg-pc-green text-black' : 'text-gray-400 hover:text-white'"
          @click="tab = t.id"
        >
          {{ t.label }}
        </button>
      </div>

      <!-- Consola -->
      <section v-if="tab === 'console'" class="space-y-3">
        <div class="flex flex-wrap gap-2">
          <BaseButton size="sm" variant="secondary" @click="copyLog">Copiar log</BaseButton>
          <BaseButton size="sm" variant="secondary" :disabled="!isTauri()" @click="exportLog">Exportar .log</BaseButton>
          <BaseButton size="sm" variant="ghost" @click="openLatestLog">Abrir carpeta logs/</BaseButton>
        </div>
        <div
          ref="consoleRef"
          class="h-80 select-text overflow-y-auto rounded-xl border border-surface-4 bg-black/40 p-3 font-mono text-xs leading-relaxed text-gray-300"
        >
          <p v-for="(line, i) in logLines" :key="i" class="whitespace-pre-wrap break-all">{{ line }}</p>
          <p v-if="!logLines.length" class="text-gray-600">
            Sin salida todavía. Usá «Preparar» si falta server.jar, luego «Iniciar». Los errores de Java aparecen aquí.
          </p>
        </div>
        <div class="flex gap-2">
          <input
            v-model="command"
            type="text"
            placeholder="Comando (ej: say Hola, op Steve, whitelist add Steve)"
            class="min-w-0 flex-1 rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
            @keyup.enter="sendCmd"
          />
          <BaseButton :disabled="!status?.running" @click="sendCmd">Enviar</BaseButton>
        </div>

        <div v-if="commandHistory.length" class="flex flex-wrap gap-1.5">
          <span class="mt-1 text-xs text-gray-500">Recientes:</span>
          <button
            v-for="(h, i) in commandHistory"
            :key="i"
            class="rounded-full border border-surface-5 bg-surface-3 px-2.5 py-0.5 font-mono text-xs text-gray-300 transition hover:border-pc-green hover:text-white"
            :disabled="!status?.running"
            @click="runHistoryCommand(h)"
          >
            /{{ h }}
          </button>
        </div>

        <!-- Acciones rápidas (Fase 4.1) -->
        <div class="rounded-xl border border-surface-4 bg-surface-2 p-4">
          <h3 class="mb-3 text-sm font-bold">Acciones rápidas</h3>
          <div class="mb-3 flex flex-wrap gap-2">
            <BaseButton
              v-for="qc in quickCommands"
              :key="qc.label"
              size="sm"
              variant="secondary"
              :disabled="!status?.running"
              @click="runQuickCommand(qc)"
            >
              {{ qc.label }}
            </BaseButton>
          </div>
          <label class="mb-2 block text-sm">
            <span class="mb-1 block text-gray-400">Jugador objetivo (para las acciones de abajo)</span>
            <input
              v-model="targetPlayer"
              placeholder="Nombre del jugador"
              class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
            />
          </label>
          <div class="flex flex-wrap gap-2">
            <BaseButton
              v-for="qc in playerCommands"
              :key="qc.label"
              size="sm"
              variant="secondary"
              :disabled="!status?.running"
              @click="runQuickCommand(qc)"
            >
              {{ qc.label }}
            </BaseButton>
          </div>
        </div>

        <div class="flex flex-wrap gap-2">
          <BaseButton
            size="sm"
            variant="secondary"
            :disabled="!isTauri()"
            title="Túnel independiente (playit.exe). El plugin de Minecraft queda desactivado."
            @click="startPlayit"
          >
            Playit.gg
          </BaseButton>
          <BaseButton size="sm" variant="ghost" :disabled="!status?.playitRunning" @click="stopPlayit">
            Detener Playit
          </BaseButton>
          <BaseButton size="sm" variant="ghost" :disabled="!isTauri()" @click="resetPlayit">
            Restaurar secret compartido
          </BaseButton>
          <BaseButton size="sm" variant="ghost" :disabled="!isTauri()" @click="resetPlayitFull">
            Reseteo total Playit
          </BaseButton>
        </div>

        <!-- Asistente Playit primera vez -->
        <div class="rounded-xl border border-surface-4 bg-surface-2 p-4">
          <h3 class="mb-3 text-sm font-bold">Asistente Playit — jugar con amigos</h3>
          <p class="mb-3 text-xs text-gray-400">
            Un claim sirve para <strong class="text-gray-300">todos tus servers</strong> del launcher (misma
            IP). Solo un server a la vez. Mundos distintos pueden ser carpetas distintas o el mismo Paper.
          </p>
          <p v-if="status?.playitPluginMode" class="mb-3 text-xs text-amber-200/90">
            Modo plugin (Paper): iniciá el servidor. Si es la primera vez, abrí
            <code class="text-pc-green">playit.gg/claim/…</code> de la consola. La IP
            <code class="text-pc-green">*.tun.ply.gg</code> se guarda y se reutiliza. Si el cupo free está
            lleno, borrá agentes en
            <a
              href="https://playit.gg/account/agents"
              target="_blank"
              rel="noopener"
              class="text-pc-green underline"
              >playit.gg/account/agents</a
            >
            y usá «Reseteo total Playit».
          </p>
          <ol class="space-y-3 text-sm">
            <li class="flex items-start gap-2">
              <span
                :class="
                  status?.playitPluginMode
                    ? status?.running
                      ? 'text-pc-green'
                      : 'text-gray-500'
                    : status?.playitRunning
                      ? 'text-pc-green'
                      : 'text-gray-500'
                "
              >
                {{
                  status?.playitPluginMode
                    ? status?.running
                      ? "✅"
                      : "①"
                    : status?.playitRunning
                      ? "✅"
                      : "①"
                }}
              </span>
              <div>
                <p class="font-semibold">{{ status?.playitPluginMode ? "Servidor (plugin) activo" : "Túnel iniciado" }}</p>
                <p class="text-xs text-gray-500">
                  <template v-if="status?.playitPluginMode">
                    {{
                      status?.running
                        ? "Paper + playit-gg corriendo; el claim sale en la consola."
                        : "Apretá Iniciar servidor — no hace falta el botón «Playit.gg»."
                    }}
                  </template>
                  <template v-else>
                    {{
                      status?.playitRunning
                        ? "El agente playit.exe está corriendo."
                        : "Apretá «Playit.gg» arriba para iniciarlo."
                    }}
                  </template>
                </p>
              </div>
            </li>
            <li class="flex items-start gap-2">
              <span :class="status?.playitClaimed ? 'text-pc-green' : 'text-gray-500'">
                {{ status?.playitClaimed ? "✅" : "②" }}
              </span>
              <div class="min-w-0 flex-1">
                <p class="font-semibold">Vincular cuenta playit.gg (para que el túnel no expire)</p>
                <p v-if="!status?.playitClaimed && claimLink" class="mt-1 text-xs">
                  <a :href="claimLink" target="_blank" class="break-all text-pc-green underline">{{ claimLink }}</a>
                </p>
                <p v-else-if="!status?.playitClaimed" class="text-xs text-gray-500">
                  El link de vinculación aparece en la consola al iniciar el túnel.
                </p>
                <BaseButton
                  v-if="!status?.playitClaimed"
                  size="sm"
                  variant="ghost"
                  class="mt-1"
                  @click="confirmPlayitClaimed"
                >
                  Ya vinculé mi cuenta
                </BaseButton>
              </div>
            </li>
            <li class="flex items-start gap-2">
              <span :class="effectiveAddress ? 'text-pc-green' : 'text-gray-500'">
                {{ effectiveAddress ? "✅" : "③" }}
              </span>
              <div class="min-w-0 flex-1">
                <p class="font-semibold">Dirección lista para compartir</p>
                <div v-if="effectiveAddress" class="mt-1 flex flex-wrap items-center gap-2">
                  <code class="rounded bg-black/40 px-2 py-1 text-xs text-pc-green">{{ effectiveAddress }}</code>
                  <BaseButton size="sm" variant="ghost" @click="copyText(effectiveAddress, 'IP Java copiada.')">
                    Copiar IP Java
                  </BaseButton>
                  <BaseButton
                    v-if="isGeyser"
                    size="sm"
                    variant="ghost"
                    @click="copyBedrockAddress"
                  >
                    Copiar IP Bedrock
                  </BaseButton>
                  <BaseButton
                    v-if="isGeyser"
                    size="sm"
                    variant="ghost"
                    @click="copyBedrockInstructions"
                  >
                    Copiar instrucciones para amigo Bedrock
                  </BaseButton>
                  <p v-if="isGeyser && status?.playitBedrockAddress" class="mt-1 w-full text-[11px] leading-snug text-pc-green/90">
                    Túnel Bedrock: <code class="text-pc-green">{{ status.playitBedrockAddress }}</code>
                    (host + puerto exactos en el cliente).
                  </p>
                  <p v-else-if="isGeyser" class="mt-1 w-full text-[11px] leading-snug text-amber-200/90">
                    Al iniciar el server Geyser el launcher crea el túnel Bedrock (y el Java si faltaba) con tu secret playit.
                    Si ya claimaste Java antes, solo agrega Bedrock. Hace falta
                    <strong class="font-semibold">playit.exe</strong>
                    (no el plugin Paper) para el tráfico UDP.
                  </p>
                  <p v-if="isGeyser" class="mt-2 w-full text-[11px] leading-snug text-gray-400">
                    Skin del celu en Java: Floodgate la sube solo si en Bedrock usás una skin
                    <strong class="font-semibold text-gray-300">clásica 64×64</strong>
                    (PNG importado), no el creador de personajes. SkinsRestorer no traduce skins Bedrock;
                    si Java ve Steve, esperá unos segundos o reconectá.
                  </p>
                  <BaseButton size="sm" variant="secondary" @click="addThisServerToFavorites">
                    + Agregar a favoritos
                  </BaseButton>
                </div>
                <p v-else class="text-xs text-gray-500">Esperando a que playit asigne una dirección…</p>
              </div>
            </li>
          </ol>
        </div>

        <div class="flex flex-wrap items-end gap-2">
          <label class="flex-1 text-sm">
            <span class="mb-1 block text-gray-400">Dirección Java (manual)</span>
            <input
              v-model="playitAddr"
              class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm"
              placeholder="ejemplo.joinmc.link:25565"
            />
          </label>
          <BaseButton size="sm" variant="secondary" @click="savePlayitAddr">Guardar</BaseButton>
        </div>
      </section>

      <!-- Propiedades -->
      <section v-else-if="tab === 'properties'" class="space-y-4">
        <div class="rounded-xl border border-surface-4 bg-surface-2 p-6">
          <h2 class="mb-1 text-lg font-bold">Memoria (RAM)</h2>
          <p class="mb-4 text-sm text-gray-500">
            Se guarda en el perfil del servidor y se aplica al iniciar (JVM
            <code class="text-gray-400">-Xmx</code>). No afecta un server ya en ejecución.
          </p>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="mb in ramOptions"
              :key="mb"
              type="button"
              class="rounded-lg border px-3 py-2 text-sm font-semibold transition"
              :class="
                editRamMb === mb
                  ? 'border-pc-green bg-pc-green/15 text-pc-green'
                  : 'border-surface-5 bg-surface-3 text-gray-300 hover:border-surface-5 hover:text-white'
              "
              :disabled="busy || !!status?.running"
              @click="editRamMb = mb"
            >
              {{ formatRamGb(mb) }}
            </button>
          </div>
          <BaseButton
            class="mt-4"
            :disabled="busy || !!status?.running || editRamMb === server.ramMb"
            @click="saveRam"
          >
            Guardar RAM
          </BaseButton>
        </div>

        <div class="rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-4 text-lg font-bold">server.properties</h2>
        <div class="grid gap-4 sm:grid-cols-2">
          <label v-for="f in PROP_FIELDS" :key="f.key" class="block text-sm">
            <span class="mb-1 block text-gray-400">{{ f.label }}</span>
            <select
              v-if="f.type === 'select'"
              v-model="props[f.key]"
              class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 outline-none focus:border-pc-green"
            >
              <option v-for="o in f.options" :key="o" :value="o">{{ o }}</option>
            </select>
            <input
              v-else
              v-model="props[f.key]"
              class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 outline-none focus:border-pc-green"
            />
          </label>
        </div>
        <BaseButton class="mt-6" :disabled="busy" @click="saveProps">Guardar propiedades</BaseButton>
        </div>
      </section>

      <!-- Plugins / Hangar / Modpacks -->
      <section v-else-if="tab === 'plugins'" class="space-y-4">
        <div v-if="isPaper" class="rounded-xl border border-surface-4 bg-surface-2 p-4">
          <h3 class="mb-2 font-bold">Buscar en Hangar</h3>
          <div class="flex gap-2">
            <input
              v-model="hangarQuery"
              placeholder="Essentials, LuckPerms, ViaVersion…"
              class="min-w-0 flex-1 rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm"
              @keyup.enter="searchHangar"
            />
            <BaseButton :disabled="busy" @click="searchHangar">Buscar</BaseButton>
          </div>
          <ul v-if="hangarResults.length" class="mt-3 divide-y divide-surface-3">
            <li v-for="p in hangarResults" :key="`${p.owner}/${p.slug}`" class="flex items-center gap-3 py-2">
              <div class="min-w-0 flex-1">
                <p class="font-medium">{{ p.name }}</p>
                <p class="truncate text-xs text-gray-500">{{ p.description }}</p>
              </div>
              <BaseButton size="sm" :disabled="busy" @click="installPlugin(p)">Instalar</BaseButton>
            </li>
          </ul>
        </div>

        <div v-if="isModpackServer" class="rounded-xl border border-surface-4 bg-surface-2 p-4">
          <h3 class="mb-1 font-bold">Modpack (Fabric / Forge / NeoForge)</h3>
          <p class="mb-3 text-sm text-gray-500">
            Importá un .mrpack de Modrinth (COBBLEVERSE, Cobblemon, etc.) a este servidor, o buscá el pack en la Tienda
            y elegí «Usar servidor existente».
          </p>
          <div class="flex flex-wrap gap-2">
            <BaseButton
              :disabled="busy || !!status?.running"
              :title="status?.running ? 'Detené el servidor primero' : undefined"
              @click="importMrpackToThisServer"
            >
              {{ busy ? "Importando…" : "Importar .mrpack…" }}
            </BaseButton>
            <BaseButton variant="secondary" :disabled="busy" @click="goStoreModpacks">
              Buscar pack en Tienda
            </BaseButton>
            <BaseButton
              variant="ghost"
              size="sm"
              :disabled="busy"
              @click="content = []; loadTabData()"
            >
              Actualizar lista
            </BaseButton>
          </div>
        </div>
        <p v-else-if="!isPaper" class="text-sm text-gray-500">
          Para mods usá la tienda del launcher o arrastrá JARs a la carpeta mods/.
        </p>

        <div class="rounded-xl border border-surface-4 bg-surface-2">
          <h3 class="border-b border-surface-3 px-4 py-2 text-xs font-bold uppercase tracking-wider text-gray-500">
            Instalados ({{ content.length }})
          </h3>
          <ul v-if="content.length" class="divide-y divide-surface-3">
            <li
              v-for="item in content"
              :key="item.path"
              class="flex items-start gap-3 px-4 py-3"
            >
              <img
                :src="contentIcon(item)"
                alt=""
                class="mt-0.5 h-12 w-12 shrink-0 rounded-lg bg-surface-3 object-cover"
              />
              <div class="min-w-0 flex-1">
                <div class="flex flex-wrap items-center gap-2">
                  <p class="truncate font-semibold text-white">{{ contentTitle(item) }}</p>
                  <span
                    v-if="item.compatible === false"
                    class="rounded bg-amber-500/15 px-1.5 py-0.5 text-[10px] font-bold uppercase text-amber-300"
                    :title="item.compatMessage ?? 'Incompatible'"
                  >
                    Incompatible
                  </span>
                  <span
                    v-else
                    class="rounded bg-pc-green/15 px-1.5 py-0.5 text-[10px] font-bold uppercase text-pc-green"
                  >
                    Activo
                  </span>
                </div>
                <p v-if="item.author" class="text-xs text-gray-500">por {{ item.author }}</p>
                <p v-if="item.description" class="mt-1 line-clamp-2 text-xs text-gray-400">
                  {{ item.description }}
                </p>
                <p class="mt-1 text-xs text-gray-600">{{ item.name }}</p>
              </div>
              <span class="shrink-0 text-sm text-gray-500">{{ formatSize(item.sizeBytes) }}</span>
            </li>
          </ul>
          <p v-else class="px-4 py-8 text-center text-gray-500">No hay plugins/mods instalados.</p>
        </div>
      </section>

      <!-- Admin -->
      <section v-else-if="tab === 'admin'" class="grid gap-4 md:grid-cols-3">
        <div class="rounded-xl border border-surface-4 bg-surface-2 p-4">
          <h3 class="mb-3 font-bold">Whitelist</h3>
          <div class="mb-2 flex gap-2">
            <input v-model="newWhitelist" placeholder="Jugador" class="flex-1 rounded-lg border border-surface-5 bg-surface-3 px-2 py-1 text-sm" />
            <BaseButton size="sm" @click="addWhitelist">+</BaseButton>
          </div>
          <ul class="max-h-40 space-y-1 overflow-y-auto text-sm">
            <li v-for="n in whitelist" :key="n" class="flex justify-between">
              <span>{{ n }}</span>
              <button class="text-red-400" @click="api.serverWhitelistRemove(serverId, n).then(() => loadTabData())">×</button>
            </li>
          </ul>
        </div>
        <div class="rounded-xl border border-surface-4 bg-surface-2 p-4">
          <h3 class="mb-3 font-bold">OPs</h3>
          <div class="mb-2 flex gap-2">
            <input v-model="newOp" placeholder="Jugador" class="flex-1 rounded-lg border border-surface-5 bg-surface-3 px-2 py-1 text-sm" />
            <BaseButton size="sm" @click="addOp">+</BaseButton>
          </div>
          <ul class="max-h-40 space-y-1 overflow-y-auto text-sm">
            <li v-for="n in ops" :key="n" class="flex justify-between">
              <span>{{ n }}</span>
              <button class="text-red-400" @click="api.serverOpRemove(serverId, n).then(() => loadTabData())">×</button>
            </li>
          </ul>
        </div>
        <div class="rounded-xl border border-surface-4 bg-surface-2 p-4">
          <h3 class="mb-3 font-bold">Baneados</h3>
          <div class="mb-2 flex gap-2">
            <input v-model="newBan" placeholder="Jugador" class="flex-1 rounded-lg border border-surface-5 bg-surface-3 px-2 py-1 text-sm" />
            <BaseButton size="sm" @click="addBan">+</BaseButton>
          </div>
          <ul class="max-h-40 space-y-1 overflow-y-auto text-sm">
            <li v-for="n in bans" :key="n" class="flex justify-between">
              <span>{{ n }}</span>
              <button class="text-red-400" @click="api.serverBanRemove(serverId, n).then(() => loadTabData())">×</button>
            </li>
          </ul>
        </div>
      </section>

      <!-- Archivos -->
      <section v-else-if="tab === 'files'" class="rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-2 text-lg font-bold">Carpeta del servidor</h2>
        <p class="mb-4 break-all font-mono text-sm text-gray-400">{{ folderPath }}</p>
        <div class="flex flex-wrap gap-2">
          <BaseButton @click="api.openServerFolder(serverId)">Abrir carpeta</BaseButton>
          <BaseButton variant="secondary" :disabled="busy" @click="backupWorld">Backup del mundo</BaseButton>
          <BaseButton
            variant="secondary"
            :disabled="busy || status?.running || !isTauri()"
            @click="repairServer"
          >
            Reparar servidor
          </BaseButton>
        </div>
        <p class="mt-4 text-xs text-gray-500">
          Reparar mueve JARs corruptos a <code class="text-gray-400">plugins/.paraguacraft-broken/</code>,
          limpia la caché Paper y analiza <code class="text-gray-400">logs/latest.log</code>.
        </p>
      </section>
    </template>

    <p v-else class="py-20 text-center text-gray-500">{{ error ?? "Servidor no encontrado." }}</p>
  </div>
</template>
