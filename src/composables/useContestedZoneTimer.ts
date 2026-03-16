import { ref, computed, onMounted, onUnmounted } from "vue";
import { commands } from "@/bindings";

const RED_PHASE = 2 * 60 * 60;    // 7200s = 2 hours
const GREEN_PHASE = 1 * 60 * 60;  // 3600s = 1 hour
const BLACK_PHASE = 5 * 60;       // 300s  = 5 minutes
const TOTAL_CYCLE = RED_PHASE + GREEN_PHASE + BLACK_PHASE; // 11100s
const LIGHT_COUNT = 5;
const RED_INTERVAL = 24 * 60;     // 1440s — one light turns green every 24 min
const GREEN_INTERVAL = 12 * 60;   // 720s  — one light turns off every 12 min

export type Phase = "red" | "green" | "black";
export type LightColor = "red" | "green" | "off";

export interface TimerState {
  /** Current phase */
  phase: Phase;
  /** Total time left in the entire cycle (seconds) */
  timeLeft: number;
  /** Time left in the current phase (seconds) */
  phaseTimeLeft: number;
  /** Color of each of the 5 lights */
  lights: LightColor[];
  /** Status text, e.g. "Hangar Closed" */
  statusText: string;
  /** Status CSS color class */
  statusColor: string;
  /** Whether the timer has loaded the cycle start epoch */
  loaded: boolean;
  /** Error message if cfg.dat fetch failed */
  error: string | null;
}

function formatTime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  return `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

export function useContestedZoneTimer() {
  const cycleStart = ref<number | null>(null);
  const loaded = ref(false);
  const error = ref<string | null>(null);
  let intervalId: ReturnType<typeof setInterval> | null = null;

  // Reactive tick counter to force recomputation every second
  const tick = ref(0);

  const state = computed<TimerState>(() => {
    // Reference tick to make this computed reactive on each interval
    void tick.value;

    if (!loaded.value || cycleStart.value === null) {
      return {
        phase: "black",
        timeLeft: 0,
        phaseTimeLeft: 0,
        lights: Array(LIGHT_COUNT).fill("off") as LightColor[],
        statusText: "Loading...",
        statusColor: "text-white/50",
        loaded: false,
        error: error.value,
      };
    }

    const nowSec = Math.floor(Date.now() / 1000);
    const elapsed = nowSec - cycleStart.value;
    const remaining = TOTAL_CYCLE - (((elapsed % TOTAL_CYCLE) + TOTAL_CYCLE) % TOTAL_CYCLE);
    const timeLeft = remaining;

    let phase: Phase;
    let phaseTimeLeft: number;
    let statusText: string;
    let statusColor: string;
    const lights: LightColor[] = [];

    if (timeLeft > GREEN_PHASE + BLACK_PHASE) {
      // Red phase
      phase = "red";
      const redTime = timeLeft - (GREEN_PHASE + BLACK_PHASE);
      phaseTimeLeft = redTime;
      const timeSinceRedStart = RED_PHASE - redTime;

      for (let i = 0; i < LIGHT_COUNT; i++) {
        lights.push(timeSinceRedStart >= (i + 1) * RED_INTERVAL ? "green" : "red");
      }

      statusText = "Hangar Closed";
      statusColor = "text-red-400";
    } else if (timeLeft > BLACK_PHASE) {
      // Green phase
      phase = "green";
      const greenTime = timeLeft - BLACK_PHASE;
      phaseTimeLeft = greenTime;
      const timeSinceGreenStart = GREEN_PHASE - greenTime;

      for (let i = 0; i < LIGHT_COUNT; i++) {
        lights.push(timeSinceGreenStart >= (i + 1) * GREEN_INTERVAL ? "off" : "green");
      }

      statusText = "Hangar Open";
      statusColor = "text-green-400";
    } else {
      // Black phase
      phase = "black";
      phaseTimeLeft = timeLeft;

      for (let i = 0; i < LIGHT_COUNT; i++) {
        lights.push("off");
      }

      statusText = "Hangar Resetting";
      statusColor = "text-yellow-400";
    }

    return {
      phase,
      timeLeft,
      phaseTimeLeft,
      lights,
      statusText,
      statusColor,
      loaded: true,
      error: null,
    };
  });

  const formattedTimeLeft = computed(() => formatTime(state.value.timeLeft));
  const formattedPhaseTimeLeft = computed(() => formatTime(state.value.phaseTimeLeft));

  async function fetchCycleStart() {
    try {
      const result = await commands.czGetCycleStart();
      if (result.status === "ok" && result.data.ok && result.data.data !== null) {
        cycleStart.value = result.data.data;
        loaded.value = true;
        error.value = null;
      } else {
        const msg = result.status === "ok" ? result.data.error : result.error;
        error.value = msg ?? "Failed to fetch cycle start";
      }
    } catch (e) {
      error.value = String(e);
    }
  }

  onMounted(async () => {
    await fetchCycleStart();
    intervalId = setInterval(() => {
      tick.value++;
    }, 1000);
  });

  onUnmounted(() => {
    if (intervalId !== null) {
      clearInterval(intervalId);
      intervalId = null;
    }
  });

  return {
    state,
    formattedTimeLeft,
    formattedPhaseTimeLeft,
    fetchCycleStart,
  };
}
