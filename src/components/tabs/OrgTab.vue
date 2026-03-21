<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useOrgStore } from "@/stores/org";
import type { OrgLookup } from "@/stores/org";
import { useBackendStore } from "@/stores/backend";
import OrgDetailView from "@/components/org/OrgDetailView.vue";
import OrgCreateModal from "@/components/org/OrgCreateModal.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";

const orgStore = useOrgStore();
const backendStore = useBackendStore();

const showCreateModal = ref(false);
const selectedOrgId = ref<number | null>(null);

// Apply/join
const applyHandle = ref("");
const applyMessage = ref("");
const applying = ref(false);
const applyError = ref<string | null>(null);
const applySent = ref(false);
const showApplyForm = ref(false);
const resolvedOrg = ref<OrgLookup | null>(null);
const resolving = ref(false);

const cleanHandle = computed(() => applyHandle.value.trim().replace(/^@/, ""));

async function resolveHandle() {
  if (!cleanHandle.value) return;
  resolving.value = true;
  applyError.value = null;
  resolvedOrg.value = null;
  const result = await orgStore.lookupOrgBySlug(cleanHandle.value);
  resolving.value = false;
  if (typeof result === "string") {
    applyError.value = result === "404" || result.toLowerCase().includes("not found")
      ? `No organization found with handle @${cleanHandle.value}`
      : result;
  } else {
    resolvedOrg.value = result;
  }
}

watch(() => backendStore.isLoggedIn, (loggedIn) => {
  if (loggedIn) {
    orgStore.loadMyOrgs();
    orgStore.loadUserInvitations();
  } else {
    orgStore.reset();
    selectedOrgId.value = null;
    showApplyForm.value = false;
    showCreateModal.value = false;
  }
}, { immediate: true });

// Refresh org data on every 30s backend-status tick
let unlistenStatus: UnlistenFn | null = null;
onMounted(async () => {
  unlistenStatus = await listen<{ ok: boolean }>("backend-status", (event) => {
    if (!event.payload.ok || !backendStore.isLoggedIn) return;
    orgStore.loadMyOrgs();
    orgStore.loadUserInvitations();
    if (orgStore.currentOrgId !== null) {
      orgStore.loadOrgDetail(orgStore.currentOrgId);
    }
  });
});
onUnmounted(() => {
  unlistenStatus?.();
  unlistenStatus = null;
});

function selectOrg(id: number) {
  orgStore.selectOrg(id);
  selectedOrgId.value = id;
}

function goBack() {
  selectedOrgId.value = null;
  orgStore.clearCurrentOrg();
}

async function acceptInvite(id: number) {
  await orgStore.acceptInvitation(id);
  if (orgStore.myOrgs.length > 0) {
    orgStore.loadMyOrgs();
  }
}

async function declineInvite(id: number) {
  await orgStore.declineInvitation(id);
}

async function applyToOrg() {
  if (!resolvedOrg.value) return;
  applying.value = true;
  applyError.value = null;
  applySent.value = false;
  const err = await orgStore.createApplication(resolvedOrg.value.id, applyMessage.value.trim() || null);
  applying.value = false;
  if (err) applyError.value = err;
  else {
    applySent.value = true;
    applyHandle.value = "";
    applyMessage.value = "";
    resolvedOrg.value = null;
    setTimeout(() => applySent.value = false, 3000);
  }
}

const refreshing = ref(false);
async function refresh() {
  if (refreshing.value) return;
  refreshing.value = true;
  await Promise.all([
    orgStore.loadMyOrgs(),
    orgStore.loadUserInvitations(),
    ...(orgStore.currentOrgId !== null ? [orgStore.loadOrgDetail(orgStore.currentOrgId)] : []),
  ]);
  refreshing.value = false;
}
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto w-full space-y-4">
    <!-- Not logged in -->
    <div v-if="!backendStore.isLoggedIn" class="flex flex-col items-center justify-center py-16 text-white/40 space-y-2">
      <svg class="w-12 h-12 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1">
        <path stroke-linecap="round" stroke-linejoin="round" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
      <p class="text-sm">Organizations require an account</p>
    </div>

    <template v-else>
      <!-- Detail view when org selected -->
      <OrgDetailView v-if="selectedOrgId" :org-id="selectedOrgId" @back="goBack" />

      <template v-else>
        <!-- Pending invitations banner -->
        <div v-if="orgStore.userInvitations.filter(i => i.status === 'pending').length > 0" class="space-y-2">
          <h3 class="text-xs text-white/50 uppercase tracking-wider font-semibold">Pending Invitations</h3>
          <div
            v-for="inv in orgStore.userInvitations.filter(i => i.status === 'pending')"
            :key="inv.id"
            class="flex items-center justify-between bg-[#0d1a18] border border-teal-500/20 rounded-lg px-4 py-3"
          >
            <div>
              <span class="text-sm text-white font-medium">{{ inv.org.name }}</span>
              <span class="text-xs text-white/30 ml-2">from {{ inv.invited_by.username }}</span>
            </div>
            <div class="flex gap-2">
              <button @click="acceptInvite(inv.id)" class="text-xs px-3 py-1.5 bg-teal-600 hover:bg-teal-500 text-white rounded-lg transition-colors">Accept</button>
              <button @click="declineInvite(inv.id)" class="text-xs text-white/30 hover:text-red-400 transition-colors px-2">Decline</button>
            </div>
          </div>
        </div>

        <!-- My orgs -->
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-xs text-white/50 uppercase tracking-wider font-semibold">My Organizations</h3>
            <div class="flex gap-2 items-center">
              <button
                @click="refresh"
                :disabled="refreshing"
                class="p-1.5 bg-[#1a1d24] border border-white/10 hover:border-white/20 text-white/40 hover:text-white/70 disabled:opacity-40 rounded-lg transition-colors"
                title="Refresh"
              >
                <svg
                  class="w-3.5 h-3.5"
                  :class="{ 'animate-spin': refreshing }"
                  fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              </button>
              <button
                @click="showApplyForm = !showApplyForm"
                class="text-xs px-3 py-1.5 bg-[#1a1d24] border border-white/10 hover:border-white/20 text-white/60 hover:text-white/90 rounded-lg transition-colors"
              >Apply / Join</button>
              <button
                @click="showCreateModal = true"
                class="text-xs px-3 py-1.5 bg-teal-600 hover:bg-teal-500 text-white rounded-lg transition-colors"
              >+ Create Org</button>
            </div>
          </div>

          <!-- Apply form -->
          <div v-if="showApplyForm" class="bg-[#1a1d24] border border-white/10 rounded-lg p-4 space-y-3">
            <h4 class="text-xs text-white/50 uppercase tracking-wider">Apply to Organization</h4>
            <div class="flex gap-2">
              <div class="flex-1">
                <label class="block text-xs text-white/40 mb-1">Org Handle</label>
                <input
                  v-model="applyHandle"
                  type="text"
                  placeholder="@test-org"
                  @keydown.enter="resolveHandle"
                  @input="resolvedOrg = null; applyError = null"
                  class="w-full bg-[#111318] border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-white/20 focus:outline-none focus:border-teal-500/50"
                />
              </div>
              <div class="flex items-end">
                <button
                  @click="resolveHandle"
                  :disabled="!cleanHandle || resolving"
                  class="text-xs px-3 py-2 bg-[#111318] border border-white/10 hover:border-teal-500/50 text-white/60 hover:text-white rounded-lg disabled:opacity-40 transition-colors"
                >{{ resolving ? "…" : "Look up" }}</button>
              </div>
            </div>

            <!-- Resolved org preview -->
            <div v-if="resolvedOrg" class="bg-[#111318] border border-teal-500/20 rounded-lg px-3 py-2 space-y-0.5">
              <div class="flex items-center gap-2">
                <span class="text-sm text-white font-medium">{{ resolvedOrg.name }}</span>
                <span class="text-xs text-white/30">@{{ resolvedOrg.slug }}</span>
                <span class="text-xs text-white/20">· {{ resolvedOrg.member_count }} member{{ resolvedOrg.member_count !== 1 ? "s" : "" }}</span>
              </div>
              <p v-if="resolvedOrg.description" class="text-xs text-white/40 truncate">{{ resolvedOrg.description }}</p>
            </div>

            <div v-if="resolvedOrg">
              <label class="block text-xs text-white/40 mb-1">Message <span class="text-white/20">(optional)</span></label>
              <textarea v-model="applyMessage" rows="2" placeholder="Why do you want to join?" class="w-full bg-[#111318] border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-white/20 focus:outline-none focus:border-teal-500/50 resize-none" />
            </div>
            <AlertBanner v-if="applyError" variant="error" :message="applyError" />
            <p v-if="applySent" class="text-xs text-teal-400">Application sent!</p>
            <button
              v-if="resolvedOrg"
              @click="applyToOrg"
              :disabled="applying"
              class="text-xs px-4 py-2 bg-teal-600 hover:bg-teal-500 text-white rounded-lg disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            >{{ applying ? "Sending…" : "Send Application" }}</button>
          </div>

          <LoadingSpinner v-if="orgStore.loadingOrgs" class="py-8" />
          <AlertBanner v-else-if="orgStore.orgsError" variant="error" :message="orgStore.orgsError" />

          <div v-else-if="orgStore.myOrgs.length === 0" class="text-center text-white/30 py-8 text-sm">
            <p>You're not in any organizations yet.</p>
            <p class="mt-1">Create one or apply to join an existing org.</p>
          </div>

          <div v-else class="grid grid-cols-1 gap-3">
            <button
              v-for="org in orgStore.myOrgs"
              :key="org.id"
              @click="selectOrg(org.id)"
              class="text-left bg-[#1a1d24] border border-white/10 rounded-xl p-4 hover:border-teal-500/30 transition-colors group"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <span v-if="org.my_role?.is_leader" class="text-yellow-400">👑</span>
                  <span class="text-white font-medium">{{ org.name }}</span>
                  <span class="text-xs text-white/30">@{{ org.slug }}</span>
                </div>
                <div class="flex items-center gap-2 text-xs text-white/40">
                  <span>{{ org.member_count }} members</span>
                  <svg class="w-3.5 h-3.5 text-white/20 group-hover:text-white/50 transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
                  </svg>
                </div>
              </div>
              <div v-if="org.description" class="text-xs text-white/40 mt-1.5 line-clamp-2">{{ org.description }}</div>
              <div v-if="org.my_role" class="text-xs text-white/30 mt-1">Role: {{ org.my_role.name }}</div>
            </button>
          </div>
        </div>
      </template>
    </template>

    <OrgCreateModal v-if="showCreateModal" @close="showCreateModal = false" />
  </div>
</template>
