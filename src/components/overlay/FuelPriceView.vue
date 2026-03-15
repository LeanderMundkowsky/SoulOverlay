<script setup lang="ts">
import { computed } from "vue";
import PriceList from "@/components/overlay/PriceList.vue";
import { fuelSortOptions } from "@/utils/sorting";
import { locationPath } from "@/utils/priceFormatters";
import type { PriceEntry } from "@/bindings";

const props = defineProps<{
  buyEntries: PriceEntry[];
  sellEntries: PriceEntry[];
}>();

/** Group fuel entries by location, keeping the lowest price per location. */
function groupByLocation(entries: PriceEntry[]): PriceEntry[] {
  const byLocation = new Map<string, PriceEntry>();
  for (const entry of entries) {
    const key = entry.orbit || entry.location || entry.terminal;
    const existing = byLocation.get(key);
    if (!existing || entry.buy_price < existing.buy_price) {
      byLocation.set(key, entry);
    }
  }
  return Array.from(byLocation.values());
}

const groupedBuy = computed(() => groupByLocation(props.buyEntries));
const groupedSell = computed(() => groupByLocation(props.sellEntries));

function label(entry: PriceEntry): string {
  return entry.orbit || entry.terminal;
}

function subLabel(entry: PriceEntry): string {
  return locationPath(entry);
}
</script>

<template>
  <div class="flex flex-col flex-1 overflow-hidden">
    <!-- Future: fuel info section (refueling summary, location overview) -->
    <PriceList
      :buy-entries="groupedBuy"
      :sell-entries="groupedSell"
      :sort-options="fuelSortOptions"
      :label-fn="label"
      :sub-label-fn="subLabel"
      buy-label="Refuel"
      buy-color="text-amber-400"
    />
  </div>
</template>
