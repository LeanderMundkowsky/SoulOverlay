<script setup lang="ts">
import IconClose from "@/components/icons/IconClose.vue";

defineProps<{
  title: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();
</script>

<template>
  <div class="flex items-center justify-between px-5 py-4 border-b border-white/10">
    <h2 class="text-white font-semibold text-base">{{ title }}</h2>
    <button
      @click="emit('close')"
      class="text-white/40 hover:text-white transition-colors"
      title="Close"
    >
      <IconClose class="w-5 h-5" />
    </button>
  </div>
</template>
