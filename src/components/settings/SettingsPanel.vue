<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
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

// Keybind capture state
const capturing = ref(false);
const captureRef = ref<HTMLButtonElement | null>(null);

onMounted(() => {
  form.value = { ...settingsStore.settings };
});

// Stop capturing if the component unmounts while recording
onUnmounted(() => {
  if (capturing.value) {
    stopCapture();
  }
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
  };
}

// ---------------------------------------------------------------------------
// Keybind capture
// ---------------------------------------------------------------------------

/** Map KeyboardEvent to the token string that Rust's parse_hotkey expects.
 *  Uses e.code (physical key) instead of e.key, because e.key produces
 *  locale-dependent characters when modifiers like Alt are held (e.g.
 *  Alt+Shift+S → "Í" on some layouts instead of "S"). */
function keyToToken(e: KeyboardEvent): string | null {
  const code = e.code;

  // Letters: "KeyA" .. "KeyZ" → "A" .. "Z"
  const letterMatch = code.match(/^Key([A-Z])$/);
  if (letterMatch) return letterMatch[1];

  // Digits: "Digit0" .. "Digit9" → "0" .. "9"
  const digitMatch = code.match(/^Digit([0-9])$/);
  if (digitMatch) return digitMatch[1];

  // Function keys: "F1" .. "F12"
  const fMatch = code.match(/^F(\d+)$/);
  if (fMatch) return `F${fMatch[1]}`;

  // Named keys — map code values to parse_hotkey tokens
  const map: Record<string, string> = {
    Space: "Space",
    Tab: "Tab",
    Escape: "Escape",
    Insert: "Insert",
    Delete: "Delete",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
  };

  return map[code] ?? null;
}

function handleCaptureKeyDown(e: KeyboardEvent) {
  e.preventDefault();
  e.stopPropagation();
  e.stopImmediatePropagation();

  // Ignore lone modifier presses — wait for the actual key
  if (["Control", "Alt", "Shift", "Meta"].includes(e.key)) return;

  const token = keyToToken(e);
  if (!token) return; // Unrecognised key, ignore

  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  parts.push(token);

  form.value.hotkey = parts.join("+");
  stopCapture();
}

function handleCaptureBlur() {
  // If the user clicks away, stop capturing without changing the value
  stopCapture();
}

function startCapture() {
  capturing.value = true;
  // Use a document-level capture-phase listener so we intercept the keydown
  // before the webview processes Alt-based accelerators or system keys.
  document.addEventListener("keydown", handleCaptureKeyDown, true);
  requestAnimationFrame(() => {
    captureRef.value?.focus();
  });
}

function stopCapture() {
  capturing.value = false;
  document.removeEventListener("keydown", handleCaptureKeyDown, true);
}
</script>

<template>
  <div class="bg-[#111318] border-l border-white/10 flex flex-col overflow-y-auto">
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
        <button
          ref="captureRef"
          @click="startCapture"
          @blur="handleCaptureBlur"
          class="w-full bg-white/5 border rounded-lg px-3 py-2 text-sm text-left transition-colors focus:outline-none"
          :class="capturing
            ? 'border-blue-500 text-blue-400 animate-pulse'
            : 'border-white/10 text-white hover:border-white/20'"
        >
          {{ capturing ? "Press a key combo..." : form.hotkey || "Click to set hotkey" }}
        </button>
        <p class="text-white/30 text-xs mt-1">Click the field, then press your desired key combination</p>
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
          Background Opacity: {{ Math.round(form.overlay_opacity * 100) }}%
        </label>
        <input
          v-model.number="form.overlay_opacity"
          type="range"
          min="0"
          max="1"
          step="0.05"
          class="w-full accent-blue-500"
        />
        <p class="text-white/30 text-xs mt-1">Controls background dimming only — UI is always fully visible</p>
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
