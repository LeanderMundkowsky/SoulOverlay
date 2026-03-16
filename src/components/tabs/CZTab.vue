<script setup lang="ts">
import { ref } from "vue";
import CZHomeView from "@/components/overlay/CZHomeView.vue";
import CZMapsView from "@/components/overlay/CZMapsView.vue";
import CZShipsView from "@/components/overlay/CZShipsView.vue";

defineProps<{
  active: boolean;
}>();

type SubTab = "home" | "maps" | "ships";

const activeSubTab = ref<SubTab>("home");

const subTabs: { id: SubTab; label: string }[] = [
  { id: "home", label: "Home" },
  { id: "maps", label: "Maps" },
  { id: "ships", label: "Ships" },
];
</script>

<template>
  <div class="flex h-full">
    <!-- Left sub-navigation -->
    <div class="flex-shrink-0 w-28 bg-[#0d0f14] border-r border-white/10 py-3 select-none">
      <div
        v-for="tab in subTabs"
        :key="tab.id"
        class="relative flex items-center px-4 py-2 cursor-pointer transition-colors text-xs font-semibold tracking-wider uppercase"
        :class="activeSubTab === tab.id
          ? 'text-white bg-white/5'
          : 'text-white/40 hover:text-white/70 hover:bg-white/[0.03]'"
        @click="activeSubTab = tab.id"
      >
        <!-- Active indicator bar -->
        <div
          class="absolute left-0 top-0 bottom-0 w-0.5 transition-colors"
          :class="activeSubTab === tab.id ? 'bg-blue-500' : 'bg-transparent'"
        />
        {{ tab.label }}
      </div>
    </div>

    <!-- Main content area -->
    <div class="flex-1 min-w-0 overflow-hidden">
      <CZHomeView v-show="activeSubTab === 'home'" />
      <CZMapsView v-show="activeSubTab === 'maps'" />
      <CZShipsView v-show="activeSubTab === 'ships'" />
    </div>
  </div>
</template>
