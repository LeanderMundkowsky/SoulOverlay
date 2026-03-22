<script setup lang="ts">
import { watch, onMounted } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import IconHeart from "@/components/icons/IconHeart.vue";
import WikiSpecsSection from "@/components/overlay/WikiSpecsSection.vue";
import { useUex } from "@/composables/useUex";
import { useFavoritesStore } from "@/stores/favorites";
import { proxyImageUrl } from "@/utils/imageProxy";
import type { EntityInfo } from "@/composables/useUex";

const props = defineProps<{
  entityId: string;
  entityKind: string;
  entityName?: string;
  entitySource?: string;
  entityUuid?: string;
}>();

const { entityInfo, entityInfoLoading, entityInfoError, getEntityInfo } = useUex();
const favoritesStore = useFavoritesStore();

function fetchInfo() {
  getEntityInfo(props.entityKind, props.entityId);
}

onMounted(() => { fetchInfo(); });
watch(() => props.entityId, () => { fetchInfo(); });

function toggleFavorite() {
  if (entityInfo.value) {
    favoritesStore.toggleFavorite({
      id: entityInfo.value.id,
      name: entityInfo.value.name,
      kind: entityInfo.value.kind,
      slug: entityInfo.value.slug,
      uuid: entityInfo.value.uuid ?? "",
      source: props.entitySource ?? "uex",
    });
  } else if (props.entitySource === "wiki") {
    favoritesStore.toggleFavorite({
      id: props.entityId,
      name: props.entityName ?? "",
      kind: props.entityKind,
      slug: "",
      uuid: props.entityUuid ?? props.entityId,
      source: "wiki",
    });
  }
}

function formatDimensions(info: EntityInfo): string {
  const parts: string[] = [];
  if (info.length) parts.push(`${info.length}m`);
  if (info.width) parts.push(`${info.width}m`);
  if (info.height) parts.push(`${info.height}m`);
  return parts.join(" × ");
}

function formatPrice(val: number): string {
  return val.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}
</script>

<template>
  <div class="bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
    <!-- Loading -->
    <div v-if="entityInfoLoading" class="px-4 py-3 flex justify-center">
      <LoadingSpinner text="Loading info..." />
    </div>

    <!-- Error — show message to help diagnose -->
    <div v-else-if="entityInfoError" class="px-4 py-2 text-xs text-white/30">{{ entityInfoError }}</div>

    <!-- Wiki-only entity (no UEX data) -->
    <template v-else-if="entitySource === 'wiki' && !entityInfo">
      <div class="px-4 py-3 space-y-2">
        <div class="flex items-center gap-2 flex-wrap text-xs">
          <span class="text-white font-semibold text-sm">{{ entityName }}</span>
          <button @click="toggleFavorite" class="p-0.5 rounded transition-colors" :class="favoritesStore.isFavorite(entityId, entityKind) ? 'text-red-400 hover:text-red-300' : 'text-white/20 hover:text-red-400'" :title="favoritesStore.isFavorite(entityId, entityKind) ? 'Remove from favorites' : 'Add to favorites'">
            <IconHeart class="w-3.5 h-3.5" :filled="favoritesStore.isFavorite(entityId, entityKind)" />
          </button>
          <span class="px-2 py-0.5 rounded bg-teal-500/15 text-teal-400 font-medium">{{ entityKind }}</span>
        </div>
        <div class="text-xs text-white/30">Not tracked by UEX — data from Star Citizen Wiki</div>
      </div>
    </template>

    <!-- Commodity info -->
    <template v-else-if="entityInfo && entityInfo.kind === 'commodity'">
      <div class="px-4 py-3 space-y-2">
        <div class="flex items-center gap-2 flex-wrap text-xs">
          <span class="text-white font-semibold text-sm">{{ entityInfo.name }}</span>
          <button @click="toggleFavorite" class="p-0.5 rounded transition-colors" :class="favoritesStore.isFavorite(entityInfo.id, entityInfo.kind) ? 'text-red-400 hover:text-red-300' : 'text-white/20 hover:text-red-400'" :title="favoritesStore.isFavorite(entityInfo.id, entityInfo.kind) ? 'Remove from favorites' : 'Add to favorites'">
            <IconHeart class="w-3.5 h-3.5" :filled="favoritesStore.isFavorite(entityInfo.id, entityInfo.kind)" />
          </button>
          <span v-if="entityInfo.commodity_kind" class="px-2 py-0.5 rounded bg-white/10 text-white/70 font-medium">{{ entityInfo.commodity_kind }}</span>
          <span v-if="entityInfo.code" class="text-white/40 font-mono">{{ entityInfo.code }}</span>
          <span v-if="entityInfo.is_illegal" class="px-1.5 py-0.5 rounded bg-red-500/20 text-red-400 font-medium">Illegal</span>
          <span v-if="entityInfo.is_harvestable" class="px-1.5 py-0.5 rounded bg-green-500/20 text-green-400 font-medium">Harvestable</span>
          <span v-if="entityInfo.is_mineral" class="px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400 font-medium">Mineral</span>
          <span v-if="entityInfo.is_raw" class="px-1.5 py-0.5 rounded bg-yellow-500/20 text-yellow-400 font-medium">Raw</span>
          <span v-if="entityInfo.is_refined" class="px-1.5 py-0.5 rounded bg-purple-500/20 text-purple-400 font-medium">Refined</span>
        </div>
        <div class="flex items-center gap-4 text-xs text-white/50">
          <span v-if="entityInfo.is_buyable">Buyable</span>
          <span v-if="entityInfo.is_sellable">Sellable</span>
          <span v-if="entityInfo.weight_scu">Weight <span class="text-white/70">{{ entityInfo.weight_scu }} kg/SCU</span></span>
          <span v-if="entityInfo.avg_buy">Avg Buy <span class="text-green-400">{{ formatPrice(entityInfo.avg_buy) }}</span></span>
          <span v-if="entityInfo.avg_sell">Avg Sell <span class="text-blue-400">{{ formatPrice(entityInfo.avg_sell) }}</span></span>
          <button v-if="entityInfo.wiki" @click="openUrl(entityInfo.wiki!)" class="text-blue-400/60 hover:text-blue-400 transition-colors cursor-pointer">Wiki ↗</button>
        </div>
      </div>
    </template>

    <!-- Item info -->
    <template v-else-if="entityInfo && entityInfo.kind === 'item'">
      <div class="px-4 py-3 space-y-2">
        <div class="flex items-center gap-2 flex-wrap text-xs">
          <span class="text-white font-semibold text-sm">{{ entityInfo.name }}</span>
          <button @click="toggleFavorite" class="p-0.5 rounded transition-colors" :class="favoritesStore.isFavorite(entityInfo.id, entityInfo.kind) ? 'text-red-400 hover:text-red-300' : 'text-white/20 hover:text-red-400'" :title="favoritesStore.isFavorite(entityInfo.id, entityInfo.kind) ? 'Remove from favorites' : 'Add to favorites'">
            <IconHeart class="w-3.5 h-3.5" :filled="favoritesStore.isFavorite(entityInfo.id, entityInfo.kind)" />
          </button>
          <span v-if="entityInfo.section" class="px-2 py-0.5 rounded bg-white/10 text-white/70 font-medium">{{ entityInfo.section }}</span>
          <span v-if="entityInfo.category" class="px-2 py-0.5 rounded bg-white/8 text-white/50">{{ entityInfo.category }}</span>
        </div>
        <div class="flex items-center gap-4 text-xs text-white/50">
          <span v-if="entityInfo.company_name">By <span class="text-white/70">{{ entityInfo.company_name }}</span></span>
          <span v-if="entityInfo.size">Size <span class="text-white/70">{{ entityInfo.size }}</span></span>
          <span v-if="entityInfo.color">Color <span class="text-white/70">{{ entityInfo.color }}</span></span>
          <span v-if="entityInfo.game_version" class="text-white/30">v{{ entityInfo.game_version }}</span>
        </div>
      </div>
    </template>

    <!-- Vehicle info -->
    <template v-else-if="entityInfo && (entityInfo.kind === 'vehicle' || entityInfo.kind === 'ground vehicle')">
      <div class="relative overflow-hidden">
        <img
          v-if="entityInfo.url_photo"
          :src="proxyImageUrl(entityInfo.url_photo)"
          :alt="entityInfo.name"
          class="absolute inset-0 w-full h-full object-cover"
          @error="($event.target as HTMLImageElement).style.display = 'none'"
        />
        <div class="relative px-4 py-3 space-y-2 min-w-0" style="background: linear-gradient(to right, rgb(17 24 39 / 0.95) 50%, transparent)">
          <div class="flex items-center gap-2 flex-wrap text-xs">
            <span class="text-white font-semibold text-sm">{{ entityInfo.name_full ?? entityInfo.name }}</span>
            <button @click="toggleFavorite" class="p-0.5 rounded transition-colors" :class="favoritesStore.isFavorite(entityInfo.id, entityInfo.kind) ? 'text-red-400 hover:text-red-300' : 'text-white/20 hover:text-red-400'" :title="favoritesStore.isFavorite(entityInfo.id, entityInfo.kind) ? 'Remove from favorites' : 'Add to favorites'">
              <IconHeart class="w-3.5 h-3.5" :filled="favoritesStore.isFavorite(entityInfo.id, entityInfo.kind)" />
            </button>
            <span v-if="entityInfo.company_name" class="px-2 py-0.5 rounded bg-white/10 text-white/70 font-medium">{{ entityInfo.company_name }}</span>
            <span v-if="entityInfo.pad_type" class="px-1.5 py-0.5 rounded bg-blue-500/15 text-blue-400/80">Pad {{ entityInfo.pad_type }}</span>
          </div>
          <div v-if="entityInfo.roles.length > 0" class="flex items-center gap-1.5 flex-wrap text-xs">
            <span v-for="role in entityInfo.roles" :key="role" class="px-1.5 py-0.5 rounded bg-white/8 text-white/50">{{ role }}</span>
          </div>
          <div class="flex items-center gap-4 text-xs text-white/50">
            <span v-if="entityInfo.scu">Cargo <span class="text-white/70">{{ entityInfo.scu }} SCU</span></span>
            <span v-if="entityInfo.crew">Crew <span class="text-white/70">{{ entityInfo.crew }}</span></span>
            <span v-if="entityInfo.mass">Mass <span class="text-white/70">{{ entityInfo.mass.toLocaleString() }} kg</span></span>
            <span v-if="formatDimensions(entityInfo)">{{ formatDimensions(entityInfo) }}</span>
            <span v-if="entityInfo.game_version" class="text-white/30">v{{ entityInfo.game_version }}</span>
          </div>
        </div>
      </div>
    </template>
  </div>

  <!-- Wiki specs enrichment section -->
  <WikiSpecsSection
    :entity-id="entityId"
    :entity-kind="entityKind"
    :entity-name="entityName ?? entityInfo?.name ?? ''"
    :uuid="entityUuid ?? entityInfo?.uuid ?? ''"
  />
</template>
