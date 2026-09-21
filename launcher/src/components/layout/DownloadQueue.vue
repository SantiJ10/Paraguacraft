<script setup lang="ts">
import { computed } from "vue";
import { useDownloadsStore } from "@/stores/downloads";
import ProgressBar from "@/components/common/ProgressBar.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import type { DownloadTask } from "@/lib/types";

const downloads = useDownloadsStore();

const visible = computed(() => downloads.queueOpen);
const items = computed(() => [...downloads.tasks].reverse());

function statusLabel(task: DownloadTask): string {
  switch (task.status) {
    case "downloading":
      return "Descargando";
    case "paused":
      return "Pausada";
    case "done":
      return "Lista";
    case "error":
      return "Error";
    case "cancelled":
      return "Cancelada";
    default:
      return "En cola";
  }
}

function canRetry(task: DownloadTask) {
  return task.status === "error" || task.status === "cancelled";
}
</script>

<template>
  <Teleport to="body">
    <Transition name="queue">
      <div v-if="visible" class="fixed inset-0 z-[320]" @click.self="downloads.toggleQueue(false)">
        <aside
          class="absolute bottom-12 right-4 flex w-[min(420px,calc(100vw-2rem))] max-h-[min(70vh,520px)] flex-col overflow-hidden rounded-2xl border border-surface-5 bg-surface-1 shadow-2xl"
        >
          <header class="flex items-center justify-between border-b border-surface-4 px-4 py-3">
            <div>
              <h2 class="text-sm font-bold">Descargas</h2>
              <p class="text-[11px] text-gray-500">
                {{ downloads.active.length }} activas
                <span v-if="downloads.failed.length"> · {{ downloads.failed.length }} con error</span>
              </p>
            </div>
            <button class="text-lg text-gray-500 hover:text-white" @click="downloads.toggleQueue(false)">
              &times;
            </button>
          </header>

          <div class="flex-1 space-y-2 overflow-y-auto p-3">
            <p v-if="!items.length" class="px-1 py-6 text-center text-sm text-gray-500">
              No hay descargas en curso.
            </p>
            <article
              v-for="task in items"
              :key="task.id"
              class="rounded-xl border border-surface-4 bg-surface-2 p-3"
            >
              <div class="flex items-start justify-between gap-2">
                <div class="min-w-0">
                  <p class="truncate text-sm font-semibold">{{ task.label }}</p>
                  <p
                    class="text-[11px]"
                    :class="
                      task.status === 'error' || task.status === 'cancelled'
                        ? 'text-red-400'
                        : task.status === 'done'
                          ? 'text-pc-green'
                          : 'text-gray-500'
                    "
                  >
                    {{ statusLabel(task) }}
                    <span v-if="task.speed"> · {{ task.speed }}</span>
                  </p>
                </div>
                <span class="shrink-0 text-xs tabular-nums text-gray-400">
                  {{ Math.round(task.progress ?? 0) }}%
                </span>
              </div>
              <ProgressBar class="mt-2" :value="task.progress ?? 0" variant="ai" />
              <p
                v-if="task.error"
                class="mt-1 line-clamp-2 text-[11px] text-red-400"
                :title="task.error"
              >
                {{ task.error }}
              </p>
              <p
                v-if="task.failedFile"
                class="mt-0.5 truncate font-mono text-[10px] text-gray-500"
                :title="task.failedFile"
              >
                {{ task.failedFile }}
              </p>
              <div class="mt-2 flex flex-wrap gap-1.5">
                <BaseButton
                  v-if="task.status === 'downloading'"
                  size="sm"
                  variant="secondary"
                  @click="downloads.pause(task.id)"
                >
                  Pausar
                </BaseButton>
                <BaseButton
                  v-if="task.status === 'paused'"
                  size="sm"
                  @click="downloads.resume(task.id)"
                >
                  Reanudar
                </BaseButton>
                <BaseButton
                  v-if="task.status === 'downloading' || task.status === 'paused'"
                  size="sm"
                  variant="danger"
                  @click="downloads.cancel(task.id)"
                >
                  Cancelar
                </BaseButton>
                <BaseButton v-if="canRetry(task)" size="sm" @click="downloads.retry(task.id)">
                  Reintentar
                </BaseButton>
                <BaseButton
                  v-if="task.folder || task.failedFile"
                  size="sm"
                  variant="ghost"
                  @click="downloads.openFolder(task)"
                >
                  Abrir carpeta
                </BaseButton>
                <BaseButton
                  v-if="task.status === 'done' || canRetry(task)"
                  size="sm"
                  variant="ghost"
                  @click="downloads.dismiss(task.id)"
                >
                  Cerrar
                </BaseButton>
              </div>
            </article>
          </div>
        </aside>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.queue-enter-active,
.queue-leave-active {
  transition: opacity 0.15s ease;
}
.queue-enter-from,
.queue-leave-to {
  opacity: 0;
}
</style>
