<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import SearchBar from "@/components/overlay/SearchBar.vue";
import PricePanel from "@/components/overlay/PricePanel.vue";
import EntityInfoCard from "@/components/overlay/EntityInfoCard.vue";
import ResizeHandle from "@/components/ui/ResizeHandle.vue";
import InventoryModal from "@/components/ui/InventoryModal.vue";
import { useSettingsStore } from "@/stores/settings";

const props = defineProps<{
  active: boolean;
}>();

interface SelectedResult {
  id: string;
  name: string;
  kind: string;
  slug?: string;
  source?: string;
  uuid?: string;
}

const settingsStore = useSettingsStore();
const searchBarRef = ref<InstanceType<typeof SearchBar> | null>(null);
const selectedResult = ref<SelectedResult | null>(null);
const navHistory = ref<SelectedResult[]>([]);
const pinnedLocation = ref<SelectedResult | null>(null);
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
  navHistory.value = [];
  selectedResult.value = result;
}

function onPinLocation(result: SelectedResult) {
  pinnedLocation.value = result;
}

function onUnpinLocation() {
  pinnedLocation.value = null;
}

function selectEntity(entity: { id: string; name: string; kind: string; slug?: string }) {
  if (selectedResult.value) {
    navHistory.value.push({ ...selectedResult.value });
  }
  selectedResult.value = entity;
}

function goBack() {
  const prev = navHistory.value.pop();
  selectedResult.value = prev ?? null;
}

function onMouseBack(e: MouseEvent) {
  if (e.button === 3 && navHistory.value.length > 0) {
    e.preventDefault();
    goBack();
  }
}

onMounted(() => window.addEventListener("mouseup", onMouseBack));
onUnmounted(() => window.removeEventListener("mouseup", onMouseBack));

function focusInput() {
  searchBarRef.value?.focusInput();
}

function handleEsc(): boolean {
  return searchBarRef.value?.handleEsc() ?? false;
}

// ── Inventory modal from search ─────────────────────────────────────────────
const showInventoryModal = ref(false);
const inventoryPrefillEntity = ref<{ id: string; name: string; kind: string } | null>(null);

function onAddToInventory(entity: { id: string; name: string; kind: string }) {
  inventoryPrefillEntity.value = entity;
  showInventoryModal.value = true;
}

defineExpose({ focusInput, handleEsc, selectEntity, pinLocation: onPinLocation });
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden gap-3 p-4">
    <!-- Alerts -->
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
        <SearchBar ref="searchBarRef" :pinned-location="pinnedLocation" @select="onResultSelected" @pin="onPinLocation" @unpin="onUnpinLocation" @add-to-inventory="onAddToInventory" />
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
            :entity-name="selectedResult.name"
            :entity-source="selectedResult.source"
            :entity-uuid="selectedResult.uuid"
          />

          <!-- Wiki-only banner -->
          <div
            v-if="selectedResult.source === 'wiki'"
            class="bg-teal-500/10 border border-teal-500/20 rounded-xl px-4 py-2 text-teal-300 text-xs flex items-center gap-2"
          >
            <span class="font-semibold">Wiki only</span>
            <span class="text-teal-300/60">— This item is not tracked by UEX. No price data available.</span>
          </div>

          <!-- Prices card (hidden for wiki-only entities) -->
          <div v-if="selectedResult.source !== 'wiki'" class="flex-1 min-h-0 bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
            <PricePanel
              :entity-id="selectedResult.id"
              :entity-name="selectedResult.name"
              :entity-kind="selectedResult.kind"
              :entity-slug="selectedResult.slug ?? ''"
              :pinned-location="pinnedLocation"
              :active="props.active"
              @close="selectedResult = null; navHistory = []"
              @select-entity="selectEntity"
              @back="goBack"
              :can-go-back="navHistory.length > 0"
            />
          </div>
        </div>
      </Transition>
    </div>

    <!-- Inventory modal triggered from search -->
    <InventoryModal
      v-if="showInventoryModal"
      mode="add"
      :prefill-entity="inventoryPrefillEntity"
      @close="showInventoryModal = false"
    />
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
