<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { commands } from "@/bindings";
import type { UexResult, InventoryEntry } from "@/bindings";
import { useInventoryStore } from "@/stores/inventory";
import IconClose from "@/components/icons/IconClose.vue";

// ── Props & Emits ──────────────────────────────────────────────────────────

export type ModalMode = "add" | "remove" | "transfer";

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

// ── Collection input ──────────────────────────────────────────────────────

const collectionQuery = ref("");
const collectionDropdownOpen = ref(false);

const filteredCollections = computed(() => {
  const q = collectionQuery.value.toLowerCase();
  if (!q) return inventoryStore.collections;
  return inventoryStore.collections.filter((c) => c.toLowerCase().includes(q));
});

function selectCollection(c: string) {
  collectionQuery.value = c;
  collectionDropdownOpen.value = false;
}

function onCollectionFocus() {
  collectionDropdownOpen.value = inventoryStore.collections.length > 0;
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
  }
});

// ── Validation ─────────────────────────────────────────────────────────────

const canSubmit = computed(() => {
  if (saving.value) return false;
  if (quantity.value < 1) return false;
  switch (props.mode) {
    case "add":
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
          collection: collectionQuery.value.trim(),
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
          targetCollection: collectionQuery.value.trim(),
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
    };
    locationQuery.value = props.prefillLocation.name;
  }

  if (props.mode === "add" && props.prefillCollection) {
    collectionQuery.value = props.prefillCollection;
  }

  if (props.mode === "remove" && props.sourceEntry) {
    quantity.value = props.sourceEntry.quantity;
  }

  if (props.mode === "transfer" && props.sourceEntry) {
    quantity.value = props.sourceEntry.quantity;
    collectionQuery.value = props.sourceEntry.collection;
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
  }
});

// ── Keyboard: Escape to close ──────────────────────────────────────────────

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    emit("close");
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

// Close entity/location dropdowns when clicking anywhere inside the modal but outside the dropdown
watch(entityDropdownOpen, (open) => {
  if (open) {
    locationDropdownOpen.value = false;
    collectionDropdownOpen.value = false;
  }
});
watch(locationDropdownOpen, (open) => {
  if (open) {
    entityDropdownOpen.value = false;
    collectionDropdownOpen.value = false;
  }
});
watch(collectionDropdownOpen, (open) => {
  if (open) {
    entityDropdownOpen.value = false;
    locationDropdownOpen.value = false;
  }
});

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
        class="bg-[#1a1d24] border border-white/10 rounded-xl shadow-2xl w-[420px] max-h-[80vh] flex flex-col overflow-hidden"
        @mousedown.stop
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-5 py-3.5 border-b border-white/10">
          <h2 class="text-white text-sm font-semibold">{{ title }}</h2>
          <button
            @click="emit('close')"
            class="text-white/30 hover:text-white transition-colors"
          >
            <IconClose class="w-4 h-4" />
          </button>
        </div>

        <!-- Body -->
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

          <!-- Entity search (add mode only) -->
          <div v-if="mode === 'add'" class="space-y-1.5 relative">
            <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">Item / Commodity</label>
            <input
              id="inv-entity-input"
              type="text"
              :value="entityQuery"
              @input="searchEntity(($event.target as HTMLInputElement).value)"
              @focus="entityDropdownOpen = entityResults.length > 0 && !selectedEntity"
              autocomplete="off"
              placeholder="Search commodities or items..."
              class="w-full bg-[#111318] border rounded-lg px-3 py-2 text-white text-sm placeholder-white/20 focus:outline-none transition-colors"
              :class="selectedEntity ? 'border-green-500/40' : 'border-white/10 focus:border-white/30'"
            />
            <div v-if="selectedEntity" class="text-green-400/60 text-xs">
              ✓ {{ selectedEntity.kind }}
            </div>
            <!-- Dropdown -->
            <div
              v-if="entityDropdownOpen && entityResults.length > 0"
              class="absolute z-10 left-0 right-0 top-full mt-1 bg-[#1e2130] border border-white/10 rounded-lg shadow-xl max-h-[200px] overflow-y-auto"
            >
              <button
                v-for="r in entityResults"
                :key="r.id + r.kind"
                @mousedown.prevent="selectEntity(r)"
                class="w-full text-left px-3 py-2 text-sm hover:bg-white/8 transition-colors flex items-center gap-2"
              >
                <span class="text-white">{{ r.name }}</span>
                <span class="text-white/30 text-xs ml-auto uppercase">{{ r.kind }}</span>
              </button>
            </div>
            <div v-if="entitySearching" class="text-white/20 text-xs mt-1">Searching...</div>
          </div>

          <!-- Location search (add + transfer) -->
          <div v-if="mode === 'add' || mode === 'transfer'" class="space-y-1.5 relative">
            <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">
              {{ mode === 'transfer' ? 'Transfer To' : 'Location' }}
            </label>
            <input
              id="inv-location-input"
              type="text"
              :value="locationQuery"
              @input="searchLocation(($event.target as HTMLInputElement).value)"
              @focus="locationDropdownOpen = locationResults.length > 0 && !selectedLocation"
              autocomplete="off"
              placeholder="Search stations, cities, ships..."
              class="w-full bg-[#111318] border rounded-lg px-3 py-2 text-white text-sm placeholder-white/20 focus:outline-none transition-colors"
              :class="selectedLocation ? 'border-green-500/40' : 'border-white/10 focus:border-white/30'"
            />
            <div v-if="selectedLocation" class="text-green-400/60 text-xs">
              ✓ {{ locationSlugLabel(selectedLocation.slug) }}
            </div>
            <!-- Dropdown -->
            <div
              v-if="locationDropdownOpen && locationResults.length > 0"
              class="absolute z-10 left-0 right-0 top-full mt-1 bg-[#1e2130] border border-white/10 rounded-lg shadow-xl max-h-[200px] overflow-y-auto"
            >
              <button
                v-for="r in locationResults"
                :key="r.id"
                @mousedown.prevent="selectLocation(r)"
                class="w-full text-left px-3 py-2 text-sm hover:bg-white/8 transition-colors flex items-center gap-2"
              >
                <span class="text-white">{{ r.name }}</span>
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

          <!-- Collection (add + transfer) -->
          <div v-if="mode === 'add' || mode === 'transfer'" class="space-y-1.5 relative">
            <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">
              Collection
              <span class="text-white/20 font-normal normal-case tracking-normal">(optional)</span>
            </label>
            <input
              type="text"
              v-model="collectionQuery"
              @focus="onCollectionFocus"
              autocomplete="off"
              placeholder="e.g. Trading, Refining..."
              class="w-full bg-[#111318] border border-white/10 rounded-lg px-3 py-2 text-white text-sm placeholder-white/20 focus:outline-none focus:border-white/30 transition-colors"
            />
            <!-- Dropdown -->
            <div
              v-if="collectionDropdownOpen && filteredCollections.length > 0"
              class="absolute z-10 left-0 right-0 top-full mt-1 bg-[#1e2130] border border-white/10 rounded-lg shadow-xl max-h-[150px] overflow-y-auto"
            >
              <button
                v-for="c in filteredCollections"
                :key="c"
                @mousedown.prevent="selectCollection(c)"
                class="w-full text-left px-3 py-2 text-sm text-white hover:bg-white/8 transition-colors"
              >
                {{ c }}
              </button>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="px-5 py-3.5 border-t border-white/10 flex items-center gap-3">
          <button
            @click="handleSubmit"
            :disabled="!canSubmit"
            class="flex-1 text-sm font-medium py-2 px-4 rounded-lg transition-colors"
            :class="mode === 'remove'
              ? 'bg-red-600 hover:bg-red-500 disabled:bg-red-600/30 text-white'
              : 'bg-blue-600 hover:bg-blue-500 disabled:bg-blue-600/30 text-white'"
          >
            {{ saving ? 'Saving...' : mode === 'remove' ? 'Remove' : mode === 'transfer' ? 'Transfer' : 'Add' }}
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
