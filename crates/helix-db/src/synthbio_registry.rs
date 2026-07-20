//! HelixSynthBio registry — accessioned designs, immutable versions,
//! bidirectional lineage, and risk review with named human authority.
//!
//! Benchling-grade invariants:
//! - Accession IDs are allocated atomically (row-locked counter upsert).
//! - Design versions are immutable (DB trigger; this repo exposes no
//!   update/delete path for them).
//! - Lineage events are append-only and hash-chained per entity.
//! - Risk review: `unknown` is never safe; non-unknown decisions require a
//!   named human reviewer; transitions are guarded single statements so a
//!   concurrent review loses instead of overwriting.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use shared_core::{HelixError, HelixResult, TenantId};

use crate::synthbio_genbank::{parse_import, ParsedRecord};

// ——— domain types ———

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RegistryDesign {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub accession: String,
    pub name: String,
    pub description: String,
    pub access_class: String,
    pub status: String,
    pub current_version: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DesignVersion {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub design_id: Uuid,
    pub version: i32,
    pub alphabet: String,
    pub topology: String,
    pub source_kind: String,
    pub source_name: String,
    pub sequence_length: i32,
    pub sequence_text: String,
    pub components: JsonValue,
    pub content_hash: String,
    pub provenance: String,
    pub notes: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RiskCase {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub design_id: Uuid,
    pub design_version_id: Option<Uuid>,
    pub state: String,
    pub intended_use: String,
    pub policy_version: String,
    pub reasons: JsonValue,
    pub conditions: String,
    pub reviewer: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub locked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LineageEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub entity_kind: String,
    pub entity_id: Uuid,
    pub event_kind: String,
    pub actor: String,
    pub details: JsonValue,
    pub content_hash: String,
    pub prev_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LineageEdge {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub parent_kind: String,
    pub parent_id: Uuid,
    pub child_kind: String,
    pub child_id: Uuid,
    pub relation: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub role_so: String,
    pub start: usize,
    pub end: usize,
    pub strand: i8,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInput {
    pub alphabet: String,
    pub topology: String,
    pub source_kind: String,
    pub source_name: String,
    pub sequence_text: String,
    pub components: Vec<Component>,
    pub provenance: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDecision {
    pub state: String, // allowed | restricted | blocked
    pub intended_use: String,
    pub policy_version: String,
    pub reasons: Vec<String>,
    pub conditions: String,
    pub expires_at: Option<DateTime<Utc>>,
    /// CAS guard: the decision only lands if the case is still in this
    /// state. `None` uses the state read at review start (re-review by
    /// design); a caller that pins the state it saw gets a strict
    /// single-winner race.
    #[serde(default)]
    pub expected_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRejected {
    pub record: String,
    pub line: usize,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportManifest {
    pub total_records: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub accepted: Vec<RegistryDesign>,
    pub rejected: Vec<ImportRejected>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Design360 {
    pub design: RegistryDesign,
    pub versions: Vec<DesignVersion>,
    pub risk_case: Option<RiskCase>,
    pub effective_risk: String,
    pub edges: Vec<LineageEdge>,
    pub events: Vec<LineageEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub bundle_version: String,
    pub generated_at: DateTime<Utc>,
    pub tenant_id: Uuid,
    pub design: RegistryDesign,
    pub versions: Vec<DesignVersion>,
    pub risk_case: Option<RiskCase>,
    pub events: Vec<LineageEvent>,
    pub edges: Vec<LineageEdge>,
    pub bundle_hash: String,
}

// ——— helpers ———

fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    format!("{:x}", h.finalize())
}

fn version_content_hash(
    alphabet: &str,
    topology: &str,
    sequence: &str,
    components: &JsonValue,
) -> String {
    let canonical_seq: String = sequence
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    sha256_hex(&format!(
        "{alphabet}|{topology}|{}|{canonical_seq}",
        serde_json::to_string(components).unwrap_or_default()
    ))
}

fn normalize_sequence(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

fn role_so_for(feature_key: &str) -> &'static str {
    match feature_key.to_lowercase().as_str() {
        "promoter" => "SO:0000167",
        "cds" => "SO:0000316",
        "rbs" | "ribosome_binding_site" => "SO:0000139",
        "terminator" => "SO:0000141",
        "gene" => "SO:0000704",
        _ => "SO:0000001",
    }
}

const RISK_STATES: [&str; 3] = ["allowed", "restricted", "blocked"];

// ——— repo ———

#[derive(Clone)]
pub struct RegistryRepo {
    pool: PgPool,
}

impl RegistryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Atomic accession allocation: one row-locked upsert per scope.
    async fn next_accession(
        &self,
        tenant_id: TenantId,
        kind: &str,
        prefix: &str,
    ) -> HelixResult<String> {
        let next: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO synthbio.accession_counters (tenant_id, kind, next_value)
            VALUES ($1, $2, 2)
            ON CONFLICT (tenant_id, kind)
            DO UPDATE SET next_value = synthbio.accession_counters.next_value + 1
            RETURNING next_value - 1
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(kind)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio accession: {e}")))?;
        Ok(format!("{prefix}-{:06}", next.0))
    }

    /// Append a hash-chained event for an entity (append-only, DB-enforced).
    async fn record_event(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: TenantId,
        entity_kind: &str,
        entity_id: Uuid,
        event_kind: &str,
        actor: &str,
        details: JsonValue,
    ) -> HelixResult<()> {
        let prev: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT content_hash FROM synthbio.lineage_events
            WHERE tenant_id = $1 AND entity_kind = $2 AND entity_id = $3
            ORDER BY created_at DESC, id DESC LIMIT 1
            FOR UPDATE
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_kind)
        .bind(entity_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio event chain: {e}")))?;
        let prev_hash = prev.map(|p| p.0).unwrap_or_default();
        let content_hash = sha256_hex(&format!(
            "{prev_hash}|{entity_kind}|{entity_id}|{event_kind}|{actor}|{}",
            serde_json::to_string(&details).unwrap_or_default()
        ));
        sqlx::query(
            r#"
            INSERT INTO synthbio.lineage_events
                (id, tenant_id, entity_kind, entity_id, event_kind, actor, details, content_hash, prev_hash, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(tenant_id.as_uuid())
        .bind(entity_kind)
        .bind(entity_id)
        .bind(event_kind)
        .bind(actor)
        .bind(&details)
        .bind(&content_hash)
        .bind(&prev_hash)
        .bind(Utc::now())
        .execute(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio record event: {e}")))?;
        Ok(())
    }

    async fn add_edge(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: TenantId,
        parent_kind: &str,
        parent_id: Uuid,
        child_kind: &str,
        child_id: Uuid,
        relation: &str,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"
            INSERT INTO synthbio.lineage_edges
                (id, tenant_id, parent_kind, parent_id, child_kind, child_id, relation, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            ON CONFLICT (tenant_id, parent_kind, parent_id, child_kind, child_id, relation)
            DO NOTHING
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(tenant_id.as_uuid())
        .bind(parent_kind)
        .bind(parent_id)
        .bind(child_kind)
        .bind(child_id)
        .bind(relation)
        .bind(Utc::now())
        .execute(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio add edge: {e}")))?;
        Ok(())
    }

    /// Create an accessioned design with immutable version 1 — one tx.
    pub async fn create_design(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        access_class: &str,
        input: &VersionInput,
        actor: &str,
    ) -> HelixResult<RegistryDesign> {
        if name.trim().is_empty() {
            return Err(HelixError::validation("design name required"));
        }
        validate_alphabet_topology(&input.alphabet, &input.topology)?;
        let accession = self.next_accession(tenant_id, "design", "DSN").await?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio create tx: {e}")))?;

        let design_id = Uuid::now_v7();
        let now = Utc::now();
        let design: RegistryDesign = sqlx::query_as(
            r#"
            INSERT INTO synthbio.registry_designs
                (id, tenant_id, accession, name, description, access_class, status,
                 current_version, created_by, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,'active',1,$7,$8,$8)
            RETURNING id, tenant_id, accession, name, description, access_class, status,
                      current_version, created_by, created_at, updated_at, NULL AS deleted_at
            "#,
        )
        .bind(design_id)
        .bind(tenant_id.as_uuid())
        .bind(&accession)
        .bind(name)
        .bind(description)
        .bind(access_class)
        .bind(actor)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio create design: {e}")))?;

        let version_id = self
            .insert_version(&mut tx, tenant_id, design_id, 1, input, actor)
            .await?;
        self.add_edge(
            &mut tx,
            tenant_id,
            "design",
            design_id,
            "design_version",
            version_id,
            "contains",
        )
        .await?;
        self.record_event(
            &mut tx,
            tenant_id,
            "design",
            design_id,
            "created",
            actor,
            serde_json::json!({"accession": accession, "source_kind": input.source_kind}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio create commit: {e}")))?;
        Ok(design)
    }

    async fn insert_version(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: TenantId,
        design_id: Uuid,
        version: i32,
        input: &VersionInput,
        actor: &str,
    ) -> HelixResult<Uuid> {
        let sequence = normalize_sequence(&input.sequence_text);
        let components = serde_json::to_value(&input.components)
            .map_err(|e| HelixError::validation(format!("components: {e}")))?;
        let hash = version_content_hash(&input.alphabet, &input.topology, &sequence, &components);
        let id = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO synthbio.design_versions
                (id, tenant_id, design_id, version, alphabet, topology, source_kind, source_name,
                 sequence_length, sequence_text, components, content_hash, provenance, notes,
                 created_by, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .bind(version)
        .bind(&input.alphabet)
        .bind(&input.topology)
        .bind(&input.source_kind)
        .bind(&input.source_name)
        .bind(sequence.len() as i32)
        .bind(&sequence)
        .bind(&components)
        .bind(&hash)
        .bind(&input.provenance)
        .bind(&input.notes)
        .bind(actor)
        .bind(Utc::now())
        .execute(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio insert version: {e}")))?;
        Ok(id)
    }

    /// Land an edit as a NEW immutable version — history is never rewritten.
    /// The design's current_version advances via a guarded UPDATE so a
    /// concurrent edit loses instead of forking the version sequence.
    pub async fn add_version(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        input: &VersionInput,
        actor: &str,
    ) -> HelixResult<DesignVersion> {
        validate_alphabet_topology(&input.alphabet, &input.topology)?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio version tx: {e}")))?;

        let design: Option<(i32, String)> = sqlx::query_as(
            "SELECT current_version, status FROM synthbio.registry_designs WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio lock design: {e}")))?;
        let (current, status) = design.ok_or_else(|| HelixError::not_found("design not found"))?;
        if status != "active" {
            return Err(HelixError::validation(format!(
                "cannot version a {status} design"
            )));
        }
        let next = current + 1;

        let version_id = self
            .insert_version(&mut tx, tenant_id, design_id, next, input, actor)
            .await?;

        let bumped: Option<(Uuid,)> = sqlx::query_as(
            r#"
            UPDATE synthbio.registry_designs
            SET current_version = $1, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND current_version = $5
            RETURNING id
            "#,
        )
        .bind(next)
        .bind(Utc::now())
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .bind(current)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio bump version: {e}")))?;
        if bumped.is_none() {
            return Err(HelixError::conflict(
                "design version advanced concurrently; retry",
            ));
        }

        self.add_edge(
            &mut tx,
            tenant_id,
            "design",
            design_id,
            "design_version",
            version_id,
            "contains",
        )
        .await?;
        let parent_version: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM synthbio.design_versions WHERE design_id = $1 AND version = $2",
        )
        .bind(design_id)
        .bind(current)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio parent version: {e}")))?;
        if let Some((pv,)) = parent_version {
            self.add_edge(
                &mut tx,
                tenant_id,
                "design_version",
                pv,
                "design_version",
                version_id,
                "derived-from",
            )
            .await?;
        }
        self.record_event(
            &mut tx,
            tenant_id,
            "design",
            design_id,
            "versioned",
            actor,
            serde_json::json!({"version": next, "source_kind": input.source_kind}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio version commit: {e}")))?;

        let v = self
            .get_version(tenant_id, design_id, next)
            .await?
            .ok_or_else(|| HelixError::internal("version vanished after commit"))?;
        Ok(v)
    }

    pub async fn get_design(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<Option<RegistryDesign>> {
        let row: Option<RegistryDesign> = sqlx::query_as(
            "SELECT id, tenant_id, accession, name, description, access_class, status, current_version, created_by, created_at, updated_at, deleted_at FROM synthbio.registry_designs WHERE tenant_id = $1 AND id = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio get design: {e}")))?;
        Ok(row)
    }

    pub async fn get_design_by_accession(
        &self,
        tenant_id: TenantId,
        accession: &str,
    ) -> HelixResult<Option<RegistryDesign>> {
        let row: Option<RegistryDesign> = sqlx::query_as(
            "SELECT id, tenant_id, accession, name, description, access_class, status, current_version, created_by, created_at, updated_at, deleted_at FROM synthbio.registry_designs WHERE tenant_id = $1 AND accession = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(accession)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio get by accession: {e}")))?;
        Ok(row)
    }

    pub async fn list_designs(
        &self,
        tenant_id: TenantId,
        include_deleted: bool,
    ) -> HelixResult<Vec<RegistryDesign>> {
        let rows: Vec<RegistryDesign> = if include_deleted {
            sqlx::query_as(
                "SELECT id, tenant_id, accession, name, description, access_class, status, current_version, created_by, created_at, updated_at, deleted_at FROM synthbio.registry_designs WHERE tenant_id = $1 ORDER BY created_at DESC",
            )
            .bind(tenant_id.as_uuid())
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(
                "SELECT id, tenant_id, accession, name, description, access_class, status, current_version, created_by, created_at, updated_at, deleted_at FROM synthbio.registry_designs WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
            )
            .bind(tenant_id.as_uuid())
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("synthbio list designs: {e}")))?;
        Ok(rows)
    }

    pub async fn get_version(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        version: i32,
    ) -> HelixResult<Option<DesignVersion>> {
        let row: Option<DesignVersion> = sqlx::query_as(
            "SELECT id, tenant_id, design_id, version, alphabet, topology, source_kind, source_name, sequence_length, sequence_text, components, content_hash, provenance, notes, created_by, created_at FROM synthbio.design_versions WHERE tenant_id = $1 AND design_id = $2 AND version = $3",
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio get version: {e}")))?;
        Ok(row)
    }

    pub async fn list_versions(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<Vec<DesignVersion>> {
        let rows: Vec<DesignVersion> = sqlx::query_as(
            "SELECT id, tenant_id, design_id, version, alphabet, topology, source_kind, source_name, sequence_length, sequence_text, components, content_hash, provenance, notes, created_by, created_at FROM synthbio.design_versions WHERE tenant_id = $1 AND design_id = $2 ORDER BY version DESC",
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio list versions: {e}")))?;
        Ok(rows)
    }

    /// The 360° view: design, versions, risk case + effective state,
    /// lineage edges (both directions), and the entity's event ledger.
    pub async fn design_360(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<Option<Design360>> {
        let design = self.get_design(tenant_id, design_id).await?;
        let Some(design) = design else {
            return Ok(None);
        };
        let versions = self.list_versions(tenant_id, design_id).await?;
        let risk_case = self.get_risk_case(tenant_id, design_id).await?;
        let effective = effective_risk(risk_case.as_ref());
        let edges: Vec<LineageEdge> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, parent_kind, parent_id, child_kind, child_id, relation, created_at
            FROM synthbio.lineage_edges
            WHERE tenant_id = $1
              AND ((parent_kind = 'design' AND parent_id = $2)
                OR (child_kind = 'design' AND child_id = $2)
                OR parent_id IN (SELECT id FROM synthbio.design_versions WHERE design_id = $2)
                OR child_id IN (SELECT id FROM synthbio.design_versions WHERE design_id = $2))
            ORDER BY created_at ASC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio 360 edges: {e}")))?;
        let events: Vec<LineageEvent> = sqlx::query_as(
            "SELECT id, tenant_id, entity_kind, entity_id, event_kind, actor, details, content_hash, prev_hash, created_at FROM synthbio.lineage_events WHERE tenant_id = $1 AND entity_kind = 'design' AND entity_id = $2 ORDER BY created_at ASC",
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio 360 events: {e}")))?;
        Ok(Some(Design360 {
            design,
            versions,
            risk_case,
            effective_risk: effective,
            edges,
            events,
        }))
    }

    // ——— risk review ———

    /// Open (or fetch) the risk case for a design. Starts at `unknown` —
    /// never at anything safer.
    pub async fn ensure_risk_case(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<RiskCase> {
        if let Some(case) = self.get_risk_case(tenant_id, design_id).await? {
            return Ok(case);
        }
        let id = Uuid::now_v7();
        let now = Utc::now();
        let row: Option<RiskCase> = sqlx::query_as(
            r#"
            INSERT INTO synthbio.risk_cases
                (id, tenant_id, design_id, state, created_at, updated_at)
            VALUES ($1,$2,$3,'unknown',$4,$4)
            ON CONFLICT DO NOTHING
            RETURNING id, tenant_id, design_id, design_version_id, state, intended_use,
                      policy_version, reasons, conditions, reviewer, decided_at, expires_at,
                      locked_at, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio ensure risk case: {e}")))?;
        match row {
            Some(r) => Ok(r),
            None => self
                .get_risk_case(tenant_id, design_id)
                .await?
                .ok_or_else(|| HelixError::internal("risk case vanished")),
        }
    }

    pub async fn get_risk_case(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<Option<RiskCase>> {
        let row: Option<RiskCase> = sqlx::query_as(
            "SELECT id, tenant_id, design_id, design_version_id, state, intended_use, policy_version, reasons, conditions, reviewer, decided_at, expires_at, locked_at, created_at, updated_at FROM synthbio.risk_cases WHERE tenant_id = $1 AND design_id = $2 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio get risk case: {e}")))?;
        Ok(row)
    }

    pub async fn risk_queue(&self, tenant_id: TenantId) -> HelixResult<Vec<(RiskCase, String)>> {
        #[derive(sqlx::FromRow)]
        struct QueueRow {
            id: Uuid,
            tenant_id: Uuid,
            design_id: Uuid,
            design_version_id: Option<Uuid>,
            state: String,
            intended_use: String,
            policy_version: String,
            reasons: JsonValue,
            conditions: String,
            reviewer: Option<String>,
            decided_at: Option<DateTime<Utc>>,
            expires_at: Option<DateTime<Utc>>,
            locked_at: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
            accession: String,
        }
        let rows: Vec<QueueRow> = sqlx::query_as(
            r#"
            SELECT c.id, c.tenant_id, c.design_id, c.design_version_id, c.state, c.intended_use,
                   c.policy_version, c.reasons, c.conditions, c.reviewer, c.decided_at,
                   c.expires_at, c.locked_at, c.created_at, c.updated_at, d.accession
            FROM synthbio.risk_cases c
            JOIN synthbio.registry_designs d ON d.id = c.design_id AND d.tenant_id = c.tenant_id
            WHERE c.tenant_id = $1 AND c.state = 'unknown' AND d.deleted_at IS NULL
            ORDER BY c.created_at ASC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio risk queue: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    RiskCase {
                        id: r.id,
                        tenant_id: r.tenant_id,
                        design_id: r.design_id,
                        design_version_id: r.design_version_id,
                        state: r.state,
                        intended_use: r.intended_use,
                        policy_version: r.policy_version,
                        reasons: r.reasons,
                        conditions: r.conditions,
                        reviewer: r.reviewer,
                        decided_at: r.decided_at,
                        expires_at: r.expires_at,
                        locked_at: r.locked_at,
                        created_at: r.created_at,
                        updated_at: r.updated_at,
                    },
                    r.accession,
                )
            })
            .collect())
    }

    /// Record a risk decision. Non-unknown states require a named human
    /// reviewer; the transition is a single guarded UPDATE, so a concurrent
    /// decision loses instead of overwriting.
    pub async fn review_risk(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        decision: &ReviewDecision,
        reviewer: &str,
    ) -> HelixResult<RiskCase> {
        if !RISK_STATES.contains(&decision.state.as_str()) {
            return Err(HelixError::validation(format!(
                "invalid risk state `{}` (allowed | restricted | blocked)",
                decision.state
            )));
        }
        if reviewer.trim().is_empty() {
            return Err(HelixError::validation(
                "a named human reviewer is required for risk decisions",
            ));
        }
        let case = self.ensure_risk_case(tenant_id, design_id).await?;
        if case.locked_at.is_some() {
            return Err(HelixError::validation(
                "risk decision is signed and locked; no further review is possible",
            ));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio review tx: {e}")))?;
        let now = Utc::now();
        // Pin the decision to the design's current version at review time.
        let pinned: Option<Uuid> = sqlx::query_scalar(
            "SELECT v.id FROM synthbio.design_versions v JOIN synthbio.registry_designs d ON d.id = v.design_id AND d.current_version = v.version WHERE d.tenant_id = $1 AND d.id = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio review pin version: {e}")))?;
        let updated: Option<(Uuid,)> = sqlx::query_as(
            r#"
            UPDATE synthbio.risk_cases
            SET state = $1, intended_use = $2, policy_version = $3, reasons = $4,
                conditions = $5, reviewer = $6, decided_at = $7, expires_at = $8,
                design_version_id = $9, updated_at = $7
            WHERE tenant_id = $10 AND id = $11 AND state = $12 AND locked_at IS NULL
            RETURNING id
            "#,
        )
        .bind(&decision.state)
        .bind(&decision.intended_use)
        .bind(&decision.policy_version)
        .bind(serde_json::to_value(&decision.reasons).unwrap_or_default())
        .bind(&decision.conditions)
        .bind(reviewer)
        .bind(now)
        .bind(decision.expires_at)
        .bind(pinned)
        .bind(tenant_id.as_uuid())
        .bind(case.id)
        .bind(
            decision
                .expected_state
                .clone()
                .unwrap_or_else(|| case.state.clone()),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio review update: {e}")))?;
        if updated.is_none() {
            return Err(HelixError::conflict(
                "risk case decided concurrently; refresh and re-review",
            ));
        }
        self.add_edge(
            &mut tx,
            tenant_id,
            "risk_case",
            case.id,
            "design",
            design_id,
            "reviews",
        )
        .await?;
        self.record_event(
            &mut tx,
            tenant_id,
            "design",
            design_id,
            "reviewed",
            reviewer,
            serde_json::json!({
                "state": decision.state,
                "policy_version": decision.policy_version,
                "reasons": decision.reasons,
            }),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio review commit: {e}")))?;
        self.get_risk_case(tenant_id, design_id)
            .await?
            .ok_or_else(|| HelixError::internal("risk case vanished after review"))
    }

    // ——— import ———

    /// Import a GenBank/FASTA body: each accepted record becomes an
    /// accessioned design with an immutable version 1; each rejected record
    /// lands in the quarantine manifest with its line number. Per-record
    /// isolation — accepted + rejected always sums to the input count.
    pub async fn import_records(
        &self,
        tenant_id: TenantId,
        format_hint: &str,
        content: &str,
        actor: &str,
    ) -> HelixResult<ImportManifest> {
        let parsed = parse_import(format_hint, content);
        let total = parsed.records.len() + parsed.errors.len();
        let mut accepted = Vec::new();
        let mut rejected: Vec<ImportRejected> = parsed
            .errors
            .iter()
            .map(|e| ImportRejected {
                record: e.record.clone(),
                line: e.line,
                reason: e.reason.clone(),
            })
            .collect();

        for rec in parsed.records {
            match self.import_one(tenant_id, &rec, format_hint, actor).await {
                Ok(design) => accepted.push(design),
                Err(e) => rejected.push(ImportRejected {
                    record: rec.name.clone(),
                    line: rec.source_line,
                    reason: e.to_string(),
                }),
            }
        }
        debug_assert_eq!(accepted.len() + rejected.len(), total);
        Ok(ImportManifest {
            total_records: total,
            accepted_count: accepted.len(),
            rejected_count: rejected.len(),
            accepted,
            rejected,
        })
    }

    async fn import_one(
        &self,
        tenant_id: TenantId,
        rec: &ParsedRecord,
        format_hint: &str,
        actor: &str,
    ) -> HelixResult<RegistryDesign> {
        let components: Vec<Component> = rec
            .features
            .iter()
            .filter(|f| f.key != "source")
            .map(|f| Component {
                name: if !f.gene.is_empty() {
                    f.gene.clone()
                } else if !f.product.is_empty() {
                    f.product.clone()
                } else {
                    f.key.clone()
                },
                role_so: role_so_for(&f.key).to_string(),
                start: f.loc.start,
                end: f.loc.end,
                strand: f.loc.strand,
                source: format!("{}:{}", rec.name, f.key),
            })
            .collect();
        let input = VersionInput {
            alphabet: rec.alphabet.clone(),
            topology: rec.topology.clone(),
            source_kind: if format_hint == "fasta" {
                "fasta".into()
            } else {
                "genbank".into()
            },
            source_name: if rec.accession.is_empty() {
                rec.name.clone()
            } else {
                rec.accession.clone()
            },
            sequence_text: rec.sequence.clone(),
            components,
            provenance: "depositor-claimed".into(),
            notes: rec.definition.clone(),
        };
        let mut design = self
            .create_design(
                tenant_id,
                &rec.name,
                &rec.definition,
                "internal",
                &input,
                actor,
            )
            .await?;
        // Imports carry the file's own name when the locus looks synthetic.
        if design.name != rec.name {
            design.name = rec.name.clone();
        }
        Ok(design)
    }

    // ——— evidence bundle ———

    pub async fn evidence_bundle(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<Option<EvidenceBundle>> {
        let Some(view) = self.design_360(tenant_id, design_id).await? else {
            return Ok(None);
        };
        let mut hasher_input = String::new();
        hasher_input.push_str(&view.design.accession);
        for v in &view.versions {
            hasher_input.push('|');
            hasher_input.push_str(&v.content_hash);
        }
        if let Some(rc) = &view.risk_case {
            hasher_input.push('|');
            hasher_input.push_str(&rc.state);
            hasher_input.push_str(rc.reviewer.as_deref().unwrap_or(""));
        }
        for e in &view.events {
            hasher_input.push('|');
            hasher_input.push_str(&e.content_hash);
        }
        let bundle = EvidenceBundle {
            bundle_version: "1.0".into(),
            generated_at: Utc::now(),
            tenant_id: tenant_id.as_uuid(),
            design: view.design,
            versions: view.versions,
            risk_case: view.risk_case,
            events: view.events,
            edges: view.edges,
            bundle_hash: sha256_hex(&hasher_input),
        };
        Ok(Some(bundle))
    }
}

fn validate_alphabet_topology(alphabet: &str, topology: &str) -> HelixResult<()> {
    if !["dna", "rna", "protein"].contains(&alphabet) {
        return Err(HelixError::validation(format!(
            "alphabet must be dna | rna | protein, got `{alphabet}`"
        )));
    }
    if !["linear", "circular"].contains(&topology) {
        return Err(HelixError::validation(format!(
            "topology must be linear | circular, got `{topology}`"
        )));
    }
    Ok(())
}

/// The state a case presents to the world: an expired decision decays to
/// `unknown` — never silently to anything safer.
fn effective_risk(case: Option<&RiskCase>) -> String {
    match case {
        None => "unknown".into(),
        Some(c) => {
            if c.state != "unknown" && c.expires_at.is_some_and(|e| e < Utc::now()) {
                "unknown".into()
            } else {
                c.state.clone()
            }
        }
    }
}

// ——— inventory (S2): samples + custody ———

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Sample {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub accession: String,
    pub name: String,
    pub kind: String,
    pub design_id: Option<Uuid>,
    pub status: String,
    pub location: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CustodyEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub sample_id: Uuid,
    pub event: String,
    pub from_location: String,
    pub to_location: String,
    pub actor: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleDetail {
    pub sample: Sample,
    pub custody: Vec<CustodyEvent>,
    pub edges: Vec<LineageEdge>,
    pub design_accession: Option<String>,
}

const SAMPLE_KINDS: [&str; 6] = [
    "strain",
    "plasmid_prep",
    "oligo",
    "protein",
    "cell_line",
    "other",
];
const CUSTODY_EVENTS: [&str; 8] = [
    "register",
    "transfer",
    "process",
    "consume",
    "store",
    "dispose",
    "aliquot",
    "reconcile",
];

// ——— measurements (S3): instrument observations with units + uncertainty ———

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Measurement {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub accession: String,
    pub sample_id: Uuid,
    pub design_version_id: Option<Uuid>,
    pub kind: String,
    pub method: String,
    pub value: Option<f64>,
    pub unit: String,
    pub uncertainty: Option<f64>,
    pub raw: JsonValue,
    pub status: String,
    pub analyst: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementInput {
    pub sample_id: Uuid,
    pub design_version_id: Option<Uuid>,
    pub kind: String,
    pub method: String,
    pub value: Option<f64>,
    pub unit: String,
    pub uncertainty: Option<f64>,
    pub raw: JsonValue,
}

const MEASUREMENT_KINDS: [&str; 6] = [
    "absorbance",
    "fluorescence",
    "qpcr",
    "gel",
    "ngs_qc",
    "other",
];

impl RegistryRepo {
    /// Record a measurement. The parent-sample guard is part of the INSERT
    /// itself: a sample deleted between check and write cannot leak data.
    pub async fn record_measurement(
        &self,
        tenant_id: TenantId,
        input: &MeasurementInput,
        analyst: &str,
    ) -> HelixResult<Measurement> {
        if !MEASUREMENT_KINDS.contains(&input.kind.as_str()) {
            return Err(HelixError::validation(format!(
                "kind must be one of {MEASUREMENT_KINDS:?}, got `{}`",
                input.kind
            )));
        }
        if input.value.is_none() && input.raw == serde_json::json!({}) {
            return Err(HelixError::validation(
                "a measurement needs a value or raw content",
            ));
        }
        if let Some(dv) = input.design_version_id {
            let exists: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM synthbio.design_versions WHERE tenant_id = $1 AND id = $2",
            )
            .bind(tenant_id.as_uuid())
            .bind(dv)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio measure version check: {e}")))?;
            if exists.is_none() {
                return Err(HelixError::not_found("design version not found"));
            }
        }

        let accession = self.next_accession(tenant_id, "measurement", "MSR").await?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio measure tx: {e}")))?;

        let id = Uuid::now_v7();
        let now = Utc::now();
        let row: Option<Measurement> = sqlx::query_as(
            r#"
            INSERT INTO synthbio.measurements
                (id, tenant_id, accession, sample_id, design_version_id, kind, method,
                 value, unit, uncertainty, raw, status, analyst, created_at, updated_at)
            SELECT $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,'draft',$12,$13,$13
            FROM synthbio.samples s
            WHERE s.tenant_id = $2 AND s.id = $4 AND s.deleted_at IS NULL
            RETURNING id, tenant_id, accession, sample_id, design_version_id, kind, method,
                      value, unit, uncertainty, raw, status, analyst, created_at, updated_at,
                      NULL AS deleted_at
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(&accession)
        .bind(input.sample_id)
        .bind(input.design_version_id)
        .bind(&input.kind)
        .bind(&input.method)
        .bind(input.value)
        .bind(&input.unit)
        .bind(input.uncertainty)
        .bind(&input.raw)
        .bind(analyst)
        .bind(now)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio record measurement: {e}")))?;
        let m = row.ok_or_else(|| HelixError::not_found("sample not found"))?;

        self.add_edge(
            &mut tx,
            tenant_id,
            "sample",
            input.sample_id,
            "measurement",
            id,
            "measured",
        )
        .await?;
        if let Some(dv) = input.design_version_id {
            self.add_edge(
                &mut tx,
                tenant_id,
                "design_version",
                dv,
                "measurement",
                id,
                "characterizes",
            )
            .await?;
        }
        self.record_event(
            &mut tx,
            tenant_id,
            "sample",
            input.sample_id,
            "measured",
            analyst,
            serde_json::json!({"accession": accession, "kind": input.kind}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio measure commit: {e}")))?;
        Ok(m)
    }

    pub async fn list_measurements(
        &self,
        tenant_id: TenantId,
        sample_id: Uuid,
    ) -> HelixResult<Vec<Measurement>> {
        let rows: Vec<Measurement> = sqlx::query_as(
            "SELECT id, tenant_id, accession, sample_id, design_version_id, kind, method, value, unit, uncertainty, raw, status, analyst, created_at, updated_at, deleted_at FROM synthbio.measurements WHERE tenant_id = $1 AND sample_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(tenant_id.as_uuid())
        .bind(sample_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio list measurements: {e}")))?;
        Ok(rows)
    }

    /// Accept or reject a draft measurement — single guarded UPDATE with
    /// the expected-from status, so a concurrent verdict loses.
    pub async fn transition_measurement(
        &self,
        tenant_id: TenantId,
        measurement_id: Uuid,
        action: &str,
        analyst: &str,
    ) -> HelixResult<Measurement> {
        let next = match action {
            "accept" => "accepted",
            "reject" => "rejected",
            other => {
                return Err(HelixError::validation(format!(
                    "cannot {other} a measurement"
                )))
            }
        };
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                HelixError::dependency(format!("synthbio measure transition tx: {e}"))
            })?;
        let row: Option<Measurement> = sqlx::query_as(
            r#"
            UPDATE synthbio.measurements
            SET status = $1, analyst = $2, updated_at = $3
            WHERE tenant_id = $4 AND id = $5 AND status = 'draft' AND deleted_at IS NULL
            RETURNING id, tenant_id, accession, sample_id, design_version_id, kind, method,
                      value, unit, uncertainty, raw, status, analyst, created_at, updated_at,
                      deleted_at
            "#,
        )
        .bind(next)
        .bind(analyst)
        .bind(Utc::now())
        .bind(tenant_id.as_uuid())
        .bind(measurement_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio measure transition: {e}")))?;
        let m =
            row.ok_or_else(|| HelixError::conflict("measurement already decided or not found"))?;
        self.record_event(
            &mut tx,
            tenant_id,
            "sample",
            m.sample_id,
            &format!("measurement_{next}"),
            analyst,
            serde_json::json!({"accession": m.accession}),
        )
        .await?;
        tx.commit().await.map_err(|e| {
            HelixError::dependency(format!("synthbio measure transition commit: {e}"))
        })?;
        Ok(m)
    }
}

// ——— claims (S4): statements + evidence links + ELN notes ———

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Claim {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub accession: String,
    pub design_id: Uuid,
    pub statement: String,
    pub status: String,
    pub attested_by: Option<String>,
    pub attested_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EvidenceLink {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub claim_id: Uuid,
    pub target_kind: String,
    pub target_id: Uuid,
    pub support: String,
    pub note: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DesignNote {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub design_id: Uuid,
    pub body: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimDetail {
    pub claim: Claim,
    pub evidence: Vec<EvidenceLink>,
}

const EVIDENCE_KINDS: [&str; 3] = ["measurement", "design_version", "analysis"];
const SUPPORT_KINDS: [&str; 3] = ["supports", "conflicts", "missing"];

impl RegistryRepo {
    /// Open a claim on a design. The parent-design guard is part of the
    /// INSERT itself: a deleted design cannot acquire claims.
    pub async fn create_claim(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        statement: &str,
        actor: &str,
    ) -> HelixResult<Claim> {
        if statement.trim().is_empty() {
            return Err(HelixError::validation("claim statement required"));
        }
        let accession = self.next_accession(tenant_id, "claim", "CLM").await?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio claim tx: {e}")))?;

        let id = Uuid::now_v7();
        let now = Utc::now();
        let row: Option<Claim> = sqlx::query_as(
            r#"
            INSERT INTO synthbio.claims
                (id, tenant_id, accession, design_id, statement, status, created_by, created_at, updated_at)
            SELECT $1,$2,$3,$4,$5,'draft',$6,$7,$7
            FROM synthbio.registry_designs d
            WHERE d.tenant_id = $2 AND d.id = $4 AND d.deleted_at IS NULL
            RETURNING id, tenant_id, accession, design_id, statement, status, attested_by,
                      attested_at, created_by, created_at, updated_at, NULL AS deleted_at
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(&accession)
        .bind(design_id)
        .bind(statement)
        .bind(actor)
        .bind(now)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio create claim: {e}")))?;
        let claim = row.ok_or_else(|| HelixError::not_found("design not found"))?;

        self.add_edge(
            &mut tx, tenant_id, "claim", id, "design", design_id, "about",
        )
        .await?;
        self.record_event(
            &mut tx,
            tenant_id,
            "design",
            design_id,
            "claim_created",
            actor,
            serde_json::json!({"accession": accession}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio claim commit: {e}")))?;
        Ok(claim)
    }

    /// Link evidence to a claim. Links are append-only (DB trigger).
    pub async fn link_evidence(
        &self,
        tenant_id: TenantId,
        claim_id: Uuid,
        target_kind: &str,
        target_id: Uuid,
        support: &str,
        note: &str,
        actor: &str,
    ) -> HelixResult<EvidenceLink> {
        if !EVIDENCE_KINDS.contains(&target_kind) {
            return Err(HelixError::validation(format!(
                "target_kind must be one of {EVIDENCE_KINDS:?}, got `{target_kind}`"
            )));
        }
        if !SUPPORT_KINDS.contains(&support) {
            return Err(HelixError::validation(format!(
                "support must be one of {SUPPORT_KINDS:?}, got `{support}`"
            )));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio evidence tx: {e}")))?;

        let claim: Option<(Uuid, Uuid)> = sqlx::query_as(
            "SELECT id, design_id FROM synthbio.claims WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(tenant_id.as_uuid())
        .bind(claim_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio evidence claim lock: {e}")))?;
        let (claim_id, design_id) =
            claim.ok_or_else(|| HelixError::not_found("claim not found"))?;

        // Referential integrity on the evidence target.
        let target_exists: bool = match target_kind {
            "measurement" => {
                sqlx::query_scalar::<_, Option<Uuid>>(
                    "SELECT id FROM synthbio.measurements WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
                )
                .bind(tenant_id.as_uuid())
                .bind(target_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| HelixError::dependency(format!("synthbio evidence target: {e}")))?
                .is_some()
            }
            "design_version" => {
                sqlx::query_scalar::<_, Option<Uuid>>(
                    "SELECT id FROM synthbio.design_versions WHERE tenant_id = $1 AND id = $2",
                )
                .bind(tenant_id.as_uuid())
                .bind(target_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| HelixError::dependency(format!("synthbio evidence target: {e}")))?
                .is_some()
            }
            _ => true, // analysis: external reference, no local table
        };
        if !target_exists {
            return Err(HelixError::not_found(format!("{target_kind} not found")));
        }

        let id = Uuid::now_v7();
        let link: EvidenceLink = sqlx::query_as(
            r#"
            INSERT INTO synthbio.evidence_links
                (id, tenant_id, claim_id, target_kind, target_id, support, note, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING id, tenant_id, claim_id, target_kind, target_id, support, note, created_at
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(claim_id)
        .bind(target_kind)
        .bind(target_id)
        .bind(support)
        .bind(note)
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio link evidence: {e}")))?;

        self.add_edge(
            &mut tx,
            tenant_id,
            "claim",
            claim_id,
            target_kind,
            target_id,
            support,
        )
        .await?;
        self.record_event(
            &mut tx,
            tenant_id,
            "design",
            design_id,
            "evidence_linked",
            actor,
            serde_json::json!({"claim": claim_id, "support": support, "target_kind": target_kind}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio evidence commit: {e}")))?;
        Ok(link)
    }

    pub async fn list_claims(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<Vec<ClaimDetail>> {
        let claims: Vec<Claim> = sqlx::query_as(
            "SELECT id, tenant_id, accession, design_id, statement, status, attested_by, attested_at, created_by, created_at, updated_at, deleted_at FROM synthbio.claims WHERE tenant_id = $1 AND design_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio list claims: {e}")))?;
        let mut out = Vec::new();
        for claim in claims {
            let evidence: Vec<EvidenceLink> = sqlx::query_as(
                "SELECT id, tenant_id, claim_id, target_kind, target_id, support, note, created_at FROM synthbio.evidence_links WHERE tenant_id = $1 AND claim_id = $2 ORDER BY created_at ASC",
            )
            .bind(tenant_id.as_uuid())
            .bind(claim.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio claim evidence: {e}")))?;
            out.push(ClaimDetail { claim, evidence });
        }
        Ok(out)
    }

    /// Human attestation: draft|under_review → accepted with a named
    /// attester, single guarded UPDATE so a concurrent attestation loses.
    pub async fn attest_claim(
        &self,
        tenant_id: TenantId,
        claim_id: Uuid,
        attestor: &str,
    ) -> HelixResult<Claim> {
        if attestor.trim().is_empty() {
            return Err(HelixError::validation("a named human attester is required"));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio attest tx: {e}")))?;
        let row: Option<Claim> = sqlx::query_as(
            r#"
            UPDATE synthbio.claims
            SET status = 'accepted', attested_by = $1, attested_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status IN ('draft','under_review') AND deleted_at IS NULL
            RETURNING id, tenant_id, accession, design_id, statement, status, attested_by,
                      attested_at, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(attestor)
        .bind(Utc::now())
        .bind(tenant_id.as_uuid())
        .bind(claim_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio attest: {e}")))?;
        let claim =
            row.ok_or_else(|| HelixError::conflict("claim already attested or not found"))?;
        self.record_event(
            &mut tx,
            tenant_id,
            "design",
            claim.design_id,
            "claim_attested",
            attestor,
            serde_json::json!({"accession": claim.accession}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio attest commit: {e}")))?;
        Ok(claim)
    }

    /// Challenge a claim — it becomes `challenged` without erasing history.
    pub async fn challenge_claim(
        &self,
        tenant_id: TenantId,
        claim_id: Uuid,
        reason: &str,
        actor: &str,
    ) -> HelixResult<Claim> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio challenge tx: {e}")))?;
        let row: Option<Claim> = sqlx::query_as(
            r#"
            UPDATE synthbio.claims
            SET status = 'challenged', updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND status IN ('draft','under_review','accepted') AND deleted_at IS NULL
            RETURNING id, tenant_id, accession, design_id, statement, status, attested_by,
                      attested_at, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(Utc::now())
        .bind(tenant_id.as_uuid())
        .bind(claim_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio challenge: {e}")))?;
        let claim =
            row.ok_or_else(|| HelixError::conflict("claim already challenged or not found"))?;
        self.record_event(
            &mut tx,
            tenant_id,
            "design",
            claim.design_id,
            "claim_challenged",
            actor,
            serde_json::json!({"accession": claim.accession, "reason": reason}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio challenge commit: {e}")))?;
        Ok(claim)
    }

    // ——— ELN notes ———

    /// Append an ELN note to a design (append-only, DB trigger).
    pub async fn add_note(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        body: &str,
        actor: &str,
    ) -> HelixResult<DesignNote> {
        if body.trim().is_empty() {
            return Err(HelixError::validation("note body required"));
        }
        let id = Uuid::now_v7();
        let row: Option<DesignNote> = sqlx::query_as(
            r#"
            INSERT INTO synthbio.notes (id, tenant_id, design_id, body, created_by, created_at)
            SELECT $1,$2,$3,$4,$5,$6
            FROM synthbio.registry_designs d
            WHERE d.tenant_id = $2 AND d.id = $3 AND d.deleted_at IS NULL
            RETURNING id, tenant_id, design_id, body, created_by, created_at
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .bind(body)
        .bind(actor)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio add note: {e}")))?;
        row.ok_or_else(|| HelixError::not_found("design not found"))
    }

    pub async fn list_notes(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<Vec<DesignNote>> {
        let rows: Vec<DesignNote> = sqlx::query_as(
            "SELECT id, tenant_id, design_id, body, created_by, created_at FROM synthbio.notes WHERE tenant_id = $1 AND design_id = $2 ORDER BY created_at DESC",
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio list notes: {e}")))?;
        Ok(rows)
    }
}

impl RegistryRepo {
    /// Register an accessioned sample, optionally linked to a design — one tx.
    pub async fn register_sample(
        &self,
        tenant_id: TenantId,
        name: &str,
        kind: &str,
        design_id: Option<Uuid>,
        location: &str,
        actor: &str,
    ) -> HelixResult<Sample> {
        if name.trim().is_empty() {
            return Err(HelixError::validation("sample name required"));
        }
        if !SAMPLE_KINDS.contains(&kind) {
            return Err(HelixError::validation(format!(
                "kind must be one of {SAMPLE_KINDS:?}, got `{kind}`"
            )));
        }
        let accession = self.next_accession(tenant_id, "sample", "SMP").await?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio sample tx: {e}")))?;

        if let Some(did) = design_id {
            let exists: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM synthbio.registry_designs WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
            )
            .bind(tenant_id.as_uuid())
            .bind(did)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio sample design check: {e}")))?;
            if exists.is_none() {
                return Err(HelixError::not_found("design not found"));
            }
        }

        let id = Uuid::now_v7();
        let now = Utc::now();
        let sample: Sample = sqlx::query_as(
            r#"
            INSERT INTO synthbio.samples
                (id, tenant_id, accession, name, kind, design_id, status, location, created_by, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,'active',$7,$8,$9,$9)
            RETURNING id, tenant_id, accession, name, kind, design_id, status, location, created_by, created_at, updated_at, NULL AS deleted_at
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(&accession)
        .bind(name)
        .bind(kind)
        .bind(design_id)
        .bind(location)
        .bind(actor)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio register sample: {e}")))?;

        if let Some(did) = design_id {
            self.add_edge(&mut tx, tenant_id, "design", did, "sample", id, "produces")
                .await?;
        }
        self.append_custody(
            &mut tx,
            tenant_id,
            id,
            "register",
            "",
            location,
            actor,
            "registered",
        )
        .await?;
        self.record_event(
            &mut tx,
            tenant_id,
            "sample",
            id,
            "registered",
            actor,
            serde_json::json!({"accession": accession, "kind": kind}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio sample commit: {e}")))?;
        Ok(sample)
    }

    async fn append_custody(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: TenantId,
        sample_id: Uuid,
        event: &str,
        from_location: &str,
        to_location: &str,
        actor: &str,
        notes: &str,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"
            INSERT INTO synthbio.custody_events
                (id, tenant_id, sample_id, event, from_location, to_location, actor, notes, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(tenant_id.as_uuid())
        .bind(sample_id)
        .bind(event)
        .bind(from_location)
        .bind(to_location)
        .bind(actor)
        .bind(notes)
        .bind(Utc::now())
        .execute(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio custody append: {e}")))?;
        Ok(())
    }

    /// Record a custody event and move the sample's current location inside
    /// the same transaction — custody and location can never disagree. The
    /// sample row is locked FOR UPDATE so concurrent moves serialize.
    pub async fn custody_event(
        &self,
        tenant_id: TenantId,
        sample_id: Uuid,
        event: &str,
        to_location: &str,
        actor: &str,
        notes: &str,
    ) -> HelixResult<Sample> {
        if !CUSTODY_EVENTS.contains(&event) {
            return Err(HelixError::validation(format!(
                "event must be one of {CUSTODY_EVENTS:?}, got `{event}`"
            )));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio custody tx: {e}")))?;

        let sample: Option<(String, String)> = sqlx::query_as(
            "SELECT status, location FROM synthbio.samples WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(tenant_id.as_uuid())
        .bind(sample_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio custody lock: {e}")))?;
        let (status, from_location) =
            sample.ok_or_else(|| HelixError::not_found("sample not found"))?;
        if status != "active" {
            return Err(HelixError::validation(format!(
                "cannot move a {status} sample"
            )));
        }

        self.append_custody(
            &mut tx,
            tenant_id,
            sample_id,
            event,
            &from_location,
            to_location,
            actor,
            notes,
        )
        .await?;

        let new_location = if to_location.is_empty() {
            from_location
        } else {
            to_location.to_string()
        };
        let updated: Option<(Uuid,)> = sqlx::query_as(
            "UPDATE synthbio.samples SET location = $1, updated_at = $2 WHERE tenant_id = $3 AND id = $4 RETURNING id",
        )
        .bind(&new_location)
        .bind(Utc::now())
        .bind(tenant_id.as_uuid())
        .bind(sample_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio custody move: {e}")))?;
        if updated.is_none() {
            return Err(HelixError::conflict("sample moved concurrently; retry"));
        }

        self.record_event(
            &mut tx,
            tenant_id,
            "sample",
            sample_id,
            event,
            actor,
            serde_json::json!({"to": new_location}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio custody commit: {e}")))?;
        self.get_sample(tenant_id, sample_id)
            .await?
            .ok_or_else(|| HelixError::internal("sample vanished after custody"))
    }

    /// Create a child sample (aliquot) with a derived-from lineage edge —
    /// one tx, so the split is serialized.
    pub async fn aliquot(
        &self,
        tenant_id: TenantId,
        parent_sample_id: Uuid,
        name: &str,
        actor: &str,
    ) -> HelixResult<Sample> {
        let parent = self
            .get_sample(tenant_id, parent_sample_id)
            .await?
            .ok_or_else(|| HelixError::not_found("parent sample not found"))?;
        let child = self
            .register_sample(
                tenant_id,
                name,
                &parent.kind,
                parent.design_id,
                &parent.location,
                actor,
            )
            .await?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio aliquot tx: {e}")))?;
        self.add_edge(
            &mut tx,
            tenant_id,
            "sample",
            parent_sample_id,
            "sample",
            child.id,
            "derived-from",
        )
        .await?;
        self.append_custody(
            &mut tx,
            tenant_id,
            parent_sample_id,
            "aliquot",
            &parent.location,
            &parent.location,
            actor,
            &format!("aliquot → {}", child.accession),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio aliquot commit: {e}")))?;
        Ok(child)
    }

    pub async fn get_sample(
        &self,
        tenant_id: TenantId,
        sample_id: Uuid,
    ) -> HelixResult<Option<Sample>> {
        let row: Option<Sample> = sqlx::query_as(
            "SELECT id, tenant_id, accession, name, kind, design_id, status, location, created_by, created_at, updated_at, deleted_at FROM synthbio.samples WHERE tenant_id = $1 AND id = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(sample_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio get sample: {e}")))?;
        Ok(row)
    }

    pub async fn list_samples(&self, tenant_id: TenantId) -> HelixResult<Vec<Sample>> {
        let rows: Vec<Sample> = sqlx::query_as(
            "SELECT id, tenant_id, accession, name, kind, design_id, status, location, created_by, created_at, updated_at, deleted_at FROM synthbio.samples WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio list samples: {e}")))?;
        Ok(rows)
    }

    pub async fn sample_detail(
        &self,
        tenant_id: TenantId,
        sample_id: Uuid,
    ) -> HelixResult<Option<SampleDetail>> {
        let Some(sample) = self.get_sample(tenant_id, sample_id).await? else {
            return Ok(None);
        };
        let custody: Vec<CustodyEvent> = sqlx::query_as(
            "SELECT id, tenant_id, sample_id, event, from_location, to_location, actor, notes, created_at FROM synthbio.custody_events WHERE tenant_id = $1 AND sample_id = $2 ORDER BY created_at ASC",
        )
        .bind(tenant_id.as_uuid())
        .bind(sample_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio sample custody: {e}")))?;
        let edges: Vec<LineageEdge> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, parent_kind, parent_id, child_kind, child_id, relation, created_at
            FROM synthbio.lineage_edges
            WHERE tenant_id = $1
              AND ((parent_kind = 'sample' AND parent_id = $2)
                OR (child_kind = 'sample' AND child_id = $2))
            ORDER BY created_at ASC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(sample_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio sample edges: {e}")))?;
        let design_accession: Option<String> = if let Some(did) = sample.design_id {
            sqlx::query_scalar(
                "SELECT accession FROM synthbio.registry_designs WHERE tenant_id = $1 AND id = $2",
            )
            .bind(tenant_id.as_uuid())
            .bind(did)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio sample design: {e}")))?
        } else {
            None
        };
        Ok(Some(SampleDetail {
            sample,
            custody,
            edges,
            design_accession,
        }))
    }
}

// ——— e-signatures (S5): sign ⇒ lock ———

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Signature {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub target_kind: String,
    pub target_id: Uuid,
    pub signer: String,
    pub meaning: String,
    pub statement: String,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
}

const SIGN_MEANINGS: [&str; 3] = ["approved", "witnessed", "reviewed"];
const SIGN_KINDS: [&str; 2] = ["design_version", "risk_case"];

impl RegistryRepo {
    /// Sign a target. The signature is append-only; for a risk case the
    /// decision locks permanently against further reviews in the same tx.
    /// Content hash pins exactly what was signed.
    pub async fn sign_target(
        &self,
        tenant_id: TenantId,
        target_kind: &str,
        target_id: Uuid,
        signer: &str,
        meaning: &str,
        statement: &str,
    ) -> HelixResult<Signature> {
        if !SIGN_KINDS.contains(&target_kind) {
            return Err(HelixError::validation(format!(
                "target_kind must be one of {SIGN_KINDS:?}, got `{target_kind}`"
            )));
        }
        if !SIGN_MEANINGS.contains(&meaning) {
            return Err(HelixError::validation(format!(
                "meaning must be one of {SIGN_MEANINGS:?}, got `{meaning}`"
            )));
        }
        if signer.trim().is_empty() {
            return Err(HelixError::validation("a named human signer is required"));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio sign tx: {e}")))?;

        // Resolve what is being signed (hash + owner design for the event).
        #[derive(sqlx::FromRow)]
        struct SignableCase {
            state: String,
            reviewer: Option<String>,
            policy_version: String,
            reasons: JsonValue,
            decided_at: Option<DateTime<Utc>>,
            locked_at: Option<DateTime<Utc>>,
            design_id: Uuid,
        }
        let (content_hash, owner_design): (String, Uuid) = match target_kind {
            "design_version" => {
                let row: Option<(String, Uuid)> = sqlx::query_as(
                    "SELECT content_hash, design_id FROM synthbio.design_versions WHERE tenant_id = $1 AND id = $2",
                )
                .bind(tenant_id.as_uuid())
                .bind(target_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| HelixError::dependency(format!("synthbio sign version: {e}")))?;
                row.ok_or_else(|| HelixError::not_found("design version not found"))?
            }
            _ => {
                let row: Option<SignableCase> = sqlx::query_as(
                    "SELECT state, reviewer, policy_version, reasons, decided_at, locked_at, design_id FROM synthbio.risk_cases WHERE tenant_id = $1 AND id = $2",
                )
                .bind(tenant_id.as_uuid())
                .bind(target_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| HelixError::dependency(format!("synthbio sign case: {e}")))?;
                let case = row.ok_or_else(|| HelixError::not_found("risk case not found"))?;
                if case.locked_at.is_some() {
                    return Err(HelixError::conflict(
                        "risk decision is already signed and locked",
                    ));
                }
                if case.decided_at.is_none() || case.state == "unknown" {
                    return Err(HelixError::validation(
                        "an undecided risk case cannot be signed",
                    ));
                }
                let hash = sha256_hex(&format!(
                    "{}|{}|{}|{}",
                    case.state,
                    case.reviewer.unwrap_or_default(),
                    case.policy_version,
                    serde_json::to_string(&case.reasons).unwrap_or_default()
                ));
                (hash, case.design_id)
            }
        };

        let id = Uuid::now_v7();
        let sig: Signature = sqlx::query_as(
            r#"
            INSERT INTO synthbio.signatures
                (id, tenant_id, target_kind, target_id, signer, meaning, statement, content_hash, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING id, tenant_id, target_kind, target_id, signer, meaning, statement, content_hash, created_at
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(target_kind)
        .bind(target_id)
        .bind(signer)
        .bind(meaning)
        .bind(statement)
        .bind(&content_hash)
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                HelixError::conflict("target already signed with this meaning")
            } else {
                HelixError::dependency(format!("synthbio sign insert: {e}"))
            }
        })?;

        if target_kind == "risk_case" && meaning == "approved" {
            let locked: Option<(Uuid,)> = sqlx::query_as(
                "UPDATE synthbio.risk_cases SET locked_at = $1, updated_at = $1 WHERE tenant_id = $2 AND id = $3 AND locked_at IS NULL RETURNING id",
            )
            .bind(Utc::now())
            .bind(tenant_id.as_uuid())
            .bind(target_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio sign lock: {e}")))?;
            if locked.is_none() {
                return Err(HelixError::conflict(
                    "risk decision locked concurrently; retry",
                ));
            }
        }

        self.record_event(
            &mut tx,
            tenant_id,
            "design",
            owner_design,
            &format!("signed_{meaning}"),
            signer,
            serde_json::json!({"target_kind": target_kind, "target_id": target_id}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio sign commit: {e}")))?;
        Ok(sig)
    }

    pub async fn list_signatures(
        &self,
        tenant_id: TenantId,
        target_kind: &str,
        target_id: Uuid,
    ) -> HelixResult<Vec<Signature>> {
        let rows: Vec<Signature> = sqlx::query_as(
            "SELECT id, tenant_id, target_kind, target_id, signer, meaning, statement, content_hash, created_at FROM synthbio.signatures WHERE tenant_id = $1 AND target_kind = $2 AND target_id = $3 ORDER BY created_at ASC",
        )
        .bind(tenant_id.as_uuid())
        .bind(target_kind)
        .bind(target_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio list signatures: {e}")))?;
        Ok(rows)
    }

    /// Every signature touching a design: its versions and its risk case.
    pub async fn design_signatures(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<Vec<Signature>> {
        let rows: Vec<Signature> = sqlx::query_as(
            r#"
            SELECT s.id, s.tenant_id, s.target_kind, s.target_id, s.signer, s.meaning, s.statement, s.content_hash, s.created_at
            FROM synthbio.signatures s
            WHERE s.tenant_id = $1
              AND ((s.target_kind = 'design_version' AND s.target_id IN
                    (SELECT id FROM synthbio.design_versions WHERE design_id = $2))
                OR (s.target_kind = 'risk_case' AND s.target_id IN
                    (SELECT id FROM synthbio.risk_cases WHERE design_id = $2)))
            ORDER BY s.created_at ASC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio design signatures: {e}")))?;
        Ok(rows)
    }
}

// ——— journeys: intent-first goals decomposed into checkable stages ———

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Journey {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub accession: String,
    pub title: String,
    pub intent: String,
    pub pathway_key: String,
    pub route_choice: String,
    pub status: String,
    pub current_stage: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JourneyStage {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub journey_id: Uuid,
    pub stage_index: i32,
    pub stage_key: String,
    pub status: String,
    pub target_kind: Option<String>,
    pub target_id: Option<Uuid>,
    pub summary: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCheck {
    pub met: bool,
    pub missing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageWithCheck {
    #[serde(flatten)]
    pub stage: JourneyStage,
    pub check: StageCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyDetail {
    pub journey: Journey,
    pub stages: Vec<StageWithCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathwayStage {
    pub stage_key: String,
    pub title: String,
    pub explanation: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathwayTemplate {
    pub key: String,
    pub title: String,
    pub description: String,
    pub stages: Vec<PathwayStage>,
}

const STAGE_SOURCE: usize = 0;
const STAGE_ROUTE: usize = 1;
const STAGE_DESIGN: usize = 2;
const STAGE_RISK: usize = 3;
const STAGE_BUILD: usize = 4;
const STAGE_TEST: usize = 5;
const STAGE_EVIDENCE: usize = 6;

/// The pathway templates — the teaching layer. Explanations are the product.
pub fn pathway_templates() -> Vec<PathwayTemplate> {
    vec![
        PathwayTemplate {
            key: "plant-to-topical".into(),
            title: "Plant → topical product".into(),
            description: "From a plant or natural source to a testable topical candidate, with risk review and evidence at every step.".into(),
            stages: spine_stages(),
        },
        PathwayTemplate {
            key: "microbe-to-ingredient".into(),
            title: "Microbe → ingredient".into(),
            description: "From an engineered or wild microbe to a characterized ingredient or compound.".into(),
            stages: spine_stages(),
        },
        PathwayTemplate {
            key: "blank".into(),
            title: "Blank journey".into(),
            description: "The same seven-stage spine with no domain framing — for any idea you want to make testable.".into(),
            stages: spine_stages(),
        },
    ]
}

fn spine_stages() -> Vec<PathwayStage> {
    vec![
        PathwayStage {
            stage_key: "source".into(),
            title: "Source material".into(),
            explanation: "Register the plant or starting material as a sample. Provenance starts here — where it came from, who provided it.".into(),
            mode: "link_sample".into(),
        },
        PathwayStage {
            stage_key: "route".into(),
            title: "Choose the route".into(),
            explanation: "Extract the active compound from the plant, or engineer a microbe to produce it. This choice shapes everything downstream.".into(),
            mode: "route_choice".into(),
        },
        PathwayStage {
            stage_key: "design".into(),
            title: "Design the construct".into(),
            explanation: "The genetic design or extraction plan — versioned and immutable. Import a GenBank file or author it here.".into(),
            mode: "link_design".into(),
        },
        PathwayStage {
            stage_key: "risk".into(),
            title: "Risk review".into(),
            explanation: "A named human biosafety authority decides. Unknown is never safe — missing evidence blocks until a person reviews it.".into(),
            mode: "auto_risk".into(),
        },
        PathwayStage {
            stage_key: "build".into(),
            title: "Build the physical form".into(),
            explanation: "The physical prep, strain, or extract — a sample derived from the design, with custody.".into(),
            mode: "link_build_sample".into(),
        },
        PathwayStage {
            stage_key: "test".into(),
            title: "Test it".into(),
            explanation: "Measurements with method, unit, and uncertainty. An accepted measurement is the first real evidence.".into(),
            mode: "auto_test".into(),
        },
        PathwayStage {
            stage_key: "evidence".into(),
            title: "Claim and prove".into(),
            explanation: "What does the evidence actually support? Claims link to supporting and conflicting results, attested by a named human.".into(),
            mode: "auto_claim".into(),
        },
    ]
}

impl RegistryRepo {
    /// Create a journey: accessioned goal with its stage spine instantiated.
    pub async fn create_journey(
        &self,
        tenant_id: TenantId,
        title: &str,
        intent: &str,
        pathway_key: &str,
        actor: &str,
    ) -> HelixResult<Journey> {
        if title.trim().is_empty() {
            return Err(HelixError::validation("journey title required"));
        }
        let template = pathway_templates()
            .into_iter()
            .find(|p| p.key == pathway_key)
            .ok_or_else(|| HelixError::validation(format!("unknown pathway `{pathway_key}`")))?;
        let accession = self.next_accession(tenant_id, "journey", "JRN").await?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio journey tx: {e}")))?;

        let id = Uuid::now_v7();
        let now = Utc::now();
        let journey: Journey = sqlx::query_as(
            r#"
            INSERT INTO synthbio.journeys
                (id, tenant_id, accession, title, intent, pathway_key, route_choice, status,
                 current_stage, created_by, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,'undecided','active',0,$7,$8,$8)
            RETURNING id, tenant_id, accession, title, intent, pathway_key, route_choice,
                      status, current_stage, created_by, created_at, updated_at, NULL AS deleted_at
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(&accession)
        .bind(title)
        .bind(intent)
        .bind(pathway_key)
        .bind(actor)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio create journey: {e}")))?;

        for (i, stage) in template.stages.iter().enumerate() {
            let status = if i == 0 { "current" } else { "pending" };
            sqlx::query(
                r#"
                INSERT INTO synthbio.journey_stages
                    (id, tenant_id, journey_id, stage_index, stage_key, status, summary, created_at, updated_at)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$8)
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(tenant_id.as_uuid())
            .bind(id)
            .bind(i as i32)
            .bind(&stage.stage_key)
            .bind(status)
            .bind(&stage.title)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio journey stage: {e}")))?;
        }

        self.record_event(
            &mut tx,
            tenant_id,
            "journey",
            id,
            "created",
            actor,
            serde_json::json!({"accession": accession, "pathway": pathway_key}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio journey commit: {e}")))?;
        Ok(journey)
    }

    pub async fn get_journey(
        &self,
        tenant_id: TenantId,
        journey_id: Uuid,
    ) -> HelixResult<Option<Journey>> {
        let row: Option<Journey> = sqlx::query_as(
            "SELECT id, tenant_id, accession, title, intent, pathway_key, route_choice, status, current_stage, created_by, created_at, updated_at, deleted_at FROM synthbio.journeys WHERE tenant_id = $1 AND id = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(journey_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio get journey: {e}")))?;
        Ok(row)
    }

    pub async fn list_journeys(&self, tenant_id: TenantId) -> HelixResult<Vec<Journey>> {
        let rows: Vec<Journey> = sqlx::query_as(
            "SELECT id, tenant_id, accession, title, intent, pathway_key, route_choice, status, current_stage, created_by, created_at, updated_at, deleted_at FROM synthbio.journeys WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio list journeys: {e}")))?;
        Ok(rows)
    }

    async fn journey_stages(
        &self,
        tenant_id: TenantId,
        journey_id: Uuid,
    ) -> HelixResult<Vec<JourneyStage>> {
        let rows: Vec<JourneyStage> = sqlx::query_as(
            "SELECT id, tenant_id, journey_id, stage_index, stage_key, status, target_kind, target_id, summary, created_at, updated_at FROM synthbio.journey_stages WHERE tenant_id = $1 AND journey_id = $2 ORDER BY stage_index ASC",
        )
        .bind(tenant_id.as_uuid())
        .bind(journey_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio journey stages: {e}")))?;
        Ok(rows)
    }

    /// Set the route choice and mark the route stage done — a guarded
    /// UPDATE so a concurrent route change loses.
    pub async fn set_route(
        &self,
        tenant_id: TenantId,
        journey_id: Uuid,
        route: &str,
        actor: &str,
    ) -> HelixResult<Journey> {
        if !["extraction", "engineered_microbe"].contains(&route) {
            return Err(HelixError::validation(
                "route must be extraction | engineered_microbe",
            ));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio route tx: {e}")))?;
        let updated: Option<(Uuid,)> = sqlx::query_as(
            "UPDATE synthbio.journeys SET route_choice = $1, updated_at = $2 WHERE tenant_id = $3 AND id = $4 AND status = 'active' AND route_choice = 'undecided' RETURNING id",
        )
        .bind(route)
        .bind(Utc::now())
        .bind(tenant_id.as_uuid())
        .bind(journey_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio set route: {e}")))?;
        if updated.is_none() {
            return Err(HelixError::conflict(
                "route already chosen or journey not active",
            ));
        }
        self.mark_stage_done(&mut tx, tenant_id, journey_id, STAGE_ROUTE, None, None)
            .await?;
        self.record_event(
            &mut tx,
            tenant_id,
            "journey",
            journey_id,
            "route_chosen",
            actor,
            serde_json::json!({"route": route}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio route commit: {e}")))?;
        self.refresh_journey(tenant_id, journey_id).await?;
        self.get_journey(tenant_id, journey_id)
            .await?
            .ok_or_else(|| HelixError::internal("journey vanished"))
    }

    async fn mark_stage_done(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: TenantId,
        journey_id: Uuid,
        stage_index: usize,
        target_kind: Option<&str>,
        target_id: Option<Uuid>,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"
            UPDATE synthbio.journey_stages
            SET status = 'done', target_kind = COALESCE($1, target_kind),
                target_id = COALESCE($2, target_id), updated_at = $3
            WHERE tenant_id = $4 AND journey_id = $5 AND stage_index = $6 AND status != 'done'
            "#,
        )
        .bind(target_kind)
        .bind(target_id)
        .bind(Utc::now())
        .bind(tenant_id.as_uuid())
        .bind(journey_id)
        .bind(stage_index as i32)
        .execute(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio stage done: {e}")))?;
        Ok(())
    }

    /// Link a stage's artifact. Source and build take samples; design takes
    /// a design. Build samples must be produced from the journey's design —
    /// a sample from anywhere else is refused with the reason.
    pub async fn link_stage_target(
        &self,
        tenant_id: TenantId,
        journey_id: Uuid,
        stage_index: usize,
        target_kind: &str,
        target_id: Uuid,
        actor: &str,
    ) -> HelixResult<JourneyDetail> {
        let stages = self.journey_stages(tenant_id, journey_id).await?;
        let stage = stages
            .get(stage_index)
            .ok_or_else(|| HelixError::not_found("stage not found"))?;
        if stage.status == "done" {
            return Err(HelixError::conflict("stage already complete"));
        }
        let journey = self
            .get_journey(tenant_id, journey_id)
            .await?
            .ok_or_else(|| HelixError::not_found("journey not found"))?;
        if journey.status != "active" {
            return Err(HelixError::validation("journey is not active"));
        }

        match (stage_index, target_kind) {
            (STAGE_SOURCE, "sample") => {
                self.require_sample(tenant_id, target_id).await?;
            }
            (STAGE_DESIGN, "design") => {
                let design = self
                    .get_design(tenant_id, target_id)
                    .await?
                    .ok_or_else(|| HelixError::not_found("design not found"))?;
                if design.deleted_at.is_some() {
                    return Err(HelixError::not_found("design not found"));
                }
            }
            (STAGE_BUILD, "sample") => {
                let sample = self.require_sample(tenant_id, target_id).await?;
                let design_target = stages
                    .get(STAGE_DESIGN)
                    .and_then(|s| s.target_id)
                    .ok_or_else(|| HelixError::validation("link the design first"))?;
                if sample.design_id != Some(design_target) {
                    return Err(HelixError::validation(
                        "this sample is not built from the journey's design",
                    ));
                }
            }
            (STAGE_ROUTE | STAGE_RISK | STAGE_TEST | STAGE_EVIDENCE, _) => {
                return Err(HelixError::validation(
                    "this stage completes automatically when its check passes",
                ));
            }
            _ => {
                return Err(HelixError::validation(format!(
                    "stage {stage_index} does not take a `{target_kind}` artifact",
                )));
            }
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio link tx: {e}")))?;
        self.mark_stage_done(
            &mut tx,
            tenant_id,
            journey_id,
            stage_index,
            Some(target_kind),
            Some(target_id),
        )
        .await?;
        self.add_edge(
            &mut tx,
            tenant_id,
            "journey",
            journey_id,
            target_kind,
            target_id,
            "uses",
        )
        .await?;
        self.record_event(
            &mut tx,
            tenant_id,
            "journey",
            journey_id,
            "stage_linked",
            actor,
            serde_json::json!({"stage": stage_index, "target_kind": target_kind}),
        )
        .await?;
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio link commit: {e}")))?;
        self.refresh_journey(tenant_id, journey_id).await?;
        self.journey_detail(tenant_id, journey_id)
            .await?
            .ok_or_else(|| HelixError::internal("journey vanished"))
    }

    async fn require_sample(&self, tenant_id: TenantId, sample_id: Uuid) -> HelixResult<Sample> {
        let sample = self
            .get_sample(tenant_id, sample_id)
            .await?
            .ok_or_else(|| HelixError::not_found("sample not found"))?;
        if sample.deleted_at.is_some() {
            return Err(HelixError::not_found("sample not found"));
        }
        Ok(sample)
    }

    /// Recompute the journey: auto-checks (risk, test, evidence) mark done
    /// when their conditions pass; the current pointer moves to the first
    /// incomplete stage; a fully-done journey completes.
    pub async fn refresh_journey(&self, tenant_id: TenantId, journey_id: Uuid) -> HelixResult<()> {
        let journey = self
            .get_journey(tenant_id, journey_id)
            .await?
            .ok_or_else(|| HelixError::not_found("journey not found"))?;
        if journey.status != "active" {
            return Ok(());
        }
        let stages = self.journey_stages(tenant_id, journey_id).await?;
        let checks = self.evaluate_stages(tenant_id, &journey, &stages).await?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio refresh tx: {e}")))?;
        for (i, check) in checks.iter().enumerate() {
            if check.met && stages.get(i).is_some_and(|s| s.status != "done") {
                self.mark_stage_done(&mut tx, tenant_id, journey_id, i, None, None)
                    .await?;
            }
        }
        let done_count = checks.iter().filter(|c| c.met).count();
        let total = checks.len();
        let current = checks.iter().position(|c| !c.met).unwrap_or(total) as i32;
        let new_status = if done_count == total {
            "completed"
        } else {
            "active"
        };
        sqlx::query(
            "UPDATE synthbio.journeys SET current_stage = $1, status = $2, updated_at = $3 WHERE tenant_id = $4 AND id = $5",
        )
        .bind(current)
        .bind(new_status)
        .bind(Utc::now())
        .bind(tenant_id.as_uuid())
        .bind(journey_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio refresh update: {e}")))?;
        if new_status == "completed" {
            self.record_event(
                &mut tx,
                tenant_id,
                "journey",
                journey_id,
                "completed",
                "system",
                serde_json::json!({}),
            )
            .await?;
        }
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio refresh commit: {e}")))?;
        Ok(())
    }

    /// The teacher: per stage, whether its check passes right now and, if
    /// not, exactly what's missing.
    async fn evaluate_stages(
        &self,
        tenant_id: TenantId,
        journey: &Journey,
        stages: &[JourneyStage],
    ) -> HelixResult<Vec<StageCheck>> {
        let mut out = Vec::with_capacity(stages.len());
        for (i, stage) in stages.iter().enumerate() {
            if stage.status == "done" {
                out.push(StageCheck {
                    met: true,
                    missing: String::new(),
                });
                continue;
            }
            let check = match i {
                STAGE_SOURCE => StageCheck {
                    met: false,
                    missing: "register and link the source sample".into(),
                },
                STAGE_ROUTE => {
                    if journey.route_choice != "undecided" {
                        StageCheck {
                            met: true,
                            missing: String::new(),
                        }
                    } else {
                        StageCheck {
                            met: false,
                            missing: "choose extraction or an engineered microbe".into(),
                        }
                    }
                }
                STAGE_DESIGN => StageCheck {
                    met: false,
                    missing: "create or import and link the design".into(),
                },
                STAGE_RISK => {
                    let design_id = stages.get(STAGE_DESIGN).and_then(|s| s.target_id);
                    match design_id {
                        None => StageCheck {
                            met: false,
                            missing: "link the design first".into(),
                        },
                        Some(did) => {
                            let case = self.get_risk_case(tenant_id, did).await?;
                            let effective = effective_risk(case.as_ref());
                            if effective == "allowed" || effective == "restricted" {
                                StageCheck {
                                    met: true,
                                    missing: String::new(),
                                }
                            } else if effective == "blocked" {
                                StageCheck {
                                    met: false,
                                    missing: "risk review decided blocked — a named authority must revisit".into(),
                                }
                            } else {
                                StageCheck {
                                    met: false,
                                    missing:
                                        "risk review pending — a named human authority must decide"
                                            .into(),
                                }
                            }
                        }
                    }
                }
                STAGE_BUILD => StageCheck {
                    met: false,
                    missing: "register and link the build sample derived from the design".into(),
                },
                STAGE_TEST => {
                    let sample_id = stages.get(STAGE_BUILD).and_then(|s| s.target_id);
                    match sample_id {
                        None => StageCheck {
                            met: false,
                            missing: "link the build sample first".into(),
                        },
                        Some(sid) => {
                            let accepted: Option<(Uuid,)> = sqlx::query_as(
                                "SELECT id FROM synthbio.measurements WHERE tenant_id = $1 AND sample_id = $2 AND status = 'accepted' AND deleted_at IS NULL LIMIT 1",
                            )
                            .bind(tenant_id.as_uuid())
                            .bind(sid)
                            .fetch_optional(&self.pool)
                            .await
                            .map_err(|e| HelixError::dependency(format!("synthbio test check: {e}")))?;
                            if accepted.is_some() {
                                StageCheck {
                                    met: true,
                                    missing: String::new(),
                                }
                            } else {
                                StageCheck {
                                    met: false,
                                    missing: "no accepted measurement on the build sample yet"
                                        .into(),
                                }
                            }
                        }
                    }
                }
                STAGE_EVIDENCE => {
                    let design_id = stages.get(STAGE_DESIGN).and_then(|s| s.target_id);
                    match design_id {
                        None => StageCheck {
                            met: false,
                            missing: "link the design first".into(),
                        },
                        Some(did) => {
                            let accepted: Option<(Uuid,)> = sqlx::query_as(
                                "SELECT id FROM synthbio.claims WHERE tenant_id = $1 AND design_id = $2 AND status = 'accepted' AND deleted_at IS NULL LIMIT 1",
                            )
                            .bind(tenant_id.as_uuid())
                            .bind(did)
                            .fetch_optional(&self.pool)
                            .await
                            .map_err(|e| HelixError::dependency(format!("synthbio evidence check: {e}")))?;
                            if accepted.is_some() {
                                StageCheck {
                                    met: true,
                                    missing: String::new(),
                                }
                            } else {
                                StageCheck {
                                    met: false,
                                    missing: "no attested claim on the design yet".into(),
                                }
                            }
                        }
                    }
                }
                _ => StageCheck {
                    met: false,
                    missing: "unknown stage".into(),
                },
            };
            out.push(check);
        }
        Ok(out)
    }

    pub async fn journey_detail(
        &self,
        tenant_id: TenantId,
        journey_id: Uuid,
    ) -> HelixResult<Option<JourneyDetail>> {
        // Reading a journey re-evaluates the auto stages (risk/test/evidence)
        // so the detail is always current — refresh is idempotent.
        if self.get_journey(tenant_id, journey_id).await?.is_none() {
            return Ok(None);
        }
        self.refresh_journey(tenant_id, journey_id).await?;
        let journey = self
            .get_journey(tenant_id, journey_id)
            .await?
            .ok_or_else(|| HelixError::not_found("journey not found"))?;
        let stages = self.journey_stages(tenant_id, journey_id).await?;
        let checks = self.evaluate_stages(tenant_id, &journey, &stages).await?;
        let stages = stages
            .into_iter()
            .zip(checks)
            .map(|(stage, check)| StageWithCheck { stage, check })
            .collect();
        Ok(Some(JourneyDetail { journey, stages }))
    }

    /// The hello-world journey: a complete, real, checkable example built
    /// end-to-end with demo provenance — one click, every guard firing.
    pub async fn demo_journey(
        &self,
        tenant_id: TenantId,
        actor: &str,
    ) -> HelixResult<JourneyDetail> {
        let journey = self
            .create_journey(
                tenant_id,
                "Demo: lavender balm for dry skin",
                "Make a soothing topical balm from lavender — the full guided path",
                "plant-to-topical",
                actor,
            )
            .await?;

        // 1. Source.
        let source = self
            .register_sample(
                tenant_id,
                "Lavender flowers (demo)",
                "other",
                None,
                "greenhouse/dry-rack",
                actor,
            )
            .await?;
        self.link_stage_target(
            tenant_id,
            journey.id,
            STAGE_SOURCE,
            "sample",
            source.id,
            actor,
        )
        .await?;

        // 2. Route.
        self.set_route(tenant_id, journey.id, "extraction", actor)
            .await?;

        // 3. Design.
        let input = VersionInput {
            alphabet: "dna".into(),
            topology: "circular".into(),
            source_kind: "manual".into(),
            source_name: "demo journey".into(),
            sequence_text: "ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT".into(),
            components: vec![
                Component {
                    name: "T7 promoter".into(),
                    role_so: "SO:0000167".into(),
                    start: 5,
                    end: 24,
                    strand: 1,
                    source: "demo journey".into(),
                },
                Component {
                    name: "lav-enzyme CDS".into(),
                    role_so: "SO:0000316".into(),
                    start: 26,
                    end: 40,
                    strand: 1,
                    source: "demo journey".into(),
                },
            ],
            provenance: "demo journey".into(),
            notes: "demo construct for the guided journey".into(),
        };
        let design = self
            .create_design(
                tenant_id,
                "pLAV-BALM-001 (demo)",
                "lavender balm expression construct (demo)",
                "internal",
                &input,
                actor,
            )
            .await?;
        self.link_stage_target(
            tenant_id,
            journey.id,
            STAGE_DESIGN,
            "design",
            design.id,
            actor,
        )
        .await?;

        // 4. Risk.
        self.review_risk(
            tenant_id,
            design.id,
            &ReviewDecision {
                state: "allowed".into(),
                intended_use: "topical balm research (demo)".into(),
                policy_version: "biosafety-v1".into(),
                reasons: vec![
                    "public backbone (demo)".into(),
                    "no sequences of concern (demo)".into(),
                ],
                conditions: String::new(),
                expires_at: None,
                expected_state: Some("unknown".into()),
            },
            "Demo Biosafety Officer",
        )
        .await?;

        // 5. Build.
        let build = self
            .register_sample(
                tenant_id,
                "Lavender balm prep A (demo)",
                "plasmid_prep",
                Some(design.id),
                "bench-1",
                actor,
            )
            .await?;
        self.link_stage_target(
            tenant_id,
            journey.id,
            STAGE_BUILD,
            "sample",
            build.id,
            actor,
        )
        .await?;

        // 6. Test.
        let m = self
            .record_measurement(
                tenant_id,
                &MeasurementInput {
                    sample_id: build.id,
                    design_version_id: None,
                    kind: "absorbance".into(),
                    method: "plate reader (demo)".into(),
                    value: Some(0.87),
                    unit: "AU".into(),
                    uncertainty: Some(0.02),
                    raw: serde_json::json!({"plate": "demo-A1"}),
                },
                actor,
            )
            .await?;
        self.transition_measurement(tenant_id, m.id, "accept", actor)
            .await?;

        // 7. Evidence.
        let claim = self
            .create_claim(
                tenant_id,
                design.id,
                "Lavender balm prep A tolerates 37C incubation (demo)",
                actor,
            )
            .await?;
        self.link_evidence(
            tenant_id,
            claim.id,
            "measurement",
            m.id,
            "supports",
            "demo plate shows stable signal",
            actor,
        )
        .await?;
        self.attest_claim(tenant_id, claim.id, "Demo Principal Investigator")
            .await?;

        self.refresh_journey(tenant_id, journey.id).await?;
        self.journey_detail(tenant_id, journey.id)
            .await?
            .ok_or_else(|| HelixError::internal("demo journey vanished"))
    }
}
