<script setup lang="ts">
defineOptions({ name: "home" });
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import mainBanner from "@/assets/main_banner.png";
import SkinAvatar from "@/components/account/SkinAvatar.vue";
import { useInstancesStore } from "@/stores/instances";
import { useAccountsStore } from "@/stores/accounts";
import { useAppStore } from "@/stores/app";
import { useSkinsStore } from "@/stores/skins";
import BaseButton from "@/components/common/BaseButton.vue";
import InstanceCard from "@/components/common/InstanceCard.vue";
import InstanceIcon from "@/components/instance/InstanceIcon.vue";
import FavoriteServersPanel from "@/components/home/FavoriteServersPanel.vue";
import GameProfilesPanel from "@/components/home/GameProfilesPanel.vue";
import { formatPlaytime } from "@/composables/useFormat";
import type { Instance } from "@/lib/types";

const router = useRouter();
const instances = useInstancesStore();
const accounts = useAccountsStore();
const app = useAppStore();
const skins = useSkinsStore();

const launchingId = ref<string | null>(null);
const launchError = ref<string | null>(null);

const featured = computed(() => instances.selected);

const avatarKey = computed(
  () => `${skins.revision}:${skins.activeSkin?.skinUrl ?? skins.activeSkin?.avatarUrl ?? ""}`,
);

const greetingName = computed(() => accounts.active?.username ?? "jugador");

function openInstance(inst: Instance) {
  instances.select(inst.id);
  router.push({ name: "instance-detail", params: { id: inst.id } });
}

async function play(inst: { id: string; name: string }) {
  launchingId.value = inst.id;
  launchError.value = null;
  try {
    await app.launch(inst.id, inst.name);
  } catch (e) {
    launchError.value = String(e);
    app.setLaunch("idle", "Listo para jugar");
  } finally {
    launchingId.value = null;
  }
}
</script>

<template>
  <div class="flex min-h-full flex-col">
    <section v-if="featured" class="relative shrink-0 overflow-hidden">
      <div class="relative flex min-h-[380px] flex-col justify-end px-8 pb-10 pt-16">
        <div class="pointer-events-none absolute inset-0 z-0" aria-hidden="true">
          <img
            :src="mainBanner"
            alt=""
            class="h-full w-full object-cover object-[center_20%] opacity-80"
            draggable="false"
          />
          <div
            class="absolute inset-0 bg-gradient-to-t from-surface-1 via-surface-1/35 to-surface-1/10"
          />
        </div>

        <div class="relative z-10 max-w-3xl">
          <div class="flex items-center gap-3">
            <SkinAvatar
              :key="avatarKey"
              size="lg"
              :uuid="accounts.active?.uuid ?? skins.activeSkin?.uuid"
              :username="accounts.active?.username ?? skins.activeSkin?.username"
              :avatar-url="skins.activeSkin?.avatarUrl"
              :avatar-data-url="skins.activeSkin?.avatarDataUrl"
              :local-avatar-path="skins.activeSkin?.localAvatarPath"
              :loading="skins.loading && !accounts.active"
            />
            <p class="text-sm font-bold uppercase tracking-wider text-pc-green">
              Hola, {{ greetingName }}
            </p>
          </div>

          <h1 class="mt-4 flex items-center gap-3 text-4xl font-black tracking-tight">
            <InstanceIcon :icon="featured.icon" size="lg" />
            <span>{{ featured.name }}</span>
          </h1>

          <p class="mt-2 text-sm text-gray-300">
            Minecraft {{ featured.mcVersion }} &middot;
            <span class="capitalize">{{ featured.loader.replace("-", " ") }}</span> &middot;
            {{ formatPlaytime(featured.totalPlayMinutes) }}
          </p>

          <div class="mt-6 flex flex-wrap items-center gap-3">
            <BaseButton
              size="lg"
              :disabled="launchingId === featured.id || app.launchPhase === 'running'"
              @click="play(featured)"
            >
              <svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
              {{ launchingId === featured.id ? "Lanzando…" : "Jugar ahora" }}
            </BaseButton>
            <BaseButton size="lg" variant="secondary" @click="openInstance(featured)">
              Gestionar instancia
            </BaseButton>
          </div>

          <p v-if="launchError" class="mt-3 text-sm text-red-400">{{ launchError }}</p>
          <p v-else-if="app.launchPhase !== 'idle'" class="mt-3 text-sm text-pc-green">
            {{ app.launchMessage }}
          </p>
        </div>
      </div>
    </section>

    <section class="flex-1 border-t border-surface-3 bg-surface-1 p-8 space-y-8">
      <GameProfilesPanel />
      <FavoriteServersPanel />

      <div>
        <h2 class="mb-4 text-lg font-bold">Jugados recientemente</h2>
        <div class="grid grid-cols-2 gap-4 xl:grid-cols-3 2xl:grid-cols-4">
          <InstanceCard
            v-for="inst in instances.recent"
            :key="inst.id"
            :instance="inst"
            :selected="inst.id === instances.selectedId"
            @open="openInstance(inst)"
            @play="play(inst)"
          />
        </div>
      </div>
    </section>
  </div>
</template>
