<script setup lang="ts">
import IconCommodity from "@/components/icons/IconCommodity.vue";
import IconPackage from "@/components/icons/IconPackage.vue";
import IconPlane from "@/components/icons/IconPlane.vue";
import IconMapPin from "@/components/icons/IconMapPin.vue";
import IconDollarSign from "@/components/icons/IconDollarSign.vue";
import IconHeart from "@/components/icons/IconHeart.vue";
import IconInfoCircle from "@/components/icons/IconInfoCircle.vue";
import { useFavoritesStore } from "@/stores/favorites";
import { useDetailsStore } from "@/stores/details";
import type { UexResult } from "@/composables/useUex";

const props = defineProps<{
  result: UexResult;
  isActive: boolean;
}>();

const emit = defineEmits<{
  (e: "select"): void;
}>();

defineOptions({ inheritAttrs: false });

const favoritesStore = useFavoritesStore();
const detailsStore = useDetailsStore();

import { ref } from "vue";
const rootEl = ref<HTMLElement | null>(null);
defineExpose({ rootEl });

function toggleFavorite() {
  favoritesStore.toggleFavorite(props.result);
}

function openInDetails() {
  detailsStore.openEntity(props.result);
}
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
      <IconCommodity v-if="props.result.kind === 'commodity'"     class="w-3.5 h-3.5" />
      <IconPlane     v-else-if="props.result.kind === 'vehicle' || props.result.kind === 'ground vehicle'" class="w-3.5 h-3.5" />
      <IconMapPin    v-else-if="props.result.kind === 'location'"  class="w-3.5 h-3.5" />
      <IconPackage   v-else                                        class="w-3.5 h-3.5" />
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
      <!-- Favorite toggle -->
      <button
        @click.stop="toggleFavorite"
        class="p-1.5 rounded-lg transition-colors"
        :class="favoritesStore.isFavorite(props.result.id, props.result.kind)
          ? 'text-red-400 hover:text-red-300'
          : 'text-white/20 hover:text-red-400'"
        :title="favoritesStore.isFavorite(props.result.id, props.result.kind) ? 'Remove from favorites' : 'Add to favorites'"
      >
        <IconHeart class="w-3.5 h-3.5" :filled="favoritesStore.isFavorite(props.result.id, props.result.kind)" />
      </button>

      <!-- Open in Details -->
      <button
        @click.stop="openInDetails"
        class="p-1.5 rounded-lg text-white/20 hover:text-blue-400 transition-colors"
        title="Open in Details tab"
      >
        <IconInfoCircle class="w-3.5 h-3.5" />
      </button>

      <!-- View prices (commodities only) -->
      <button
        v-if="props.result.kind === 'commodity'"
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

      <!-- Generic select button for non-commodity types -->
      <button
        v-else
        @click.stop="emit('select')"
        class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border text-xs font-medium transition-colors"
        :class="isActive
          ? 'bg-blue-500/20 border-blue-500/40 text-blue-300 hover:bg-blue-500/30'
          : 'bg-blue-500/10 border-blue-500/20 text-blue-400 hover:bg-blue-500/20 hover:border-blue-500/40'"
        title="Select (Enter)"
      >
        <IconMapPin    v-if="props.result.kind === 'location'"  class="w-3 h-3" />
        <IconPlane     v-else-if="props.result.kind === 'vehicle' || props.result.kind === 'ground vehicle'" class="w-3 h-3" />
        <IconPackage   v-else                                   class="w-3 h-3" />
        View
        <span v-if="isActive" class="ml-0.5 opacity-50 font-normal text-[10px]">↵</span>
      </button>
    </div>
  </div>
</template>
