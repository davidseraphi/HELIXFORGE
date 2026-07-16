//! Workspaces, folders, comments, and mentions for HelixCollab.

use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{AclPermission, CollabFolder, DocumentComment, Mention};
use serde::Deserialize;
use service_kit::ApiError;
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError};
use uuid::Uuid;

use super::documents::{ensure_doc_access_pub, Auth};
use super::CollabState;

const PRODUCT: &str = "helix-collab";

pub fn routes() -> Router<CollabState> {
    // Note: /v1/workspaces is owned by ProductService (shared workspace list/create).
    Router::new()
        .route(
            "/v1/workspaces/{ws_id}/folders",
            get(list_folders).post(create_folder),
        )
        .route(
            "/v1/folders/{id}",
            axum::routing::patch(rename_folder).delete(delete_folder),
        )
        .route(
            "/v1/documents/{id}/comments",
            get(list_comments).post(add_comment),
        )
        .route(
            "/v1/documents/{id}/comments/{cid}",
            axum::routing::patch(update_comment).delete(delete_comment),
        )
        .route(
            "/v1/documents/{id}/comments/{cid}/resolve",
            post(resolve_comment),
        )
        .route("/v1/documents/{id}/mention-suggest", get(mention_suggest))
        .route("/v1/documents/{id}/activity", get(list_activity))
        .route("/v1/mentions/inbox", get(mentions_inbox))
        .route("/v1/documents/{id}/move", post(move_document))
}

#[derive(Deserialize)]
struct CreateFolderBody {
    name: String,
    #[serde(default)]
    parent_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct RenameFolderBody {
    name: String,
}

#[derive(Deserialize)]
struct CommentBody {
    body: String,
    #[serde(default)]
    parent_id: Option<Uuid>,
    #[serde(default)]
    author_label: Option<String>,
    #[serde(default)]
    anchor_start: Option<i32>,
    #[serde(default)]
    anchor_end: Option<i32>,
    #[serde(default)]
    anchor_quote: Option<String>,
}

#[derive(Deserialize)]
struct ResolveBody {
    #[serde(default = "default_true")]
    resolved: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct MoveBody {
    #[serde(default)]
    folder_id: Option<Uuid>,
    #[serde(default)]
    workspace_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct InboxQuery {
    #[serde(default)]
    label: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    50
}

async fn list_folders(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(ws_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<CollabFolder>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    Ok(Json(ApiResponse::ok(
        repo.list_folders(p.tenant_id, ws_id).await?,
    )))
}

async fn create_folder(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(ws_id): Path<Uuid>,
    Json(body): Json<CreateFolderBody>,
) -> Result<Json<ApiResponse<CollabFolder>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let folder = repo
        .create_folder(p.tenant_id, ws_id, body.parent_id, &body.name, p.user_id)
        .await?;
    Ok(Json(ApiResponse::ok(folder)))
}

async fn rename_folder(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<RenameFolderBody>,
) -> Result<Json<ApiResponse<CollabFolder>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    Ok(Json(ApiResponse::ok(
        repo.rename_folder(p.tenant_id, id, &body.name).await?,
    )))
}

async fn delete_folder(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    repo.delete_folder(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "deleted": id }))))
}

async fn list_comments(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<DocumentComment>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    Ok(Json(ApiResponse::ok(
        repo.list_comments(p.tenant_id, id).await?,
    )))
}

async fn add_comment(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<CommentBody>,
) -> Result<Json<ApiResponse<DocumentComment>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let label = body
        .author_label
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| p.user_id.to_string());
    let comment = repo
        .add_comment_anchored(
            p.tenant_id,
            id,
            p.user_id,
            &label,
            &body.body,
            body.parent_id,
            body.anchor_start,
            body.anchor_end,
            body.anchor_quote.as_deref().unwrap_or(""),
        )
        .await?;
    state.hub.publish(
        id,
        &super::realtime::CollabMessage::CommentEvent {
            action: "created".into(),
            comment_id: comment.id.to_string(),
            anchor_start: comment.anchor_start,
            anchor_end: comment.anchor_end,
        },
    );
    let _ = state
        .core
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "document.comment".into(),
            resource_type: "document".into(),
            resource_id: id.to_string(),
            metadata: serde_json::json!({
                "comment_id": comment.id,
                "mentions": comment.mentions.iter().map(|m| &m.mentioned_label).collect::<Vec<_>>(),
                "anchored": comment.anchor_start.is_some(),
            }),
            residency_region: p.residency_region.clone(),
        })
        .await;
    let _ = state
        .core
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            PRODUCT,
            "comments.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await;
    Ok(Json(ApiResponse::ok(comment)))
}

async fn update_comment(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path((id, cid)): Path<(Uuid, Uuid)>,
    Json(body): Json<CommentBody>,
) -> Result<Json<ApiResponse<DocumentComment>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    Ok(Json(ApiResponse::ok(
        repo.update_comment(p.tenant_id, id, cid, p.user_id, &body.body)
            .await?,
    )))
}

async fn delete_comment(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path((id, cid)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    repo.delete_comment(p.tenant_id, id, cid).await?;
    state.hub.publish(
        id,
        &super::realtime::CollabMessage::CommentEvent {
            action: "deleted".into(),
            comment_id: cid.to_string(),
            anchor_start: None,
            anchor_end: None,
        },
    );
    Ok(Json(ApiResponse::ok(serde_json::json!({ "deleted": cid }))))
}

async fn resolve_comment(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path((id, cid)): Path<(Uuid, Uuid)>,
    Json(body): Json<ResolveBody>,
) -> Result<Json<ApiResponse<DocumentComment>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let c = repo
        .resolve_comment(p.tenant_id, id, cid, p.user_id, body.resolved)
        .await?;
    state.hub.publish(
        id,
        &super::realtime::CollabMessage::CommentEvent {
            action: if body.resolved {
                "resolved".into()
            } else {
                "unresolved".into()
            },
            comment_id: cid.to_string(),
            anchor_start: c.anchor_start,
            anchor_end: c.anchor_end,
        },
    );
    Ok(Json(ApiResponse::ok(c)))
}

async fn list_activity(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<helix_db::DocActivity>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    Ok(Json(ApiResponse::ok(
        repo.list_activity(p.tenant_id, id, 50).await?,
    )))
}

/// Suggest mention targets from recent presence + ACL principals on the doc.
async fn mention_suggest(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let mut labels: Vec<String> = Vec::new();
    if let Some(repo) = state.core.clients.collab.as_ref() {
        for peer in repo.list_presence(id).await.unwrap_or_default() {
            if !peer.display_name.is_empty() {
                labels.push(peer.display_name);
            }
            labels.push(peer.user_id.to_string());
        }
    }
    if let Some(acl) = state.core.clients.acl.as_ref() {
        for e in acl
            .list_for_resource(p.tenant_id, "document", &id.to_string())
            .await
            .unwrap_or_default()
        {
            labels.push(e.principal_id);
        }
    }
    labels.push(p.user_id.to_string());
    labels.sort();
    labels.dedup();
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "suggestions": labels }),
    )))
}

async fn mentions_inbox(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Query(q): Query<InboxQuery>,
) -> Result<Json<ApiResponse<Vec<Mention>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let label = q
        .label
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| p.user_id.to_string());
    Ok(Json(ApiResponse::ok(
        repo.list_mentions_for_user(p.tenant_id, &label, q.limit)
            .await?,
    )))
}

async fn move_document(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<MoveBody>,
) -> Result<Json<ApiResponse<helix_db::CollabDocument>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    Ok(Json(ApiResponse::ok(
        repo.move_document(p.tenant_id, id, body.folder_id, body.workspace_id)
            .await?,
    )))
}
