<script setup lang="ts">
import { ref } from "vue";
import { useBackendStore } from "@/stores/backend";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const backendStore = useBackendStore();

const activeTab = ref<"login" | "register">("login");

// Login form
const loginUsername = ref("");
const loginPassword = ref("");
const loginError = ref<string | null>(null);
const loginLoading = ref(false);

// Register form
const registerUsername = ref("");
const registerEmail = ref("");
const registerPassword = ref("");
const registerError = ref<string | null>(null);
const registerLoading = ref(false);

async function submitLogin() {
  if (!loginUsername.value || !loginPassword.value) return;
  loginLoading.value = true;
  loginError.value = null;
  const err = await backendStore.login(loginUsername.value, loginPassword.value);
  loginLoading.value = false;
  if (err) {
    loginError.value = err;
  } else {
    emit("close");
  }
}

async function submitRegister() {
  if (!registerUsername.value || !registerEmail.value || !registerPassword.value) return;
  registerLoading.value = true;
  registerError.value = null;
  const err = await backendStore.register(registerUsername.value, registerEmail.value, registerPassword.value);
  registerLoading.value = false;
  if (err) {
    registerError.value = err;
  } else {
    emit("close");
  }
}
</script>

<template>
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60"
    @click.self="emit('close')"
  >
    <div class="bg-[#1a1d24] border border-white/10 rounded-xl w-[380px] shadow-2xl">
      <!-- Header -->
      <div class="flex items-center justify-between px-5 pt-5 pb-4 border-b border-white/10">
        <h2 class="text-white font-semibold text-base">SoulOverlay Account</h2>
        <button
          @click="emit('close')"
          class="text-white/40 hover:text-white/80 transition-colors"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Tab switcher -->
      <div class="flex border-b border-white/10 px-5 mt-4">
        <button
          v-for="tab in ['login', 'register'] as const"
          :key="tab"
          class="mr-4 pb-2 text-sm font-medium transition-colors border-b-2 -mb-px"
          :class="activeTab === tab
            ? 'text-white border-blue-500'
            : 'text-white/40 border-transparent hover:text-white/70'"
          @click="activeTab = tab"
        >
          {{ tab === 'login' ? 'Login' : 'Register' }}
        </button>
      </div>

      <!-- Login form -->
      <form v-if="activeTab === 'login'" class="p-5 space-y-4" @submit.prevent="submitLogin">
        <div>
          <label class="block text-white/60 text-xs font-medium uppercase tracking-wider mb-1.5">Username</label>
          <input
            v-model="loginUsername"
            type="text"
            autocomplete="username"
            placeholder="Your username"
            class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500/50 transition-colors"
          />
        </div>
        <div>
          <label class="block text-white/60 text-xs font-medium uppercase tracking-wider mb-1.5">Password</label>
          <input
            v-model="loginPassword"
            type="password"
            autocomplete="current-password"
            placeholder="Your password"
            class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500/50 transition-colors"
          />
        </div>
        <div v-if="loginError" class="text-red-400 text-xs rounded-lg bg-red-500/10 border border-red-500/20 px-3 py-2">
          {{ loginError }}
        </div>
        <button
          type="submit"
          :disabled="loginLoading || !loginUsername || !loginPassword"
          class="w-full bg-blue-600 hover:bg-blue-500 disabled:opacity-40 disabled:cursor-not-allowed text-white text-sm font-medium rounded-lg px-4 py-2 transition-colors"
        >
          {{ loginLoading ? "Signing in…" : "Sign In" }}
        </button>
      </form>

      <!-- Register form -->
      <form v-else class="p-5 space-y-4" @submit.prevent="submitRegister">
        <div>
          <label class="block text-white/60 text-xs font-medium uppercase tracking-wider mb-1.5">Username</label>
          <input
            v-model="registerUsername"
            type="text"
            autocomplete="username"
            placeholder="Choose a username"
            class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500/50 transition-colors"
          />
        </div>
        <div>
          <label class="block text-white/60 text-xs font-medium uppercase tracking-wider mb-1.5">Email</label>
          <input
            v-model="registerEmail"
            type="email"
            autocomplete="email"
            placeholder="Your email address"
            class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500/50 transition-colors"
          />
        </div>
        <div>
          <label class="block text-white/60 text-xs font-medium uppercase tracking-wider mb-1.5">Password</label>
          <input
            v-model="registerPassword"
            type="password"
            autocomplete="new-password"
            placeholder="Choose a password"
            class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500/50 transition-colors"
          />
        </div>
        <div v-if="registerError" class="text-red-400 text-xs rounded-lg bg-red-500/10 border border-red-500/20 px-3 py-2">
          {{ registerError }}
        </div>
        <button
          type="submit"
          :disabled="registerLoading || !registerUsername || !registerEmail || !registerPassword"
          class="w-full bg-blue-600 hover:bg-blue-500 disabled:opacity-40 disabled:cursor-not-allowed text-white text-sm font-medium rounded-lg px-4 py-2 transition-colors"
        >
          {{ registerLoading ? "Creating account…" : "Create Account" }}
        </button>
      </form>
    </div>
  </div>
</template>
