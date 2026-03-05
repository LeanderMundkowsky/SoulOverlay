<script setup lang="ts">
import IconClose from "@/components/icons/IconClose.vue";
import InventoryBar from "@/components/ui/InventoryBar.vue";
import { useWatchlistStore } from "@/stores/watchlist";
import { useDragDrop } from "@/composables/useDragDrop";
import { commands } from "@/bindings";
import { formatPrice, formatScu, inventoryPercent, shortTerminal } from "@/utils/priceFormatters";
import type { WatchEntry, PriceEntry, ApiResponse, Result } from "@/bindings";
import { ref, watch, onMounted } from "vue";

const watchlistStore = useWatchlistStore();
const { payload, dragging } = useDragDrop();

const emit = defineEmits<{
  select: [entry: WatchEntry];
}>();

const dragOver = ref(false);
const livePrices = ref<Map<string, PriceEntry[]>>(new Map());

type PriceCommand = (id: string) => Promise<Result<ApiResponse<PriceEntry[]>, string>>;
const priceCommandMap: Record<string, PriceCommand> = {
  commodity: (id) => commands.apiCommodityPrices(id),
  raw_commodity: (id) => commands.apiRawCommodityPrices(id),
  item: (id) => commands.apiItemPrices(id),
  vehicle: (id) => commands.apiVehiclePurchasePrices(id),
  "ground vehicle": (id) => commands.apiVehiclePurchasePrices(id),
  vehicle_rental: (id) => commands.apiVehicleRentalPrices(id),
  fuel: (id) => commands.apiFuelPrices(id),
  location: (id) => commands.apiFuelPrices(id),
};

function entityKey(entityId: string, entityKind: string): string {
  return `${entityId}:${entityKind}`;
}

function findLiveEntry(w: WatchEntry): PriceEntry | undefined {
  const prices = livePrices.value.get(entityKey(w.entity_id, w.entity_kind));
  if (!prices) return undefined;
  return prices.find((p) => p.terminal_id === w.terminal_id);
}

function livePrice(w: WatchEntry): number | undefined {
  const entry = findLiveEntry(w);
  if (!entry) return undefined;
  return w.price_type === "buy" ? (entry.price_last ?? entry.buy_price) : (entry.price_last ?? entry.sell_price);
}

function liveInventory(w: WatchEntry): number {
  const entry = findLiveEntry(w);
  if (!entry) return 0;
  return inventoryPercent(entry);
}

async function fetchPricesForEntity(entityId: string, entityKind: string) {
  const command = priceCommandMap[entityKind];
  if (!command) return;
  try {
    const result = await command(entityId);
    if (result.status === "error") return;
    const resp = result.data;
    if (resp.ok && resp.data) {
      livePrices.value.set(entityKey(entityId, entityKind), resp.data);
    }
  } catch {
    // Silently skip
  }
}

async function refreshLivePrices() {
  const seen = new Set<string>();
  const fetches: Promise<void>[] = [];
  for (const w of watchlistStore.entries) {
    const key = entityKey(w.entity_id, w.entity_kind);
    if (seen.has(key)) continue;
    seen.add(key);
    fetches.push(fetchPricesForEntity(w.entity_id, w.entity_kind));
  }
  await Promise.all(fetches);
}

onMounted(async () => {
  if (watchlistStore.entries.length > 0) await refreshLivePrices();
});

watch(() => watchlistStore.entries.length, async (len) => {
  if (len > 0) await refreshLivePrices();
});

function onPointerEnter() {
  if (dragging.value && payload.value?.type === "price") dragOver.value = true;
}

function onPointerLeave() {
  dragOver.value = false;
}

function onPointerUp() {
  if (!dragging.value || !payload.value) return;
  if (payload.value.type !== "price") return;
  const p = payload.value.data;
  if (!watchlistStore.isWatched(p.entityId, p.terminalId, p.priceType)) {
    watchlistStore.addEntry(p);
  }
  dragOver.value = false;
}

async function removeEntry(w: WatchEntry) {
  await watchlistStore.removeEntry(w.entity_id, w.terminal_id, w.price_type);
}
</script>

<template>
  <div
    class="w-full flex flex-col bg-[#1a1d24] border rounded-xl overflow-hidden transition-colors"
    :class="dragOver ? 'border-blue-500/50 bg-blue-500/5' : 'border-white/10'"
    @pointerenter="onPointerEnter"
    @pointerleave="onPointerLeave"
    @pointerup="onPointerUp"
  >
    <!-- Header -->
    <div class="px-3 py-2 border-b border-white/10">
      <span class="text-xs font-semibold text-white/50 uppercase tracking-widest">Watch List</span>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <template v-if="watchlistStore.entries.length === 0">
        <div class="px-3 py-4 text-xs text-white/30 text-center">
          No watched prices yet
        </div>
      </template>

      <div
        v-for="w in watchlistStore.entries"
        :key="`${w.entity_id}-${w.terminal_id}-${w.price_type}`"
        class="group flex items-center gap-2 px-3 py-1.5 cursor-pointer hover:bg-white/5 transition-colors"
        @click="emit('select', w)"
      >
        <!-- Buy/Sell badge -->
        <span
          class="shrink-0 px-1 py-0.5 rounded text-[0.5625rem] font-bold uppercase"
          :class="w.price_type === 'buy' ? 'bg-green-400/15 text-green-400/70' : 'bg-blue-400/15 text-blue-400/70'"
        >{{ w.price_type === "buy" ? "B" : "S" }}</span>

        <!-- Entity + location info -->
        <div class="flex-1 min-w-0">
          <div class="text-xs text-white/70 truncate">{{ w.entity_name }}</div>
          <div class="text-[0.625rem] text-white/30 truncate">{{ shortTerminal(w.terminal_name) }}</div>
        </div>

        <!-- Live price + inventory -->
        <div class="shrink-0 text-right">
          <div class="text-xs text-orange-300/80 font-medium">
            {{ livePrice(w) != null ? formatPrice(livePrice(w)) : '—' }}
          </div>
          <div class="flex items-center gap-1">
            <template v-if="findLiveEntry(w)">
              <span class="text-[0.5625rem] text-white/25">
                {{ formatScu(findLiveEntry(w)!.scu_last) }} / {{ formatScu(findLiveEntry(w)!.scu_max) }}
              </span>
            </template>
          </div>
        </div>

        <!-- Inventory bar -->
        <div class="shrink-0 w-10">
          <InventoryBar :percent="liveInventory(w)" />
        </div>

        <!-- Remove button -->
        <button
          class="shrink-0 opacity-0 group-hover:opacity-100 transition-opacity p-0.5 rounded hover:bg-white/10"
          title="Remove from watch list"
          @click.stop="removeEntry(w)"
        >
          <IconClose class="w-3 h-3 text-white/30 hover:text-red-400" />
        </button>
      </div>
    </div>
  </div>
</template>
