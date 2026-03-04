<script setup lang="ts">
import { ref } from "vue";
import SortControls from "@/components/ui/SortControls.vue";
import InventoryBar from "@/components/ui/InventoryBar.vue";
import { formatScu, formatPrice, inventoryPercent, relativeAge, shortSystem, shortFaction, shortTerminal, avgOf, avgInventoryPercent } from "@/utils/priceFormatters";
import { richSortOptions, sortEntries } from "@/utils/sorting";
import type { SortOption } from "@/utils/sorting";
import type { PriceEntry } from "@/bindings";

const props = defineProps<{
  buyEntries: PriceEntry[];
  sellEntries: PriceEntry[];
}>();

const buySortKey = ref<keyof PriceEntry>("price_last");
const buySortAsc = ref(true);
const sellSortKey = ref<keyof PriceEntry>("price_last");
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
            class="border border-white/10 rounded-lg bg-white/[0.02] hover:bg-white/[0.05] transition-colors px-2.5 py-1.5"
          >
            <div class="flex items-center justify-between gap-1.5">
              <div class="flex items-center gap-1.5 min-w-0">
                <span class="text-white/80 text-xs font-medium truncate" :title="entry.terminal">{{ shortTerminal(entry.terminal) }}</span>
                <span class="px-1 py-0.5 rounded bg-white/10 text-white/40 text-[0.625rem] font-medium shrink-0">{{ shortSystem(entry.system) }}</span>
              </div>
              <div class="flex items-center gap-1.5 shrink-0">
                <span class="text-orange-300 text-xs font-semibold">{{ formatPrice(entry.price_last) }}</span>
                <span class="text-white/25 text-[0.625rem]">{{ relativeAge(entry.date_updated) }}</span>
              </div>
            </div>
            <div class="flex items-center justify-between gap-1.5 mt-0.5">
              <div class="flex items-center gap-1 text-[0.6875rem] text-white/35 min-w-0 truncate">
                <span>{{ entry.orbit }}</span>
                <template v-if="entry.faction">
                  <span class="text-white/15">·</span>
                  <span>{{ shortFaction(entry.faction) }}</span>
                </template>
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
            class="border border-white/10 rounded-lg bg-white/[0.02] hover:bg-white/[0.05] transition-colors px-2.5 py-1.5"
          >
            <div class="flex items-center justify-between gap-1.5">
              <div class="flex items-center gap-1.5 min-w-0">
                <span class="text-white/80 text-xs font-medium truncate" :title="entry.terminal">{{ shortTerminal(entry.terminal) }}</span>
                <span class="px-1 py-0.5 rounded bg-white/10 text-white/40 text-[0.625rem] font-medium shrink-0">{{ shortSystem(entry.system) }}</span>
              </div>
              <div class="flex items-center gap-1.5 shrink-0">
                <span class="text-orange-300 text-xs font-semibold">{{ formatPrice(entry.price_last) }}</span>
                <span class="text-white/25 text-[0.625rem]">{{ relativeAge(entry.date_updated) }}</span>
              </div>
            </div>
            <div class="flex items-center justify-between gap-1.5 mt-0.5">
              <div class="flex items-center gap-1 text-[0.6875rem] text-white/35 min-w-0 truncate">
                <span>{{ entry.orbit }}</span>
                <template v-if="entry.faction">
                  <span class="text-white/15">·</span>
                  <span>{{ shortFaction(entry.faction) }}</span>
                </template>
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
