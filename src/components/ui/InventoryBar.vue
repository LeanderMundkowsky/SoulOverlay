<script setup lang="ts">
import { inventoryBarColor } from "@/utils/priceFormatters";

defineProps<{
  percent: number;
}>();
</script>

<template>
  <div class="flex items-center gap-1.5">
    <div class="flex-1 h-[2px] bg-white/5 rounded-full overflow-hidden" :title="percent + '%'">
      <div
        class="h-full rounded-full transition-all"
        :class="inventoryBarColor(percent)"
        :style="{ width: percent + '%' }"
      ></div>
    </div>
    <span class="text-[0.625rem] text-white/25 shrink-0">{{ percent }}%</span>
  </div>
</template>
