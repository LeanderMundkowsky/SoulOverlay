import { ref, computed, onMounted, onUnmounted } from "vue";
import { commands } from "@/bindings";
import type { CzSelfTimer } from "@/bindings";

export type TimerColor = "white" | "red" | "yellow" | "green";

export interface SelfTimerDisplay extends CzSelfTimer {
  displayTime: string;
  color: TimerColor;
}

function formatShortTime(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

function getTimerColor(timer: CzSelfTimer, remaining: number): TimerColor {
  if (timer.status === "done") return "green";
  if (timer.status === "running") {
    if (remaining <= 180) return "yellow";
    return "red";
  }
  return "white";
}

export function useCzSelfTimers() {
  const timers = ref<CzSelfTimer[]>([]);
  const loaded = ref(false);
  const error = ref<string | null>(null);
  let intervalId: ReturnType<typeof setInterval> | null = null;

  // Tick counter for reactivity
  const tick = ref(0);

  /** Unique zone names in insertion order */
  const zones = computed(() => {
    const seen = new Set<string>();
    const result: string[] = [];
    for (const t of timers.value) {
      if (!seen.has(t.zone)) {
        seen.add(t.zone);
        result.push(t.zone);
      }
    }
    return result;
  });

  /** Timers grouped by zone with display info */
  const timersByZone = computed(() => {
    void tick.value;
    const now = Math.floor(Date.now() / 1000);
    const map = new Map<string, SelfTimerDisplay[]>();

    for (const t of timers.value) {
      let remaining = t.remaining_seconds;
      let status = t.status;

      if (t.status === "running" && t.end_epoch > 0) {
        remaining = Math.max(0, t.end_epoch - now);
        if (remaining <= 0) {
          remaining = 0;
          status = "done";
        }
      }

      const display: SelfTimerDisplay = {
        ...t,
        status,
        remaining_seconds: remaining,
        displayTime: formatShortTime(remaining),
        color: getTimerColor({ ...t, status }, remaining),
      };

      if (!map.has(t.zone)) map.set(t.zone, []);
      map.get(t.zone)!.push(display);
    }

    return map;
  });

  async function loadTimers() {
    try {
      const result = await commands.czLoadSelfTimers();
      if (result.status === "ok") {
        timers.value = result.data;
        loaded.value = true;

        // Resume running timers: if end_epoch is in the past, mark as done
        const now = Math.floor(Date.now() / 1000);
        for (const t of timers.value) {
          if (t.status === "running" && t.end_epoch > 0 && t.end_epoch <= now) {
            t.status = "done";
            t.remaining_seconds = 0;
            await persistTimer(t);
          }
        }
      } else {
        error.value = result.error;
      }
    } catch (e) {
      error.value = String(e);
    }
  }

  async function persistTimer(timer: CzSelfTimer) {
    try {
      await commands.czSaveSelfTimer(timer);
    } catch (e) {
      console.error("Failed to persist timer:", e);
    }
  }

  function findTimer(id: string): CzSelfTimer | undefined {
    return timers.value.find((t) => t.id === id);
  }

  async function startTimer(id: string) {
    const timer = findTimer(id);
    if (!timer) return;

    const duration = timer.status === "idle" ? timer.remaining_seconds : timer.remaining_seconds;
    const now = Math.floor(Date.now() / 1000);
    timer.end_epoch = now + duration;
    timer.status = "running";
    await persistTimer(timer);
  }

  async function resetTimer(id: string) {
    const timer = findTimer(id);
    if (!timer) return;

    timer.remaining_seconds = timer.default_seconds;
    timer.end_epoch = 0;
    timer.status = "idle";
    await persistTimer(timer);
  }

  async function adjustTimer(id: string, deltaMinutes: number) {
    const timer = findTimer(id);
    if (!timer) return;

    // Only adjust if idle or done
    if (timer.status === "running") return;

    const newSeconds = Math.max(0, Math.min(timer.default_seconds, timer.remaining_seconds + deltaMinutes * 60));
    timer.remaining_seconds = newSeconds;
    timer.status = "idle";
    await persistTimer(timer);
  }

  async function resetAll() {
    try {
      await commands.czResetAllSelfTimers();
      await loadTimers();
    } catch (e) {
      console.error("Failed to reset all timers:", e);
    }
  }

  onMounted(async () => {
    await loadTimers();
    intervalId = setInterval(() => {
      tick.value++;

      // Check running timers and mark done
      const now = Math.floor(Date.now() / 1000);
      for (const t of timers.value) {
        if (t.status === "running" && t.end_epoch > 0 && t.end_epoch <= now) {
          t.status = "done";
          t.remaining_seconds = 0;
          persistTimer(t);
        }
      }
    }, 1000);
  });

  onUnmounted(() => {
    if (intervalId !== null) {
      clearInterval(intervalId);
      intervalId = null;
    }
  });

  return {
    timers,
    zones,
    timersByZone,
    loaded,
    error,
    startTimer,
    resetTimer,
    adjustTimer,
    resetAll,
  };
}
