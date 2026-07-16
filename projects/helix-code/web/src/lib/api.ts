import { API, DEV_USER } from "./config";

async function req<T>(
  path: string,
  init: RequestInit = {},
): Promise<T> {
  const headers = new Headers(init.headers);
  headers.set("Content-Type", "application/json");
  headers.set("x-helix-dev-user", DEV_USER);
  const res = await fetch(`${API}${path}`, { ...init, headers });
  const body = await res.json().catch(() => ({}));
  if (!res.ok) {
    const msg =
      body?.error?.message ||
      body?.message ||
      `${res.status} ${res.statusText}`;
    throw new Error(msg);
  }
  return (body.data ?? body) as T;
}

export type CodeRepo = {
  id: string;
  name: string;
  description?: string;
  default_branch?: string;
  head_sha?: string | null;
  visibility?: string;
};

export type TreeEntry = {
  path: string;
  kind: string;
  mode: string;
  oid: string;
};

export type CommitInfo = {
  sha: string;
  message: string;
  author: string;
  time: string;
};

export type DomainStatus = {
  domain: string;
  phase: string;
  durable: boolean;
  git_store_root?: string;
  planes?: Record<string, unknown>;
  ci?: Record<string, unknown>;
};

export type LspDiagnostic = {
  path: string;
  range: {
    start_line: number;
    start_character: number;
    end_line: number;
    end_character: number;
  };
  severity: number;
  message: string;
  source: string;
  code?: string | null;
};

export type LspSession = {
  session_id: string;
  repo_id: string;
  repo_name: string;
  root: string;
  server: string;
};

export type LspCompletionItem = {
  label: string;
  kind?: number | null;
  detail?: string | null;
  insert_text?: string | null;
  documentation?: string | null;
};

export type LspLocation = {
  path: string;
  range: {
    start_line: number;
    start_character: number;
    end_line: number;
    end_character: number;
  };
};

export type PipelineRun = {
  id: string;
  status: string;
  exit_code?: number | null;
  isolation?: string;
  log_text?: string;
  commit_sha?: string | null;
};

export type AgentJob = {
  id: string;
  status: string;
  kind: string;
  isolation?: string;
  commit_sha?: string | null;
  files_changed?: string[] | unknown;
  result_summary?: string;
  mesh_steps?: unknown;
};

export const api = {
  domainStatus: () => req<DomainStatus>("/v1/domain/status"),

  listRepos: async () => {
    const data = await req<{ items: CodeRepo[] }>("/v1/repos");
    return data.items ?? [];
  },

  createRepo: (name: string, description = "") =>
    req<CodeRepo>("/v1/repos", {
      method: "POST",
      body: JSON.stringify({ name, description, visibility: "private" }),
    }),

  tree: (repoId: string, rev = "main", path = "") =>
    req<{ rev: string; path: string; entries: TreeEntry[] }>(
      `/v1/repos/${repoId}/tree?rev=${encodeURIComponent(rev)}&path=${encodeURIComponent(path)}`,
    ),

  blob: (repoId: string, path: string, rev = "main") =>
    req<{ path: string; rev: string; content: string }>(
      `/v1/repos/${repoId}/blob?rev=${encodeURIComponent(rev)}&path=${encodeURIComponent(path)}`,
    ),

  commit: (
    repoId: string,
    body: { path: string; content: string; message: string; branch?: string },
  ) =>
    req<{ commit_sha: string; path: string; branch: string }>(
      `/v1/repos/${repoId}/commits`,
      { method: "POST", body: JSON.stringify(body) },
    ),

  log: (repoId: string, rev = "main", limit = 12) =>
    req<{ commits: CommitInfo[] }>(
      `/v1/repos/${repoId}/log?rev=${encodeURIComponent(rev)}&limit=${limit}`,
    ),

  listFiles: (repoId: string, rev = "main", max = 2000) =>
    req<{ rev: string; count: number; files: string[] }>(
      `/v1/repos/${repoId}/files?rev=${encodeURIComponent(rev)}&max=${max}`,
    ),

  search: (repoId: string, q: string, rev = "main", max = 50) =>
    req<{
      query: string;
      rev: string;
      count: number;
      hits: { path: string; line: number; preview: string }[];
    }>(
      `/v1/repos/${repoId}/search?q=${encodeURIComponent(q)}&rev=${encodeURIComponent(rev)}&max=${max}`,
    ),

  commitBatch: (
    repoId: string,
    body: {
      files: { path: string; content: string }[];
      message: string;
      branch?: string;
    },
  ) =>
    req<{ commit_sha: string; paths: string[]; count: number }>(
      `/v1/repos/${repoId}/commits/batch`,
      { method: "POST", body: JSON.stringify(body) },
    ),

  createWorkspace: (repoId: string, name: string, branch = "main") =>
    req<{ id: string; name: string; branch: string }>(
      "/v1/code/workspaces",
      {
        method: "POST",
        body: JSON.stringify({
          repo_id: repoId,
          name,
          branch,
          root_path: "",
        }),
      },
    ),

  listPipelines: (repoId: string) =>
    req<{ items: { id: string; name: string }[] }>(
      `/v1/repos/${repoId}/pipelines`,
    ),

  createPipeline: (
    repoId: string,
    name: string,
    definition: Record<string, unknown>,
  ) =>
    req<{ id: string; name: string }>(`/v1/repos/${repoId}/pipelines`, {
      method: "POST",
      body: JSON.stringify({ name, definition }),
    }),

  triggerPipeline: (pipelineId: string, trigger_ref = "refs/heads/main") =>
    req<PipelineRun>(`/v1/pipelines/${pipelineId}/runs`, {
      method: "POST",
      body: JSON.stringify({ trigger_ref }),
    }),

  getPipelineRun: (runId: string) =>
    req<PipelineRun>(`/v1/pipeline-runs/${runId}`),

  createAgentJob: (
    repoId: string,
    body: {
      prompt: string;
      kind?: string;
      branch?: string;
      commit?: boolean;
      commit_message?: string;
      patches?: { path: string; content: string; create?: boolean }[];
      agents?: string[];
    },
  ) =>
    req<AgentJob>(`/v1/repos/${repoId}/agent-jobs`, {
      method: "POST",
      body: JSON.stringify(body),
    }),

  getAgentJob: (jobId: string) => req<AgentJob>(`/v1/agent-jobs/${jobId}`),

  mlsStatus: () =>
    req<{ openmls: boolean; ciphersuite: string; user_hydrated: boolean }>(
      "/v1/mls/status",
    ),

  mlsIdentity: (label = "forge") =>
    req<{ user_key: string; signature_public_b64: string }>(
      "/v1/mls/identity",
      { method: "POST", body: JSON.stringify({ label }) },
    ),

  mlsCreateGroup: (name = "web-group", repoId?: string) =>
    req<{ group_id: string; epoch: number; member_count: number }>(
      "/v1/mls/groups",
      {
        method: "POST",
        body: JSON.stringify({ name, repo_id: repoId }),
      },
    ),

  lspStatus: () =>
    req<{ available: boolean; command: string }>("/v1/lsp/status"),

  lspOpenSession: (repoId: string, rev = "main") =>
    req<LspSession>(`/v1/repos/${repoId}/lsp/session`, {
      method: "POST",
      body: JSON.stringify({ rev }),
    }),

  lspDidOpen: (
    sessionId: string,
    path: string,
    content: string,
    language_id?: string,
  ) =>
    req<{ path: string; diagnostics: LspDiagnostic[] }>(
      `/v1/lsp/sessions/${sessionId}/did-open`,
      {
        method: "POST",
        body: JSON.stringify({ path, content, language_id }),
      },
    ),

  lspDidChange: (sessionId: string, path: string, content: string) =>
    req<{ version: number; diagnostics: LspDiagnostic[] }>(
      `/v1/lsp/sessions/${sessionId}/did-change`,
      {
        method: "POST",
        body: JSON.stringify({ path, content }),
      },
    ),

  lspDiagnostics: (sessionId: string, path?: string) =>
    req<{ items: LspDiagnostic[] }>(
      `/v1/lsp/sessions/${sessionId}/diagnostics${
        path ? `?path=${encodeURIComponent(path)}` : ""
      }`,
    ),

  lspHover: (
    sessionId: string,
    path: string,
    line: number,
    character: number,
  ) =>
    req<{ hover: { contents: string } | null }>(
      `/v1/lsp/sessions/${sessionId}/hover`,
      {
        method: "POST",
        body: JSON.stringify({ path, line, character }),
      },
    ),

  lspCompletion: (
    sessionId: string,
    path: string,
    line: number,
    character: number,
  ) =>
    req<{ items: LspCompletionItem[] }>(
      `/v1/lsp/sessions/${sessionId}/completion`,
      {
        method: "POST",
        body: JSON.stringify({ path, line, character }),
      },
    ),

  lspDefinition: (
    sessionId: string,
    path: string,
    line: number,
    character: number,
  ) =>
    req<{ items: LspLocation[] }>(
      `/v1/lsp/sessions/${sessionId}/definition`,
      {
        method: "POST",
        body: JSON.stringify({ path, line, character }),
      },
    ),

  // —— end-state ——
  listIssues: (repoId: string) =>
    req<{ items: unknown[] }>(`/v1/repos/${repoId}/issues`),
  createIssue: (repoId: string, title: string, body = "") =>
    req(`/v1/repos/${repoId}/issues`, {
      method: "POST",
      body: JSON.stringify({ title, body }),
    }),
  listPulls: (repoId: string) =>
    req<{ items: unknown[] }>(`/v1/repos/${repoId}/pulls`),
  createPull: (
    repoId: string,
    body: {
      title: string;
      source_branch: string;
      target_branch?: string;
      body?: string;
    },
  ) =>
    req(`/v1/repos/${repoId}/pulls`, {
      method: "POST",
      body: JSON.stringify(body),
    }),
  mergePull: (repoId: string, number: number) =>
    req(`/v1/repos/${repoId}/pulls/${number}/merge`, {
      method: "POST",
      body: "{}",
    }),
  listProtections: (repoId: string) =>
    req<{ items: unknown[] }>(`/v1/repos/${repoId}/protections`),
  putProtection: (repoId: string, branch_pattern: string) =>
    req(`/v1/repos/${repoId}/protections`, {
      method: "PUT",
      body: JSON.stringify({
        branch_pattern,
        require_pr: true,
        deny_force_push: true,
      }),
    }),
  gitStatus: (repoId: string) => req(`/v1/repos/${repoId}/status`),
  gitDiff: (repoId: string, path = "", rev = "main") =>
    req(
      `/v1/repos/${repoId}/diff?path=${encodeURIComponent(path)}&rev=${encodeURIComponent(rev)}`,
    ),
  listPipelineRuns: (repoId: string) =>
    req<{ items: PipelineRun[] }>(`/v1/repos/${repoId}/pipeline-runs`),
  cancelPipelineRun: (runId: string) =>
    req(`/v1/pipeline-runs/${runId}/cancel`, {
      method: "POST",
      body: "{}",
    }),
  listAgentJobs: (repoId: string) =>
    req<{ items: AgentJob[] }>(`/v1/repos/${repoId}/agent-jobs`),
  agentEvents: (jobId: string, after = 0) =>
    req<{ items: unknown[] }>(
      `/v1/agent-jobs/${jobId}/events?after=${after}`,
    ),
  lspServers: () =>
    req<{ servers: { language_id: string; command: string; available: boolean }[] }>(
      "/v1/lsp/servers",
    ),
  getSettings: () => req<{ settings: Record<string, unknown> }>("/v1/me/code-settings"),
  putSettings: (settings: Record<string, unknown>) =>
    req("/v1/me/code-settings", {
      method: "PUT",
      body: JSON.stringify({ settings }),
    }),
  quotas: () => req("/v1/quotas"),
  createTerminal: (repoId: string, rev = "main") =>
    req<{ terminal_id: string }>(`/v1/repos/${repoId}/terminals`, {
      method: "POST",
      body: JSON.stringify({ rev }),
    }),
  termWrite: (terminalId: string, command: string) =>
    req<{ log: string }>(`/v1/terminals/${terminalId}`, {
      method: "POST",
      body: JSON.stringify({ command }),
    }),
  listExtensions: () => req<{ items: unknown[] }>("/v1/extensions"),
  debugLaunch: (repoId: string) =>
    req(`/v1/repos/${repoId}/debug/launch`, {
      method: "POST",
      body: JSON.stringify({ config: "launch" }),
    }),
  mlsDevices: () => req<{ items: unknown[] }>("/v1/mls/devices"),
  mlsRegisterDevice: (device_id: string, label = "web") =>
    req("/v1/mls/devices", {
      method: "POST",
      body: JSON.stringify({ device_id, label, public_identity_b64: "" }),
    }),

  listDeployKeys: (repoId: string) =>
    req<{ items: unknown[] }>(`/v1/repos/${repoId}/deploy-keys`),
  createDeployKey: (repoId: string, name: string, scope = "read") =>
    req<{ key: unknown; token: string }>(`/v1/repos/${repoId}/deploy-keys`, {
      method: "POST",
      body: JSON.stringify({ name, scope }),
    }),
  setBreakpoints: (sessionId: string, breakpoints: unknown[]) =>
    req(`/v1/debug/sessions/${sessionId}/breakpoints`, {
      method: "POST",
      body: JSON.stringify({ breakpoints }),
    }),
  debugContinue: (sessionId: string) =>
    req(`/v1/debug/sessions/${sessionId}/continue`, {
      method: "POST",
      body: "{}",
    }),
};
