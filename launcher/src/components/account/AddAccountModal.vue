<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useAccountsStore } from "@/stores/accounts";
import { isTauri, api } from "@/lib/ipc";
import { MS_DEVICE_LINK, msLoginQrImageUrl, parseMsAuthCode } from "@/lib/msLogin";
import BaseButton from "@/components/common/BaseButton.vue";
import SkinAvatar from "@/components/account/SkinAvatar.vue";

const props = defineProps<{ startMode?: "microsoft" | "offline" }>();
const emit = defineEmits<{ (e: "close"): void }>();
const accounts = useAccountsStore();

type Mode = "choose" | "offline" | "microsoft";
const mode = ref<Mode>("choose");
const offlineName = ref("");
const busy = ref(false);
const localError = ref<string | null>(null);
const msStatus = ref<string | null>(null);
const showBrowserPaste = ref(false);
const redirectUrl = ref("");
const msWasPolling = ref(false);

const desktop = isTauri();
const canAddOffline = computed(() => offlineName.value.trim().length >= 3);
const offlinePreviewUuid = ref<string | null>(null);
const qrImageUrl = computed(() => msLoginQrImageUrl(168));

watch(offlineName, async (name) => {
  const clean = name.trim();
  if (clean.length < 3 || !isTauri()) {
    offlinePreviewUuid.value = null;
    return;
  }
  try {
    const skin = await api.getOfflineSkin(clean);
    offlinePreviewUuid.value = skin.uuid;
  } catch {
    offlinePreviewUuid.value = null;
  }
});

async function addOffline() {
  if (!canAddOffline.value) return;
  busy.value = true;
  localError.value = null;
  try {
    await accounts.addOffline(offlineName.value.trim());
    emit("close");
  } catch (err) {
    localError.value = String(err);
  } finally {
    busy.value = false;
  }
}

async function startMicrosoft() {
  mode.value = "microsoft";
  localError.value = null;
  msStatus.value = null;
  showBrowserPaste.value = false;
  redirectUrl.value = "";
  msWasPolling.value = false;
  try {
    await accounts.startMicrosoftLogin();
    msStatus.value = "Código listo — escaneá el QR o copiá el código";
    await copyCode(true);
  } catch (err) {
    localError.value = String(err);
  }
}

async function startBrowserLogin() {
  localError.value = null;
  showBrowserPaste.value = true;
  accounts.stopMicrosoftLogin();
  msWasPolling.value = false;
  msStatus.value = "Completá el login en el navegador y pegá la URL de redirección";
  try {
    const url = await api.msLoginUrl();
    const { open } = await import("@tauri-apps/plugin-shell");
    await open(url);
  } catch (err) {
    localError.value = String(err);
  }
}

async function completeBrowserLogin() {
  const code = parseMsAuthCode(redirectUrl.value);
  if (!code) {
    localError.value = "URL inválida. Pegá la URL completa con ?code=...";
    return;
  }
  busy.value = true;
  localError.value = null;
  try {
    await api.msLoginCompleteCode(code);
    await accounts.load(true);
    emit("close");
  } catch (err) {
    localError.value = String(err);
  } finally {
    busy.value = false;
  }
}

async function openMsLink() {
  await copyCode(true);
  const { open } = await import("@tauri-apps/plugin-shell");
  await open(MS_DEVICE_LINK);
  msStatus.value = "Abrí microsoft.com/link — pegá el código cuando te lo pida";
}

async function copyCode(silent = false) {
  if (!accounts.msDevice) return;
  await navigator.clipboard.writeText(accounts.msDevice.userCode);
  if (!silent) msStatus.value = "Código copiado — pegalo en microsoft.com/link";
}

function close() {
  accounts.stopMicrosoftLogin();
  emit("close");
}

function backToChoose() {
  accounts.stopMicrosoftLogin();
  showBrowserPaste.value = false;
  msWasPolling.value = false;
  mode.value = "choose";
}

watch(
  () => accounts.msPolling,
  (polling) => {
    if (mode.value !== "microsoft" || showBrowserPaste.value) return;
    if (msWasPolling.value && !polling && !accounts.msDevice && !accounts.msError) {
      emit("close");
    }
    msWasPolling.value = polling;
  },
);

onMounted(() => {
  if (props.startMode === "microsoft" && desktop) void startMicrosoft();
  else if (props.startMode === "offline") mode.value = "offline";
});
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm" @click.self="close">
    <div
      class="w-full rounded-2xl border border-surface-4 bg-surface-2 shadow-2xl"
      :class="mode === 'microsoft' ? 'max-w-[400px]' : 'max-w-md'"
    >
      <div class="flex items-center justify-between border-b border-surface-4 px-5 py-4">
        <h3 class="text-base font-bold">
          {{
            mode === "microsoft"
              ? "Iniciar sesión con Microsoft"
              : mode === "offline"
                ? "Cuenta offline"
                : "Agregar cuenta"
          }}
        </h3>
        <button class="text-xl leading-none text-gray-500 hover:text-white" @click="close">&times;</button>
      </div>

      <div class="p-5">
        <div v-if="mode === 'choose'" class="space-y-3">
          <button
            class="flex w-full items-center gap-3 rounded-xl border border-surface-4 bg-surface-3 p-4 text-left transition hover:border-pc-green"
            :disabled="!desktop"
            :class="{ 'cursor-not-allowed opacity-50': !desktop }"
            @click="startMicrosoft"
          >
            <svg class="h-7 w-7 shrink-0" viewBox="0 0 23 23">
              <rect x="1" y="1" width="10" height="10" fill="#f25022" />
              <rect x="12" y="1" width="10" height="10" fill="#7fba00" />
              <rect x="1" y="12" width="10" height="10" fill="#00a4ef" />
              <rect x="12" y="12" width="10" height="10" fill="#ffb900" />
            </svg>
            <span>
              <span class="block font-semibold">Microsoft (Premium)</span>
              <span class="block text-xs text-gray-400">QR, código o navegador.</span>
            </span>
          </button>
          <button
            class="flex w-full items-center gap-3 rounded-xl border border-surface-4 bg-surface-3 p-4 text-left transition hover:border-pc-green"
            @click="mode = 'offline'"
          >
            <span class="font-emoji text-2xl">&#128100;</span>
            <span>
              <span class="block font-semibold">Offline (No-Premium)</span>
              <span class="block text-xs text-gray-400">Juega en servidores en modo offline.</span>
            </span>
          </button>
          <p v-if="!desktop" class="text-center text-xs text-gray-500">
            El login Microsoft requiere la app de escritorio.
          </p>
        </div>

        <div v-else-if="mode === 'offline'" class="space-y-4">
          <div v-if="offlinePreviewUuid" class="flex items-center gap-3 rounded-lg bg-surface-3 p-3">
            <SkinAvatar :uuid="offlinePreviewUuid" :username="offlineName.trim()" size="sm" />
            <p class="text-xs text-gray-400">Vista previa de skin offline</p>
          </div>
          <label class="block">
            <span class="mb-1 block text-sm text-gray-300">Nombre de usuario</span>
            <input
              v-model="offlineName"
              type="text"
              placeholder="Steve"
              maxlength="16"
              class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
              @keyup.enter="addOffline"
            />
          </label>
          <p v-if="localError" class="text-xs text-red-400">{{ localError }}</p>
          <div class="flex justify-between">
            <BaseButton variant="ghost" @click="mode = 'choose'">Atras</BaseButton>
            <BaseButton :disabled="!canAddOffline || busy" @click="addOffline">Agregar</BaseButton>
          </div>
        </div>

        <div v-else class="flex flex-col gap-4">
          <BaseButton class="w-full justify-center gap-2" @click="startBrowserLogin">
            <svg class="h-4 w-4" viewBox="0 0 23 23">
              <rect x="1" y="1" width="10" height="10" fill="#f25022" />
              <rect x="12" y="1" width="10" height="10" fill="#7fba00" />
              <rect x="1" y="12" width="10" height="10" fill="#00a4ef" />
              <rect x="12" y="12" width="10" height="10" fill="#ffb900" />
            </svg>
            Iniciar sesión en el navegador
          </BaseButton>

          <div v-if="showBrowserPaste" class="space-y-2 rounded-xl border border-surface-4 bg-surface-3 p-3">
            <p class="text-xs text-gray-400">
              Copiá la URL completa de la barra del navegador después de iniciar sesión:
            </p>
            <input
              v-model="redirectUrl"
              type="text"
              placeholder="https://login.live.com/oauth20_desktop.srf?code=..."
              class="w-full rounded-lg border border-surface-5 bg-surface-1 px-3 py-2 text-xs outline-none focus:border-pc-green"
            />
            <BaseButton size="sm" class="w-full" :disabled="busy" @click="completeBrowserLogin">
              Verificar enlace
            </BaseButton>
          </div>

          <div class="flex items-center gap-3">
            <div class="h-px flex-1 bg-surface-4" />
            <span class="text-[10px] font-bold uppercase tracking-widest text-gray-600">o</span>
            <div class="h-px flex-1 bg-surface-4" />
          </div>

          <template v-if="accounts.msDevice">
            <div class="flex flex-col items-center gap-3">
              <img
                :src="qrImageUrl"
                alt="QR — abre microsoft.com/link"
                class="h-40 w-40 rounded-xl bg-white p-2"
                title="Escaneá para abrir microsoft.com/link"
              />
              <p class="px-1 text-center text-xs leading-relaxed text-gray-400">
                Escaneá el QR (abre
                <span class="text-pc-green">microsoft.com/link</span>) e ingresá este código:
              </p>
              <button
                type="button"
                class="group flex w-full max-w-[280px] items-center justify-center gap-2 rounded-xl border-2 border-dashed border-pc-green/40 bg-surface-3 px-4 py-3 transition hover:border-pc-green hover:bg-surface-4"
                title="Clic para copiar el código"
                @click="copyCode()"
              >
                <span class="select-all text-xl font-black tracking-[0.35em] text-pc-green">
                  {{ accounts.msDevice.userCode }}
                </span>
                <span class="shrink-0 text-xs text-gray-500 group-hover:text-pc-green">&#128203;</span>
              </button>
              <p class="flex min-h-[14px] items-center justify-center gap-2 text-center text-[10px] text-gray-500">
                <span v-if="accounts.msPolling" class="h-2 w-2 animate-pulse-dot rounded-full bg-pc-green" />
                {{ msStatus ?? "Esperando autorización…" }}
              </p>
            </div>
            <div class="flex gap-2">
              <BaseButton class="flex-1" @click="copyCode()">Copiar código</BaseButton>
              <BaseButton class="flex-1" variant="secondary" @click="openMsLink">
                Abrir microsoft.com/link
              </BaseButton>
            </div>
          </template>
          <p v-else-if="accounts.msError" class="text-sm text-red-400">{{ accounts.msError }}</p>
          <p v-else-if="!showBrowserPaste" class="text-center text-sm text-gray-400">Generando código…</p>

          <p v-if="localError" class="text-xs text-red-400">{{ localError }}</p>
          <BaseButton variant="ghost" @click="backToChoose">Atras</BaseButton>
        </div>
      </div>
    </div>
  </div>
</template>
