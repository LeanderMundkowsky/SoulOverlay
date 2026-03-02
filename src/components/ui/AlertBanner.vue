<script setup lang="ts">
import IconInfoCircle from "@/components/icons/IconInfoCircle.vue";

defineProps<{
  variant: "warning" | "error" | "success" | "info";
  title?: string;
  message: string;
}>();
</script>

<template>
  <div
    class="flex items-start gap-3 rounded-xl px-4 py-3 text-sm border"
    :class="{
      'bg-yellow-900 border-yellow-500/60 text-yellow-100': variant === 'warning',
      'bg-red-950 border-red-500/40 text-red-200': variant === 'error',
      'bg-green-950 border-green-500/40 text-green-200': variant === 'success',
      'bg-blue-950 border-blue-500/40 text-blue-200': variant === 'info',
    }"
  >
    <IconInfoCircle
      class="w-4 h-4 mt-0.5 flex-shrink-0"
      :class="{
        'text-yellow-400': variant === 'warning',
        'text-red-400': variant === 'error',
        'text-green-400': variant === 'success',
        'text-blue-400': variant === 'info',
      }"
    />
    <div>
      <p v-if="title" class="font-semibold">{{ title }}</p>
      <p :class="title ? 'text-xs mt-0.5' : ''">{{ message }}</p>
    </div>
  </div>
</template>
