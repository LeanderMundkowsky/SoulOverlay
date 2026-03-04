<script setup lang="ts">
import EntityInfoCard from "@/components/overlay/EntityInfoCard.vue";
import PricePanel from "@/components/overlay/PricePanel.vue";
import IconInfoCircle from "@/components/icons/IconInfoCircle.vue";
import { useDetailsStore } from "@/stores/details";

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
      <!-- Entity info card -->
      <EntityInfoCard
        :entity-id="detailsStore.currentEntity.id"
        :entity-kind="detailsStore.currentEntity.kind"
      />

      <!-- Price table -->
      <div class="flex-1 min-h-0 bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
        <PricePanel
          :entity-id="detailsStore.currentEntity.id"
          :entity-name="detailsStore.currentEntity.name"
          :entity-kind="detailsStore.currentEntity.kind"
          @close="detailsStore.clear()"
        />
      </div>
    </template>
  </div>
</template>
