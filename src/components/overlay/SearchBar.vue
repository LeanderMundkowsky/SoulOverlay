<script setup lang="ts">
import { ref, watch } from "vue";
import { useUex, type UexResult } from "@/composables/useUex";

const emit = defineEmits<{
  (e: "select", commodity: { id: string; name: string }): void;
}>();

const query = ref("");
const { loading, error, results, search } = useUex();

// Debounced search
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(query, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    search(val);
  }, 300);
});

function selectResult(result: UexResult) {
  emit("select", { id: result.id, name: result.name });
  query.value = "";
  results.value = [];
}
</script>

<template>
  <div class="relative">
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
        v-model="query"
        type="text"
        placeholder="Search UEX commodities..."
        class="w-full bg-white/5 border border-white/10 rounded-lg pl-10 pr-4 py-2.5 text-white placeholder-white/30 focus:outline-none focus:border-blue-500/50 focus:bg-white/10 transition-colors text-sm"
      />
      <div
        v-if="loading"
        class="absolute right-3 top-1/2 -translate-y-1/2"
      >
        <div class="w-4 h-4 border-2 border-blue-400/30 border-t-blue-400 rounded-full animate-spin"></div>
      </div>
    </div>

    <!-- Error message -->
    <div
      v-if="error"
      class="mt-2 text-red-400 text-xs px-2"
    >
      {{ error }}
    </div>

    <!-- Results dropdown -->
    <div
      v-if="results.length > 0 && query.trim()"
      class="absolute top-full left-0 right-0 mt-1 bg-gray-900/95 border border-white/10 rounded-lg shadow-2xl max-h-64 overflow-y-auto z-50"
    >
      <button
        v-for="result in results"
        :key="result.id"
        @click="selectResult(result)"
        class="w-full text-left px-4 py-2.5 hover:bg-white/10 transition-colors border-b border-white/5 last:border-b-0"
      >
        <div class="flex items-center justify-between">
          <span class="text-white text-sm">{{ result.name }}</span>
          <span class="text-white/40 text-xs uppercase tracking-wide">{{ result.kind }}</span>
        </div>
      </button>
    </div>

    <!-- No results -->
    <div
      v-if="!loading && results.length === 0 && query.trim().length > 2"
      class="absolute top-full left-0 right-0 mt-1 bg-gray-900/95 border border-white/10 rounded-lg px-4 py-3 text-white/40 text-sm"
    >
      No results found for "{{ query }}"
    </div>
  </div>
</template>
