<script setup lang="ts">
import launcherIcon from "@/assets/launcher-icon.png";

const features = [
  { icon: "\u26A1", title: "Rendimiento extremo", desc: "RAM minima y CPU suspendida al jugar." },
  { icon: "\uD83E\uDDE9", title: "Todos los loaders", desc: "Vanilla, Forge, Fabric, NeoForge, OptiFine, Quilt." },
  { icon: "\uD83D\uDED2", title: "Tienda integrada", desc: "Modrinth + CurseForge en un solo lugar." },
  { icon: "\uD83E\uDD16", title: "Paraguabot", desc: "Diagnostico de crashes y ayuda en todo el launcher." },
];
</script>

<template>
  <div class="flex flex-col items-center text-center">
    <div class="mb-5 flex h-20 w-20 items-center justify-center overflow-hidden rounded-2xl bg-pc-green/10 ring-1 ring-pc-green/25">
      <img :src="launcherIcon" alt="Paraguacraft" class="h-14 w-14 object-contain" />
    </div>
    <h1 class="text-3xl font-black tracking-tight">Bienvenido a Paraguacraft</h1>
    <p class="mt-2 max-w-md text-gray-400">
      El launcher de Minecraft Java mas rapido y completo. Configuremos todo en unos pocos pasos.
    </p>

    <div class="mt-8 grid w-full max-w-xl grid-cols-2 gap-3">
      <div
        v-for="f in features"
        :key="f.title"
        class="flex items-start gap-3 rounded-xl border border-surface-4 bg-surface-2 p-4 text-left"
      >
        <span class="text-2xl font-emoji">{{ f.icon }}</span>
        <div>
          <p class="font-semibold">{{ f.title }}</p>
          <p class="text-xs text-gray-500">{{ f.desc }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
