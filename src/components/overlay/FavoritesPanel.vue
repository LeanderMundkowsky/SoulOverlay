<script setup lang="ts">
import IconCommodity from "@/components/icons/IconCommodity.vue";
import IconPackage from "@/components/icons/IconPackage.vue";
import IconPlane from "@/components/icons/IconPlane.vue";
import IconMapPin from "@/components/icons/IconMapPin.vue";
import IconClose from "@/components/icons/IconClose.vue";
import { useFavoritesStore } from "@/stores/favorites";
import { useDragDrop } from "@/composables/useDragDrop";
import type { Favorite } from "@/stores/favorites";

const favoritesStore = useFavoritesStore();
const { payload, dragging } = useDragDrop();

const emit = defineEmits<{
  select: [fav: Favorite];
}>();

import { ref } from "vue";
const collapsedGroups = ref<Record<string, boolean>>({});
const dragOver = ref(false);

interface GroupConfig {
  kind: string;
  label: string;
}

const groups: GroupConfig[] = [
  { kind: "commodity", label: "Commodities" },
  { kind: "vehicle", label: "Vehicles" },
  { kind: "ground vehicle", label: "Ground Vehicles" },
  { kind: "item", label: "Items" },
  { kind: "location", label: "Locations" },
];

function onPointerEnter() {
  if (dragging.value && payload.value?.type === "entity") dragOver.value = true;
}

function onPointerLeave() {
  dragOver.value = false;
}

function onPointerUp() {
  if (!dragging.value || !payload.value) return;
  if (payload.value.type !== "entity") return;
  const entity = payload.value.data;
  if (!favoritesStore.isFavorite(entity.id, entity.kind)) {
    favoritesStore.addFavorite(entity);
  }
  dragOver.value = false;
}

function toggleGroup(kind: string) {
  collapsedGroups.value[kind] = !collapsedGroups.value[kind];
}

function favoritesForKind(kind: string): Favorite[] {
  return favoritesStore.favorites.filter((f) => f.kind === kind);
}

function openFavorite(fav: Favorite) {
  emit("select", fav);
}

async function removeFavorite(fav: Favorite) {
  await favoritesStore.removeFavorite(fav.id, fav.kind);
}
</script>

<template>
  <div
    class="w-full flex flex-col bg-[#1a1d24] border rounded-xl overflow-hidden transition-colors"
    :class="dragOver ? 'border-blue-500/50 bg-blue-500/5' : 'border-white/10'"
    @pointerenter="onPointerEnter"
    @pointerleave="onPointerLeave"
    @pointerup="onPointerUp"
  >
    <!-- Header -->
    <div class="px-3 py-2 border-b border-white/10">
      <span class="text-xs font-semibold text-white/50 uppercase tracking-widest">Favorites</span>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <template v-if="favoritesStore.favorites.length === 0">
        <div class="px-3 py-4 text-xs text-white/30 text-center">
          No favorites yet
        </div>
      </template>

      <template v-for="group in groups" :key="group.kind">
        <template v-if="favoritesForKind(group.kind).length > 0">
          <!-- Group header -->
          <div
            class="flex items-center gap-1.5 px-3 py-1.5 cursor-pointer select-none hover:bg-white/5"
            @click="toggleGroup(group.kind)"
          >
            <span
              class="text-xs text-white/30 transition-transform duration-150"
              :class="collapsedGroups[group.kind] ? '-rotate-90' : 'rotate-0'"
            >▼</span>
            <span class="text-xs uppercase tracking-widest text-white/40 font-semibold">{{ group.label }}</span>
            <span class="text-xs text-white/20 ml-auto">{{ favoritesForKind(group.kind).length }}</span>
          </div>

          <!-- Group items -->
          <div v-if="!collapsedGroups[group.kind]">
            <div
              v-for="fav in favoritesForKind(group.kind)"
              :key="`${fav.kind}-${fav.id}`"
              class="group flex items-center gap-2 px-3 py-1.5 cursor-pointer hover:bg-white/5 transition-colors"
              @click="openFavorite(fav)"
            >
              <!-- Entity icon -->
              <div class="shrink-0 w-5 h-5 rounded flex items-center justify-center bg-white/5 text-white/40">
                <IconCommodity v-if="fav.kind === 'commodity'" class="w-3 h-3" />
                <IconPlane v-else-if="fav.kind === 'vehicle' || fav.kind === 'ground vehicle'" class="w-3 h-3" />
                <IconMapPin v-else-if="fav.kind === 'location'" class="w-3 h-3" />
                <IconPackage v-else class="w-3 h-3" />
              </div>

              <!-- Name -->
              <span class="flex-1 text-xs text-white/70 truncate">{{ fav.name }}</span>

              <!-- Remove button -->
              <button
                class="shrink-0 opacity-0 group-hover:opacity-100 transition-opacity p-0.5 rounded hover:bg-white/10"
                title="Remove from favorites"
                @click.stop="removeFavorite(fav)"
              >
                <IconClose class="w-3 h-3 text-white/30 hover:text-red-400" />
              </button>
            </div>
          </div>
        </template>
      </template>
    </div>
  </div>
</template>
