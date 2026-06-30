<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import BaseButton from "@/components/common/BaseButton.vue";
import BaseToggle from "@/components/common/BaseToggle.vue";
import { useMusicStore } from "@/stores/music";
import { parseMusicUrl } from "@/lib/musicUrl";
import { isTauri, api } from "@/lib/ipc";

const music = useMusicStore();

const error = ref<string | null>(null);
const tab = ref<"youtube" | "spotify">("youtube");
const manualCode = ref("");
const showManualCode = ref(false);
const credsOk = ref<boolean | null>(null);
const dashboardSaved = ref(false);
const clientIdHint = ref<string | null>(null);

const examples = {
  youtube: "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
};

function submitYoutube() {
  error.value = null;
  if (!music.loadFromInput()) {
    error.value = "URL invalida. Pega un enlace de YouTube.";
    return;
  }
}

function tryExample() {
  music.inputUrl = examples.youtube;
  submitYoutube();
}

async function openDashboard() {
  await saveCredentialsQuiet();
  await api.spotifyOpenDashboard(music.spotifyClientId.trim() || undefined);
}

async function saveCredentialsQuiet() {
  if (!music.spotifyClientId.trim() || !music.spotifyClientSecret.trim()) return;
  try {
    await api.spotifySaveCredentials(
      music.spotifyClientId.trim(),
      music.spotifyClientSecret.trim(),
      music.spotifyRedirectUri.trim(),
    );
    const info = await api.spotifySetupInfo();
    clientIdHint.value = info.clientIdOk
      ? null
      : `Client ID tiene ${info.clientIdLength} caracteres (deben ser 32)`;
  } catch {
    /* ignore */
  }
}

async function validateCredentials() {
  error.value = null;
  credsOk.value = null;
  clientIdHint.value = null;
  try {
    await music.validateSpotifyCredentials();
    const info = await api.spotifySetupInfo();
    if (!info.clientIdOk) {
      throw new Error(`Client ID invalido (${info.clientIdLength} caracteres). Deben ser 32.`);
    }
    credsOk.value = true;
  } catch (e) {
    credsOk.value = false;
    error.value = String(e);
  }
}

async function authorizeSpotify() {
  error.value = null;
  if (!dashboardSaved.value) {
    error.value = "Marca la casilla de que ya guardaste el Redirect URI con SAVE en Spotify Dashboard.";
    return;
  }
  try {
    await music.startSpotifyAuth();
  } catch (e) {
    error.value = String(e);
  }
}

async function connectManualCode() {
  error.value = null;
  try {
    await music.connectSpotifyManualCode(manualCode.value);
    manualCode.value = "";
  } catch (e) {
    error.value = String(e);
  }
}

function copyRedirectUri() {
  void navigator.clipboard.writeText(music.spotifyRedirectUri);
}

watch(tab, (v) => {
  error.value = null;
  if (v === "spotify") void music.trySpotifyAutoconnect();
});

watch(
  () => music.inputUrl,
  (v) => {
    if (!v.trim()) return;
    const p = parseMusicUrl(v);
    if (p?.source === "youtube") tab.value = "youtube";
  },
);

onMounted(() => {
  if (tab.value === "spotify") void music.trySpotifyAutoconnect();
});
</script>

<template>
  <Transition name="panel">
    <section
      v-if="music.panelOpen"
      class="absolute right-0 top-full z-50 mt-2 w-[min(22rem,calc(100vw-2rem))] overflow-hidden rounded-xl border border-surface-4 bg-surface-2 shadow-2xl"
    >
      <header class="flex items-center justify-between border-b border-surface-4 px-4 py-3">
        <div>
          <h3 class="font-bold">Musica</h3>
          <p class="text-xs text-gray-500">YouTube embebido · Spotify API</p>
        </div>
        <button type="button" class="text-gray-500 hover:text-white" @click="music.closePanel()">
          &times;
        </button>
      </header>

      <div class="border-b border-surface-4 p-1">
        <div class="flex gap-1">
          <button
            type="button"
            class="flex-1 rounded-lg py-2 text-xs font-semibold transition-colors"
            :class="tab === 'youtube' ? 'bg-red-500/20 text-red-400' : 'text-gray-500 hover:bg-surface-3'"
            @click="tab = 'youtube'"
          >
            YouTube
          </button>
          <button
            type="button"
            class="flex-1 rounded-lg py-2 text-xs font-semibold transition-colors"
            :class="tab === 'spotify' ? 'bg-emerald-500/20 text-emerald-400' : 'text-gray-500 hover:bg-surface-3'"
            @click="tab = 'spotify'"
          >
            Spotify
          </button>
        </div>
      </div>

      <div class="space-y-3 p-4">
        <!-- YouTube -->
        <template v-if="tab === 'youtube'">
          <div>
            <label class="mb-1 block text-xs text-gray-500">Enlace de video, playlist, shorts o music.youtube.com</label>
            <input
              v-model="music.inputUrl"
              :placeholder="examples.youtube"
              class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
              @keydown.enter="submitYoutube"
            />
          </div>
          <div class="flex gap-2">
            <BaseButton class="flex-1" size="sm" @click="submitYoutube">Reproducir</BaseButton>
            <BaseButton class="flex-1" size="sm" variant="secondary" @click="tryExample">Ejemplo</BaseButton>
          </div>
          <div v-if="music.embedUrl" class="rounded-lg border border-surface-4 bg-surface-3/50 p-3">
            <div class="mb-2 flex items-center gap-2">
              <img
                v-if="music.youtubeNow?.thumbnail"
                :src="music.youtubeNow.thumbnail"
                alt=""
                class="h-9 w-9 shrink-0 rounded object-cover"
              />
              <div class="min-w-0">
                <p class="truncate text-xs text-gray-400">Reproduciendo</p>
                <p class="truncate text-sm font-semibold">{{ music.youtubeNow?.title || music.label }}</p>
              </div>
            </div>
            <div class="mt-2 flex items-center gap-2">
              <button
                type="button"
                class="rounded-md bg-surface-4 px-3 py-1 text-xs font-semibold hover:bg-surface-5"
                @click="music.togglePlay()"
              >
                {{ music.playing ? "Pausar" : "Play" }}
              </button>
              <button
                type="button"
                class="rounded-md px-2 py-1 text-xs text-gray-500 hover:text-red-400"
                @click="music.stop()"
              >
                Detener
              </button>
            </div>
            <label class="mt-3 flex items-center gap-2 text-xs text-gray-400">
              Volumen
              <input
                type="range"
                min="0"
                max="100"
                :value="music.volume"
                class="flex-1 accent-pc-green"
                @input="music.setVolume(Number(($event.target as HTMLInputElement).value))"
              />
              {{ music.volume }}%
            </label>
          </div>
        </template>

        <!-- Spotify (API como launcher Python) -->
        <template v-else>
          <div v-if="!music.spotifyConnected">
            <h4 class="mb-1 text-sm font-bold">Conectar Spotify</h4>

            <div class="mb-3 space-y-1.5 rounded-lg border border-amber-500/30 bg-amber-500/5 p-2.5 text-[11px] leading-relaxed text-amber-200/90">
              <p class="font-semibold text-amber-300">Configura Spotify Dashboard (app nueva):</p>
              <p>
                1. Pulsa <strong>Abrir mi app</strong> → pestaña <strong>Settings</strong>.
              </p>
              <p>2. Redirect URIs → pega y pulsa <strong>Add</strong>:</p>
              <p class="font-mono text-[10px] text-white">http://127.0.0.1:8888/callback</p>
              <p class="font-semibold text-red-300">
                3. Baja al final de la pagina y pulsa <strong>SAVE</strong> (sin Save no se guarda).
              </p>
              <p>4. User Management → email EXACTO de tu cuenta Spotify.</p>
              <p class="text-amber-400/90">No uses localhost — Spotify lo rechaza en apps nuevas.</p>
            </div>

            <div class="space-y-2">
              <label class="block text-xs text-gray-500">Redirect URI (obligatorio)</label>
              <input
                v-model="music.spotifyRedirectUri"
                type="text"
                readonly
                spellcheck="false"
                class="w-full cursor-default rounded-lg border border-surface-5 bg-surface-3/80 px-3 py-2 font-mono text-xs text-gray-400 outline-none"
              />
              <div class="flex flex-wrap gap-2">
                <button
                  type="button"
                  class="rounded-md border border-surface-5 px-2 py-1 text-[10px] text-gray-400 hover:text-white"
                  @click="copyRedirectUri"
                >
                  Copiar URI
                </button>
                <button
                  type="button"
                  class="rounded-md border border-emerald-500/40 bg-emerald-500/10 px-2 py-1 text-[10px] text-emerald-300 hover:bg-emerald-500/20"
                  @click="openDashboard"
                >
                  Abrir mi app
                </button>
              </div>
              <input
                v-model="music.spotifyClientId"
                type="text"
                placeholder="Client ID (32 caracteres)"
                maxlength="32"
                spellcheck="false"
                class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 font-mono text-sm outline-none focus:border-emerald-500"
                @blur="saveCredentialsQuiet"
              />
              <p v-if="clientIdHint" class="text-[10px] text-red-400">{{ clientIdHint }}</p>
              <input
                v-model="music.spotifyClientSecret"
                type="password"
                placeholder="Client Secret"
                class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-emerald-500"
              />
              <div class="flex gap-2">
                <BaseButton
                  class="flex-1"
                  size="sm"
                  variant="secondary"
                  :disabled="!isTauri()"
                  @click="validateCredentials"
                >
                  Probar credenciales
                </BaseButton>
                <span
                  v-if="credsOk === true"
                  class="flex items-center text-xs text-emerald-400"
                >
                  OK
                </span>
              </div>
              <label class="flex items-start gap-2 text-[11px] text-gray-400">
                <input v-model="dashboardSaved" type="checkbox" class="mt-0.5 accent-emerald-500" />
                <span>Ya agregue el Redirect URI y pulse <strong class="text-gray-200">SAVE</strong> en Spotify Dashboard.</span>
              </label>
              <BaseButton
                class="w-full"
                size="sm"
                :disabled="!isTauri() || music.spotifyAuthBusy"
                @click="authorizeSpotify"
              >
                {{ music.spotifyAuthBusy ? "Esperando autorizacion..." : "Autorizar en Spotify" }}
              </BaseButton>
              <p class="text-[10px] text-gray-600">
                Si falla: cierra sesion en open.spotify.com, espera 1 min tras guardar User Management, reintenta.
              </p>
              <button
                type="button"
                class="text-xs text-gray-500 underline hover:text-gray-300"
                @click="showManualCode = !showManualCode"
              >
                Pegar codigo manualmente
              </button>
              <div v-if="showManualCode" class="flex gap-2">
                <input
                  v-model="manualCode"
                  type="text"
                  placeholder="code= de la URL de retorno"
                  class="flex-1 rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none"
                />
                <BaseButton size="sm" @click="connectManualCode">Conectar</BaseButton>
              </div>
            </div>
          </div>

          <div v-else class="rounded-lg border border-emerald-500/30 bg-emerald-500/5 p-3">
            <div class="flex items-center gap-3">
              <img
                :src="music.spotifyNow?.imageUrl || 'https://via.placeholder.com/64/1DB954/000?text=%E2%99%AB'"
                alt=""
                class="h-16 w-16 shrink-0 rounded-lg object-cover"
              />
              <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-bold">
                  {{ music.spotifyNow?.title || "Sin reproduccion activa" }}
                </p>
                <p class="truncate text-xs text-gray-400">{{ music.spotifyNow?.artist }}</p>
                <p class="truncate text-[10px] text-gray-600">{{ music.spotifyNow?.album }}</p>
              </div>
            </div>
            <div class="mt-3 h-1 overflow-hidden rounded-full bg-surface-4">
              <div
                class="h-full bg-emerald-500 transition-all"
                :style="{ width: `${music.spotifyProgressPct}%` }"
              />
            </div>
            <div class="mt-4 flex items-center justify-center gap-3">
              <button
                type="button"
                class="text-sm"
                :class="music.spotifyNow?.shuffle ? 'text-emerald-400' : 'text-gray-500'"
                @click="music.spotifyToggleShuffle()"
              >
                🔀
              </button>
              <button type="button" class="text-xl text-gray-300 hover:text-emerald-400" @click="music.spotifyControl('prev')">
                ⏮
              </button>
              <button
                type="button"
                class="flex h-10 w-10 items-center justify-center rounded-full bg-emerald-500 text-sm font-black text-black hover:bg-emerald-400"
                @click="music.spotifyControl('play')"
              >
                {{ music.spotifyPlaying ? "⏸" : "▶" }}
              </button>
              <button type="button" class="text-xl text-gray-300 hover:text-emerald-400" @click="music.spotifyControl('next')">
                ⏭
              </button>
              <button
                type="button"
                class="text-sm"
                :class="music.spotifyNow?.repeatState !== 'off' ? 'text-emerald-400' : 'text-gray-500'"
                @click="music.spotifyToggleRepeat()"
              >
                🔁
              </button>
            </div>
            <button
              type="button"
              class="mt-3 w-full text-center text-xs text-gray-500 hover:text-red-400"
              @click="music.disconnectSpotify()"
            >
              Desconectar Spotify
            </button>
          </div>
        </template>

        <p v-if="error" class="whitespace-pre-line text-xs text-red-400">{{ error }}</p>

        <div class="space-y-2 border-t border-surface-4 pt-3">
          <p class="text-xs font-semibold uppercase tracking-wide text-gray-500">Opciones de overlay</p>
          <label class="flex items-center justify-between gap-3 text-sm">
            <span class="text-gray-300">Musica de fondo YouTube</span>
            <BaseToggle v-model="music.launcherBackground" />
          </label>
          <label class="flex items-center justify-between gap-3 text-sm">
            <span class="text-gray-300">Overlay en launcher</span>
            <BaseToggle v-model="music.launcherOverlay" />
          </label>
          <label class="flex items-center justify-between gap-3 text-sm">
            <span class="text-gray-300">Overlay mientras jugas</span>
            <BaseToggle v-model="music.inGameOverlay" />
          </label>
          <p class="text-[11px] leading-relaxed text-gray-600">
            Spotify controla la reproduccion de tu cuenta (app de escritorio o movil). El overlay muestra lo que suena
            ahora. Funciona en juego si el launcher queda abierto.
          </p>
        </div>
      </div>
    </section>
  </Transition>
</template>

<style scoped>
.panel-enter-active,
.panel-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.panel-enter-from,
.panel-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
