<script setup lang="ts">
import { useUpdateStore } from "@/stores/update";
import IconInfoCircle from "@/components/icons/IconInfoCircle.vue";

const updateStore = useUpdateStore();
</script>

<template>
  <div
    v-if="updateStore.updateAvailable && !updateStore.dismissed"
    class="flex items-center gap-3 px-4 py-2 bg-blue-950/80 border-b border-blue-500/30 text-sm"
  >
    <IconInfoCircle class="w-4 h-4 text-blue-400 flex-shrink-0" />
    <span class="text-blue-200 flex-1">
      Update <span class="font-semibold text-blue-100">v{{ updateStore.updateInfo?.version }}</span> is available
    </span>
    <button
      @click="updateStore.installUpdate()"
      :disabled="updateStore.installing"
      class="px-3 py-1 bg-blue-600 hover:bg-blue-500 disabled:bg-blue-600/50 text-white text-xs font-medium rounded-md transition-colors"
    >
      {{ updateStore.installing ? "Installing..." : "Install" }}
    </button>
    <button
      @click="updateStore.dismiss()"
      class="text-white/30 hover:text-white/60 text-xs transition-colors"
    >
      Dismiss
    </button>
  </div>
</template>
