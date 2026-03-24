<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import SearchableDropdown from "@/components/ui/SearchableDropdown.vue";
import type { DropdownOption } from "@/components/ui/SearchableDropdown.vue";
import { useHomeLocationStore } from "@/stores/homeLocation";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const homeLocationStore = useHomeLocationStore();

const selected = ref<DropdownOption | null>(null);
const saving = ref(false);
const error = ref<string | null>(null);

// Pre-select current home location if one is already set
onMounted(() => {
  if (homeLocationStore.homeLocation) {
    const loc = homeLocationStore.homeLocation;
    selected.value = {
      id: String(loc.id),
      label: `${loc.system_name} → ${loc.name}`,
      meta: loc.type_name,
    };
  }
});

const canSave = computed(() => selected.value !== null);

async function save() {
  if (!selected.value) return;
  saving.value = true;
  error.value = null;
  const err = await homeLocationStore.setHomeLocation(Number(selected.value.id));
  saving.value = false;
  if (err) {
    error.value = err;
  } else {
    emit("close");
  }
}

function skip() {
  homeLocationStore.dismissPrompt();
  emit("close");
}
</script>

<template>
  <div
    class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60"
    @click.self="skip"
  >
    <div class="bg-[#1a1d24] border border-white/10 rounded-xl w-[420px] shadow-2xl">
      <!-- Header -->
      <div class="flex items-center justify-between px-5 pt-5 pb-4 border-b border-white/10">
        <div>
          <h2 class="text-white font-semibold text-base">Set Home Location</h2>
          <p class="text-white/40 text-xs mt-0.5">Where your inventory resets after a major patch</p>
        </div>
        <button
          @click="skip"
          class="text-white/40 hover:text-white/80 transition-colors"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="p-5 space-y-4">
        <p class="text-white/50 text-sm leading-relaxed">
          Choose the location where you normally spawn in Star Citizen.
          This is used to bulk-transfer your inventory after patches wipe your items.
        </p>

        <!-- Current saved value (shown when re-prompting due to invalid location) -->
        <div v-if="homeLocationStore.homeLocation" class="flex items-center justify-between py-2 px-3 bg-white/5 rounded-lg">
          <span class="text-white/40 text-xs">Current Home Location</span>
          <span class="text-white/80 text-xs font-medium">
            {{ homeLocationStore.homeLocation.system_name }} → {{ homeLocationStore.homeLocation.name }}
          </span>
        </div>

        <div>
          <label class="block text-white/40 text-xs mb-1.5">
            {{ homeLocationStore.homeLocation ? 'Change to' : 'Select location' }}
          </label>
          <SearchableDropdown
            v-model="selected"
            :options="homeLocationStore.dropdownOptions"
            :loading="homeLocationStore.loading"
            :show-meta="false"
            placeholder="Search for a location..."
          />
        </div>

        <div v-if="error" class="text-red-400 text-xs rounded-lg bg-red-500/10 border border-red-500/20 px-3 py-2">
          {{ error }}
        </div>

        <div class="flex items-center gap-3">
          <button
            @click="save"
            :disabled="!canSave || saving"
            class="flex-1 bg-blue-600 hover:bg-blue-500 disabled:opacity-40 disabled:cursor-not-allowed text-white text-sm font-medium rounded-lg px-4 py-2 transition-colors"
          >
            {{ saving ? "Saving…" : "Save Home Location" }}
          </button>
          <button
            @click="skip"
            class="text-white/40 hover:text-white/70 text-sm transition-colors"
          >
            Skip for now
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
