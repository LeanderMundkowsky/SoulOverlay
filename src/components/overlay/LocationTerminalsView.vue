<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { commands } from "@/bindings";
import type { LocationTerminal } from "@/bindings";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";

const props = defineProps<{
  entityId: string;
  entitySlug: string;
}>();

const emit = defineEmits<{
  (e: "select-terminal", terminal: { id: string; name: string; kind: string; slug: string }): void;
}>();

const terminals = ref<LocationTerminal[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

async function fetchTerminals() {
  loading.value = true;
  error.value = null;
  terminals.value = [];

  try {
    const result = await commands.apiLocationTerminals(props.entitySlug, props.entityId);
    if (result.status === "error") throw result.error;
    const resp = result.data;
    if (resp.ok && resp.data) {
      terminals.value = resp.data;
    } else {
      error.value = resp.error ?? "Unknown error";
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function breadcrumb(t: LocationTerminal): string {
  const parts: string[] = [];
  if (t.orbit_name) parts.push(t.orbit_name);
  if (t.planet_name) parts.push(t.planet_name);
  if (t.system_name) parts.push(t.system_name);
  return parts.join(", ");
}

function selectTerminal(t: LocationTerminal) {
  emit("select-terminal", {
    id: t.id,
    name: `[Terminal] ${t.name}`,
    kind: "location",
    slug: "terminal",
  });
}

onMounted(() => fetchTerminals());
watch(() => props.entityId, () => fetchTerminals());
</script>

<template>
  <div class="flex flex-col flex-1 overflow-hidden">
    <!-- Loading -->
    <div v-if="loading" class="px-4 py-8 flex justify-center">
      <LoadingSpinner text="Loading terminals..." />
    </div>

    <!-- Error -->
    <div v-else-if="error" class="px-4 py-4 text-red-400 text-sm">{{ error }}</div>

    <!-- Terminal list -->
    <div v-else-if="terminals.length > 0" class="flex flex-col flex-1 overflow-hidden">
      <div class="flex items-center px-3 py-1.5 border-b border-white/5 shrink-0">
        <span class="text-xs font-medium text-cyan-400/70">Terminals ({{ terminals.length }})</span>
      </div>
      <div class="overflow-y-auto flex-1 p-1.5 space-y-1">
        <button
          v-for="t in terminals"
          :key="t.id"
          class="w-full text-left border border-white/10 rounded-lg bg-white/[0.02] hover:bg-white/[0.05] transition-colors px-2.5 py-1.5 cursor-pointer"
          @click="selectTerminal(t)"
        >
          <div class="flex items-center justify-between gap-2">
            <span class="text-white/80 text-xs font-medium truncate" :title="t.name">
              {{ t.nickname || t.name }}
            </span>
            <span class="text-cyan-400/60 text-[0.6875rem] shrink-0">View →</span>
          </div>
          <div v-if="breadcrumb(t)" class="mt-0.5">
            <span class="text-white/30 text-[0.6875rem] truncate block">{{ breadcrumb(t) }}</span>
          </div>
        </button>
      </div>
    </div>

    <!-- Empty state -->
    <div v-else class="px-4 py-8 text-center text-white/40 text-sm">
      No terminals found for this location.
    </div>
  </div>
</template>
