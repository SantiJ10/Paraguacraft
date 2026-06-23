import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { DownloadTask } from "@/lib/types";
import { isTauri } from "@/lib/ipc";

// Cola de descargas. En Tauri los eventos llegan via `download://progress`
// emitidos por el backend (p. ej. la descarga de Temurin). En navegador suelto
// se puede simular progreso con `enqueueDemo` para validar la UI.
export const useDownloadsStore = defineStore("downloads", () => {
  const tasks = ref<DownloadTask[]>([]);
  let listening = false;

  const active = computed(() => tasks.value.filter((t) => t.status === "downloading"));
  const failed = computed(() => tasks.value.filter((t) => t.status === "error"));
  const hasActivity = computed(() => active.value.length > 0 || failed.value.length > 0);

  function normalizeTask(p: DownloadTask): DownloadTask {
    const raw = p as DownloadTask & { failed_file?: string | null };
    return {
      ...p,
      failedFile: p.failedFile ?? raw.failed_file ?? null,
    };
  }

  function applyProgress(p: DownloadTask) {
    const task = normalizeTask(p);
    const idx = tasks.value.findIndex((t) => t.id === task.id);
    if (idx >= 0) tasks.value[idx] = task;
    else tasks.value.push(task);

    const delay = task.status === "error" ? 12000 : 2500;
    if (task.status === "done" || task.status === "error") {
      setTimeout(() => {
        tasks.value = tasks.value.filter((t) => t.id !== task.id);
      }, delay);
    }
  }

  /** Suscribe a los eventos de progreso del backend (idempotente). */
  async function initEvents() {
    if (listening || !isTauri()) return;
    listening = true;
    const { listen } = await import("@tauri-apps/api/event");
    await listen<DownloadTask>("download://progress", (event) => {
      applyProgress(event.payload);
    });
  }

  function enqueueDemo(label: string) {
    const task: DownloadTask = {
      id: `dl-${Date.now()}`,
      label,
      progress: 0,
      status: "downloading",
      speed: "0 MB/s",
    };
    tasks.value.push(task);
    const timer = setInterval(() => {
      task.progress = Math.min(100, task.progress + Math.random() * 18);
      task.speed = `${(2 + Math.random() * 10).toFixed(1)} MB/s`;
      if (task.progress >= 100) {
        task.status = "done";
        task.speed = "";
        clearInterval(timer);
        setTimeout(() => {
          tasks.value = tasks.value.filter((t) => t.id !== task.id);
        }, 2500);
      }
    }, 500);
  }

  return { tasks, active, failed, hasActivity, initEvents, applyProgress, enqueueDemo };
});
