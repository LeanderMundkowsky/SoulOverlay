<script setup lang="ts">
import { ref, onMounted } from "vue";
import PanelHeader from "@/components/ui/PanelHeader.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import ToggleSwitch from "@/components/ui/ToggleSwitch.vue";
import SettingsField from "@/components/settings/SettingsField.vue";
import HotkeyCapture from "@/components/settings/HotkeyCapture.vue";
import OpacitySlider from "@/components/settings/OpacitySlider.vue";
import CacheSettingsPanel from "@/components/settings/CacheSettingsPanel.vue";
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
});

const saving = ref(false);
const saveError = ref<string | null>(null);
const saveSuccess = ref(false);

onMounted(() => {
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
    overlay_opacity: 0.85,
    esc_closes_overlay: true,
    reset_on_open: true,
  };
}
</script>

<template>
  <div class="bg-[#111318] border-l border-white/10 flex flex-col overflow-y-auto">
    <PanelHeader title="Settings" @close="emit('close')" />

    <div class="flex-1 px-5 py-4 space-y-5">
      <!-- Hotkey -->
      <SettingsField
        label="Toggle Hotkey"
        hint="Click the field, then press your desired key combination"
      >
        <HotkeyCapture v-model="form.hotkey" />
      </SettingsField>

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
      <SettingsField
        label="Game Log Path (optional)"
        hint="Default: %APPDATA%\..\Local\Star Citizen\StarCitizen\LIVE\game.log"
      >
        <input
          v-model="form.log_path"
          type="text"
          placeholder="Leave empty for default path"
          class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500/50 transition-colors"
        />
      </SettingsField>

      <!-- Opacity Slider -->
      <OpacitySlider v-model="form.overlay_opacity" />

      <!-- ESC closes overlay -->
      <ToggleSwitch
        v-model="form.esc_closes_overlay"
        label="ESC closes overlay"
        description="When disabled, ESC only clears the search bar"
      />

      <!-- Reset on open -->
      <ToggleSwitch
        v-model="form.reset_on_open"
        label="Reset on open"
        description="Switch to Search tab and focus input when overlay opens"
      />

      <!-- Divider -->
      <div class="border-t border-white/10"></div>

      <!-- Cache Management -->
      <CacheSettingsPanel />

      <!-- Feedback -->
      <AlertBanner v-if="saveError" variant="error" :message="saveError" />
      <AlertBanner v-if="saveSuccess" variant="success" message="Settings saved successfully!" />
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
