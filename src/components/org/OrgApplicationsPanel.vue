<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useOrgStore } from "@/stores/org";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";

const props = defineProps<{ orgId: number }>();
const orgStore = useOrgStore();

const actionError = ref<string | null>(null);
const acceptingId = ref<number | null>(null);
const acceptRoleId = ref<Record<number, number | null>>({});

const canView = computed(() => orgStore.can("view_applications") || orgStore.can("manage_applications"));
const canManage = computed(() => orgStore.can("manage_applications"));

watch(() => props.orgId, () => {
  if (canView.value) orgStore.loadApplications(props.orgId);
}, { immediate: true });

function toggleAccept(appId: number) {
  acceptingId.value = acceptingId.value === appId ? null : appId;
}

async function accept(appId: number) {
  actionError.value = null;
  const roleId = acceptRoleId.value[appId] ?? null;
  const err = await orgStore.acceptApplication(props.orgId, appId, roleId);
  if (err) actionError.value = err;
  else acceptingId.value = null;
}

async function reject(appId: number) {
  actionError.value = null;
  const err = await orgStore.rejectApplication(props.orgId, appId);
  if (err) actionError.value = err;
}
</script>

<template>
  <div class="bg-[#1a1d24] border border-white/10 rounded-xl p-4 space-y-3">
    <div class="flex items-center justify-between">
      <h3 class="text-xs text-white/50 uppercase tracking-wider">Applications</h3>
    </div>

    <AlertBanner v-if="actionError" variant="error" :message="actionError" />

    <div v-if="!canView" class="text-xs text-white/30 py-2">You don't have permission to view applications.</div>
    <template v-else>
      <LoadingSpinner v-if="orgStore.loadingApplications" class="py-8" />
      <AlertBanner v-else-if="orgStore.applicationsError" variant="error" :message="orgStore.applicationsError" />
      <div v-else-if="!orgStore.orgApplications.length" class="text-xs text-white/30 py-2">No pending applications.</div>
      <div v-else class="space-y-2">
        <div
          v-for="app in orgStore.orgApplications"
          :key="app.id"
          class="bg-[#111318] border border-white/10 rounded-lg p-3 space-y-2"
        >
          <div class="flex items-center justify-between">
            <div>
              <span class="text-sm text-white">{{ (app.applicant?.username ?? "Unknown") }}</span>
              <span v-if="app.message" class="block text-xs text-white/40 mt-0.5">{{ app.message }}</span>
            </div>
            <div v-if="canManage" class="flex items-center gap-2">
              <button
                @click="toggleAccept(app.id)"
                class="text-xs px-2 py-1 bg-teal-600 hover:bg-teal-500 text-white rounded transition-colors"
              >Accept</button>
              <button
                @click="reject(app.id)"
                class="text-xs text-white/30 hover:text-red-400 transition-colors"
              >Reject</button>
            </div>
          </div>

          <!-- Role picker shown when accepting -->
          <div v-if="acceptingId === app.id" class="flex items-center gap-2 pt-1">
            <select
              v-model="acceptRoleId[app.id]"
              class="flex-1 bg-[#111318] border border-white/10 rounded px-2 py-1.5 text-xs text-white focus:outline-none"
            >
              <option :value="null" class="bg-[#1a1d24]">Assign role later</option>
              <option v-for="r in orgStore.orgRoles.filter(r => !r.is_leader)" :key="r.id" :value="r.id" class="bg-[#1a1d24]">{{ r.name }}</option>
            </select>
            <button
              @click="accept(app.id)"
              class="text-xs px-3 py-1.5 bg-teal-600 hover:bg-teal-500 text-white rounded transition-colors"
            >Confirm</button>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
