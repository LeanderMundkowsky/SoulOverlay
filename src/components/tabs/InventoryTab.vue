<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useInventoryStore } from "@/stores/inventory";
import type { InventoryEntry } from "@/stores/inventory";
import { useBackendStore } from "@/stores/backend";
import { useOrgStore } from "@/stores/org";
import InventoryModal from "@/components/ui/InventoryModal.vue";
import type { ModalMode } from "@/components/ui/InventoryModal.vue";
import OrgInventoryPanel from "@/components/org/OrgInventoryPanel.vue";
import IconSearch from "@/components/icons/IconSearch.vue";
import IconClose from "@/components/icons/IconClose.vue";
import IconCommodity from "@/components/icons/IconCommodity.vue";
import IconPackage from "@/components/icons/IconPackage.vue";
import SearchableDropdown from "@/components/ui/SearchableDropdown.vue";
import type { DropdownOption } from "@/components/ui/SearchableDropdown.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";

const inventoryStore = useInventoryStore();
const backendStore = useBackendStore();
const orgStore = useOrgStore();

// Scope: null = personal inventory, number = org ID
const inventoryScope = ref<number | null>(null);

// ── Load data ──────────────────────────────────────────────────────────────

onMounted(() => {
  if (!backendStore.account) return;
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

const selectedFilter = ref<DropdownOption | null>(null);

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
  const seen = new Set<number>();
  const result: DropdownOption[] = [];
  for (const e of inventoryStore.entries) {
    for (const c of e.collections) {
      if (!seen.has(c.id)) {
        seen.add(c.id);
        result.push({ id: String(c.id), label: c.name });
      }
    }
  }
  return result.sort((a, b) => a.label.localeCompare(b.label));
});

const filterOptions = computed(() =>
  groupMode.value === "location" ? uniqueLocations.value : uniqueCollections.value,
);

function clearFilter() {
  selectedFilter.value = null;
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
    inventoryStore.pendingLocationFilter = null;
  }
}, { immediate: true });

// ── Filtered + grouped entries ─────────────────────────────────────────────

const filteredEntries = computed(() => {
  let result = inventoryStore.entries;

  // Apply sidebar collection filter
  if (sidebarCollection.value !== null) {
    const targetId = sidebarCollection.value;
    result = result.filter((e) => e.collections.some((c) => c.id === targetId));
  }

  // Apply dropdown filter
  if (selectedFilter.value) {
    if (groupMode.value === "location") {
      result = result.filter((e) => e.location_id === selectedFilter.value!.id);
    } else {
      const collId = Number(selectedFilter.value.id);
      result = result.filter((e) => e.collections.some((c) => c.id === collId));
    }
  }

  // Apply text search
  const q = searchQuery.value.toLowerCase();
  if (q) {
    result = result.filter(
      (e) =>
        e.entity_name.toLowerCase().includes(q) ||
        e.location_name.toLowerCase().includes(q) ||
        e.collections.some((c) => c.name.toLowerCase().includes(q)),
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
    if (groupMode.value === "location") {
      const key = entry.location_id;
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(entry);
    } else {
      // Entry appears in each of its collections (or "__none__" if none)
      if (entry.collections.length === 0) {
        if (!map.has("__none__")) map.set("__none__", []);
        map.get("__none__")!.push(entry);
      } else {
        for (const coll of entry.collections) {
          const key = String(coll.id);
          if (!map.has(key)) map.set(key, []);
          map.get(key)!.push(entry);
        }
      }
    }
  }

  const groups: Group[] = [];
  for (const [key, entries] of map) {
    const label =
      groupMode.value === "location"
        ? entries[0].location_name
        : key === "__none__"
          ? "No Collection"
          : inventoryStore.collections.find((c) => String(c.id) === key)?.name ?? key;
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
const modalPrefillCollection = ref<number | null>(null);

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
    modalPrefillCollection.value = group.key === "__none__" ? null : Number(group.key);
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

// ── Sidebar collection filter ──────────────────────────────────────────────

const sidebarCollection = ref<number | null>(null);

const collectionEntryCounts = computed(() => {
  const map = new Map<number, number>();
  for (const e of inventoryStore.entries) {
    for (const c of e.collections) {
      map.set(c.id, (map.get(c.id) ?? 0) + 1);
    }
  }
  return map;
});

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
    <!-- Not logged in prompt -->
    <div
      v-if="!backendStore.account"
      class="flex flex-col items-center justify-center py-20 gap-4 text-center"
    >
      <div class="text-4xl">📦</div>
      <h3 class="text-white/70 text-base font-semibold">Inventory requires an account</h3>
      <p class="text-white/40 text-sm max-w-xs">
        Log in to sync your inventory across sessions and devices.
      </p>
    </div>

    <template v-else>
    <!-- Scope selector (personal / orgs) -->
    <div v-if="orgStore.myOrgs.length > 0" class="flex items-center gap-1 bg-white/5 border border-white/10 rounded-lg p-0.5 self-start">
      <button
        @click="inventoryScope = null"
        class="text-xs px-3 py-1.5 rounded transition-colors"
        :class="inventoryScope === null ? 'bg-white/10 text-white' : 'text-white/40 hover:text-white/70'"
      >Personal</button>
      <button
        v-for="org in orgStore.myOrgs"
        :key="org.id"
        @click="inventoryScope = org.id"
        class="text-xs px-3 py-1.5 rounded transition-colors"
        :class="inventoryScope === org.id ? 'bg-teal-500/20 text-teal-300' : 'text-white/40 hover:text-white/70'"
      >{{ org.name }}</button>
    </div>

    <!-- Org inventory panel when org scope selected -->
    <OrgInventoryPanel v-if="inventoryScope !== null" :org-id="inventoryScope" />

    <template v-else>
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

    <!-- Sidebar + list row (only when entries exist) -->
    <div v-if="inventoryStore.entries.length > 0" class="flex gap-4 items-start">

      <!-- Collections sidebar -->
      <div class="w-44 flex-shrink-0 bg-[#1a1d24] border border-white/10 rounded-xl p-2 space-y-0.5">
        <div class="text-white/40 text-xs font-semibold uppercase tracking-wider px-1 pb-1.5">Collections</div>
        <!-- All -->
        <button
          @click="sidebarCollection = null"
          class="w-full text-left px-2.5 py-1.5 rounded-lg text-sm transition-colors flex items-center justify-between gap-1"
          :class="sidebarCollection === null
            ? 'bg-white/10 text-white'
            : 'text-white/50 hover:bg-white/5 hover:text-white/80'"
        >
          <span class="truncate">All</span>
          <span class="text-white/30 text-xs shrink-0">{{ inventoryStore.entries.length }}</span>
        </button>
        <!-- Each collection -->
        <button
          v-for="coll in inventoryStore.collections"
          :key="coll.id"
          @click="sidebarCollection = coll.id"
          class="w-full text-left px-2.5 py-1.5 rounded-lg text-sm transition-colors flex items-center justify-between gap-1"
          :class="sidebarCollection === coll.id
            ? 'bg-blue-500/20 text-blue-300'
            : 'text-white/50 hover:bg-white/5 hover:text-white/80'"
        >
          <span class="truncate">{{ coll.name }}</span>
          <span class="text-white/30 text-xs shrink-0">{{ collectionEntryCounts.get(coll.id) ?? 0 }}</span>
        </button>
      </div>

      <!-- Grouped list -->
      <div class="flex-1 min-w-0 space-y-3">
        <!-- Search + Dropdown filter row -->
        <div class="flex items-center gap-2">
          <!-- Text search -->
          <div class="flex-1 relative">
            <IconSearch class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-white/20" />
            <input
              v-model="searchQuery"
              type="text"
              placeholder="Filter inventory..."
              class="w-full bg-[#111318] border border-white/10 rounded-lg pl-9 py-2 text-white text-sm placeholder-white/20 focus:outline-none focus:border-white/30 transition-colors"
              :class="searchQuery ? 'pr-8' : 'pr-3'"
            />
            <button
              v-if="searchQuery"
              @click="searchQuery = ''"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-white/30 hover:text-white transition-colors"
            >
              <IconClose class="w-3.5 h-3.5" />
            </button>
          </div>

          <!-- Dropdown filter (location or collection depending on mode) -->
          <div class="w-48 flex-shrink-0">
            <SearchableDropdown
              v-model="selectedFilter"
              :options="filterOptions"
              :placeholder="groupMode === 'location' ? 'Filter by location...' : 'Filter by collection...'"
            />
          </div>
        </div>

        <!-- No results for filter -->
        <div
          v-if="groupedEntries.length === 0"
          class="text-center text-white/30 py-8 text-sm"
        >
          No matching entries found.
        </div>

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
                class="flex items-center gap-3 px-4 py-2 hover:bg-white/5 transition-colors group/entry"
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
                  <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-white text-sm truncate">{{ entry.entity_name }}</span>
                    <span class="text-white/60 text-xs font-medium shrink-0">{{ entry.quantity }}×</span>
                    <template v-if="groupMode === 'location'">
                      <span
                        v-for="c in entry.collections"
                        :key="c.id"
                        class="text-xs px-1.5 py-0.5 rounded bg-blue-500/15 text-blue-400/70 shrink-0 truncate max-w-[80px]"
                      >{{ c.name }}</span>
                    </template>
                  </div>
                  <div v-if="groupMode === 'collection'" class="text-white/30 text-xs truncate mt-0.5">
                    {{ entry.location_name }}
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
      @saved="showModal = false"
    />
    </template>
    </template>
  </div>
</template>
