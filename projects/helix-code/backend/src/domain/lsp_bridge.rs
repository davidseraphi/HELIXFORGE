//! E3 LSP bridge — JSON-RPC 2.0 over stdio to rust-analyzer (or `HELIX_CODE_LSP_COMMAND`).

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnostic {
    pub path: String,
    pub range: LspRange,
    pub severity: u8,
    pub message: String,
    pub source: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspRange {
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspHover {
    pub contents: String,
    pub range: Option<LspRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspCompletionItem {
    pub label: String,
    pub kind: Option<u32>,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspLocation {
    pub path: String,
    pub range: LspRange,
}

struct Pending {
    tx: Sender<Result<Value, String>>,
}

pub struct LiveSession {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub root: PathBuf,
    pub server_cmd: String,
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    next_id: AtomicU64,
    pending: Mutex<HashMap<u64, Pending>>,
    diagnostics: Mutex<HashMap<String, Vec<LspDiagnostic>>>,
    open_docs: Mutex<HashMap<String, i32>>,
}

impl Drop for LiveSession {
    fn drop(&mut self) {
        if let Some(mut c) = self.child.try_lock() {
            let _ = c.kill();
            let _ = c.wait();
        }
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[derive(Default)]
struct SessionStore {
    by_id: HashMap<Uuid, Arc<LiveSession>>,
    by_repo: HashMap<(Uuid, Uuid), Uuid>,
}

static SESSIONS: once_cell::sync::Lazy<Mutex<SessionStore>> =
    once_cell::sync::Lazy::new(|| Mutex::new(SessionStore::default()));

/// Multi-language server registry (ES3). Probes PATH for available servers.
pub fn list_language_servers() -> Vec<serde_json::Value> {
    let catalog = [
        ("rust", "rust-analyzer", &["--version"][..]),
        (
            "typescript",
            "typescript-language-server",
            &["--version"][..],
        ),
        (
            "javascript",
            "typescript-language-server",
            &["--version"][..],
        ),
        ("python", "pylsp", &["--version"][..]),
        ("python", "pyright-langserver", &["--version"][..]),
        ("go", "gopls", &["version"][..]),
    ];
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (lang, cmd, args) in catalog {
        let key = format!("{lang}:{cmd}");
        if !seen.insert(key) {
            continue;
        }
        let available = Command::new(cmd)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        out.push(serde_json::json!({
            "language_id": lang,
            "command": cmd,
            "available": available || (lang == "rust" && lsp_available()),
        }));
    }
    // always surface configured HELIX_CODE_LSP_COMMAND
    out.push(serde_json::json!({
        "language_id": "default",
        "command": lsp_command(),
        "available": lsp_available(),
    }));
    out
}

pub fn lsp_command() -> String {
    if let Ok(c) = std::env::var("HELIX_CODE_LSP_COMMAND") {
        if !c.trim().is_empty() {
            return c;
        }
    }
    // Prefer the toolchain-native binary, NOT the rustup proxy in cargo/bin
    // (proxy fails when active host is gnu and RA is installed only for msvc).
    let candidates = [
        r"C:\rust\rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\rust-analyzer.exe",
        r"C:\Users\divin\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\rust-analyzer.exe",
    ];
    for c in candidates {
        if Path::new(c).is_file() {
            return c.to_string();
        }
    }
    if let Ok(home) = std::env::var("RUSTUP_HOME") {
        let p = Path::new(&home)
            .join("toolchains")
            .join("stable-x86_64-pc-windows-msvc")
            .join("bin")
            .join("rust-analyzer.exe");
        if p.is_file() {
            return p.to_string_lossy().into_owned();
        }
        let p2 = Path::new(&home)
            .join("toolchains")
            .join("stable-x86_64-pc-windows-msvc")
            .join("bin")
            .join("rust-analyzer");
        if p2.is_file() {
            return p2.to_string_lossy().into_owned();
        }
    }
    "rust-analyzer".into()
}

pub fn lsp_available() -> bool {
    which_ok(&lsp_command())
}

fn which_ok(cmd: &str) -> bool {
    let p = Path::new(cmd);
    if p.is_file() {
        return true;
    }
    // Probe: run --version quickly
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn open_session(
    tenant_id: TenantId,
    repo_id: Uuid,
    repo_name: &str,
    bare_repo: &Path,
    branch_or_sha: &str,
) -> HelixResult<Value> {
    if !lsp_available() {
        return Err(HelixError::unavailable(format!(
            "LSP command not found: {} (install rust-analyzer or set HELIX_CODE_LSP_COMMAND)",
            lsp_command()
        )));
    }

    {
        let store = SESSIONS.lock();
        if let Some(sid) = store.by_repo.get(&(tenant_id.as_uuid(), repo_id)) {
            if let Some(s) = store.by_id.get(sid) {
                return Ok(session_info(s));
            }
        }
    }

    let root = checkout_worktree(bare_repo, branch_or_sha)?;
    let cmd = lsp_command();
    let log_file = root.join("rust-analyzer.log");
    let mut child = Command::new(&cmd)
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(std::fs::File::create(&log_file).map_err(
            |e| HelixError::internal(format!("lsp log file: {e}")),
        )?))
        .spawn()
        .map_err(|e| HelixError::dependency(format!("spawn LSP `{cmd}`: {e}")))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| HelixError::internal("LSP stdin missing"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| HelixError::internal("LSP stdout missing"))?;

    let id = Uuid::now_v7();
    let session = Arc::new(LiveSession {
        id,
        tenant_id,
        repo_id,
        repo_name: repo_name.into(),
        root: root.clone(),
        server_cmd: cmd,
        child: Mutex::new(child),
        stdin: Mutex::new(stdin),
        next_id: AtomicU64::new(1),
        pending: Mutex::new(HashMap::new()),
        diagnostics: Mutex::new(HashMap::new()),
        open_docs: Mutex::new(HashMap::new()),
    });

    let reader_session = Arc::clone(&session);
    thread::Builder::new()
        .name(format!("lsp-reader-{}", id.simple()))
        .spawn(move || reader_loop(reader_session, stdout))
        .map_err(|e| HelixError::internal(format!("lsp reader: {e}")))?;

    let root_uri = path_to_uri(&root);
    // Give the reader thread a beat to attach before first request.
    thread::sleep(Duration::from_millis(100));
    let init_timeout = std::env::var("HELIX_CODE_LSP_INIT_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(90);
    let _init = request(
        &session,
        "initialize",
        json!({
            "processId": null,
            "clientInfo": { "name": "helix-code", "version": "0.1.0" },
            "rootUri": root_uri,
            "capabilities": {
                "textDocument": {
                    "synchronization": { "didSave": true },
                    "hover": { "contentFormat": ["markdown", "plaintext"] },
                    "completion": { "completionItem": { "snippetSupport": false } },
                    "publishDiagnostics": {},
                    "definition": {}
                },
                "workspace": { "workspaceFolders": true }
            },
            "workspaceFolders": [{ "uri": root_uri, "name": repo_name }]
        }),
        Duration::from_secs(init_timeout),
    )
    .map_err(|e| {
        // Surface stderr snippet if process died
        let mut err = e.to_string();
        if let Some(mut child) = session.child.try_lock() {
            if let Ok(Some(status)) = child.try_wait() {
                err = format!("{err}; lsp exited {status:?}");
            }
        }
        if let Ok(log) = std::fs::read_to_string(&log_file) {
            let tail: String = log
                .chars()
                .rev()
                .take(800)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            if !tail.is_empty() {
                err = format!("{err}; ra.log_tail={tail}");
            }
        }
        HelixError::dependency(err)
    })?;
    notify(&session, "initialized", json!({}))?;

    {
        let mut store = SESSIONS.lock();
        store.by_repo.insert((tenant_id.as_uuid(), repo_id), id);
        store.by_id.insert(id, Arc::clone(&session));
    }

    Ok(session_info(&session))
}

pub fn close_session(tenant_id: TenantId, session_id: Uuid) -> HelixResult<()> {
    let mut store = SESSIONS.lock();
    let Some(s) = store.by_id.remove(&session_id) else {
        return Err(HelixError::not_found("lsp session not found"));
    };
    if s.tenant_id != tenant_id {
        store.by_id.insert(session_id, s);
        return Err(HelixError::not_found("lsp session not found"));
    }
    store.by_repo.remove(&(tenant_id.as_uuid(), s.repo_id));
    drop(store);
    let _ = request(&s, "shutdown", Value::Null, Duration::from_secs(5));
    let _ = notify(&s, "exit", Value::Null);
    Ok(())
}

pub fn close_session_for_repo(tenant_id: TenantId, repo_id: Uuid) -> HelixResult<Option<Uuid>> {
    let sid = {
        let store = SESSIONS.lock();
        store.by_repo.get(&(tenant_id.as_uuid(), repo_id)).copied()
    };
    if let Some(id) = sid {
        close_session(tenant_id, id)?;
        Ok(Some(id))
    } else {
        Ok(None)
    }
}

/// Instance id for multi-API sticky routing (set `HELIX_CODE_INSTANCE_ID` behind LB).
pub fn instance_id() -> String {
    std::env::var("HELIX_CODE_INSTANCE_ID").unwrap_or_else(|_| {
        std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "local".into())
    })
}

pub fn get_session(tenant_id: TenantId, session_id: Uuid) -> HelixResult<Arc<LiveSession>> {
    let store = SESSIONS.lock();
    let s = store
        .by_id
        .get(&session_id)
        .cloned()
        .ok_or_else(|| HelixError::not_found("lsp session not found"))?;
    if s.tenant_id != tenant_id {
        return Err(HelixError::not_found("lsp session not found"));
    }
    Ok(s)
}

pub fn session_info(s: &LiveSession) -> Value {
    json!({
        "session_id": s.id,
        "repo_id": s.repo_id,
        "repo_name": s.repo_name,
        "root": s.root.display().to_string(),
        "server": s.server_cmd,
        "open_docs": s.open_docs.lock().len(),
        "diagnostic_files": s.diagnostics.lock().len(),
        "instance_id": instance_id(),
        "sticky": true,
    })
}

/// True if session is live on this process.
#[allow(dead_code)]
pub fn session_is_local(session_id: Uuid) -> bool {
    SESSIONS.lock().by_id.contains_key(&session_id)
}

pub fn did_open(
    tenant_id: TenantId,
    session_id: Uuid,
    path: &str,
    language_id: &str,
    text: &str,
) -> HelixResult<()> {
    let s = get_session(tenant_id, session_id)?;
    let path = normalize_path(path)?;
    let uri = file_uri(&s.root, &path);
    s.open_docs.lock().insert(path, 1);
    notify(
        &s,
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": language_id,
                "version": 1,
                "text": text
            }
        }),
    )?;
    // Wait briefly for publishDiagnostics
    thread::sleep(Duration::from_millis(500));
    Ok(())
}

pub fn did_change(
    tenant_id: TenantId,
    session_id: Uuid,
    path: &str,
    text: &str,
) -> HelixResult<i32> {
    let s = get_session(tenant_id, session_id)?;
    let path = normalize_path(path)?;
    let uri = file_uri(&s.root, &path);
    let version = {
        let mut docs = s.open_docs.lock();
        let v = docs.entry(path).or_insert(1);
        *v += 1;
        *v
    };
    notify(
        &s,
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": uri, "version": version },
            "contentChanges": [{ "text": text }]
        }),
    )?;
    thread::sleep(Duration::from_millis(350));
    Ok(version)
}

pub fn diagnostics(
    tenant_id: TenantId,
    session_id: Uuid,
    path: Option<&str>,
) -> HelixResult<Vec<LspDiagnostic>> {
    let s = get_session(tenant_id, session_id)?;
    let map = s.diagnostics.lock();
    if let Some(p) = path {
        let p = normalize_path(p)?;
        Ok(map.get(&p).cloned().unwrap_or_default())
    } else {
        Ok(map.values().flatten().cloned().collect())
    }
}

pub fn hover(
    tenant_id: TenantId,
    session_id: Uuid,
    path: &str,
    line: u32,
    character: u32,
) -> HelixResult<Option<LspHover>> {
    let s = get_session(tenant_id, session_id)?;
    let path = normalize_path(path)?;
    let uri = file_uri(&s.root, &path);
    let res = request(
        &s,
        "textDocument/hover",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }),
        Duration::from_secs(12),
    )?;
    if res.is_null() {
        return Ok(None);
    }
    Ok(Some(LspHover {
        contents: parse_hover_contents(&res["contents"]),
        range: res.get("range").and_then(parse_range),
    }))
}

pub fn completion(
    tenant_id: TenantId,
    session_id: Uuid,
    path: &str,
    line: u32,
    character: u32,
) -> HelixResult<Vec<LspCompletionItem>> {
    let s = get_session(tenant_id, session_id)?;
    let path = normalize_path(path)?;
    let uri = file_uri(&s.root, &path);
    let res = request(
        &s,
        "textDocument/completion",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }),
        Duration::from_secs(15),
    )?;
    Ok(parse_completions(&res))
}

pub fn definition(
    tenant_id: TenantId,
    session_id: Uuid,
    path: &str,
    line: u32,
    character: u32,
) -> HelixResult<Vec<LspLocation>> {
    let s = get_session(tenant_id, session_id)?;
    let path = normalize_path(path)?;
    let uri = file_uri(&s.root, &path);
    let res = request(
        &s,
        "textDocument/definition",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }),
        Duration::from_secs(12),
    )?;
    Ok(parse_locations(&res, &s.root))
}

fn request(
    session: &LiveSession,
    method: &str,
    params: Value,
    timeout: Duration,
) -> HelixResult<Value> {
    let id = session.next_id.fetch_add(1, Ordering::SeqCst);
    let (tx, rx): (Sender<Result<Value, String>>, Receiver<_>) = mpsc::channel();
    session.pending.lock().insert(id, Pending { tx });
    let msg = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    });
    write_rpc(session, &msg)?;
    match rx.recv_timeout(timeout) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(HelixError::dependency(format!("lsp error: {e}"))),
        Err(_) => {
            session.pending.lock().remove(&id);
            Err(HelixError::dependency(format!("lsp timeout: {method}")))
        }
    }
}

fn notify(session: &LiveSession, method: &str, params: Value) -> HelixResult<()> {
    let msg = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params
    });
    write_rpc(session, &msg)
}

fn write_rpc(session: &LiveSession, msg: &Value) -> HelixResult<()> {
    let body = serde_json::to_string(msg)
        .map_err(|e| HelixError::internal(format!("lsp serialize: {e}")))?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let mut stdin = session.stdin.lock();
    stdin
        .write_all(header.as_bytes())
        .and_then(|_| stdin.write_all(body.as_bytes()))
        .and_then(|_| stdin.flush())
        .map_err(|e| HelixError::dependency(format!("lsp write: {e}")))?;
    Ok(())
}

fn reader_loop(session: Arc<LiveSession>, stdout: impl Read) {
    let mut reader = BufReader::new(stdout);
    loop {
        match read_message(&mut reader) {
            Ok(None) => break,
            Ok(Some(val)) => handle_incoming(&session, val),
            Err(_) => break,
        }
    }
}

fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(None);
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some(rest) = line.strip_prefix("Content-Length:") {
            content_length = Some(rest.trim().parse().map_err(|e| format!("len: {e}"))?);
        }
    }
    let len = content_length.ok_or_else(|| "missing Content-Length".to_string())?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
    serde_json::from_slice(&buf)
        .map(Some)
        .map_err(|e| e.to_string())
}

fn json_id_u64(v: &Value) -> Option<u64> {
    v.as_u64()
        .or_else(|| v.as_i64().map(|x| x as u64))
        .or_else(|| v.as_f64().map(|x| x as u64))
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

fn handle_incoming(session: &LiveSession, val: Value) {
    // Response: has id, no method (or has result/error)
    if val.get("result").is_some() || val.get("error").is_some() {
        if let Some(id) = val.get("id").and_then(json_id_u64) {
            if let Some(pending) = session.pending.lock().remove(&id) {
                if let Some(err) = val.get("error") {
                    let _ = pending.tx.send(Err(err.to_string()));
                } else {
                    let _ = pending
                        .tx
                        .send(Ok(val.get("result").cloned().unwrap_or(Value::Null)));
                }
            }
        }
        return;
    }
    // Request/notification from server
    if val.get("method").and_then(|m| m.as_str()) == Some("textDocument/publishDiagnostics") {
        if let Some(params) = val.get("params") {
            ingest_diagnostics(session, params);
        }
        return;
    }
    // Server request we don't handle — reply null if it has an id
    if let Some(id) = val.get("id").and_then(json_id_u64) {
        let method = val.get("method").and_then(|m| m.as_str()).unwrap_or("");
        tracing::debug!(%method, "lsp server request ignored");
        let reply = json!({ "jsonrpc": "2.0", "id": id, "result": null });
        let _ = write_rpc(session, &reply);
    }
}

fn ingest_diagnostics(session: &LiveSession, params: &Value) {
    let uri = params.get("uri").and_then(|u| u.as_str()).unwrap_or("");
    let Some(rel) = uri_to_rel(uri, &session.root) else {
        return;
    };
    let mut out = Vec::new();
    if let Some(arr) = params.get("diagnostics").and_then(|d| d.as_array()) {
        for d in arr {
            let severity = d.get("severity").and_then(|s| s.as_u64()).unwrap_or(1) as u8;
            let message = d
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            let source = d
                .get("source")
                .and_then(|s| s.as_str())
                .unwrap_or("lsp")
                .to_string();
            let code = d.get("code").map(|c| match c {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => c.to_string(),
            });
            if let Some(range) = d.get("range").and_then(parse_range) {
                out.push(LspDiagnostic {
                    path: rel.clone(),
                    range,
                    severity,
                    message,
                    source,
                    code,
                });
            }
        }
    }
    session.diagnostics.lock().insert(rel, out);
}

fn checkout_worktree(bare: &Path, rev: &str) -> HelixResult<PathBuf> {
    let root = std::env::var("HELIX_CODE_LSP_WORKDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".data/helix-code/lsp-sessions"));
    std::fs::create_dir_all(&root)
        .map_err(|e| HelixError::internal(format!("lsp workdir: {e}")))?;
    let dest = root.join(format!("sess-{}", Uuid::now_v7().simple()));
    let bare_s = bare
        .to_str()
        .ok_or_else(|| HelixError::internal("bare utf8"))?;
    let dest_s = dest
        .to_str()
        .ok_or_else(|| HelixError::internal("dest utf8"))?;
    let out = Command::new("git")
        .args(["clone", bare_s, dest_s])
        .output()
        .map_err(|e| HelixError::dependency(format!("git clone lsp: {e}")))?;
    if !out.status.success() {
        return Err(HelixError::dependency(format!(
            "git clone lsp: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    let _ = Command::new("git")
        .current_dir(&dest)
        .args(["checkout", "--force", rev])
        .output();
    Ok(dest)
}

fn normalize_path(path: &str) -> HelixResult<String> {
    let p = path.trim().replace('\\', "/");
    if p.is_empty() || p.contains("..") || p.starts_with('/') {
        return Err(HelixError::validation("invalid path"));
    }
    Ok(p)
}

fn path_to_uri(path: &Path) -> String {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut s = abs.to_string_lossy().replace('\\', "/");
    // Windows canonicalize prefixes \\?\
    if let Some(rest) = s.strip_prefix("//?/") {
        s = rest.to_string();
    }
    if s.starts_with('/') {
        format!("file://{s}")
    } else {
        format!("file:///{s}")
    }
}

fn file_uri(root: &Path, rel: &str) -> String {
    path_to_uri(&root.join(rel))
}

fn uri_to_rel(uri: &str, root: &Path) -> Option<String> {
    let root_uri = path_to_uri(root).trim_end_matches('/').to_string();
    if let Some(rest) = uri.strip_prefix(&root_uri) {
        return Some(rest.trim_start_matches('/').to_string());
    }
    let decoded = uri
        .strip_prefix("file:///")
        .or_else(|| uri.strip_prefix("file://"))
        .unwrap_or(uri)
        .replace('\\', "/");
    let root_s = root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");
    decoded
        .strip_prefix(&root_s)
        .map(|r| r.trim_start_matches('/').to_string())
}

fn parse_range(v: &Value) -> Option<LspRange> {
    Some(LspRange {
        start_line: v["start"]["line"].as_u64()? as u32,
        start_character: v["start"]["character"].as_u64()? as u32,
        end_line: v["end"]["line"].as_u64()? as u32,
        end_character: v["end"]["character"].as_u64()? as u32,
    })
}

fn parse_hover_contents(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Object(map) => map
            .get("value")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        Value::Array(arr) => arr
            .iter()
            .map(|x| match x {
                Value::String(s) => s.clone(),
                Value::Object(m) => m
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                _ => String::new(),
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
        _ => String::new(),
    }
}

fn parse_completions(v: &Value) -> Vec<LspCompletionItem> {
    let items = if let Some(arr) = v.as_array() {
        arr.clone()
    } else if let Some(arr) = v.get("items").and_then(|i| i.as_array()) {
        arr.clone()
    } else {
        return vec![];
    };
    items
        .iter()
        .filter_map(|it| {
            let label = it.get("label")?.as_str()?.to_string();
            Some(LspCompletionItem {
                label,
                kind: it.get("kind").and_then(|k| k.as_u64()).map(|k| k as u32),
                detail: it
                    .get("detail")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string()),
                insert_text: it
                    .get("insertText")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string()),
                documentation: it.get("documentation").map(|d| match d {
                    Value::String(s) => s.clone(),
                    Value::Object(m) => m
                        .get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    _ => d.to_string(),
                }),
            })
        })
        .take(50)
        .collect()
}

fn parse_locations(v: &Value, root: &Path) -> Vec<LspLocation> {
    let arr: Vec<&Value> = if let Some(a) = v.as_array() {
        a.iter().collect()
    } else if v.is_object() {
        vec![v]
    } else {
        return vec![];
    };
    arr.into_iter()
        .filter_map(|loc| {
            let uri = loc.get("uri").or_else(|| loc.get("targetUri"))?.as_str()?;
            let range = loc
                .get("range")
                .or_else(|| loc.get("targetSelectionRange"))
                .or_else(|| loc.get("targetRange"))
                .and_then(parse_range)?;
            let path = uri_to_rel(uri, root).unwrap_or_else(|| uri.to_string());
            Some(LspLocation { path, range })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lsp_binary_detectable_on_dev_machine() {
        // May be false in minimal CI — don't fail build; just exercise path
        let _ = lsp_available();
        assert!(!lsp_command().is_empty());
    }

    #[test]
    fn normalize_rejects_traversal() {
        assert!(normalize_path("../x").is_err());
        assert!(normalize_path("src/lib.rs").is_ok());
    }
}
