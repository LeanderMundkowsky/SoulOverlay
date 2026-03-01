<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useSettingsStore, type Settings } from "@/stores/settings";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const settingsStore = useSettingsStore();

// Local form state
const form = ref<Settings>({
  hotkey: "",
  uex_api_key: "",
  log_path: null,
  overlay_opacity: 1.0,
});

const saving = ref(false);
const saveError = ref<string | null>(null);
const saveSuccess = ref(false);

onMounted(() => {
  // Copy current settings to form
  form.value = { ...settingsStore.settings };
});

async function handleSave() {
  saving.value = true;
  saveError.value = null;
  saveSuccess.value = false;

  try {
    await settingsStore.saveSettings(form.value);
    saveSuccess.value = true;
    setTimeout(() => {
      saveSuccess.value = false;
    }, 2000);
  } catch (e) {
    saveError.value = String(e);
  } finally {
    saving.value = false;
  }
}

function resetDefaults() {
  form.value = {
    hotkey: "Alt+Shift+S",
    uex_api_key: "",
    log_path: null,
    overlay_opacity: 1.0,
  };
}
</script>

<template>
  <div class="bg-gray-900/98 border-l border-white/10 flex flex-col overflow-y-auto">
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-4 border-b border-white/10">
      <h2 class="text-white font-semibold text-base">Settings</h2>
      <button
        @click="emit('close')"
        class="text-white/40 hover:text-white transition-colors"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>

    <!-- Form -->
    <div class="flex-1 px-5 py-4 space-y-5">
      <!-- Hotkey -->
      <div>
        <label class="block text-white/60 text-xs font-medium uppercase tracking-wider mb-1.5">
          Toggle Hotkey
        </label>
        <input
          v-model="form.hotkey"
          type="text"
          placeholder="Alt+Shift+S"
          class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500/50 transition-colors"
        />
        <p class="text-white/30 text-xs mt-1">Format: Ctrl+Alt+Shift+Key (e.g., Alt+Shift+S)</p>
      </div>

      <!-- UEX API Key -->
      <div>
        <label class="block text-white/60 text-xs font-medium uppercase tracking-wider mb-1.5">
          UEX API Key
        </label>
        <input
          v-model="form.uex_api_key"
          type="password"
          placeholder="Enter your UEX Corp API key"
          class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500/50 transition-colors"
        />
        <p class="text-white/30 text-xs mt-1">
          Get your API key from
          <a href="https://uexcorp.space" class="text-blue-400/60 hover:text-blue-400">uexcorp.space</a>
        </p>
      </div>

      <!-- Log Path -->
      <div>
        <label class="block text-white/60 text-xs font-medium uppercase tracking-wider mb-1.5">
          Game Log Path (optional)
        </label>
        <input
          v-model="form.log_path"
          type="text"
          placeholder="Leave empty for default path"
          class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500/50 transition-colors"
        />
        <p class="text-white/30 text-xs mt-1">Default: %APPDATA%\..\Local\Star Citizen\StarCitizen\LIVE\game.log</p>
      </div>

      <!-- Overlay Opacity -->
      <div>
        <label class="block text-white/60 text-xs font-medium uppercase tracking-wider mb-1.5">
          Overlay Opacity: {{ Math.round(form.overlay_opacity * 100) }}%
        </label>
        <input
          v-model.number="form.overlay_opacity"
          type="range"
          min="0.1"
          max="1"
          step="0.05"
          class="w-full accent-blue-500"
        />
      </div>

      <!-- Error -->
      <div v-if="saveError" class="bg-red-500/10 border border-red-500/30 rounded-lg px-3 py-2 text-red-400 text-sm">
        {{ saveError }}
      </div>

      <!-- Success -->
      <div v-if="saveSuccess" class="bg-green-500/10 border border-green-500/30 rounded-lg px-3 py-2 text-green-400 text-sm">
        Settings saved successfully!
      </div>
    </div>

    <!-- Actions -->
    <div class="px-5 py-4 border-t border-white/10 flex items-center gap-3">
      <button
        @click="handleSave"
        :disabled="saving"
        class="flex-1 bg-blue-600 hover:bg-blue-500 disabled:bg-blue-600/50 text-white text-sm font-medium py-2 px-4 rounded-lg transition-colors"
      >
        {{ saving ? "Saving..." : "Save Settings" }}
      </button>
      <button
        @click="resetDefaults"
        class="text-white/40 hover:text-white text-sm py-2 px-3 rounded-lg hover:bg-white/5 transition-colors"
      >
        Reset
      </button>
    </div>
  </div>
</template>
