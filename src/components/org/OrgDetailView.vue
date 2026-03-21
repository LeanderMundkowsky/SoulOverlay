<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useOrgStore } from "@/stores/org";
import OrgMembersPanel from "@/components/org/OrgMembersPanel.vue";
import OrgRolesPanel from "@/components/org/OrgRolesPanel.vue";
import OrgApplicationsPanel from "@/components/org/OrgApplicationsPanel.vue";
import OrgInventoryPanel from "@/components/org/OrgInventoryPanel.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";

const props = defineProps<{ orgId: number }>();
const emit = defineEmits<{ back: [] }>();
const orgStore = useOrgStore();

type SubTab = "members" | "roles" | "inventory" | "applications";
const activeTab = ref<SubTab>("members");

const canManageOrg = computed(() => orgStore.can("manage_org"));
const canSeeApplications = computed(() => orgStore.can("view_applications") || orgStore.can("manage_applications"));

const visibleTabs = computed<SubTab[]>(() => {
  const tabs: SubTab[] = ["members", "roles", "inventory"];
  if (canSeeApplications.value) tabs.push("applications");
  return tabs;
});

watch(canSeeApplications, (can) => {
  if (!can && activeTab.value === "applications") activeTab.value = "members";
});

watch(() => props.orgId, (id) => {
  orgStore.loadOrgDetail(id);
  orgStore.loadRoles(id);
}, { immediate: true });

const detail = computed(() => orgStore.currentOrgDetail);

// ── Org settings edit ─────────────────────────────────────────────────────
const editing = ref(false);
const editName = ref("");
const editDesc = ref("");
const editSaving = ref(false);
const editError = ref<string | null>(null);

function startEdit() {
  if (!detail.value) return;
  editName.value = detail.value.name;
  editDesc.value = detail.value.description ?? "";
  editError.value = null;
  editing.value = true;
}

async function saveEdit() {
  editSaving.value = true;
  editError.value = null;
  const err = await orgStore.updateOrg(props.orgId, editName.value.trim() || null, editDesc.value.trim() || null);
  editSaving.value = false;
  if (err) editError.value = err;
  else editing.value = false;
}

const deleteConfirm = ref(false);
const deleting = ref(false);
const deleteError = ref<string | null>(null);

async function confirmDelete() {
  deleting.value = true;
  deleteError.value = null;
  const err = await orgStore.deleteOrg(props.orgId);
  deleting.value = false;
  if (err) deleteError.value = err;
  else emit("back");
}

// Leave org
const myMembership = computed(() => orgStore.currentOrgMyRole);
</script>

<template>
  <div class="space-y-4">
    <!-- Back button + header -->
    <div class="flex items-start gap-3 bg-[#1a1d24] border border-white/10 rounded-xl px-4 py-3">
      <button @click="emit('back')" class="text-white/40 hover:text-white/80 transition-colors mt-0.5">
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
        </svg>
      </button>

      <LoadingSpinner v-if="orgStore.loadingDetail" class="py-8" />
      <AlertBanner v-else-if="orgStore.detailError" variant="error" :message="orgStore.detailError" />

      <div v-else-if="detail" class="flex-1 min-w-0">
        <template v-if="!editing">
          <div class="flex items-center gap-3">
            <h2 class="text-white font-semibold text-lg">{{ detail.name }}</h2>
            <span v-if="myMembership?.is_leader" class="text-yellow-400 text-xs">👑 Leader</span>
            <span v-else-if="myMembership" class="text-xs text-white/40">{{ myMembership.name }}</span>
          </div>
          <p v-if="detail.description" class="text-sm text-white/40 mt-1">{{ detail.description }}</p>
          <div class="flex items-center gap-4 mt-2 text-xs text-white/30">
            <span>{{ detail.member_count }} members</span>
            <button v-if="canManageOrg" @click="startEdit" class="text-teal-400/60 hover:text-teal-400 transition-colors">Edit org</button>
            <button v-if="canManageOrg" @click="deleteConfirm = true" class="text-red-400/60 hover:text-red-400 transition-colors">Delete org</button>
          </div>
        </template>

        <template v-else>
          <div class="space-y-2 max-w-md">
            <input v-model="editName" type="text" maxlength="100" class="w-full bg-[#111318] border border-white/10 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-teal-500/50" />
            <textarea v-model="editDesc" rows="2" maxlength="1000" class="w-full bg-[#111318] border border-white/10 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-teal-500/50 resize-none" />
            <AlertBanner v-if="editError" variant="error" :message="editError" />
            <div class="flex gap-2">
              <button @click="saveEdit" :disabled="editSaving" class="text-xs px-3 py-1.5 bg-teal-600 hover:bg-teal-500 text-white rounded-lg disabled:opacity-40 transition-colors">{{ editSaving ? "Saving…" : "Save" }}</button>
              <button @click="editing = false" class="text-xs text-white/40 hover:text-white/70 transition-colors">Cancel</button>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- Delete confirmation -->
    <div v-if="deleteConfirm" class="bg-[#2a1010] border border-red-500/30 rounded-lg p-4 space-y-3">
      <p class="text-sm text-red-300">Are you sure you want to delete this org? This cannot be undone.</p>
      <AlertBanner v-if="deleteError" variant="error" :message="deleteError" />
      <div class="flex gap-2">
        <button @click="confirmDelete" :disabled="deleting" class="text-xs px-3 py-1.5 bg-red-700 hover:bg-red-600 text-white rounded-lg disabled:opacity-40 transition-colors">{{ deleting ? "Deleting…" : "Yes, delete" }}</button>
        <button @click="deleteConfirm = false" class="text-xs text-white/40 hover:text-white/70 transition-colors">Cancel</button>
      </div>
    </div>

    <!-- Sub-tab bar -->
    <div v-if="detail" class="bg-[#1a1d24] border border-white/10 rounded-xl flex gap-1 px-2 py-2">
      <button
        v-for="tab in visibleTabs"
        :key="tab"
        @click="activeTab = tab"
        class="px-3 py-1.5 text-xs rounded-lg transition-colors capitalize"
        :class="activeTab === tab
          ? 'bg-[#111318] text-white'
          : 'text-white/40 hover:text-white/70'"
      >{{ tab }}</button>
    </div>

    <!-- Sub-tab content -->
    <template v-if="detail">
      <OrgMembersPanel v-show="activeTab === 'members'" :org-id="orgId" />
      <OrgRolesPanel v-show="activeTab === 'roles'" :org-id="orgId" />
      <OrgInventoryPanel v-show="activeTab === 'inventory'" :org-id="orgId" />
      <OrgApplicationsPanel v-show="activeTab === 'applications'" :org-id="orgId" />
    </template>
  </div>
</template>
