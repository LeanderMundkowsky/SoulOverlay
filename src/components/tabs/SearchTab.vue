<script setup lang="ts">
import { ref } from "vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import SearchBar from "@/components/overlay/SearchBar.vue";
import CommodityPanel from "@/components/overlay/CommodityPanel.vue";
import EntityInfoCard from "@/components/overlay/EntityInfoCard.vue";

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
  <div class="flex flex-col h-full overflow-hidden gap-3 p-4">
    <!-- Alerts -->
    <AlertBanner
      v-if="!scDetected"
      variant="warning"
      title="Star Citizen not detected"
      message="Make sure Star Citizen is running in Borderless Windowed mode."
    />
    <AlertBanner
      v-if="searchBarRef?.stale"
      variant="info"
      message="Showing cached data. Refreshing in the background..."
    />

    <!-- Main area: search centered → search left + detail right -->
    <div class="flex-1 flex gap-3 overflow-hidden min-h-0">
      <!-- Search column: animates from centered to left when a result is selected -->
      <div
        class="flex-shrink-0 flex flex-col bg-[#1a1d24] border border-white/10 rounded-xl overflow-y-auto"
        :style="{
          width: selectedResult ? '42%' : 'min(100%, 40rem)',
          marginLeft: selectedResult ? '0' : 'auto',
          marginRight: selectedResult ? '0' : 'auto',
          transition: 'width 0.3s ease, margin-left 0.3s ease, margin-right 0.3s ease',
        }"
      >
        <SearchBar ref="searchBarRef" @select="onResultSelected" />
      </div>

      <!-- Detail panel: slides in from the right when a result is selected -->
      <Transition name="detail-panel">
        <div
          v-if="selectedResult"
          class="flex-1 min-w-0 flex flex-col gap-3 overflow-hidden"
        >
          <!-- Entity info card -->
          <EntityInfoCard
            :entity-id="selectedResult.id"
            :entity-kind="selectedResult.kind"
          />

          <!-- Prices card -->
          <div class="flex-1 min-h-0 bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
            <CommodityPanel
              :entity-id="selectedResult.id"
              :entity-name="selectedResult.name"
              :entity-kind="selectedResult.kind"
              @close="selectedResult = null"
            />
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.detail-panel-enter-active,
.detail-panel-leave-active {
  transition: opacity 0.25s ease, transform 0.25s ease;
}
.detail-panel-enter-from,
.detail-panel-leave-to {
  opacity: 0;
  transform: translateX(16px);
}
.detail-panel-enter-to,
.detail-panel-leave-from {
  opacity: 1;
  transform: translateX(0);
}
</style>
