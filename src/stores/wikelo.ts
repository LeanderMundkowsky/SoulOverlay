import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { commands } from "@/bindings";
import type { WikeloTrade } from "@/bindings";

export type { WikeloTrade };

export const useWikeloStore = defineStore("wikelo", () => {
  const trades = ref<WikeloTrade[]>([]);
  const completions = ref<Set<string>>(new Set());
  const loading = ref(false);
  const error = ref<string | null>(null);

  const completedCount = computed(() => completions.value.size);

  async function loadTrades() {
    loading.value = true;
    error.value = null;
    try {
      const result = await commands.wikeloGetTrades();
      if (result.status === "error") throw result.error;
      const resp = result.data;
      if (resp.ok && resp.data) {
        trades.value = resp.data;
      } else {
        error.value = resp.error ?? "Unknown error loading trades";
      }
    } catch (e) {
      error.value = String(e);
      console.error("Failed to load Wikelo trades:", e);
    } finally {
      loading.value = false;
    }
  }

  async function loadCompletions() {
    try {
      const result = await commands.wikeloGetCompletions();
      if (result.status === "error") throw result.error;
      completions.value = new Set(result.data);
    } catch (e) {
      console.error("Failed to load Wikelo completions:", e);
    }
  }

  async function toggleCompletion(missionId: string) {
    try {
      const result = await commands.wikeloToggleCompletion(missionId);
      if (result.status === "error") throw result.error;
      if (result.data) {
        completions.value.add(missionId);
      } else {
        completions.value.delete(missionId);
      }
      // Trigger reactivity: replace the Set reference
      completions.value = new Set(completions.value);
    } catch (e) {
      console.error("Failed to toggle Wikelo completion:", e);
    }
  }

  function isCompleted(missionId: string): boolean {
    return completions.value.has(missionId);
  }

  return {
    trades,
    completions,
    loading,
    error,
    completedCount,
    loadTrades,
    loadCompletions,
    toggleCompletion,
    isCompleted,
  };
});
