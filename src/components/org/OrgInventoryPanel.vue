<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useOrgStore } from "@/stores/org";
import type { OrgInventoryEntry } from "@/stores/org";
import InventoryModal from "@/components/ui/InventoryModal.vue";
import type { ModalMode } from "@/components/ui/InventoryModal.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";

const props = defineProps<{ orgId: number }>();
const orgStore = useOrgStore();

const searchQuery = ref("");
type GroupMode = "location" | "collection";
const groupMode = ref<GroupMode>("location");
const collapsedGroups = ref<Set<string>>(new Set());
const sidebarCollection = ref<number | null>(null);

const canManageInventory = computed(() => orgStore.can("manage_inventory"));
const canManageCollections = computed(() => orgStore.can("manage_collections"));

watch(() => props.orgId, () => { orgStore.loadInventory(props.orgId); }, { immediate: true });

const entries = computed(() => orgStore.getInventory(props.orgId));
const collections = computed(() => orgStore.getCollections(props.orgId));

const filteredEntries = computed(() => {
  let result = entries.value;
  if (sidebarCollection.value !== null) {
    const targetId = sidebarCollection.value;
    result = result.filter((e) => e.collections.some((c) => c.id === targetId));
  }
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
  entries: OrgInventoryEntry[];
}

const groupedEntries = computed((): Group[] => {
  const map = new Map<string, OrgInventoryEntry[]>();
  for (const entry of filteredEntries.value) {
    if (groupMode.value === "location") {
      const key = entry.location_id;
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(entry);
    } else {
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
  for (const [key, groupEntries] of map) {
    const label =
      groupMode.value === "location"
        ? groupEntries[0].location_name
        : key === "__none__"
          ? "No Collection"
          : collections.value.find((c) => String(c.id) === key)?.name ?? key;
    const totalQuantity = groupEntries.reduce((sum, e) => sum + e.quantity, 0);
    groups.push({ key, label, totalQuantity, entries: groupEntries });
  }
  groups.sort((a, b) => a.label.localeCompare(b.label));
  return groups;
});

const totalItems = computed(() => entries.value.reduce((sum, e) => sum + e.quantity, 0));

const collectionEntryCounts = computed(() => {
  const map = new Map<number, number>();
  for (const e of entries.value) {
    for (const c of e.collections) {
      map.set(c.id, (map.get(c.id) ?? 0) + 1);
    }
  }
  return map;
});

function toggleGroup(key: string) {
  if (collapsedGroups.value.has(key)) collapsedGroups.value.delete(key);
  else collapsedGroups.value.add(key);
}

// ── Collection management ──────────────────────────────────────────────────

const editingCollId = ref<number | null>(null);
const editingCollName = ref("");
const newCollName = ref("");
const collError = ref<string | null>(null);

async function createCollection() {
  if (!newCollName.value.trim() || !canManageCollections.value) return;
  collError.value = null;
  const err = await orgStore.createCollection(props.orgId, newCollName.value.trim());
  if (err) collError.value = err;
  else newCollName.value = "";
}

function startEditColl(id: number, name: string) {
  editingCollId.value = id;
  editingCollName.value = name;
}

async function saveEditColl() {
  if (!editingCollId.value || !editingCollName.value.trim()) return;
  collError.value = null;
  const err = await orgStore.updateCollection(props.orgId, editingCollId.value, editingCollName.value.trim());
  if (err) collError.value = err;
  else editingCollId.value = null;
}

async function deleteColl(id: number) {
  collError.value = null;
  const err = await orgStore.deleteCollection(props.orgId, id);
  if (err) collError.value = err;
  else if (sidebarCollection.value === id) sidebarCollection.value = null;
}

// ── Modal ──────────────────────────────────────────────────────────────────

const showModal = ref(false);
const modalMode = ref<ModalMode>("add");
const modalSourceEntry = ref<OrgInventoryEntry | null>(null);
const modalPrefillLocation = ref<{ id: string; name: string; slug: string } | null>(null);
const modalPrefillCollection = ref<number | null>(null);

function openAddModal() {
  modalMode.value = "add";
  modalSourceEntry.value = null;
  modalPrefillLocation.value = null;
  modalPrefillCollection.value = null;
  showModal.value = true;
}

function openAddForGroup(group: Group) {
  modalMode.value = "add";
  modalSourceEntry.value = null;
  if (groupMode.value === "location") {
    const first = group.entries[0];
    modalPrefillLocation.value = { id: first.location_id, name: first.location_name, slug: first.location_slug };
    modalPrefillCollection.value = null;
  } else {
    modalPrefillLocation.value = null;
    modalPrefillCollection.value = group.key === "__none__" ? null : Number(group.key);
  }
  showModal.value = true;
}

function openEditModal(entry: OrgInventoryEntry) {
  modalMode.value = "edit";
  modalSourceEntry.value = entry;
  modalPrefillLocation.value = null;
  modalPrefillCollection.value = null;
  showModal.value = true;
}

function openRemoveModal(entry: OrgInventoryEntry) {
  modalMode.value = "remove";
  modalSourceEntry.value = entry;
  modalPrefillLocation.value = null;
  modalPrefillCollection.value = null;
  showModal.value = true;
}

function openTransferModal(entry: OrgInventoryEntry) {
  modalMode.value = "transfer";
  modalSourceEntry.value = entry;
  modalPrefillLocation.value = null;
  modalPrefillCollection.value = null;
  showModal.value = true;
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
  <div class="space-y-4">
    <AlertBanner v-if="orgStore.inventoryError" variant="error" :message="orgStore.inventoryError" />

    <!-- Header -->
    <div class="flex items-center justify-between gap-3">
      <h3 class="text-white/80 text-sm font-semibold uppercase tracking-wider">
        Org Inventory
        <span v-if="totalItems > 0" class="text-white/40 font-normal ml-2">({{ totalItems }} items)</span>
      </h3>
      <div class="flex items-center gap-2">
        <button
          @click="groupMode = groupMode === 'location' ? 'collection' : 'location'"
          class="text-xs px-2.5 py-1 rounded-lg border transition-colors"
          :class="groupMode === 'location'
            ? 'border-green-500/30 bg-[#15261c] text-green-400'
            : 'border-blue-500/30 bg-[#172035] text-blue-400'"
        >
          {{ groupMode === 'location' ? '📍 By Location' : '🏷️ By Collection' }}
        </button>
        <button
          v-if="canManageInventory"
          @click="openAddModal"
          class="text-xs px-3 py-1 rounded-lg bg-teal-600 hover:bg-teal-500 text-white font-medium transition-colors"
        >+ Add</button>
      </div>
    </div>

    <!-- Search -->
    <input
      v-model="searchQuery"
      type="text"
      placeholder="Search items, locations, collections..."
      class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-white/20 focus:outline-none focus:border-teal-500/50"
    />

    <LoadingSpinner v-if="orgStore.loadingInventory && entries.length === 0" class="py-12" />

    <div v-else-if="entries.length === 0 && !orgStore.loadingInventory && collections.length === 0 && !canManageCollections" class="text-center text-white/30 py-12 text-sm">
      <p>No items in the org inventory.</p>
      <p v-if="canManageInventory" class="mt-1">Click <strong>+ Add</strong> to add items.</p>
    </div>

    <!-- Sidebar + list (always shown when there are collections OR user can manage) -->
    <div v-if="!orgStore.loadingInventory && (entries.length > 0 || collections.length > 0 || canManageCollections)" class="flex gap-4 items-start">
      <!-- Collections sidebar -->
      <div class="w-44 flex-shrink-0 bg-[#1a1d24] border border-white/10 rounded-xl p-2 space-y-0.5">
        <div class="text-white/40 text-xs font-semibold uppercase tracking-wider px-1 pb-1.5">Collections</div>
        <button
          @click="sidebarCollection = null"
          class="w-full text-left px-2.5 py-1.5 rounded-lg text-sm transition-colors flex items-center justify-between gap-1"
          :class="sidebarCollection === null ? 'bg-white/10 text-white' : 'text-white/50 hover:bg-white/5 hover:text-white/80'"
        >
          <span class="truncate">All</span>
          <span class="text-white/30 text-xs shrink-0">{{ entries.length }}</span>
        </button>
        <div v-for="coll in collections" :key="coll.id" class="group flex items-center gap-1">
          <button
            @click="sidebarCollection = coll.id"
            class="flex-1 text-left px-2.5 py-1.5 rounded-lg text-sm transition-colors flex items-center justify-between gap-1 min-w-0"
            :class="sidebarCollection === coll.id ? 'bg-teal-500/20 text-teal-300' : 'text-white/50 hover:bg-white/5 hover:text-white/80'"
          >
            <template v-if="editingCollId === coll.id">
              <input
                v-model="editingCollName"
                @click.stop
                @keydown.enter.stop="saveEditColl"
                @keydown.escape.stop="editingCollId = null"
                class="flex-1 min-w-0 bg-transparent border-b border-teal-500/50 text-xs text-white focus:outline-none"
              />
            </template>
            <template v-else>
              <span class="truncate">{{ coll.name }}</span>
              <span class="text-white/30 text-xs shrink-0">{{ collectionEntryCounts.get(coll.id) ?? 0 }}</span>
            </template>
          </button>
          <div v-if="canManageCollections" class="hidden group-hover:flex items-center gap-0.5">
            <button @click.stop="startEditColl(coll.id, coll.name)" class="text-white/30 hover:text-teal-400 text-xs p-0.5">✏️</button>
            <button @click.stop="deleteColl(coll.id)" class="text-white/30 hover:text-red-400 text-xs p-0.5">✕</button>
          </div>
        </div>
        <!-- New collection -->
        <div v-if="canManageCollections" class="pt-1 border-t border-white/10">
          <p v-if="collError" class="text-red-400 text-[10px] px-1 pb-1">{{ collError }}</p>
          <div class="flex gap-1">
            <input
              v-model="newCollName"
              placeholder="New..."
              @keydown.enter="createCollection"
              class="flex-1 min-w-0 bg-white/5 border border-white/10 rounded px-2 py-1 text-xs text-white placeholder-white/20 focus:outline-none"
            />
            <button @click="createCollection" :disabled="!newCollName.trim()" class="text-xs px-1.5 py-1 rounded bg-teal-600 hover:bg-teal-500 disabled:opacity-30 text-white">+</button>
          </div>
        </div>
      </div>

      <!-- Grouped entries -->
      <div class="flex-1 min-w-0 space-y-3">
        <div v-if="entries.length === 0 && !orgStore.loadingInventory" class="text-center text-white/30 py-12 text-sm">
          <p>No items in the org inventory.</p>
          <p v-if="canManageInventory" class="mt-1">Click <strong>+ Add</strong> to add items.</p>
        </div>
        <div v-for="group in groupedEntries" :key="group.key" class="bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
          <!-- Group header -->
          <button
            @click="toggleGroup(group.key)"
            class="w-full flex items-center justify-between px-4 py-2.5 text-left hover:bg-white/5 transition-colors"
          >
            <div class="flex items-center gap-2 text-sm text-white/80">
              <span v-if="groupMode === 'location'" class="text-base">{{ slugIcon(group.entries[0]?.location_slug ?? '') }}</span>
              <span class="font-medium">{{ group.label }}</span>
            </div>
            <div class="flex items-center gap-3 text-xs text-white/40">
              <span>{{ group.totalQuantity }} items</span>
              <button
                v-if="canManageInventory"
                @click.stop="openAddForGroup(group)"
                class="text-teal-400/60 hover:text-teal-400 text-xs"
              >+ Add</button>
              <svg class="w-3.5 h-3.5 transition-transform" :class="collapsedGroups.has(group.key) ? '' : 'rotate-90'" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
              </svg>
            </div>
          </button>

          <!-- Entries -->
          <div v-if="!collapsedGroups.has(group.key)" class="divide-y divide-white/5">
            <div
              v-for="entry in group.entries"
              :key="entry.id"
              class="flex items-center gap-3 px-4 py-2.5 hover:bg-white/5 transition-colors group"
            >
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-sm text-white truncate">{{ entry.entity_name }}</span>
                  <span class="text-xs text-white/30">{{ entry.entity_kind }}</span>
                </div>
                <div class="flex items-center gap-2 mt-0.5">
                  <span v-for="c in entry.collections" :key="c.id" class="text-xs text-teal-400/60">{{ c.name }}</span>
                  <span v-if="groupMode === 'collection'" class="text-xs text-white/30">📍 {{ entry.location_name }}</span>
                  <span class="text-xs text-white/20">by {{ entry.created_by.username }}</span>
                </div>
              </div>
              <div class="flex items-center gap-3 shrink-0">
                <span class="text-sm font-medium text-white/80">{{ entry.quantity }}×</span>
                <div v-if="canManageInventory" class="hidden group-hover:flex items-center gap-1.5">
                  <button @click="openEditModal(entry)" class="text-xs text-white/30 hover:text-teal-400 transition-colors">Edit</button>
                  <button @click="openRemoveModal(entry)" class="text-xs text-white/30 hover:text-red-400 transition-colors">Remove</button>
                  <button @click="openTransferModal(entry)" class="text-xs text-white/30 hover:text-blue-400 transition-colors">Transfer</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Inventory Modal -->
    <InventoryModal
      v-if="showModal"
      :mode="modalMode"
      :org-id="orgId"
      :source-entry="modalSourceEntry"
      :prefill-location="modalPrefillLocation"
      :prefill-collection="modalPrefillCollection"
      @close="showModal = false"
      @saved="orgStore.loadInventory(orgId)"
    />
  </div>
</template>
