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
                      created_at, updated_at
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
            "SELECT id, tenant_id, design_id, design_version_id, state, intended_use, policy_version, reasons, conditions, reviewer, decided_at, expires_at, created_at, updated_at FROM synthbio.risk_cases WHERE tenant_id = $1 AND design_id = $2 ORDER BY created_at DESC LIMIT 1",
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
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
            accession: String,
        }
        let rows: Vec<QueueRow> = sqlx::query_as(
            r#"
            SELECT c.id, c.tenant_id, c.design_id, c.design_version_id, c.state, c.intended_use,
                   c.policy_version, c.reasons, c.conditions, c.reviewer, c.decided_at,
                   c.expires_at, c.created_at, c.updated_at, d.accession
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
            WHERE tenant_id = $10 AND id = $11 AND state = $12
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
