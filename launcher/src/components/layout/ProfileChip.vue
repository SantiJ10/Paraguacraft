<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import SkinAvatar from "@/components/account/SkinAvatar.vue";
import { useAccountsStore } from "@/stores/accounts";
import { useSkinsStore } from "@/stores/skins";

const router = useRouter();
const accounts = useAccountsStore();
const skins = useSkinsStore();

const open = ref(false);
const root = ref<HTMLElement | null>(null);

const active = computed(() => accounts.active);
const avatarKey = computed(
  () => `${skins.revision}:${skins.activeSkin?.skinUrl ?? skins.activeSkin?.avatarUrl ?? ""}`,
);

function toggle() {
  open.value = !open.value;
}

function close() {
  open.value = false;
}

function goSettings(section?: string) {
  close();
  router.push({ name: "settings", query: section ? { tab: section } : {} });
}

function onDocClick(e: MouseEvent) {
  if (!open.value) return;
  if (root.value && !root.value.contains(e.target as Node)) close();
}

onMounted(() => document.addEventListener("click", onDocClick));
onUnmounted(() => document.removeEventListener("click", onDocClick));

watch(
  () => accounts.active?.id,
  () => {
    void skins.refresh();
  },
);
</script>

<template>
  <div ref="root" class="relative">
    <button
      type="button"
      class="flex items-center gap-2.5 rounded-lg border border-surface-4 bg-surface-2/80 px-2.5 py-1.5 transition-colors hover:border-surface-5 hover:bg-surface-3"
      @click.stop="toggle"
    >
      <SkinAvatar
        :key="avatarKey"
        size="sm"
        :uuid="active?.uuid ?? skins.activeSkin?.uuid"
        :username="active?.username ?? skins.activeSkin?.username"
        :avatar-url="skins.activeSkin?.avatarUrl"
        :avatar-data-url="skins.activeSkin?.avatarDataUrl"
        :local-avatar-path="skins.activeSkin?.localAvatarPath"
        :loading="skins.loading && !active"
      />
      <div class="hidden min-w-0 text-left sm:block">
        <p class="truncate text-sm font-semibold leading-tight">
          {{ active?.username ?? "Sin cuenta" }}
        </p>
        <p
          class="text-[10px] font-bold uppercase tracking-wide"
          :class="active?.premium ? 'text-pc-green' : 'text-gray-500'"
        >
          {{ active?.premium ? "Premium" : active ? "Offline" : "Invitado" }}
        </p>
      </div>
      <svg
        class="hidden h-4 w-4 text-gray-500 sm:block"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M6 9l6 6 6-6" />
      </svg>
    </button>

    <Transition name="pop">
      <div
        v-if="open"
        class="absolute right-0 top-full z-50 mt-2 w-56 overflow-hidden rounded-xl border border-surface-4 bg-surface-2 shadow-2xl"
        @click.stop
      >
        <div class="border-b border-surface-4 px-3 py-2.5">
          <p class="truncate text-sm font-bold">{{ active?.username ?? "Sin cuenta" }}</p>
          <p class="text-xs text-gray-500">
            {{ active?.premium ? "Cuenta Premium" : active ? "Cuenta offline" : "Agrega una cuenta" }}
          </p>
        </div>
        <button type="button" class="profile-menu-item" @click="goSettings('accounts')">
          Gestionar cuentas
        </button>
        <button type="button" class="profile-menu-item" @click="router.push('/skins'); close()">
          Mis skins
        </button>
        <button type="button" class="profile-menu-item" @click="goSettings()">
          Ajustes
        </button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.profile-menu-item {
  @apply flex w-full px-3 py-2.5 text-left text-sm text-gray-300 transition-colors hover:bg-surface-3 hover:text-white;
}

.pop-enter-active,
.pop-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.pop-enter-from,
.pop-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
