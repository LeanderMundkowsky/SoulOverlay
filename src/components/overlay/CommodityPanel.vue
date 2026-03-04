<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import IconClose from "@/components/icons/IconClose.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import { useUex, type PriceEntry } from "@/composables/useUex";

const props = defineProps<{
  entityId: string;
  entityName: string;
  entityKind: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const { loading, error, prices, getEntityPrices } = useUex();

const buySortKey = ref<keyof PriceEntry>("price_last");
const buySortAsc = ref(true);
const buyDropdownOpen = ref(false);

const sellSortKey = ref<keyof PriceEntry>("price_last");
const sellSortAsc = ref(false);
const sellDropdownOpen = ref(false);

const hasRichData = ref(false);

const sortedBuy = ref<PriceEntry[]>([]);
const sortedSell = ref<PriceEntry[]>([]);

const kindLabels: Record<string, string> = {
  commodity: "Commodity",
  raw_commodity: "Raw Commodity",
  item: "Item",
  vehicle: "Vehicle",
  "ground vehicle": "Ground Vehicle",
  vehicle_rental: "Vehicle Rental",
  fuel: "Fuel",
  location: "Location",
};

interface SortOption {
  key: keyof PriceEntry;
  label: string;
  defaultAsc: boolean;
}

const richSortOptions: SortOption[] = [
  { key: "price_last", label: "Price", defaultAsc: false },
  { key: "scu_last", label: "SCU", defaultAsc: false },
  { key: "terminal", label: "Terminal", defaultAsc: true },
  { key: "date_updated", label: "Age", defaultAsc: false },
];

const simpleSortOptions: SortOption[] = [
  { key: "buy_price", label: "Buy Price", defaultAsc: false },
  { key: "sell_price", label: "Sell Price", defaultAsc: false },
  { key: "terminal", label: "Terminal", defaultAsc: true },
  { key: "rent_price", label: "Rent Price", defaultAsc: false },
];

function sortEntries(source: PriceEntry[], key: keyof PriceEntry, asc: boolean): PriceEntry[] {
  const sorted = [...source];
  sorted.sort((a, b) => {
    const aVal = a[key] ?? 0;
    const bVal = b[key] ?? 0;
    if (typeof aVal === "number" && typeof bVal === "number") {
      return asc ? aVal - bVal : bVal - aVal;
    }
    return asc
      ? String(aVal).localeCompare(String(bVal))
      : String(bVal).localeCompare(String(aVal));
  });
  return sorted;
}

watch(
  [prices, buySortKey, buySortAsc, sellSortKey, sellSortAsc],
  () => {
    const all = [...prices.value];
    sortedBuy.value = sortEntries(all.filter((p) => p.buy_price > 0), buySortKey.value, buySortAsc.value);
    sortedSell.value = sortEntries(all.filter((p) => p.sell_price > 0), sellSortKey.value, sellSortAsc.value);
    hasRichData.value = props.entityKind === "commodity" || props.entityKind === "raw_commodity";
  },
  { immediate: true }
);

function selectBuySort(opt: SortOption) {
  buySortKey.value = opt.key;
  buySortAsc.value = opt.defaultAsc;
  buyDropdownOpen.value = false;
}

function selectSellSort(opt: SortOption) {
  sellSortKey.value = opt.key;
  sellSortAsc.value = opt.defaultAsc;
  sellDropdownOpen.value = false;
}

function sortLabel(key: keyof PriceEntry): string {
  const options = hasRichData.value ? richSortOptions : simpleSortOptions;
  const found = options.find((o) => o.key === key);
  return found ? found.label : "Price";
}

function closeDropdowns() {
  buyDropdownOpen.value = false;
  sellDropdownOpen.value = false;
}

function formatScu(val: number | undefined): string {
  if (!val || val === 0) return "-";
  if (val >= 1000) return (val / 1000).toFixed(1).replace(/\.0$/, "") + "K";
  return val.toLocaleString("en-US", { maximumFractionDigits: 0 });
}

function formatPrice(val: number | undefined): string {
  if (!val || val === 0) return "-";
  return val.toLocaleString("en-US", { minimumFractionDigits: 0, maximumFractionDigits: 0 });
}

function inventoryPercent(entry: PriceEntry): number {
  if (!entry.scu_max || entry.scu_max <= 0) return 0;
  return Math.round(((entry.scu_avg ?? 0) / entry.scu_max) * 100);
}

function inventoryBarColor(pct: number): string {
  if (pct >= 80) return "bg-green-400/50";
  if (pct >= 40) return "bg-yellow-400/50";
  if (pct > 0) return "bg-blue-400/50";
  return "bg-white/10";
}

function relativeAge(timestamp: string): string {
  if (!timestamp) return "-";
  const epoch = parseInt(timestamp, 10);
  if (isNaN(epoch)) return "-";
  const now = Math.floor(Date.now() / 1000);
  const diff = now - epoch;
  if (diff < 60) return "<1m";
  if (diff < 3600) return Math.floor(diff / 60) + "m";
  if (diff < 86400) return Math.floor(diff / 3600) + "h";
  return Math.floor(diff / 86400) + "d";
}

function shortSystem(system: string | undefined): string {
  if (!system) return "—";
  const map: Record<string, string> = {
    Stanton: "ST",
    Pyro: "PY",
    Nyx: "NY",
  };
  return map[system] ?? system.substring(0, 3).toUpperCase();
}

function shortFaction(faction: string | undefined): string {
  if (!faction) return "—";
  const map: Record<string, string> = {
    "United Empire of Earth": "UEE",
    "Citizens for Prosperity": "CitPro",
    "Rest & Relax": "R&R",
  };
  return map[faction] ?? faction;
}

function shortTerminal(terminal: string): string {
  return terminal.replace(/^Admin - /, "");
}

function fetchPrices() {
  getEntityPrices(props.entityKind, props.entityId);
}

onMounted(() => { fetchPrices(); });
watch(() => props.entityId, () => { fetchPrices(); });

function avgOf(entries: PriceEntry[], key: keyof PriceEntry): number {
  const vals = entries.map((e) => e[key] as number).filter((v) => v > 0);
  if (vals.length === 0) return 0;
  return vals.reduce((sum, v) => sum + v, 0) / vals.length;
}

function avgInventoryPercent(entries: PriceEntry[]): number {
  const pcts = entries.map((e) => inventoryPercent(e)).filter((v) => v > 0);
  if (pcts.length === 0) return 0;
  return Math.round(pcts.reduce((sum, v) => sum + v, 0) / pcts.length);
}

function formatSimplePrice(val: number): string {
  if (val === 0) return "-";
  return val.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function hasData(): boolean {
  return sortedBuy.value.length > 0 || sortedSell.value.length > 0;
}
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 bg-white/5 border-b border-white/10">
      <div class="flex items-center gap-2 min-w-0">
        <span class="px-1.5 py-0.5 rounded bg-white/10 text-white/70 text-xs font-medium uppercase tracking-wide shrink-0">{{ kindLabels[entityKind] ?? entityKind }}</span>
        <h2 class="text-white font-semibold text-sm truncate">{{ entityName }}</h2>
      </div>
      <button @click="emit('close')" class="text-white/40 hover:text-white transition-colors shrink-0 ml-2">
        <IconClose class="w-4 h-4" />
      </button>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="px-4 py-8 flex justify-center">
      <LoadingSpinner text="Loading prices..." />
    </div>

    <!-- Error -->
    <div v-else-if="error" class="px-4 py-4 text-red-400 text-sm">{{ error }}</div>

    <!-- Rich data: side-by-side buy/sell columns (commodity/raw_commodity) -->
    <div v-else-if="hasData() && hasRichData" class="flex flex-1 overflow-hidden" @click="closeDropdowns()">
      <!-- Buy column -->
      <div v-if="sortedBuy.length > 0" class="flex flex-col min-w-0" :class="sortedSell.length > 0 ? 'flex-1 border-r border-white/5' : 'flex-1'">
        <div class="flex items-center justify-between px-3 py-1.5 border-b border-white/5 shrink-0">
          <span class="text-xs font-medium text-green-400/70">Buy ({{ sortedBuy.length }})</span>
          <div class="flex items-center gap-0.5">
            <div class="relative flex items-center">
              <button @click.stop="buyDropdownOpen = !buyDropdownOpen; sellDropdownOpen = false" class="px-1.5 py-0.5 rounded text-[0.625rem] text-white/30 hover:text-white/50 hover:bg-white/10 transition-colors">
                {{ sortLabel(buySortKey) }}
              </button>
              <div v-if="buyDropdownOpen" class="absolute right-0 top-full mt-1 bg-gray-900 border border-white/10 rounded-lg shadow-lg z-20 min-w-[7.5rem] py-1">
                <button v-for="opt in richSortOptions" :key="opt.key" @click.stop="selectBuySort(opt)" class="w-full text-left px-3 py-1.5 text-xs transition-colors" :class="buySortKey === opt.key ? 'text-white bg-white/10' : 'text-white/50 hover:text-white/80 hover:bg-white/5'">
                  {{ opt.label }}
                </button>
              </div>
            </div>
            <button @click.stop="buySortAsc = !buySortAsc" class="px-1 py-0.5 rounded text-[0.625rem] text-white/30 hover:text-white/50 hover:bg-white/10 transition-colors">{{ buySortAsc ? '▲' : '▼' }}</button>
          </div>
        </div>
        <div class="overflow-y-auto flex-1 p-1.5 space-y-1">
          <div
            v-for="(entry, idx) in sortedBuy"
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
            <div class="flex items-center gap-1.5 mt-1">
              <div class="flex-1 h-[2px] bg-white/5 rounded-full overflow-hidden" :title="inventoryPercent(entry) + '%'">
                <div class="h-full rounded-full transition-all" :class="inventoryBarColor(inventoryPercent(entry))" :style="{ width: inventoryPercent(entry) + '%' }"></div>
              </div>
              <span class="text-[0.625rem] text-white/25 shrink-0">{{ inventoryPercent(entry) }}%</span>
            </div>
          </div>
        </div>
      </div>
      <!-- Sell column -->
      <div v-if="sortedSell.length > 0" class="flex flex-col min-w-0" :class="sortedBuy.length > 0 ? 'flex-1' : 'flex-1'">
        <div class="flex items-center justify-between px-3 py-1.5 border-b border-white/5 shrink-0">
          <span class="text-xs font-medium text-blue-400/70">Sell ({{ sortedSell.length }})</span>
          <div class="flex items-center gap-0.5">
            <div class="relative flex items-center">
              <button @click.stop="sellDropdownOpen = !sellDropdownOpen; buyDropdownOpen = false" class="px-1.5 py-0.5 rounded text-[0.625rem] text-white/30 hover:text-white/50 hover:bg-white/10 transition-colors">
                {{ sortLabel(sellSortKey) }}
              </button>
              <div v-if="sellDropdownOpen" class="absolute right-0 top-full mt-1 bg-gray-900 border border-white/10 rounded-lg shadow-lg z-20 min-w-[7.5rem] py-1">
                <button v-for="opt in richSortOptions" :key="opt.key" @click.stop="selectSellSort(opt)" class="w-full text-left px-3 py-1.5 text-xs transition-colors" :class="sellSortKey === opt.key ? 'text-white bg-white/10' : 'text-white/50 hover:text-white/80 hover:bg-white/5'">
                  {{ opt.label }}
                </button>
              </div>
            </div>
            <button @click.stop="sellSortAsc = !sellSortAsc" class="px-1 py-0.5 rounded text-[0.625rem] text-white/30 hover:text-white/50 hover:bg-white/10 transition-colors">{{ sellSortAsc ? '▲' : '▼' }}</button>
          </div>
        </div>
        <div class="overflow-y-auto flex-1 p-1.5 space-y-1">
          <div
            v-for="(entry, idx) in sortedSell"
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
            <div class="flex items-center gap-1.5 mt-1">
              <div class="flex-1 h-[2px] bg-white/5 rounded-full overflow-hidden" :title="inventoryPercent(entry) + '%'">
                <div class="h-full rounded-full transition-all" :class="inventoryBarColor(inventoryPercent(entry))" :style="{ width: inventoryPercent(entry) + '%' }"></div>
              </div>
              <span class="text-[0.625rem] text-white/25 shrink-0">{{ inventoryPercent(entry) }}%</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Simple data: side-by-side buy/sell columns (non-commodity types) -->
    <div v-else-if="hasData()" class="flex flex-1 overflow-hidden" @click="closeDropdowns()">
      <div v-if="sortedBuy.length > 0" class="flex flex-col min-w-0" :class="sortedSell.length > 0 ? 'flex-1 border-r border-white/5' : 'flex-1'">
        <div class="flex items-center justify-between px-3 py-1.5 border-b border-white/5 shrink-0">
          <span class="text-xs font-medium text-green-400/70">Buy ({{ sortedBuy.length }})</span>
          <div class="flex items-center gap-0.5">
            <div class="relative flex items-center">
              <button @click.stop="buyDropdownOpen = !buyDropdownOpen; sellDropdownOpen = false" class="px-1.5 py-0.5 rounded text-[0.625rem] text-white/30 hover:text-white/50 hover:bg-white/10 transition-colors">
                {{ sortLabel(buySortKey) }}
              </button>
              <div v-if="buyDropdownOpen" class="absolute right-0 top-full mt-1 bg-gray-900 border border-white/10 rounded-lg shadow-lg z-20 min-w-[7.5rem] py-1">
                <button v-for="opt in simpleSortOptions" :key="opt.key" @click.stop="selectBuySort(opt)" class="w-full text-left px-3 py-1.5 text-xs transition-colors" :class="buySortKey === opt.key ? 'text-white bg-white/10' : 'text-white/50 hover:text-white/80 hover:bg-white/5'">
                  {{ opt.label }}
                </button>
              </div>
            </div>
            <button @click.stop="buySortAsc = !buySortAsc" class="px-1 py-0.5 rounded text-[0.625rem] text-white/30 hover:text-white/50 hover:bg-white/10 transition-colors">{{ buySortAsc ? '▲' : '▼' }}</button>
          </div>
        </div>
        <div class="overflow-y-auto flex-1 p-1.5 space-y-1">
          <div
            v-for="(entry, idx) in sortedBuy"
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
      <div v-if="sortedSell.length > 0" class="flex flex-col min-w-0" :class="sortedBuy.length > 0 ? 'flex-1' : 'flex-1'">
        <div class="flex items-center justify-between px-3 py-1.5 border-b border-white/5 shrink-0">
          <span class="text-xs font-medium text-blue-400/70">Sell ({{ sortedSell.length }})</span>
          <div class="flex items-center gap-0.5">
            <div class="relative flex items-center">
              <button @click.stop="sellDropdownOpen = !sellDropdownOpen; buyDropdownOpen = false" class="px-1.5 py-0.5 rounded text-[0.625rem] text-white/30 hover:text-white/50 hover:bg-white/10 transition-colors">
                {{ sortLabel(sellSortKey) }}
              </button>
              <div v-if="sellDropdownOpen" class="absolute right-0 top-full mt-1 bg-gray-900 border border-white/10 rounded-lg shadow-lg z-20 min-w-[7.5rem] py-1">
                <button v-for="opt in simpleSortOptions" :key="opt.key" @click.stop="selectSellSort(opt)" class="w-full text-left px-3 py-1.5 text-xs transition-colors" :class="sellSortKey === opt.key ? 'text-white bg-white/10' : 'text-white/50 hover:text-white/80 hover:bg-white/5'">
                  {{ opt.label }}
                </button>
              </div>
            </div>
            <button @click.stop="sellSortAsc = !sellSortAsc" class="px-1 py-0.5 rounded text-[0.625rem] text-white/30 hover:text-white/50 hover:bg-white/10 transition-colors">{{ sellSortAsc ? '▲' : '▼' }}</button>
          </div>
        </div>
        <div class="overflow-y-auto flex-1 p-1.5 space-y-1">
          <div
            v-for="(entry, idx) in sortedSell"
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

    <!-- Empty state -->
    <div v-else-if="!loading" class="px-4 py-8 text-center text-white/40 text-sm">
      No price data available.
    </div>

    <!-- Summary footer -->
    <div v-if="!loading && !error && hasData() && hasRichData" class="flex border-t border-white/10 text-[0.6875rem] shrink-0">
      <div v-if="sortedBuy.length > 1" class="flex items-center gap-3 px-3 py-1.5 bg-white/[0.03]" :class="sortedSell.length > 0 ? 'flex-1 border-r border-white/5' : 'flex-1'">
        <span class="text-white/30">{{ sortedBuy.length }}loc</span>
        <span class="text-orange-300/60">ø {{ formatPrice(avgOf(sortedBuy, "price_last")) }}</span>
        <div class="flex items-center gap-1 flex-1">
          <div class="flex-1 h-[2px] bg-white/5 rounded-full overflow-hidden">
            <div class="h-full rounded-full bg-green-400/40" :style="{ width: avgInventoryPercent(sortedBuy) + '%' }"></div>
          </div>
          <span class="text-white/20">{{ avgInventoryPercent(sortedBuy) }}%</span>
        </div>
      </div>
      <div v-if="sortedSell.length > 1" class="flex items-center gap-3 px-3 py-1.5 bg-white/[0.03] flex-1">
        <span class="text-white/30">{{ sortedSell.length }}loc</span>
        <span class="text-orange-300/60">ø {{ formatPrice(avgOf(sortedSell, "price_last")) }}</span>
        <div class="flex items-center gap-1 flex-1">
          <div class="flex-1 h-[2px] bg-white/5 rounded-full overflow-hidden">
            <div class="h-full rounded-full bg-blue-400/40" :style="{ width: avgInventoryPercent(sortedSell) + '%' }"></div>
          </div>
          <span class="text-white/20">{{ avgInventoryPercent(sortedSell) }}%</span>
        </div>
      </div>
    </div>
  </div>
</template>
