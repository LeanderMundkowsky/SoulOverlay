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

const sortKey = ref<keyof PriceEntry>("price_last");
const sortAsc = ref(true);
const sortedPrices = ref<PriceEntry[]>([]);

const showRentColumn = ref(false);
const hasRichData = ref(false);

const buyLocations = ref<PriceEntry[]>([]);
const sellLocations = ref<PriceEntry[]>([]);
const activeTab = ref<"buy" | "sell">("buy");

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

watch(
  [prices, sortKey, sortAsc],
  () => {
    const all = [...prices.value];

    // Separate buy/sell locations
    buyLocations.value = all.filter((p) => p.buy_price > 0);
    sellLocations.value = all.filter((p) => p.sell_price > 0);

    // Auto-select tab based on which has data
    if (buyLocations.value.length === 0 && sellLocations.value.length > 0) {
      activeTab.value = "sell";
    } else if (buyLocations.value.length > 0 && sellLocations.value.length === 0) {
      activeTab.value = "buy";
    }

    // Sort the active tab's data
    const source = activeTab.value === "buy" ? buyLocations.value : sellLocations.value;
    const sorted = [...source];
    sorted.sort((a, b) => {
      const aVal = a[sortKey.value] ?? 0;
      const bVal = b[sortKey.value] ?? 0;
      if (typeof aVal === "number" && typeof bVal === "number") {
        return sortAsc.value ? aVal - bVal : bVal - aVal;
      }
      return sortAsc.value
        ? String(aVal).localeCompare(String(bVal))
        : String(bVal).localeCompare(String(aVal));
    });
    sortedPrices.value = sorted;

    showRentColumn.value = sorted.some((p) => p.rent_price > 0);
    hasRichData.value = props.entityKind === "commodity" || props.entityKind === "raw_commodity";
  },
  { immediate: true }
);

watch(activeTab, () => {
  // Re-trigger sort when tab changes
  const source = activeTab.value === "buy" ? buyLocations.value : sellLocations.value;
  const sorted = [...source];
  sorted.sort((a, b) => {
    const aVal = a[sortKey.value] ?? 0;
    const bVal = b[sortKey.value] ?? 0;
    if (typeof aVal === "number" && typeof bVal === "number") {
      return sortAsc.value ? aVal - bVal : bVal - aVal;
    }
    return sortAsc.value
      ? String(aVal).localeCompare(String(bVal))
      : String(bVal).localeCompare(String(aVal));
  });
  sortedPrices.value = sorted;
});

function toggleSort(key: keyof PriceEntry) {
  if (sortKey.value === key) {
    sortAsc.value = !sortAsc.value;
  } else {
    sortKey.value = key;
    sortAsc.value = key === "terminal" || key === "orbit";
  }
}

function sortIndicator(key: keyof PriceEntry): string {
  if (sortKey.value !== key) return "";
  return sortAsc.value ? " ▲" : " ▼";
}

function formatScu(val: number): string {
  if (val === 0) return "-";
  if (val >= 1000) return (val / 1000).toFixed(1).replace(/\.0$/, "") + "K";
  return val.toLocaleString("en-US", { maximumFractionDigits: 0 });
}

function formatPrice(val: number): string {
  if (val === 0) return "-";
  return val.toLocaleString("en-US", { minimumFractionDigits: 0, maximumFractionDigits: 0 });
}

function inventoryPercent(entry: PriceEntry): number {
  if (entry.scu_max <= 0) return 0;
  return Math.round((entry.scu_avg / entry.scu_max) * 100);
}

function inventoryBarColor(pct: number): string {
  if (pct >= 80) return "bg-green-400";
  if (pct >= 40) return "bg-yellow-400";
  if (pct > 0) return "bg-blue-400";
  return "bg-white/20";
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

function shortSystem(system: string): string {
  const map: Record<string, string> = {
    Stanton: "ST",
    Pyro: "PY",
    Nyx: "NY",
  };
  return map[system] ?? system.substring(0, 3).toUpperCase();
}

function shortFaction(faction: string): string {
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

function avgInventoryPercent(): number {
  const pcts = sortedPrices.value.map((e) => inventoryPercent(e)).filter((v) => v > 0);
  if (pcts.length === 0) return 0;
  return Math.round(pcts.reduce((sum, v) => sum + v, 0) / pcts.length);
}

function formatSimplePrice(val: number): string {
  if (val === 0) return "-";
  return val.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
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

    <!-- Tab bar (buy/sell) -->
    <div v-if="!loading && !error && (buyLocations.length > 0 || sellLocations.length > 0)" class="flex border-b border-white/10 bg-white/[0.02]">
      <button
        v-if="buyLocations.length > 0"
        @click="activeTab = 'buy'"
        class="px-4 py-2 text-xs font-medium transition-colors"
        :class="activeTab === 'buy' ? 'text-green-400 border-b-2 border-green-400' : 'text-white/40 hover:text-white/60'"
      >
        Buy ({{ buyLocations.length }})
      </button>
      <button
        v-if="sellLocations.length > 0"
        @click="activeTab = 'sell'"
        class="px-4 py-2 text-xs font-medium transition-colors"
        :class="activeTab === 'sell' ? 'text-blue-400 border-b-2 border-blue-400' : 'text-white/40 hover:text-white/60'"
      >
        Sell ({{ sellLocations.length }})
      </button>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="px-4 py-8 flex justify-center">
      <LoadingSpinner text="Loading prices..." />
    </div>

    <!-- Error -->
    <div v-else-if="error" class="px-4 py-4 text-red-400 text-sm">{{ error }}</div>

    <!-- Rich data table (commodity/raw_commodity) -->
    <div v-else-if="sortedPrices.length > 0 && hasRichData" class="overflow-x-auto overflow-y-auto flex-1">
      <table class="w-full text-xs whitespace-nowrap">
        <thead>
          <tr class="text-white/30 text-xs uppercase tracking-wider sticky top-0 bg-[#1a1d24] z-10">
            <!-- Location group -->
            <th colspan="2" class="text-left px-2 pt-2 pb-0.5 text-white/50 border-b border-white/5">Location</th>
            <!-- Region group -->
            <th colspan="2" class="text-left px-2 pt-2 pb-0.5 text-white/50 border-b border-white/5 border-l border-white/5">Region</th>
            <!-- SCU group -->
            <th colspan="4" class="text-center px-2 pt-2 pb-0.5 text-white/50 border-b border-white/5 border-l border-white/5">SCU</th>
            <!-- Inventory -->
            <th class="text-center px-2 pt-2 pb-0.5 text-white/50 border-b border-white/5 border-l border-white/5">Inv</th>
            <!-- UEC group -->
            <th colspan="4" class="text-center px-2 pt-2 pb-0.5 text-white/50 border-b border-white/5 border-l border-white/5">UEC</th>
            <!-- CS -->
            <th class="text-center px-2 pt-2 pb-0.5 text-white/50 border-b border-white/5 border-l border-white/5">CS</th>
            <!-- Age -->
            <th class="text-center px-2 pt-2 pb-0.5 text-white/50 border-b border-white/5 border-l border-white/5">⏱</th>
          </tr>
          <tr class="text-white/40 text-xs uppercase tracking-wider sticky top-[26px] bg-[#1a1d24] z-10 border-b border-white/10">
            <th @click="toggleSort('terminal')" class="text-left px-2 py-1.5 cursor-pointer hover:text-white/70">Name{{ sortIndicator("terminal") }}</th>
            <th @click="toggleSort('orbit')" class="text-left px-2 py-1.5 cursor-pointer hover:text-white/70">Orbit{{ sortIndicator("orbit") }}</th>
            <th class="text-left px-2 py-1.5 border-l border-white/5">Sys</th>
            <th class="text-left px-2 py-1.5">Fac</th>
            <th @click="toggleSort('scu_last')" class="text-right px-2 py-1.5 cursor-pointer hover:text-white/70 border-l border-white/5">Last{{ sortIndicator("scu_last") }}</th>
            <th @click="toggleSort('scu_avg')" class="text-right px-2 py-1.5 cursor-pointer hover:text-white/70">Avg{{ sortIndicator("scu_avg") }}</th>
            <th @click="toggleSort('scu_min')" class="text-right px-2 py-1.5 cursor-pointer hover:text-white/70">Min{{ sortIndicator("scu_min") }}</th>
            <th @click="toggleSort('scu_max')" class="text-right px-2 py-1.5 cursor-pointer hover:text-white/70">Max{{ sortIndicator("scu_max") }}</th>
            <th class="text-center px-2 py-1.5 border-l border-white/5">Avg</th>
            <th @click="toggleSort('price_last')" class="text-right px-2 py-1.5 cursor-pointer hover:text-white/70 border-l border-white/5">Last{{ sortIndicator("price_last") }}</th>
            <th @click="toggleSort('price_avg')" class="text-right px-2 py-1.5 cursor-pointer hover:text-white/70">Avg{{ sortIndicator("price_avg") }}</th>
            <th @click="toggleSort('price_min')" class="text-right px-2 py-1.5 cursor-pointer hover:text-white/70">Min{{ sortIndicator("price_min") }}</th>
            <th @click="toggleSort('price_max')" class="text-right px-2 py-1.5 cursor-pointer hover:text-white/70">Max{{ sortIndicator("price_max") }}</th>
            <th class="text-center px-2 py-1.5 border-l border-white/5">SCU</th>
            <th @click="toggleSort('date_updated')" class="text-center px-2 py-1.5 cursor-pointer hover:text-white/70 border-l border-white/5">{{ sortIndicator("date_updated") || "" }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(entry, idx) in sortedPrices"
            :key="idx"
            class="border-t border-white/5 hover:bg-white/5 transition-colors"
          >
            <!-- Name -->
            <td class="px-2 py-1.5 text-white/80 font-medium max-w-[160px] truncate" :title="entry.terminal">{{ shortTerminal(entry.terminal) }}</td>
            <!-- Orbit -->
            <td class="px-2 py-1.5 text-white/50 font-medium">{{ entry.orbit }}</td>
            <!-- System -->
            <td class="px-2 py-1.5 text-white/40 border-l border-white/5">{{ shortSystem(entry.system) }}</td>
            <!-- Faction -->
            <td class="px-2 py-1.5 text-white/40">{{ shortFaction(entry.faction) }}</td>
            <!-- SCU Last -->
            <td class="px-2 py-1.5 text-right text-orange-300 border-l border-white/5">{{ formatScu(entry.scu_last) }}</td>
            <!-- SCU Avg -->
            <td class="px-2 py-1.5 text-right text-white/50">{{ formatScu(entry.scu_avg) }}</td>
            <!-- SCU Min -->
            <td class="px-2 py-1.5 text-right text-white/50">{{ formatScu(entry.scu_min) }}</td>
            <!-- SCU Max -->
            <td class="px-2 py-1.5 text-right text-white/50">{{ formatScu(entry.scu_max) }}</td>
            <!-- Inventory bar -->
            <td class="px-2 py-1.5 border-l border-white/5">
              <div class="w-16 h-3 bg-white/10 rounded-sm overflow-hidden" :title="inventoryPercent(entry) + '%'">
                <div
                  class="h-full rounded-sm transition-all"
                  :class="inventoryBarColor(inventoryPercent(entry))"
                  :style="{ width: inventoryPercent(entry) + '%' }"
                ></div>
              </div>
            </td>
            <!-- UEC Last -->
            <td class="px-2 py-1.5 text-right text-orange-300 border-l border-white/5">{{ formatPrice(entry.price_last) }}</td>
            <!-- UEC Avg -->
            <td class="px-2 py-1.5 text-right text-white/50">{{ formatPrice(entry.price_avg) }}</td>
            <!-- UEC Min -->
            <td class="px-2 py-1.5 text-right text-white/50">{{ formatPrice(entry.price_min) }}</td>
            <!-- UEC Max -->
            <td class="px-2 py-1.5 text-right text-white/50">{{ formatPrice(entry.price_max) }}</td>
            <!-- Container sizes -->
            <td class="px-2 py-1.5 text-center text-white/40 border-l border-white/5">{{ entry.container_sizes || "-" }}</td>
            <!-- Age -->
            <td class="px-2 py-1.5 text-center text-white/30 border-l border-white/5">{{ relativeAge(entry.date_updated) }}</td>
          </tr>
        </tbody>
        <!-- Averages footer -->
        <tfoot v-if="sortedPrices.length > 1">
          <tr class="border-t border-white/10 bg-white/[0.03] text-white/50 font-medium">
            <td class="px-2 py-1.5" colspan="2">{{ sortedPrices.length }} location{{ sortedPrices.length !== 1 ? "s" : "" }}</td>
            <td class="px-2 py-1.5 border-l border-white/5" colspan="2">Averages</td>
            <td class="px-2 py-1.5 text-right text-orange-300/70 border-l border-white/5">{{ formatScu(avgOf(sortedPrices, "scu_last")) }}</td>
            <td class="px-2 py-1.5 text-right text-white/40">{{ formatScu(avgOf(sortedPrices, "scu_avg")) }}</td>
            <td class="px-2 py-1.5 text-right text-white/40">{{ formatScu(avgOf(sortedPrices, "scu_min")) }}</td>
            <td class="px-2 py-1.5 text-right text-white/40">{{ formatScu(avgOf(sortedPrices, "scu_max")) }}</td>
            <td class="px-2 py-1.5 border-l border-white/5">
              <div class="w-16 h-3 bg-white/10 rounded-sm overflow-hidden">
                <div
                  class="h-full rounded-sm bg-green-400/60"
                  :style="{ width: avgInventoryPercent() + '%' }"
                ></div>
              </div>
            </td>
            <td class="px-2 py-1.5 text-right text-orange-300/70 border-l border-white/5">{{ formatPrice(avgOf(sortedPrices, "price_last")) }}</td>
            <td class="px-2 py-1.5 text-right text-white/40">{{ formatPrice(avgOf(sortedPrices, "price_avg")) }}</td>
            <td class="px-2 py-1.5 text-right text-white/40">{{ formatPrice(avgOf(sortedPrices, "price_min")) }}</td>
            <td class="px-2 py-1.5 text-right text-white/40">{{ formatPrice(avgOf(sortedPrices, "price_max")) }}</td>
            <td class="px-2 py-1.5 border-l border-white/5"></td>
            <td class="px-2 py-1.5 border-l border-white/5"></td>
          </tr>
        </tfoot>
      </table>
    </div>

    <!-- Simple table fallback (non-commodity types) -->
    <div v-else-if="sortedPrices.length > 0" class="overflow-x-auto overflow-y-auto flex-1">
      <table class="w-full text-sm">
        <thead>
          <tr class="text-white/50 text-xs uppercase tracking-wider sticky top-0 bg-[#1a1d24]">
            <th @click="toggleSort('location')" class="text-left px-4 py-2 cursor-pointer hover:text-white/80 transition-colors">Location{{ sortIndicator("location") }}</th>
            <th @click="toggleSort('terminal')" class="text-left px-4 py-2 cursor-pointer hover:text-white/80 transition-colors">Terminal{{ sortIndicator("terminal") }}</th>
            <th @click="toggleSort('buy_price')" class="text-right px-4 py-2 cursor-pointer hover:text-white/80 transition-colors">Buy{{ sortIndicator("buy_price") }}</th>
            <th @click="toggleSort('sell_price')" class="text-right px-4 py-2 cursor-pointer hover:text-white/80 transition-colors">Sell{{ sortIndicator("sell_price") }}</th>
            <th v-if="showRentColumn" @click="toggleSort('rent_price')" class="text-right px-4 py-2 cursor-pointer hover:text-white/80 transition-colors">Rent{{ sortIndicator("rent_price") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(entry, idx) in sortedPrices"
            :key="idx"
            class="border-t border-white/5 hover:bg-white/5 transition-colors"
          >
            <td class="px-4 py-2 text-white/80">{{ entry.location }}</td>
            <td class="px-4 py-2 text-white/60">{{ entry.terminal }}</td>
            <td class="px-4 py-2 text-right text-green-400">{{ formatSimplePrice(entry.buy_price) }}</td>
            <td class="px-4 py-2 text-right text-blue-400">{{ formatSimplePrice(entry.sell_price) }}</td>
            <td v-if="showRentColumn" class="px-4 py-2 text-right text-yellow-400">{{ formatSimplePrice(entry.rent_price) }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Empty state -->
    <div v-else class="px-4 py-8 text-center text-white/40 text-sm">
      No price data available.
    </div>
  </div>
</template>
