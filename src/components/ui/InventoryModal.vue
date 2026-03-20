<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from "vue";
import { commands } from "@/bindings";
import type { UexResult, InventoryEntry } from "@/bindings";
import { useInventoryStore } from "@/stores/inventory";
import IconClose from "@/components/icons/IconClose.vue";
import SearchableDropdown from "@/components/ui/SearchableDropdown.vue";
import type { DropdownOption } from "@/components/ui/SearchableDropdown.vue";

// ── Props & Emits ──────────────────────────────────────────────────────────

export type ModalMode = "add" | "remove" | "transfer" | "edit";

const props = defineProps<{
  mode: ModalMode;
  /** Pre-fill entity (add mode from search row) */
  prefillEntity?: { id: string; name: string; kind: string } | null;
  /** Pre-fill location (add mode from inventory group header) */
  prefillLocation?: { id: string; name: string; slug: string } | null;
  /** Pre-fill collection (add mode from inventory group header — collection ID) */
  prefillCollection?: number | null;
  /** Source entry for remove/transfer modes */
  sourceEntry?: InventoryEntry | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "saved"): void;
}>();

const inventoryStore = useInventoryStore();

// ── Shared state ───────────────────────────────────────────────────────────

const saving = ref(false);
const errorMsg = ref<string | null>(null);
const quantity = ref(1);

// ── Entity search (add mode) ──────────────────────────────────────────────

const entityResults = ref<UexResult[]>([]);
const selectedEntity = ref<UexResult | null>(null);
const entitySearching = ref(false);
let entityDebounce: ReturnType<typeof setTimeout> | null = null;
const entityDropdownRef = ref<InstanceType<typeof SearchableDropdown> | null>(null);

const entityDropdownValue = computed<DropdownOption | null>(() =>
  selectedEntity.value
    ? { id: `${selectedEntity.value.id}:${selectedEntity.value.kind}`, label: selectedEntity.value.name, meta: selectedEntity.value.kind }
    : null,
);

const entityDropdownOptions = computed<DropdownOption[]>(() =>
  entityResults.value.map((r) => ({
    id: `${r.id}:${r.kind}`,
    label: r.name,
    meta: r.kind,
  })),
);

function onEntitySelect(opt: DropdownOption | null) {
  if (!opt) { selectedEntity.value = null; return; }
  const [id, kind] = opt.id.split(":");
  selectedEntity.value = entityResults.value.find((r) => r.id === id && r.kind === kind) ?? null;
}

async function searchEntity(q: string) {
  if (entityDebounce) clearTimeout(entityDebounce);
  if (q.trim().length < 2) { entityResults.value = []; return; }
  entityDebounce = setTimeout(async () => {
    entitySearching.value = true;
    try {
      const [commodities, items] = await Promise.all([
        commands.apiSearchCommodities(q),
        commands.apiSearchItems(q),
      ]);
      const results: UexResult[] = [];
      if (commodities.status === "ok" && commodities.data.data) results.push(...commodities.data.data);
      if (items.status === "ok" && items.data.data) results.push(...items.data.data);
      entityResults.value = results.slice(0, 20);
    } catch {
      entityResults.value = [];
    } finally {
      entitySearching.value = false;
    }
  }, 250);
}

// ── Location search ───────────────────────────────────────────────────────

const locationResults = ref<UexResult[]>([]);
const selectedLocation = ref<UexResult | null>(null);
const locationSearching = ref(false);
let locationDebounce: ReturnType<typeof setTimeout> | null = null;
const locationDropdownRef = ref<InstanceType<typeof SearchableDropdown> | null>(null);

const locationDropdownValue = computed<DropdownOption | null>(() =>
  selectedLocation.value
    ? { id: selectedLocation.value.id, label: selectedLocation.value.name, meta: locationSlugLabel(selectedLocation.value.slug) }
    : null,
);

const locationDropdownOptions = computed<DropdownOption[]>(() =>
  locationResults.value.map((r) => ({
    id: r.id,
    label: r.name,
    meta: locationSlugLabel(r.slug),
  })),
);

function onLocationSelect(opt: DropdownOption | null) {
  if (!opt) { selectedLocation.value = null; return; }
  selectedLocation.value = locationResults.value.find((r) => r.id === opt.id) ?? null;
}

async function searchLocation(q: string) {
  if (locationDebounce) clearTimeout(locationDebounce);
  locationDebounce = setTimeout(async () => {
    locationSearching.value = true;
    try {
      const result = await commands.getStorageLocations(q);
      if (result.status === "ok") locationResults.value = result.data;
    } catch {
      locationResults.value = [];
    } finally {
      locationSearching.value = false;
    }
  }, 200);
}

// ── Collections multi-select ──────────────────────────────────────────────

const selectedCollectionIds = ref<number[]>([]);
const newCollectionInput = ref("");
const addingCollection = ref(false);
const addCollectionError = ref<string | null>(null);

function toggleCollection(id: number) {
  const idx = selectedCollectionIds.value.indexOf(id);
  if (idx >= 0) {
    selectedCollectionIds.value.splice(idx, 1);
  } else {
    selectedCollectionIds.value.push(id);
  }
}

async function addNewCollection() {
  const name = newCollectionInput.value.trim();
  if (!name || addingCollection.value) return;
  addingCollection.value = true;
  addCollectionError.value = null;
  try {
    const coll = await inventoryStore.createCollection(name);
    if (!selectedCollectionIds.value.includes(coll.id)) {
      selectedCollectionIds.value.push(coll.id);
    }
    newCollectionInput.value = "";
  } catch (e) {
    addCollectionError.value = String(e);
  } finally {
    addingCollection.value = false;
  }
}

// ── Form title ─────────────────────────────────────────────────────────────

const title = computed(() => {
  switch (props.mode) {
    case "add":
      return "Add to Inventory";
    case "remove":
      return "Remove from Inventory";
    case "transfer":
      return "Transfer Item";
    case "edit":
      return "Edit Inventory Entry";
  }
});

// ── Validation ─────────────────────────────────────────────────────────────

const canSubmit = computed(() => {
  if (saving.value) return false;
  if (quantity.value < 1) return false;
  switch (props.mode) {
    case "add":
    case "edit":
      return selectedEntity.value !== null && selectedLocation.value !== null;
    case "remove":
      return true;
    case "transfer":
      return selectedLocation.value !== null;
  }
});

// ── Submit ─────────────────────────────────────────────────────────────────

async function handleSubmit() {
  if (!canSubmit.value) return;
  saving.value = true;
  errorMsg.value = null;

  try {
    switch (props.mode) {
      case "add": {
        const entity = selectedEntity.value!;
        const loc = selectedLocation.value!;
        await inventoryStore.addEntry({
          entityId: entity.id,
          entityName: entity.name,
          entityKind: entity.kind,
          locationId: loc.id,
          locationName: loc.name,
          locationSlug: loc.slug,
          quantity: quantity.value,
          collectionIds: [...selectedCollectionIds.value],
        });
        break;
      }
      case "edit": {
        const entry = props.sourceEntry!;
        const entity = selectedEntity.value!;
        const loc = selectedLocation.value!;
        await inventoryStore.updateEntry({
          id: entry.id,
          entityId: entity.id,
          entityName: entity.name,
          entityKind: entity.kind,
          locationId: loc.id,
          locationName: loc.name,
          locationSlug: loc.slug,
          quantity: quantity.value,
          collectionIds: [...selectedCollectionIds.value],
        });
        break;
      }
      case "remove": {
        const entry = props.sourceEntry!;
        if (quantity.value >= entry.quantity) {
          await inventoryStore.removeEntry(entry.id);
        } else {
          await inventoryStore.removeQuantity(entry.id, quantity.value);
        }
        break;
      }
      case "transfer": {
        const entry = props.sourceEntry!;
        const loc = selectedLocation.value!;
        await inventoryStore.transferEntry({
          id: entry.id,
          quantity: quantity.value,
          targetLocationId: loc.id,
          targetLocationName: loc.name,
          targetLocationSlug: loc.slug,
          targetCollectionIds: [...selectedCollectionIds.value],
        });
        break;
      }
    }
    emit("saved");
    emit("close");
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    saving.value = false;
  }
}

// ── Max quantity helper for remove/transfer ─────────────────────────────────

const maxQuantity = computed(() => props.sourceEntry?.quantity ?? 999999);

// ── Prefill on mount ───────────────────────────────────────────────────────

onMounted(async () => {
  await inventoryStore.loadCollections();

  if (props.mode === "add" && props.prefillEntity) {
    selectedEntity.value = {
      id: props.prefillEntity.id,
      name: props.prefillEntity.name,
      kind: props.prefillEntity.kind,
      slug: "",
      uuid: "",
      source: "uex",
    };
  }

  if (props.mode === "add" && props.prefillLocation) {
    selectedLocation.value = {
      id: props.prefillLocation.id,
      name: props.prefillLocation.name,
      slug: props.prefillLocation.slug,
      kind: "",
      uuid: "",
      source: "uex",
    };
  }

  if (props.mode === "add" && props.prefillCollection != null) {
    selectedCollectionIds.value = [props.prefillCollection];
  }

  if (props.mode === "remove" && props.sourceEntry) {
    quantity.value = props.sourceEntry.quantity;
  }

  if (props.mode === "edit" && props.sourceEntry) {
    const e = props.sourceEntry;
    selectedEntity.value = { id: e.entity_id, name: e.entity_name, kind: e.entity_kind, slug: "", uuid: "", source: "uex" };
    selectedLocation.value = { id: e.location_id, name: e.location_name, slug: e.location_slug, kind: "", uuid: "", source: "uex" };
    quantity.value = e.quantity;
    selectedCollectionIds.value = e.collections.map((c) => c.id);
  }

  if (props.mode === "transfer" && props.sourceEntry) {
    quantity.value = props.sourceEntry.quantity;
    selectedCollectionIds.value = props.sourceEntry.collections.map((c) => c.id);
  }

  // Load initial locations list for the location dropdown.
  const locResult = await commands.getStorageLocations("");
  if (locResult.status === "ok") {
    locationResults.value = locResult.data;
  }

  await nextTick();
  if (props.mode === "add" && !props.prefillEntity) {
    entityDropdownRef.value?.focus();
  } else if (props.mode === "add") {
    locationDropdownRef.value?.focus();
  } else if (props.mode === "remove") {
    document.getElementById("inv-quantity-input")?.focus();
  } else if (props.mode === "transfer") {
    locationDropdownRef.value?.focus();
  } else if (props.mode === "edit") {
    entityDropdownRef.value?.focus();
  }
});

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    emit("close");
  } else if (e.key === "Enter" && canSubmit.value) {
    // Don't submit when the new-collection input is focused
    if ((e.target as HTMLElement)?.id === "inv-new-collection-input") return;
    // Don't submit when a dropdown item is highlighted — those take priority
    const dropdownItemActive =
      entityDropdownRef.value?.isDropdownActive() ||
      locationDropdownRef.value?.isDropdownActive();
    if (!dropdownItemActive) {
      e.preventDefault();
      handleSubmit();
    }
  }
}

onMounted(() => window.addEventListener("keydown", onKeyDown, true));
onUnmounted(() => window.removeEventListener("keydown", onKeyDown, true));

function onBackdropClick(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (target.classList.contains("modal-backdrop")) {
    emit("close");
  }
}

function locationSlugLabel(slug: string): string {
  switch (slug) {
    case "space_station": return "Station";
    case "city": return "City";
    case "outpost": return "Outpost";
    case "poi": return "POI";
    case "fleet_vehicle": return "Ship";
    default: return slug;
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      class="modal-backdrop fixed inset-0 z-[100] flex items-center justify-center bg-black/60"
      @mousedown="onBackdropClick"
    >
      <div
        class="bg-[#1a1d24] border border-white/10 rounded-xl shadow-2xl max-h-[80vh] flex flex-col overflow-hidden"
        :class="mode === 'remove' ? 'w-[420px]' : 'w-[580px]'"
        @mousedown.stop
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-5 py-3.5 border-b border-white/10 shrink-0">
          <h2 class="text-white text-sm font-semibold">{{ title }}</h2>
          <button @click="emit('close')" class="text-white/30 hover:text-white transition-colors">
            <IconClose class="w-4 h-4" />
          </button>
        </div>

        <!-- Body: two-column for add/edit/transfer, single-column for remove -->
        <div class="flex flex-1 min-h-0">
          <!-- Left: main form fields -->
          <div class="flex-1 overflow-y-auto px-5 py-4 space-y-4">
            <!-- Error banner -->
            <div
              v-if="errorMsg"
              class="bg-red-500/10 border border-red-500/30 text-red-400 text-xs rounded-lg px-3 py-2"
            >
              {{ errorMsg }}
            </div>

            <!-- Source info (remove/transfer) -->
            <div
              v-if="(mode === 'remove' || mode === 'transfer') && sourceEntry"
              class="bg-white/5 border border-white/10 rounded-lg px-3 py-2 space-y-1"
            >
              <div class="text-white text-sm font-medium">{{ sourceEntry.entity_name }}</div>
              <div class="text-white/40 text-xs flex items-center gap-2">
                <span>{{ sourceEntry.location_name }}</span>
                <template v-if="sourceEntry.collections.length > 0">
                  <span v-for="c in sourceEntry.collections" :key="c.id" class="text-blue-400/60">• {{ c.name }}</span>
                </template>
              </div>
              <div class="text-white/30 text-xs">Current: {{ sourceEntry.quantity }}×</div>
            </div>

            <!-- Entity search (add + edit) -->
            <div v-if="mode === 'add' || mode === 'edit'" class="space-y-1.5">
              <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">Item / Commodity</label>
              <SearchableDropdown
                ref="entityDropdownRef"
                :model-value="entityDropdownValue"
                :options="entityDropdownOptions"
                :loading="entitySearching"
                :clearable="false"
                placeholder="Search commodities or items..."
                @update:model-value="onEntitySelect"
                @search="searchEntity"
              />
            </div>

            <!-- Location search (add + edit + transfer) -->
            <div v-if="mode === 'add' || mode === 'edit' || mode === 'transfer'" class="space-y-1.5">
              <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">
                {{ mode === 'transfer' ? 'Transfer To' : 'Location' }}
              </label>
              <SearchableDropdown
                ref="locationDropdownRef"
                :model-value="locationDropdownValue"
                :options="locationDropdownOptions"
                :loading="locationSearching"
                :clearable="false"
                placeholder="Search stations, cities, ships..."
                @update:model-value="onLocationSelect"
                @search="searchLocation"
              />
            </div>

            <!-- Quantity -->
            <div class="space-y-1.5">
              <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">Quantity</label>
              <input
                id="inv-quantity-input"
                type="number"
                v-model.number="quantity"
                :min="1"
                :max="mode === 'remove' || mode === 'transfer' ? maxQuantity : undefined"
                :placeholder="mode === 'remove' ? `Max: ${maxQuantity}` : '1'"
                class="w-full bg-[#111318] border border-white/10 rounded-lg px-3 py-2 text-white text-sm placeholder-white/20 focus:outline-none focus:border-white/30 transition-colors"
              />
              <div v-if="mode === 'remove'" class="text-white/20 text-xs">
                Leave empty or enter {{ maxQuantity }} to remove all
              </div>
            </div>
          </div>

          <!-- Right: collections panel (add/edit/transfer only) -->
          <div
            v-if="mode !== 'remove'"
            class="w-[168px] flex-shrink-0 border-l border-white/10 flex flex-col"
          >
            <div class="px-3 py-2.5 text-white/40 text-xs font-semibold uppercase tracking-wider border-b border-white/10 shrink-0">
              Collections
            </div>
            <!-- List -->
            <div class="flex-1 overflow-y-auto py-1.5 px-1.5 space-y-0.5">
              <div v-if="inventoryStore.collections.length === 0" class="text-white/20 text-xs px-2 py-1.5">
                No collections yet
              </div>
              <button
                v-for="c in inventoryStore.collections"
                :key="c.id"
                @click="toggleCollection(c.id)"
                class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-xs transition-colors text-left"
                :class="selectedCollectionIds.includes(c.id)
                  ? 'bg-blue-500/20 text-blue-300'
                  : 'text-white/50 hover:bg-white/5 hover:text-white/80'"
              >
                <span
                  class="w-3.5 h-3.5 rounded border flex items-center justify-center shrink-0 transition-colors"
                  :class="selectedCollectionIds.includes(c.id) ? 'border-blue-400 bg-blue-400/20' : 'border-white/20'"
                >
                  <span v-if="selectedCollectionIds.includes(c.id)" class="text-blue-400 text-[9px] leading-none font-bold">✓</span>
                </span>
                <span class="truncate">{{ c.name }}</span>
              </button>
            </div>
            <!-- Add new collection -->
            <div class="px-1.5 pb-2 pt-1.5 border-t border-white/10 shrink-0">
              <div
                v-if="addCollectionError"
                class="text-red-400 text-[10px] px-1 pb-1 truncate"
              >{{ addCollectionError }}</div>
              <div class="flex items-center gap-1">
                <input
                  id="inv-new-collection-input"
                  type="text"
                  v-model="newCollectionInput"
                  @keydown.enter.stop.prevent="addNewCollection"
                  placeholder="New..."
                  class="flex-1 min-w-0 bg-[#111318] border border-white/10 rounded-lg px-2 py-1.5 text-white text-xs placeholder-white/20 focus:outline-none focus:border-white/30 transition-colors"
                />
                <button
                  @click="addNewCollection"
                  :disabled="!newCollectionInput.trim() || addingCollection"
                  class="px-2 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 disabled:bg-blue-600/20 disabled:cursor-not-allowed text-white text-xs transition-colors shrink-0"
                >{{ addingCollection ? '…' : '+' }}</button>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="px-5 py-3.5 border-t border-white/10 flex items-center gap-3 shrink-0">
          <button
            @click="handleSubmit"
            :disabled="!canSubmit"
            class="flex-1 text-sm font-medium py-2 px-4 rounded-lg transition-colors"
            :class="mode === 'remove'
              ? 'bg-red-600 hover:bg-red-500 disabled:bg-red-600/30 text-white'
              : 'bg-blue-600 hover:bg-blue-500 disabled:bg-blue-600/30 text-white'"
          >
            {{ saving ? 'Saving...' : mode === 'remove' ? 'Remove' : mode === 'transfer' ? 'Transfer' : mode === 'edit' ? 'Save' : 'Add' }}
          </button>
          <button
            @click="emit('close')"
            class="text-white/40 hover:text-white text-sm py-2 px-3 rounded-lg hover:bg-white/5 transition-colors"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
