<script setup lang="ts">
defineProps<{ title?: string }>();
const open = defineModel<boolean>("open", { default: false });
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="open"
        class="fixed inset-0 z-[300] flex items-center justify-center bg-black/70 p-6"
        @click.self="open = false"
      >
        <div class="w-full max-w-lg rounded-2xl border border-surface-5 bg-surface-2 shadow-2xl">
          <header
            v-if="title"
            class="flex items-center justify-between border-b border-surface-4 px-6 py-4"
          >
            <h3 class="text-lg font-bold">{{ title }}</h3>
            <button class="text-gray-500 hover:text-white" @click="open = false">&times;</button>
          </header>
          <div class="p-6">
            <slot />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
