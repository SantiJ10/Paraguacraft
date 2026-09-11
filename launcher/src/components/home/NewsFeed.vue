<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api, openUrl } from "@/lib/ipc";
import { useI18n } from "@/composables/useI18n";
import type { HomeNewsItem } from "@/lib/types";

const { t } = useI18n();
const items = ref<HomeNewsItem[]>([]);
const loading = ref(true);
const filter = ref<"all" | "launcher" | "mobile">("all");

const MAX_NEWS = 3;

const filtered = computed(() => {
  const list = filter.value === "all" ? items.value : items.value.filter((n) => n.kind === filter.value);
  return list.slice(0, MAX_NEWS);
});

function formatDate(iso: string) {
  if (!iso) return "";
  try {
    return new Date(iso).toLocaleDateString(undefined, {
      day: "numeric",
      month: "short",
      year: "numeric",
    });
  } catch {
    return iso.slice(0, 10);
  }
}

function kindLabel(kind: string) {
  if (kind === "mobile") return "Android";
  if (kind === "launcher") return "PC";
  return kind;
}

async function openItem(item: HomeNewsItem) {
  if (item.htmlUrl) await openUrl(item.htmlUrl);
}

onMounted(async () => {
  try {
    items.value = await api.listHomeNews();
  } catch {
    items.value = [];
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <section>
    <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
      <h2 class="text-lg font-bold">{{ t("home_news") }}</h2>
      <div class="flex gap-1 rounded-lg bg-surface-3/80 p-0.5 text-xs font-semibold">
        <button
          v-for="opt in (['all', 'launcher', 'mobile'] as const)"
          :key="opt"
          type="button"
          class="rounded-md px-2.5 py-1 transition"
          :class="filter === opt ? 'bg-surface-5 text-white' : 'text-gray-400 hover:text-white'"
          @click="filter = opt"
        >
          {{ opt === "all" ? t("home_news_all") : opt === "launcher" ? t("home_news_launcher") : t("home_news_mobile") }}
        </button>
      </div>
    </div>

    <div v-if="loading" class="space-y-2">
      <div v-for="n in MAX_NEWS" :key="n" class="h-12 animate-pulse rounded-xl bg-surface-3" />
    </div>
    <p v-else-if="!filtered.length" class="text-sm text-gray-500">
      {{ t("home_news_empty") }}
    </p>
    <div v-else class="space-y-1.5">
      <button
        v-for="item in filtered"
        :key="item.tag"
        type="button"
        class="glass-card w-full rounded-xl border border-surface-4 px-3 py-2 text-left transition hover:border-pc-green/50"
        @click="openItem(item)"
      >
        <div class="flex items-center gap-2">
          <span
            class="rounded-md px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide"
            :class="item.kind === 'mobile' ? 'bg-sky-500/15 text-sky-300' : 'bg-pc-green/15 text-pc-green'"
          >
            {{ kindLabel(item.kind) }}
          </span>
          <span class="truncate text-sm font-semibold text-white">{{ item.name }}</span>
          <span class="ml-auto shrink-0 text-[11px] text-gray-500">{{ formatDate(item.publishedAt) }}</span>
        </div>
        <p v-if="item.body" class="mt-1 line-clamp-2 text-[12px] leading-snug text-gray-400">{{ item.body }}</p>
      </button>
    </div>
  </section>
</template>
