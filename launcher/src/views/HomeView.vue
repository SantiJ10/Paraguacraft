<script setup lang="ts">
import { computed } from "vue";
import { useInstancesStore } from "@/stores/instances";
import { useAccountsStore } from "@/stores/accounts";
import { useAppStore } from "@/stores/app";
import { useDownloadsStore } from "@/stores/downloads";
import BaseButton from "@/components/common/BaseButton.vue";
import InstanceCard from "@/components/common/InstanceCard.vue";
import { formatPlaytime } from "@/composables/useFormat";

const instances = useInstancesStore();
const accounts = useAccountsStore();
const app = useAppStore();
const downloads = useDownloadsStore();

const featured = computed(() => instances.selected);

function play(name: string) {
  app.setLaunch("preparing", `Preparando ${name}...`);
  downloads.enqueueDemo(`Verificando archivos de ${name}`);
  setTimeout(() => app.setLaunch("launching", `Lanzando ${name}...`), 1800);
  setTimeout(() => app.setLaunch("running", `Jugando ${name} (CPU del launcher suspendida)`), 3600);
}
</script>

<template>
  <div>
    <!-- Hero de la instancia destacada -->
    <section
      v-if="featured"
      class="relative flex min-h-[280px] flex-col justify-end overflow-hidden p-8"
      style="background: radial-gradient(120% 100% at 80% 0%, rgba(46, 204, 113, 0.18), transparent 60%), #0a0a0a"
    >
      <p class="text-sm font-semibold uppercase tracking-wider text-pc-green">
        Hola, {{ accounts.active?.username ?? "jugador" }}
      </p>
      <h1 class="mt-1 flex items-center gap-3 text-4xl font-black">
        <span class="font-emoji">{{ featured.icon }}</span>{{ featured.name }}
      </h1>
      <p class="mt-2 text-gray-400">
        Minecraft {{ featured.mcVersion }} &middot;
        <span class="capitalize">{{ featured.loader.replace("-", " ") }}</span> &middot;
        {{ formatPlaytime(featured.totalPlayMinutes) }}
      </p>
      <div class="mt-6 flex items-center gap-3">
        <BaseButton size="lg" @click="play(featured.name)">
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
          Jugar ahora
        </BaseButton>
        <BaseButton size="lg" variant="secondary">Editar instancia</BaseButton>
      </div>
    </section>

    <!-- Recientes -->
    <section class="p-8">
      <h2 class="mb-4 text-lg font-bold">Jugados recientemente</h2>
      <div class="grid grid-cols-2 gap-4 xl:grid-cols-3 2xl:grid-cols-4">
        <InstanceCard
          v-for="inst in instances.recent"
          :key="inst.id"
          :instance="inst"
          :selected="inst.id === instances.selectedId"
          @select="instances.select(inst.id)"
          @play="play(inst.name)"
        />
      </div>
    </section>
  </div>
</template>
