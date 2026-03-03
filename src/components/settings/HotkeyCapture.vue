<script setup lang="ts">
import { ref, onUnmounted } from "vue";

defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const capturing = ref(false);
const captureRef = ref<HTMLButtonElement | null>(null);

onUnmounted(() => {
  if (capturing.value) stopCapture();
});

/** Map KeyboardEvent to the token string that Rust's parse_hotkey expects.
 *  Uses e.code (physical key) instead of e.key, because e.key produces
 *  locale-dependent characters when modifiers like Alt are held. */
function keyToToken(e: KeyboardEvent): string | null {
  const code = e.code;

  const letterMatch = code.match(/^Key([A-Z])$/);
  if (letterMatch) return letterMatch[1];

  const digitMatch = code.match(/^Digit([0-9])$/);
  if (digitMatch) return digitMatch[1];

  const fMatch = code.match(/^F(\d+)$/);
  if (fMatch) return `F${fMatch[1]}`;

  const map: Record<string, string> = {
    Space: "Space",
    Tab: "Tab",
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

  if (["Control", "Alt", "Shift", "Meta"].includes(e.key)) return;

  // ESC cancels capture without assigning
  if (e.code === "Escape") {
    stopCapture();
    return;
  }

  const token = keyToToken(e);
  if (!token) return;

  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  parts.push(token);

  emit("update:modelValue", parts.join("+"));
  stopCapture();
}

function handleCaptureBlur() {
  stopCapture();
}

function startCapture() {
  capturing.value = true;
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
  <button
    ref="captureRef"
    @click="startCapture"
    @blur="handleCaptureBlur"
    class="w-full bg-white/5 border rounded-lg px-3 py-2 text-sm text-left transition-colors focus:outline-none"
    :class="capturing
      ? 'border-blue-500 text-blue-400 animate-pulse'
      : 'border-white/10 text-white hover:border-white/20'"
  >
    {{ capturing ? "Press a key combo..." : modelValue || "Click to set hotkey" }}
  </button>
</template>
