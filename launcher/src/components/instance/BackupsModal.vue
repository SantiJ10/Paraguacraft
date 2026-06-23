<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useInstancesStore } from "@/stores/instances";
import type { BackupInfo, Instance } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";

const props = defineProps<{ instance: Instance }>();
const emit = defineEmits<{ (e: "close"): void }>();
const instances = useInstancesStore();

const backups = ref<BackupInfo[]>([]);
const busy = ref(false);
const error = ref<string | null>(null);

function fmtSize(bytes: number) {
  if (bytes > 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024).toFixed(0)} KB`;
}

function fmtDate(epoch: number) {
  return new Date(epoch * 1000).toLocaleString();
}

async function refresh() {
  backups.value = await instances.backups(props.instance.id);
}

async function create() {
  busy.value = true;
  error.value = null;
  try {
    await instances.createBackup(props.instance.id);
    await refresh();
  } catch (err) {
    error.value = String(err);
  } finally {
    busy.value = false;
  }
}

async function restore(name: string) {
  busy.value = true;
  error.value = null;
  try {
    await instances.restoreBackup(props.instance.id, name);
  } catch (err) {
    error.value = String(err);
  } finally {
    busy.value = false;
  }
}

async function remove(name: string) {
  await instances.deleteBackup(props.instance.id, name);
  await refresh();
}

onMounted(refresh);
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" @click.self="emit('close')">
    <div class="flex max-h-[80vh] w-full max-w-lg flex-col rounded-2xl border border-surface-4 bg-surface-2 p-6 shadow-2xl">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-lg font-bold">Backups - {{ instance.name }}</h3>
        <button class="text-gray-500 hover:text-white" @click="emit('close')">&times;</button>
      </div>

      <BaseButton class="mb-4 self-start" size="sm" :disabled="busy" @click="create">
        {{ busy ? "Trabajando..." : "+ Crear backup" }}
      </BaseButton>

      <div class="flex-1 space-y-2 overflow-y-auto">
        <div
          v-for="b in backups"
          :key="b.name"
          class="flex items-center gap-3 rounded-lg border border-surface-4 p-3"
        >
          <span class="font-emoji text-lg">&#128230;</span>
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-semibold">{{ fmtDate(b.createdAt) }}</p>
            <p class="text-xs text-gray-500">{{ fmtSize(b.sizeBytes) }}</p>
          </div>
          <BaseButton size="sm" variant="ghost" :disabled="busy" @click="restore(b.name)">Restaurar</BaseButton>
          <button class="text-gray-600 hover:text-red-400" @click="remove(b.name)">&times;</button>
        </div>
        <p v-if="!backups.length" class="py-8 text-center text-sm text-gray-500">
          Aun no hay backups. Crea uno para proteger tus mundos.
        </p>
      </div>

      <p v-if="error" class="mt-3 text-xs text-red-400">{{ error }}</p>
    </div>
  </div>
</template>
