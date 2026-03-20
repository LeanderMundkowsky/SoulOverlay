use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::commands::backend::{extract_error_message, http_client};
use crate::constants::BACKEND_URL;
use crate::state::AppState;

// ── IPC types (exposed to frontend via specta) ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct OrgPermissions {
    pub manage_org: bool,
    pub manage_members: bool,
    pub manage_roles: bool,
    pub invite_members: bool,
    pub view_applications: bool,
    pub manage_applications: bool,
    pub manage_inventory: bool,
    pub manage_collections: bool,
}

impl Default for OrgPermissions {
    fn default() -> Self {
        Self {
            manage_org: false,
            manage_members: false,
            manage_roles: false,
            invite_members: false,
            view_applications: false,
            manage_applications: false,
            manage_inventory: false,
            manage_collections: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgUserRef {
    pub id: u32,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgRef {
    pub id: u32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgRoleRef {
    pub id: u32,
    pub name: String,
    pub is_leader: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgRoleWithPerms {
    pub id: u32,
    pub name: String,
    pub is_leader: bool,
    pub permissions: OrgPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgRole {
    pub id: u32,
    pub name: String,
    pub is_leader: bool,
    pub permissions: OrgPermissions,
    pub sort_order: i32,
    pub member_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgMemberInfo {
    pub user_id: u32,
    pub username: String,
    pub role: OrgRoleRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgSummary {
    pub id: u32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub member_count: u32,
    pub my_role: OrgRoleWithPerms,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgDetail {
    pub id: u32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub members: Vec<OrgMemberInfo>,
    pub roles: Vec<OrgRole>,
    pub member_count: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgInvitation {
    pub id: u32,
    pub org_id: u32,
    pub org_name: String,
    pub invited_user: OrgUserRef,
    pub invited_by: OrgUserRef,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UserInvitation {
    pub id: u32,
    pub org: OrgRef,
    pub invited_by: OrgUserRef,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgApplication {
    pub id: u32,
    pub org_id: u32,
    pub applicant: Option<OrgUserRef>,
    pub message: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgInventoryCollection {
    pub id: u32,
    pub name: String,
    pub created_by: OrgUserRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgInventoryEntry {
    pub id: u32,
    pub entity_id: String,
    pub entity_name: String,
    pub entity_kind: String,
    pub location_id: String,
    pub location_name: String,
    pub location_slug: String,
    pub quantity: i32,
    pub collections: Vec<OrgInventoryCollection>,
    pub created_by: OrgUserRef,
    pub added_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct OrgTransferResult {
    pub source: Option<OrgInventoryEntry>,
    pub target: OrgInventoryEntry,
}

// ── Private DTOs (backend JSON shapes, not exported) ──────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgPermissions {
    #[serde(default)] manage_org: bool,
    #[serde(default)] manage_members: bool,
    #[serde(default)] manage_roles: bool,
    #[serde(default)] invite_members: bool,
    #[serde(default)] view_applications: bool,
    #[serde(default)] manage_applications: bool,
    #[serde(default)] manage_inventory: bool,
    #[serde(default)] manage_collections: bool,
}

impl From<BkOrgPermissions> for OrgPermissions {
    fn from(p: BkOrgPermissions) -> Self {
        OrgPermissions {
            manage_org: p.manage_org,
            manage_members: p.manage_members,
            manage_roles: p.manage_roles,
            invite_members: p.invite_members,
            view_applications: p.view_applications,
            manage_applications: p.manage_applications,
            manage_inventory: p.manage_inventory,
            manage_collections: p.manage_collections,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgUserRef {
    id: u32,
    username: String,
}

impl From<BkOrgUserRef> for OrgUserRef {
    fn from(u: BkOrgUserRef) -> Self {
        OrgUserRef { id: u.id, username: u.username }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgRef {
    id: u32,
    name: String,
    slug: String,
}

impl From<BkOrgRef> for OrgRef {
    fn from(r: BkOrgRef) -> Self {
        OrgRef { id: r.id, name: r.name, slug: r.slug }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgRoleRef {
    id: u32,
    name: String,
    #[serde(default)]
    is_leader: bool,
}

impl From<BkOrgRoleRef> for OrgRoleRef {
    fn from(r: BkOrgRoleRef) -> Self {
        OrgRoleRef { id: r.id, name: r.name, is_leader: r.is_leader }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgRoleWithPerms {
    id: u32,
    name: String,
    #[serde(default)]
    is_leader: bool,
    permissions: BkOrgPermissions,
}

impl From<BkOrgRoleWithPerms> for OrgRoleWithPerms {
    fn from(r: BkOrgRoleWithPerms) -> Self {
        OrgRoleWithPerms {
            id: r.id,
            name: r.name,
            is_leader: r.is_leader,
            permissions: r.permissions.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgRole {
    id: u32,
    name: String,
    #[serde(default)]
    is_leader: bool,
    permissions: BkOrgPermissions,
    sort_order: i32,
    member_count: u32,
}

impl From<BkOrgRole> for OrgRole {
    fn from(r: BkOrgRole) -> Self {
        OrgRole {
            id: r.id,
            name: r.name,
            is_leader: r.is_leader,
            permissions: r.permissions.into(),
            sort_order: r.sort_order,
            member_count: r.member_count,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgMemberInfo {
    user_id: u32,
    username: String,
    role: BkOrgRoleRef,
}

impl From<BkOrgMemberInfo> for OrgMemberInfo {
    fn from(m: BkOrgMemberInfo) -> Self {
        OrgMemberInfo { user_id: m.user_id, username: m.username, role: m.role.into() }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgSummary {
    id: u32,
    name: String,
    slug: String,
    description: Option<String>,
    member_count: u32,
    #[serde(default)]
    my_role: Option<BkOrgRoleWithPerms>,
}

impl From<BkOrgSummary> for OrgSummary {
    fn from(s: BkOrgSummary) -> Self {
        // If the backend doesn't include myRole (e.g. org create response),
        // the current user is always the leader.
        let my_role = s.my_role.map(|r| r.into()).unwrap_or_else(|| OrgRoleWithPerms {
            id: 0,
            name: "Leader".to_string(),
            is_leader: true,
            permissions: crate::commands::orgs::OrgPermissions::default(),
        });
        OrgSummary {
            id: s.id,
            name: s.name,
            slug: s.slug,
            description: s.description,
            member_count: s.member_count,
            my_role,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgDetail {
    id: u32,
    name: String,
    slug: String,
    description: Option<String>,
    members: Vec<BkOrgMemberInfo>,
    roles: Vec<BkOrgRole>,
    member_count: u32,
    created_at: String,
}

impl From<BkOrgDetail> for OrgDetail {
    fn from(d: BkOrgDetail) -> Self {
        OrgDetail {
            id: d.id,
            name: d.name,
            slug: d.slug,
            description: d.description,
            members: d.members.into_iter().map(|m| m.into()).collect(),
            roles: d.roles.into_iter().map(|r| r.into()).collect(),
            member_count: d.member_count,
            created_at: d.created_at,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgInvitation {
    id: u32,
    org_id: u32,
    org_name: String,
    invited_user: BkOrgUserRef,
    invited_by: BkOrgUserRef,
    status: String,
    created_at: String,
}

impl From<BkOrgInvitation> for OrgInvitation {
    fn from(i: BkOrgInvitation) -> Self {
        OrgInvitation {
            id: i.id,
            org_id: i.org_id,
            org_name: i.org_name,
            invited_user: i.invited_user.into(),
            invited_by: i.invited_by.into(),
            status: i.status,
            created_at: i.created_at,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkUserInvitation {
    id: u32,
    org: BkOrgRef,
    invited_by: BkOrgUserRef,
    status: String,
    created_at: String,
}

impl From<BkUserInvitation> for UserInvitation {
    fn from(i: BkUserInvitation) -> Self {
        UserInvitation {
            id: i.id,
            org: i.org.into(),
            invited_by: i.invited_by.into(),
            status: i.status,
            created_at: i.created_at,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgApplication {
    id: u32,
    org_id: u32,
    applicant: Option<BkOrgUserRef>,
    message: Option<String>,
    status: String,
    created_at: String,
}

impl From<BkOrgApplication> for OrgApplication {
    fn from(a: BkOrgApplication) -> Self {
        OrgApplication {
            id: a.id,
            org_id: a.org_id,
            applicant: a.applicant.map(|u| u.into()),
            message: a.message,
            status: a.status,
            created_at: a.created_at,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgInventoryCollection {
    id: u32,
    name: String,
    created_by: BkOrgUserRef,
}

impl From<BkOrgInventoryCollection> for OrgInventoryCollection {
    fn from(c: BkOrgInventoryCollection) -> Self {
        OrgInventoryCollection { id: c.id, name: c.name, created_by: c.created_by.into() }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkOrgInventoryEntry {
    id: u32,
    entity_id: String,
    entity_name: String,
    entity_kind: String,
    location_id: String,
    location_name: String,
    location_slug: String,
    quantity: i32,
    collections: Vec<BkOrgInventoryCollection>,
    created_by: BkOrgUserRef,
    added_at: String,
    updated_at: String,
}

impl From<BkOrgInventoryEntry> for OrgInventoryEntry {
    fn from(e: BkOrgInventoryEntry) -> Self {
        OrgInventoryEntry {
            id: e.id,
            entity_id: e.entity_id,
            entity_name: e.entity_name,
            entity_kind: e.entity_kind,
            location_id: e.location_id,
            location_name: e.location_name,
            location_slug: e.location_slug,
            quantity: e.quantity,
            collections: e.collections.into_iter().map(|c| c.into()).collect(),
            created_by: e.created_by.into(),
            added_at: e.added_at,
            updated_at: e.updated_at,
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn get_token(state: &State<AppState>) -> Result<String, String> {
    let token = state.current_settings.lock().unwrap().backend_api_token.clone();
    if token.is_empty() {
        Err("Not logged in".to_string())
    } else {
        Ok(token)
    }
}

fn parse_data<T: for<'de> Deserialize<'de>>(
    json: &serde_json::Value,
    context: &str,
) -> Result<T, String> {
    serde_json::from_value::<T>(json["data"].clone())
        .map_err(|e| format!("Failed to parse {context}: {e}"))
}

fn parse_data_array<T: for<'de> Deserialize<'de>>(
    json: &serde_json::Value,
    context: &str,
) -> Result<Vec<T>, String> {
    serde_json::from_value::<Vec<T>>(json["data"].clone())
        .map_err(|e| format!("Failed to parse {context}: {e}"))
}

async fn api_get(url: &str, token: &str) -> Result<serde_json::Value, String> {
    let client = http_client()?;
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let status = resp.status();
    let json: serde_json::Value =
        resp.json().await.map_err(|e| format!("Failed to parse response: {e}"))?;
    if !status.is_success() {
        return Err(extract_error_message(&json));
    }
    Ok(json)
}

async fn api_post(
    url: &str,
    token: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let client = http_client()?;
    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let status = resp.status();
    let json: serde_json::Value =
        resp.json().await.map_err(|e| format!("Failed to parse response: {e}"))?;
    if !status.is_success() {
        return Err(extract_error_message(&json));
    }
    Ok(json)
}

async fn api_patch(
    url: &str,
    token: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let client = http_client()?;
    let resp = client
        .patch(url)
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let status = resp.status();
    let json: serde_json::Value =
        resp.json().await.map_err(|e| format!("Failed to parse response: {e}"))?;
    if !status.is_success() {
        return Err(extract_error_message(&json));
    }
    Ok(json)
}

async fn api_delete(url: &str, token: &str) -> Result<(), String> {
    let client = http_client()?;
    let resp = client
        .delete(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    if !resp.status().is_success() {
        let json: serde_json::Value =
            resp.json().await.unwrap_or(serde_json::Value::Null);
        return Err(extract_error_message(&json));
    }
    Ok(())
}

// ── Org CRUD ───────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn org_list_my_orgs(
    state: State<'_, AppState>,
) -> Result<Vec<OrgSummary>, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs");
    let json = api_get(&url, &token).await?;
    let dtos: Vec<BkOrgSummary> = parse_data_array(&json, "org list")?;
    Ok(dtos.into_iter().map(|s| s.into()).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn org_create(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<OrgSummary, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs");
    let json = api_post(&url, &token, serde_json::json!({ "name": name, "description": description })).await?;
    let dto: BkOrgSummary = parse_data(&json, "org create")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_get(
    id: u32,
    state: State<'_, AppState>,
) -> Result<OrgDetail, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{id}");
    let json = api_get(&url, &token).await?;
    let dto: BkOrgDetail = parse_data(&json, "org detail")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_update(
    id: u32,
    name: Option<String>,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<OrgSummary, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{id}");
    let mut body = serde_json::Map::new();
    if let Some(n) = name { body.insert("name".into(), serde_json::Value::String(n)); }
    if let Some(d) = description { body.insert("description".into(), serde_json::Value::String(d)); }
    let json = api_patch(&url, &token, serde_json::Value::Object(body)).await?;
    let dto: BkOrgSummary = parse_data(&json, "org update")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_delete(
    id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{id}");
    api_delete(&url, &token).await
}

// ── Roles ──────────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn org_list_roles(
    org_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<OrgRole>, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/roles");
    let json = api_get(&url, &token).await?;
    let dtos: Vec<BkOrgRole> = parse_data_array(&json, "roles")?;
    Ok(dtos.into_iter().map(|r| r.into()).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn org_create_role(
    org_id: u32,
    name: String,
    permissions: OrgPermissions,
    sort_order: i32,
    state: State<'_, AppState>,
) -> Result<OrgRole, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/roles");
    let json = api_post(&url, &token, serde_json::json!({
        "name": name,
        "permissions": permissions,
        "sortOrder": sort_order,
    })).await?;
    let dto: BkOrgRole = parse_data(&json, "create role")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_update_role(
    org_id: u32,
    role_id: u32,
    name: Option<String>,
    permissions: Option<OrgPermissions>,
    sort_order: Option<i32>,
    state: State<'_, AppState>,
) -> Result<OrgRole, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/roles/{role_id}");
    let mut body = serde_json::Map::new();
    if let Some(n) = name { body.insert("name".into(), serde_json::Value::String(n)); }
    if let Some(p) = permissions {
        body.insert("permissions".into(), serde_json::to_value(p).unwrap_or_default());
    }
    if let Some(s) = sort_order { body.insert("sortOrder".into(), serde_json::Value::Number(s.into())); }
    let json = api_patch(&url, &token, serde_json::Value::Object(body)).await?;
    let dto: BkOrgRole = parse_data(&json, "update role")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_delete_role(
    org_id: u32,
    role_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/roles/{role_id}");
    api_delete(&url, &token).await
}

// ── Leadership Transfer result type ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgLeadershipTransferMember {
    pub user_id: u32,
    pub username: String,
    pub role: OrgRoleRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrgLeadershipTransfer {
    pub new_leader: OrgLeadershipTransferMember,
    pub previous_leader: OrgLeadershipTransferMember,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkLeadershipMember {
    user_id: u32,
    username: String,
    role: BkOrgRoleRef,
}

impl From<BkLeadershipMember> for OrgLeadershipTransferMember {
    fn from(m: BkLeadershipMember) -> Self {
        OrgLeadershipTransferMember { user_id: m.user_id, username: m.username, role: m.role.into() }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BkLeadershipTransfer {
    new_leader: BkLeadershipMember,
    previous_leader: BkLeadershipMember,
}

impl From<BkLeadershipTransfer> for OrgLeadershipTransfer {
    fn from(t: BkLeadershipTransfer) -> Self {
        OrgLeadershipTransfer {
            new_leader: t.new_leader.into(),
            previous_leader: t.previous_leader.into(),
        }
    }
}

// ── Members ────────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn org_update_member(
    org_id: u32,
    user_id: u32,
    role_id: u32,
    state: State<'_, AppState>,
) -> Result<OrgMemberInfo, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/members/{user_id}");
    let json = api_patch(&url, &token, serde_json::json!({ "roleId": role_id })).await?;
    let dto: BkOrgMemberInfo = parse_data(&json, "update member")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_remove_member(
    org_id: u32,
    user_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/members/{user_id}");
    api_delete(&url, &token).await
}

#[tauri::command]
#[specta::specta]
pub async fn org_transfer_leadership(
    org_id: u32,
    target_user_id: u32,
    new_role_id: Option<u32>,
    state: State<'_, AppState>,
) -> Result<OrgLeadershipTransfer, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/transfer-leadership");
    let json = api_post(&url, &token, serde_json::json!({
        "targetUserId": target_user_id,
        "newRoleId": new_role_id,
    })).await?;
    let dto: BkLeadershipTransfer = parse_data(&json, "transfer leadership")?;
    Ok(dto.into())
}

// ── Org Invitations ────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn org_list_invitations(
    org_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<OrgInvitation>, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/invitations");
    let json = api_get(&url, &token).await?;
    let dtos: Vec<BkOrgInvitation> = parse_data_array(&json, "org invitations")?;
    Ok(dtos.into_iter().map(|i| i.into()).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn org_create_invitation(
    org_id: u32,
    username: String,
    role_id: Option<u32>,
    state: State<'_, AppState>,
) -> Result<OrgInvitation, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/invitations");
    let json = api_post(&url, &token, serde_json::json!({
        "username": username,
        "roleId": role_id,
    })).await?;
    let dto: BkOrgInvitation = parse_data(&json, "create invitation")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_cancel_invitation(
    org_id: u32,
    inv_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/invitations/{inv_id}");
    api_delete(&url, &token).await
}

// ── User Invitations (incoming) ────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn user_list_invitations(
    state: State<'_, AppState>,
) -> Result<Vec<UserInvitation>, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/invitations?status=pending");
    let json = api_get(&url, &token).await?;
    let dtos: Vec<BkUserInvitation> = parse_data_array(&json, "user invitations")?;
    Ok(dtos.into_iter().map(|i| i.into()).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn user_accept_invitation(
    id: u32,
    state: State<'_, AppState>,
) -> Result<OrgSummary, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/invitations/{id}/accept");
    let json = api_post(&url, &token, serde_json::json!({})).await?;
    let dto: BkOrgSummary = parse_data(&json, "accept invitation")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn user_decline_invitation(
    id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/invitations/{id}/decline");
    api_post(&url, &token, serde_json::json!({})).await?;
    Ok(())
}

// ── Applications ───────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn org_list_applications(
    org_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<OrgApplication>, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/applications?status=pending");
    let json = api_get(&url, &token).await?;
    let dtos: Vec<BkOrgApplication> = parse_data_array(&json, "applications")?;
    Ok(dtos.into_iter().map(|a| a.into()).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn org_create_application(
    org_id: u32,
    message: Option<String>,
    state: State<'_, AppState>,
) -> Result<OrgApplication, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/applications");
    let json = api_post(&url, &token, serde_json::json!({ "message": message })).await?;
    // Create response has org nested, not org_id directly — extract from context
    let mut dto: BkOrgApplication = parse_data(&json, "create application")?;
    // Ensure org_id is set from the URL parameter if not in response
    if dto.org_id == 0 { dto.org_id = org_id; }
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_accept_application(
    org_id: u32,
    app_id: u32,
    role_id: Option<u32>,
    state: State<'_, AppState>,
) -> Result<OrgApplication, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/applications/{app_id}/accept");
    let json = api_post(&url, &token, serde_json::json!({ "roleId": role_id })).await?;
    let mut dto: BkOrgApplication = parse_data(&json, "accept application")?;
    if dto.org_id == 0 { dto.org_id = org_id; }
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_reject_application(
    org_id: u32,
    app_id: u32,
    state: State<'_, AppState>,
) -> Result<OrgApplication, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/applications/{app_id}/reject");
    let json = api_post(&url, &token, serde_json::json!({})).await?;
    let mut dto: BkOrgApplication = parse_data(&json, "reject application")?;
    if dto.org_id == 0 { dto.org_id = org_id; }
    Ok(dto.into())
}

// ── Org Inventory ──────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn org_list_inventory(
    org_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<OrgInventoryEntry>, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/inventory");
    let json = api_get(&url, &token).await?;
    let dtos: Vec<BkOrgInventoryEntry> = parse_data_array(&json, "org inventory")?;
    Ok(dtos.into_iter().map(|e| e.into()).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn org_add_inventory_entry(
    org_id: u32,
    entity_id: String,
    entity_name: String,
    entity_kind: String,
    location_id: String,
    location_name: String,
    location_slug: String,
    quantity: i32,
    collection_ids: Vec<u32>,
    state: State<'_, AppState>,
) -> Result<OrgInventoryEntry, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/inventory");
    let json = api_post(&url, &token, serde_json::json!({
        "entityId": entity_id,
        "entityName": entity_name,
        "entityKind": entity_kind,
        "locationId": location_id,
        "locationName": location_name,
        "locationSlug": location_slug,
        "quantity": quantity,
        "collectionIds": collection_ids,
    })).await?;
    let dto: BkOrgInventoryEntry = parse_data(&json, "add org inventory")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_update_inventory_entry(
    org_id: u32,
    entry_id: u32,
    entity_name: Option<String>,
    entity_kind: Option<String>,
    location_name: Option<String>,
    location_slug: Option<String>,
    quantity: Option<i32>,
    collection_ids: Option<Vec<u32>>,
    state: State<'_, AppState>,
) -> Result<OrgInventoryEntry, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/inventory/{entry_id}");
    let mut body = serde_json::Map::new();
    if let Some(v) = entity_name { body.insert("entityName".into(), v.into()); }
    if let Some(v) = entity_kind { body.insert("entityKind".into(), v.into()); }
    if let Some(v) = location_name { body.insert("locationName".into(), v.into()); }
    if let Some(v) = location_slug { body.insert("locationSlug".into(), v.into()); }
    if let Some(v) = quantity { body.insert("quantity".into(), v.into()); }
    if let Some(v) = collection_ids {
        body.insert("collectionIds".into(), serde_json::to_value(v).unwrap_or_default());
    }
    let json = api_patch(&url, &token, serde_json::Value::Object(body)).await?;
    let dto: BkOrgInventoryEntry = parse_data(&json, "update org inventory")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_delete_inventory_entry(
    org_id: u32,
    entry_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/inventory/{entry_id}");
    api_delete(&url, &token).await
}

#[tauri::command]
#[specta::specta]
pub async fn org_remove_inventory_quantity(
    org_id: u32,
    entry_id: u32,
    quantity: i32,
    state: State<'_, AppState>,
) -> Result<Option<OrgInventoryEntry>, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/inventory/{entry_id}/remove-quantity");
    let json = api_post(&url, &token, serde_json::json!({ "quantity": quantity })).await?;
    if json["data"].is_null() {
        return Ok(None);
    }
    let dto: BkOrgInventoryEntry = parse_data(&json, "remove org inventory quantity")?;
    Ok(Some(dto.into()))
}

#[tauri::command]
#[specta::specta]
pub async fn org_transfer_inventory(
    org_id: u32,
    entry_id: u32,
    quantity: i32,
    target_location_id: String,
    target_location_name: String,
    target_location_slug: String,
    state: State<'_, AppState>,
) -> Result<OrgTransferResult, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/inventory/{entry_id}/transfer");
    let json = api_post(&url, &token, serde_json::json!({
        "quantity": quantity,
        "locationId": target_location_id,
        "locationName": target_location_name,
        "locationSlug": target_location_slug,
    })).await?;
    let source: Option<OrgInventoryEntry> = if json["data"]["source"].is_null() {
        None
    } else {
        let dto: BkOrgInventoryEntry = serde_json::from_value(json["data"]["source"].clone())
            .map_err(|e| format!("Failed to parse transfer source: {e}"))?;
        Some(dto.into())
    };
    let target_dto: BkOrgInventoryEntry = serde_json::from_value(json["data"]["target"].clone())
        .map_err(|e| format!("Failed to parse transfer target: {e}"))?;
    Ok(OrgTransferResult { source, target: target_dto.into() })
}

// ── Org Inventory Collections ──────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn org_list_collections(
    org_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<OrgInventoryCollection>, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/inventory/collections");
    let json = api_get(&url, &token).await?;
    let dtos: Vec<BkOrgInventoryCollection> = parse_data_array(&json, "org collections")?;
    Ok(dtos.into_iter().map(|c| c.into()).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn org_create_collection(
    org_id: u32,
    name: String,
    state: State<'_, AppState>,
) -> Result<OrgInventoryCollection, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/inventory/collections");
    let json = api_post(&url, &token, serde_json::json!({ "name": name })).await?;
    let dto: BkOrgInventoryCollection = parse_data(&json, "create org collection")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_update_collection(
    org_id: u32,
    coll_id: u32,
    name: String,
    state: State<'_, AppState>,
) -> Result<OrgInventoryCollection, String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/inventory/collections/{coll_id}");
    let json = api_patch(&url, &token, serde_json::json!({ "name": name })).await?;
    let dto: BkOrgInventoryCollection = parse_data(&json, "update org collection")?;
    Ok(dto.into())
}

#[tauri::command]
#[specta::specta]
pub async fn org_delete_collection(
    org_id: u32,
    coll_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let token = get_token(&state)?;
    let url = format!("{BACKEND_URL}/api/orgs/{org_id}/inventory/collections/{coll_id}");
    api_delete(&url, &token).await
}
