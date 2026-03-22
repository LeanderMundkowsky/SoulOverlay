<script setup lang="ts">
defineEmits<{
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();
</script>

<template>
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60"
    @click.self="$emit('cancel')"
  >
    <div class="bg-[#1a1d24] border border-white/10 rounded-xl w-[440px] shadow-2xl">
      <!-- Header -->
      <div class="flex items-center justify-between px-5 pt-5 pb-4 border-b border-white/10">
        <div class="flex items-center gap-2.5">
          <svg class="w-4 h-4 text-amber-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" />
          </svg>
          <h2 class="text-white font-semibold text-base">Import Fleet from UEX</h2>
        </div>
        <button @click="$emit('cancel')" class="text-white/40 hover:text-white/80 transition-colors">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="px-5 py-5 space-y-4 text-sm text-white/60 leading-relaxed">
        <p>
          SoulOverlay manages your fleet in its <span class="text-white/80">own backend</span> — not on UEX. This gives you full control: custom names, notes, photos, and private entries that never leave your account.
        </p>
        <p>
          The <span class="text-white/80">"Import from UEX"</span> function is a one-time starting point to populate your hangar from your existing UEX fleet. UEX only exposes a read-only endpoint, so <span class="text-white/80">changes you make here are never written back to UEX</span>.
        </p>
        <div class="bg-amber-500/10 border border-amber-500/25 rounded-lg px-4 py-3 text-amber-300/90 text-xs flex gap-2.5">
          <svg class="w-4 h-4 flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" />
          </svg>
          <span>This will <strong>delete all existing hangar entries</strong> and replace them with the current data from your UEX account.</span>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-5 pb-5 flex items-center justify-end gap-3">
        <button
          @click="$emit('cancel')"
          class="text-white/40 hover:text-white text-sm py-2 px-4 rounded-lg hover:bg-white/5 transition-colors"
        >
          Cancel
        </button>
        <button
          @click="$emit('confirm')"
          class="flex items-center gap-2 bg-amber-600 hover:bg-amber-500 text-white text-sm font-medium py-2 px-4 rounded-lg transition-colors"
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
          </svg>
          Import & Replace
        </button>
      </div>
    </div>
  </div>
</template>
