//! HelixPulse durable store — `pulse` schema (monitors + incidents).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PulseSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_incidents: i64,
    pub open_incidents: i64,
    pub acknowledged_incidents: i64,
    pub resolved_incidents: i64,
}

#[derive(Debug, Clone, Default)]
pub struct MonitorUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct IncidentUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate a monitor lifecycle transition and return the resulting status.
pub fn next_monitor_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "activate") => Ok("active"),
        ("active", "pause") => Ok("paused"),
        ("paused", "resume") => Ok("active"),
        (_, "activate") => Err(HelixError::validation(format!(
            "cannot activate a {current} monitor"
        ))),
        (_, "pause") => Err(HelixError::validation(format!(
            "cannot pause a {current} monitor"
        ))),
        (_, "resume") => Err(HelixError::validation(format!(
            "cannot resume a {current} monitor"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown monitor action {action}"
        ))),
    }
}

/// Validate an incident lifecycle transition and return the resulting status.
pub fn next_incident_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("open", "acknowledge") => Ok("acknowledged"),
        ("open", "resolve") | ("acknowledged", "resolve") => Ok("resolved"),
        (_, "acknowledge") => Err(HelixError::validation(format!(
            "cannot acknowledge a {current} incident"
        ))),
        (_, "resolve") => Err(HelixError::validation(format!(
            "cannot resolve a {current} incident"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown incident action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct MonitorRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    activated_at: Option<DateTime<Utc>>,
    paused_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl MonitorRow {
    fn into_monitor(self) -> Monitor {
        Monitor {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            activated_at: self.activated_at,
            paused_at: self.paused_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct IncidentRow {
    id: Uuid,
    tenant_id: Uuid,
    parent_id: Uuid,
    title: String,
    body: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    acknowledged_at: Option<DateTime<Utc>>,
    resolved_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl IncidentRow {
    fn into_incident(self) -> Incident {
        Incident {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            parent_id: self.parent_id,
            title: self.title,
            body: self.body,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
            acknowledged_at: self.acknowledged_at,
            resolved_at: self.resolved_at,
            deleted_at: self.deleted_at,
        }
    }
}

const MONITOR_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           activated_at, paused_at, deleted_at
    FROM pulse.monitors
"#;

const MONITOR_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              activated_at, paused_at, deleted_at
"#;

const INCIDENT_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, acknowledged_at, resolved_at, deleted_at
    FROM pulse.incidents
"#;

const INCIDENT_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, acknowledged_at, resolved_at, deleted_at
"#;

#[derive(Clone)]
pub struct PulseRepo {
    pool: PgPool,
}

impl PulseRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Monitors ---

    pub async fn list_monitors(&self, tenant_id: TenantId) -> HelixResult<Vec<Monitor>> {
        let rows: Vec<MonitorRow> = sqlx::query_as(&format!(
            "{MONITOR_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse list monitors: {e}")))?;
        Ok(rows.into_iter().map(MonitorRow::into_monitor).collect())
    }

    pub async fn create_monitor(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Monitor> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: MonitorRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO pulse.monitors
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {MONITOR_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(description)
        .bind(&metadata)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse create monitor: {e}")))?;
        Ok(row.into_monitor())
    }

    pub async fn get_monitor(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
    ) -> HelixResult<Option<Monitor>> {
        let row: Option<MonitorRow> = sqlx::query_as(&format!(
            "{MONITOR_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse get monitor: {e}")))?;
        Ok(row.map(MonitorRow::into_monitor))
    }

    async fn fetch_monitor_any(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
    ) -> HelixResult<Option<Monitor>> {
        let row: Option<MonitorRow> = sqlx::query_as(&format!(
            "{MONITOR_SELECT} WHERE tenant_id = $1 AND id = $2"
        ))
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse fetch monitor: {e}")))?;
        Ok(row.map(MonitorRow::into_monitor))
    }

    pub async fn update_monitor(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
        update: MonitorUpdate,
    ) -> HelixResult<Monitor> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE pulse.monitors SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(n) = update.name {
            builder.push(", name = ");
            builder.push_bind(n);
        }
        if let Some(d) = update.description {
            builder.push(", description = ");
            builder.push_bind(d);
        }
        if let Some(m) = update.metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND id = ");
        builder.push_bind(monitor_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {MONITOR_RETURNING}"));

        let row: Option<MonitorRow> = builder
            .build_query_as::<MonitorRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("pulse update monitor: {e}")))?;

        row.map(MonitorRow::into_monitor)
            .ok_or_else(|| HelixError::not_found("monitor not found"))
    }

    pub async fn activate_monitor(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
    ) -> HelixResult<Monitor> {
        let monitor = self
            .get_monitor(tenant_id, monitor_id)
            .await?
            .ok_or_else(|| HelixError::not_found("monitor not found"))?;
        let next = next_monitor_status(&monitor.status, "activate")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<MonitorRow> = sqlx::query_as(&format!(
            r#"
            UPDATE pulse.monitors
            SET status = $1, activated_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = $5 AND deleted_at IS NULL
            {MONITOR_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .bind(&monitor.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse activate monitor: {e}")))?;

        row.map(MonitorRow::into_monitor)
            .ok_or_else(|| HelixError::conflict("monitor changed during activate; retry"))
    }

    /// Pause an active monitor. Rejected while open incidents remain.
    pub async fn pause_monitor(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
    ) -> HelixResult<Monitor> {
        let monitor = self
            .get_monitor(tenant_id, monitor_id)
            .await?
            .ok_or_else(|| HelixError::not_found("monitor not found"))?;
        let next = next_monitor_status(&monitor.status, "pause")?;

        let open: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pulse.incidents WHERE tenant_id = $1 AND parent_id = $2 AND status = 'open' AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse pause incident count: {e}")))?;
        if open > 0 {
            return Err(HelixError::validation(format!(
                "monitor has {open} open incident(s); acknowledge or resolve them first"
            )));
        }

        let now = Utc::now();
        // The active-status and no-open-incidents guards are part of the
        // UPDATE itself: a concurrent pause or an incident opened
        // mid-flight cannot slip through a check-then-act window.
        let row: Option<MonitorRow> = sqlx::query_as(&format!(
            r#"
            UPDATE pulse.monitors
            SET status = $1, paused_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = 'active' AND deleted_at IS NULL
              AND NOT EXISTS (
                  SELECT 1 FROM pulse.incidents i
                  WHERE i.tenant_id = $3 AND i.parent_id = $4
                    AND i.status = 'open' AND i.deleted_at IS NULL
              )
            {MONITOR_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse pause monitor: {e}")))?;

        row.map(MonitorRow::into_monitor).ok_or_else(|| {
            HelixError::conflict("monitor changed during pause or gained an open incident; retry")
        })
    }

    pub async fn resume_monitor(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
    ) -> HelixResult<Monitor> {
        let monitor = self
            .get_monitor(tenant_id, monitor_id)
            .await?
            .ok_or_else(|| HelixError::not_found("monitor not found"))?;
        let next = next_monitor_status(&monitor.status, "resume")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<MonitorRow> = sqlx::query_as(&format!(
            r#"
            UPDATE pulse.monitors
            SET status = $1, paused_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = $5 AND deleted_at IS NULL
            {MONITOR_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .bind(&monitor.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse resume monitor: {e}")))?;

        row.map(MonitorRow::into_monitor)
            .ok_or_else(|| HelixError::conflict("monitor changed during resume; retry"))
    }

    pub async fn soft_delete_monitor(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
    ) -> HelixResult<Monitor> {
        let monitor = self
            .get_monitor(tenant_id, monitor_id)
            .await?
            .ok_or_else(|| HelixError::not_found("monitor not found"))?;
        if monitor.status == "deleted" {
            return Err(HelixError::validation("monitor is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<MonitorRow> = sqlx::query_as(&format!(
            r#"
            UPDATE pulse.monitors
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {MONITOR_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse soft-delete monitor: {e}")))?;

        row.map(MonitorRow::into_monitor)
            .ok_or_else(|| HelixError::not_found("monitor not found"))
    }

    /// Restore a soft-deleted monitor, returning it to its pre-delete status.
    pub async fn restore_monitor(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
    ) -> HelixResult<Monitor> {
        let monitor = self
            .fetch_monitor_any(tenant_id, monitor_id)
            .await?
            .ok_or_else(|| HelixError::not_found("monitor not found"))?;
        if monitor.deleted_at.is_none() {
            return Err(HelixError::validation("monitor is not deleted"));
        }
        let restored = if monitor.paused_at.is_some() {
            "paused"
        } else if monitor.activated_at.is_some() {
            "active"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<MonitorRow> = sqlx::query_as(&format!(
            r#"
            UPDATE pulse.monitors
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {MONITOR_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse restore monitor: {e}")))?;

        row.map(MonitorRow::into_monitor)
            .ok_or_else(|| HelixError::not_found("monitor not found or not deleted"))
    }

    // --- Incidents ---

    pub async fn list_incidents(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
    ) -> HelixResult<Vec<Incident>> {
        let rows: Vec<IncidentRow> = sqlx::query_as(&format!(
            "{INCIDENT_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse list incidents: {e}")))?;
        Ok(rows.into_iter().map(IncidentRow::into_incident).collect())
    }

    pub async fn create_incident(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Incident> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        // The non-deleted-parent guard is part of the INSERT itself: a
        // monitor soft-deleted between a separate check and insert cannot
        // leak incidents.
        let row: Option<IncidentRow> = sqlx::query_as(&format!(
            r#"
            INSERT INTO pulse.incidents
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            SELECT $1,$2,$3,$4,$5,'open',$6,$7,$7
            FROM pulse.monitors m
            WHERE m.tenant_id = $2 AND m.id = $3 AND m.deleted_at IS NULL
            {INCIDENT_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .bind(title)
        .bind(body)
        .bind(&metadata)
        .bind(created_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse create incident: {e}")))?;
        row.map(IncidentRow::into_incident)
            .ok_or_else(|| HelixError::not_found("monitor not found"))
    }

    pub async fn get_incident(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
        incident_id: Uuid,
    ) -> HelixResult<Option<Incident>> {
        let row: Option<IncidentRow> = sqlx::query_as(&format!(
            "{INCIDENT_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .bind(incident_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse get incident: {e}")))?;
        Ok(row.map(IncidentRow::into_incident))
    }

    async fn fetch_incident_any(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
        incident_id: Uuid,
    ) -> HelixResult<Option<Incident>> {
        let row: Option<IncidentRow> = sqlx::query_as(&format!(
            "{INCIDENT_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .bind(incident_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse fetch incident: {e}")))?;
        Ok(row.map(IncidentRow::into_incident))
    }

    pub async fn update_incident(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
        incident_id: Uuid,
        update: IncidentUpdate,
    ) -> HelixResult<Incident> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE pulse.incidents SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(t) = update.title {
            builder.push(", title = ");
            builder.push_bind(t);
        }
        if let Some(b) = update.body {
            builder.push(", body = ");
            builder.push_bind(b);
        }
        if let Some(m) = update.metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND parent_id = ");
        builder.push_bind(monitor_id);
        builder.push(" AND id = ");
        builder.push_bind(incident_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {INCIDENT_RETURNING}"));

        let row: Option<IncidentRow> = builder
            .build_query_as::<IncidentRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("pulse update incident: {e}")))?;

        row.map(IncidentRow::into_incident)
            .ok_or_else(|| HelixError::not_found("incident not found"))
    }

    async fn transition_incident(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
        incident_id: Uuid,
        action: &str,
    ) -> HelixResult<Incident> {
        let incident = self
            .get_incident(tenant_id, monitor_id, incident_id)
            .await?
            .ok_or_else(|| HelixError::not_found("incident not found"))?;
        let next = next_incident_status(&incident.status, action)?;
        let now = Utc::now();
        let (acknowledged_at, resolved_at) = match next {
            "acknowledged" => (Some(now), None),
            "resolved" => (incident.acknowledged_at, Some(now)),
            _ => (incident.acknowledged_at, incident.resolved_at),
        };
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<IncidentRow> = sqlx::query_as(&format!(
            r#"
            UPDATE pulse.incidents
            SET status = $1, acknowledged_at = $2, resolved_at = $3, updated_at = $4
            WHERE tenant_id = $5 AND parent_id = $6 AND id = $7 AND status = $8 AND deleted_at IS NULL
            {INCIDENT_RETURNING}
            "#
        ))
        .bind(next)
        .bind(acknowledged_at)
        .bind(resolved_at)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .bind(incident_id)
        .bind(&incident.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse {action} incident: {e}")))?;

        row.map(IncidentRow::into_incident)
            .ok_or_else(|| HelixError::conflict("incident changed during transition; retry"))
    }

    pub async fn acknowledge_incident(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
        incident_id: Uuid,
    ) -> HelixResult<Incident> {
        self.transition_incident(tenant_id, monitor_id, incident_id, "acknowledge")
            .await
    }

    pub async fn resolve_incident(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
        incident_id: Uuid,
    ) -> HelixResult<Incident> {
        self.transition_incident(tenant_id, monitor_id, incident_id, "resolve")
            .await
    }

    pub async fn soft_delete_incident(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
        incident_id: Uuid,
    ) -> HelixResult<Incident> {
        let incident = self
            .get_incident(tenant_id, monitor_id, incident_id)
            .await?
            .ok_or_else(|| HelixError::not_found("incident not found"))?;
        if incident.status == "deleted" {
            return Err(HelixError::validation("incident is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<IncidentRow> = sqlx::query_as(&format!(
            r#"
            UPDATE pulse.incidents
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {INCIDENT_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .bind(incident_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse soft-delete incident: {e}")))?;

        row.map(IncidentRow::into_incident)
            .ok_or_else(|| HelixError::not_found("incident not found"))
    }

    /// Restore a soft-deleted incident, returning it to its pre-delete status.
    pub async fn restore_incident(
        &self,
        tenant_id: TenantId,
        monitor_id: Uuid,
        incident_id: Uuid,
    ) -> HelixResult<Incident> {
        let incident = self
            .fetch_incident_any(tenant_id, monitor_id, incident_id)
            .await?
            .ok_or_else(|| HelixError::not_found("incident not found"))?;
        if incident.deleted_at.is_none() {
            return Err(HelixError::validation("incident is not deleted"));
        }
        let restored = if incident.resolved_at.is_some() {
            "resolved"
        } else if incident.acknowledged_at.is_some() {
            "acknowledged"
        } else {
            "open"
        };
        let now = Utc::now();
        let row: Option<IncidentRow> = sqlx::query_as(&format!(
            r#"
            UPDATE pulse.incidents
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {INCIDENT_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(monitor_id)
        .bind(incident_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse restore incident: {e}")))?;

        row.map(IncidentRow::into_incident)
            .ok_or_else(|| HelixError::not_found("incident not found or not deleted"))
    }

    // --- Reports ---

    /// Per-monitor incident counts by status for non-deleted monitors.
    pub async fn get_pulse_summary(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<Vec<PulseSummaryRow>> {
        let rows: Vec<PulseSummaryRow> = sqlx::query_as(
            r#"
            SELECT m.id, m.name, m.status,
                   COUNT(i.id) AS total_incidents,
                   COUNT(i.id) FILTER (WHERE i.status = 'open') AS open_incidents,
                   COUNT(i.id) FILTER (WHERE i.status = 'acknowledged') AS acknowledged_incidents,
                   COUNT(i.id) FILTER (WHERE i.status = 'resolved') AS resolved_incidents
            FROM pulse.monitors m
            LEFT JOIN pulse.incidents i
                   ON i.parent_id = m.id AND i.tenant_id = m.tenant_id
                  AND i.deleted_at IS NULL
            WHERE m.tenant_id = $1 AND m.deleted_at IS NULL
            GROUP BY m.id, m.name, m.status, m.created_at
            ORDER BY m.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pulse summary: {e}")))?;
        Ok(rows)
    }
}
