<script setup lang="ts">
import PriceList from "@/components/overlay/PriceList.vue";
import { terminalSortOptions } from "@/utils/sorting";
import type { PriceEntry } from "@/bindings";

defineProps<{
  buyEntries: PriceEntry[];
  sellEntries: PriceEntry[];
}>();

const priceTypeLabels: Record<string, string> = {
  commodity: "Commodity",
  raw_commodity: "Raw Commodity",
  item: "Item",
  fuel: "Fuel",
  vehicle_purchase: "Vehicle (Buy)",
  vehicle_rental: "Vehicle (Rent)",
};

function label(entry: PriceEntry): string {
  return entry.entity_name;
}

function subLabel(entry: PriceEntry): string {
  if (entry.category) return entry.category;
  return priceTypeLabels[entry.price_type] ?? entry.price_type;
}
</script>

<template>
  <div class="flex flex-col flex-1 overflow-hidden">
    <!-- Future: terminal info section (location, faction, etc.) -->
    <PriceList
      :buy-entries="buyEntries"
      :sell-entries="sellEntries"
      :sort-options="terminalSortOptions"
      :label-fn="label"
      :sub-label-fn="subLabel"
    />
  </div>
</template>
