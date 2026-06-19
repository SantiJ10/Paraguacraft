// Capa de abstraccion IPC.
//
// Objetivo (Fase 1): la UI nunca llama `invoke` directamente. Habla con `api.*`,
// que hoy devuelve datos mock y, cuando el backend Rust este listo (Fase 2+),
// delegara en los comandos reales de Tauri sin tocar la UI.
//
// Para `get_hardware_info` ya intentamos el comando real como prueba del puente:
// si Tauri responde, usamos su dato; si no (modo navegador), caemos al mock.

import type {
  Account,
  AppSettings,
  DownloadTask,
  HardwareInfo,
  Instance,
  LoaderInfo,
  MinecraftVersion,
  StoreItem,
} from "@/lib/types";
import {
  mockAccounts,
  mockHardware,
  mockInstances,
  mockLoaders,
  mockStoreItems,
  mockVersions,
} from "@/lib/mock/data";

/** True si corremos dentro del runtime de Tauri (no en un navegador suelto). */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Invoca un comando Tauri de forma segura; lanza si no hay runtime. */
async function invokeReal<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

function delay<T>(value: T, ms = 120): Promise<T> {
  return new Promise((resolve) => setTimeout(() => resolve(value), ms));
}

export const api = {
  async getHardwareInfo(): Promise<HardwareInfo> {
    if (isTauri()) {
      try {
        return await invokeReal<HardwareInfo>("get_hardware_info");
      } catch (err) {
        console.warn("[ipc] get_hardware_info fallback a mock:", err);
      }
    }
    return delay(mockHardware);
  },

  async getAccounts(): Promise<Account[]> {
    return delay(mockAccounts);
  },

  async getInstances(): Promise<Instance[]> {
    return delay(mockInstances);
  },

  async getVersions(): Promise<MinecraftVersion[]> {
    return delay(mockVersions);
  },

  async getLoaders(_mcVersion: string): Promise<LoaderInfo[]> {
    return delay(mockLoaders);
  },

  async searchStore(query: string): Promise<StoreItem[]> {
    const q = query.trim().toLowerCase();
    const items = q
      ? mockStoreItems.filter((i) => i.title.toLowerCase().includes(q))
      : mockStoreItems;
    return delay(items);
  },

  async getSettings(): Promise<AppSettings> {
    return delay({
      ramMb: mockHardware.recommendedRamMb,
      gcType: mockHardware.recommendedGc,
      javaPath: null,
      closeOnLaunch: false,
      optimizeGraphics: false,
      gpuCompatMode: "off",
      theme: "dark",
      accent: "green",
      language: "es",
    });
  },

  async getDownloads(): Promise<DownloadTask[]> {
    return delay([]);
  },
};
