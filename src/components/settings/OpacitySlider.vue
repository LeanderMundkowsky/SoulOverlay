<script setup lang="ts">
defineProps<{
  modelValue: number;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: number): void;
}>();
</script>

<template>
  <div>
    <label class="block text-white/60 text-xs font-medium uppercase tracking-wider mb-1.5">
      Background Opacity: {{ Math.round(modelValue * 100) }}%
    </label>
    <input
      :value="modelValue"
      @input="emit('update:modelValue', Number(($event.target as HTMLInputElement).value))"
      type="range"
      min="0"
      max="1"
      step="0.05"
      class="w-full accent-blue-500"
    />
    <p class="text-white/30 text-xs mt-1">Controls background dimming only — UI is always fully visible</p>
  </div>
</template>
