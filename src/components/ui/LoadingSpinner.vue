<script setup lang="ts">
withDefaults(defineProps<{
  text?: string;
  size?: "sm" | "md";
}>(), {
  text: "",
  size: "md",
});
</script>

<template>
  <div class="flex flex-col items-center gap-2 text-white/40 text-sm">
    <div
      class="border-2 border-blue-400/30 border-t-blue-400 rounded-full animate-spin"
      :class="size === 'sm' ? 'w-4 h-4' : 'w-6 h-6'"
    ></div>
    <span v-if="text">{{ text }}</span>
  </div>
</template>
