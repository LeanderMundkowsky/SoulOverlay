<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from "vue";
import SortControls from "@/components/ui/SortControls.vue";
import InventoryBar from "@/components/ui/InventoryBar.vue";
import IconEye from "@/components/icons/IconEye.vue";
import { formatScu, formatPrice, inventoryPercent, relativeAge, shortSystem, shortTerminal, locationPath, avgOf, avgInventoryPercent } from "@/utils/priceFormatters";
import { richSortOptions, sortEntries } from "@/utils/sorting";
import type { SortOption } from "@/utils/sorting";
import type { PriceEntry } from "@/bindings";
import { useWatchlistStore } from "@/stores/watchlist";
import { startPriceDrag } from "@/composables/useDragDrop";

const props = defineProps<{
  buyEntries: PriceEntry[];
  sellEntries: PriceEntry[];
  entityId?: string;
  entityName?: string;
  entityKind?: string;
  entitySlug?: string;
  active?: boolean;
}>();

const watchlistStore = useWatchlistStore();

const buySortKey = ref<keyof PriceEntry>("price_last");
const buySortAsc = ref(true);
const sellSortKey = ref<keyof PriceEntry>("price_last");
const sellSortAsc = ref(false);

const highlightedTerminalId = ref<string | null>(null);
const highlightPulsing = ref(false);

function sortedBuy(): PriceEntry[] {
  return sortEntries(props.buyEntries, buySortKey.value, buySortAsc.value);
}

function sortedSell(): PriceEntry[] {
  return sortEntries(props.sellEntries, sellSortKey.value, sellSortAsc.value);
}

function onBuySelect(opt: SortOption) {
  buySortKey.value = opt.key;
  buySortAsc.value = opt.defaultAsc;
}

function onSellSelect(opt: SortOption) {
  sellSortKey.value = opt.key;
  sellSortAsc.value = opt.defaultAsc;
}

function toggleWatch(entry: PriceEntry, priceType: string) {
  if (!props.entityId) return;
  const tid = entry.terminal_id ?? "";
  const tname = entry.terminal ?? "";
  if (watchlistStore.isWatched(props.entityId, tid, priceType)) {
    watchlistStore.removeEntry(props.entityId, tid, priceType);
  } else {
    watchlistStore.addEntry({
      entityId: props.entityId,
      entityName: props.entityName ?? "",
      entityKind: props.entityKind ?? "",
      entitySlug: props.entitySlug ?? "",
      terminalId: tid,
      terminalName: tname,
      priceType,
    });
  }
}

function onPriceDrag(e: PointerEvent, entry: PriceEntry, priceType: string) {
  if (!props.entityId) return;
  startPriceDrag(e, {
    entityId: props.entityId,
    entityName: props.entityName ?? "",
    entityKind: props.entityKind ?? "",
    entitySlug: props.entitySlug ?? "",
    terminalId: entry.terminal_id ?? "",
    terminalName: entry.terminal ?? "",
    priceType,
  });
}

// Apply highlight: scroll into view + pulse for 2s, then keep static border
function applyHighlight(terminalId: string) {
  highlightedTerminalId.value = terminalId;
  highlightPulsing.value = true;
  // Double nextTick: first for Vue to update :class bindings, second for DOM paint
  nextTick(() => {
    nextTick(() => {
      const el = document.querySelector(`[data-terminal-id="${terminalId}"]`);
      el?.scrollIntoView({ behavior: "smooth", block: "center" });
    });
  });
  setTimeout(() => { highlightPulsing.value = false; }, 1000);
}

// Try to consume the highlight target once the matching element exists in entries
function tryConsumeHighlight() {
  if (props.active === false) return;
  const target = watchlistStore.highlightTarget;
  if (!target) return;
  const allEntries = [...props.buyEntries, ...props.sellEntries];
  const found = allEntries.some((e) => e.terminal_id === target.terminalId);
  if (found) {
    watchlistStore.highlightTarget = null;
    applyHighlight(target.terminalId);
  }
}

// Watch price entries changing — handles async data loading
watch([() => props.buyEntries.length, () => props.sellEntries.length], () => tryConsumeHighlight());

// Watch highlight target directly — handles switching between entries of the same entity
watch(() => watchlistStore.highlightTarget, () => tryConsumeHighlight());

// Also try on mount (for cases where entries are already populated)
onMounted(() => nextTick(() => tryConsumeHighlight()));
</script>

<template>
  <div class="flex flex-col flex-1 overflow-hidden">
    <div class="flex flex-1 overflow-hidden">
      <!-- Buy column -->
      <div v-if="buyEntries.length > 0" class="flex flex-col min-w-0" :class="sellEntries.length > 0 ? 'flex-1 border-r border-white/5' : 'flex-1'">
        <div class="flex items-center justify-between px-3 py-1.5 border-b border-white/5 shrink-0">
          <span class="text-xs font-medium text-green-400/70">Buy ({{ buyEntries.length }})</span>
          <SortControls
            :options="richSortOptions"
            :current-key="buySortKey"
            :ascending="buySortAsc"
            @select="onBuySelect"
            @toggle-direction="buySortAsc = !buySortAsc"
          />
        </div>
        <div class="overflow-y-auto flex-1 p-1.5 space-y-1">
          <div
            v-for="(entry, idx) in sortedBuy()"
            :key="idx"
            :data-terminal-id="entry.terminal_id"
            class="border rounded-lg bg-white/[0.02] hover:bg-white/[0.05] transition-all px-2.5 py-1.5"
            :class="[
              highlightedTerminalId === entry.terminal_id
                ? ['border-blue-400/60', highlightPulsing ? 'watch-glow' : '']
                : 'border-white/10'
            ]"
            @pointerdown="onPriceDrag($event, entry, 'buy')"
          >
            <div class="flex items-center justify-between gap-1.5">
              <div class="flex items-center gap-1.5 min-w-0">
                <span class="text-white/80 text-xs font-medium truncate" :title="entry.terminal">{{ shortTerminal(entry.terminal) }}</span>
                <span class="px-1 py-0.5 rounded bg-white/10 text-white/40 text-[0.625rem] font-medium shrink-0">{{ shortSystem(entry.system) }}</span>
              </div>
              <div class="flex items-center gap-1.5 shrink-0">
                <button
                  v-if="entityId"
                  class="p-0.5 rounded hover:bg-white/10 transition-colors"
                  :title="watchlistStore.isWatched(entityId, entry.terminal_id ?? '', 'buy') ? 'Remove from watch list' : 'Add to watch list'"
                  @click.stop="toggleWatch(entry, 'buy')"
                  @pointerdown.stop
                >
                  <IconEye class="w-3 h-3" :class="watchlistStore.isWatched(entityId, entry.terminal_id ?? '', 'buy') ? 'text-blue-400' : 'text-white/20 hover:text-blue-400'" />
                </button>
                <span class="text-orange-300 text-xs font-semibold">{{ formatPrice(entry.price_last) }}</span>
                <span class="text-white/25 text-[0.625rem]">{{ relativeAge(entry.date_updated) }}</span>
              </div>
            </div>
            <div class="flex items-center justify-between gap-1.5 mt-0.5">
              <div class="flex items-center gap-1 text-[0.6875rem] text-white/35 min-w-0 truncate">
                <span>{{ locationPath(entry, false) }}</span>
              </div>
              <div class="flex items-center gap-2 shrink-0 text-[0.6875rem]">
                <span class="text-white/40">{{ formatScu(entry.scu_last) }}<span class="text-white/20"> / {{ formatScu(entry.scu_max) }}</span></span>
                <span v-if="entry.container_sizes" class="text-white/20">{{ entry.container_sizes }}</span>
              </div>
            </div>
            <InventoryBar :percent="inventoryPercent(entry)" class="mt-1" />
          </div>
        </div>
      </div>

      <!-- Sell column -->
      <div v-if="sellEntries.length > 0" class="flex flex-col min-w-0 flex-1">
        <div class="flex items-center justify-between px-3 py-1.5 border-b border-white/5 shrink-0">
          <span class="text-xs font-medium text-blue-400/70">Sell ({{ sellEntries.length }})</span>
          <SortControls
            :options="richSortOptions"
            :current-key="sellSortKey"
            :ascending="sellSortAsc"
            @select="onSellSelect"
            @toggle-direction="sellSortAsc = !sellSortAsc"
          />
        </div>
        <div class="overflow-y-auto flex-1 p-1.5 space-y-1">
          <div
            v-for="(entry, idx) in sortedSell()"
            :key="idx"
            :data-terminal-id="entry.terminal_id"
            class="border rounded-lg bg-white/[0.02] hover:bg-white/[0.05] transition-all px-2.5 py-1.5"
            :class="[
              highlightedTerminalId === entry.terminal_id
                ? ['border-blue-400/60', highlightPulsing ? 'watch-glow' : '']
                : 'border-white/10'
            ]"
            @pointerdown="onPriceDrag($event, entry, 'sell')"
          >
            <div class="flex items-center justify-between gap-1.5">
              <div class="flex items-center gap-1.5 min-w-0">
                <span class="text-white/80 text-xs font-medium truncate" :title="entry.terminal">{{ shortTerminal(entry.terminal) }}</span>
                <span class="px-1 py-0.5 rounded bg-white/10 text-white/40 text-[0.625rem] font-medium shrink-0">{{ shortSystem(entry.system) }}</span>
              </div>
              <div class="flex items-center gap-1.5 shrink-0">
                <button
                  v-if="entityId"
                  class="p-0.5 rounded hover:bg-white/10 transition-colors"
                  :title="watchlistStore.isWatched(entityId, entry.terminal_id ?? '', 'sell') ? 'Remove from watch list' : 'Add to watch list'"
                  @click.stop="toggleWatch(entry, 'sell')"
                  @pointerdown.stop
                >
                  <IconEye class="w-3 h-3" :class="watchlistStore.isWatched(entityId, entry.terminal_id ?? '', 'sell') ? 'text-blue-400' : 'text-white/20 hover:text-blue-400'" />
                </button>
                <span class="text-orange-300 text-xs font-semibold">{{ formatPrice(entry.price_last) }}</span>
                <span class="text-white/25 text-[0.625rem]">{{ relativeAge(entry.date_updated) }}</span>
              </div>
            </div>
            <div class="flex items-center justify-between gap-1.5 mt-0.5">
              <div class="flex items-center gap-1 text-[0.6875rem] text-white/35 min-w-0 truncate">
                <span>{{ locationPath(entry, false) }}</span>
              </div>
              <div class="flex items-center gap-2 shrink-0 text-[0.6875rem]">
                <span class="text-white/40">{{ formatScu(entry.scu_last) }}<span class="text-white/20"> / {{ formatScu(entry.scu_max) }}</span></span>
                <span v-if="entry.container_sizes" class="text-white/20">{{ entry.container_sizes }}</span>
              </div>
            </div>
            <InventoryBar :percent="inventoryPercent(entry)" class="mt-1" />
          </div>
        </div>
      </div>
    </div>

    <!-- Summary footer -->
    <div v-if="buyEntries.length > 1 || sellEntries.length > 1" class="flex border-t border-white/10 text-[0.6875rem] shrink-0">
      <div v-if="buyEntries.length > 1" class="flex items-center gap-3 px-3 py-1.5 bg-white/[0.03]" :class="sellEntries.length > 0 ? 'flex-1 border-r border-white/5' : 'flex-1'">
        <span class="text-white/30">{{ buyEntries.length }}loc</span>
        <span class="text-orange-300/60">ø {{ formatPrice(avgOf(buyEntries, "price_last")) }}</span>
        <div class="flex items-center gap-1 flex-1">
          <div class="flex-1 h-[2px] bg-white/5 rounded-full overflow-hidden">
            <div class="h-full rounded-full bg-green-400/40" :style="{ width: avgInventoryPercent(buyEntries) + '%' }"></div>
          </div>
          <span class="text-white/20">{{ avgInventoryPercent(buyEntries) }}%</span>
        </div>
      </div>
      <div v-if="sellEntries.length > 1" class="flex items-center gap-3 px-3 py-1.5 bg-white/[0.03] flex-1">
        <span class="text-white/30">{{ sellEntries.length }}loc</span>
        <span class="text-orange-300/60">ø {{ formatPrice(avgOf(sellEntries, "price_last")) }}</span>
        <div class="flex items-center gap-1 flex-1">
          <div class="flex-1 h-[2px] bg-white/5 rounded-full overflow-hidden">
            <div class="h-full rounded-full bg-blue-400/40" :style="{ width: avgInventoryPercent(sellEntries) + '%' }"></div>
          </div>
          <span class="text-white/20">{{ avgInventoryPercent(sellEntries) }}%</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.watch-glow {
  animation: border-glow 1s ease-in-out 1;
}

@keyframes border-glow {
  0%, 100% { box-shadow: 0 0 0 0 rgba(96, 165, 250, 0); }
  50% { box-shadow: 0 0 8px 2px rgba(96, 165, 250, 0.45); }
}
</style>
