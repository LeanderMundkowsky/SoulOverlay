<script setup lang="ts">
defineProps<{
  modelValue: boolean;
  label: string;
  description?: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: boolean): void;
}>();
</script>

<template>
  <label class="flex items-center justify-between gap-3 cursor-pointer select-none">
    <div>
      <p class="text-white/60 text-xs font-medium uppercase tracking-wider">{{ label }}</p>
      <p v-if="description" class="text-white/30 text-xs mt-0.5">{{ description }}</p>
    </div>
    <button
      type="button"
      role="switch"
      :aria-checked="modelValue"
      @click="emit('update:modelValue', !modelValue)"
      class="relative inline-flex h-5 w-9 flex-shrink-0 rounded-full border-2 border-transparent transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-blue-500/50"
      :class="modelValue ? 'bg-blue-600' : 'bg-white/10'"
    >
      <span
        class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow transition-transform duration-200"
        :class="modelValue ? 'translate-x-4' : 'translate-x-0'"
      ></span>
    </button>
  </label>
</template>
