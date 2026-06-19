import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { DownloadTask } from "@/lib/types";

// Store de la cola de descargas. En Fase 3 los eventos llegan via Tauri events
// desde el motor `tokio`; en Fase 1 simulamos progreso para validar la UI.
export const useDownloadsStore = defineStore("downloads", () => {
  const tasks = ref<DownloadTask[]>([]);

  const active = computed(() => tasks.value.filter((t) => t.status === "downloading"));
  const hasActivity = computed(() => active.value.length > 0);

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

  return { tasks, active, hasActivity, enqueueDemo };
});
