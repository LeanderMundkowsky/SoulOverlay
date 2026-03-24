<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import IconClose from "@/components/icons/IconClose.vue";
import IconChevronDown from "@/components/icons/IconChevronDown.vue";

export interface DropdownOption {
  id: string;
  label: string;
  /** Optional right-aligned metadata (e.g. item kind, location type). */
  meta?: string;
}

const props = withDefaults(
  defineProps<{
    modelValue: DropdownOption | null;
    options: DropdownOption[];
    placeholder?: string;
    loading?: boolean;
    disabled?: boolean;
    clearable?: boolean;
    /** Whether to show the "✓ meta" confirmation hint below the input (default: true). */
    showMeta?: boolean;
  }>(),
  { placeholder: "Search...", loading: false, disabled: false, clearable: true, showMeta: true },
);

const emit = defineEmits<{
  (e: "update:modelValue", value: DropdownOption | null): void;
  /** Fired on every keystroke — lets the parent drive async searches. */
  (e: "search", query: string): void;
}>();

// ── Internal state ─────────────────────────────────────────────────────────

const inputEl = ref<HTMLInputElement | null>(null);
const query = ref(props.modelValue?.label ?? "");
const open = ref(false);
const highlightIndex = ref(-1);
const isFocused = ref(false);
const dropdownId = `sd-${Math.random().toString(36).slice(2, 9)}`;

// ── Sync query from external modelValue changes (e.g. prefill, clear) ──────

watch(
  () => props.modelValue,
  (val) => {
    // Only update when the input isn't being actively edited.
    if (!isFocused.value) {
      query.value = val?.label ?? "";
    }
  },
);

// ── Filtered options (client-side) ─────────────────────────────────────────

const filteredOptions = computed(() => {
  // When something is selected the query shows its label — don't filter by that.
  if (props.modelValue) return props.options;
  if (!query.value) return props.options;
  const q = query.value.toLowerCase();
  return props.options.filter((o) => o.label.toLowerCase().includes(q));
});

// Open dropdown when async options arrive (parent updates after network call).
watch(
  () => props.options,
  (opts) => {
    if (isFocused.value && query.value && !props.modelValue) {
      open.value = opts.length > 0;
      highlightIndex.value = -1;
    }
  },
);

// Reset highlight when visible options change.
watch(filteredOptions, () => {
  highlightIndex.value = -1;
});

// ── Input events ───────────────────────────────────────────────────────────

function onInput(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  query.value = val;
  // Clear confirmed selection when user starts typing.
  if (props.modelValue !== null) {
    emit("update:modelValue", null);
  }
  emit("search", val);
  open.value = filteredOptions.value.length > 0;
  highlightIndex.value = -1;
}

function onFocus() {
  isFocused.value = true;
  if (!props.modelValue && filteredOptions.value.length > 0) {
    open.value = true;
  }
}

function onBlur() {
  isFocused.value = false;
  // Delay so mousedown.prevent on dropdown items can fire first.
  setTimeout(() => {
    open.value = false;
    // Reset free-typed text that didn't result in a selection.
    query.value = props.modelValue?.label ?? "";
  }, 150);
}

// ── Keyboard navigation ────────────────────────────────────────────────────

function scrollToHighlight() {
  nextTick(() => {
    document
      .querySelector(`#${dropdownId} [data-idx="${highlightIndex.value}"]`)
      ?.scrollIntoView({ block: "nearest" });
  });
}

function onKeyDown(e: KeyboardEvent) {
  if (!open.value) {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      open.value = filteredOptions.value.length > 0;
    }
    return;
  }
  if (e.key === "ArrowDown") {
    e.preventDefault();
    highlightIndex.value = Math.min(
      highlightIndex.value + 1,
      filteredOptions.value.length - 1,
    );
    scrollToHighlight();
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    highlightIndex.value = Math.max(highlightIndex.value - 1, 0);
    scrollToHighlight();
  } else if (e.key === "Enter" && highlightIndex.value >= 0) {
    e.preventDefault();
    select(filteredOptions.value[highlightIndex.value]);
  } else if (e.key === "Escape") {
    // Close dropdown but let Escape bubble so modal can respond too.
    open.value = false;
    if (!props.modelValue) query.value = "";
  }
}

// ── Selection & clear ──────────────────────────────────────────────────────

function select(opt: DropdownOption) {
  emit("update:modelValue", opt);
  query.value = opt.label;
  open.value = false;
  highlightIndex.value = -1;
}

function clear() {
  emit("update:modelValue", null);
  query.value = "";
  open.value = false;
}

// ── Exposed API (used by parent modal for keydown guard) ───────────────────

defineExpose({
  focus: () => inputEl.value?.focus(),
  isDropdownActive: () => open.value && highlightIndex.value >= 0,
});
</script>

<template>
  <div>
    <!-- Input row — own relative container so top-1/2 is always relative to input height -->
    <div class="relative">
      <input
        ref="inputEl"
        type="text"
        :value="query"
        :placeholder="placeholder"
        :disabled="disabled"
        autocomplete="off"
        spellcheck="false"
        class="w-full bg-[#111318] border rounded-lg px-3 py-2 pr-8 text-white text-sm placeholder-white/20 focus:outline-none transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
        :class="modelValue ? 'border-green-500/40' : 'border-white/10 focus:border-white/30'"
        @input="onInput"
        @focus="onFocus"
        @blur="onBlur"
        @keydown="onKeyDown"
      />

      <!-- Right-side icon: clear (when selected + clearable), loading dots, or chevron -->
      <button
        v-if="modelValue && clearable"
        tabindex="-1"
        class="absolute right-2 top-1/2 -translate-y-1/2 w-5 h-5 flex items-center justify-center text-white/30 hover:text-white transition-colors"
        @mousedown.prevent="clear"
      >
        <IconClose class="w-3.5 h-3.5" />
      </button>
      <span
        v-else-if="loading"
        class="absolute right-2.5 top-1/2 -translate-y-1/2 text-white/20 text-xs select-none pointer-events-none"
      >
        •••
      </span>
      <button
        v-else
        tabindex="-1"
        class="absolute right-2 top-1/2 -translate-y-1/2 w-5 h-5 flex items-center justify-center transition-colors"
        :class="modelValue ? 'text-green-500/60 hover:text-green-400' : 'text-white/25 hover:text-white/50'"
        :disabled="disabled"
        @mousedown.prevent="open ? (open = false) : (filteredOptions.length > 0 && (open = true), inputEl?.focus())"
      >
        <IconChevronDown
          class="w-3.5 h-3.5 transition-transform"
          :class="open ? 'rotate-180' : ''"
        />
      </button>

      <!-- Dropdown list -->
      <div
        v-if="open && filteredOptions.length > 0"
        :id="dropdownId"
        class="absolute z-10 left-0 right-0 top-full mt-1 bg-[#1e2130] border border-white/10 rounded-lg shadow-xl max-h-[200px] overflow-y-auto"
      >
        <button
          v-for="(opt, i) in filteredOptions"
          :key="opt.id"
          :data-idx="i"
          class="w-full text-left px-3 py-2 text-sm transition-colors flex items-center gap-2"
          :class="
            i === highlightIndex
              ? 'bg-white/10 text-white'
              : 'hover:bg-white/[0.08] text-white'
          "
          @mousedown.prevent="select(opt)"
        >
          <span class="truncate">{{ opt.label }}</span>
          <span v-if="opt.meta" class="text-white/30 text-xs ml-auto uppercase shrink-0">
            {{ opt.meta }}
          </span>
        </button>
      </div>
    </div>

    <!-- Confirmed-selection indicator -->
    <div v-if="showMeta && modelValue?.meta" class="text-green-400/60 text-xs mt-1">
      ✓ {{ modelValue.meta }}
    </div>
  </div>
</template>
