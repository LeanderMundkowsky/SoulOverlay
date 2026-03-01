<script setup lang="ts">
defineProps<{
  label: string;
  hint?: string;
}>();
</script>

<template>
  <div>
    <label class="block text-white/60 text-xs font-medium uppercase tracking-wider mb-1.5">
      {{ label }}
    </label>
    <slot />
    <p v-if="hint" class="text-white/30 text-xs mt-1">{{ hint }}</p>
  </div>
</template>
