<script setup lang="ts">
import { useGameStore } from "@/stores/game";

defineProps<{
  scDetected: boolean;
}>();

const emit = defineEmits<{
  (e: "toggle-debug"): void;
}>();

const gameStore = useGameStore();

function formatTime(date: Date | null): string {
  if (!date) return "";
  return date.toLocaleTimeString("en-US", {
    hour: "2-digit",
    minute: "2-digit",
  });
}
</script>

<template>
  <div class="flex-shrink-0 flex items-center justify-between px-5 py-2 bg-[#111318] border-t border-white/10 text-xs">
    <!-- Left: SC status + location + ship -->
    <div class="flex items-center gap-4">
      <!-- SC connection dot -->
      <div
        class="flex items-center gap-1.5"
        :class="scDetected ? 'text-green-400' : 'text-yellow-400/70'"
      >
        <span
          class="w-1.5 h-1.5 rounded-full flex-shrink-0"
          :class="scDetected ? 'bg-green-400' : 'bg-yellow-400/70'"
        ></span>
        <span>{{ scDetected ? "Star Citizen Connected" : "Waiting for Star Citizen..." }}</span>
      </div>

      <div v-if="gameStore.location" class="flex items-center gap-1.5 text-white/60">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"></path>
          <circle cx="12" cy="10" r="3"></circle>
        </svg>
        <span>{{ gameStore.location }}</span>
      </div>

      <div v-if="gameStore.ship" class="flex items-center gap-1.5 text-white/60">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M17.8 19.2 16 11l3.5-3.5C21 6 21.5 4 21 3c-1-.5-3 0-4.5 1.5L13 8 4.8 6.2c-.5-.1-.9.1-1.1.5l-.3.5c-.2.5-.1 1 .3 1.3L9 12l-2 3H4l-1 1 3 2 2 3 1-1v-3l3-2 3.5 5.3c.3.4.8.5 1.3.3l.5-.2c.4-.3.6-.7.5-1.2z"></path>
        </svg>
        <span>{{ gameStore.ship }}</span>
      </div>
    </div>

    <!-- Right: events + debug toggle + ESC hint -->
    <div class="flex items-center gap-4">
      <div v-if="gameStore.lastKill" class="flex items-center gap-1.5 text-green-400/70">
        <span>Kill: {{ gameStore.lastKill }}</span>
        <span class="text-white/30">{{ formatTime(gameStore.lastKillTime) }}</span>
      </div>
      <div v-if="gameStore.lastDeath" class="flex items-center gap-1.5 text-red-400/70">
        <span>Death: {{ gameStore.lastDeath }}</span>
        <span class="text-white/30">{{ formatTime(gameStore.lastDeathTime) }}</span>
      </div>

      <!-- Debug toggle -->
      <button
        @click="emit('toggle-debug')"
        class="text-white/30 hover:text-white/70 transition-colors px-1 py-0.5 rounded hover:bg-white/10"
        title="Toggle debug panel"
      >
        DEBUG
      </button>

      <div class="text-white/20">ESC to close</div>
    </div>
  </div>
</template>
