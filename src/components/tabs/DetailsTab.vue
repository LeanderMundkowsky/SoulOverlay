<script setup lang="ts">
import EntityInfoCard from "@/components/overlay/EntityInfoCard.vue";
import PricePanel from "@/components/overlay/PricePanel.vue";
import IconInfoCircle from "@/components/icons/IconInfoCircle.vue";
import { useDetailsStore } from "@/stores/details";

const props = defineProps<{
  active: boolean;
}>();

const detailsStore = useDetailsStore();
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden p-4 gap-3">
    <!-- Empty state -->
    <template v-if="!detailsStore.currentEntity">
      <div class="flex-1 flex flex-col items-center justify-center text-white/30">
        <IconInfoCircle class="w-10 h-10 mb-3 text-white/15" />
        <p class="text-sm font-medium">No entity selected</p>
        <p class="text-xs mt-1">Select an entity from search or favorites to view details</p>
      </div>
    </template>

    <!-- Entity details -->
    <template v-else>
      <!-- Wiki-only banner -->
      <div
        v-if="detailsStore.currentEntity.source === 'wiki'"
        class="bg-teal-500/10 border border-teal-500/20 rounded-xl px-4 py-2 text-teal-300 text-xs flex items-center gap-2"
      >
        <span class="font-semibold">Wiki only</span>
        <span class="text-teal-300/60">— This item is not tracked by UEX. No price data available.</span>
      </div>

      <!-- Entity info card -->
      <EntityInfoCard
        :entity-id="detailsStore.currentEntity.id"
        :entity-kind="detailsStore.currentEntity.kind"
        :entity-name="detailsStore.currentEntity.name"
        :entity-source="detailsStore.currentEntity.source"
        :entity-uuid="detailsStore.currentEntity.uuid"
      />

      <!-- Price table (hidden for wiki-only) -->
      <div v-if="detailsStore.currentEntity.source !== 'wiki'" class="flex-1 min-h-0 bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
        <PricePanel
          :entity-id="detailsStore.currentEntity.id"
          :entity-name="detailsStore.currentEntity.name"
          :entity-kind="detailsStore.currentEntity.kind"
          :entity-slug="detailsStore.currentEntity.slug ?? ''"
          :active="props.active"
          @close="detailsStore.clear()"
        />
      </div>
    </template>
  </div>
</template>
