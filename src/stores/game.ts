import { defineStore } from "pinia";
import { ref } from "vue";

export const useGameStore = defineStore("game", () => {
  const scConnected = ref(false);
  const location = ref("");
  const ship = ref("");
  const lastKill = ref("");
  const lastDeath = ref("");
  const lastKillTime = ref<Date | null>(null);
  const lastDeathTime = ref<Date | null>(null);

  function setLocation(loc: string) {
    location.value = loc;
  }

  function setShip(s: string) {
    ship.value = s;
  }

  function addKill(victim: string) {
    lastKill.value = victim;
    lastKillTime.value = new Date();
  }

  function addDeath(killer: string) {
    lastDeath.value = killer;
    lastDeathTime.value = new Date();
  }

  function reset() {
    scConnected.value = false;
    location.value = "";
    ship.value = "";
    lastKill.value = "";
    lastDeath.value = "";
    lastKillTime.value = null;
    lastDeathTime.value = null;
  }

  return {
    scConnected,
    location,
    ship,
    lastKill,
    lastDeath,
    lastKillTime,
    lastDeathTime,
    setLocation,
    setShip,
    addKill,
    addDeath,
    reset,
  };
});
