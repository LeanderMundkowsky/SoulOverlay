<script setup lang="ts">
import { ref } from "vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import { useBackendStore } from "@/stores/backend";

const backendStore = useBackendStore();

const secretKeyInput = ref(backendStore.account?.uex_secret_key ?? "");
const secretKeySaving = ref(false);
const secretKeyError = ref<string | null>(null);
const secretKeySaved = ref(false);

// Keep input in sync if account updates elsewhere
import { watch } from "vue";
watch(
  () => backendStore.account?.uex_secret_key,
  (val) => { secretKeyInput.value = val ?? ""; },
);

async function saveSecretKey() {
  secretKeySaving.value = true;
  secretKeyError.value = null;
  secretKeySaved.value = false;
  const val = secretKeyInput.value.trim() || null;
  const err = await backendStore.updateSecretKey(val);
  secretKeySaving.value = false;
  if (err) {
    secretKeyError.value = err;
  } else {
    secretKeySaved.value = true;
    setTimeout(() => { secretKeySaved.value = false; }, 3000);
  }
}

async function handleLogout() {
  await backendStore.logout();
}

function formatDate(ts: string | undefined): string {
  if (!ts) return "—";
  try {
    return new Date(ts).toLocaleDateString(undefined, {
      year: "numeric", month: "short", day: "numeric",
    });
  } catch {
    return ts;
  }
}
</script>

<template>
  <div class="p-6 max-w-2xl mx-auto w-full space-y-4">
    <!-- Re-login hint: token present but session expired -->
    <AlertBanner
      v-if="backendStore.showReloginHint"
      variant="warning"
      message="Your session has expired. Please log in again."
    />

    <!-- Not logged in at all -->
    <div
      v-if="!backendStore.isLoggedIn && !backendStore.showReloginHint"
      class="flex flex-col items-center justify-center py-16 text-white/40 space-y-2"
    >
      <svg class="w-12 h-12 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1">
        <path stroke-linecap="round" stroke-linejoin="round" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
      </svg>
      <p class="text-sm">Not logged in</p>
      <p class="text-xs text-white/30">Use the Login button in the top right to sign in.</p>
    </div>

    <!-- Logged in — account info -->
    <template v-if="backendStore.isLoggedIn && backendStore.account">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <h2 class="text-white/80 text-sm font-semibold uppercase tracking-wider">My Account</h2>
        <button
          @click="handleLogout"
          class="text-xs text-red-400/70 hover:text-red-400 transition-colors"
        >
          Sign Out
        </button>
      </div>

      <!-- Identity card -->
      <div class="bg-[#1a1d24] border border-white/10 rounded-lg p-5">
        <div class="flex items-center gap-4">
          <div class="w-14 h-14 rounded-full bg-white/10 flex items-center justify-center shrink-0">
            <span class="text-white/60 text-2xl font-bold select-none">
              {{ backendStore.account.username.charAt(0).toUpperCase() }}
            </span>
          </div>
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="text-white text-lg font-semibold">{{ backendStore.account.username }}</span>
              <!-- Roles -->
              <span
                v-for="role in backendStore.account.roles"
                :key="role"
                class="text-xs px-1.5 py-0.5 rounded"
                :class="role === 'ROLE_ADMIN'
                  ? 'bg-red-500/20 text-red-400'
                  : 'bg-white/10 text-white/50'"
              >
                {{ role.replace('ROLE_', '') }}
              </span>
            </div>
            <div class="text-white/40 text-sm mt-0.5">{{ backendStore.account.email }}</div>
            <div class="text-white/30 text-xs mt-1">Member since {{ formatDate(backendStore.account.created_at) }}</div>
          </div>
        </div>
      </div>

      <!-- UEX Secret Key -->
      <div class="bg-[#1a1d24] border border-white/10 rounded-lg p-5 space-y-3">
        <h3 class="text-white/60 text-xs font-semibold uppercase tracking-wider">UEX Secret Key</h3>
        <p class="text-white/30 text-xs leading-relaxed">
          Required for Hangar access. Found in your
          <a href="https://uexcorp.space/account" class="text-blue-400/60 hover:text-blue-400">UEX account settings</a>.
        </p>
        <div class="flex gap-2">
          <input
            v-model="secretKeyInput"
            type="password"
            placeholder="Enter your UEX Corp secret key"
            class="flex-1 bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500/50 transition-colors"
          />
          <button
            @click="saveSecretKey"
            :disabled="secretKeySaving"
            class="px-4 py-2 text-sm font-medium rounded-lg transition-colors disabled:opacity-40"
            :class="secretKeySaved
              ? 'bg-green-600/80 text-white'
              : 'bg-blue-600 hover:bg-blue-500 text-white'"
          >
            {{ secretKeySaving ? "Saving…" : secretKeySaved ? "Saved!" : "Save" }}
          </button>
        </div>
        <div v-if="secretKeyError" class="text-red-400 text-xs bg-red-500/10 border border-red-500/20 rounded-lg px-3 py-2">
          {{ secretKeyError }}
        </div>
        <!-- Current state indicator -->
        <div v-if="backendStore.account.uex_secret_key" class="flex items-center gap-1.5 text-xs text-green-400/70">
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
          </svg>
          Secret key is configured
        </div>
        <div v-else class="text-xs text-yellow-400/60">No secret key set — Hangar will be unavailable.</div>
      </div>
    </template>

    <!-- Loading fallback -->
    <div v-if="backendStore.loading" class="flex justify-center py-12">
      <LoadingSpinner />
    </div>
  </div>
</template>
