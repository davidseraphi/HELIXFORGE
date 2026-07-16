//! Agent framework — tools, runs, and orchestrators for HelixForge products.
//!
//! Every product reuses this crate for agentic capabilities. Product-specific
//! tools register into a shared [`ToolRegistry`]; the [`AgentRuntime`] executes
//! multi-step plans with audit-friendly run records.

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use shared_core::ids::{TenantId, UserId};
use shared_core::time::UtcTimestamp;
use shared_core::{HelixError, HelixResult};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub tools: Vec<String>,
    pub max_steps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: Uuid,
    pub agent: String,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub input: serde_json::Value,
    pub status: RunStatus,
    pub steps: Vec<AgentStep>,
    pub output: Option<serde_json::Value>,
    pub started_at: UtcTimestamp,
    pub finished_at: Option<UtcTimestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub index: u32,
    pub kind: StepKind,
    pub name: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Thought,
    ToolCall,
    Final,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub name: String,
    pub args: serde_json::Value,
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub ok: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn invoke(&self, req: ToolRequest) -> HelixResult<ToolResponse>;
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, tool: Arc<dyn Tool>) {
        self.tools.write().insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.read().get(name).cloned()
    }

    pub fn list(&self) -> Vec<(String, String)> {
        self.tools
            .read()
            .values()
            .map(|t| (t.name().to_string(), t.description().to_string()))
            .collect()
    }
}

/// Echo tool — always available for smoke tests.
pub struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echo arguments back as the tool result"
    }

    async fn invoke(&self, req: ToolRequest) -> HelixResult<ToolResponse> {
        Ok(ToolResponse {
            ok: true,
            data: req.args,
            error: None,
        })
    }
}

/// Catalog tool — lists HelixForge products (uses shared_core catalog).
pub struct CatalogTool;

#[async_trait]
impl Tool for CatalogTool {
    fn name(&self) -> &str {
        "product_catalog"
    }

    fn description(&self) -> &str {
        "List HelixForge product catalog entries"
    }

    async fn invoke(&self, _req: ToolRequest) -> HelixResult<ToolResponse> {
        let items: Vec<_> = shared_core::PRODUCT_CATALOG
            .iter()
            .map(|p| {
                serde_json::json!({
                    "order": p.order,
                    "slug": p.slug,
                    "title": p.title,
                    "tier": p.tier,
                })
            })
            .collect();
        Ok(ToolResponse {
            ok: true,
            data: serde_json::json!({ "products": items }),
            error: None,
        })
    }
}

/// Clock tool — UTC now for agent planning / audit timestamps.
pub struct TimeTool;

#[async_trait]
impl Tool for TimeTool {
    fn name(&self) -> &str {
        "utc_now"
    }

    fn description(&self) -> &str {
        "Return current UTC timestamp"
    }

    async fn invoke(&self, _req: ToolRequest) -> HelixResult<ToolResponse> {
        let ts = UtcTimestamp::now();
        Ok(ToolResponse {
            ok: true,
            data: serde_json::json!({ "utc": ts }),
            error: None,
        })
    }
}

/// Tenant context tool — echoes resolved tenant/user for multi-tenant agents.
pub struct TenantContextTool;

#[async_trait]
impl Tool for TenantContextTool {
    fn name(&self) -> &str {
        "tenant_context"
    }

    fn description(&self) -> &str {
        "Return the agent run tenant_id and user_id"
    }

    async fn invoke(&self, req: ToolRequest) -> HelixResult<ToolResponse> {
        Ok(ToolResponse {
            ok: true,
            data: serde_json::json!({
                "tenant_id": req.tenant_id.to_string(),
                "user_id": req.user_id.to_string(),
            }),
            error: None,
        })
    }
}

#[derive(Clone)]
pub struct AgentRuntime {
    specs: Arc<RwLock<HashMap<String, AgentSpec>>>,
    tools: Arc<ToolRegistry>,
    runs: Arc<RwLock<HashMap<Uuid, AgentRun>>>,
}

impl AgentRuntime {
    pub fn new(tools: Arc<ToolRegistry>) -> Self {
        // Register defaults
        tools.register(Arc::new(EchoTool));
        tools.register(Arc::new(CatalogTool));
        tools.register(Arc::new(TimeTool));
        tools.register(Arc::new(TenantContextTool));
        Self {
            specs: Arc::new(RwLock::new(HashMap::new())),
            tools,
            runs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(Arc::new(ToolRegistry::new()))
    }

    pub fn register_agent(&self, spec: AgentSpec) {
        self.specs.write().insert(spec.name.clone(), spec);
    }

    pub fn tools(&self) -> Arc<ToolRegistry> {
        self.tools.clone()
    }

    pub async fn run(
        &self,
        agent_name: &str,
        tenant_id: TenantId,
        user_id: UserId,
        input: serde_json::Value,
    ) -> HelixResult<AgentRun> {
        let spec = self
            .specs
            .read()
            .get(agent_name)
            .cloned()
            .ok_or_else(|| HelixError::not_found(format!("agent {agent_name}")))?;

        let id = Uuid::now_v7();
        let mut run = AgentRun {
            id,
            agent: agent_name.into(),
            tenant_id,
            user_id,
            input: input.clone(),
            status: RunStatus::Running,
            steps: vec![],
            output: None,
            started_at: UtcTimestamp::now(),
            finished_at: None,
        };

        // Minimal planner: invoke requested tools in order, then finalize.
        let tool_names: Vec<String> = input
            .get("tools")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(|| spec.tools.clone());

        let args = input
            .get("args")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let mut step_idx = 0u32;
        let mut last_output = serde_json::json!({});

        // Per-tool timeout (env override); default 30s — Kimi agent runtime depth.
        let tool_timeout_secs: u64 = std::env::var("HELIX_AGENT_TOOL_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        let tool_timeout = std::time::Duration::from_secs(tool_timeout_secs);
        let cancel = input
            .get("cancel")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if cancel {
            run.status = RunStatus::Cancelled;
            run.output = Some(serde_json::json!({"reason": "cancel requested"}));
            run.finished_at = Some(UtcTimestamp::now());
            self.runs.write().insert(id, run.clone());
            return Ok(run);
        }

        for name in tool_names.iter().take(spec.max_steps as usize) {
            // Allow only tools listed on the agent spec (sandbox permission boundary).
            if !spec.tools.is_empty() && !spec.tools.iter().any(|t| t == name) {
                run.status = RunStatus::Failed;
                run.output = Some(serde_json::json!({
                    "error": format!("tool {name} not permitted for agent {}", spec.name)
                }));
                run.finished_at = Some(UtcTimestamp::now());
                self.runs.write().insert(id, run.clone());
                return Ok(run);
            }

            let tool = self
                .tools
                .get(name)
                .ok_or_else(|| HelixError::validation(format!("unknown tool {name}")))?;

            let started = std::time::Instant::now();
            let invoke_fut = tool.invoke(ToolRequest {
                name: name.clone(),
                args: args.clone(),
                tenant_id,
                user_id,
            });
            let resp = match tokio::time::timeout(tool_timeout, invoke_fut).await {
                Ok(Ok(r)) => r,
                Ok(Err(e)) => {
                    run.status = RunStatus::Failed;
                    run.output = Some(serde_json::json!({ "error": e.to_string() }));
                    run.finished_at = Some(UtcTimestamp::now());
                    self.runs.write().insert(id, run.clone());
                    return Ok(run);
                }
                Err(_) => {
                    run.status = RunStatus::Failed;
                    run.output = Some(serde_json::json!({
                        "error": format!("tool {name} timed out after {tool_timeout_secs}s")
                    }));
                    run.finished_at = Some(UtcTimestamp::now());
                    run.steps.push(AgentStep {
                        index: step_idx,
                        kind: StepKind::ToolCall,
                        name: name.clone(),
                        input: args.clone(),
                        output: serde_json::json!({"timeout": true}),
                        duration_ms: started.elapsed().as_millis() as u64,
                    });
                    self.runs.write().insert(id, run.clone());
                    return Ok(run);
                }
            };
            let duration_ms = started.elapsed().as_millis() as u64;

            last_output = resp.data.clone();
            run.steps.push(AgentStep {
                index: step_idx,
                kind: StepKind::ToolCall,
                name: name.clone(),
                input: args.clone(),
                output: serde_json::to_value(&resp).unwrap_or_default(),
                duration_ms,
            });
            step_idx += 1;

            if !resp.ok {
                run.status = RunStatus::Failed;
                run.output = Some(serde_json::json!({ "error": resp.error }));
                run.finished_at = Some(UtcTimestamp::now());
                self.runs.write().insert(id, run.clone());
                return Ok(run);
            }
        }

        run.steps.push(AgentStep {
            index: step_idx,
            kind: StepKind::Final,
            name: "finalize".into(),
            input: serde_json::json!({ "agent": agent_name }),
            output: last_output.clone(),
            duration_ms: 0,
        });
        run.status = RunStatus::Succeeded;
        run.output = Some(last_output);
        run.finished_at = Some(UtcTimestamp::now());
        self.runs.write().insert(id, run.clone());
        Ok(run)
    }

    pub fn get_run(&self, id: Uuid) -> Option<AgentRun> {
        self.runs.read().get(&id).cloned()
    }

    /// Cancel a running/pending in-memory run (best-effort; does not abort in-flight tool).
    pub fn cancel_run(&self, id: Uuid) -> Option<AgentRun> {
        let mut guard = self.runs.write();
        let run = guard.get_mut(&id)?;
        if matches!(run.status, RunStatus::Running | RunStatus::Pending) {
            run.status = RunStatus::Cancelled;
            run.finished_at = Some(UtcTimestamp::now());
            run.output = Some(serde_json::json!({"reason": "cancelled"}));
        }
        Some(run.clone())
    }

    pub fn list_agents(&self) -> Vec<AgentSpec> {
        self.specs.read().values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_echo_agent() {
        let rt = AgentRuntime::with_defaults();
        rt.register_agent(AgentSpec {
            name: "echo-bot".into(),
            description: "echo".into(),
            system_prompt: "echo".into(),
            tools: vec!["echo".into()],
            max_steps: 3,
        });
        let run = rt
            .run(
                "echo-bot",
                TenantId::new(),
                UserId::new(),
                serde_json::json!({"args": {"msg": "hi"}}),
            )
            .await
            .unwrap();
        assert_eq!(run.status, RunStatus::Succeeded);
        assert_eq!(run.output.unwrap()["msg"], "hi");
    }

    #[tokio::test]
    async fn rejects_tool_not_on_spec() {
        let rt = AgentRuntime::with_defaults();
        rt.register_agent(AgentSpec {
            name: "strict".into(),
            description: "strict".into(),
            system_prompt: "strict".into(),
            tools: vec!["echo".into()],
            max_steps: 3,
        });
        let run = rt
            .run(
                "strict",
                TenantId::new(),
                UserId::new(),
                serde_json::json!({"tools": ["product_catalog"], "args": {}}),
            )
            .await
            .unwrap();
        assert_eq!(run.status, RunStatus::Failed);
    }

    #[tokio::test]
    async fn multi_tool_default_set() {
        let rt = AgentRuntime::with_defaults();
        rt.register_agent(AgentSpec {
            name: "multi".into(),
            description: "multi".into(),
            system_prompt: "multi".into(),
            tools: vec![
                "echo".into(),
                "product_catalog".into(),
                "utc_now".into(),
                "tenant_context".into(),
            ],
            max_steps: 8,
        });
        let tid = TenantId::new();
        let uid = UserId::new();
        let run = rt
            .run(
                "multi",
                tid,
                uid,
                serde_json::json!({
                    "tools": ["tenant_context", "utc_now", "product_catalog"],
                    "args": {}
                }),
            )
            .await
            .unwrap();
        assert_eq!(run.status, RunStatus::Succeeded);
        assert!(run.steps.len() >= 3);
    }
}
