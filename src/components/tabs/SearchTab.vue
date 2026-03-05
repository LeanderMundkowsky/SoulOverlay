<script setup lang="ts">
import { ref, watch } from "vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import SearchBar from "@/components/overlay/SearchBar.vue";
import PricePanel from "@/components/overlay/PricePanel.vue";
import EntityInfoCard from "@/components/overlay/EntityInfoCard.vue";
import ResizeHandle from "@/components/ui/ResizeHandle.vue";
import { useSettingsStore } from "@/stores/settings";

defineProps<{
  scDetected: boolean;
}>();

interface SelectedResult {
  id: string;
  name: string;
  kind: string;
}

const settingsStore = useSettingsStore();
const searchBarRef = ref<InstanceType<typeof SearchBar> | null>(null);
const selectedResult = ref<SelectedResult | null>(null);
const searchSplitPct = ref(50);  // width when detail panel is open
const searchSoloPct = ref(50);   // width when search is the only panel
const mainAreaRef = ref<HTMLElement | null>(null);
const isDragging = ref(false);

watch(
  () => settingsStore.settings.layout_widths.search_split_pct,
  (val) => { searchSplitPct.value = val; },
  { immediate: true },
);
watch(
  () => settingsStore.settings.layout_widths.search_solo_pct,
  (val) => { searchSoloPct.value = val; },
  { immediate: true },
);

let saveDebounce: ReturnType<typeof setTimeout> | null = null;
function onSearchResize(containerEl: HTMLElement | null, newPx: number) {
  if (!containerEl) return;
  const containerW = containerEl.getBoundingClientRect().width;
  if (containerW === 0) return;
  const pct = Math.round((newPx / containerW) * 100);
  const clamped = Math.min(80, Math.max(20, pct));
  if (selectedResult.value) {
    searchSplitPct.value = clamped;
  } else {
    searchSoloPct.value = clamped;
  }
  if (saveDebounce) clearTimeout(saveDebounce);
  saveDebounce = setTimeout(() => {
    const s = settingsStore.settings;
    settingsStore.saveSettings({
      ...s,
      layout_widths: {
        ...s.layout_widths,
        search_split_pct: searchSplitPct.value,
        search_solo_pct: searchSoloPct.value,
      },
    });
  }, 500);
}
function onSearchReset() {
  if (selectedResult.value) {
    searchSplitPct.value = 50;
  } else {
    searchSoloPct.value = 50;
  }
  const s = settingsStore.settings;
  settingsStore.saveSettings({
    ...s,
    layout_widths: {
      ...s.layout_widths,
      search_split_pct: searchSplitPct.value,
      search_solo_pct: searchSoloPct.value,
    },
  });
}

function onResultSelected(result: SelectedResult) {
  selectedResult.value = result;
}

function selectEntity(entity: { id: string; name: string; kind: string }) {
  selectedResult.value = entity;
}

function focusInput() {
  searchBarRef.value?.focusInput();
}

function handleEsc(): boolean {
  return searchBarRef.value?.handleEsc() ?? false;
}

defineExpose({ focusInput, handleEsc, selectEntity });
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
    <div ref="mainAreaRef" class="flex-1 flex gap-3 overflow-hidden min-h-0">
      <!-- Search column: centered when solo, left when detail is open -->
      <div
        class="relative flex-shrink-0 flex flex-col bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden"
        :style="{
          width: (selectedResult ? searchSplitPct : searchSoloPct) + '%',
          marginLeft: selectedResult ? '0' : 'auto',
          marginRight: selectedResult ? '0' : 'auto',
          transition: isDragging ? 'none' : 'margin-left 0.3s ease, margin-right 0.3s ease',
        }"
      >
        <SearchBar ref="searchBarRef" @select="onResultSelected" />
        <ResizeHandle
          :default-px="0"
          @resize="(px) => onSearchResize(mainAreaRef, px)"
          @reset="onSearchReset"
          @drag-start="isDragging = true"
          @drag-end="isDragging = false"
        />
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
            <PricePanel
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
