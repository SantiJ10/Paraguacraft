<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "@/lib/ipc";
import { useInstancesStore } from "@/stores/instances";
import { useAppStore } from "@/stores/app";
import type { Instance, InstanceMeta, GcType } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";

const props = defineProps<{ instance: Instance }>();
const emit = defineEmits<{ (e: "close"): void }>();

const instances = useInstancesStore();
const app = useAppStore();

const meta = ref<InstanceMeta | null>(null);
const ramMb = ref(4096);
const gc = ref<GcType>("Auto");
const jvmArgs = ref("");
const javaPath = ref("");
const autoManaged = ref(true);
const busy = ref(false);
const error = ref<string | null>(null);

const gcOptions: GcType[] = ["Auto", "G1GC", "ZGC", "Shenandoah"];
const maxRam = (app.hardware?.ramGb ?? 16) * 1024;

onMounted(async () => {
  try {
    const m = await api.getInstanceMeta(props.instance.id);
    meta.value = m;
    ramMb.value = m.ramMb || 4096;
    gc.value = (m.gc as GcType) ?? "Auto";
    jvmArgs.value = m.jvmArgs ?? "";
    javaPath.value = m.javaPath ?? "";
    autoManaged.value = m.autoManaged;
  } catch (e) {
    error.value = String(e);
  }
});

async function save() {
  busy.value = true;
  error.value = null;
  try {
    await api.setInstanceConfig({
      id: props.instance.id,
      ramMb: ramMb.value,
      jvmArgs: jvmArgs.value || null,
      gc: gc.value,
      javaPath: javaPath.value || null,
    });
    await instances.load(true);
    emit("close");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function backToAuto() {
  busy.value = true;
  try {
    const m = await api.setInstanceAutoManaged(props.instance.id, true);
    autoManaged.value = m.autoManaged;
    jvmArgs.value = "";
    gc.value = "Auto";
    await instances.load(true);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" @click.self="emit('close')">
    <div class="w-full max-w-lg rounded-2xl border border-surface-4 bg-surface-2 p-6 shadow-2xl">
      <div class="mb-1 flex items-center justify-between">
        <h3 class="text-lg font-bold">RAM &amp; JVM — {{ instance.name }}</h3>
        <button class="text-gray-500 hover:text-white" @click="emit('close')">&times;</button>
      </div>
      <p class="mb-4 text-xs" :class="autoManaged ? 'text-pc-green' : 'text-amber-400'">
        {{ autoManaged ? "Autogestionado por hardware" : "Configuración manual (no se sobrescribe)" }}
      </p>

      <div class="space-y-4">
        <label class="block">
          <span class="mb-1 flex justify-between text-sm text-gray-300">
            <span>Memoria RAM</span><span class="font-semibold text-pc-green">{{ (ramMb / 1024).toFixed(1) }} GB</span>
          </span>
          <input v-model.number="ramMb" type="range" min="1024" :max="maxRam" step="512" class="w-full accent-pc-green" />
        </label>

        <label class="block">
          <span class="mb-1 block text-sm text-gray-300">Garbage Collector</span>
          <select v-model="gc" class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green">
            <option v-for="g in gcOptions" :key="g" :value="g">{{ g }}</option>
          </select>
        </label>

        <label class="block">
          <span class="mb-1 block text-sm text-gray-300">Argumentos JVM extra (opcional)</span>
          <input
            v-model="jvmArgs"
            type="text"
            placeholder="-Dsun.java2d.opengl=true"
            class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
          />
        </label>

        <label class="block">
          <span class="mb-1 block text-sm text-gray-300">Ruta de Java (opcional)</span>
          <input
            v-model="javaPath"
            type="text"
            placeholder="Auto según la versión"
            class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
          />
        </label>

        <p v-if="error" class="text-xs text-red-400">{{ error }}</p>

        <div class="flex items-center justify-between pt-2">
          <BaseButton v-if="!autoManaged" variant="ghost" :disabled="busy" @click="backToAuto">
            Volver a automático
          </BaseButton>
          <span v-else></span>
          <div class="flex gap-2">
            <BaseButton variant="ghost" @click="emit('close')">Cancelar</BaseButton>
            <BaseButton :disabled="busy" @click="save">{{ busy ? "Guardando…" : "Guardar" }}</BaseButton>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
