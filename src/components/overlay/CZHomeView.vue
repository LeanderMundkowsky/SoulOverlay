<script setup lang="ts">
import { useContestedZoneTimer } from "@/composables/useContestedZoneTimer";
import { useCzSelfTimers } from "@/composables/useCzSelfTimers";
import type { SelfTimerDisplay } from "@/composables/useCzSelfTimers";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";

const { state, formattedTimeLeft, formattedPhaseTimeLeft } = useContestedZoneTimer();
const { zones, timersByZone, loaded: selfTimersLoaded, startTimer, resetTimer, adjustTimer, resetAll } = useCzSelfTimers();

function lightBgClass(color: string): string {
  if (color === "red") return "bg-red-500 border-red-700 shadow-red-500/40 shadow-md";
  if (color === "green") return "bg-green-500 border-green-700 shadow-green-500/40 shadow-md";
  return "bg-neutral-800 border-neutral-600";
}

function timerColorClass(timer: SelfTimerDisplay): string {
  if (timer.color === "green") return "text-green-400";
  if (timer.color === "yellow") return "text-yellow-400";
  if (timer.color === "red") return "text-red-400";
  return "text-white";
}

function categoryTimers(list: SelfTimerDisplay[], category: string): SelfTimerDisplay[] {
  return list.filter((t) => t.category === category);
}

function hasCategoryTimers(list: SelfTimerDisplay[], category: string): boolean {
  return list.some((t) => t.category === category);
}
</script>

<template>
  <div class="h-full overflow-y-auto px-4 py-3 space-y-4 select-none">
    <!-- Executive Hangar Timer -->
    <div class="bg-[#1a1d24] border border-white/10 rounded-lg p-5 text-center">
      <h2 class="text-xs font-semibold text-white/50 tracking-widest uppercase mb-4">
        Executive Hangar Timer
      </h2>

      <!-- Loading/Error state -->
      <div v-if="!state.loaded && !state.error" class="py-6">
        <LoadingSpinner />
        <p class="text-white/40 text-xs mt-2">Fetching timer data...</p>
      </div>
      <div v-else-if="state.error" class="py-4">
        <p class="text-red-400 text-sm">{{ state.error }}</p>
        <p class="text-white/30 text-xs mt-1">Timer sync requires internet connection</p>
      </div>

      <!-- Timer display -->
      <template v-else>
        <!-- Main countdown -->
        <div class="text-4xl font-mono font-bold text-white tracking-wider mb-4">
          {{ formattedTimeLeft }}
        </div>

        <!-- 5 phase lights -->
        <div class="flex justify-center gap-3 mb-4">
          <div
            v-for="(color, i) in state.lights"
            :key="i"
            class="w-8 h-8 rounded-full border-2 transition-all duration-300"
            :class="lightBgClass(color)"
          />
        </div>

        <!-- Status text -->
        <div class="flex items-center justify-center gap-2">
          <span class="text-lg font-bold" :class="state.statusColor">
            {{ state.statusText }}
          </span>
          <span v-if="state.phase !== 'black'" class="text-white/40 text-sm">
            ·
            {{ state.phase === "red" ? "Opens in" : "Resets in" }}
            {{ formattedPhaseTimeLeft }}
          </span>
        </div>
      </template>
    </div>

    <!-- Self Timers -->
    <div v-if="selfTimersLoaded">
      <div class="flex items-center justify-between mb-3">
        <h2 class="text-xs font-semibold text-white/50 tracking-widest uppercase">
          Self Timers
        </h2>
        <button
          class="text-xs text-white/30 hover:text-white/60 transition-colors"
          @click="resetAll"
        >
          Reset All
        </button>
      </div>

      <!-- Zone sections -->
      <div v-for="zone in zones" :key="zone" class="bg-[#1a1d24] border border-white/10 rounded-lg p-4 mb-3">
        <h3 class="text-xs font-semibold text-white/60 tracking-wider uppercase mb-3">
          {{ zone }}
        </h3>

        <div class="space-y-3">
          <!-- Keycards -->
          <div v-if="hasCategoryTimers(timersByZone.get(zone) ?? [], 'keycard')">
            <div class="text-[10px] text-white/30 uppercase tracking-wider mb-1.5">Keycards</div>
            <div class="grid grid-cols-3 gap-2">
              <div
                v-for="timer in categoryTimers(timersByZone.get(zone) ?? [], 'keycard')"
                :key="timer.id"
                class="bg-[#141720] border border-white/5 rounded-lg px-2.5 py-2 text-center"
              >
                <div class="text-[10px] text-white/50 mb-1 truncate">{{ timer.label }}</div>
                <div class="font-mono text-sm font-bold" :class="timerColorClass(timer)">
                  {{ timer.displayTime }}
                </div>
                <div class="flex justify-center gap-1 mt-1.5">
                  <button
                    class="px-1.5 py-0.5 text-[10px] bg-white/5 border border-white/10 hover:bg-white/10 rounded transition-colors"
                    @click="adjustTimer(timer.id, -1)"
                  >−</button>
                  <button
                    class="px-1.5 py-0.5 text-[10px] bg-white/5 border border-white/10 hover:bg-white/10 rounded transition-colors"
                    @click="resetTimer(timer.id)"
                  >R</button>
                  <button
                    class="px-1.5 py-0.5 text-[10px] bg-white/5 border border-white/10 hover:bg-white/10 rounded transition-colors"
                    @click="adjustTimer(timer.id, 1)"
                  >+</button>
                </div>
                <button
                  class="mt-1.5 w-full px-1 py-0.5 text-[10px] font-semibold rounded transition-colors"
                  :class="timer.status === 'running'
                    ? 'bg-red-500/20 text-red-400 hover:bg-red-500/30 border border-red-500/20'
                    : 'bg-teal-500/20 text-teal-400 hover:bg-teal-500/30 border border-teal-500/20'"
                  @click="startTimer(timer.id)"
                >
                  {{ timer.status === "running" ? "Restart" : "Start" }}
                </button>
              </div>
            </div>
          </div>

          <!-- Compboards -->
          <div v-if="hasCategoryTimers(timersByZone.get(zone) ?? [], 'compboard')">
            <div class="text-[10px] text-white/30 uppercase tracking-wider mb-1.5">
              {{ zone === "PYAM-SUPVISR" ? "Red Keycards" : "Compboards" }}
            </div>
            <div class="grid grid-cols-3 gap-2">
              <div
                v-for="timer in categoryTimers(timersByZone.get(zone) ?? [], 'compboard')"
                :key="timer.id"
                class="bg-[#141720] border border-white/5 rounded-lg px-2.5 py-2 text-center"
              >
                <div class="text-[10px] text-white/50 mb-1 truncate">{{ timer.label }}</div>
                <div class="font-mono text-sm font-bold" :class="timerColorClass(timer)">
                  {{ timer.displayTime }}
                </div>
                <div class="flex justify-center gap-1 mt-1.5">
                  <button
                    class="px-1.5 py-0.5 text-[10px] bg-white/5 border border-white/10 hover:bg-white/10 rounded transition-colors"
                    @click="adjustTimer(timer.id, -1)"
                  >−</button>
                  <button
                    class="px-1.5 py-0.5 text-[10px] bg-white/5 border border-white/10 hover:bg-white/10 rounded transition-colors"
                    @click="resetTimer(timer.id)"
                  >R</button>
                  <button
                    class="px-1.5 py-0.5 text-[10px] bg-white/5 border border-white/10 hover:bg-white/10 rounded transition-colors"
                    @click="adjustTimer(timer.id, 1)"
                  >+</button>
                </div>
                <button
                  class="mt-1.5 w-full px-1 py-0.5 text-[10px] font-semibold rounded transition-colors"
                  :class="timer.status === 'running'
                    ? 'bg-red-500/20 text-red-400 hover:bg-red-500/30 border border-red-500/20'
                    : 'bg-teal-500/20 text-teal-400 hover:bg-teal-500/30 border border-teal-500/20'"
                  @click="startTimer(timer.id)"
                >
                  {{ timer.status === "running" ? "Restart" : "Start" }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Attribution -->
    <div class="text-center text-[10px] text-white/20 pb-2">
      Timer data from
      <a href="https://contestedzonetimers.com" target="_blank" class="underline hover:text-white/40">
        contestedzonetimers.com
      </a>
    </div>
  </div>
</template>
