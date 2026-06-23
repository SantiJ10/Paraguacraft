<script setup lang="ts">
import { computed, ref, watch } from "vue";
import BaseButton from "@/components/common/BaseButton.vue";
import { CONTENT_FOLDERS, contentFolderIcon, contentFolderLabel } from "@/lib/contentIcons";
import { loaderIconSrc } from "@/lib/loaderIcons";
import { api } from "@/lib/ipc";
import type { GcType, InstanceContentItem, InstanceMeta, LoaderInfo } from "@/lib/types";
import { useAppStore } from "@/stores/app";

type SettingsTab = "content" | "files" | "java" | "versions";
type ContentTab = "mods" | "resourcepacks" | "shaderpacks" | "datapacks";

const props = defineProps<{
  open: boolean;
  instanceId: string | null;
  initialTab?: SettingsTab;
  mcVersion?: string;
}>();

const emit = defineEmits<{ close: [] }>();

const app = useAppStore();
const tab = ref<SettingsTab>("content");
const contentTab = ref<ContentTab>("mods");
const meta = ref<InstanceMeta | null>(null);
const content = ref<InstanceContentItem[]>([]);
const folderPath = ref("");
const loaders = ref<LoaderInfo[]>([]);
const loading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);
const message = ref<string | null>(null);

const ramMb = ref(4096);
const gc = ref<GcType>("Auto");
const jvmArgs = ref("");
const javaPath = ref("");
const loader = ref("vanilla");
const loaderVersion = ref("");
const gcOptions: GcType[] = ["Auto", "G1GC", "ZGC", "Shenandoah"];
const maxRam = computed(() => (app.hardware?.ramGb ?? 16) * 1024);

const nav: Array<{ id: SettingsTab; label: string; icon: string }> = [
  { id: "content", label: "Contenido", icon: "M3 7h5l2 2h11v10a2 2 0 01-2 2H5a2 2 0 01-2-2V7z" },
  { id: "files", label: "Archivos", icon: "M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V7z" },
  { id: "java", label: "Entorno Java", icon: "M8 9l-3 3 3 3M16 9l3 3-3 3M14 4l-4 16" },
  { id: "versions", label: "Versiones", icon: "M6 12h4v4H6v-4zM14 6h4v10h-4V6z" },
];

const filteredContent = computed(() =>
  content.value.filter((c) => c.folder === contentTab.value),
);

async function load() {
  if (!props.instanceId) return;
  loading.value = true;
  error.value = null;
  try {
    meta.value = await api.getInstanceMeta(props.instanceId);
    content.value = await api.listInstanceContent(props.instanceId);
    folderPath.value = await api.getInstanceFolderPath(props.instanceId);
    ramMb.value = meta.value.ramMb || 4096;
    gc.value = (meta.value.gc as GcType) ?? "Auto";
    jvmArgs.value = meta.value.jvmArgs ?? "";
    javaPath.value = meta.value.javaPath ?? "";
    loader.value = meta.value.loader;
    loaderVersion.value = meta.value.loaderVersion;
    loaders.value = await api.getLoaders(meta.value.mcVersion);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

watch(
  () => [props.open, props.instanceId] as const,
  ([open]) => {
    if (open && props.instanceId) {
      tab.value = props.initialTab ?? "content";
      void load();
    }
  },
  { immediate: true },
);

async function saveJava() {
  if (!props.instanceId) return;
  saving.value = true;
  error.value = null;
  try {
    meta.value = await api.setInstanceConfig({
      id: props.instanceId,
      ramMb: ramMb.value,
      jvmArgs: jvmArgs.value || null,
      gc: gc.value,
      javaPath: javaPath.value || null,
    });
    message.value = "Java guardado.";
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

async function saveLoader() {
  if (!props.instanceId || !meta.value) return;
  saving.value = true;
  error.value = null;
  try {
    if (loader.value !== meta.value.loader || loaderVersion.value !== meta.value.loaderVersion) {
      await api.setInstanceLoader(props.instanceId, loader.value, loaderVersion.value);
    }
    meta.value = await api.getInstanceMeta(props.instanceId);
    message.value = "Loader actualizado.";
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

async function toggleItem(item: InstanceContentItem) {
  if (!props.instanceId) return;
  await api.toggleInstanceContent(props.instanceId, item.path, !item.enabled);
  content.value = await api.listInstanceContent(props.instanceId);
}

async function openFolder() {
  if (!props.instanceId) return;
  await api.openInstanceFolder(props.instanceId);
}

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

const selectedLoaderInfo = computed(() => loaders.value.find((l) => l.id === loader.value));
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm"
      @click.self="emit('close')"
    >
      <div class="flex h-[min(720px,92vh)] w-full max-w-4xl overflow-hidden rounded-2xl border border-surface-4 bg-surface-1 shadow-2xl">
        <!-- Sidebar -->
        <aside class="flex w-52 shrink-0 flex-col border-r border-surface-3 bg-surface-0 p-3">
          <p class="mb-3 px-2 text-xs font-bold uppercase tracking-wider text-gray-500">Ajustes</p>
          <nav class="flex flex-1 flex-col gap-1">
            <button
              v-for="item in nav"
              :key="item.id"
              type="button"
              class="flex items-center gap-2 rounded-lg px-3 py-2.5 text-left text-sm font-semibold transition"
              :class="tab === item.id ? 'bg-surface-3 text-white' : 'text-gray-400 hover:bg-surface-2 hover:text-gray-200'"
              @click="tab = item.id"
            >
              <svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path :d="item.icon" />
              </svg>
              {{ item.label }}
            </button>
          </nav>
          <p class="px-2 text-[10px] leading-relaxed text-gray-600">
            No afiliado con Mojang o Microsoft.
          </p>
        </aside>

        <!-- Main -->
        <div class="flex min-w-0 flex-1 flex-col">
          <header class="flex items-center justify-between border-b border-surface-3 px-6 py-4">
            <div>
              <h2 class="text-lg font-bold">
                {{ tab === "content" ? "Contenido" : tab === "files" ? "Archivos" : tab === "java" ? "Entorno Java" : "Versiones" }}
              </h2>
              <p v-if="meta" class="text-xs text-gray-500">Minecraft {{ meta.mcVersion }} · {{ meta.name }}</p>
            </div>
            <button type="button" class="rounded-lg p-2 text-gray-400 hover:bg-surface-3 hover:text-white" @click="emit('close')">
              ✕
            </button>
          </header>

          <div class="flex-1 overflow-y-auto p-6">
            <p v-if="loading" class="text-sm text-gray-500">Cargando…</p>
            <p v-if="error" class="mb-4 text-sm text-red-400">{{ error }}</p>
            <p v-if="message" class="mb-4 text-sm text-pc-green">{{ message }}</p>

            <!-- Contenido: mods, packs, shaders, datapacks -->
            <section v-else-if="tab === 'content'">
              <div class="mb-4 flex flex-wrap gap-2">
                <button
                  v-for="folder in CONTENT_FOLDERS"
                  :key="folder.id"
                  type="button"
                  class="flex items-center gap-2 rounded-lg border px-3 py-2 text-sm font-semibold transition"
                  :class="contentTab === folder.id ? 'border-pc-green bg-pc-green/10 text-pc-green' : 'border-surface-4 text-gray-400 hover:border-surface-5'"
                  @click="contentTab = folder.id"
                >
                  <img :src="folder.icon" :alt="folder.label" class="h-5 w-5 object-contain" />
                  {{ folder.label }}
                </button>
              </div>

              <p v-if="!filteredContent.length" class="rounded-xl border border-dashed border-surface-4 py-16 text-center text-gray-500">
                No hay {{ contentFolderLabel(contentTab).toLowerCase() }} instalados.
              </p>
              <ul v-else class="divide-y divide-surface-3 rounded-xl border border-surface-4 bg-surface-2">
                <li
                  v-for="item in filteredContent"
                  :key="item.path"
                  class="flex items-center gap-3 px-4 py-3"
                  :class="!item.enabled ? 'opacity-50' : ''"
                >
                  <img :src="contentFolderIcon(item.folder)" alt="" class="h-8 w-8 object-contain" />
                  <div class="min-w-0 flex-1">
                    <p class="truncate font-medium">{{ item.name }}</p>
                    <p class="text-xs text-gray-500">{{ formatSize(item.sizeBytes) }}</p>
                  </div>
                  <button
                    type="button"
                    class="rounded-lg px-3 py-1 text-xs font-semibold transition"
                    :class="item.enabled ? 'bg-surface-4 text-gray-300' : 'bg-pc-green/20 text-pc-green'"
                    @click="toggleItem(item)"
                  >
                    {{ item.enabled ? "Desactivar" : "Activar" }}
                  </button>
                </li>
              </ul>
            </section>

            <!-- Archivos -->
            <section v-else-if="tab === 'files'" class="space-y-4">
              <p class="text-sm text-gray-400">Selecciona desde qué directorio iniciar Minecraft.</p>
              <div class="rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-100">
                Perfil aislado — mundos, mods y ajustes separados de otras instancias.
              </div>
              <div class="flex items-center gap-2 rounded-lg border border-surface-4 bg-surface-2 px-3 py-2">
                <code class="min-w-0 flex-1 truncate text-xs text-gray-300">{{ folderPath }}</code>
              </div>
              <BaseButton variant="secondary" @click="openFolder">Abrir carpeta</BaseButton>
            </section>

            <!-- Java -->
            <section v-else-if="tab === 'java'" class="space-y-4">
              <label class="block">
                <span class="mb-1 block text-sm text-gray-400">RAM (MB)</span>
                <input v-model.number="ramMb" type="range" min="1024" :max="maxRam" step="256" class="w-full" />
                <span class="text-sm font-semibold">{{ ramMb }} MB</span>
              </label>
              <label class="block">
                <span class="mb-1 block text-sm text-gray-400">Recolector GC</span>
                <select v-model="gc" class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm">
                  <option v-for="g in gcOptions" :key="g" :value="g">{{ g }}</option>
                </select>
              </label>
              <label class="block">
                <span class="mb-1 block text-sm text-gray-400">Java (opcional)</span>
                <input
                  v-model="javaPath"
                  type="text"
                  placeholder="Auto-detectar"
                  class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm"
                />
              </label>
              <label class="block">
                <span class="mb-1 block text-sm text-gray-400">JVM args extra</span>
                <input
                  v-model="jvmArgs"
                  type="text"
                  class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm font-mono"
                />
              </label>
              <BaseButton :disabled="saving" @click="saveJava">{{ saving ? "Guardando…" : "Guardar Java" }}</BaseButton>
            </section>

            <!-- Versiones / loader -->
            <section v-else-if="tab === 'versions'" class="space-y-5">
              <label class="block">
                <span class="mb-1 block text-sm text-gray-400">Versión del juego</span>
                <input
                  :value="meta?.mcVersion"
                  readonly
                  class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm text-gray-300"
                />
              </label>
              <div>
                <span class="mb-2 block text-sm text-gray-400">Cargador</span>
                <div class="grid grid-cols-4 gap-2">
                  <button
                    v-for="l in loaders"
                    :key="l.id"
                    type="button"
                    class="relative flex flex-col items-center gap-1 rounded-xl border p-2 transition"
                    :class="loader === l.id ? 'border-pc-green bg-pc-green/10' : 'border-surface-4 hover:border-surface-5'"
                    @click="loader = l.id"
                  >
                    <img :src="loaderIconSrc(l.id)" :alt="l.name" class="h-10 w-10 object-contain" />
                    <span class="text-center text-[10px] font-bold leading-tight">{{ l.name }}</span>
                    <span
                      v-if="loader === l.id"
                      class="absolute -right-1 -top-1 flex h-5 w-5 items-center justify-center rounded-full bg-pc-green text-[10px] text-black"
                    >✓</span>
                  </button>
                </div>
              </div>
              <label v-if="selectedLoaderInfo?.versions.length" class="block">
                <span class="mb-1 block text-sm text-gray-400">Versión del cargador</span>
                <select
                  v-model="loaderVersion"
                  class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm"
                >
                  <option v-for="ver in selectedLoaderInfo.versions" :key="ver" :value="ver">{{ ver }}</option>
                </select>
              </label>
              <BaseButton :disabled="saving" @click="saveLoader">{{ saving ? "Guardando…" : "Guardar loader" }}</BaseButton>
            </section>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
