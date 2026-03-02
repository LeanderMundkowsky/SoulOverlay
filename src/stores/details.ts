import { defineStore } from "pinia";
import { ref } from "vue";

import type { UexResult } from "@/composables/useUex";

export const useDetailsStore = defineStore("details", () => {
  const currentEntity = ref<UexResult | null>(null);
  const requestTabSwitch = ref(false);

  function openEntity(entity: UexResult) {
    currentEntity.value = { ...entity };
    requestTabSwitch.value = true;
  }

  function clearTabSwitchRequest() {
    requestTabSwitch.value = false;
  }

  function clear() {
    currentEntity.value = null;
  }

  return {
    currentEntity,
    requestTabSwitch,
    openEntity,
    clearTabSwitchRequest,
    clear,
  };
});
