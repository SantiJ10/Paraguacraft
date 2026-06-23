<script setup lang="ts">
import { ref } from "vue";
import type { VersionCardModel } from "@/lib/versionCatalog";
import { versionCardImageUrl } from "@/lib/versionCatalog";

const props = defineProps<{
  card: VersionCardModel;
  selected?: boolean;
}>();

defineEmits<{ select: [card: VersionCardModel] }>();

const imgFailed = ref(false);
const imgUrl = versionCardImageUrl(props.card.imageKey);

const cardStyle = {
  borderColor: props.selected
    ? props.card.accent ?? "#2ECC71"
    : props.card.accent
      ? `${props.card.accent}55`
      : undefined,
  background:
    props.card.kind === "bedrock"
      ? "#0A1628"
      : props.card.kind === "snapshots"
        ? "#1A0F26"
        : props.card.kind === "alpha_beta"
          ? "#26190F"
          : "#1A1A1A",
};
</script>

<template>
  <button
    type="button"
    class="group relative h-[168px] overflow-hidden rounded-xl text-left transition-all duration-200"
    :class="selected ? 'ring-2 ring-pc-green scale-[1.02]' : 'hover:scale-[1.02] hover:ring-1 hover:ring-surface-5'"
    :style="cardStyle"
    @click="$emit('select', card)"
  >
    <img
      v-if="!imgFailed"
      :src="imgUrl"
      :alt="card.title"
      class="absolute inset-0 h-full w-full object-cover opacity-75 transition duration-500 group-hover:scale-105"
      @error="imgFailed = true"
    />
    <div class="absolute inset-0 bg-gradient-to-t from-black/95 via-black/50 to-black/20" />

    <div class="absolute inset-0 flex flex-col justify-between p-4">
      <div class="flex items-start justify-between gap-2">
        <h3
          class="text-xl font-black leading-tight drop-shadow-md"
          :class="card.accent ? '' : 'text-white'"
          :style="card.accent ? { color: card.accent } : undefined"
        >
          {{ card.title }}
        </h3>
        <span
          v-if="card.subs.length && card.kind !== 'bedrock'"
          class="shrink-0 rounded px-2 py-0.5 text-[10px] font-black"
          :class="card.accent ? '' : 'bg-pc-green/20 text-pc-green'"
          :style="card.accent ? { background: `${card.accent}33`, color: card.accent } : undefined"
        >
          {{ card.subs.length }}
        </span>
      </div>

      <div>
        <p v-if="card.subtitle" class="mb-1 text-xs text-gray-400">{{ card.subtitle }}</p>
        <div v-if="card.kind === 'installed' && card.instances?.length" class="flex flex-wrap gap-1">
          <span
            v-for="inst in card.instances.slice(0, 4)"
            :key="inst.id"
            class="rounded bg-pc-green/20 px-1.5 py-0.5 text-[10px] font-bold text-pc-green"
          >
            {{ inst.mcVersion }} · {{ inst.loader }}
          </span>
          <span v-if="(card.instances?.length ?? 0) > 4" class="text-[10px] text-gray-500">
            +{{ (card.instances?.length ?? 0) - 4 }}
          </span>
        </div>
        <p v-else-if="card.kind === 'bedrock'" class="text-xs text-gray-400">
          Xbox / Microsoft Store
        </p>
      </div>
    </div>
  </button>
</template>
