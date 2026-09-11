<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import SkinAvatar from "@/components/account/SkinAvatar.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import InstanceIcon from "@/components/instance/InstanceIcon.vue";
import { useAccountsStore } from "@/stores/accounts";
import { useSkinsStore } from "@/stores/skins";
import { useInstancesStore } from "@/stores/instances";
import { useAppStore } from "@/stores/app";
import { useMusicStore } from "@/stores/music";
import { useI18n } from "@/composables/useI18n";
import type { Instance } from "@/lib/types";

const emit = defineEmits<{ play: [inst: Instance] }>();

const router = useRouter();
const { t } = useI18n();
const accounts = useAccountsStore();
const skins = useSkinsStore();
const instances = useInstancesStore();
const app = useAppStore();
const music = useMusicStore();

const featured = computed(() => instances.recent[0] ?? instances.selected);
const avatarKey = computed(
  () => `${skins.revision}:${skins.activeSkin?.skinUrl ?? skins.activeSkin?.avatarUrl ?? ""}`,
);

const launching = computed(
  () =>
    app.launchPhase === "running" ||
    app.launchPhase === "preparing" ||
    app.launchPhase === "downloading" ||
    app.launchPhase === "launching",
);

function goSkins() {
  router.push({ name: "skins" });
}

function goSettings() {
  router.push({ name: "settings" });
}
</script>

<template>
  <section class="glass-card space-y-4 rounded-xl border border-surface-4 p-4">
    <h2 class="text-lg font-bold">{{ t("home_status") }}</h2>

    <button type="button" class="flex w-full items-center gap-3 rounded-lg bg-surface-3/60 p-2 text-left hover:bg-surface-3" @click="goSkins">
      <SkinAvatar
        :key="avatarKey"
        size="md"
        :uuid="accounts.active?.uuid ?? skins.activeSkin?.uuid"
        :username="accounts.active?.username ?? skins.activeSkin?.username"
        :avatar-url="skins.activeSkin?.avatarUrl"
        :avatar-data-url="skins.activeSkin?.avatarDataUrl"
        :local-avatar-path="skins.activeSkin?.localAvatarPath"
        :loading="skins.loading && !accounts.active"
      />
      <div class="min-w-0">
        <p class="truncate text-sm font-semibold">{{ accounts.active?.username ?? "Sin cuenta" }}</p>
        <p class="text-[10px] font-bold uppercase tracking-wide" :class="accounts.active?.premium ? 'text-pc-green' : 'text-gray-500'">
          {{ accounts.active?.premium ? "Premium" : accounts.active ? "Offline" : "Invitado" }}
        </p>
      </div>
    </button>

    <div v-if="featured" class="rounded-lg bg-surface-3/60 p-3">
      <div class="mb-2 flex items-center gap-2">
        <InstanceIcon :icon="featured.icon" size="sm" />
        <p class="truncate text-sm font-semibold">{{ featured.name }}</p>
      </div>
      <p class="mb-3 text-xs text-gray-400">Minecraft {{ featured.mcVersion }}</p>
      <BaseButton class="w-full" :disabled="launching" @click="emit('play', featured)">
        {{ launching ? app.launchMessage : t("play_now") }}
      </BaseButton>
    </div>

    <div class="rounded-lg bg-surface-3/60 p-3">
      <p class="text-[10px] font-bold uppercase tracking-wide text-gray-500">{{ t("home_now_playing") }}</p>
      <div class="mt-2 flex items-center gap-3">
        <img
          v-if="music.overlayImage"
          :src="music.overlayImage"
          alt=""
          class="h-12 w-12 shrink-0 rounded-md object-cover"
        />
        <div
          v-else
          class="flex h-12 w-12 shrink-0 items-center justify-center rounded-md bg-surface-4 text-gray-500"
          aria-hidden="true"
        >
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor"><path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3h-6z" /></svg>
        </div>
        <p class="min-w-0 truncate text-sm" :class="music.isPlaying ? 'text-white' : 'text-gray-500'">
          {{ music.overlayLabel || t("home_nothing_playing") }}
        </p>
      </div>
    </div>

    <button
      v-if="app.updateInfo?.updateAvailable"
      type="button"
      class="w-full rounded-lg border border-pc-green/30 bg-pc-green/10 px-3 py-2 text-left text-xs text-pc-green"
      @click="goSettings"
    >
      Nueva versión {{ app.updateInfo.latestVersion }}
    </button>
  </section>
</template>
