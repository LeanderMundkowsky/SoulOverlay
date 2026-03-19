<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { commands } from "@/bindings";
import type { UexResult, InventoryEntry } from "@/bindings";
import { useInventoryStore } from "@/stores/inventory";
import IconClose from "@/components/icons/IconClose.vue";

// ── Props & Emits ──────────────────────────────────────────────────────────

export type ModalMode = "add" | "remove" | "transfer" | "edit";

const props = defineProps<{
  mode: ModalMode;
  /** Pre-fill entity (add mode from search row) */
  prefillEntity?: { id: string; name: string; kind: string } | null;
  /** Pre-fill location (add mode from inventory group header) */
  prefillLocation?: { id: string; name: string; slug: string } | null;
  /** Pre-fill collection (add mode from inventory group header) */
  prefillCollection?: string | null;
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

const entityQuery = ref("");
const entityResults = ref<UexResult[]>([]);
const selectedEntity = ref<UexResult | null>(null);
const entityDropdownOpen = ref(false);
const entitySearching = ref(false);
let entityDebounce: ReturnType<typeof setTimeout> | null = null;

function searchEntity(q: string) {
  entityQuery.value = q;
  selectedEntity.value = null;
  if (entityDebounce) clearTimeout(entityDebounce);
  if (q.trim().length < 2) {
    entityResults.value = [];
    entityDropdownOpen.value = false;
    return;
  }
  entityDebounce = setTimeout(async () => {
    entitySearching.value = true;
    try {
      // Search commodities and items only
      const [commodities, items] = await Promise.all([
        commands.apiSearchCommodities(q),
        commands.apiSearchItems(q),
      ]);
      const results: UexResult[] = [];
      if (commodities.status === "ok" && commodities.data.data) {
        results.push(...commodities.data.data);
      }
      if (items.status === "ok" && items.data.data) {
        results.push(...items.data.data);
      }
      entityResults.value = results.slice(0, 20);
      entityDropdownOpen.value = results.length > 0;
    } catch {
      entityResults.value = [];
    } finally {
      entitySearching.value = false;
    }
  }, 250);
}

function selectEntity(r: UexResult) {
  selectedEntity.value = r;
  entityQuery.value = r.name;
  entityDropdownOpen.value = false;
}

// ── Location search ───────────────────────────────────────────────────────

const locationQuery = ref("");
const locationResults = ref<UexResult[]>([]);
const selectedLocation = ref<UexResult | null>(null);
const locationDropdownOpen = ref(false);
const locationSearching = ref(false);
let locationDebounce: ReturnType<typeof setTimeout> | null = null;

function searchLocation(q: string) {
  locationQuery.value = q;
  selectedLocation.value = null;
  if (locationDebounce) clearTimeout(locationDebounce);
  locationDebounce = setTimeout(async () => {
    locationSearching.value = true;
    try {
      const result = await commands.getStorageLocations(q);
      if (result.status === "ok") {
        locationResults.value = result.data;
        locationDropdownOpen.value = result.data.length > 0;
      }
    } catch {
      locationResults.value = [];
    } finally {
      locationSearching.value = false;
    }
  }, 200);
}

function selectLocation(r: UexResult) {
  selectedLocation.value = r;
  locationQuery.value = r.name;
  locationDropdownOpen.value = false;
}

// ── Collections multi-select ──────────────────────────────────────────────

const selectedCollections = ref<string[]>([]);
const newCollectionInput = ref("");
const newlyAdded = ref<string[]>([]);
const allCollections = computed(() => {
  const combined = new Set([...inventoryStore.collections, ...newlyAdded.value]);
  return [...combined].sort();
});

function toggleCollection(c: string) {
  const idx = selectedCollections.value.indexOf(c);
  if (idx >= 0) {
    selectedCollections.value.splice(idx, 1);
  } else {
    selectedCollections.value.push(c);
  }
}

function addNewCollection() {
  const name = newCollectionInput.value.trim();
  if (!name) return;
  if (!allCollections.value.includes(name)) {
    newlyAdded.value.push(name);
  }
  if (!selectedCollections.value.includes(name)) {
    selectedCollections.value.push(name);
  }
  newCollectionInput.value = "";
}

function serializeCollections(): string {
  return [...selectedCollections.value].sort().join(",");
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
          collection: serializeCollections(),
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
          collection: serializeCollections(),
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
          targetCollection: serializeCollections(),
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

function parseCollections(raw: string): string[] {
  return raw.split(",").map((c) => c.trim()).filter(Boolean);
}

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
    entityQuery.value = props.prefillEntity.name;
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
    locationQuery.value = props.prefillLocation.name;
  }

  if (props.mode === "add" && props.prefillCollection) {
    selectedCollections.value = parseCollections(props.prefillCollection);
  }

  if (props.mode === "remove" && props.sourceEntry) {
    quantity.value = props.sourceEntry.quantity;
  }

  if (props.mode === "edit" && props.sourceEntry) {
    const e = props.sourceEntry;
    selectedEntity.value = { id: e.entity_id, name: e.entity_name, kind: e.entity_kind, slug: "", uuid: "", source: "uex" };
    entityQuery.value = e.entity_name;
    selectedLocation.value = { id: e.location_id, name: e.location_name, slug: e.location_slug, kind: "", uuid: "", source: "uex" };
    locationQuery.value = e.location_name;
    quantity.value = e.quantity;
    selectedCollections.value = parseCollections(e.collection);
  }

  if (props.mode === "transfer" && props.sourceEntry) {
    quantity.value = props.sourceEntry.quantity;
    selectedCollections.value = parseCollections(props.sourceEntry.collection);
  }

  // Load initial locations
  const locResult = await commands.getStorageLocations("");
  if (locResult.status === "ok") {
    locationResults.value = locResult.data;
  }

  await nextTick();
  // Focus the first relevant input
  if (props.mode === "add" && !props.prefillEntity) {
    document.getElementById("inv-entity-input")?.focus();
  } else if (props.mode === "add") {
    document.getElementById("inv-location-input")?.focus();
  } else if (props.mode === "remove") {
    document.getElementById("inv-quantity-input")?.focus();
  } else if (props.mode === "transfer") {
    document.getElementById("inv-location-input")?.focus();
  } else if (props.mode === "edit") {
    document.getElementById("inv-entity-input")?.focus();
  }
});

// ── Keyboard: Escape to close ──────────────────────────────────────────────

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    emit("close");
  } else if (e.key === "Enter" && canSubmit.value) {
    // Don't submit when the new-collection input is focused
    if ((e.target as HTMLElement)?.id === "inv-new-collection-input") return;
    // Only submit when no dropdown item is highlighted — those take priority
    const dropdownItemActive =
      (entityDropdownOpen.value && entityHighlight.value >= 0) ||
      (locationDropdownOpen.value && locationHighlight.value >= 0);
    if (!dropdownItemActive) {
      e.preventDefault();
      handleSubmit();
    }
  }
}

onMounted(() => window.addEventListener("keydown", onKeyDown, true));
onUnmounted(() => window.removeEventListener("keydown", onKeyDown, true));

// ── Close dropdowns on outside click ───────────────────────────────────────

function onBackdropClick(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (target.classList.contains("modal-backdrop")) {
    emit("close");
  }
}

watch(entityDropdownOpen, (open) => {
  if (open) locationDropdownOpen.value = false;
});
watch(locationDropdownOpen, (open) => {
  if (open) entityDropdownOpen.value = false;
});

// ── Dropdown keyboard navigation ──────────────────────────────────────────

const entityHighlight = ref(-1);
const locationHighlight = ref(-1);

watch(entityDropdownOpen, (open) => { if (!open) entityHighlight.value = -1; });
watch(locationDropdownOpen, (open) => { if (!open) locationHighlight.value = -1; });

function scrollHighlighted(dropdownId: string, index: number) {
  nextTick(() => {
    document.querySelector(`#${dropdownId} [data-idx="${index}"]`)
      ?.scrollIntoView({ block: "nearest" });
  });
}

function onEntityKeyDown(e: KeyboardEvent) {
  if (!entityDropdownOpen.value) return;
  if (e.key === "ArrowDown") {
    e.preventDefault();
    entityHighlight.value = Math.min(entityHighlight.value + 1, entityResults.value.length - 1);
    scrollHighlighted("inv-entity-dropdown", entityHighlight.value);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    entityHighlight.value = Math.max(entityHighlight.value - 1, 0);
    scrollHighlighted("inv-entity-dropdown", entityHighlight.value);
  } else if (e.key === "Enter" && entityHighlight.value >= 0) {
    e.preventDefault();
    selectEntity(entityResults.value[entityHighlight.value]);
  }
}

function onLocationKeyDown(e: KeyboardEvent) {
  if (!locationDropdownOpen.value) return;
  if (e.key === "ArrowDown") {
    e.preventDefault();
    locationHighlight.value = Math.min(locationHighlight.value + 1, locationResults.value.length - 1);
    scrollHighlighted("inv-location-dropdown", locationHighlight.value);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    locationHighlight.value = Math.max(locationHighlight.value - 1, 0);
    scrollHighlighted("inv-location-dropdown", locationHighlight.value);
  } else if (e.key === "Enter" && locationHighlight.value >= 0) {
    e.preventDefault();
    selectLocation(locationResults.value[locationHighlight.value]);
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
                <span v-if="sourceEntry.collection" class="text-blue-400/60">• {{ sourceEntry.collection }}</span>
              </div>
              <div class="text-white/30 text-xs">Current: {{ sourceEntry.quantity }}×</div>
            </div>

            <!-- Entity search (add + edit) -->
            <div v-if="mode === 'add' || mode === 'edit'" class="space-y-1.5 relative">
              <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">Item / Commodity</label>
              <input
                id="inv-entity-input"
                type="text"
                :value="entityQuery"
                @input="searchEntity(($event.target as HTMLInputElement).value)"
                @focus="entityDropdownOpen = entityResults.length > 0 && !selectedEntity"
                @keydown="onEntityKeyDown"
                autocomplete="off"
                placeholder="Search commodities or items..."
                class="w-full bg-[#111318] border rounded-lg px-3 py-2 text-white text-sm placeholder-white/20 focus:outline-none transition-colors"
                :class="selectedEntity ? 'border-green-500/40' : 'border-white/10 focus:border-white/30'"
              />
              <div v-if="selectedEntity" class="text-green-400/60 text-xs">✓ {{ selectedEntity.kind }}</div>
              <div
                v-if="entityDropdownOpen && entityResults.length > 0"
                id="inv-entity-dropdown"
                class="absolute z-10 left-0 right-0 top-full mt-1 bg-[#1e2130] border border-white/10 rounded-lg shadow-xl max-h-[200px] overflow-y-auto"
              >
                <button
                  v-for="(r, i) in entityResults"
                  :key="r.id + r.kind"
                  :data-idx="i"
                  @mousedown.prevent="selectEntity(r)"
                  class="w-full text-left px-3 py-2 text-sm transition-colors flex items-center gap-2"
                  :class="i === entityHighlight ? 'bg-white/10 text-white' : 'hover:bg-white/8 text-white'"
                >
                  <span>{{ r.name }}</span>
                  <span class="text-white/30 text-xs ml-auto uppercase">{{ r.kind }}</span>
                </button>
              </div>
              <div v-if="entitySearching" class="text-white/20 text-xs mt-1">Searching...</div>
            </div>

            <!-- Location search (add + edit + transfer) -->
            <div v-if="mode === 'add' || mode === 'edit' || mode === 'transfer'" class="space-y-1.5 relative">
              <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">
                {{ mode === 'transfer' ? 'Transfer To' : 'Location' }}
              </label>
              <input
                id="inv-location-input"
                type="text"
                :value="locationQuery"
                @input="searchLocation(($event.target as HTMLInputElement).value)"
                @focus="locationDropdownOpen = locationResults.length > 0 && !selectedLocation"
                @keydown="onLocationKeyDown"
                autocomplete="off"
                placeholder="Search stations, cities, ships..."
                class="w-full bg-[#111318] border rounded-lg px-3 py-2 text-white text-sm placeholder-white/20 focus:outline-none transition-colors"
                :class="selectedLocation ? 'border-green-500/40' : 'border-white/10 focus:border-white/30'"
              />
              <div v-if="selectedLocation" class="text-green-400/60 text-xs">
                ✓ {{ locationSlugLabel(selectedLocation.slug) }}
              </div>
              <div
                v-if="locationDropdownOpen && locationResults.length > 0"
                id="inv-location-dropdown"
                class="absolute z-10 left-0 right-0 top-full mt-1 bg-[#1e2130] border border-white/10 rounded-lg shadow-xl max-h-[200px] overflow-y-auto"
              >
                <button
                  v-for="(r, i) in locationResults"
                  :key="r.id"
                  :data-idx="i"
                  @mousedown.prevent="selectLocation(r)"
                  class="w-full text-left px-3 py-2 text-sm transition-colors flex items-center gap-2"
                  :class="i === locationHighlight ? 'bg-white/10 text-white' : 'hover:bg-white/8 text-white'"
                >
                  <span>{{ r.name }}</span>
                  <span class="text-white/30 text-xs ml-auto">{{ locationSlugLabel(r.slug) }}</span>
                </button>
              </div>
              <div v-if="locationSearching" class="text-white/20 text-xs mt-1">Searching...</div>
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
              <div v-if="allCollections.length === 0" class="text-white/20 text-xs px-2 py-1.5">
                No collections yet
              </div>
              <button
                v-for="c in allCollections"
                :key="c"
                @click="toggleCollection(c)"
                class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-xs transition-colors text-left"
                :class="selectedCollections.includes(c)
                  ? 'bg-blue-500/20 text-blue-300'
                  : 'text-white/50 hover:bg-white/5 hover:text-white/80'"
              >
                <span
                  class="w-3.5 h-3.5 rounded border flex items-center justify-center shrink-0 transition-colors"
                  :class="selectedCollections.includes(c) ? 'border-blue-400 bg-blue-400/20' : 'border-white/20'"
                >
                  <span v-if="selectedCollections.includes(c)" class="text-blue-400 text-[9px] leading-none font-bold">✓</span>
                </span>
                <span class="truncate">{{ c }}</span>
              </button>
            </div>
            <!-- Add new collection -->
            <div class="px-1.5 pb-2 pt-1.5 border-t border-white/10 shrink-0">
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
                  :disabled="!newCollectionInput.trim()"
                  class="px-2 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 disabled:bg-blue-600/20 disabled:cursor-not-allowed text-white text-xs transition-colors shrink-0"
                >+</button>
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
