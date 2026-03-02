<script setup lang="ts">
import { ref, onMounted } from "vue";
import HotkeyCapture from "@/components/settings/HotkeyCapture.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import { useSettingsStore, type Settings } from "@/stores/settings";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const settingsStore = useSettingsStore();

const form = ref<Settings>({
  hotkey: "",
  uex_api_key: "",
  log_path: null,
  overlay_opacity: 1.0,
  esc_closes_overlay: true,
  reset_on_open: true,
  max_search_results: 50,
  cache_ttl_prices_secs: 3600,
  cache_ttl_catalog_secs: 86400,
  layout_widths: { left_panel_px: 280, settings_panel_px: 448, search_split_pct: 50, search_solo_pct: 50 },
  font_size: 14,
  keybinds: { toggle_settings: "F12", toggle_debug: "F11" },
});

const saving = ref(false);
const saveError = ref<string | null>(null);
const saveSuccess = ref(false);

onMounted(() => {
  form.value = {
    ...settingsStore.settings,
    hotkey: settingsStore.settings.hotkey,
    keybinds: { ...settingsStore.settings.keybinds },
  };
});

async function handleSave() {
  saving.value = true;
  saveError.value = null;
  saveSuccess.value = false;
  try {
    const updated: Settings = {
      ...form.value,
    };
    await settingsStore.saveSettings(updated);
    saveSuccess.value = true;
    setTimeout(() => {
      saveSuccess.value = false;
      emit("close");
    }, 800);
  } catch (e) {
    saveError.value = String(e);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <!-- Backdrop -->
    <div
      class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center"
      @mousedown.self="emit('close')"
    >
      <!-- Modal box: 70 % of viewport -->
      <div
        class="bg-[#111318] border border-white/10 rounded-xl flex flex-col overflow-hidden shadow-2xl"
        style="width: 70%; height: 70%;"
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-5 py-3 border-b border-white/10 flex-shrink-0">
          <h2 class="text-white font-semibold text-sm tracking-wide uppercase">Keybinds</h2>
          <button
            @click="emit('close')"
            class="text-white/40 hover:text-white transition-colors p-1 rounded"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Body -->
        <div class="flex-1 overflow-y-auto px-6 py-5 space-y-6">
          <p class="text-white/40 text-xs">Click a field, then press the key combination you want to assign.</p>

          <!-- Overlay Toggle -->
          <div class="space-y-1.5">
            <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">Overlay Toggle</label>
            <p class="text-white/30 text-xs">Global hotkey to show / hide the overlay (handled by Rust hook)</p>
            <HotkeyCapture v-model="form.hotkey" />
          </div>

          <!-- Toggle Settings Panel -->
          <div class="space-y-1.5">
            <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">Open Settings (F12)</label>
            <p class="text-white/30 text-xs">Opens or closes the Settings side panel</p>
            <HotkeyCapture v-model="form.keybinds.toggle_settings" />
          </div>

          <!-- Toggle Debug Panel -->
          <div class="space-y-1.5">
            <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">Open Debug (F11)</label>
            <p class="text-white/30 text-xs">Opens or closes the Debug side panel</p>
            <HotkeyCapture v-model="form.keybinds.toggle_debug" />
          </div>

          <AlertBanner v-if="saveError" variant="error" :message="saveError" />
          <AlertBanner v-if="saveSuccess" variant="success" message="Keybinds saved!" />
        </div>

        <!-- Footer -->
        <div class="px-6 py-4 border-t border-white/10 flex items-center gap-3 flex-shrink-0">
          <button
            @click="handleSave"
            :disabled="saving"
            class="flex-1 bg-blue-600 hover:bg-blue-500 disabled:bg-blue-600/50 text-white text-sm font-medium py-2 px-4 rounded-lg transition-colors"
          >
            {{ saving ? "Saving..." : "Save Keybinds" }}
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
