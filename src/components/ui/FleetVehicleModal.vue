<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import type { HangarVehicle, UexResult } from "@/bindings";
import { commands } from "@/bindings";
import IconClose from "@/components/icons/IconClose.vue";
import SearchableDropdown from "@/components/ui/SearchableDropdown.vue";
import type { DropdownOption } from "@/components/ui/SearchableDropdown.vue";
import { useHangarStore } from "@/stores/hangar";

export type FleetModalMode = "add" | "edit";

const props = defineProps<{
  mode: FleetModalMode;
  vehicle?: HangarVehicle | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "saved", vehicle: HangarVehicle): void;
}>();

const hangarStore = useHangarStore();
const vehicleDropdownRef = ref<InstanceType<typeof SearchableDropdown> | null>(null);

const saving = ref(false);
const errorMsg = ref<string | null>(null);

// Vehicle search state
const vehicleResults = ref<UexResult[]>([]);
const selectedVehicle = ref<UexResult | null>(null);
const vehicleSearching = ref(false);
let vehicleDebounce: ReturnType<typeof setTimeout> | null = null;

const vehicleDropdownValue = computed<DropdownOption | null>(() =>
  selectedVehicle.value
    ? { id: selectedVehicle.value.id, label: selectedVehicle.value.name, meta: selectedVehicle.value.kind }
    : null,
);

const vehicleDropdownOptions = computed<DropdownOption[]>(() =>
  vehicleResults.value.map((r) => ({ id: r.id, label: r.name, meta: r.kind })),
);

function onVehicleSelect(opt: DropdownOption | null) {
  if (!opt) { selectedVehicle.value = null; return; }
  selectedVehicle.value = vehicleResults.value.find((r) => r.id === opt.id) ?? null;
}

async function searchVehicles(q: string) {
  if (vehicleDebounce) clearTimeout(vehicleDebounce);
  if (q.trim().length < 2) { vehicleResults.value = []; return; }
  vehicleDebounce = setTimeout(async () => {
    vehicleSearching.value = true;
    try {
      const result = await commands.apiSearchVehicles(q);
      if (result.status === "ok" && result.data.data) {
        vehicleResults.value = result.data.data.filter((r) => r.source === "uex");
      } else {
        vehicleResults.value = [];
      }
    } catch {
      vehicleResults.value = [];
    } finally {
      vehicleSearching.value = false;
    }
  }, 250);
}

// Form fields
const name = ref("");
const serial = ref("");
const description = ref("");
const isPledged = ref(false);
const isHidden = ref(false);

// Populate form when editing
watch(() => props.vehicle, (v) => {
  if (v && props.mode === "edit") {
    name.value = v.name !== v.model_name ? v.name : "";
    description.value = v.description ?? "";
  }
}, { immediate: true });

async function save() {
  errorMsg.value = null;
  saving.value = true;
  try {
    let result: HangarVehicle | null = null;
    if (props.mode === "add") {
      if (!selectedVehicle.value) {
        errorMsg.value = "Please select a ship from the list.";
        return;
      }
      result = await hangarStore.addVehicle(
        selectedVehicle.value.name,
        selectedVehicle.value.id,
        name.value.trim() || null,
        serial.value.trim() || null,
        description.value.trim() || null,
        isPledged.value,
        isHidden.value,
      );
    } else if (props.mode === "edit" && props.vehicle) {
      result = await hangarStore.updateVehicle(
        props.vehicle.id,
        name.value.trim() || null,
        description.value.trim() || null,
      );
    }
    if (result) {
      emit("saved", result);
    } else if (hangarStore.error) {
      errorMsg.value = hangarStore.error;
    }
  } finally {
    saving.value = false;
  }
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    emit("close");
  } else if (e.key === "Enter") {
    const dropdownActive = vehicleDropdownRef.value?.isDropdownActive();
    if (!dropdownActive) {
      e.preventDefault();
      save();
    }
  }
}

onMounted(() => window.addEventListener("keydown", onKeyDown, true));
onUnmounted(() => window.removeEventListener("keydown", onKeyDown, true));
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    @mousedown.self="emit('close')"
  >
    <div class="bg-[#16181f] border border-white/10 rounded-2xl shadow-2xl w-full max-w-md mx-4 p-6 space-y-5">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <h2 class="text-white font-semibold text-base">
          {{ mode === "add" ? "Add Ship" : "Edit Ship" }}
        </h2>
        <button
          @click="emit('close')"
          class="text-white/30 hover:text-white/70 transition-colors"
        >
          <IconClose class="w-5 h-5" />
        </button>
      </div>

      <!-- Error -->
      <div v-if="errorMsg" class="text-red-400 text-xs bg-red-950/50 border border-red-500/20 rounded-lg px-3 py-2">
        {{ errorMsg }}
      </div>

      <!-- Form -->
      <div class="space-y-4">
        <!-- Ship model selector (add only) -->
        <div v-if="mode === 'add'" class="space-y-1.5">
          <label class="text-white/60 text-xs font-medium">
            Ship Model <span class="text-red-400">*</span>
          </label>
          <SearchableDropdown
            ref="vehicleDropdownRef"
            :model-value="vehicleDropdownValue"
            :options="vehicleDropdownOptions"
            :loading="vehicleSearching"
            placeholder="Start typing a ship name…"
            @update:model-value="onVehicleSelect"
            @search="searchVehicles"
          />
          <p class="text-white/30 text-xs">Search from the UEX vehicle catalog (ships &amp; ground vehicles).</p>
        </div>

        <!-- Context header for edit mode -->
        <div v-if="mode === 'edit' && vehicle" class="text-white/40 text-xs">
          Editing <span class="text-white/70">{{ vehicle.model_name }}</span>
        </div>

        <!-- Custom Name / Alias -->
        <div class="space-y-1.5">
          <label class="text-white/60 text-xs font-medium">
            Custom Name
            <span class="text-white/30 font-normal ml-1">optional alias</span>
          </label>
          <input
            v-model="name"
            type="text"
            placeholder="Leave blank to use model name"
            class="w-full bg-[#1e2028] border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-white/20 focus:outline-none focus:border-white/30 transition-colors"
          />
        </div>

        <!-- Serial (add only) -->
        <div v-if="mode === 'add'" class="space-y-1.5">
          <label class="text-white/60 text-xs font-medium">Serial Number <span class="text-white/30 font-normal">optional</span></label>
          <input
            v-model="serial"
            type="text"
            placeholder="e.g. SC-001"
            class="w-full bg-[#1e2028] border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-white/20 focus:outline-none focus:border-white/30 transition-colors"
          />
        </div>

        <!-- Notes -->
        <div class="space-y-1.5">
          <label class="text-white/60 text-xs font-medium">Notes <span class="text-white/30 font-normal">optional</span></label>
          <textarea
            v-model="description"
            rows="3"
            placeholder="Personal notes about this ship…"
            class="w-full bg-[#1e2028] border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-white/20 focus:outline-none focus:border-white/30 transition-colors resize-none"
          />
        </div>

        <!-- Checkboxes (add only) -->
        <div v-if="mode === 'add'" class="flex items-center gap-6">
          <label class="flex items-center gap-2 cursor-pointer select-none">
            <input v-model="isPledged" type="checkbox" class="w-4 h-4 rounded accent-yellow-500" />
            <span class="text-white/60 text-sm">Pledged</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer select-none">
            <input v-model="isHidden" type="checkbox" class="w-4 h-4 rounded accent-white" />
            <span class="text-white/60 text-sm">Hidden</span>
          </label>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex items-center justify-end gap-3 pt-1">
        <button
          @click="emit('close')"
          class="px-4 py-2 text-sm text-white/50 hover:text-white/80 transition-colors"
        >
          Cancel
        </button>
        <button
          @click="save"
          :disabled="saving"
          class="px-4 py-2 text-sm rounded-lg bg-blue-600 hover:bg-blue-500 disabled:opacity-40 disabled:pointer-events-none text-white font-medium transition-colors"
        >
          {{ saving ? "Saving…" : (mode === "add" ? "Add Ship" : "Save") }}
        </button>
      </div>
    </div>
  </div>
</template>
