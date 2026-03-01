<script setup lang="ts">
import { ref } from "vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import SearchBar from "@/components/overlay/SearchBar.vue";
import CommodityPanel from "@/components/overlay/CommodityPanel.vue";

defineProps<{
  scDetected: boolean;
}>();

interface Commodity {
  id: string;
  name: string;
}

const searchBarRef = ref<InstanceType<typeof SearchBar> | null>(null);
const selectedCommodity = ref<Commodity | null>(null);

function onCommoditySelected(commodity: Commodity) {
  selectedCommodity.value = commodity;
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

    <!-- Search card -->
    <div class="bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
      <SearchBar ref="searchBarRef" @select="onCommoditySelected" />
    </div>

    <!-- Commodity prices card -->
    <div v-if="selectedCommodity" class="bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
      <CommodityPanel
        :commodity-id="selectedCommodity.id"
        :commodity-name="selectedCommodity.name"
        @close="selectedCommodity = null"
      />
    </div>
  </div>
</template>
