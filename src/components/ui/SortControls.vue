<script setup lang="ts">
import { ref } from "vue";
import type { SortOption } from "@/utils/sorting";
import type { PriceEntry } from "@/bindings";

defineProps<{
  options: SortOption[];
  currentKey: keyof PriceEntry;
  ascending: boolean;
}>();

const emit = defineEmits<{
  (e: "select", opt: SortOption): void;
  (e: "toggleDirection"): void;
}>();

const dropdownOpen = ref(false);

function pickOption(opt: SortOption) {
  emit("select", opt);
  dropdownOpen.value = false;
}

function currentLabel(options: SortOption[], key: keyof PriceEntry): string {
  const found = options.find((o) => o.key === key);
  return found ? found.label : "Price";
}
</script>

<template>
  <div class="flex items-center gap-0.5">
    <div class="relative flex items-center">
      <button
        @click.stop="dropdownOpen = !dropdownOpen"
        class="px-1.5 py-0.5 rounded text-[0.625rem] text-white/30 hover:text-white/50 hover:bg-white/10 transition-colors"
      >
        {{ currentLabel(options, currentKey) }}
      </button>
      <div
        v-if="dropdownOpen"
        class="absolute right-0 top-full mt-1 bg-gray-900 border border-white/10 rounded-lg shadow-lg z-20 min-w-[7.5rem] py-1"
      >
        <button
          v-for="opt in options"
          :key="opt.key"
          @click.stop="pickOption(opt)"
          class="w-full text-left px-3 py-1.5 text-xs transition-colors"
          :class="currentKey === opt.key ? 'text-white bg-white/10' : 'text-white/50 hover:text-white/80 hover:bg-white/5'"
        >
          {{ opt.label }}
        </button>
      </div>
    </div>
    <button
      @click.stop="emit('toggleDirection')"
      class="px-1 py-0.5 rounded text-[0.625rem] text-white/30 hover:text-white/50 hover:bg-white/10 transition-colors"
    >
      {{ ascending ? '▲' : '▼' }}
    </button>
  </div>
</template>
