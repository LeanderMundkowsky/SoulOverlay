<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from "vue";
import { useUex, type UexResult } from "@/composables/useUex";

const emit = defineEmits<{
  (e: "select", commodity: { id: string; name: string }): void;
}>();

const query = ref("");
const { loading, error, results, search } = useUex();

// -1 means nothing highlighted; >= 0 is the active row index
const activeIndex = ref(-1);

const inputEl = ref<HTMLInputElement | null>(null);
const rowEls = ref<HTMLElement[]>([]);

// Auto-focus input on mount (search tab is v-if, so this fires on every tab switch)
onMounted(() => {
  inputEl.value?.focus();
});

// Reset highlight whenever the result list changes
watch(results, () => {
  activeIndex.value = -1;
});

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(query, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    search(val);
  }, 300);
});

function selectResult(result: UexResult) {
  emit("select", { id: result.id, name: result.name });
}

// ── Keyboard handling ──────────────────────────────────────────────────────

function onInputKeydown(e: KeyboardEvent) {
  if (!results.value.length && e.key !== "Escape") return;

  if (e.key === "ArrowDown") {
    e.preventDefault();
    setActiveIndex(0);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    setActiveIndex(results.value.length - 1);
  } else if (e.key === "Escape") {
    e.preventDefault();
    // Only consume the event if there is something to clear.
    // If the field is already empty, let it propagate so App.vue can close the overlay.
    if (query.value || results.value.length) {
      e.stopImmediatePropagation();
      query.value = "";
      results.value = [];
    }
  }
}

function onRowKeydown(e: KeyboardEvent, index: number, result: UexResult) {
  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (index < results.value.length - 1) {
      setActiveIndex(index + 1);
    }
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (index > 0) {
      setActiveIndex(index - 1);
    } else {
      // Back to input
      activeIndex.value = -1;
      inputEl.value?.focus();
    }
  } else if (e.key === "Enter") {
    e.preventDefault();
    selectResult(result);
  } else if (e.key === "Escape") {
    e.preventDefault();
    e.stopImmediatePropagation();
    query.value = "";
    results.value = [];
    activeIndex.value = -1;
    inputEl.value?.focus();
  }
}

function setActiveIndex(index: number) {
  activeIndex.value = index;
  nextTick(() => {
    rowEls.value[index]?.focus();
    rowEls.value[index]?.scrollIntoView({ block: "nearest" });
  });
}

// Collect row element refs by index
function setRowRef(el: Element | null, index: number) {
  if (el instanceof HTMLElement) {
    rowEls.value[index] = el;
  }
}

// Exposed so App.vue can focus the input after overlay-shown
function focusInput() {
  inputEl.value?.focus();
}

defineExpose({ focusInput });
</script>

<template>
  <div class="flex flex-col">
    <!-- Search input -->
    <div class="p-4 border-b border-white/5">
      <div class="relative">
        <svg
          class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-white/40"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          ref="inputEl"
          v-model="query"
          type="text"
          placeholder="Search UEX commodities..."
          class="w-full bg-white/5 border border-white/10 rounded-lg pl-10 pr-10 py-2.5 text-white placeholder-white/30 focus:outline-none focus:border-blue-500/50 focus:bg-white/10 transition-colors text-sm"
          @keydown="onInputKeydown"
        />
        <div v-if="loading" class="absolute right-3 top-1/2 -translate-y-1/2">
          <div class="w-4 h-4 border-2 border-blue-400/30 border-t-blue-400 rounded-full animate-spin"></div>
        </div>
      </div>

      <!-- Error -->
      <div v-if="error" class="mt-2 text-red-400 text-xs px-1">
        {{ error }}
      </div>
    </div>

    <!-- Results list -->
    <div v-if="results.length > 0 && query.trim()">
      <!-- Result count -->
      <div class="px-4 py-2 flex items-center gap-2">
        <span class="text-white/30 text-xs uppercase tracking-widest">Results</span>
        <span class="bg-white/10 text-white/50 text-[10px] font-bold px-1.5 py-0.5 rounded-full">
          {{ results.length }}
        </span>
      </div>

      <!-- Rows -->
      <div
        v-for="(result, index) in results"
        :key="result.id"
        :ref="(el) => setRowRef(el as Element | null, index)"
        tabindex="0"
        class="group flex items-center gap-3 px-4 py-3 border-t border-white/5 transition-colors outline-none cursor-default"
        :class="activeIndex === index ? 'bg-white/8 ring-1 ring-inset ring-blue-500/30' : 'hover:bg-white/5 focus:bg-white/8'"
        @keydown="(e) => onRowKeydown(e, index, result)"
        @focus="activeIndex = index"
      >
        <!-- Entity icon -->
        <div
          class="flex-shrink-0 w-7 h-7 rounded-lg border flex items-center justify-center transition-colors"
          :class="activeIndex === index ? 'bg-blue-500/10 border-blue-500/20 text-blue-400' : 'bg-white/5 border-white/10 text-white/40'"
        >
          <!-- commodity icon -->
          <svg v-if="result.kind === 'commodity'" class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <ellipse cx="12" cy="5" rx="9" ry="3"></ellipse>
            <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path>
            <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>
          </svg>
          <!-- fallback generic icon -->
          <svg v-else class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
            <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
            <line x1="12" y1="22.08" x2="12" y2="12"></line>
          </svg>
        </div>

        <!-- Name + kind -->
        <div class="flex-1 min-w-0 flex items-center gap-2">
          <span class="text-white text-sm font-medium truncate">{{ result.name }}</span>
          <span class="flex-shrink-0 text-[10px] uppercase tracking-widest text-white/30 bg-white/5 border border-white/10 px-1.5 py-0.5 rounded">
            {{ result.kind }}
          </span>
        </div>

        <!-- Action buttons: always visible on keyboard-active row, fade-in on mouse hover -->
        <div
          class="flex items-center gap-1 flex-shrink-0 transition-opacity duration-150"
          :class="activeIndex === index ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'"
        >
          <!-- View prices (Enter shortcut hint shown on active row) -->
          <button
            @click.stop="selectResult(result)"
            class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border text-xs font-medium transition-colors"
            :class="activeIndex === index
              ? 'bg-blue-500/20 border-blue-500/40 text-blue-300 hover:bg-blue-500/30'
              : 'bg-blue-500/10 border-blue-500/20 text-blue-400 hover:bg-blue-500/20 hover:border-blue-500/40'"
            title="View prices (Enter)"
          >
            <svg class="w-3 h-3" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="1" x2="12" y2="23"></line>
              <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"></path>
            </svg>
            Prices
            <span v-if="activeIndex === index" class="ml-0.5 opacity-50 font-normal text-[10px]">↵</span>
          </button>

          <!-- Add to inventory (UI only) -->
          <button
            @click.stop
            class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 border border-white/10 hover:border-white/20 text-white/50 hover:text-white/80 text-xs font-medium transition-colors"
            title="Add to inventory"
          >
            <svg class="w-3 h-3" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
              <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
              <line x1="12" y1="22.08" x2="12" y2="12"></line>
            </svg>
            Add to Inventory
          </button>
        </div>
      </div>
    </div>

    <!-- No results -->
    <div
      v-else-if="!loading && query.trim().length > 2"
      class="px-4 py-8 text-center text-white/30 text-sm"
    >
      No results found for "{{ query }}"
    </div>

    <!-- Empty / hint state -->
    <div
      v-else-if="!query.trim()"
      class="px-4 py-6 text-center text-white/20 text-xs select-none"
    >
      Type to search commodities, ships, items&hellip;
    </div>
  </div>
</template>
