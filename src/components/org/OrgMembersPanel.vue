<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useOrgStore } from "@/stores/org";
import { useBackendStore } from "@/stores/backend";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";

const props = defineProps<{ orgId: number }>();
const orgStore = useOrgStore();
const backendStore = useBackendStore();

const inviteUsername = ref("");
const inviteRoleId = ref<number | null>(null);
const sending = ref(false);
const inviteError = ref<string | null>(null);
const inviteSent = ref(false);
const cancelError = ref<string | null>(null);
const changeRoleError = ref<string | null>(null);

// Transfer leadership state
const transferTargetId = ref<number | null>(null);
const transferNewRoleId = ref<number | null>(null);
const transferring = ref(false);
const transferError = ref<string | null>(null);
const showTransferConfirm = ref(false);

const canInvite = computed(() => orgStore.can("invite_members"));
const canManageMembers = computed(() => orgStore.can("manage_members"));
const amLeader = computed(() => orgStore.currentOrgMyRole?.is_leader === true);

watch(() => props.orgId, () => { orgStore.loadOrgInvitations(props.orgId); }, { immediate: true });

async function sendInvite() {
  if (!inviteUsername.value.trim()) return;
  sending.value = true;
  inviteError.value = null;
  inviteSent.value = false;
  const err = await orgStore.createInvitation(props.orgId, inviteUsername.value.trim(), inviteRoleId.value);
  sending.value = false;
  if (err) inviteError.value = err;
  else { inviteSent.value = true; inviteUsername.value = ""; inviteRoleId.value = null; setTimeout(() => inviteSent.value = false, 3000); }
}

async function cancelInvite(invitationId: number) {
  cancelError.value = null;
  const err = await orgStore.cancelInvitation(props.orgId, invitationId);
  if (err) cancelError.value = err;
}

async function changeRole(userId: number, roleId: number) {
  changeRoleError.value = null;
  const err = await orgStore.updateMember(props.orgId, userId, roleId);
  if (err) changeRoleError.value = err;
}

async function kickMember(userId: number) {
  const err = await orgStore.removeMember(props.orgId, userId);
  if (err) changeRoleError.value = err;
}

function openTransferConfirm(userId: number) {
  transferTargetId.value = userId;
  transferNewRoleId.value = null;
  transferError.value = null;
  showTransferConfirm.value = true;
}

async function confirmTransfer() {
  if (!transferTargetId.value) return;
  transferring.value = true;
  transferError.value = null;
  const result = await orgStore.transferLeadership(props.orgId, transferTargetId.value, transferNewRoleId.value);
  transferring.value = false;
  if (typeof result === "string") {
    transferError.value = result;
  } else {
    showTransferConfirm.value = false;
    transferTargetId.value = null;
  }
}

const myUserId = computed(() => backendStore.account?.id);
const detail = computed(() => orgStore.currentOrgDetail);
const transferTargetName = computed(() =>
  detail.value?.members.find((m) => m.user_id === transferTargetId.value)?.username ?? ""
);
</script>

<template>
  <div class="space-y-4">
    <!-- Transfer Leadership confirm overlay -->
    <div
      v-if="showTransferConfirm"
      class="bg-yellow-500/10 border border-yellow-500/40 rounded-lg p-4 space-y-3"
    >
      <h4 class="text-sm text-yellow-300 font-medium">Transfer Leadership to {{ transferTargetName }}?</h4>
      <p class="text-xs text-white/50">You will lose the leader role. This cannot be undone without the new leader's cooperation.</p>
      <div v-if="orgStore.orgRoles.filter(r => !r.is_leader).length">
        <label class="block text-xs text-white/40 mb-1">Assign yourself a role after transfer <span class="text-white/20">(optional)</span></label>
        <select
          v-model="transferNewRoleId"
          class="bg-white/5 border border-white/10 rounded-lg px-2 py-1.5 text-xs text-white focus:outline-none w-full"
        >
          <option :value="null" class="bg-[#1a1a2e]">Default (first non-leader role)</option>
          <option v-for="r in orgStore.orgRoles.filter(r => !r.is_leader)" :key="r.id" :value="r.id" class="bg-[#1a1a2e]">{{ r.name }}</option>
        </select>
      </div>
      <AlertBanner v-if="transferError" variant="error" :message="transferError" />
      <div class="flex gap-2">
        <button
          @click="confirmTransfer"
          :disabled="transferring"
          class="text-xs px-4 py-2 bg-yellow-500/20 border border-yellow-500/40 text-yellow-300 rounded-lg hover:bg-yellow-500/30 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >{{ transferring ? "Transferring…" : "Confirm Transfer" }}</button>
        <button
          @click="showTransferConfirm = false"
          :disabled="transferring"
          class="text-xs px-3 py-2 text-white/40 hover:text-white/70 transition-colors"
        >Cancel</button>
      </div>
    </div>

    <!-- Invite form -->
    <div v-if="canInvite" class="bg-white/5 border border-white/10 rounded-lg p-4 space-y-3">
      <h4 class="text-xs text-white/50 uppercase tracking-wider">Invite Player</h4>
      <div class="flex gap-2">
        <input
          v-model="inviteUsername"
          type="text"
          placeholder="Username"
          class="flex-1 bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-white/20 focus:outline-none focus:border-teal-500/50"
          @keydown.enter="sendInvite"
        />
        <select
          v-if="orgStore.orgRoles.length"
          v-model="inviteRoleId"
          class="bg-white/5 border border-white/10 rounded-lg px-2 py-2 text-sm text-white focus:outline-none"
        >
          <option :value="null" class="bg-[#1a1a2e]">No role</option>
          <option v-for="r in orgStore.orgRoles.filter(r => !r.is_leader)" :key="r.id" :value="r.id" class="bg-[#1a1a2e]">{{ r.name }}</option>
        </select>
        <button
          @click="sendInvite"
          :disabled="!inviteUsername.trim() || sending"
          class="px-3 py-2 text-xs bg-teal-500/20 border border-teal-500/40 text-teal-300 rounded-lg hover:bg-teal-500/30 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >{{ sending ? "…" : "Invite" }}</button>
      </div>
      <p v-if="inviteError" class="text-xs text-red-400">{{ inviteError }}</p>
      <p v-if="inviteSent" class="text-xs text-teal-400">Invitation sent!</p>
    </div>

    <!-- Pending sent invitations -->
    <div class="space-y-2">
      <div class="flex items-center justify-between">
        <h4 class="text-xs text-white/50 uppercase tracking-wider">Pending Invitations</h4>
      </div>
      <AlertBanner v-if="cancelError" variant="error" :message="cancelError" />
      <LoadingSpinner v-if="orgStore.loadingOrgInvitations" class="py-4" />
      <AlertBanner v-else-if="orgStore.orgInvitationsError" variant="error" :message="orgStore.orgInvitationsError" />
      <div v-else-if="!orgStore.orgInvitations.length" class="text-xs text-white/30 py-2">No pending invitations.</div>
      <div v-else class="space-y-1.5">
        <div
          v-for="inv in orgStore.orgInvitations"
          :key="inv.id"
          class="flex items-center justify-between bg-white/5 border border-white/10 rounded-lg px-3 py-2"
        >
          <div>
            <span class="text-sm text-white">{{ inv.invited_user.username }}</span>
            </div>
          <button
            v-if="canInvite"
            @click="cancelInvite(inv.id)"
            class="text-xs text-white/30 hover:text-red-400 transition-colors"
          >Cancel</button>
        </div>
      </div>
    </div>

    <!-- Members list -->
    <div v-if="detail" class="space-y-2">
      <h4 class="text-xs text-white/50 uppercase tracking-wider">Members ({{ detail.member_count }})</h4>
      <AlertBanner v-if="changeRoleError" variant="error" :message="changeRoleError" />
      <div class="space-y-1.5">
        <div
          v-for="member in detail.members"
          :key="member.user_id"
          class="flex items-center justify-between bg-white/5 border border-white/10 rounded-lg px-3 py-2"
        >
          <div class="flex items-center gap-2">
            <span v-if="member.role.is_leader" class="text-yellow-400 text-xs">👑</span>
            <span class="text-sm text-white">{{ member.username }}</span>
            <span class="text-xs text-white/40">{{ member.role.name }}</span>
          </div>
          <div v-if="member.user_id !== myUserId" class="flex items-center gap-2">
            <!-- Transfer leadership (leader-only, for non-leader members) -->
            <button
              v-if="amLeader && !member.role.is_leader"
              @click="openTransferConfirm(member.user_id)"
              class="text-xs text-yellow-400/50 hover:text-yellow-300 transition-colors"
              title="Transfer leadership"
            >👑 Transfer</button>
            <!-- Role change -->
            <template v-if="canManageMembers && !member.role.is_leader">
              <select
                :value="member.role.id"
                @change="changeRole(member.user_id, Number(($event.target as HTMLSelectElement).value))"
                class="bg-white/5 border border-white/10 rounded px-2 py-1 text-xs text-white focus:outline-none"
              >
                <option v-for="r in orgStore.orgRoles.filter(r => !r.is_leader)" :key="r.id" :value="r.id" class="bg-[#1a1a2e]">{{ r.name }}</option>
              </select>
              <button @click="kickMember(member.user_id)" class="text-xs text-white/30 hover:text-red-400 transition-colors">Kick</button>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
