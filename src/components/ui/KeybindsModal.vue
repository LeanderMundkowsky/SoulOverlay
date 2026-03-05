<script setup lang="ts">
import { ref, onMounted, toRaw } from "vue";
import HotkeyCapture from "@/components/settings/HotkeyCapture.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import PanelHeader from "@/components/ui/PanelHeader.vue";
import { useSettingsStore } from "@/stores/settings";
import type { Settings } from "@/stores/settings";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const settingsStore = useSettingsStore();

const form = ref<Settings>(structuredClone(toRaw(settingsStore.settings)));

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
    }, 800);
  } catch (e) {
    saveError.value = String(e);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="h-full bg-[#111318] border-l border-white/10 flex flex-col overflow-hidden">
    <PanelHeader title="Keybinds" @close="emit('close')" />

    <!-- Body -->
    <div class="flex-1 overflow-y-auto px-5 py-4 space-y-5">
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

      <!-- Pin Location -->
      <div class="space-y-1.5">
        <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">Pin Location (Ctrl+Enter)</label>
        <p class="text-white/30 text-xs">Pins the selected location to scope search results</p>
        <HotkeyCapture v-model="form.keybinds.pin_location" />
      </div>

      <AlertBanner v-if="saveError" variant="error" :message="saveError" />
      <AlertBanner v-if="saveSuccess" variant="success" message="Keybinds saved!" />
    </div>

    <!-- Footer -->
    <div class="px-5 py-4 border-t border-white/10 flex items-center gap-3 flex-shrink-0">
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
</template>
