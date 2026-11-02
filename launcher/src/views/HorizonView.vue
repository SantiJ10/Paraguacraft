<script setup lang="ts">
defineOptions({ name: "horizon" });
import { computed, onActivated, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import BaseButton from "@/components/common/BaseButton.vue";
import { api, isTauri, openUrl } from "@/lib/ipc";
import type { HorizonStatus } from "@/lib/types";
import { useAccountsStore } from "@/stores/accounts";
import { useInstancesStore } from "@/stores/instances";
import { useDownloadsStore } from "@/stores/downloads";
import { useAppStore } from "@/stores/app";

const HORIZON_REPO_URL = "https://github.com/horizon-rust/horizon";

const router = useRouter();
const accounts = useAccountsStore();
const instances = useInstancesStore();
const downloads = useDownloadsStore();
const app = useAppStore();

const status = ref<HorizonStatus | null>(null);
const loading = ref(true);
const busy = ref<null | "assets" | "install" | "pick" | "launch" | "fallback">(null);
const error = ref<string | null>(null);
const message = ref<string | null>(null);

const canUseHorizon = computed(
  () =>
    isTauri() &&
    !!status.value?.platformSupported &&
    !!status.value?.premiumAllowed,
);

const updateAvailable = computed(() => {
  const inst = status.value?.installedTag;
  const latest = status.value?.latestTag;
  if (!inst || !latest || status.value?.customBinary) return false;
  return inst !== latest;
});

async function refresh(checkRemote = false) {
  loading.value = !status.value;
  error.value = null;
  try {
    await accounts.load();
    status.value = await api.getHorizonStatus(checkRemote);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void refresh(true);
});

onActivated(() => {
  void refresh(false);
});

async function prepareAssets() {
  busy.value = "assets";
  error.value = null;
  message.value = null;
  try {
    await downloads.initEvents();
    status.value = await api.prepareHorizonAssets();
    message.value = "Assets oficiales de Minecraft 26.3 descargados y verificados.";
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}

async function installEngine() {
  busy.value = "install";
  error.value = null;
  message.value = null;
  try {
    await downloads.initEvents();
    status.value = await api.installHorizonEngine();
    message.value = `Motor Horizon listo (${status.value.installedTag ?? "instalado"}).`;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}

async function pickLocalBinary() {
  busy.value = "pick";
  error.value = null;
  message.value = null;
  try {
    const picked = await api.pickHorizonBinary();
    if (picked) {
      status.value = picked;
      message.value = "Ejecutable local de Horizon vinculado.";
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}

async function launchHorizon() {
  busy.value = "launch";
  error.value = null;
  message.value = null;
  try {
    await downloads.initEvents();
    // Obtener o crear la instancia de Horizon
    const instanceId = await api.getOrCreateHorizonInstance();
    // Lanzar usando el sistema normal de instancias
    const pid = await api.launchInstance(instanceId, null, false);
    try {
      localStorage.setItem("pc.horizon.lastPlayed", new Date().toISOString());
    } catch {
      /* ignore */
    }
    app.setLaunch("running", `Horizon 26.3 activo (PID ${pid})`);
    message.value = `Horizon iniciado correctamente (PID ${pid}).`;
    status.value = await api.getHorizonStatus(false);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}

async function openFolder() {
  try {
    await api.openHorizonFolder();
  } catch (e) {
    error.value = String(e);
  }
}

async function createOrOpenJava263() {
  busy.value = "fallback";
  error.value = null;
  try {
    await instances.load(true);
    const existing = instances.instances.find(
      (i) => i.mcVersion === "26.3" && i.loader.includes("optimized"),
    );
    if (existing) {
      instances.select(existing.id);
      router.push({ name: "instance-detail", params: { id: existing.id } });
      return;
    }
    const created = await api.createInstance({
      name: "Paraguacraft 26.3 Optimized",
      mcVersion: "26.3",
      loader: "paraguacraft-optimized",
      loaderVersion: "",
      icon: "mc:grass_block",
      ramMb: app.hardware?.recommendedRamMb ?? 4096,
    });
    await instances.load(true);
    instances.select(created.id);
    router.push({ name: "instance-detail", params: { id: created.id } });
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}
</script>

<template>
  <div class="mx-auto max-w-5xl space-y-6 p-8">
    <!-- Hero Banner al estilo Paraguacraft -->
    <section
      class="relative overflow-hidden rounded-2xl border border-pc-green/35 bg-gradient-to-br from-emerald-950/60 via-surface-2 to-amber-950/30 p-6"
    >
      <div class="flex flex-wrap items-start justify-between gap-4">
        <div class="max-w-2xl">
          <div class="inline-flex items-center gap-2 rounded-full border border-pc-green/40 bg-pc-green/15 px-3 py-1 text-xs font-bold uppercase tracking-wider text-pc-green">
            <span>⚡ PARAGUA × HORIZON</span>
            <span class="text-amber-300">· Experimental</span>
          </div>
          <h1 class="mt-3 text-3xl font-black tracking-tight text-white">
            Motor Nativo Rust + Vulkan para Minecraft 26.3
          </h1>
          <p class="mt-2 text-sm leading-relaxed text-gray-300">
            Horizon reescribe Minecraft <span class="font-semibold text-white">26.3</span> desde cero en
            <span class="font-semibold text-amber-300">Rust</span> con renderizador
            <span class="font-semibold text-pc-green">Vulkan 1.2+</span>: arranca en menos de 1 segundo sin Java,
            usa ~300 MB de RAM base y alcanza hasta <span class="font-semibold text-white">1024 chunks</span> con LODs acelerados por GPU.
          </p>
        </div>
        <div class="flex flex-col items-end gap-2">
          <BaseButton
            size="lg"
            :disabled="!canUseHorizon || !!busy"
            :title="!status?.premiumAllowed ? 'Requiere cuenta Microsoft oficial activa' : undefined"
            @click="launchHorizon"
          >
            {{ busy === "launch" ? "Iniciando Horizon…" : "⚡ Jugar con Horizon" }}
          </BaseButton>
          <button
            type="button"
            class="text-xs font-semibold text-gray-400 underline decoration-gray-600 hover:text-white"
            @click="openUrl(HORIZON_REPO_URL)"
          >
            Ver proyecto oficial horizon-rust/horizon ↗
          </button>
        </div>
      </div>

      <!-- Métricas comparativas rápidas -->
      <div class="mt-6 grid gap-3 sm:grid-cols-3">
        <div class="rounded-xl border border-surface-4/80 bg-black/25 p-3">
          <p class="text-[11px] font-bold uppercase tracking-wider text-gray-400">Arranque y Memoria</p>
          <p class="mt-1 text-lg font-black text-pc-green">&lt; 1 seg · ~300 MB</p>
          <p class="text-xs text-gray-400">100% código nativo en Rust, sin pausas de Garbage Collector JVM.</p>
        </div>
        <div class="rounded-xl border border-surface-4/80 bg-black/25 p-3">
          <p class="text-[11px] font-bold uppercase tracking-wider text-gray-400">Distancia de Visión</p>
          <p class="mt-1 text-lg font-black text-amber-300">Hasta 1024 Chunks</p>
          <p class="text-xs text-gray-400">Sistema integrado de niveles de detalle (LOD) sobre Vulkan.</p>
        </div>
        <div class="rounded-xl border border-surface-4/80 bg-black/25 p-3">
          <p class="text-[11px] font-bold uppercase tracking-wider text-gray-400">Tu GPU Detectada</p>
          <p class="mt-1 truncate text-sm font-bold text-white">{{ status?.gpuName ?? "Detectando…" }}</p>
          <p class="mt-0.5 text-[11px] text-gray-400">{{ status?.vulkanHint ?? "Requiere Vulkan 1.2+" }}</p>
        </div>
      </div>
    </section>

    <p v-if="error" class="rounded-xl border border-red-500/40 bg-red-950/30 px-4 py-3 text-sm text-red-300">
      {{ error }}
    </p>
    <p v-if="message" class="rounded-xl border border-pc-green/40 bg-pc-green/10 px-4 py-3 text-sm text-pc-green">
      {{ message }}
    </p>

    <!-- Checklist de requisitos y preparación -->
    <section class="grid gap-4 md:grid-cols-3">
      <!-- Paso 1: Cuenta Microsoft Oficial -->
      <article
        class="flex flex-col justify-between rounded-2xl border p-5"
        :class="
          status?.premiumAllowed
            ? 'border-pc-green/40 bg-surface-2'
            : 'border-amber-500/45 bg-amber-950/20'
        "
      >
        <div>
          <div class="flex items-center justify-between gap-2">
            <span class="text-xs font-bold uppercase tracking-wider text-gray-400">Paso 1 · Propiedad</span>
            <span
              class="rounded-full px-2.5 py-0.5 text-[10px] font-bold uppercase"
              :class="status?.premiumAllowed ? 'bg-pc-green/20 text-pc-green' : 'bg-amber-500/20 text-amber-300'"
            >
              {{ status?.premiumAllowed ? "Verificada" : "Requerida" }}
            </span>
          </div>
          <h2 class="mt-2 text-base font-bold text-white">Cuenta Microsoft Oficial</h2>
          <p v-if="status?.premiumAllowed" class="mt-1.5 text-xs text-gray-300">
            Sesión oficial activa como <span class="font-bold text-pc-green">{{ status.username }}</span>.
            Cumple con la verificación legítima de propiedad exigida por Horizon.
          </p>
          <p v-else class="mt-1.5 text-xs text-amber-100/90">
            Por licencia y diseño de Horizon, es <span class="font-semibold">obligatorio</span> iniciar sesión con una cuenta Microsoft que posea Minecraft oficial.
          </p>
        </div>
        <div class="mt-4">
          <BaseButton
            v-if="!status?.premiumAllowed"
            size="sm"
            class="w-full"
            @click="router.push({ name: 'settings' })"
          >
            Iniciar sesión en Ajustes →
          </BaseButton>
          <BaseButton
            v-else
            size="sm"
            variant="ghost"
            class="w-full"
            @click="router.push({ name: 'settings' })"
          >
            Cambiar cuenta
          </BaseButton>
        </div>
      </article>

      <!-- Paso 2: Assets Oficiales 26.3 -->
      <article
        class="flex flex-col justify-between rounded-2xl border p-5"
        :class="status?.assets263Ready ? 'border-pc-green/40 bg-surface-2' : 'border-surface-4 bg-surface-2'"
      >
        <div>
          <div class="flex items-center justify-between gap-2">
            <span class="text-xs font-bold uppercase tracking-wider text-gray-400">Paso 2 · Recursos</span>
            <span
              class="rounded-full px-2.5 py-0.5 text-[10px] font-bold uppercase"
              :class="status?.assets263Ready ? 'bg-pc-green/20 text-pc-green' : 'bg-surface-4 text-gray-300'"
            >
              {{ status?.assets263Ready ? "Listos" : "Pendiente" }}
            </span>
          </div>
          <h2 class="mt-2 text-base font-bold text-white">Assets Oficiales MC 26.3</h2>
          <p class="mt-1.5 text-xs text-gray-400">
            Horizon no redistribuye texturas ni sonidos de Mojang: los lee directamente desde tu instalación oficial vanilla 26.3 en <span class="font-mono text-gray-300">.minecraft</span>.
          </p>
        </div>
        <div class="mt-4">
          <BaseButton
            size="sm"
            :variant="status?.assets263Ready ? 'secondary' : 'primary'"
            class="w-full"
            :disabled="!canUseHorizon || !!busy"
            @click="prepareAssets"
          >
            {{
              busy === "assets"
                ? "Descargando 26.3…"
                : status?.assets263Ready
                  ? "Verificar / Re-chequear 26.3"
                  : "Descargar Assets 26.3"
            }}
          </BaseButton>
        </div>
      </article>

      <!-- Paso 3: Binario Nativo Horizon -->
      <article
        class="flex flex-col justify-between rounded-2xl border p-5"
        :class="status?.installed ? 'border-pc-green/40 bg-surface-2' : 'border-surface-4 bg-surface-2'"
      >
        <div>
          <div class="flex items-center justify-between gap-2">
            <span class="text-xs font-bold uppercase tracking-wider text-gray-400">Paso 3 · Motor</span>
            <span
              class="rounded-full px-2.5 py-0.5 text-[10px] font-bold uppercase"
              :class="
                updateAvailable
                  ? 'bg-amber-500/20 text-amber-300'
                  : status?.installed
                    ? 'bg-pc-green/20 text-pc-green'
                    : 'bg-surface-4 text-gray-300'
              "
            >
              {{
                updateAvailable
                  ? "Update disponible"
                  : status?.installed
                    ? status.installedTag || "Instalado"
                    : "No instalado"
              }}
            </span>
          </div>
          <h2 class="mt-2 text-base font-bold text-white">Cliente Horizon Windows x64</h2>
          <p class="mt-1.5 text-xs text-gray-400">
            Descarga automática desde los builds oficiales de <span class="font-mono text-gray-300">horizon-rust/horizon</span> o vinculá un <span class="font-mono text-gray-300">.exe</span> compilado por vos.
          </p>
          <p v-if="status?.latestTag" class="mt-1 text-[11px] text-gray-500">
            Último build remoto: <span class="font-mono text-gray-300">{{ status.latestTag }}</span>
          </p>
        </div>
        <div class="mt-4 space-y-2">
          <BaseButton
            size="sm"
            :variant="status?.installed && !updateAvailable ? 'secondary' : 'primary'"
            class="w-full"
            :disabled="!canUseHorizon || !!busy"
            @click="installEngine"
          >
            {{
              busy === "install"
                ? "Descargando Horizon…"
                : updateAvailable
                  ? "Actualizar Horizon"
                  : status?.installed
                    ? "Reinstalar Horizon"
                    : "Descargar Horizon"
            }}
          </BaseButton>
          <div class="flex gap-2">
            <BaseButton
              size="sm"
              variant="ghost"
              class="flex-1"
              :disabled="!canUseHorizon || !!busy"
              @click="pickLocalBinary"
            >
              Elegir .exe
            </BaseButton>
            <BaseButton
              size="sm"
              variant="ghost"
              class="flex-1"
              :disabled="!isTauri()"
              @click="openFolder"
            >
              Carpeta
            </BaseButton>
          </div>
        </div>
      </article>
    </section>

    <!-- Alternativas integradas en Paraguacraft -->
    <section class="grid gap-4 md:grid-cols-2">
      <div class="rounded-2xl border border-surface-4 bg-surface-2 p-5">
        <h3 class="text-sm font-bold text-white">¿Sin cuenta Microsoft o querés jugar con Mods Fabric/Forge?</h3>
        <p class="mt-1 text-xs leading-relaxed text-gray-400">
          Horizon es un motor experimental puro en Rust (no carga archivos <span class="font-mono">.jar</span> de Forge/Fabric ni cuentas offline). Si querés jugar Minecraft 26.3 con máximo FPS en cualquier cuenta o con mods, creá una instancia <span class="text-white">Java 26.3 Optimized</span>.
        </p>
        <div class="mt-4">
          <BaseButton size="sm" variant="secondary" :disabled="busy === 'fallback'" @click="createOrOpenJava263">
            {{ busy === "fallback" ? "Preparando…" : "Abrir / Crear instancia Java 26.3 Optimized" }}
          </BaseButton>
        </div>
      </div>

      <div class="rounded-2xl border border-amber-500/30 bg-surface-2 p-5">
        <h3 class="text-sm font-bold text-amber-300">⚡ Servidor Ultraligero Rust + Playit.gg para Amigos</h3>
        <p class="mt-1 text-xs leading-relaxed text-gray-400">
          ¿Querés hostear un servidor con la misma filosofía ultraligera en Rust (~80 MB RAM, sin Java, arranque instantáneo) y compartirlo con tus amigos por Playit.gg? Ya lo tenés integrado como segunda opción en Servidores.
        </p>
        <div class="mt-4">
          <BaseButton size="sm" variant="secondary" @click="router.push({ name: 'servers' })">
            Ir a Servidores para Amigos →
          </BaseButton>
        </div>
      </div>
    </section>
  </div>
</template>
