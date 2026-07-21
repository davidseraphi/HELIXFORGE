//! HelixForge database layer — pool, migrations, durable product repositories.

pub mod acl;
pub mod agents;
pub mod api_keys;
pub mod atomic;
pub mod audit_archive;
pub mod audit_pg;
pub mod capital;
pub mod climate;
pub mod code;
pub mod code_endstate;
pub mod code_residuals;
pub mod collab;
pub mod collab_sovereign;
pub mod commerce;
pub mod cura;
pub mod edu;
pub mod flow;
pub mod governance;
pub mod grid;
pub mod insights;
pub mod jobs;
pub mod lex;
pub mod membership;
pub mod meter_pg;
pub mod network;
pub mod nova;
pub mod orbit;
pub mod outbox;
pub mod payments;
pub mod plans_pg;
pub mod pool;
pub mod pulse;
pub mod quantum;
pub mod regions;
pub mod studio;
pub mod tenants;
pub mod terra;
pub mod vault_objects;
pub mod vault_pg;
pub mod vita;
pub mod well;
pub mod workspace;

pub use acl::{AclEntry, AclPermission, ResourceAclRepo};
pub use agents::AgentRunStore;
pub use api_keys::{ApiKeyRecord, ApiKeyStore, IssuedApiKey};
pub use audit_archive::ObjectStoreArchiveSink;
pub use audit_pg::{PgAuditSink, TransactionalAuditSink};
pub use capital::{Account, CapitalRepo, Journal, JournalLine, JournalLineInput, TrialBalanceRow};
pub use climate::{
    next_scenario_status, next_score_status, ClimateRepo, ClimateSummaryRow, RiskScore, Scenario,
    ScenarioUpdate, ScoreUpdate,
};
pub use code::{
    CodeAgentJob, CodePipeline, CodePipelineArtifact, CodePipelineRun, CodeRef, CodeRepo,
    CodeRepoStore, CodeWorkspace, CryptoGroup, SealedObjectMeta,
};
pub use code_endstate::{
    CodeAgentJobEvent, CodeBranchProtection, CodeIssue, CodeMlsDevice, CodePrReview,
    CodePullRequest, CodeRunner, CodeTenantQuota, CodeWebhook, CodeWebhookDelivery,
};
pub use code_residuals::{
    hash_deploy_token, CodeDebugSession, CodeDeployKey, CodeDeployKeyIssued, CodeLspSessionReg,
    CodeProcessSession, CodeTenantBreakglass,
};
pub use collab::{
    parse_mentions, CollabDocument, CollabFolder, CollabRepo, DocActivity, DocumentComment,
    DocumentPatch, DocumentRevision, Mention, PresencePeer,
};
pub use collab_sovereign::{
    forbids_cleartext, requires_client_e2ee, validate_classification, AttachmentMeta, CollabSpace,
    DeviceKey, FederationReceipt, KeyShare, RecoveryCeremony, ResidencyProof, SovereignCollabRepo,
};
pub use commerce::{CommerceRepo, Order, OrderItem, OrderLineInput, Product};
pub use cura::{
    next_case_status, next_note_status, CareCase, CareNote, CaseUpdate, CuraRepo, CuraSummaryRow,
    NoteUpdate,
};
pub use edu::{Course, EduRepo, Enrollment};
pub use flow::{FlowRepo, Workflow, WorkflowRun};
pub use governance::{DeleteDecision, GovernanceRepo, LegalHold, PurposeBinding, RetentionPolicy};
pub use grid::{
    next_reading_status, next_site_status, GridRepo, GridSite, GridSummaryRow, Reading,
    ReadingUpdate, SiteUpdate,
};
pub use insights::{AggregateResult, Dataset, InsightsRepo, MetricDef, MetricPoint};
pub use jobs::{Job, JobCheckpoint, JobRepo, JobStatus};
pub use lex::{
    next_filing_status, next_matter_status, Filing, FilingUpdate, LexRepo, LexSummaryRow, Matter,
    MatterUpdate,
};
pub use membership::MembershipRepo;
pub use meter_pg::PgMetering;
pub use network::{
    can_revive_connection, next_opportunity_status, next_profile_status, Connection, NetworkRepo,
    NetworkSummaryRow, Opportunity, OpportunityUpdate, Profile, ProfileUpdate,
};
pub use nova::{
    next_experiment_status, next_finding_status, Experiment, ExperimentUpdate, Finding,
    FindingUpdate, NovaRepo, NovaSummaryRow,
};
pub use orbit::{
    next_asset_status, next_pass_status, AssetUpdate, OrbitRepo, OrbitSummaryRow, Pass, PassUpdate,
    SpaceAsset,
};
pub use outbox::{OutboxItem, OutboxRepo};
pub use payments::{PaymentIntent, PaymentStatus, PaymentStore};
pub use plans_pg::PgPlanStore;
pub use pool::{
    connect_and_migrate, connect_and_migrate_with_config, try_connect_and_migrate,
    try_connect_and_migrate_with_config, DbPool, DbStatus,
};
pub use pulse::{
    next_incident_status, next_monitor_status, Incident, IncidentUpdate, Monitor, MonitorUpdate,
    PulseRepo, PulseSummaryRow,
};
pub use quantum::{
    next_circuit_status, next_job_status, Circuit, CircuitUpdate, JobUpdate, QuantumJob,
    QuantumRepo, QuantumSummaryRow,
};
pub use regions::{RegionRecord, RegionRepo};
pub use studio::{
    next_app_status, next_page_status, App, AppUpdate, Page, PageUpdate, StudioRepo,
    StudioSummaryRow,
};
pub use tenants::{TenantRecord, TenantRepo, TenantStatus};
pub use terra::{
    next_field_status, next_observation_status, Field, FieldUpdate, Observation, ObservationUpdate,
    TerraRepo, TerraSummaryRow,
};
pub use vault_objects::{VaultObjectRef, VaultObjectStore};
pub use vault_pg::PgVault;
// re-export path used by service_kit
pub use atomic::AtomicWork;
pub use vita::{
    next_cohort_status, next_study_status, Cohort, CohortUpdate, Study, StudyUpdate, VitaRepo,
    VitaSummaryRow,
};
pub use well::{
    next_habit_status, validate_optional_scale, CheckIn, CheckInEdit, CheckInUpdate, Habit,
    HabitLog, HabitSummaryRow, HabitUpdate, WellRepo,
};
pub use workspace::{WorkspaceRecord, WorkspaceRepo};

/// Pin the Postgres session variable `app.current_tenant` for RLS policies.
/// Call this at the start of every transaction that touches a tenant-scoped
/// table protected by `helix_core.set_tenant_context()`.
pub async fn set_tenant_context(
    conn: &mut sqlx::PgConnection,
    tenant_id: shared_core::ids::TenantId,
) -> shared_core::HelixResult<()> {
    sqlx::query("SELECT helix_core.set_tenant_context($1)")
        .bind(tenant_id.as_uuid())
        .execute(conn)
        .await
        .map_err(|e| shared_core::HelixError::dependency(format!("set tenant context: {e}")))?;
    Ok(())
}
