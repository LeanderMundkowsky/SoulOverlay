<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";

export interface MenuItem {
  label: string;
  icon?: string;
  danger?: boolean;
  separator?: false;
  action: () => void;
}
export interface MenuSeparator {
  separator: true;
}

defineProps<{
  x: number;
  y: number;
  items: (MenuItem | MenuSeparator)[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

function onMouseDown(e: MouseEvent) {
  // close if click is outside the menu
  const target = e.target as HTMLElement;
  if (!target.closest("[data-context-menu]")) emit("close");
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") emit("close");
}

onMounted(() => {
  // defer so the triggering mousedown doesn't immediately close it
  setTimeout(() => {
    window.addEventListener("mousedown", onMouseDown);
    window.addEventListener("keydown", onKeyDown);
  }, 0);
});

onUnmounted(() => {
  window.removeEventListener("mousedown", onMouseDown);
  window.removeEventListener("keydown", onKeyDown);
});

function isSeparator(item: MenuItem | MenuSeparator): item is MenuSeparator {
  return (item as MenuSeparator).separator === true;
}
</script>

<template>
  <Teleport to="body">
    <div
      data-context-menu
      class="fixed z-50 min-w-[160px] bg-[#1e2130] border border-white/10 rounded-xl shadow-xl py-1 text-sm overflow-hidden"
      :style="{ left: x + 'px', top: y + 'px' }"
    >
      <template v-for="(item, i) in items" :key="i">
        <div v-if="isSeparator(item)" class="my-1 border-t border-white/10" />
        <button
          v-else
          class="w-full flex items-center gap-2.5 px-3 py-1.5 text-left transition-colors"
          :class="item.danger
            ? 'text-red-400 hover:bg-red-500/10'
            : 'text-white/70 hover:text-white hover:bg-white/8'"
          @click="item.action(); emit('close')"
        >
          <span v-if="item.icon" class="text-base leading-none">{{ item.icon }}</span>
          {{ item.label }}
        </button>
      </template>
    </div>
  </Teleport>
</template>
