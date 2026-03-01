<script setup lang="ts">
import IconCommodity from "@/components/icons/IconCommodity.vue";
import IconPackage from "@/components/icons/IconPackage.vue";
import IconDollarSign from "@/components/icons/IconDollarSign.vue";
import type { UexResult } from "@/composables/useUex";

const props = defineProps<{
  result: UexResult;
  isActive: boolean;
}>();

const emit = defineEmits<{
  (e: "select"): void;
}>();

// Forward all attrs (tabindex, @keydown, @focus, etc.) to the root div
defineOptions({ inheritAttrs: false });

import { ref } from "vue";
const rootEl = ref<HTMLElement | null>(null);
defineExpose({ rootEl });
</script>

<template>
  <div
    ref="rootEl"
    v-bind="$attrs"
    class="group flex items-center gap-3 px-4 py-3 border-t border-white/5 transition-colors outline-none cursor-default"
    :class="isActive ? 'bg-white/8 ring-1 ring-inset ring-blue-500/30' : 'hover:bg-white/5 focus:bg-white/8'"
  >
    <!-- Entity icon -->
    <div
      class="flex-shrink-0 w-7 h-7 rounded-lg border flex items-center justify-center transition-colors"
      :class="isActive ? 'bg-blue-500/10 border-blue-500/20 text-blue-400' : 'bg-white/5 border-white/10 text-white/40'"
    >
      <IconCommodity v-if="props.result.kind === 'commodity'" class="w-3.5 h-3.5" />
      <IconPackage   v-else                                   class="w-3.5 h-3.5" />
    </div>

    <!-- Name + kind badge -->
    <div class="flex-1 min-w-0 flex items-center gap-2">
      <span class="text-white text-sm font-medium truncate">{{ result.name }}</span>
      <span class="flex-shrink-0 text-[10px] uppercase tracking-widest text-white/30 bg-white/5 border border-white/10 px-1.5 py-0.5 rounded">
        {{ result.kind }}
      </span>
    </div>

    <!-- Action buttons -->
    <div
      class="flex items-center gap-1 flex-shrink-0 transition-opacity duration-150"
      :class="isActive ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'"
    >
      <!-- View prices -->
      <button
        @click.stop="emit('select')"
        class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border text-xs font-medium transition-colors"
        :class="isActive
          ? 'bg-blue-500/20 border-blue-500/40 text-blue-300 hover:bg-blue-500/30'
          : 'bg-blue-500/10 border-blue-500/20 text-blue-400 hover:bg-blue-500/20 hover:border-blue-500/40'"
        title="View prices (Enter)"
      >
        <IconDollarSign class="w-3 h-3" />
        Prices
        <span v-if="isActive" class="ml-0.5 opacity-50 font-normal text-[10px]">↵</span>
      </button>

      <!-- Add to inventory (UI only) -->
      <button
        @click.stop
        class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 border border-white/10 hover:border-white/20 text-white/50 hover:text-white/80 text-xs font-medium transition-colors"
        title="Add to inventory"
      >
        <IconPackage class="w-3 h-3" />
        Add to Inventory
      </button>
    </div>
  </div>
</template>
