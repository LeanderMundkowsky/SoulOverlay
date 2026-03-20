import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/bindings";
import type { BackendAccount } from "@/bindings";

export const useBackendStore = defineStore("backend", () => {
  const account = ref<BackendAccount | null>(null);
  const tokenPresent = ref(false);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Backend status banner
  const backendOnline = ref(true);
  const statusBannerDismissed = ref(false);

  const isLoggedIn = computed(() => account.value !== null);
  const showReloginHint = computed(() => tokenPresent.value && !isLoggedIn.value);
  const showStatusBanner = computed(() => !backendOnline.value && !statusBannerDismissed.value);

  async function initialize() {
    // Restore session state from Rust (account was fetched on startup if token was present)
    const result = await commands.backendGetAccount();
    if (result.status === "error") return;
    account.value = result.data.account;
    tokenPresent.value = result.data.token_present;

    // Listen for backend status events from the 30s timer
    await listen<{ ok: boolean }>("backend-status", (event) => {
      backendOnline.value = event.payload.ok;
      // If backend comes back online, reset dismissed flag so the banner goes away naturally
      if (event.payload.ok) {
        statusBannerDismissed.value = false;
      }
    });
  }

  async function login(username: string, password: string): Promise<string | null> {
    loading.value = true;
    error.value = null;
    try {
      const result = await commands.backendLogin(username, password);
      if (result.status === "error") {
        error.value = result.error;
        return result.error;
      }
      account.value = result.data.account;
      tokenPresent.value = true;
      return null;
    } catch (e) {
      const msg = String(e);
      error.value = msg;
      return msg;
    } finally {
      loading.value = false;
    }
  }

  async function register(username: string, email: string, password: string): Promise<string | null> {
    loading.value = true;
    error.value = null;
    try {
      const result = await commands.backendRegister(username, email, password);
      if (result.status === "error") {
        error.value = result.error;
        return result.error;
      }
      account.value = result.data.account;
      tokenPresent.value = true;
      return null;
    } catch (e) {
      const msg = String(e);
      error.value = msg;
      return msg;
    } finally {
      loading.value = false;
    }
  }

  async function logout() {
    const result = await commands.backendLogout();
    if (result.status === "ok") {
      account.value = null;
      tokenPresent.value = false;
    }
  }

  async function updateSecretKey(secretKey: string | null): Promise<string | null> {
    try {
      const result = await commands.backendUpdateSecretKey(secretKey);
      if (result.status === "error") return result.error;
      account.value = result.data;
      return null;
    } catch (e) {
      return String(e);
    }
  }

  function dismissStatusBanner() {
    statusBannerDismissed.value = true;
  }

  return {
    account,
    tokenPresent,
    loading,
    error,
    backendOnline,
    isLoggedIn,
    showReloginHint,
    showStatusBanner,
    initialize,
    login,
    register,
    logout,
    updateSecretKey,
    dismissStatusBanner,
  };
});
