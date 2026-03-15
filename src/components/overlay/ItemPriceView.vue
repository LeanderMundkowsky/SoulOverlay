<script setup lang="ts">
import PriceList from "@/components/overlay/PriceList.vue";
import { itemSortOptions } from "@/utils/sorting";
import { locationPath } from "@/utils/priceFormatters";
import type { PriceEntry } from "@/bindings";

defineProps<{
  buyEntries: PriceEntry[];
  sellEntries: PriceEntry[];
}>();

function label(entry: PriceEntry): string {
  return entry.terminal;
}

function subLabel(entry: PriceEntry): string {
  return locationPath(entry);
}
</script>

<template>
  <div class="flex flex-col flex-1 overflow-hidden">
    <!-- Future: item info section -->
    <PriceList
      :buy-entries="buyEntries"
      :sell-entries="sellEntries"
      :sort-options="itemSortOptions"
      :label-fn="label"
      :sub-label-fn="subLabel"
    />
  </div>
</template>
