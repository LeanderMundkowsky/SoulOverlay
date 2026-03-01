<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from "vue";
import IconSearch from "@/components/icons/IconSearch.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import SearchResultRow from "@/components/overlay/SearchResultRow.vue";
import { useUex, type UexResult } from "@/composables/useUex";

const emit = defineEmits<{
  (e: "select", result: { id: string; name: string; kind: string }): void;
}>();

const query = ref("");
const { loading, error, results, stale, search } = useUex();
const activeIndex = ref(-1);
const inputEl = ref<HTMLInputElement | null>(null);
const rowRefs = ref<InstanceType<typeof SearchResultRow>[]>([]);

onMounted(() => {
  inputEl.value?.focus();
});

watch(results, () => {
  activeIndex.value = -1;
  rowRefs.value = [];
});

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(query, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => { search(val); }, 300);
});

function selectResult(result: UexResult) {
  emit("select", { id: result.id, name: result.name, kind: result.kind });
}

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
    if (index < results.value.length - 1) setActiveIndex(index + 1);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (index > 0) {
      setActiveIndex(index - 1);
    } else {
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
    const el = rowRefs.value[index]?.rootEl;
    el?.focus();
    el?.scrollIntoView({ block: "nearest" });
  });
}

function setRowRef(el: InstanceType<typeof SearchResultRow> | null, index: number) {
  if (el) rowRefs.value[index] = el;
}

function focusInput() {
  inputEl.value?.focus();
}

defineExpose({ focusInput, stale });
</script>

<template>
  <div class="flex flex-col">
    <!-- Search input -->
    <div class="p-4 border-b border-white/5">
      <div class="relative">
        <IconSearch class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-white/40" />
        <input
          ref="inputEl"
          v-model="query"
          type="text"
          placeholder="Search commodities, vehicles, items, locations…"
          class="w-full bg-white/5 border border-white/10 rounded-lg pl-10 pr-10 py-2.5 text-white placeholder-white/30 focus:outline-none focus:border-blue-500/50 focus:bg-white/10 transition-colors text-sm"
          @keydown="onInputKeydown"
        />
        <div v-if="loading" class="absolute right-3 top-1/2 -translate-y-1/2">
          <LoadingSpinner size="sm" />
        </div>
      </div>
      <div v-if="error" class="mt-2 text-red-400 text-xs px-1">{{ error }}</div>
    </div>

    <!-- Results list -->
    <div v-if="results.length > 0 && query.trim()">
      <div class="px-4 py-2 flex items-center gap-2">
        <span class="text-white/30 text-xs uppercase tracking-widest">Results</span>
        <span class="bg-white/10 text-white/50 text-[10px] font-bold px-1.5 py-0.5 rounded-full">
          {{ results.length }}
        </span>
      </div>

      <SearchResultRow
        v-for="(result, index) in results"
        :key="result.id"
        :ref="(el) => setRowRef(el as InstanceType<typeof SearchResultRow> | null, index)"
        :result="result"
        :is-active="activeIndex === index"
        tabindex="0"
        @keydown="(e: KeyboardEvent) => onRowKeydown(e, index, result)"
        @focus="activeIndex = index"
        @select="selectResult(result)"
      />
    </div>

    <!-- No results -->
    <div
      v-else-if="!loading && query.trim().length > 2"
      class="px-4 py-8 text-center text-white/30 text-sm"
    >
      No results found for "{{ query }}"
    </div>

    <!-- Empty hint -->
    <div
      v-else-if="!query.trim()"
      class="px-4 py-6 text-center text-white/20 text-xs select-none"
    >
      Type to search commodities, vehicles, items, locations&hellip;
    </div>
  </div>
</template>
