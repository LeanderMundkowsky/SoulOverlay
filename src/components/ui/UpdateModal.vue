<script setup lang="ts">
import { useUpdateStore } from "@/stores/update";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import IconInfoCircle from "@/components/icons/IconInfoCircle.vue";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const updateStore = useUpdateStore();
const appVersion = __APP_VERSION__;

async function handleCheck() {
  await updateStore.checkForUpdates();
}

async function handleInstall() {
  await updateStore.installUpdate();
}
</script>

<template>
  <div class="fixed inset-0 z-[9998] flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="emit('close')">
    <div class="bg-[#111318] border border-white/10 rounded-2xl shadow-2xl w-[420px] max-h-[80vh] flex flex-col overflow-hidden">
      <!-- Header -->
      <div class="px-5 pt-5 pb-3 border-b border-white/5">
        <h2 class="text-lg font-semibold text-white">Software Update</h2>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto px-5 py-4">
        <!-- Checking state -->
        <div v-if="updateStore.checking" class="flex flex-col items-center py-8">
          <LoadingSpinner text="Checking for updates..." />
        </div>

        <!-- Installing state -->
        <div v-else-if="updateStore.installing" class="flex flex-col items-center py-8">
          <LoadingSpinner text="Downloading and installing update..." />
          <p class="text-white/30 text-xs mt-3">The app will restart automatically.</p>
        </div>

        <!-- Update available -->
        <div v-else-if="updateStore.updateAvailable && updateStore.updateInfo" class="space-y-4">
          <div class="flex items-start gap-3">
            <IconInfoCircle class="w-5 h-5 text-blue-400 mt-0.5 flex-shrink-0" />
            <div>
              <p class="text-white font-medium">
                Version {{ updateStore.updateInfo.version }} is available
              </p>
              <p class="text-white/40 text-xs mt-0.5">
                You are currently running v{{ appVersion }}
              </p>
            </div>
          </div>

          <!-- Release notes -->
          <div
            v-if="updateStore.updateInfo.body"
            class="bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-sm text-white/60 max-h-48 overflow-y-auto whitespace-pre-wrap"
          >
            {{ updateStore.updateInfo.body }}
          </div>
        </div>

        <!-- Up to date -->
        <div v-else-if="!updateStore.error" class="flex flex-col items-center py-8 text-center">
          <div class="w-10 h-10 rounded-full bg-green-500/10 flex items-center justify-center mb-3">
            <svg class="w-5 h-5 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
            </svg>
          </div>
          <p class="text-white font-medium">You're up to date!</p>
          <p class="text-white/40 text-xs mt-1">Version {{ appVersion }}</p>
        </div>

        <!-- Error -->
        <div v-if="updateStore.error" class="mt-4 bg-red-950 border border-red-500/40 rounded-lg px-4 py-3 text-sm text-red-200">
          {{ updateStore.error }}
        </div>
      </div>

      <!-- Actions -->
      <div class="px-5 py-4 border-t border-white/10 flex items-center gap-3">
        <template v-if="updateStore.updateAvailable && !updateStore.installing && !updateStore.checking">
          <button
            @click="handleInstall"
            class="flex-1 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium py-2 px-4 rounded-lg transition-colors"
          >
            Download &amp; Install
          </button>
          <button
            @click="emit('close')"
            class="text-white/40 hover:text-white text-sm py-2 px-3 rounded-lg hover:bg-white/5 transition-colors"
          >
            Later
          </button>
        </template>
        <template v-else-if="!updateStore.checking && !updateStore.installing">
          <button
            @click="handleCheck"
            class="flex-1 bg-white/5 hover:bg-white/10 text-white text-sm font-medium py-2 px-4 rounded-lg border border-white/10 transition-colors"
          >
            Check Again
          </button>
          <button
            @click="emit('close')"
            class="text-white/40 hover:text-white text-sm py-2 px-3 rounded-lg hover:bg-white/5 transition-colors"
          >
            Close
          </button>
        </template>
      </div>
    </div>
  </div>
</template>
