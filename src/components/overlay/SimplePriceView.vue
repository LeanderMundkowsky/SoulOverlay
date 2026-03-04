<script setup lang="ts">
import { ref } from "vue";
import SortControls from "@/components/ui/SortControls.vue";
import { formatSimplePrice } from "@/utils/priceFormatters";
import { simpleSortOptions, sortEntries } from "@/utils/sorting";
import type { SortOption } from "@/utils/sorting";
import type { PriceEntry } from "@/bindings";

const props = defineProps<{
  buyEntries: PriceEntry[];
  sellEntries: PriceEntry[];
}>();

const buySortKey = ref<keyof PriceEntry>("buy_price");
const buySortAsc = ref(true);
const sellSortKey = ref<keyof PriceEntry>("sell_price");
const sellSortAsc = ref(false);

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
</script>

<template>
  <div class="flex flex-1 overflow-hidden">
    <!-- Buy column -->
    <div v-if="buyEntries.length > 0" class="flex flex-col min-w-0" :class="sellEntries.length > 0 ? 'flex-1 border-r border-white/5' : 'flex-1'">
      <div class="flex items-center justify-between px-3 py-1.5 border-b border-white/5 shrink-0">
        <span class="text-xs font-medium text-green-400/70">Buy ({{ buyEntries.length }})</span>
        <SortControls
          :options="simpleSortOptions"
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
          class="border border-white/10 rounded-lg bg-white/[0.02] hover:bg-white/[0.05] transition-colors px-2.5 py-1.5"
        >
          <div class="flex items-center justify-between gap-2">
            <span class="text-white/80 text-xs font-medium truncate" :title="entry.terminal">{{ entry.terminal }}</span>
            <span class="text-green-400 text-xs font-semibold shrink-0">{{ formatSimplePrice(entry.buy_price) }}</span>
          </div>
          <div class="flex items-center justify-between gap-2 mt-0.5">
            <span class="text-white/30 text-[0.6875rem] truncate">{{ entry.location }}</span>
            <span v-if="entry.rent_price > 0" class="text-yellow-400/60 text-[0.6875rem] shrink-0">Rent: {{ formatSimplePrice(entry.rent_price) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Sell column -->
    <div v-if="sellEntries.length > 0" class="flex flex-col min-w-0 flex-1">
      <div class="flex items-center justify-between px-3 py-1.5 border-b border-white/5 shrink-0">
        <span class="text-xs font-medium text-blue-400/70">Sell ({{ sellEntries.length }})</span>
        <SortControls
          :options="simpleSortOptions"
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
          class="border border-white/10 rounded-lg bg-white/[0.02] hover:bg-white/[0.05] transition-colors px-2.5 py-1.5"
        >
          <div class="flex items-center justify-between gap-2">
            <span class="text-white/80 text-xs font-medium truncate" :title="entry.terminal">{{ entry.terminal }}</span>
            <span class="text-blue-400 text-xs font-semibold shrink-0">{{ formatSimplePrice(entry.sell_price) }}</span>
          </div>
          <div class="flex items-center justify-between gap-2 mt-0.5">
            <span class="text-white/30 text-[0.6875rem] truncate">{{ entry.location }}</span>
            <span v-if="entry.rent_price > 0" class="text-yellow-400/60 text-[0.6875rem] shrink-0">Rent: {{ formatSimplePrice(entry.rent_price) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
