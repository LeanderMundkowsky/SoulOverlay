<script setup lang="ts">
import { ref } from "vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import SearchBar from "@/components/overlay/SearchBar.vue";
import CommodityPanel from "@/components/overlay/CommodityPanel.vue";

defineProps<{
  scDetected: boolean;
}>();

interface SelectedResult {
  id: string;
  name: string;
  kind: string;
}

const searchBarRef = ref<InstanceType<typeof SearchBar> | null>(null);
const selectedResult = ref<SelectedResult | null>(null);

function onResultSelected(result: SelectedResult) {
  selectedResult.value = result;
}

function focusInput() {
  searchBarRef.value?.focusInput();
}

defineExpose({ focusInput });
</script>

<template>
  <div class="p-6 grid grid-cols-1 gap-4 max-w-4xl mx-auto w-full">
    <!-- SC not detected notice -->
    <AlertBanner
      v-if="!scDetected"
      variant="warning"
      title="Star Citizen not detected"
      message="Make sure Star Citizen is running in Borderless Windowed mode."
    />

    <!-- Stale cache data notice -->
    <AlertBanner
      v-if="searchBarRef?.stale"
      variant="info"
      message="Showing cached data. Refreshing in the background..."
    />

    <!-- Search card -->
    <div class="bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
      <SearchBar ref="searchBarRef" @select="onResultSelected" />
    </div>

    <!-- Commodity prices card (only for commodities) -->
    <div v-if="selectedResult && selectedResult.kind === 'commodity'" class="bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
      <CommodityPanel
        :commodity-id="selectedResult.id"
        :commodity-name="selectedResult.name"
        @close="selectedResult = null"
      />
    </div>
  </div>
</template>
