import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { DownloadTask } from "@/lib/types";
import { api, isTauri } from "@/lib/ipc";

// Cola de descargas. En Tauri los eventos llegan via `download://progress`
// emitidos por el backend. En navegador suelto se puede simular con `enqueueDemo`.
export const useDownloadsStore = defineStore("downloads", () => {
  const tasks = ref<DownloadTask[]>([]);
  const queueOpen = ref(false);
  let listening = false;
  const retries = new Map<string, () => Promise<void>>();
  const dismissTimers = new Map<string, number>();
  let lastRetry: (() => Promise<void>) | null = null;

  const active = computed(() =>
    tasks.value.filter((t) => t.status === "downloading" || t.status === "paused" || t.status === "queued"),
  );
  const failed = computed(() =>
    tasks.value.filter((t) => t.status === "error" || t.status === "cancelled"),
  );
  const hasActivity = computed(() => active.value.length > 0 || failed.value.length > 0);

  function normalizeTask(p: DownloadTask): DownloadTask {
    const raw = p as DownloadTask & { failed_file?: string | null; folder?: string | null };
    return {
      ...p,
      failedFile: p.failedFile ?? raw.failed_file ?? null,
      folder: p.folder ?? raw.folder ?? null,
    };
  }

  function clearDismissTimer(id: string) {
    const t = dismissTimers.get(id);
    if (t != null) {
      window.clearTimeout(t);
      dismissTimers.delete(id);
    }
  }

  function applyProgress(p: DownloadTask) {
    const incoming = normalizeTask(p);
    const idx = tasks.value.findIndex((t) => t.id === incoming.id);
    const prev = idx >= 0 ? tasks.value[idx] : null;
    const task: DownloadTask = {
      ...incoming,
      folder: incoming.folder || prev?.folder || null,
    };
    if (idx >= 0) tasks.value[idx] = task;
    else tasks.value.push(task);

    clearDismissTimer(task.id);
    if (task.status === "done") {
      const timer = window.setTimeout(() => {
        if (!queueOpen.value) {
          tasks.value = tasks.value.filter((t) => t.id !== task.id);
        }
        dismissTimers.delete(task.id);
      }, 8000);
      dismissTimers.set(task.id, timer);
    }
  }

  let unlisten: (() => void) | null = null;

  /** Suscribe a los eventos de progreso del backend (idempotente). */
  async function initEvents() {
    if (listening || !isTauri()) return;
    listening = true;
    const { listen } = await import("@tauri-apps/api/event");
    unlisten = await listen<DownloadTask>("download://progress", (event) => {
      applyProgress(event.payload);
    });
  }

  function disposeEvents() {
    unlisten?.();
    unlisten = null;
    listening = false;
  }

  function bindRetry(ids: string[], run: () => Promise<void>) {
    lastRetry = run;
    for (const id of ids) retries.set(id, run);
  }

  async function cancel(id: string) {
    await api.downloadControl(id, "cancel");
  }

  async function pause(id: string) {
    await api.downloadControl(id, "pause");
  }

  async function resume(id: string) {
    await api.downloadControl(id, "resume");
  }

  async function retry(id: string) {
    const fn = retries.get(id) ?? lastRetry;
    if (!fn) return;
    const current = tasks.value.find((t) => t.id === id);
    if (current) {
      applyProgress({ ...current, status: "downloading", progress: 0, error: null });
    }
    await fn();
  }

  async function openFolder(task: DownloadTask) {
    if (task.folder) {
      await api.revealDownloadPath(task.folder);
      return;
    }
    if (task.failedFile && (task.failedFile.includes("/") || task.failedFile.includes("\\"))) {
      await api.revealDownloadPath(task.failedFile);
    }
  }

  function dismiss(id: string) {
    clearDismissTimer(id);
    tasks.value = tasks.value.filter((t) => t.id !== id);
  }

  function toggleQueue(open?: boolean) {
    queueOpen.value = open ?? !queueOpen.value;
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
        applyProgress({ ...task });
      }
    }, 500);
  }

  return {
    tasks,
    active,
    failed,
    hasActivity,
    queueOpen,
    initEvents,
    disposeEvents,
    applyProgress,
    enqueueDemo,
    bindRetry,
    cancel,
    pause,
    resume,
    retry,
    openFolder,
    dismiss,
    toggleQueue,
  };
});
