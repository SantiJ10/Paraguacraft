<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useInstancesStore } from "@/stores/instances";
import { useAppStore } from "@/stores/app";
import { useDownloadsStore } from "@/stores/downloads";
import { api } from "@/lib/ipc";
import type { InstanceContentItem, InstanceMeta, GcType, Instance } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";
import BackupsModal from "@/components/instance/BackupsModal.vue";
import InstanceIcon from "@/components/instance/InstanceIcon.vue";
import InstanceIconPicker from "@/components/instance/InstanceIconPicker.vue";
import { resolveInstanceIcon } from "@/lib/instanceIcons";
import { formatPlaytime, formatRelative } from "@/composables/useFormat";

type TabId = "content" | "files" | "settings";

const route = useRoute();
const router = useRouter();
const instances = useInstancesStore();
const app = useAppStore();
const downloads = useDownloadsStore();

const tab = ref<TabId>("content");
const meta = ref<InstanceMeta | null>(null);
const content = ref<InstanceContentItem[]>([]);
const folderPath = ref("");
const loading = ref(true);
const error = ref<string | null>(null);
const message = ref<string | null>(null);
const launching = ref(false);
const updating = ref(false);
const showBackups = ref(false);

const instanceId = computed(() => String(route.params.id ?? ""));
const instance = computed(() => instances.instances.find((i) => i.id === instanceId.value) ?? null);
const isExternal = computed(() => instanceId.value.startsWith("ext::"));

/** Cabecera: lista o metadata (instancias legacy recién resueltas). */
const displayInstance = computed((): Instance | null => {
  if (instance.value) return instance.value;
  if (meta.value && !isExternal.value) {
    return {
      id: instanceId.value,
      name: meta.value.name,
      icon: meta.value.icon,
      mcVersion: meta.value.mcVersion,
      loader: meta.value.loader as Instance["loader"],
      loaderVersion: meta.value.loaderVersion,
      source: meta.value.source as Instance["source"],
      lastPlayed: meta.value.lastPlayed,
      totalPlayMinutes: meta.value.totalPlayMinutes,
      ramMb: meta.value.ramMb,
      modCount: 0,
    };
  }
  return null;
});

const tabs: Array<{ id: TabId; label: string }> = [
  { id: "content", label: "Contenido" },
  { id: "files", label: "Archivos" },
  { id: "settings", label: "Configuración" },
];

// ── Config form ──
const ramMb = ref(4096);
const gc = ref<GcType>("Auto");
const jvmArgs = ref("");
const javaPath = ref("");
const loader = ref("");
const loaderVersion = ref("");
const autoManaged = ref(true);
const configBusy = ref(false);
const appearanceBusy = ref(false);
const editName = ref("");
const editIcon = ref("");
const gcOptions: GcType[] = ["Auto", "G1GC", "ZGC", "Shenandoah"];
const maxRam = computed(() => (app.hardware?.ramGb ?? 16) * 1024);

async function loadAll() {
  loading.value = true;
  error.value = null;
  message.value = null;
  try {
    await instances.load(true);
    const id = instanceId.value;

    if (isExternal.value) {
      if (!instance.value) {
        await instances.scan();
      }
      if (!instance.value) {
        error.value = "Instancia no encontrada.";
        return;
      }
    } else if (!instance.value) {
      // Carpeta local accesible por URL aunque aún no esté en la lista en memoria.
      try {
        meta.value = await api.getInstanceMeta(id);
        await instances.load(true);
      } catch {
        error.value = "Instancia no encontrada.";
        return;
      }
    }

    if (!meta.value) {
      meta.value = await api.getInstanceMeta(id);
      if (!isExternal.value) {
        await instances.load(true);
      }
    }
    if (!isExternal.value) {
      ramMb.value = meta.value.ramMb || 4096;
      gc.value = (meta.value.gc as GcType) ?? "Auto";
      jvmArgs.value = meta.value.jvmArgs ?? "";
      javaPath.value = meta.value.javaPath ?? "";
      loader.value = meta.value.loader;
      loaderVersion.value = meta.value.loaderVersion;
      autoManaged.value = meta.value.autoManaged;
      editName.value = meta.value.name;
      editIcon.value = resolveInstanceIcon(meta.value.icon);
    } else {
      loader.value = meta.value.loader;
      loaderVersion.value = meta.value.loaderVersion;
    }
    content.value = await api.listInstanceContent(instanceId.value);
    folderPath.value = await api.getInstanceFolderPath(instanceId.value);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  app.loadHardware();
  void loadAll();
});

watch(instanceId, () => {
  tab.value = "content";
  void loadAll();
});

async function play() {
  const inst = displayInstance.value;
  if (!inst) return;
  launching.value = true;
  error.value = null;
  try {
    await app.launch(inst.id, inst.name);
  } catch (e) {
    error.value = String(e);
  } finally {
    launching.value = false;
  }
}

async function toggleItem(item: InstanceContentItem) {
  try {
    await api.toggleInstanceContent(instanceId.value, item.path, !item.enabled);
    content.value = await api.listInstanceContent(instanceId.value);
  } catch (e) {
    error.value = String(e);
  }
}

async function openFolder() {
  try {
    await api.openInstanceFolder(instanceId.value);
  } catch (e) {
    error.value = String(e);
  }
}

async function updateContent() {
  updating.value = true;
  message.value = null;
  try {
    await downloads.initEvents();
    const n = await api.updateInstanceContent(instanceId.value);
    message.value = n > 0 ? `${n} archivo(s) actualizados.` : "Todo está al día.";
    content.value = await api.listInstanceContent(instanceId.value);
  } catch (e) {
    error.value = String(e);
  } finally {
    updating.value = false;
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function goStore() {
  if (instance.value) instances.select(instance.value.id);
  router.push({ name: "store" });
}

async function saveAppearance() {
  if (isExternal.value) return;
  const name = editName.value.trim();
  if (!name) {
    error.value = "El nombre no puede estar vacío.";
    return;
  }
  appearanceBusy.value = true;
  error.value = null;
  try {
    await instances.rename(instanceId.value, name, editIcon.value);
    message.value = "Nombre e icono actualizados.";
    if (meta.value) {
      meta.value.name = name;
      meta.value.icon = editIcon.value;
    }
    await loadAll();
  } catch (e) {
    error.value = String(e);
  } finally {
    appearanceBusy.value = false;
  }
}

async function saveConfig() {
  configBusy.value = true;
  error.value = null;
  try {
    if (loader.value !== meta.value?.loader || loaderVersion.value !== meta.value?.loaderVersion) {
      await api.setInstanceLoader(instanceId.value, loader.value, loaderVersion.value);
    }
    await api.setInstanceConfig({
      id: instanceId.value,
      ramMb: ramMb.value,
      jvmArgs: jvmArgs.value || null,
      gc: gc.value,
      javaPath: javaPath.value || null,
    });
    message.value = "Configuración guardada.";
    await loadAll();
  } catch (e) {
    error.value = String(e);
  } finally {
    configBusy.value = false;
  }
}

async function reinstallLoader() {
  configBusy.value = true;
  error.value = null;
  try {
    if (loader.value !== meta.value?.loader || loaderVersion.value !== meta.value?.loaderVersion) {
      await api.setInstanceLoader(instanceId.value, loader.value, loaderVersion.value);
    }
    await api.reinstallInstanceLoader(instanceId.value);
    message.value = "Loader reinstalado correctamente.";
    await instances.load(true);
    await loadAll();
  } catch (e) {
    error.value = String(e);
  } finally {
    configBusy.value = false;
  }
}

async function backToAuto() {
  configBusy.value = true;
  try {
    await api.setInstanceAutoManaged(instanceId.value, true);
    await loadAll();
  } catch (e) {
    error.value = String(e);
  } finally {
    configBusy.value = false;
  }
}

async function duplicateInst() {
  if (!instance.value) return;
  try {
    const copy = await instances.duplicate(instanceId.value, `${instance.value.name} (copia)`);
    router.push({ name: "instance-detail", params: { id: copy.id } });
  } catch (e) {
    error.value = String(e);
  }
}

async function deleteInst() {
  if (!confirm(`¿Eliminar "${instance.value?.name}"?`)) return;
  try {
    await instances.remove(instanceId.value);
    router.push({ name: "instances" });
  } catch (e) {
    error.value = String(e);
  }
}

async function importExternal() {
  try {
    const inst = await instances.importExternal(instanceId.value);
    router.replace({ name: "instance-detail", params: { id: inst.id } });
  } catch (e) {
    error.value = String(e);
  }
}

const contentByFolder = computed(() => {
  const map = new Map<string, InstanceContentItem[]>();
  for (const item of content.value) {
    const list = map.get(item.folder) ?? [];
    list.push(item);
    map.set(item.folder, list);
  }
  return map;
});
</script>

<template>
  <div class="mx-auto max-w-5xl p-8">
    <button class="mb-4 text-sm text-gray-500 transition hover:text-white" @click="router.push({ name: 'instances' })">
      ← Instancias
    </button>

    <div v-if="loading" class="py-20 text-center text-gray-500">Cargando instancia…</div>

    <template v-else-if="displayInstance">
      <!-- Header -->
      <header class="mb-6 flex flex-wrap items-start gap-4">
        <InstanceIcon :icon="displayInstance.icon" size="lg" />
        <div class="min-w-0 flex-1">
          <h1 class="text-2xl font-bold">{{ displayInstance.name }}</h1>
          <p class="text-sm text-gray-400">
            Minecraft {{ displayInstance.mcVersion }} ·
            <span class="capitalize">{{ displayInstance.loader.replace(/-/g, " ") }}</span>
            <span v-if="displayInstance.loaderVersion"> {{ displayInstance.loaderVersion }}</span>
            · {{ formatRelative(displayInstance.lastPlayed) }} · {{ formatPlaytime(displayInstance.totalPlayMinutes) }}
          </p>
        </div>
        <div class="flex flex-wrap gap-2">
          <BaseButton size="lg" :disabled="launching || app.launchPhase === 'running'" @click="play">
            {{ launching ? "Lanzando…" : "Jugar" }}
          </BaseButton>
          <BaseButton v-if="isExternal" variant="secondary" @click="importExternal">
            Importar a Paraguacraft
          </BaseButton>
          <BaseButton v-else variant="secondary" @click="showBackups = true">Backups</BaseButton>
        </div>
      </header>

      <p v-if="isExternal" class="mb-4 rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-2 text-sm text-amber-200">
        Instancia detectada en otro launcher. Podes jugarla, instalar mods desde la tienda o importarla a Paraguacraft.
      </p>

      <p v-if="error" class="mb-4 text-sm text-red-400">{{ error }}</p>
      <p v-if="message" class="mb-4 text-sm text-pc-green">{{ message }}</p>

      <!-- Tabs -->
      <div class="mb-6 flex gap-1 rounded-xl bg-surface-2 p-1">
        <button
          v-for="t in tabs"
          :key="t.id"
          class="flex-1 rounded-lg px-4 py-2 text-sm font-semibold transition-colors"
          :class="tab === t.id ? 'bg-pc-green text-black' : 'text-gray-400 hover:text-white'"
          @click="tab = t.id"
        >
          {{ t.label }}
        </button>
      </div>

      <!-- Contenido -->
      <section v-if="tab === 'content'" class="space-y-4">
        <div class="flex flex-wrap gap-2">
          <BaseButton size="sm" variant="secondary" :disabled="updating" @click="updateContent">
            {{ updating ? "Actualizando…" : "Actualizar mods" }}
          </BaseButton>
          <BaseButton size="sm" variant="secondary" @click="goStore">Explorar tienda</BaseButton>
        </div>

        <p v-if="!content.length" class="rounded-xl border border-dashed border-surface-4 py-16 text-center text-gray-500">
          No hay mods ni packs instalados. Usa la tienda o arrastra archivos a la carpeta de la instancia.
        </p>

        <div v-for="[folder, items] in contentByFolder" :key="folder" class="rounded-xl border border-surface-4 bg-surface-2">
          <h3 class="border-b border-surface-3 px-4 py-2 text-xs font-bold uppercase tracking-wider text-gray-500">
            {{ folder }}
          </h3>
          <ul class="divide-y divide-surface-3">
            <li
              v-for="item in items"
              :key="item.path"
              class="flex items-center gap-3 px-4 py-3"
              :class="!item.enabled ? 'opacity-50' : ''"
            >
              <div class="min-w-0 flex-1">
                <p class="truncate font-medium">{{ item.name }}</p>
                <p class="text-xs text-gray-500">
                  {{ formatSize(item.sizeBytes) }}
                  <span v-if="item.sha1"> · {{ item.sha1.slice(0, 8) }}…</span>
                </p>
              </div>
              <button
                class="rounded-lg px-3 py-1 text-xs font-semibold transition-colors"
                :class="item.enabled ? 'bg-surface-4 text-gray-300 hover:bg-red-900/40 hover:text-red-300' : 'bg-pc-green/20 text-pc-green'"
                @click="toggleItem(item)"
              >
                {{ item.enabled ? "Desactivar" : "Activar" }}
              </button>
            </li>
          </ul>
        </div>
      </section>

      <!-- Archivos -->
      <section v-else-if="tab === 'files'" class="rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-2 text-lg font-bold">Carpeta de la instancia</h2>
        <p class="mb-4 break-all font-mono text-sm text-gray-400">{{ folderPath }}</p>
        <p class="mb-6 text-sm text-gray-500">
          Abrí la carpeta para agregar mods, resource packs o editar archivos manualmente.
        </p>
        <BaseButton @click="openFolder">Abrir carpeta</BaseButton>
      </section>

      <!-- Configuración (solo instancias locales) -->
      <section v-else-if="tab === 'settings' && isExternal" class="rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-2 text-lg font-bold">Configuración</h2>
        <p class="text-sm text-gray-400">
          Importá esta instancia a Paraguacraft para editar RAM, JVM y loader. Mientras tanto, podes jugarla e instalar mods desde la tienda.
        </p>
        <BaseButton class="mt-4" @click="importExternal">Importar a Paraguacraft</BaseButton>
      </section>

      <section v-else-if="tab === 'settings' && !isExternal" class="space-y-6">
        <div class="rounded-xl border border-surface-4 bg-surface-2 p-6">
          <h2 class="mb-4 text-lg font-bold">Apariencia</h2>
          <div class="mb-4 flex items-center gap-4">
            <InstanceIcon :icon="editIcon" size="lg" />
            <label class="min-w-0 flex-1">
              <span class="mb-1 block text-sm text-gray-400">Nombre de la instancia</span>
              <input
                v-model="editName"
                type="text"
                class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
              />
            </label>
          </div>
          <InstanceIconPicker v-model="editIcon" />
          <div class="mt-4 flex justify-end">
            <BaseButton size="sm" :disabled="appearanceBusy" @click="saveAppearance">
              {{ appearanceBusy ? "Guardando…" : "Guardar apariencia" }}
            </BaseButton>
          </div>
        </div>

        <div class="rounded-xl border border-surface-4 bg-surface-2 p-6">
          <h2 class="mb-4 text-lg font-bold">Instalación</h2>
          <div class="grid gap-4 sm:grid-cols-2">
            <label class="block">
              <span class="mb-1 block text-sm text-gray-400">Versión Minecraft</span>
              <input
                :value="meta?.mcVersion"
                disabled
                class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm text-gray-500"
              />
            </label>
            <label class="block">
              <span class="mb-1 block text-sm text-gray-400">Loader</span>
              <input
                v-model="loader"
                class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
              />
            </label>
            <label class="block sm:col-span-2">
              <span class="mb-1 block text-sm text-gray-400">Versión del loader</span>
              <input
                v-model="loaderVersion"
                class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
              />
            </label>
          </div>
          <p v-if="meta?.versionId" class="mt-2 text-xs text-gray-500">Perfil instalado: {{ meta.versionId }}</p>
          <div class="mt-4 flex flex-wrap gap-2">
            <BaseButton size="sm" variant="secondary" :disabled="configBusy" @click="reinstallLoader">
              {{ configBusy ? "Instalando…" : "Reinstalar loader" }}
            </BaseButton>
          </div>
        </div>

        <div class="rounded-xl border border-surface-4 bg-surface-2 p-6">
          <h2 class="mb-1 text-lg font-bold">Java y memoria</h2>
          <p class="mb-4 text-xs" :class="autoManaged ? 'text-pc-green' : 'text-amber-400'">
            {{ autoManaged ? "Autogestionado por hardware" : "Configuración manual" }}
          </p>

          <div class="space-y-4">
            <label class="block">
              <span class="mb-1 flex justify-between text-sm text-gray-300">
                <span>Memoria RAM</span>
                <span class="font-semibold text-pc-green">{{ (ramMb / 1024).toFixed(1) }} GB</span>
              </span>
              <input v-model.number="ramMb" type="range" min="1024" :max="maxRam" step="512" class="w-full accent-pc-green" />
            </label>

            <label class="block">
              <span class="mb-1 block text-sm text-gray-300">Garbage Collector</span>
              <select v-model="gc" class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green">
                <option v-for="g in gcOptions" :key="g" :value="g">{{ g }}</option>
              </select>
            </label>

            <label class="block">
              <span class="mb-1 block text-sm text-gray-300">Argumentos JVM</span>
              <input
                v-model="jvmArgs"
                type="text"
                placeholder="-Dsun.java2d.opengl=true"
                class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
              />
            </label>

            <label class="block">
              <span class="mb-1 block text-sm text-gray-300">Ruta de Java</span>
              <input
                v-model="javaPath"
                type="text"
                placeholder="Auto según la versión"
                class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
              />
            </label>
          </div>

          <div class="mt-6 flex flex-wrap items-center justify-between gap-2">
            <BaseButton v-if="!autoManaged" variant="ghost" :disabled="configBusy" @click="backToAuto">
              Volver a automático
            </BaseButton>
            <span v-else></span>
            <BaseButton :disabled="configBusy" @click="saveConfig">
              {{ configBusy ? "Guardando…" : "Guardar" }}
            </BaseButton>
          </div>
        </div>

        <div class="rounded-xl border border-surface-4 bg-surface-2 p-6">
          <h2 class="mb-4 text-lg font-bold">General</h2>
          <div class="flex flex-wrap gap-2">
            <BaseButton variant="secondary" @click="duplicateInst">Duplicar instancia</BaseButton>
            <BaseButton variant="ghost" class="!text-red-400" @click="deleteInst">Eliminar instancia</BaseButton>
          </div>
        </div>
      </section>
    </template>

    <p v-else class="py-20 text-center text-gray-500">{{ error ?? "Instancia no encontrada." }}</p>

    <BackupsModal v-if="showBackups && displayInstance" :instance="displayInstance" @close="showBackups = false" />
  </div>
</template>
