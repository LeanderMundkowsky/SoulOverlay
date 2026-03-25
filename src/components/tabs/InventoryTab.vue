<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useInventoryStore } from "@/stores/inventory";
import type { InventoryEntry } from "@/stores/inventory";
import { useBackendStore } from "@/stores/backend";
import { useOrgStore } from "@/stores/org";
import type { OrgInventoryEntry } from "@/stores/org";
import { useHomeLocationStore } from "@/stores/homeLocation";
import { commands } from "@/bindings";
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
const homeLocationStore = useHomeLocationStore();

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
    const filter = sidebarCollection.value;
    if (filter.kind === 'personal') {
      result = result.filter((e) => e.collections.some((c) => c.id === filter.id));
    } else {
      result = []; // org collection selected — hide all personal entries
    }
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

type OrgEntryWithOrg = { entry: OrgInventoryEntry; orgId: number; orgName: string };

const myOrgEntries = computed((): OrgEntryWithOrg[] => {
  const username = backendStore.account?.username;
  if (!username) return [];
  const result: OrgEntryWithOrg[] = [];
  for (const org of orgStore.myOrgs) {
    for (const entry of orgStore.getInventory(org.id)) {
      if (entry.created_by.username === username) {
        result.push({ entry, orgId: org.id, orgName: org.name });
      }
    }
  }
  return result;
});

const filteredMyOrgEntries = computed((): OrgEntryWithOrg[] => {
  let result = myOrgEntries.value;

  if (sidebarCollection.value !== null) {
    const filter = sidebarCollection.value;
    if (filter.kind === 'personal') {
      const targetName = inventoryStore.collections.find((c) => c.id === filter.id)?.name;
      if (targetName) {
        result = result.filter(({ entry }) => entry.collections.some((c) => c.name === targetName));
      } else {
        result = [];
      }
    } else {
      result = result.filter(
        ({ entry, orgId }) =>
          orgId === filter.orgId && entry.collections.some((c) => c.id === filter.collectionId),
      );
    }
  }

  if (selectedFilter.value) {
    if (groupMode.value === "location") {
      result = result.filter(({ entry }) => entry.location_id === selectedFilter.value!.id);
    } else {
      const collName = inventoryStore.collections.find((c) => String(c.id) === selectedFilter.value!.id)?.name;
      if (collName) {
        result = result.filter(({ entry }) => entry.collections.some((c) => c.name === collName));
      } else {
        result = [];
      }
    }
  }

  const q = searchQuery.value.toLowerCase();
  if (q) {
    result = result.filter(
      ({ entry }) =>
        entry.entity_name.toLowerCase().includes(q) ||
        entry.location_name.toLowerCase().includes(q) ||
        entry.collections.some((c) => c.name.toLowerCase().includes(q)),
    );
  }

  return result;
});

interface Group {
  key: string;
  label: string;
  totalQuantity: number;
  entries: InventoryEntry[];
  orgEntries: OrgEntryWithOrg[];
}

const groupedEntries = computed((): Group[] => {
  type MapVal = { label: string; entries: InventoryEntry[]; orgEntries: OrgEntryWithOrg[] };
  const map = new Map<string, MapVal>();

  function getOrCreate(key: string, label: string): MapVal {
    if (!map.has(key)) map.set(key, { label, entries: [], orgEntries: [] });
    return map.get(key)!;
  }

  for (const entry of filteredEntries.value) {
    if (groupMode.value === "location") {
      getOrCreate(entry.location_id, entry.location_name).entries.push(entry);
    } else {
      if (entry.collections.length === 0) {
        getOrCreate("__none__", "No Collection").entries.push(entry);
      } else {
        for (const coll of entry.collections) {
          getOrCreate(String(coll.id), coll.name).entries.push(entry);
        }
      }
    }
  }

  for (const item of filteredMyOrgEntries.value) {
    const { entry } = item;
    if (groupMode.value === "location") {
      getOrCreate(entry.location_id, entry.location_name).orgEntries.push(item);
    } else {
      if (entry.collections.length === 0) {
        getOrCreate("__none__", "No Collection").orgEntries.push(item);
      } else {
        for (const coll of entry.collections) {
          const personalColl = inventoryStore.collections.find((c) => c.name === coll.name);
          const key = personalColl ? String(personalColl.id) : `org:${coll.id}`;
          getOrCreate(key, coll.name).orgEntries.push(item);
        }
      }
    }
  }

  const groups: Group[] = [];
  for (const [key, { label, entries, orgEntries }] of map) {
    const resolvedLabel =
      groupMode.value === "location"
        ? label
        : key === "__none__"
          ? "No Collection"
          : inventoryStore.collections.find((c) => String(c.id) === key)?.name ?? label;
    const totalQuantity =
      entries.reduce((sum, e) => sum + e.quantity, 0) +
      orgEntries.reduce((sum, { entry }) => sum + entry.quantity, 0);
    groups.push({ key, label: resolvedLabel, totalQuantity, entries, orgEntries });
  }

  groups.sort((a, b) => a.label.localeCompare(b.label));
  return groups;
});

// ── Total count ────────────────────────────────────────────────────────────

const totalItems = computed(() =>
  inventoryStore.entries.reduce((sum, e) => sum + e.quantity, 0) +
  myOrgEntries.value.reduce((sum, { entry }) => sum + entry.quantity, 0),
);

// ── Modal state ────────────────────────────────────────────────────────────

const showModal = ref(false);
const modalMode = ref<ModalMode>("add");
const modalSourceEntry = ref<InventoryEntry | null>(null);
const modalPrefillLocation = ref<{ id: string; name: string; slug: string } | null>(null);
const modalPrefillCollection = ref<number | null>(null);
const modalPrefillEntity = ref<{ id: string; name: string; kind: string } | null>(null);
const modalLockEntity = ref(false);
const modalPrefillQuantity = ref<number | null>(null);

// ── Sharable orgs (user has manage_inventory permission) ─────────────────

const sharableOrgs = computed(() =>
  orgStore.myOrgs.filter((org) => orgStore.canInOrg(org.id, "manage_inventory")),
);

// ── Share / Unshare state ──────────────────────────────────────────────────

// Personal entry to delete after a successful share-to-org save
const pendingDeletePersonalEntryId = ref<{ id: number; originalQty: number } | null>(null);
// Org entry to delete after a successful unshare-to-personal save
const pendingDeleteOrgRef = ref<{ orgId: number; entryId: number; originalQty: number } | null>(null);

function openAddModal() {
  modalMode.value = "add";
  modalSourceEntry.value = null;
  modalPrefillLocation.value = null;
  modalPrefillCollection.value = null;
  modalPrefillEntity.value = null;
  modalLockEntity.value = false;
  modalPrefillQuantity.value = null;
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

type SidebarCollectionFilter =
  | null
  | { kind: 'personal'; id: number }
  | { kind: 'org'; orgId: number; collectionId: number };

const sidebarCollection = ref<SidebarCollectionFilter>(null);

const collectionEntryCounts = computed(() => {
  const map = new Map<number, number>();
  for (const e of inventoryStore.entries) {
    for (const c of e.collections) {
      map.set(c.id, (map.get(c.id) ?? 0) + 1);
    }
  }
  return map;
});

const orgCollectionEntryCounts = computed(() => {
  const map = new Map<string, number>(); // key: `${orgId}:${collectionId}`
  for (const { entry, orgId } of myOrgEntries.value) {
    for (const c of entry.collections) {
      const key = `${orgId}:${c.id}`;
      map.set(key, (map.get(key) ?? 0) + 1);
    }
  }
  return map;
});

const sidebarOrgsWithCollections = computed(() => {
  const result: { orgId: number; orgName: string; collections: { id: number; name: string }[] }[] = [];
  const orgIds = [...new Set(myOrgEntries.value.map((e) => e.orgId))];
  for (const orgId of orgIds) {
    const org = orgStore.myOrgs.find((o) => o.id === orgId);
    if (!org) continue;
    const allColls = orgStore.getCollections(orgId);
    const usedIds = new Set<number>();
    for (const { entry, orgId: eOrgId } of myOrgEntries.value) {
      if (eOrgId !== orgId) continue;
      for (const c of entry.collections) usedIds.add(c.id);
    }
    const usedColls = allColls.filter((c) => usedIds.has(c.id));
    if (usedColls.length > 0) result.push({ orgId, orgName: org.name, collections: usedColls });
  }
  return result;
});

// ── Org entry modal (for org entries shown in personal view) ──────────────

const showOrgModal = ref(false);
const orgModalOrgId = ref<number>(0);
const orgModalMode = ref<ModalMode>("edit");
const orgModalSourceEntry = ref<OrgInventoryEntry | null>(null);
const orgModalPrefillEntity = ref<{ id: string; name: string; kind: string } | null>(null);
const orgModalPrefillLocation = ref<{ id: string; name: string; slug: string } | null>(null);
const orgModalLockEntity = ref(false);
const orgModalPrefillQuantity = ref<number | null>(null);

function openOrgEditModal(orgId: number, entry: OrgInventoryEntry) {
  pendingDeletePersonalEntryId.value = null;
  pendingDeleteOrgRef.value = null;
  orgModalOrgId.value = orgId;
  orgModalMode.value = "edit";
  orgModalSourceEntry.value = entry;
  orgModalPrefillEntity.value = null;
  orgModalPrefillLocation.value = null;
  orgModalLockEntity.value = false;
  orgModalPrefillQuantity.value = null;
  showOrgModal.value = true;
}

function openOrgRemoveModal(orgId: number, entry: OrgInventoryEntry) {
  pendingDeletePersonalEntryId.value = null;
  pendingDeleteOrgRef.value = null;
  orgModalOrgId.value = orgId;
  orgModalMode.value = "remove";
  orgModalSourceEntry.value = entry;
  orgModalPrefillEntity.value = null;
  orgModalPrefillLocation.value = null;
  orgModalLockEntity.value = false;
  orgModalPrefillQuantity.value = null;
  showOrgModal.value = true;
}

function openOrgTransferModal(orgId: number, entry: OrgInventoryEntry) {
  pendingDeletePersonalEntryId.value = null;
  pendingDeleteOrgRef.value = null;
  orgModalOrgId.value = orgId;
  orgModalMode.value = "transfer";
  orgModalSourceEntry.value = entry;
  orgModalPrefillEntity.value = null;
  orgModalPrefillLocation.value = null;
  orgModalLockEntity.value = false;
  orgModalPrefillQuantity.value = null;
  showOrgModal.value = true;
}

// ── Load org inventories for personal view ────────────────────────────────

watch(
  () => orgStore.myOrgs,
  (orgs) => {
    if (!backendStore.account) return;
    for (const org of orgs) {
      orgStore.loadInventory(org.id);
    }
  },
  { immediate: true },
);

// ── Share personal entry to org ────────────────────────────────────────────

async function shareToOrg(entry: InventoryEntry) {
  // Preload collections for all shareable orgs so the modal can show them
  for (const org of sharableOrgs.value) {
    orgStore.loadInventory(org.id);
  }
  const defaultOrgId = sharableOrgs.value[0]?.id ?? 0;
  pendingDeletePersonalEntryId.value = { id: entry.id, originalQty: entry.quantity };
  pendingDeleteOrgRef.value = null;
  orgModalOrgId.value = defaultOrgId;
  orgModalMode.value = "add";
  orgModalSourceEntry.value = null;
  orgModalPrefillEntity.value = { id: entry.entity_id, name: entry.entity_name, kind: entry.entity_kind };
  orgModalPrefillLocation.value = { id: entry.location_id, name: entry.location_name, slug: entry.location_slug };
  orgModalPrefillQuantity.value = entry.quantity;
  orgModalLockEntity.value = true;
  showOrgModal.value = true;
}

// ── Move org entry back to personal inventory ─────────────────────────────

function unshareToPersonal(orgId: number, entry: OrgInventoryEntry) {
  pendingDeleteOrgRef.value = { orgId, entryId: entry.id, originalQty: entry.quantity };
  pendingDeletePersonalEntryId.value = null;
  modalMode.value = "add";
  modalSourceEntry.value = null;
  modalPrefillEntity.value = { id: entry.entity_id, name: entry.entity_name, kind: entry.entity_kind };
  modalPrefillLocation.value = { id: entry.location_id, name: entry.location_name, slug: entry.location_slug };
  modalPrefillQuantity.value = entry.quantity;
  modalLockEntity.value = true;
  modalPrefillCollection.value = null;
  showModal.value = true;
}

// ── Modal save handlers (personal & org) ─────────────────────────────────

const unshareError = ref<string | null>(null);

async function onPersonalModalSaved(savedQty: number) {
  showModal.value = false;
  if (pendingDeleteOrgRef.value) {
    const { orgId, entryId, originalQty } = pendingDeleteOrgRef.value;
    pendingDeleteOrgRef.value = null;
    const err = savedQty >= originalQty
      ? await orgStore.deleteInventoryEntry(orgId, entryId)
      : await orgStore.removeInventoryQuantity(orgId, entryId, savedQty);
    if (err) unshareError.value = `Item was added to personal inventory but could not be removed from org: ${err}`;
  }
}

async function onOrgModalSaved(savedQty: number) {
  showOrgModal.value = false;
  if (pendingDeletePersonalEntryId.value !== null) {
    const { id, originalQty } = pendingDeletePersonalEntryId.value;
    pendingDeletePersonalEntryId.value = null;
    if (savedQty >= originalQty) {
      await inventoryStore.removeEntry(id);
    } else {
      await inventoryStore.removeQuantity(id, savedQty);
    }
  }
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

// ── Transfer All to Home Location ──────────────────────────────────────────

const showTransferAllConfirm = ref(false);
const transferAllLoading = ref(false);
const transferAllError = ref<string | null>(null);
const transferAllDone = ref(false);

/** Personal entries not already at the home location */
const entriesToTransfer = computed(() => {
  const loc = homeLocationStore.homeLocation;
  if (!loc || !loc.uex_id) return [];
  return inventoryStore.entries.filter(
    (e) => e.location_id !== loc.uex_id,
  );
});

/** My org entries not already at the home location */
const orgEntriesToTransfer = computed(() => {
  const loc = homeLocationStore.homeLocation;
  if (!loc || !loc.uex_id) return [] as OrgEntryWithOrg[];
  return myOrgEntries.value.filter(
    ({ entry }) => entry.location_id !== loc.uex_id,
  );
});

const totalEntriesToTransfer = computed(
  () => entriesToTransfer.value.length + orgEntriesToTransfer.value.length,
);

function openTransferAllConfirm() {
  transferAllError.value = null;
  transferAllDone.value = false;
  showTransferAllConfirm.value = true;
}

function cancelTransferAll() {
  showTransferAllConfirm.value = false;
}

async function confirmTransferAll() {
  const loc = homeLocationStore.homeLocation;
  if (!loc?.uex_id) return;

  transferAllLoading.value = true;
  transferAllError.value = null;

  const entries = entriesToTransfer.value.slice();
  const orgEntries = orgEntriesToTransfer.value.slice();
  let failed = 0;

  for (const entry of entries) {
    const result = await commands.transferInventory(
      entry.id,
      entry.quantity,
      loc.uex_id,
      loc.name,
      loc.type_name,
      [],
    );
    if (result.status === "error") failed++;
  }

  for (const { entry, orgId } of orgEntries) {
    const err = await orgStore.transferInventory(
      orgId,
      entry.id,
      entry.quantity,
      loc.uex_id,
      loc.name,
      loc.type_name,
    );
    if (err) failed++;
  }

  transferAllLoading.value = false;

  if (failed > 0) {
    transferAllError.value = `${failed} item(s) could not be transferred.`;
  } else {
    transferAllDone.value = true;
    showTransferAllConfirm.value = false;
    // Reload inventory to reflect changes
    await inventoryStore.loadInventory();
    setTimeout(() => { transferAllDone.value = false; }, 3000);
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
    <AlertBanner
      v-if="unshareError"
      variant="error"
      :message="unshareError"
      @click="unshareError = null"
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
        <!-- Transfer All to Home Location -->
        <button
          v-if="homeLocationStore.homeLocation && (inventoryStore.entries.length > 0 || myOrgEntries.length > 0)"
          @click="openTransferAllConfirm"
          :disabled="totalEntriesToTransfer === 0 || transferAllLoading"
          class="text-xs px-2.5 py-1 rounded-lg border transition-colors"
          :class="transferAllDone
            ? 'border-green-500/30 bg-[#15261c] text-green-400'
            : 'border-orange-500/30 bg-orange-500/5 text-orange-400 hover:bg-orange-500/10 disabled:opacity-40 disabled:cursor-not-allowed'"
          :title="totalEntriesToTransfer === 0
            ? 'All items already at home location'
            : `Transfer ${totalEntriesToTransfer} item(s) to ${homeLocationStore.homeLocation.name}`"
        >
          {{ transferAllDone ? '✓ Transferred' : `🏠 Transfer All (${totalEntriesToTransfer})` }}
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

    <!-- Transfer All confirmation panel -->
    <div
      v-if="showTransferAllConfirm"
      class="bg-[#2a1400] border border-orange-700 rounded-xl p-4 space-y-3"
    >
      <div class="flex items-start gap-3">
        <span class="text-xl shrink-0">🏠</span>
        <div class="flex-1 min-w-0">
          <p class="text-white/80 text-sm font-medium">
            Transfer {{ totalEntriesToTransfer }} item(s) to
            <span class="text-orange-300">{{ homeLocationStore.homeLocation?.system_name }} → {{ homeLocationStore.homeLocation?.name }}</span>?
          </p>
          <p class="text-white/40 text-xs mt-1">
            This simulates a patch wipe — all personal and shared org items not already at your home location will be moved there.
            Items already at home are unaffected.<br>You can change your home location in the Profile tab.
          </p>
        </div>
      </div>
      <div v-if="transferAllError" class="text-red-400 text-xs bg-red-500/10 border border-red-500/20 rounded-lg px-3 py-2">
        {{ transferAllError }}
      </div>
      <div class="flex items-center gap-2">
        <button
          @click="confirmTransferAll"
          :disabled="transferAllLoading"
          class="text-xs px-3 py-1.5 rounded-lg bg-orange-600 hover:bg-orange-500 disabled:opacity-40 disabled:cursor-not-allowed text-white font-medium transition-colors"
        >
          {{ transferAllLoading ? "Transferring…" : "Confirm Transfer" }}
        </button>
        <button
          @click="cancelTransferAll"
          :disabled="transferAllLoading"
          class="text-xs px-3 py-1.5 rounded-lg text-white/40 hover:text-white/70 transition-colors disabled:opacity-40"
        >
          Cancel
        </button>
      </div>
    </div>

    <!-- Transfer All error (when panel is closed) -->
    <AlertBanner
      v-if="transferAllError && !showTransferAllConfirm"
      variant="error"
      :message="transferAllError"
    />

    <!-- Loading -->
    <div v-if="inventoryStore.loading && inventoryStore.entries.length === 0" class="flex justify-center py-12">
      <LoadingSpinner />
    </div>
    <!-- Empty state -->
    <div
      v-if="!inventoryStore.loading && inventoryStore.entries.length === 0 && myOrgEntries.length === 0 && !inventoryStore.error"
      class="text-center text-white/30 py-12 text-sm"
    >
      <p>No items in your inventory.</p>
      <p class="mt-1">Click <strong>+ Add</strong> or use the 📦 button in search results.</p>
    </div>

    <!-- Sidebar + list row (only when entries exist) -->
    <div v-if="inventoryStore.entries.length > 0 || myOrgEntries.length > 0" class="flex gap-4 items-start">

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
        <!-- Personal collections -->
        <button
          v-for="coll in inventoryStore.collections"
          :key="coll.id"
          @click="sidebarCollection = { kind: 'personal', id: coll.id }"
          class="w-full text-left px-2.5 py-1.5 rounded-lg text-sm transition-colors flex items-center justify-between gap-1"
          :class="sidebarCollection?.kind === 'personal' && sidebarCollection.id === coll.id
            ? 'bg-blue-500/20 text-blue-300'
            : 'text-white/50 hover:bg-white/5 hover:text-white/80'"
        >
          <span class="truncate">{{ coll.name }}</span>
          <span class="text-white/30 text-xs shrink-0">{{ collectionEntryCounts.get(coll.id) ?? 0 }}</span>
        </button>
        <!-- Org collections (one section per org) -->
        <template v-for="orgGroup in sidebarOrgsWithCollections" :key="orgGroup.orgId">
          <div class="border-t border-white/5 mt-1 pt-1.5">
            <div class="text-teal-500/40 text-[10px] font-semibold uppercase tracking-wider px-1 pb-1">
              {{ orgGroup.orgName }}
            </div>
            <button
              v-for="coll in orgGroup.collections"
              :key="coll.id"
              @click="sidebarCollection = { kind: 'org', orgId: orgGroup.orgId, collectionId: coll.id }"
              class="w-full text-left px-2.5 py-1.5 rounded-lg text-sm transition-colors flex items-center justify-between gap-1"
              :class="sidebarCollection?.kind === 'org' && sidebarCollection.orgId === orgGroup.orgId && sidebarCollection.collectionId === coll.id
                ? 'bg-teal-500/20 text-teal-300'
                : 'text-white/50 hover:bg-white/5 hover:text-white/80'"
            >
              <span class="truncate">{{ coll.name }}</span>
              <span class="text-white/30 text-xs shrink-0">{{ orgCollectionEntryCounts.get(`${orgGroup.orgId}:${coll.id}`) ?? 0 }}</span>
            </button>
          </div>
        </template>
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
              <!-- Org entries (my entries shared to this org) — shown first -->
              <div
                v-for="{ entry: orgEntry, orgId, orgName } in group.orgEntries"
                :key="`org-${orgId}-${orgEntry.id}`"
                class="flex items-center gap-3 px-4 py-2 hover:bg-teal-500/5 transition-colors group/orgentry border-l-2 border-teal-500/20"
              >
                <div class="flex-shrink-0 w-6 h-6 rounded-md border bg-teal-500/5 border-teal-500/20 flex items-center justify-center text-teal-400/40">
                  <IconCommodity v-if="orgEntry.entity_kind === 'commodity'" class="w-3 h-3" />
                  <IconPackage v-else class="w-3 h-3" />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-white text-sm truncate">{{ orgEntry.entity_name }}</span>
                    <span class="text-white/60 text-xs font-medium shrink-0">{{ orgEntry.quantity }}×</span>
                    <span class="text-xs px-1.5 py-0.5 rounded bg-teal-500/15 text-teal-400/80 shrink-0">{{ orgName }}</span>
                    <template v-if="groupMode === 'location'">
                      <span
                        v-for="c in orgEntry.collections"
                        :key="c.id"
                        class="text-xs px-1.5 py-0.5 rounded bg-blue-500/15 text-blue-400/70 shrink-0 truncate max-w-[80px]"
                      >{{ c.name }}</span>
                    </template>
                  </div>
                  <div v-if="groupMode === 'collection'" class="text-white/30 text-xs truncate mt-0.5">
                    {{ orgEntry.location_name }}
                  </div>
                </div>
                <div class="flex items-center gap-1 opacity-0 group-hover/orgentry:opacity-100 transition-opacity shrink-0">
                  <template v-if="orgStore.canInOrg(orgId, 'manage_inventory')">
                    <button
                      @click.stop="openOrgEditModal(orgId, orgEntry)"
                      class="text-xs px-2 py-1 rounded-lg text-white/30 hover:text-yellow-400 hover:bg-yellow-400/10 transition-colors"
                      title="Edit"
                    >✎ Edit</button>
                    <button
                      @click.stop="openOrgTransferModal(orgId, orgEntry)"
                      class="text-xs px-2 py-1 rounded-lg text-white/30 hover:text-blue-400 hover:bg-blue-400/10 transition-colors"
                      title="Transfer"
                    >↗ Transfer</button>
                    <button
                      @click.stop="openOrgRemoveModal(orgId, orgEntry)"
                      class="text-xs px-2 py-1 rounded-lg text-white/30 hover:text-red-400 hover:bg-red-400/10 transition-colors"
                      title="Remove"
                    >✕ Remove</button>
                  </template>
                  <button
                    @click.stop="unshareToPersonal(orgId, orgEntry)"
                    class="text-xs px-2 py-1 rounded-lg text-white/30 hover:text-green-400 hover:bg-green-400/10 transition-colors"
                    title="Move to personal inventory"
                  >
                    ⬇ Move to Personal
                  </button>
                </div>
              </div>

              <!-- Personal entries -->
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
                <div
                  class="flex items-center gap-1 opacity-0 group-hover/entry:opacity-100 transition-opacity shrink-0"
                >
                  <button
                    @click.stop="openEditModal(entry)"
                    class="text-xs px-2 py-1 rounded-lg text-white/30 hover:text-yellow-400 hover:bg-yellow-400/10 transition-colors"
                    title="Edit"
                  >
                    ✎ Edit
                  </button>
                  <template v-if="sharableOrgs.length > 0">
                    <button
                      @click.stop="shareToOrg(entry)"
                      class="text-xs px-2 py-1 rounded-lg text-white/30 hover:text-teal-400 hover:bg-teal-400/10 transition-colors"
                      title="Share to org"
                    >
                      ⬆ Share
                    </button>
                  </template>
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
      :prefill-entity="modalPrefillEntity"
      :prefill-location="modalPrefillLocation"
      :prefill-collection="modalPrefillCollection"
      :lock-entity="modalLockEntity"
      :prefill-quantity="modalPrefillQuantity"
      @close="showModal = false; pendingDeleteOrgRef = null"
      @saved="onPersonalModalSaved"
    />
    <!-- Org entry modal (Edit / Transfer / Remove / Share from personal view) -->
    <InventoryModal
      v-if="showOrgModal"
      :mode="orgModalMode"
      :org-id="orgModalMode === 'add' ? undefined : orgModalOrgId"
      :org-choices="orgModalMode === 'add' && sharableOrgs.length >= 1 ? sharableOrgs : undefined"
      :source-entry="orgModalSourceEntry"
      :prefill-entity="orgModalPrefillEntity"
      :prefill-location="orgModalPrefillLocation"
      :lock-entity="orgModalLockEntity"
      :prefill-quantity="orgModalPrefillQuantity"
      @close="showOrgModal = false; pendingDeletePersonalEntryId = null"
      @saved="onOrgModalSaved"
    />
    </template>
    </template>
  </div>
</template>
