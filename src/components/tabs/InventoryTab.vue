<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useInventoryStore } from "@/stores/inventory";
import type { InventoryEntry } from "@/stores/inventory";
import InventoryModal from "@/components/ui/InventoryModal.vue";
import type { ModalMode } from "@/components/ui/InventoryModal.vue";
import IconSearch from "@/components/icons/IconSearch.vue";
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

// ── Filtered + grouped entries ─────────────────────────────────────────────

const filteredEntries = computed(() => {
  const q = searchQuery.value.toLowerCase();
  if (!q) return inventoryStore.entries;
  return inventoryStore.entries.filter(
    (e) =>
      e.entity_name.toLowerCase().includes(q) ||
      e.location_name.toLowerCase().includes(q) ||
      e.collection.toLowerCase().includes(q),
  );
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

function openAddModal() {
  modalMode.value = "add";
  modalSourceEntry.value = null;
  showModal.value = true;
}

function openRemoveModal(entry: InventoryEntry) {
  modalMode.value = "remove";
  modalSourceEntry.value = entry;
  showModal.value = true;
}

function openTransferModal(entry: InventoryEntry) {
  modalMode.value = "transfer";
  modalSourceEntry.value = entry;
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

// ── Collection filter ──────────────────────────────────────────────────────

const collectionFilter = ref<string | null>(null);

const displayedGroups = computed(() => {
  if (!collectionFilter.value) return groupedEntries.value;
  return groupedEntries.value.map((g) => ({
    ...g,
    entries: g.entries.filter(
      (e) =>
        (collectionFilter.value === "__none__" && e.collection === "") ||
        e.collection === collectionFilter.value,
    ),
  })).filter((g) => g.entries.length > 0);
});

const allCollections = computed(() => {
  const set = new Set<string>();
  for (const e of inventoryStore.entries) {
    set.add(e.collection || "__none__");
  }
  return Array.from(set).sort((a, b) => {
    if (a === "__none__") return 1;
    if (b === "__none__") return -1;
    return a.localeCompare(b);
  });
});

// Reset filter if grouping changes
watch(groupMode, () => {
  collectionFilter.value = null;
});
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
            ? 'border-green-500/30 bg-green-500/10 text-green-400'
            : 'border-blue-500/30 bg-blue-500/10 text-blue-400'"
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

    <!-- Search + Collection filter -->
    <div class="flex items-center gap-2">
      <div class="flex-1 relative">
        <IconSearch class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-white/20" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Filter inventory..."
          class="w-full bg-[#111318] border border-white/10 rounded-lg pl-9 pr-3 py-2 text-white text-sm placeholder-white/20 focus:outline-none focus:border-white/30 transition-colors"
        />
      </div>
      <!-- Collection filter chips (when grouping by location) -->
      <div v-if="groupMode === 'location' && allCollections.length > 1" class="flex items-center gap-1 flex-shrink-0">
        <button
          @click="collectionFilter = null"
          class="text-xs px-2 py-1 rounded-lg transition-colors"
          :class="collectionFilter === null
            ? 'bg-white/10 text-white'
            : 'text-white/30 hover:text-white hover:bg-white/5'"
        >
          All
        </button>
        <button
          v-for="c in allCollections"
          :key="c"
          @click="collectionFilter = collectionFilter === c ? null : c"
          class="text-xs px-2 py-1 rounded-lg transition-colors truncate max-w-[100px]"
          :class="collectionFilter === c
            ? 'bg-blue-500/20 text-blue-400'
            : 'text-white/30 hover:text-white hover:bg-white/5'"
        >
          {{ c === '__none__' ? 'No Collection' : c }}
        </button>
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
      v-if="inventoryStore.entries.length > 0 && displayedGroups.length === 0"
      class="text-center text-white/30 py-8 text-sm"
    >
      No matching entries found.
    </div>

    <!-- Grouped list -->
    <div v-if="displayedGroups.length > 0" class="space-y-3">
      <div
        v-for="group in displayedGroups"
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
      @close="showModal = false"
      @saved="onModalSaved"
    />
  </div>
</template>
