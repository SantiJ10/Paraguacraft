<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import BaseButton from "@/components/common/BaseButton.vue";
import InstanceIcon from "@/components/instance/InstanceIcon.vue";
import { api, isTauri } from "@/lib/ipc";
import type { BedrockStatus } from "@/lib/types";
import { useAccountsStore } from "@/stores/accounts";
import { useAppStore } from "@/stores/app";

withDefaults(
  defineProps<{ compact?: boolean }>(),
  { compact: false },
);

const router = useRouter();
const accounts = useAccountsStore();
const app = useAppStore();

const status = ref<BedrockStatus | null>(null);
const busy = ref(false);
const error = ref<string | null>(null);
const bedrockMessage = ref("");

const premiumLocked = computed(
  () => status.value != null && status.value.platformSupported && !status.value.premiumAllowed,
);
const notInstalled = computed(
  () => status.value?.platformSupported && status.value.premiumAllowed && !status.value.installed,
);
const canLaunch = computed(
  () =>
    isTauri() &&
    status.value?.platformSupported &&
    status.value.premiumAllowed &&
    status.value.installed,
);
const unsupported = computed(() => status.value != null && !status.value.platformSupported);

let unlistenStatus: (() => void) | null = null;
let unlistenStarted: (() => void) | null = null;
let unlistenExited: (() => void) | null = null;

async function refresh() {
  if (!isTauri()) {
    status.value = {
      platformSupported: false,
      installed: false,
      premiumAllowed: false,
      username: null,
    };
    return;
  }
  status.value = await api.getBedrockStatus();
}

async function bindEvents() {
  if (!isTauri()) return;
  const { listen } = await import("@tauri-apps/api/event");
  unlistenStatus = await listen<{ message: string }>("bedrock://status", (ev) => {
    bedrockMessage.value = ev.payload.message ?? "";
  });
  unlistenStarted = await listen("bedrock://started", () => {
    app.setLaunch("running", "Bedrock activo — launcher minimizado");
  });
  unlistenExited = await listen("bedrock://exited", () => {
    app.setLaunch("idle", "Listo para jugar");
    bedrockMessage.value = "";
    busy.value = false;
  });
}

async function launch() {
  if (!canLaunch.value || busy.value) return;
  busy.value = true;
  error.value = null;
  bedrockMessage.value = "Abriendo Bedrock…";
  app.setLaunch("launching", "Iniciando Minecraft: Bedrock Edition…");
  try {
    await api.launchBedrock();
  } catch (e) {
    error.value = String(e);
    bedrockMessage.value = "";
    app.setLaunch("idle", "Listo para jugar");
    busy.value = false;
  }
}

function goAccounts() {
  router.push({ name: "settings", hash: "#accounts" });
}

onMounted(async () => {
  await accounts.load();
  await refresh();
  await bindEvents();
});

onUnmounted(() => {
  unlistenStatus?.();
  unlistenStarted?.();
  unlistenExited?.();
});

watch(() => accounts.active?.id, () => {
  void refresh();
});
</script>

<template>
  <section
    class="overflow-hidden rounded-xl border border-[#1A4A7A]/60 bg-gradient-to-br from-[#0A1628] to-surface-2"
    :class="compact ? 'p-4' : 'p-6'"
  >
    <div class="flex flex-wrap items-start gap-4" :class="compact ? 'flex-row' : 'flex-col sm:flex-row'">
      <div class="flex min-w-0 flex-1 items-start gap-3">
        <InstanceIcon icon="mc:bedrock" size="lg" class="shrink-0" />
        <div class="min-w-0">
          <h2 class="text-lg font-bold text-[#3498DB]">Minecraft: Bedrock Edition</h2>
          <p class="mt-0.5 text-sm text-gray-400">
            Xbox / Microsoft Store · requiere cuenta Microsoft
          </p>
          <p v-if="status?.username && status.premiumAllowed" class="mt-1 text-xs text-gray-500">
            Cuenta activa: {{ status.username }}
          </p>
        </div>
      </div>

      <div class="w-full shrink-0 sm:w-auto sm:min-w-[200px]">
        <div v-if="unsupported" class="rounded-lg border border-surface-4 bg-surface-3/80 p-3 text-sm text-gray-400">
          Bedrock solo está disponible en Windows con la app de Microsoft Store.
        </div>

        <div
          v-else-if="premiumLocked"
          class="rounded-lg border border-[#F39C12]/30 bg-[#1A1A1A] p-3"
        >
          <p class="text-sm font-semibold text-[#F39C12]">Cuenta Premium requerida</p>
          <p class="mt-1 text-xs text-gray-500">
            Iniciá sesión con Microsoft para jugar Bedrock.
          </p>
          <BaseButton size="sm" variant="secondary" class="mt-3 w-full" @click="goAccounts">
            Agregar cuenta Microsoft
          </BaseButton>
        </div>

        <div v-else-if="notInstalled" class="rounded-lg border border-surface-4 bg-surface-3/80 p-3 text-sm text-gray-400">
          No se detectó Bedrock. Instalalo desde Xbox o Microsoft Store e intentá de nuevo.
        </div>

        <template v-else-if="canLaunch">
          <BaseButton
            class="w-full !bg-[#0078D4] hover:!bg-[#106EBE]"
            size="lg"
            :disabled="busy"
            @click="launch"
          >
            {{ busy ? "Abriendo…" : "Abrir Bedrock" }}
          </BaseButton>
          <p v-if="bedrockMessage" class="mt-2 text-center text-xs font-semibold text-[#3498DB]">
            {{ bedrockMessage }}
          </p>
        </template>
      </div>
    </div>

    <p v-if="error" class="mt-3 text-sm text-red-400">{{ error }}</p>
  </section>
</template>
