<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useInventoryStore } from "@/stores/inventory";
import type { InventoryEntry } from "@/stores/inventory";
import InventoryModal from "@/components/ui/InventoryModal.vue";
import type { ModalMode } from "@/components/ui/InventoryModal.vue";
import IconSearch from "@/components/icons/IconSearch.vue";
import IconClose from "@/components/icons/IconClose.vue";
import IconCommodity from "@/components/icons/IconCommodity.vue";
import IconPackage from "@/components/icons/IconPackage.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";

const inventoryStore = useInventoryStore();

// ── Load data ──────────────────────────────────────────────────────────────

onMounted(() => {
  if (inventoryStore.entries.length === 0) {
    inventoryStore.loadInventory();
  }
  inventoryStore.loadCollections();
});

// ── Search filter ──────────────────────────────────────────────────────────

const searchQuery = ref("");

// ── Grouping ───────────────────────────────────────────────────────────────

type GroupMode = "location" | "collection";
const groupMode = ref<GroupMode>("location");

function toggleGroupMode() {
  groupMode.value = groupMode.value === "location" ? "collection" : "location";
}

// ── Collapsed groups ───────────────────────────────────────────────────────

const collapsedGroups = ref<Set<string>>(new Set());

function toggleGroup(key: string) {
  if (collapsedGroups.value.has(key)) {
    collapsedGroups.value.delete(key);
  } else {
    collapsedGroups.value.add(key);
  }
}

// ── Dropdown filter (location in location mode, collection in collection mode) ──

const filterQuery = ref("");
const filterDropdownOpen = ref(false);
const selectedFilter = ref<{ id: string; label: string } | null>(null);

/** Unique locations from current inventory entries */
const uniqueLocations = computed(() => {
  const map = new Map<string, string>();
  for (const e of inventoryStore.entries) {
    if (!map.has(e.location_id)) map.set(e.location_id, e.location_name);
  }
  return Array.from(map, ([id, name]) => ({ id, label: name }))
    .sort((a, b) => a.label.localeCompare(b.label));
});

/** Unique collections from current inventory entries */
const uniqueCollections = computed(() => {
  const set = new Set<string>();
  for (const e of inventoryStore.entries) set.add(e.collection);
  return Array.from(set)
    .sort((a, b) => {
      if (a === "") return 1;
      if (b === "") return -1;
      return a.localeCompare(b);
    })
    .map((c) => ({ id: c, label: c === "" ? "No Collection" : c }));
});

const filterOptions = computed(() =>
  groupMode.value === "location" ? uniqueLocations.value : uniqueCollections.value,
);

const filteredFilterOptions = computed(() => {
  const q = filterQuery.value.toLowerCase();
  if (!q) return filterOptions.value;
  return filterOptions.value.filter((o) => o.label.toLowerCase().includes(q));
});

function selectFilter(option: { id: string; label: string }) {
  selectedFilter.value = option;
  filterQuery.value = option.label;
  filterDropdownOpen.value = false;
}

function clearFilter() {
  selectedFilter.value = null;
  filterQuery.value = "";
  filterDropdownOpen.value = false;
}

function onFilterInput(value: string) {
  filterQuery.value = value;
  selectedFilter.value = null;
  filterDropdownOpen.value = value.length === 0
    ? filterOptions.value.length > 0
    : filteredFilterOptions.value.length > 0;
}

function onFilterFocus() {
  if (!selectedFilter.value) {
    filterDropdownOpen.value = filterOptions.value.length > 0;
  }
}

function closeFilterDropdownDelayed() {
  globalThis.setTimeout(() => { filterDropdownOpen.value = false; }, 150);
}

// Reset filter when grouping changes
watch(groupMode, () => {
  clearFilter();
  collapsedGroups.value.clear();
});

// ── Consume pending filter from store (cross-tab navigation) ──────────────

watch(() => inventoryStore.pendingLocationFilter, (pending) => {
  if (pending) {
    groupMode.value = "location";
    selectedFilter.value = { id: pending.id, label: pending.name };
    filterQuery.value = pending.name;
    inventoryStore.pendingLocationFilter = null;
  }
}, { immediate: true });

// ── Filtered + grouped entries ─────────────────────────────────────────────

const filteredEntries = computed(() => {
  let result = inventoryStore.entries;

  // Apply dropdown filter
  if (selectedFilter.value) {
    if (groupMode.value === "location") {
      result = result.filter((e) => e.location_id === selectedFilter.value!.id);
    } else {
      const coll = selectedFilter.value.id; // "" for no collection
      result = result.filter((e) => e.collection === coll);
    }
  }

  // Apply text search
  const q = searchQuery.value.toLowerCase();
  if (q) {
    result = result.filter(
      (e) =>
        e.entity_name.toLowerCase().includes(q) ||
        e.location_name.toLowerCase().includes(q) ||
        e.collection.toLowerCase().includes(q),
    );
  }

  return result;
});

interface Group {
  key: string;
  label: string;
  totalQuantity: number;
  entries: InventoryEntry[];
}

const groupedEntries = computed((): Group[] => {
  const map = new Map<string, InventoryEntry[]>();
  for (const entry of filteredEntries.value) {
    const key =
      groupMode.value === "location"
        ? entry.location_id
        : entry.collection || "__none__";
    if (!map.has(key)) map.set(key, []);
    map.get(key)!.push(entry);
  }

  const groups: Group[] = [];
  for (const [key, entries] of map) {
    const label =
      groupMode.value === "location"
        ? entries[0].location_name
        : key === "__none__"
          ? "No Collection"
          : key;
    const totalQuantity = entries.reduce((sum, e) => sum + e.quantity, 0);
    groups.push({ key, label, totalQuantity, entries });
  }

  groups.sort((a, b) => a.label.localeCompare(b.label));
  return groups;
});

// ── Total count ────────────────────────────────────────────────────────────

const totalItems = computed(() =>
  inventoryStore.entries.reduce((sum, e) => sum + e.quantity, 0),
);

// ── Modal state ────────────────────────────────────────────────────────────

const showModal = ref(false);
const modalMode = ref<ModalMode>("add");
const modalSourceEntry = ref<InventoryEntry | null>(null);
const modalPrefillLocation = ref<{ id: string; name: string; slug: string } | null>(null);
const modalPrefillCollection = ref<string | null>(null);

function openAddModal() {
  modalMode.value = "add";
  modalSourceEntry.value = null;
  modalPrefillLocation.value = null;
  modalPrefillCollection.value = null;
  showModal.value = true;
}

function openAddModalForGroup(group: Group) {
  modalMode.value = "add";
  modalSourceEntry.value = null;
  if (groupMode.value === "location") {
    const first = group.entries[0];
    modalPrefillLocation.value = {
      id: first.location_id,
      name: first.location_name,
      slug: first.location_slug,
    };
    modalPrefillCollection.value = null;
  } else {
    modalPrefillLocation.value = null;
    modalPrefillCollection.value = group.key === "__none__" ? null : group.key;
  }
  showModal.value = true;
}

function openEditModal(entry: InventoryEntry) {
  modalMode.value = "edit";
  modalSourceEntry.value = entry;
  modalPrefillLocation.value = null;
  modalPrefillCollection.value = null;
  showModal.value = true;
}

function openRemoveModal(entry: InventoryEntry) {
  modalMode.value = "remove";
  modalSourceEntry.value = entry;
  modalPrefillLocation.value = null;
  modalPrefillCollection.value = null;
  showModal.value = true;
}

function openTransferModal(entry: InventoryEntry) {
  modalMode.value = "transfer";
  modalSourceEntry.value = entry;
  modalPrefillLocation.value = null;
  modalPrefillCollection.value = null;
  showModal.value = true;
}

function onModalSaved() {
  inventoryStore.loadInventory();
  inventoryStore.loadCollections();
}

// ── Subtext for entries ────────────────────────────────────────────────────

function entrySubtext(entry: InventoryEntry): string {
  if (groupMode.value === "location") {
    return entry.collection || "";
  }
  return entry.location_name;
}

function slugIcon(slug: string): string {
  switch (slug) {
    case "space_station": return "🛰️";
    case "city": return "🏙️";
    case "outpost": return "🏕️";
    case "poi": return "📍";
    case "fleet_vehicle": return "🚀";
    default: return "📦";
  }
}
</script>

<template>
  <div class="p-6 max-w-5xl mx-auto w-full space-y-4">
    <!-- Error -->
    <AlertBanner
      v-if="inventoryStore.error"
      variant="error"
      :message="inventoryStore.error"
    />

    <!-- Header -->
    <div class="flex items-center justify-between gap-3">
      <h2 class="text-white/80 text-sm font-semibold uppercase tracking-wider">
        Inventory
        <span v-if="totalItems > 0" class="text-white/40 font-normal ml-2">
          ({{ totalItems }} items)
        </span>
      </h2>
      <div class="flex items-center gap-2">
        <!-- Group toggle -->
        <button
          @click="toggleGroupMode"
          class="text-xs px-2.5 py-1 rounded-lg border transition-colors"
          :class="groupMode === 'location'
            ? 'border-green-500/30 bg-[#15261c] text-green-400'
            : 'border-blue-500/30 bg-[#172035] text-blue-400'"
        >
          {{ groupMode === 'location' ? '📍 By Location' : '🏷️ By Collection' }}
        </button>
        <!-- Add button -->
        <button
          @click="openAddModal"
          class="text-xs px-3 py-1 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-medium transition-colors"
        >
          + Add
        </button>
      </div>
    </div>

    <!-- Search + Dropdown filter row -->
    <div class="flex items-center gap-2">
      <!-- Text search -->
      <div class="flex-1 relative">
        <IconSearch class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-white/20" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Filter inventory..."
          class="w-full bg-[#111318] border border-white/10 rounded-lg pl-9 pr-3 py-2 text-white text-sm placeholder-white/20 focus:outline-none focus:border-white/30 transition-colors"
        />
      </div>

      <!-- Dropdown filter (location or collection depending on mode) -->
      <div class="relative w-56 flex-shrink-0">
        <input
          type="text"
          :value="filterQuery"
          @input="onFilterInput(($event.target as HTMLInputElement).value)"
          @focus="onFilterFocus"
          @blur="closeFilterDropdownDelayed"
          autocomplete="off"
          :placeholder="groupMode === 'location' ? 'Filter by location...' : 'Filter by collection...'"
          class="w-full bg-[#111318] border rounded-lg pl-3 pr-8 py-2 text-white text-sm placeholder-white/20 focus:outline-none transition-colors"
          :class="selectedFilter ? 'border-green-500/40' : 'border-white/10 focus:border-white/30'"
        />
        <!-- Clear button -->
        <button
          v-if="selectedFilter"
          @mousedown.prevent="clearFilter"
          class="absolute right-2 top-1/2 -translate-y-1/2 text-white/30 hover:text-white transition-colors"
        >
          <IconClose class="w-3.5 h-3.5" />
        </button>
        <!-- Dropdown -->
        <div
          v-if="filterDropdownOpen && filteredFilterOptions.length > 0"
          class="absolute z-10 left-0 right-0 top-full mt-1 bg-[#1e2130] border border-white/10 rounded-lg shadow-xl max-h-[200px] overflow-y-auto"
        >
          <button
            v-for="opt in filteredFilterOptions"
            :key="opt.id"
            @mousedown.prevent="selectFilter(opt)"
            class="w-full text-left px-3 py-2 text-sm text-white hover:bg-white/8 transition-colors truncate"
          >
            {{ opt.label }}
          </button>
        </div>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="inventoryStore.loading && inventoryStore.entries.length === 0" class="flex justify-center py-12">
      <LoadingSpinner />
    </div>

    <!-- Empty state -->
    <div
      v-if="!inventoryStore.loading && inventoryStore.entries.length === 0 && !inventoryStore.error"
      class="text-center text-white/30 py-12 text-sm"
    >
      <p>No items in your inventory.</p>
      <p class="mt-1">Click <strong>+ Add</strong> or use the 📦 button in search results.</p>
    </div>

    <!-- No results for filter -->
    <div
      v-if="inventoryStore.entries.length > 0 && groupedEntries.length === 0"
      class="text-center text-white/30 py-8 text-sm"
    >
      No matching entries found.
    </div>

    <!-- Grouped list -->
    <div v-if="groupedEntries.length > 0" class="space-y-3">
      <div
        v-for="group in groupedEntries"
        :key="group.key"
        class="bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden"
      >
        <!-- Group header -->
        <button
          @click="toggleGroup(group.key)"
          class="w-full flex items-center gap-2 px-4 py-2.5 text-left hover:bg-white/5 transition-colors"
        >
          <span class="text-xs text-white/30 transition-transform" :class="collapsedGroups.has(group.key) ? '' : 'rotate-90'">▶</span>
          <span v-if="groupMode === 'location'" class="text-sm">{{ slugIcon(group.entries[0]?.location_slug ?? '') }}</span>
          <span v-else class="text-sm">🏷️</span>
          <span class="text-white text-sm font-medium flex-1">{{ group.label }}</span>
          <span class="text-white/30 text-xs">{{ group.totalQuantity }}× total</span>
          <span
            @click.stop="openAddModalForGroup(group)"
            class="text-white/20 hover:text-green-400 text-xs px-1.5 py-0.5 rounded-md hover:bg-green-400/10 transition-colors ml-1"
            title="Add item here"
          >+</span>
        </button>

        <!-- Group entries -->
        <div v-show="!collapsedGroups.has(group.key)" class="border-t border-white/5">
          <div
            v-for="entry in group.entries"
            :key="entry.id"
            class="flex items-center gap-3 px-4 py-2 hover:bg-white/5 transition-colors group/entry border-b border-white/5 last:border-b-0"
          >
            <!-- Icon -->
            <div
              class="flex-shrink-0 w-6 h-6 rounded-md border bg-white/5 border-white/10 flex items-center justify-center text-white/40"
            >
              <IconCommodity v-if="entry.entity_kind === 'commodity'" class="w-3 h-3" />
              <IconPackage v-else class="w-3 h-3" />
            </div>

            <!-- Info -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-white text-sm truncate">{{ entry.entity_name }}</span>
                <span class="text-white/60 text-xs font-medium shrink-0">{{ entry.quantity }}×</span>
                <span
                  v-if="entry.collection && groupMode === 'location'"
                  class="text-xs px-1.5 py-0.5 rounded bg-blue-500/15 text-blue-400/70 shrink-0 truncate max-w-[100px]"
                >
                  {{ entry.collection }}
                </span>
              </div>
              <div v-if="entrySubtext(entry)" class="text-white/30 text-xs truncate mt-0.5">
                {{ entrySubtext(entry) }}
              </div>
            </div>

            <!-- Action buttons -->
            <div class="flex items-center gap-1 opacity-0 group-hover/entry:opacity-100 transition-opacity shrink-0">
              <button
                @click.stop="openEditModal(entry)"
                class="text-xs px-2 py-1 rounded-lg text-white/30 hover:text-yellow-400 hover:bg-yellow-400/10 transition-colors"
                title="Edit"
              >
                ✎ Edit
              </button>
              <button
                @click.stop="openTransferModal(entry)"
                class="text-xs px-2 py-1 rounded-lg text-white/30 hover:text-blue-400 hover:bg-blue-400/10 transition-colors"
                title="Transfer"
              >
                ↗ Transfer
              </button>
              <button
                @click.stop="openRemoveModal(entry)"
                class="text-xs px-2 py-1 rounded-lg text-white/30 hover:text-red-400 hover:bg-red-400/10 transition-colors"
                title="Remove"
              >
                ✕ Remove
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Modal -->
    <InventoryModal
      v-if="showModal"
      :mode="modalMode"
      :source-entry="modalSourceEntry"
      :prefill-location="modalPrefillLocation"
      :prefill-collection="modalPrefillCollection"
      @close="showModal = false"
      @saved="onModalSaved"
    />
  </div>
</template>
