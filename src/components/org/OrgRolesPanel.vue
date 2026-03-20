<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useOrgStore } from "@/stores/org";
import type { OrgRole, OrgPermissions } from "@/stores/org";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";

const props = defineProps<{ orgId: number }>();
const orgStore = useOrgStore();

const roleDialog = ref<null | { mode: "create" | "edit"; roleId?: number; name: string; permissions: Record<string, boolean>; sortOrder: number }>(null);
const saving = ref(false);
const saveError = ref<string | null>(null);
const deleteError = ref<string | null>(null);

const PERMISSIONS = [
  { key: "manage_org", label: "Manage Org", desc: "Edit name/description, delete org" },
  { key: "manage_members", label: "Manage Members", desc: "Kick members, change roles" },
  { key: "manage_roles", label: "Manage Roles", desc: "Create/edit/delete roles" },
  { key: "invite_members", label: "Invite Members", desc: "Send invitations to players" },
  { key: "view_applications", label: "View Applications", desc: "See pending applications" },
  { key: "manage_applications", label: "Manage Applications", desc: "Accept/reject applications" },
  { key: "manage_inventory", label: "Manage Inventory", desc: "Add/edit/delete inventory entries" },
  { key: "manage_collections", label: "Manage Collections", desc: "Create/rename/delete collections" },
] as const;

const canManageRoles = computed(() => orgStore.can("manage_roles"));

watch(() => props.orgId, () => { orgStore.loadRoles(props.orgId); }, { immediate: true });

function openCreate() {
  const defaultPerms: Record<string, boolean> = {};
  for (const p of PERMISSIONS) defaultPerms[p.key] = false;
  roleDialog.value = { mode: "create", name: "", permissions: defaultPerms, sortOrder: (orgStore.orgRoles.length + 1) * 10 };
  saveError.value = null;
}

function openEdit(role: OrgRole) {
  if (role.is_leader) return;
  const perms: Record<string, boolean> = { ...role.permissions };
  roleDialog.value = { mode: "edit", roleId: role.id, name: role.name, permissions: perms, sortOrder: role.sort_order };
  saveError.value = null;
}

async function saveRole() {
  if (!roleDialog.value || !roleDialog.value.name.trim()) return;
  saving.value = true;
  saveError.value = null;
  const d = roleDialog.value;
  const perms = d.permissions as OrgPermissions;
  let err: string | null;
  if (d.mode === "create") {
    err = await orgStore.createRole(props.orgId, d.name.trim(), perms, d.sortOrder);
  } else {
    err = await orgStore.updateRole(props.orgId, d.roleId!, d.name.trim(), perms, d.sortOrder);
  }
  saving.value = false;
  if (err) saveError.value = err;
  else roleDialog.value = null;
}

async function deleteRole(roleId: number) {
  deleteError.value = null;
  const err = await orgStore.deleteRole(props.orgId, roleId);
  if (err) deleteError.value = err;
}
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <h3 class="text-xs text-white/50 uppercase tracking-wider">Roles</h3>
      <button
        v-if="canManageRoles"
        @click="openCreate"
        class="text-xs text-teal-400 hover:text-teal-300 transition-colors"
      >+ New Role</button>
    </div>

    <AlertBanner v-if="deleteError" variant="error" :message="deleteError" />

    <LoadingSpinner v-if="orgStore.loadingRoles" class="py-8" />
    <AlertBanner v-else-if="orgStore.rolesError" variant="error" :message="orgStore.rolesError" />

    <div v-else class="space-y-2">
      <div
        v-for="role in orgStore.orgRoles"
        :key="role.id"
        class="bg-[#1a1d24] border rounded-lg p-3 space-y-2"
        :class="role.is_leader ? 'border-yellow-500/30' : 'border-white/10'"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span v-if="role.is_leader" class="text-yellow-400 text-xs">👑</span>
            <span class="text-sm text-white font-medium">{{ role.name }}</span>
            <span v-if="role.is_leader" class="text-xs text-yellow-400/60">Leader</span>
          </div>
          <div class="flex items-center gap-2 text-xs text-white/40">
            <span>{{ role.member_count }} member{{ role.member_count !== 1 ? "s" : "" }}</span>
            <button
              v-if="canManageRoles && !role.is_leader"
              @click="openEdit(role)"
              class="text-white/40 hover:text-teal-400 transition-colors"
            >Edit</button>
            <button
              v-if="canManageRoles && !role.is_leader"
              @click="deleteRole(role.id)"
              class="text-white/40 hover:text-red-400 transition-colors"
            >Delete</button>
          </div>
        </div>

        <!-- Permission pills -->
        <div class="flex flex-wrap gap-1">
          <template v-for="p in PERMISSIONS" :key="p.key">
            <span
              v-if="role.is_leader || role.permissions[p.key as keyof typeof role.permissions]"
              class="text-xs px-2 py-0.5 rounded-full"
              :class="role.is_leader ? 'bg-[#2a2210] text-yellow-400/70' : 'bg-[#0d1a18] text-teal-400/70'"
            >{{ p.label }}</span>
          </template>
          <span
            v-if="!role.is_leader && !PERMISSIONS.some(p => role.permissions[p.key as keyof typeof role.permissions])"
            class="text-xs text-white/20"
          >No permissions</span>
        </div>
      </div>
    </div>

    <!-- Role edit/create dialog -->
    <div v-if="roleDialog" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60" @click.self="roleDialog = null">
      <div class="bg-[#1a1d24] border border-white/10 rounded-xl w-full max-w-md mx-4 p-6 space-y-4 max-h-[90vh] overflow-y-auto">
        <h3 class="text-white font-semibold text-sm uppercase tracking-wider">
          {{ roleDialog.mode === "create" ? "New Role" : "Edit Role" }}
        </h3>

        <div class="space-y-3">
          <div>
            <label class="block text-xs text-white/50 mb-1">Role Name</label>
            <input
              v-model="roleDialog.name"
              type="text"
              maxlength="80"
              placeholder="Commander"
              class="w-full bg-[#111318] border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-white/20 focus:outline-none focus:border-teal-500/50"
            />
          </div>

          <div>
            <label class="block text-xs text-white/50 mb-2">Permissions</label>
            <div class="space-y-2">
              <label v-for="p in PERMISSIONS" :key="p.key" class="flex items-start gap-3 cursor-pointer group">
                <input
                  type="checkbox"
                  :checked="roleDialog.permissions[p.key]"
                  @change="roleDialog!.permissions[p.key] = ($event.target as HTMLInputElement).checked"
                  class="mt-0.5 rounded border-white/20 bg-[#111318] text-teal-500 focus:ring-teal-500/30 cursor-pointer"
                />
                <div>
                  <div class="text-xs text-white group-hover:text-white/80">{{ p.label }}</div>
                  <div class="text-xs text-white/30">{{ p.desc }}</div>
                </div>
              </label>
            </div>
          </div>

          <div>
            <label class="block text-xs text-white/50 mb-1">Sort Order</label>
            <input
              v-model.number="roleDialog.sortOrder"
              type="number"
              class="w-full bg-[#111318] border border-white/10 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-teal-500/50"
            />
          </div>

          <AlertBanner v-if="saveError" variant="error" :message="saveError" />
        </div>

        <div class="flex gap-2 justify-end">
          <button @click="roleDialog = null" class="px-4 py-2 text-xs text-white/50 hover:text-white/80 transition-colors">Cancel</button>
          <button
            @click="saveRole"
            :disabled="!roleDialog.name.trim() || saving"
            class="px-4 py-2 text-xs bg-teal-600 hover:bg-teal-500 text-white rounded-lg disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >{{ saving ? "Saving…" : "Save" }}</button>
        </div>
      </div>
    </div>
  </div>
</template>
