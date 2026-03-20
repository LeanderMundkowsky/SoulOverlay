<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useWikeloStore } from "@/stores/wikelo";
import { useInventoryStore } from "@/stores/inventory";
import { useOrgStore } from "@/stores/org";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import IconSearch from "@/components/icons/IconSearch.vue";
import IconClose from "@/components/icons/IconClose.vue";
import SearchableDropdown from "@/components/ui/SearchableDropdown.vue";
import type { DropdownOption } from "@/components/ui/SearchableDropdown.vue";

const wikeloStore = useWikeloStore();
const inventoryStore = useInventoryStore();
const orgStore = useOrgStore();

// Inventory scope: null = personal, orgId = that org
const inventoryScope = ref<number | null>(null);

// -- Data loading ----------------------------------------------------------

onMounted(async () => {
  if (wikeloStore.trades.length === 0) {
    await wikeloStore.loadTrades();
  }
  await wikeloStore.loadCompletions();
  if (inventoryStore.entries.length === 0) {
    inventoryStore.loadInventory();
  }
});

// -- Filters ----------------------------------------------------------------

const searchQuery = ref("");
const showActiveOnly = ref(false);
const selectedCategory = ref<DropdownOption | null>(null);
const selectedReputation = ref<DropdownOption | null>(null);

const categories = computed(() => {
  const set = new Set<string>();
  for (const t of wikeloStore.trades) set.add(t.category);
  return Array.from(set).sort();
});

const categoryOptions = computed<DropdownOption[]>(() =>
  categories.value.map((c) => ({ id: c, label: c })),
);

const reputationRanks = computed(() => {
  const order = ["New Customer", "Very Good Customer", "Very Best Customer"];
  const set = new Set<string>();
  for (const t of wikeloStore.trades) set.add(t.reputation);
  return order.filter((r) => set.has(r));
});

const reputationOptions = computed<DropdownOption[]>(() =>
  reputationRanks.value.map((r) => ({ id: r, label: r })),
);

const filteredTrades = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  return wikeloStore.trades.filter((t) => {
    if (showActiveOnly.value && !t.active) return false;
    if (selectedCategory.value && t.category !== selectedCategory.value.id) return false;
    if (selectedReputation.value && t.reputation !== selectedReputation.value.id) return false;
    if (q) {
      const haystack =
        t.mission_name.toLowerCase() +
        " " +
        t.reward_names.join(" ").toLowerCase() +
        " " +
        t.required_items.map((r) => r.item).join(" ").toLowerCase();
      if (!haystack.includes(q)) return false;
    }
    return true;
  });
});

const completedInView = computed(
  () => filteredTrades.value.filter((t) => wikeloStore.isCompleted(t.id)).length,
);

// -- Inventory cross-reference ---------------------------------------------

function ownedQuantity(itemName: string): number {
  const key = itemName.toLowerCase();
  let total = 0;
  const entries = inventoryScope.value === null
    ? inventoryStore.entries
    : orgStore.getInventory(inventoryScope.value);
  for (const entry of entries) {
    if (entry.entity_name.toLowerCase() === key) {
      total += entry.quantity;
    }
  }
  return total;
}

// -- Reputation color helper -----------------------------------------------

const REPUTATION_COLORS: Record<string, string> = {
  "New Customer": "text-white/40",
  "Very Good Customer": "text-teal-400",
  "Very Best Customer": "text-yellow-400",
};

function reputationColor(rep: string): string {
  return REPUTATION_COLORS[rep] ?? "text-white/60";
}
</script>

<template>
  <div class="p-4 max-w-5xl mx-auto w-full space-y-4">

    <!-- Loading -->
    <div v-if="wikeloStore.loading" class="flex justify-center py-16">
      <LoadingSpinner />
    </div>

    <!-- Error -->
    <AlertBanner v-else-if="wikeloStore.error" variant="error" :message="wikeloStore.error" />

    <template v-else>
      <!-- Header + stats -->
      <div class="flex items-center justify-between">
        <div class="space-y-0.5">
          <h2 class="text-sm font-bold tracking-widest text-white uppercase">Wikelo Contracts</h2>
          <p class="text-xs text-white/40">
            {{ completedInView }} / {{ filteredTrades.length }} completed in view
            <span v-if="wikeloStore.trades.length !== filteredTrades.length">
              ({{ wikeloStore.trades.length }} total)
            </span>
          </p>
        </div>
        <div class="flex items-center gap-2">
          <!-- Inventory scope selector -->
          <div v-if="orgStore.myOrgs.length > 0" class="flex items-center gap-1 bg-white/5 border border-white/10 rounded-lg p-0.5">
            <button
              @click="inventoryScope = null"
              class="text-xs px-2 py-1 rounded transition-colors"
              :class="inventoryScope === null ? 'bg-white/10 text-white' : 'text-white/40 hover:text-white/70'"
            >Personal</button>
            <button
              v-for="org in orgStore.myOrgs"
              :key="org.id"
              @click="inventoryScope = org.id; orgStore.loadInventory(org.id)"
              class="text-xs px-2 py-1 rounded transition-colors"
              :class="inventoryScope === org.id ? 'bg-teal-500/20 text-teal-300' : 'text-white/40 hover:text-white/70'"
            >{{ org.name }}</button>
          </div>
          <button
            class="text-xs text-white/40 hover:text-white/70 transition-colors"
            @click.stop="wikeloStore.loadTrades()"
          >
            Refresh
          </button>
        </div>
      </div>

      <!-- Filters -->
      <div class="flex flex-wrap items-center gap-2">
        <!-- Search -->
        <div class="relative flex-1 min-w-[180px]">
          <IconSearch class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-white/30 pointer-events-none" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search contracts..."
            class="w-full bg-[#111318] border border-white/10 rounded-lg pl-8 pr-8 py-2 text-sm text-white placeholder:text-white/25 focus:outline-none focus:border-white/30 transition-colors"
          />
          <button
            v-if="searchQuery"
            class="absolute right-2 top-1/2 -translate-y-1/2 text-white/30 hover:text-white/60"
            @click="searchQuery = ''"
          >
            <IconClose class="w-3.5 h-3.5" />
          </button>
        </div>

        <!-- Active toggle -->
        <button
          class="px-3 py-2 rounded-lg text-sm font-medium transition-colors border"
          :class="
            showActiveOnly
              ? 'bg-teal-500/20 border-teal-500/50 text-teal-300'
              : 'bg-[#111318] border-white/10 text-white/50 hover:text-white/70'
          "
          @click.stop="showActiveOnly = !showActiveOnly"
        >
          Active only
        </button>

        <!-- Category filter -->
        <div class="w-40 flex-shrink-0">
          <SearchableDropdown
            v-model="selectedCategory"
            :options="categoryOptions"
            placeholder="All categories"
          />
        </div>

        <!-- Reputation filter -->
        <div class="w-48 flex-shrink-0">
          <SearchableDropdown
            v-model="selectedReputation"
            :options="reputationOptions"
            placeholder="All reputation"
          />
        </div>
      </div>

      <!-- Empty state -->
      <p v-if="filteredTrades.length === 0 && !wikeloStore.loading" class="text-center text-white/30 text-sm py-8">
        No contracts match your filters.
      </p>

      <!-- Contract cards -->
      <div class="space-y-2">
        <div
          v-for="trade in filteredTrades"
          :key="trade.id"
          class="rounded border transition-colors"
          :class="
            wikeloStore.isCompleted(trade.id)
              ? 'bg-[#0d1a14] border-teal-500/20'
              : 'bg-[#1a1d24] border-white/10 hover:border-white/20'
          "
        >
          <div class="p-3 space-y-2">
            <!-- Card header -->
            <div class="flex items-start justify-between gap-2">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 flex-wrap">
                  <!-- Inactive badge -->
                  <span
                    v-if="!trade.active"
                    class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-red-500/20 text-red-400 border border-red-500/30 flex-shrink-0"
                  >
                    INACTIVE
                  </span>
                  <span class="text-sm font-semibold text-white truncate">{{ trade.mission_name }}</span>
                </div>
                <div class="flex items-center gap-3 mt-0.5 flex-wrap">
                  <span :class="['text-xs', reputationColor(trade.reputation)]">
                    {{ trade.reputation }}
                  </span>
                  <span class="text-xs text-white/30">{{ trade.category }}</span>
                  <span class="text-xs text-white/20">{{ trade.patch }}</span>
                </div>
              </div>

              <!-- Completion toggle -->
              <button
                class="flex-shrink-0 w-5 h-5 rounded border transition-colors flex items-center justify-center"
                :class="
                  wikeloStore.isCompleted(trade.id)
                    ? 'bg-teal-500 border-teal-400 text-white'
                    : 'bg-transparent border-white/20 text-transparent hover:border-white/40'
                "
                :title="wikeloStore.isCompleted(trade.id) ? 'Mark incomplete' : 'Mark complete'"
                @click.stop="wikeloStore.toggleCompletion(trade.id)"
              >
                <svg class="w-3 h-3" viewBox="0 0 12 12" fill="none">
                  <path d="M2 6l3 3 5-5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
              </button>
            </div>

            <!-- Reward -->
            <div class="flex items-start gap-2">
              <span class="text-[10px] font-semibold text-white/30 uppercase tracking-wider mt-0.5 flex-shrink-0 w-14">Reward</span>
              <div class="flex flex-wrap gap-1">
                <span
                  v-for="(reward, i) in trade.reward_names"
                  :key="i"
                  class="px-1.5 py-0.5 rounded bg-yellow-500/10 border border-yellow-500/20 text-yellow-300 text-xs"
                >
                  {{ reward }}
                </span>
              </div>
            </div>

            <!-- Required items -->
            <div class="flex items-start gap-2">
              <span class="text-[10px] font-semibold text-white/30 uppercase tracking-wider mt-0.5 flex-shrink-0 w-14">Needs</span>
              <div class="flex flex-wrap gap-1.5">
                <div
                  v-for="(req, i) in trade.required_items"
                  :key="i"
                  class="flex items-center gap-1 px-1.5 py-0.5 rounded text-xs"
                  :class="
                    ownedQuantity(req.item) >= req.quantity
                      ? 'bg-teal-500/10 border border-teal-500/20 text-teal-300'
                      : 'bg-[#0d0f14] border border-white/10 text-white/70'
                  "
                >
                  <span>{{ req.item }}</span>
                  <span class="text-white/40">x{{ req.quantity }}</span>
                  <span
                    v-if="ownedQuantity(req.item) > 0"
                    class="text-[10px]"
                    :class="ownedQuantity(req.item) >= req.quantity ? 'text-teal-400' : 'text-orange-400'"
                  >
                    (have {{ ownedQuantity(req.item) }})
                  </span>
                </div>
              </div>
            </div>

            <!-- Description -->
            <p v-if="trade.description" class="text-xs text-white/35 leading-relaxed">
              {{ trade.description }}
            </p>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>