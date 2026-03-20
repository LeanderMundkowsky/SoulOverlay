import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { commands } from "@/bindings";
import type {
  OrgSummary,
  OrgDetail,
  OrgRole,
  OrgPermissions,
  OrgMemberInfo,
  OrgInvitation,
  UserInvitation,
  OrgApplication,
  OrgInventoryEntry,
  OrgInventoryCollection,
  OrgLeadershipTransfer,
} from "@/bindings";

export type { OrgSummary, OrgDetail, OrgRole, OrgPermissions, OrgMemberInfo,
  OrgInvitation, UserInvitation, OrgApplication, OrgInventoryEntry, OrgInventoryCollection,
  OrgLeadershipTransfer };

export const useOrgStore = defineStore("org", () => {
  // ── My orgs ──────────────────────────────────────────────────────────────
  const myOrgs = ref<OrgSummary[]>([]);
  const loadingOrgs = ref(false);
  const orgsError = ref<string | null>(null);

  // ── Current org detail ────────────────────────────────────────────────────
  const currentOrgId = ref<number | null>(null);
  const currentOrgDetail = ref<OrgDetail | null>(null);
  const loadingDetail = ref(false);
  const detailError = ref<string | null>(null);

  // ── Roles (for current org) ───────────────────────────────────────────────
  const orgRoles = ref<OrgRole[]>([]);
  const loadingRoles = ref(false);
  const rolesError = ref<string | null>(null);

  // ── Sent invitations (for current org) ────────────────────────────────────
  const orgInvitations = ref<OrgInvitation[]>([]);
  const loadingOrgInvitations = ref(false);
  const orgInvitationsError = ref<string | null>(null);

  // ── Applications (for current org) ───────────────────────────────────────
  const orgApplications = ref<OrgApplication[]>([]);
  const loadingApplications = ref(false);
  const applicationsError = ref<string | null>(null);

  // ── Org inventory ─────────────────────────────────────────────────────────
  const orgInventory = ref<Map<number, OrgInventoryEntry[]>>(new Map());
  const orgCollections = ref<Map<number, OrgInventoryCollection[]>>(new Map());
  const loadingInventory = ref(false);
  const inventoryError = ref<string | null>(null);

  // ── User's incoming invitations ───────────────────────────────────────────
  const userInvitations = ref<UserInvitation[]>([]);
  const loadingUserInvitations = ref(false);
  const userInvitationsError = ref<string | null>(null);

  // ── Computed ─────────────────────────────────────────────────────────────
  const pendingInvitationCount = computed(
    () => userInvitations.value.filter((i) => i.status === "pending").length,
  );

  const currentOrgMyRole = computed(() => {
    if (!currentOrgId.value) return null;
    return myOrgs.value.find((o) => o.id === currentOrgId.value)?.my_role ?? null;
  });

  // ── Helpers ───────────────────────────────────────────────────────────────
  function can(permission: keyof OrgPermissions): boolean {
    const role = currentOrgMyRole.value;
    if (!role) return false;
    if (role.is_leader) return true;
    return role.permissions[permission];
  }

  // ── My orgs ──────────────────────────────────────────────────────────────
  async function loadMyOrgs(): Promise<void> {
    loadingOrgs.value = true;
    orgsError.value = null;
    const res = await commands.orgListMyOrgs();
    if (res.status === "ok") {
      myOrgs.value = res.data;
    } else {
      orgsError.value = res.error;
    }
    loadingOrgs.value = false;
  }

  async function createOrg(name: string, description: string | null): Promise<string | null> {
    const res = await commands.orgCreate(name, description);
    if (res.status === "ok") {
      myOrgs.value.push(res.data);
      return null;
    }
    return res.error;
  }

  async function updateOrg(id: number, name: string | null, description: string | null): Promise<string | null> {
    const res = await commands.orgUpdate(id, name, description);
    if (res.status === "ok") {
      const idx = myOrgs.value.findIndex((o) => o.id === id);
      if (idx >= 0) myOrgs.value[idx] = res.data;
      if (currentOrgDetail.value?.id === id) {
        currentOrgDetail.value = { ...currentOrgDetail.value, name: res.data.name, slug: res.data.slug, description: res.data.description };
      }
      return null;
    }
    return res.error;
  }

  async function deleteOrg(id: number): Promise<string | null> {
    const res = await commands.orgDelete(id);
    if (res.status === "ok") {
      myOrgs.value = myOrgs.value.filter((o) => o.id !== id);
      if (currentOrgId.value === id) {
        currentOrgId.value = null;
        currentOrgDetail.value = null;
      }
      return null;
    }
    return res.error;
  }

  // ── Org detail ─────────────────────────────────────────────────────────────
  async function loadOrgDetail(orgId: number): Promise<void> {
    loadingDetail.value = true;
    detailError.value = null;
    const res = await commands.orgGet(orgId);
    if (res.status === "ok") {
      currentOrgDetail.value = res.data;
      currentOrgId.value = orgId;
    } else {
      detailError.value = res.error;
    }
    loadingDetail.value = false;
  }

  function selectOrg(orgId: number): void {
    if (currentOrgId.value !== orgId) {
      currentOrgId.value = orgId;
      currentOrgDetail.value = null;
      orgRoles.value = [];
      orgInvitations.value = [];
      orgApplications.value = [];
    }
  }

  function clearCurrentOrg(): void {
    currentOrgId.value = null;
    currentOrgDetail.value = null;
    orgRoles.value = [];
    orgInvitations.value = [];
    orgApplications.value = [];
  }

  // ── Roles ──────────────────────────────────────────────────────────────────
  async function loadRoles(orgId: number): Promise<void> {
    loadingRoles.value = true;
    rolesError.value = null;
    const res = await commands.orgListRoles(orgId);
    if (res.status === "ok") {
      orgRoles.value = res.data;
    } else {
      rolesError.value = res.error;
    }
    loadingRoles.value = false;
  }

  async function createRole(orgId: number, name: string, permissions: OrgPermissions, sortOrder: number): Promise<string | null> {
    const res = await commands.orgCreateRole(orgId, name, permissions, sortOrder);
    if (res.status === "ok") {
      orgRoles.value.push(res.data);
      orgRoles.value.sort((a, b) => a.sort_order - b.sort_order);
      return null;
    }
    return res.error;
  }

  async function updateRole(orgId: number, roleId: number, name: string | null, permissions: OrgPermissions | null, sortOrder: number | null): Promise<string | null> {
    const res = await commands.orgUpdateRole(orgId, roleId, name, permissions, sortOrder);
    if (res.status === "ok") {
      const idx = orgRoles.value.findIndex((r) => r.id === roleId);
      if (idx >= 0) orgRoles.value[idx] = res.data;
      orgRoles.value.sort((a, b) => a.sort_order - b.sort_order);
      return null;
    }
    return res.error;
  }

  async function deleteRole(orgId: number, roleId: number): Promise<string | null> {
    const res = await commands.orgDeleteRole(orgId, roleId);
    if (res.status === "ok") {
      orgRoles.value = orgRoles.value.filter((r) => r.id !== roleId);
      return null;
    }
    return res.error;
  }

  // ── Members ────────────────────────────────────────────────────────────────
  async function updateMember(orgId: number, userId: number, roleId: number): Promise<string | null> {
    const res = await commands.orgUpdateMember(orgId, userId, roleId);
    if (res.status === "ok") {
      if (currentOrgDetail.value) {
        const idx = currentOrgDetail.value.members.findIndex((m) => m.user_id === userId);
        if (idx >= 0) currentOrgDetail.value.members[idx] = res.data;
      }
      return null;
    }
    return res.error;
  }

  async function removeMember(orgId: number, userId: number): Promise<string | null> {
    const res = await commands.orgRemoveMember(orgId, userId);
    if (res.status === "ok") {
      if (currentOrgDetail.value) {
        currentOrgDetail.value.members = currentOrgDetail.value.members.filter(
          (m) => m.user_id !== userId,
        );
        currentOrgDetail.value.member_count = Math.max(0, currentOrgDetail.value.member_count - 1);
      }
      // Remove from myOrgs if user left their own org
      const orgIdx = myOrgs.value.findIndex((o) => o.id === orgId);
      if (orgIdx >= 0 && currentOrgDetail.value?.members.every((m) => m.user_id !== userId)) {
        // Could be a self-leave; caller handles refresh
      }
      return null;
    }
    return res.error;
  }

  async function transferLeadership(
    orgId: number,
    targetUserId: number,
    newRoleId: number | null,
  ): Promise<OrgLeadershipTransfer | string> {
    const res = await commands.orgTransferLeadership(orgId, targetUserId, newRoleId);
    if (res.status === "ok") {
      // Update member roles in detail view
      if (currentOrgDetail.value) {
        const { new_leader, previous_leader } = res.data;
        for (const m of currentOrgDetail.value.members) {
          if (m.user_id === new_leader.user_id) m.role = new_leader.role;
          else if (m.user_id === previous_leader.user_id) m.role = previous_leader.role;
        }
      }
      // Refresh myOrgs so our own myRole updates
      await loadMyOrgs();
      return res.data;
    }
    return res.error;
  }

  // ── Org invitations ────────────────────────────────────────────────────────
  async function loadOrgInvitations(orgId: number): Promise<void> {
    loadingOrgInvitations.value = true;
    orgInvitationsError.value = null;
    const res = await commands.orgListInvitations(orgId);
    if (res.status === "ok") {
      orgInvitations.value = res.data;
    } else {
      orgInvitationsError.value = res.error;
    }
    loadingOrgInvitations.value = false;
  }

  async function createInvitation(orgId: number, username: string, roleId: number | null): Promise<string | null> {
    const res = await commands.orgCreateInvitation(orgId, username, roleId);
    if (res.status === "ok") {
      orgInvitations.value.unshift(res.data);
      return null;
    }
    return res.error;
  }

  async function cancelInvitation(orgId: number, invId: number): Promise<string | null> {
    const res = await commands.orgCancelInvitation(orgId, invId);
    if (res.status === "ok") {
      orgInvitations.value = orgInvitations.value.filter((i) => i.id !== invId);
      return null;
    }
    return res.error;
  }

  // ── User invitations (incoming) ────────────────────────────────────────────
  async function loadUserInvitations(): Promise<void> {
    loadingUserInvitations.value = true;
    userInvitationsError.value = null;
    const res = await commands.userListInvitations();
    if (res.status === "ok") {
      userInvitations.value = res.data;
    } else {
      userInvitationsError.value = res.error;
    }
    loadingUserInvitations.value = false;
  }

  async function acceptInvitation(id: number): Promise<string | null> {
    const res = await commands.userAcceptInvitation(id);
    if (res.status === "ok") {
      userInvitations.value = userInvitations.value.filter((i) => i.id !== id);
      myOrgs.value.push(res.data);
      return null;
    }
    return res.error;
  }

  async function declineInvitation(id: number): Promise<string | null> {
    const res = await commands.userDeclineInvitation(id);
    if (res.status === "ok") {
      userInvitations.value = userInvitations.value.filter((i) => i.id !== id);
      return null;
    }
    return res.error;
  }

  // ── Applications ───────────────────────────────────────────────────────────
  async function loadApplications(orgId: number): Promise<void> {
    loadingApplications.value = true;
    applicationsError.value = null;
    const res = await commands.orgListApplications(orgId);
    if (res.status === "ok") {
      orgApplications.value = res.data;
    } else {
      applicationsError.value = res.error;
    }
    loadingApplications.value = false;
  }

  async function createApplication(orgId: number, message: string | null): Promise<string | null> {
    const res = await commands.orgCreateApplication(orgId, message);
    if (res.status === "ok") return null;
    return res.error;
  }

  async function acceptApplication(orgId: number, appId: number, roleId: number | null): Promise<string | null> {
    const res = await commands.orgAcceptApplication(orgId, appId, roleId);
    if (res.status === "ok") {
      orgApplications.value = orgApplications.value.filter((a) => a.id !== appId);
      if (currentOrgDetail.value) currentOrgDetail.value.member_count++;
      return null;
    }
    return res.error;
  }

  async function rejectApplication(orgId: number, appId: number): Promise<string | null> {
    const res = await commands.orgRejectApplication(orgId, appId);
    if (res.status === "ok") {
      orgApplications.value = orgApplications.value.filter((a) => a.id !== appId);
      return null;
    }
    return res.error;
  }

  // ── Org inventory ──────────────────────────────────────────────────────────
  function getInventory(orgId: number): OrgInventoryEntry[] {
    return orgInventory.value.get(orgId) ?? [];
  }

  function getCollections(orgId: number): OrgInventoryCollection[] {
    return orgCollections.value.get(orgId) ?? [];
  }

  async function loadInventory(orgId: number): Promise<void> {
    loadingInventory.value = true;
    inventoryError.value = null;
    const [invRes, collRes] = await Promise.all([
      commands.orgListInventory(orgId),
      commands.orgListCollections(orgId),
    ]);
    if (invRes.status === "ok") {
      orgInventory.value.set(orgId, invRes.data);
    } else {
      inventoryError.value = invRes.error;
    }
    if (collRes.status === "ok") {
      orgCollections.value.set(orgId, collRes.data);
    }
    loadingInventory.value = false;
  }

  function setInventoryEntries(orgId: number, entries: OrgInventoryEntry[]): void {
    orgInventory.value.set(orgId, entries);
  }

  function upsertInventoryEntry(orgId: number, entry: OrgInventoryEntry): void {
    const entries = orgInventory.value.get(orgId) ?? [];
    const idx = entries.findIndex((e) => e.id === entry.id);
    if (idx >= 0) entries[idx] = entry;
    else entries.push(entry);
    orgInventory.value.set(orgId, entries);
  }

  function removeInventoryEntry(orgId: number, entryId: number): void {
    const entries = orgInventory.value.get(orgId) ?? [];
    orgInventory.value.set(orgId, entries.filter((e) => e.id !== entryId));
  }

  async function addInventoryEntry(
    orgId: number,
    entityId: string, entityName: string, entityKind: string,
    locationId: string, locationName: string, locationSlug: string,
    quantity: number, collectionIds: number[],
  ): Promise<string | null> {
    const res = await commands.orgAddInventoryEntry(orgId, entityId, entityName, entityKind, locationId, locationName, locationSlug, quantity, collectionIds);
    if (res.status === "ok") {
      upsertInventoryEntry(orgId, res.data);
      return null;
    }
    return res.error;
  }

  async function deleteInventoryEntry(orgId: number, entryId: number): Promise<string | null> {
    const res = await commands.orgDeleteInventoryEntry(orgId, entryId);
    if (res.status === "ok") {
      removeInventoryEntry(orgId, entryId);
      return null;
    }
    return res.error;
  }

  async function removeInventoryQuantity(orgId: number, entryId: number, quantity: number): Promise<string | null> {
    const res = await commands.orgRemoveInventoryQuantity(orgId, entryId, quantity);
    if (res.status === "ok") {
      if (res.data === null) removeInventoryEntry(orgId, entryId);
      else upsertInventoryEntry(orgId, res.data);
      return null;
    }
    return res.error;
  }

  async function transferInventory(orgId: number, entryId: number, quantity: number, targetLocationId: string, targetLocationName: string, targetLocationSlug: string): Promise<string | null> {
    const res = await commands.orgTransferInventory(orgId, entryId, quantity, targetLocationId, targetLocationName, targetLocationSlug);
    if (res.status === "ok") {
      if (res.data.source === null) removeInventoryEntry(orgId, entryId);
      else upsertInventoryEntry(orgId, res.data.source);
      upsertInventoryEntry(orgId, res.data.target);
      return null;
    }
    return res.error;
  }

  // ── Collections ────────────────────────────────────────────────────────────
  async function createCollection(orgId: number, name: string): Promise<string | null> {
    const res = await commands.orgCreateCollection(orgId, name);
    if (res.status === "ok") {
      const cols = orgCollections.value.get(orgId) ?? [];
      cols.push(res.data);
      orgCollections.value.set(orgId, cols);
      return null;
    }
    return res.error;
  }

  async function updateCollection(orgId: number, collId: number, name: string): Promise<string | null> {
    const res = await commands.orgUpdateCollection(orgId, collId, name);
    if (res.status === "ok") {
      const cols = orgCollections.value.get(orgId) ?? [];
      const idx = cols.findIndex((c) => c.id === collId);
      if (idx >= 0) cols[idx] = res.data;
      orgCollections.value.set(orgId, cols);
      return null;
    }
    return res.error;
  }

  async function deleteCollection(orgId: number, collId: number): Promise<string | null> {
    const res = await commands.orgDeleteCollection(orgId, collId);
    if (res.status === "ok") {
      const cols = orgCollections.value.get(orgId) ?? [];
      orgCollections.value.set(orgId, cols.filter((c) => c.id !== collId));
      return null;
    }
    return res.error;
  }

  function reset(): void {
    myOrgs.value = [];
    currentOrgId.value = null;
    currentOrgDetail.value = null;
    orgRoles.value = [];
    orgInvitations.value = [];
    orgApplications.value = [];
    orgInventory.value = new Map();
    orgCollections.value = new Map();
    userInvitations.value = [];
    orgsError.value = null;
  }

  return {
    // State
    myOrgs, loadingOrgs, orgsError,
    currentOrgId, currentOrgDetail, loadingDetail, detailError,
    orgRoles, loadingRoles, rolesError,
    orgInvitations, loadingOrgInvitations, orgInvitationsError,
    orgApplications, loadingApplications, applicationsError,
    userInvitations, loadingUserInvitations, userInvitationsError,
    loadingInventory, inventoryError,
    // Computed
    pendingInvitationCount, currentOrgMyRole,
    // Helpers
    can, getInventory, getCollections,
    // Actions
    loadMyOrgs, createOrg, updateOrg, deleteOrg,
    loadOrgDetail, selectOrg, clearCurrentOrg,
    loadRoles, createRole, updateRole, deleteRole,
    updateMember, removeMember, transferLeadership,
    loadOrgInvitations, createInvitation, cancelInvitation,
    loadUserInvitations, acceptInvitation, declineInvitation,
    loadApplications, createApplication, acceptApplication, rejectApplication,
    loadInventory, setInventoryEntries, upsertInventoryEntry, removeInventoryEntry,
    addInventoryEntry, deleteInventoryEntry, removeInventoryQuantity, transferInventory,
    createCollection, updateCollection, deleteCollection,
    reset,
  };
});
