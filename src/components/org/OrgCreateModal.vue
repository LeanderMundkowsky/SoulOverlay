<script setup lang="ts">
import { ref } from "vue";
import { useOrgStore } from "@/stores/org";

const emit = defineEmits<{ close: [] }>();
const orgStore = useOrgStore();

const name = ref("");
const description = ref("");
const saving = ref(false);
const error = ref<string | null>(null);

async function submit() {
  if (!name.value.trim()) return;
  saving.value = true;
  error.value = null;
  const err = await orgStore.createOrg(name.value.trim(), description.value.trim() || null);
  saving.value = false;
  if (err) {
    error.value = err;
  } else {
    emit("close");
  }
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60" @click.self="emit('close')">
    <div class="bg-[#1a1a2e] border border-white/10 rounded-xl w-full max-w-md mx-4 p-6 space-y-4">
      <div class="flex items-center justify-between">
        <h2 class="text-white font-semibold text-sm uppercase tracking-wider">Create Organization</h2>
        <button @click="emit('close')" class="text-white/40 hover:text-white/80 transition-colors">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="space-y-3">
        <div>
          <label class="block text-xs text-white/50 mb-1">Organization Name</label>
          <input
            v-model="name"
            type="text"
            maxlength="100"
            placeholder="Iron Wing"
            class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-white/20 focus:outline-none focus:border-teal-500/50"
            @keydown.enter="submit"
          />
        </div>
        <div>
          <label class="block text-xs text-white/50 mb-1">Description <span class="text-white/20">(optional)</span></label>
          <textarea
            v-model="description"
            rows="3"
            maxlength="1000"
            placeholder="Tell people what your org is about..."
            class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-white/20 focus:outline-none focus:border-teal-500/50 resize-none"
          />
        </div>

        <p v-if="error" class="text-xs text-red-400">{{ error }}</p>
      </div>

      <div class="flex gap-2 justify-end">
        <button
          @click="emit('close')"
          class="px-4 py-2 text-xs text-white/50 hover:text-white/80 transition-colors"
        >Cancel</button>
        <button
          @click="submit"
          :disabled="!name.trim() || saving"
          class="px-4 py-2 text-xs bg-teal-500/20 border border-teal-500/40 text-teal-300 rounded-lg hover:bg-teal-500/30 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          {{ saving ? "Creating…" : "Create" }}
        </button>
      </div>
    </div>
  </div>
</template>
