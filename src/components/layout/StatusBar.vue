<script setup lang="ts">
import IconMapPin from "@/components/icons/IconMapPin.vue";
import IconPlane from "@/components/icons/IconPlane.vue";
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
        <IconMapPin class="w-3 h-3" />
        <span>{{ gameStore.location }}</span>
      </div>

      <div v-if="gameStore.ship" class="flex items-center gap-1.5 text-white/60">
        <IconPlane class="w-3 h-3" />
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
