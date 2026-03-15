<script setup lang="ts">
import PriceList from "@/components/overlay/PriceList.vue";
import { itemSortOptions } from "@/utils/sorting";
import type { PriceEntry } from "@/bindings";

defineProps<{
  buyEntries: PriceEntry[];
  sellEntries: PriceEntry[];
}>();

function label(entry: PriceEntry): string {
  return entry.terminal;
}

function subLabel(entry: PriceEntry): string {
  if (entry.location && entry.location !== "Unknown") return entry.location;
  if (entry.orbit) return entry.orbit;
  if (entry.system) return entry.system;
  return "";
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
