//! Usage metering client. Products emit meter events; billing-service aggregates.
//! Plan catalog is static product pricing; tenant plan assignment is optional depth.
//! Money paths use i64 cents / milli-cents (Kimi P1); meter quantities stay f64 for SQL compat.

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::time::UtcTimestamp;
use shared_core::{HelixError, HelixResult};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterEvent {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub product: String,
    pub metric: String,
    pub quantity: f64,
    pub unit: String,
    pub dimensions: serde_json::Value,
    pub occurred_at: UtcTimestamp,
    /// Optional idempotency key — duplicate inserts are no-ops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub tenant_id: TenantId,
    pub product: String,
    pub metric: String,
    pub total: f64,
    pub unit: String,
}

/// Payment provider id used on intents (`local_sim` default; `stripe` when configured).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentProviderKind {
    LocalSim,
    Stripe,
}

impl PaymentProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LocalSim => "local_sim",
            Self::Stripe => "stripe",
        }
    }

    pub fn from_env() -> Self {
        Self::parse(&std::env::var("HELIX_PAYMENT_PROVIDER").unwrap_or_else(|_| "local_sim".into()))
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "stripe" => Self::Stripe,
            _ => Self::LocalSim,
        }
    }
}

/// Commercial plan catalog entry. Money fields are integer cents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub price_cents_month: i64,
    /// Included usage units (display units; not money).
    pub included_meter_units: f64,
    /// Overage price in milli-cents per unit (0.05 cents = 50 milli-cents).
    pub overage_milli_cents_per_unit: i64,
    pub description: String,
}

impl Plan {
    /// Cents per unit as f64 for display.
    pub fn overage_cents_per_unit(&self) -> f64 {
        self.overage_milli_cents_per_unit as f64 / 1000.0
    }
}

/// Static plan catalog for HelixForge local/dev and self-host defaults.
pub fn plan_catalog() -> Vec<Plan> {
    vec![
        Plan {
            id: "free".into(),
            name: "Free".into(),
            price_cents_month: 0,
            included_meter_units: 1_000.0,
            overage_milli_cents_per_unit: 0,
            description: "Local/dev and evaluation".into(),
        },
        Plan {
            id: "team".into(),
            name: "Team".into(),
            price_cents_month: 4_900,
            included_meter_units: 50_000.0,
            overage_milli_cents_per_unit: 50, // 0.05 cents/unit
            description: "Small orgs - included usage + overage".into(),
        },
        Plan {
            id: "enterprise".into(),
            name: "Enterprise".into(),
            price_cents_month: 49_900,
            included_meter_units: 1_000_000.0,
            overage_milli_cents_per_unit: 10, // 0.01 cents/unit
            description: "Sovereign deploy - high included usage".into(),
        },
    ]
}

pub fn plan_by_id(id: &str) -> Option<Plan> {
    plan_catalog().into_iter().find(|p| p.id == id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantPlan {
    pub tenant_id: TenantId,
    pub plan_id: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub product: String,
    pub metric: String,
    pub quantity: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBillingSummary {
    pub tenant_id: TenantId,
    pub plan: Plan,
    pub usage_units: f64,
    pub included_units: f64,
    pub overage_units: f64,
    pub base_cents: i64,
    /// Integer cents (Kimi P1 — no f64 money).
    pub overage_cents: i64,
    pub estimated_total_cents: i64,
    pub lines: Vec<InvoiceLine>,
}

#[async_trait]
pub trait Metering: Send + Sync {
    async fn record(&self, event: MeterEvent) -> HelixResult<()>;
    async fn summarize(&self, tenant_id: TenantId, product: &str)
        -> HelixResult<Vec<UsageSummary>>;
}

#[async_trait]
pub trait PlanStore: Send + Sync {
    async fn get_plan(&self, tenant_id: TenantId) -> HelixResult<TenantPlan>;
    async fn set_plan(&self, tenant_id: TenantId, plan_id: &str) -> HelixResult<TenantPlan>;
}

#[derive(Default)]
struct MemoryPlanStore {
    plans: RwLock<HashMap<String, String>>,
}

#[async_trait]
impl PlanStore for MemoryPlanStore {
    async fn get_plan(&self, tenant_id: TenantId) -> HelixResult<TenantPlan> {
        let plan_id = self
            .plans
            .read()
            .get(&tenant_id.to_string())
            .cloned()
            .unwrap_or_else(|| "free".into());
        Ok(TenantPlan {
            tenant_id,
            plan_id,
            updated_at: "memory".into(),
        })
    }

    async fn set_plan(&self, tenant_id: TenantId, plan_id: &str) -> HelixResult<TenantPlan> {
        if plan_by_id(plan_id).is_none() {
            return Err(HelixError::validation(format!("unknown plan {plan_id}")));
        }
        self.plans
            .write()
            .insert(tenant_id.to_string(), plan_id.into());
        Ok(TenantPlan {
            tenant_id,
            plan_id: plan_id.into(),
            updated_at: "memory".into(),
        })
    }
}

#[derive(Clone)]
pub struct BillingClient {
    inner: Arc<dyn Metering>,
    plans: Arc<dyn PlanStore>,
}

impl BillingClient {
    pub fn memory() -> Self {
        Self {
            inner: Arc::new(MemoryMetering::default()),
            plans: Arc::new(MemoryPlanStore::default()),
        }
    }

    pub fn new(inner: Arc<dyn Metering>) -> Self {
        Self {
            inner,
            plans: Arc::new(MemoryPlanStore::default()),
        }
    }

    pub fn with_plan_store(inner: Arc<dyn Metering>, plans: Arc<dyn PlanStore>) -> Self {
        Self { inner, plans }
    }

    pub async fn record_usage(
        &self,
        tenant_id: TenantId,
        product: &str,
        metric: &str,
        quantity: f64,
        unit: &str,
        dimensions: serde_json::Value,
    ) -> HelixResult<()> {
        self.record_usage_idempotent(tenant_id, product, metric, quantity, unit, dimensions, None)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn record_usage_idempotent(
        &self,
        tenant_id: TenantId,
        product: &str,
        metric: &str,
        quantity: f64,
        unit: &str,
        dimensions: serde_json::Value,
        idempotency_key: Option<String>,
    ) -> HelixResult<()> {
        self.inner
            .record(MeterEvent {
                id: Uuid::now_v7(),
                tenant_id,
                product: product.into(),
                metric: metric.into(),
                quantity,
                unit: unit.into(),
                dimensions,
                occurred_at: UtcTimestamp::now(),
                idempotency_key,
            })
            .await
    }

    pub async fn summarize(
        &self,
        tenant_id: TenantId,
        product: &str,
    ) -> HelixResult<Vec<UsageSummary>> {
        self.inner.summarize(tenant_id, product).await
    }

    pub fn plans(&self) -> Vec<Plan> {
        plan_catalog()
    }

    pub async fn get_tenant_plan(&self, tenant_id: TenantId) -> HelixResult<TenantPlan> {
        self.plans.get_plan(tenant_id).await
    }

    pub async fn set_tenant_plan(
        &self,
        tenant_id: TenantId,
        plan_id: &str,
    ) -> HelixResult<TenantPlan> {
        self.plans.set_plan(tenant_id, plan_id).await
    }

    /// Roll up usage across product slugs into an estimated invoice against the tenant plan.
    pub async fn tenant_billing_summary(
        &self,
        tenant_id: TenantId,
        products: &[String],
    ) -> HelixResult<TenantBillingSummary> {
        let tp = self.get_tenant_plan(tenant_id).await?;
        let plan = plan_by_id(&tp.plan_id).unwrap_or_else(|| plan_catalog()[0].clone());
        let mut lines = Vec::new();
        let mut usage_units = 0.0;
        for product in products {
            for row in self.summarize(tenant_id, product).await? {
                usage_units += row.total;
                lines.push(InvoiceLine {
                    product: row.product,
                    metric: row.metric,
                    quantity: row.total,
                    unit: row.unit,
                });
            }
        }
        let overage_units = (usage_units - plan.included_meter_units).max(0.0);
        // Integer milli-cents: floor(overage_units * milli_cents_per_unit)
        let overage_milli =
            (overage_units * plan.overage_milli_cents_per_unit as f64).round() as i64;
        let overage_cents = overage_milli / 1000;
        Ok(TenantBillingSummary {
            tenant_id,
            base_cents: plan.price_cents_month,
            included_units: plan.included_meter_units,
            overage_units,
            overage_cents,
            estimated_total_cents: plan.price_cents_month.saturating_add(overage_cents),
            usage_units,
            plan,
            lines,
        })
    }
}

#[derive(Default)]
struct MemoryMetering {
    events: RwLock<Vec<MeterEvent>>,
}

#[async_trait]
impl Metering for MemoryMetering {
    async fn record(&self, event: MeterEvent) -> HelixResult<()> {
        if event.quantity < 0.0 {
            return Err(HelixError::validation("quantity must be >= 0"));
        }
        let mut guard = self.events.write();
        if let Some(key) = event.idempotency_key.as_deref().filter(|k| !k.is_empty()) {
            let dup = guard.iter().any(|e| {
                e.tenant_id == event.tenant_id
                    && e.product == event.product
                    && e.idempotency_key.as_deref() == Some(key)
            });
            if dup {
                return Ok(());
            }
        }
        guard.push(event);
        Ok(())
    }

    async fn summarize(
        &self,
        tenant_id: TenantId,
        product: &str,
    ) -> HelixResult<Vec<UsageSummary>> {
        let guard = self.events.read();
        let mut map: HashMap<String, (f64, String)> = HashMap::new();
        for e in guard
            .iter()
            .filter(|e| e.tenant_id == tenant_id && e.product == product)
        {
            let entry = map.entry(e.metric.clone()).or_insert((0.0, e.unit.clone()));
            entry.0 += e.quantity;
        }
        Ok(map
            .into_iter()
            .map(|(metric, (total, unit))| UsageSummary {
                tenant_id,
                product: product.into(),
                metric,
                total,
                unit,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn meters_and_summarizes() {
        let client = BillingClient::memory();
        let tid = TenantId::new();
        client
            .record_usage(
                tid,
                "helix-collab",
                "api.calls",
                3.0,
                "count",
                serde_json::json!({}),
            )
            .await
            .unwrap();
        client
            .record_usage(
                tid,
                "helix-collab",
                "api.calls",
                2.0,
                "count",
                serde_json::json!({}),
            )
            .await
            .unwrap();
        let summary = client.summarize(tid, "helix-collab").await.unwrap();
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].total, 5.0);
    }

    #[tokio::test]
    async fn plan_and_invoice_summary() {
        let client = BillingClient::memory();
        let tid = TenantId::new();
        client.set_tenant_plan(tid, "team").await.unwrap();
        client
            .record_usage(
                tid,
                "helix-core",
                "agents.runs",
                10.0,
                "count",
                serde_json::json!({}),
            )
            .await
            .unwrap();
        let inv = client
            .tenant_billing_summary(tid, &["helix-core".into()])
            .await
            .unwrap();
        assert_eq!(inv.plan.id, "team");
        assert_eq!(inv.usage_units, 10.0);
        assert_eq!(inv.base_cents, 4_900);
        assert!(!inv.lines.is_empty());
        // i64 money path
        let _: i64 = inv.estimated_total_cents;
        let _: i64 = inv.overage_cents;
    }

    #[tokio::test]
    async fn idempotency_dedupes() {
        let client = BillingClient::memory();
        let tid = TenantId::new();
        for _ in 0..3 {
            client
                .record_usage_idempotent(
                    tid,
                    "helix-core",
                    "api.calls",
                    1.0,
                    "count",
                    serde_json::json!({}),
                    Some("idem-1".into()),
                )
                .await
                .unwrap();
        }
        let s = client.summarize(tid, "helix-core").await.unwrap();
        assert_eq!(s[0].total, 1.0);
    }
}
