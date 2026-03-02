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
    class="group flex flex-col px-4 py-2 border-t border-white/5 transition-colors outline-none cursor-default"
    :class="isActive ? 'bg-white/8 ring-1 ring-inset ring-blue-500/30' : 'hover:bg-white/5 focus:bg-white/8'"
    @dblclick.stop="emit('select')"
  >
    <!-- Row 1: icon + name + kind -->
    <div class="flex items-center gap-3">
      <div
        class="flex-shrink-0 w-7 h-7 rounded-lg border flex items-center justify-center transition-colors"
        :class="isActive ? 'bg-blue-500/10 border-blue-500/20 text-blue-400' : 'bg-white/5 border-white/10 text-white/40'"
      >
        <IconCommodity v-if="props.result.kind === 'commodity'"     class="w-3.5 h-3.5" />
        <IconPlane     v-else-if="props.result.kind === 'vehicle' || props.result.kind === 'ground vehicle'" class="w-3.5 h-3.5" />
        <IconMapPin    v-else-if="props.result.kind === 'location'"  class="w-3.5 h-3.5" />
        <IconPackage   v-else                                        class="w-3.5 h-3.5" />
      </div>

      <span class="flex-1 text-white text-sm font-medium leading-snug">{{ result.name }}</span>

      <span
        class="flex-shrink-0 text-xs uppercase tracking-widest font-medium"
        :class="{
          'text-blue-400/50':   result.kind === 'commodity',
          'text-purple-400/50': result.kind === 'vehicle' || result.kind === 'ground vehicle',
          'text-green-400/50':  result.kind === 'location',
          'text-yellow-400/50': result.kind === 'item',
          'text-white/25':      !['commodity','vehicle','ground vehicle','location','item'].includes(result.kind),
        }"
      >{{ result.kind }}</span>
    </div>

    <!-- Row 2: action buttons — expand only when active (click/keyboard), no hover expansion -->
    <div
      class="overflow-hidden transition-all duration-150"
      :class="isActive ? 'max-h-10 mt-1.5 opacity-100' : 'max-h-0 mt-0 opacity-0'"
    >
      <div class="flex items-center gap-1 pl-10">
        <button
          @click.stop="toggleFavorite"
          class="flex items-center gap-1.5 px-2 py-0.5 rounded-lg text-xs transition-colors"
          :class="favoritesStore.isFavorite(props.result.id, props.result.kind)
            ? 'text-red-400 hover:text-red-300 bg-red-400/10'
            : 'text-white/30 hover:text-red-400 hover:bg-red-400/10'"
        >
          <IconHeart class="w-3 h-3" :filled="favoritesStore.isFavorite(props.result.id, props.result.kind)" />
          {{ favoritesStore.isFavorite(props.result.id, props.result.kind) ? 'Unfavorite' : 'Favorite' }}
        </button>

        <button
          @click.stop="openInDetails"
          class="flex items-center gap-1.5 px-2 py-0.5 rounded-lg text-xs text-white/30 hover:text-blue-400 hover:bg-blue-400/10 transition-colors"
        >
          <IconInfoCircle class="w-3 h-3" />
          Details
        </button>

        <button
          @click.stop="emit('select')"
          class="flex items-center gap-1.5 px-2 py-0.5 rounded-lg border text-xs font-medium ml-auto transition-colors bg-blue-500/20 border-blue-500/40 text-blue-300 hover:bg-blue-500/30"
        >
          <IconDollarSign v-if="props.result.kind === 'commodity'" class="w-3 h-3" />
          <IconMapPin     v-else-if="props.result.kind === 'location'" class="w-3 h-3" />
          <IconPlane      v-else-if="props.result.kind === 'vehicle' || props.result.kind === 'ground vehicle'" class="w-3 h-3" />
          <IconPackage    v-else class="w-3 h-3" />
          {{ props.result.kind === 'commodity' ? 'Prices' : 'View' }}
          <span class="opacity-50 font-normal">↵</span>
        </button>
      </div>
    </div>
  </div>
</template>
