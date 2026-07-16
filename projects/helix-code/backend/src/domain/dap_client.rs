//! Debug Adapter Protocol (DAP) client over stdio — lldb-dap / gdb --interpreter=dap.
//!
//! Protocol: Content-Length framed JSON messages (same framing as LSP).
//! Env:
//! - `HELIX_CODE_DAP_COMMAND` — override adapter argv (space-separated)
//! - Default probes: `lldb-dap`, `lldb-vscode`, `gdb --interpreter=dap`, `rust-gdb`
//!
//! Full control surface: initialize, launch, setBreakpoints, configurationDone,
//! continue, next, stepIn, stepOut, pause, threads, stackTrace, scopes, variables,
//! evaluate, disconnect.

use parking_lot::Mutex;
use serde_json::{json, Value};
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

struct Pending {
    tx: Sender<Result<Value, String>>,
}

pub struct DapSession {
    pub adapter_cmd: String,
    pub adapter_kind: String,
    pub status: Mutex<String>,
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    next_id: AtomicU64,
    pending: Mutex<HashMap<u64, Pending>>,
    /// Signaled when adapter sends the `initialized` event.
    initialized_tx: Mutex<Option<Sender<()>>>,
    /// Latest stop event body (if any).
    pub last_stopped: Mutex<Option<Value>>,
    pub threads: Mutex<Value>,
    pub stack: Mutex<Value>,
    pub capabilities: Mutex<Value>,
}

impl Drop for DapSession {
    fn drop(&mut self) {
        let _ = self.request_sync("disconnect", json!({ "terminateDebuggee": true }), 3);
        if let Some(mut c) = self.child.try_lock() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

static DAP: once_cell::sync::Lazy<Mutex<HashMap<Uuid, Arc<DapSession>>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

/// Common Windows / portable install locations for adapters.
fn extra_path_candidates(cmd: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from);
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    if let Some(h) = &home {
        out.push(h.join(r"scoop\shims").join(cmd));
        out.push(h.join(r"scoop\apps\gdb\current\bin").join(cmd));
        out.push(h.join(r"scoop\apps\llvm\current\bin").join(cmd));
        out.push(h.join(r".cargo\bin").join(cmd));
    }
    if let Some(l) = &local {
        out.push(l.join(r"Programs\LLVM\bin").join(cmd));
        out.push(l.join(r"Microsoft\WinGet\Links").join(cmd));
    }
    out.push(PathBuf::from(r"C:\msys64\mingw64\bin").join(cmd));
    out.push(PathBuf::from(r"C:\Program Files\LLVM\bin").join(cmd));
    out.push(PathBuf::from(r"C:\Program Files\mingw-w64\bin").join(cmd));
    // With .exe suffix variants
    if !cmd.ends_with(".exe") {
        let exe = format!("{cmd}.exe");
        out.extend(extra_path_candidates(&exe));
    }
    out
}

fn resolve_executable(cmd: &str) -> Option<String> {
    if Path::new(cmd).is_file() {
        return Some(cmd.to_string());
    }
    if which_ok(cmd) {
        return Some(cmd.to_string());
    }
    for p in extra_path_candidates(cmd) {
        if p.is_file() {
            return Some(p.to_string_lossy().into_owned());
        }
    }
    None
}

/// True when `cmd args` can speak DAP (lldb-dap always; gdb only if built with dap).
fn supports_dap(cmd: &str, args: &[&str]) -> bool {
    // Pure DAP adapters
    let base = Path::new(cmd)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(cmd)
        .to_ascii_lowercase();
    if base == "lldb-dap" || base == "lldb-vscode" {
        return resolve_executable(cmd).is_some() || which_ok(cmd);
    }
    // gdb: many Windows builds advertise --interpreter= but lack the `dap` UI
    if base == "gdb" || base == "rust-gdb" {
        let resolved = resolve_executable(cmd).unwrap_or_else(|| cmd.to_string());
        let mut c = Command::new(&resolved);
        for a in args {
            c.arg(a);
        }
        // Spawn briefly; if stderr says unrecognized, reject.
        let Ok(mut child) = c
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        else {
            return false;
        };
        thread::sleep(Duration::from_millis(150));
        let mut bad = false;
        if let Some(mut err) = child.stderr.take() {
            thread::sleep(Duration::from_millis(200));
            let mut buf = String::new();
            let _ = err.read_to_string(&mut buf);
            let lower = buf.to_ascii_lowercase();
            if lower.contains("unrecognized")
                || lower.contains("interpreter `dap'")
                || lower.contains("interpreter 'dap'")
            {
                bad = true;
            }
        }
        let _ = child.kill();
        let _ = child.wait();
        return !bad;
    }
    resolve_executable(cmd).is_some() || which_ok(cmd)
}

/// Resolve adapter command line.
pub fn resolve_dap_adapter() -> (String, Vec<String>, String) {
    if let Ok(raw) = std::env::var("HELIX_CODE_DAP_COMMAND") {
        let mut parts = raw
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        if !parts.is_empty() {
            let cmd = parts.remove(0);
            let kind = if cmd.contains("lldb") { "lldb" } else { "gdb" }.to_string();
            return (cmd, parts, kind);
        }
    }
    // Prefer DAP-native adapters; skip gdb builds without dap interpreter.
    let candidates: &[(&str, &[&str], &str)] = &[
        ("lldb-dap", &[], "lldb"),
        ("lldb-dap.exe", &[], "lldb"),
        ("lldb-vscode", &[], "lldb"),
        ("lldb-vscode.exe", &[], "lldb"),
        ("gdb", &["--interpreter=dap"], "gdb"),
        ("gdb.exe", &["--interpreter=dap"], "gdb"),
        ("rust-gdb", &["--interpreter=dap"], "gdb"),
        ("rust-gdb.exe", &["--interpreter=dap"], "gdb"),
    ];
    for (cmd, args, kind) in candidates {
        if resolve_executable(cmd).is_none() && !which_ok(cmd) {
            continue;
        }
        if supports_dap(cmd, args) {
            let resolved = resolve_executable(cmd).unwrap_or_else(|| (*cmd).to_string());
            return (
                resolved,
                args.iter().map(|s| (*s).to_string()).collect(),
                (*kind).to_string(),
            );
        }
    }
    // Last resort: still report gdb path (start_session will error with install hint)
    (
        resolve_executable("gdb")
            .or_else(|| resolve_executable("gdb.exe"))
            .unwrap_or_else(|| "gdb".into()),
        vec!["--interpreter=dap".into()],
        "gdb".into(),
    )
}

fn which_ok(cmd: &str) -> bool {
    if Path::new(cmd).is_file() {
        return true;
    }
    // `where` on Windows is more reliable than --version for some shims
    #[cfg(windows)]
    {
        if Command::new("where")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }
    }
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
        || Command::new(cmd)
            .arg("--help")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
}

pub fn adapter_probe() -> Value {
    let (cmd, args, kind) = resolve_dap_adapter();
    let available = resolve_executable(&cmd).is_some() || which_ok(&cmd);
    json!({
        "command": cmd,
        "args": args,
        "kind": kind,
        "available": available,
        "lldb_dap": resolve_executable("lldb-dap").is_some()
            || resolve_executable("lldb-vscode").is_some()
            || which_ok("lldb-dap")
            || which_ok("lldb-vscode"),
        "gdb_dap": resolve_executable("gdb").is_some()
            || resolve_executable("gdb.exe").is_some()
            || which_ok("gdb")
            || which_ok("gdb.exe"),
        "rust_gdb": which_ok("rust-gdb") || which_ok("rust-gdb.exe"),
        "override_env": "HELIX_CODE_DAP_COMMAND",
        "capabilities_surface": [
            "initialize", "launch", "setBreakpoints", "configurationDone",
            "continue", "next", "stepIn", "stepOut", "pause",
            "threads", "stackTrace", "scopes", "variables", "evaluate", "disconnect"
        ]
    })
}

pub fn start_session(
    session_id: Uuid,
    program: Option<&Path>,
    cwd: Option<&Path>,
    args: &[String],
) -> HelixResult<Value> {
    let (cmd, cmd_args, kind) = resolve_dap_adapter();
    if resolve_executable(&cmd).is_none() && !which_ok(&cmd) {
        return Err(HelixError::unavailable(format!(
            "DAP adapter not found: {cmd} (install lldb-dap or gdb with --interpreter=dap, or set HELIX_CODE_DAP_COMMAND)"
        )));
    }

    let spawn_cmd = resolve_executable(&cmd).unwrap_or_else(|| cmd.clone());
    let spawn_path = PathBuf::from(&spawn_cmd);
    let adapter_dir = spawn_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    // lldb-dap (LLVM Windows) needs its bin dir + python311.dll on PATH.
    let mut child_cmd = Command::new(&spawn_cmd);
    child_cmd
        .args(&cmd_args)
        .current_dir(&adapter_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Prepend adapter dir and common Python 3.11 install locations for liblldb.
    {
        let mut path_prefix = vec![adapter_dir.display().to_string()];
        if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
            let h = PathBuf::from(home);
            path_prefix.push(
                h.join(r"AppData\Local\Programs\Python\Python311")
                    .display()
                    .to_string(),
            );
            path_prefix.push(h.join(r"scoop\apps\python\current").display().to_string());
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            path_prefix.push(
                PathBuf::from(local)
                    .join(r"Programs\Python\Python311")
                    .display()
                    .to_string(),
            );
        }
        path_prefix.push(r"C:\Program Files\Python311".into());
        let old = std::env::var_os("PATH").unwrap_or_default();
        let joined = format!(
            "{}{}{}",
            path_prefix.join(";"),
            if cfg!(windows) { ";" } else { ":" },
            old.to_string_lossy()
        );
        child_cmd.env("PATH", joined);
        // PYTHONHOME helps liblldb locate stdlib if needed
        for py in &path_prefix {
            let p = Path::new(py);
            if p.join("python311.dll").is_file() || p.join("python.exe").is_file() {
                child_cmd.env("PYTHONHOME", py);
                break;
            }
        }
    }

    let mut child = child_cmd
        .spawn()
        .map_err(|e| HelixError::dependency(format!("spawn DAP `{spawn_cmd}`: {e}")))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| HelixError::internal("DAP stdin missing"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| HelixError::internal("DAP stdout missing"))?;

    let (init_tx, init_rx) = mpsc::channel();

    let session = Arc::new(DapSession {
        adapter_cmd: format!("{spawn_cmd} {}", cmd_args.join(" ")),
        adapter_kind: kind.clone(),
        status: Mutex::new("starting".into()),
        child: Mutex::new(child),
        stdin: Mutex::new(stdin),
        next_id: AtomicU64::new(1),
        pending: Mutex::new(HashMap::new()),
        initialized_tx: Mutex::new(Some(init_tx)),
        last_stopped: Mutex::new(None),
        threads: Mutex::new(json!([])),
        stack: Mutex::new(json!([])),
        capabilities: Mutex::new(json!({})),
    });

    let reader = Arc::clone(&session);
    thread::Builder::new()
        .name(format!("dap-reader-{}", session_id.simple()))
        .spawn(move || reader_loop(reader, stdout))
        .map_err(|e| HelixError::internal(format!("dap reader: {e}")))?;

    // Give the adapter a moment to start (esp. on Windows).
    thread::sleep(Duration::from_millis(120));

    // initialize
    let init = session.request_sync(
        "initialize",
        json!({
            "clientID": "helix-code",
            "clientName": "HelixCode",
            "adapterID": kind,
            "pathFormat": "path",
            "linesStartAt1": true,
            "columnsStartAt1": true,
            "supportsVariableType": true,
            "supportsVariablePaging": false,
            "supportsRunInTerminalRequest": false,
            "locale": "en-US"
        }),
        20,
    )?;
    *session.capabilities.lock() = init.clone();

    // Adapter sends `initialized` event; wait briefly (non-fatal if late/missing).
    let _ = init_rx.recv_timeout(Duration::from_secs(5));

    // launch if program provided
    let mut launch_result = json!(null);
    if let Some(prog) = program {
        let mut launch = json!({
            "name": "HelixCode launch",
            "type": kind,
            "request": "launch",
            "program": prog.display().to_string(),
            "stopOnEntry": true,
            "cwd": cwd.map(|c| c.display().to_string()).unwrap_or_default(),
        });
        // gdb-dap / lldb-dap common keys
        if kind == "gdb" {
            launch["MIMode"] = json!("gdb");
            launch["miDebuggerPath"] = json!(spawn_cmd);
        }
        if !args.is_empty() {
            launch["args"] = json!(args);
        }
        match session.request_sync("launch", launch, 45) {
            Ok(v) => launch_result = v,
            Err(e) => {
                // Some adapters accept launch only after configurationDone — try reverse order.
                tracing::warn!(error = %e, "DAP launch first attempt failed; trying configurationDone then launch");
                let _ = session.request_sync("configurationDone", json!({}), 10);
                launch_result = session.request_sync(
                    "launch",
                    json!({
                        "name": "HelixCode launch",
                        "type": kind,
                        "request": "launch",
                        "program": prog.display().to_string(),
                        "stopOnEntry": true,
                        "cwd": cwd.map(|c| c.display().to_string()).unwrap_or_default(),
                        "args": args,
                    }),
                    45,
                )?;
            }
        }
        let _ = session.request_sync("configurationDone", json!({}), 10);
    } else {
        // No program: leave adapter initialized for later attach/launch via evaluate path.
        let _ = session.request_sync("configurationDone", json!({}), 5);
    }

    *session.status.lock() = "ready".into();
    DAP.lock().insert(session_id, Arc::clone(&session));

    Ok(json!({
        "session_id": session_id,
        "adapter": session.adapter_cmd,
        "kind": kind,
        "status": "ready",
        "initialize": init,
        "launch": launch_result,
        "protocol": "dap",
        "capabilities_surface": adapter_probe()["capabilities_surface"].clone(),
    }))
}

pub fn get(session_id: Uuid) -> HelixResult<Arc<DapSession>> {
    DAP.lock()
        .get(&session_id)
        .cloned()
        .ok_or_else(|| HelixError::not_found("dap session not found (process-local)"))
}

pub fn set_breakpoints(session_id: Uuid, source_path: &str, lines: &[u32]) -> HelixResult<Value> {
    let s = get(session_id)?;
    let bps: Vec<Value> = lines.iter().map(|l| json!({ "line": l })).collect();
    s.request_sync(
        "setBreakpoints",
        json!({
            "source": { "path": source_path },
            "breakpoints": bps,
            "sourceModified": false
        }),
        15,
    )
}

fn with_thread(session_id: Uuid, thread_id: u64, command: &str) -> HelixResult<Value> {
    let s = get(session_id)?;
    *s.status.lock() = match command {
        "continue" => "running".into(),
        "pause" => "paused".into(),
        _ => s.status.lock().clone(),
    };
    s.request_sync(command, json!({ "threadId": thread_id }), 15)
}

pub fn continue_exec(session_id: Uuid) -> HelixResult<Value> {
    with_thread(session_id, 1, "continue")
}

#[allow(dead_code)]
pub fn continue_thread(session_id: Uuid, thread_id: u64) -> HelixResult<Value> {
    with_thread(session_id, thread_id, "continue")
}

pub fn next(session_id: Uuid, thread_id: u64) -> HelixResult<Value> {
    with_thread(session_id, thread_id, "next")
}

pub fn step_in(session_id: Uuid, thread_id: u64) -> HelixResult<Value> {
    with_thread(session_id, thread_id, "stepIn")
}

pub fn step_out(session_id: Uuid, thread_id: u64) -> HelixResult<Value> {
    with_thread(session_id, thread_id, "stepOut")
}

pub fn pause(session_id: Uuid, thread_id: u64) -> HelixResult<Value> {
    with_thread(session_id, thread_id, "pause")
}

pub fn stack_trace(session_id: Uuid, thread_id: u64) -> HelixResult<Value> {
    let s = get(session_id)?;
    let st = s.request_sync(
        "stackTrace",
        json!({ "threadId": thread_id, "startFrame": 0, "levels": 40 }),
        10,
    )?;
    *s.stack.lock() = st.clone();
    Ok(st)
}

pub fn threads(session_id: Uuid) -> HelixResult<Value> {
    let s = get(session_id)?;
    let t = s.request_sync("threads", json!({}), 10)?;
    *s.threads.lock() = t.clone();
    Ok(t)
}

pub fn scopes(session_id: Uuid, frame_id: u64) -> HelixResult<Value> {
    let s = get(session_id)?;
    s.request_sync("scopes", json!({ "frameId": frame_id }), 10)
}

pub fn variables(session_id: Uuid, variables_reference: u64) -> HelixResult<Value> {
    let s = get(session_id)?;
    s.request_sync(
        "variables",
        json!({ "variablesReference": variables_reference }),
        10,
    )
}

pub fn evaluate(
    session_id: Uuid,
    expression: &str,
    frame_id: Option<u64>,
    context: &str,
) -> HelixResult<Value> {
    let s = get(session_id)?;
    let mut args = json!({
        "expression": expression,
        "context": context,
    });
    if let Some(fid) = frame_id {
        args["frameId"] = json!(fid);
    }
    s.request_sync("evaluate", args, 15)
}

pub fn disconnect(session_id: Uuid) -> HelixResult<()> {
    if let Some(s) = DAP.lock().remove(&session_id) {
        *s.status.lock() = "stopped".into();
        let _ = s.request_sync("disconnect", json!({ "terminateDebuggee": true }), 5);
        if let Some(mut c) = s.child.try_lock() {
            let _ = c.kill();
        }
    }
    Ok(())
}

pub fn snapshot(session_id: Uuid) -> HelixResult<Value> {
    let s = get(session_id)?;
    Ok(json!({
        "session_id": session_id,
        "adapter": s.adapter_cmd,
        "kind": s.adapter_kind,
        "status": s.status.lock().clone(),
        "last_stopped": s.last_stopped.lock().clone(),
        "threads": s.threads.lock().clone(),
        "stack": s.stack.lock().clone(),
        "capabilities": s.capabilities.lock().clone(),
    }))
}

impl DapSession {
    fn request_sync(
        &self,
        command: &str,
        arguments: Value,
        timeout_secs: u64,
    ) -> HelixResult<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx): (Sender<Result<Value, String>>, Receiver<_>) = mpsc::channel();
        self.pending.lock().insert(id, Pending { tx });
        let msg = json!({
            "seq": id,
            "type": "request",
            "command": command,
            "arguments": arguments
        });
        write_message(&self.stdin, &msg)?;
        match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => Err(HelixError::dependency(format!("DAP {command}: {e}"))),
            Err(_) => {
                self.pending.lock().remove(&id);
                Err(HelixError::dependency(format!(
                    "DAP {command} timed out after {timeout_secs}s"
                )))
            }
        }
    }
}

fn write_message(stdin: &Mutex<ChildStdin>, msg: &Value) -> HelixResult<()> {
    let body =
        serde_json::to_vec(msg).map_err(|e| HelixError::internal(format!("DAP serialize: {e}")))?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let mut g = stdin.lock();
    g.write_all(header.as_bytes())
        .and_then(|_| g.write_all(&body))
        .and_then(|_| g.flush())
        .map_err(|e| HelixError::dependency(format!("DAP write: {e}")))?;
    Ok(())
}

fn reader_loop(session: Arc<DapSession>, stdout: impl Read + Send + 'static) {
    let mut reader = BufReader::new(stdout);
    loop {
        let mut content_len: Option<usize> = None;
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => return,
                Ok(_) => {}
                Err(_) => return,
            }
            let t = line.trim_end();
            if t.is_empty() {
                break;
            }
            if let Some(rest) = t.strip_prefix("Content-Length:") {
                content_len = rest.trim().parse().ok();
            }
        }
        let Some(len) = content_len else {
            continue;
        };
        let mut buf = vec![0u8; len];
        if reader.read_exact(&mut buf).is_err() {
            return;
        }
        let Ok(msg) = serde_json::from_slice::<Value>(&buf) else {
            continue;
        };
        let ty = msg.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match ty {
            "response" => {
                let id = msg.get("request_seq").and_then(|i| i.as_u64()).unwrap_or(0);
                let success = msg
                    .get("success")
                    .and_then(|s| s.as_bool())
                    .unwrap_or(false);
                let body = msg.get("body").cloned().unwrap_or(json!({}));
                if let Some(p) = session.pending.lock().remove(&id) {
                    if success {
                        let _ = p.tx.send(Ok(body));
                    } else {
                        let m = msg
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("DAP error")
                            .to_string();
                        let _ = p.tx.send(Err(m));
                    }
                }
            }
            "event" => {
                let event = msg.get("event").and_then(|e| e.as_str()).unwrap_or("");
                let body = msg.get("body").cloned().unwrap_or(json!({}));
                match event {
                    "initialized" => {
                        if let Some(tx) = session.initialized_tx.lock().take() {
                            let _ = tx.send(());
                        }
                    }
                    "stopped" => {
                        // Do not request_sync from the reader thread (deadlock).
                        *session.last_stopped.lock() = Some(body);
                        *session.status.lock() = "stopped".into();
                    }
                    "terminated" | "exited" => {
                        *session.status.lock() = "exited".into();
                    }
                    "output" => {
                        tracing::debug!(?body, "DAP output");
                    }
                    "thread" => {
                        tracing::debug!(?body, "DAP thread event");
                    }
                    _ => {}
                }
            }
            "request" => {
                // Reverse requests from adapter (e.g. runInTerminal) — decline safely.
                let command = msg.get("command").and_then(|c| c.as_str()).unwrap_or("");
                let seq = msg.get("seq").and_then(|s| s.as_u64()).unwrap_or(0);
                let resp = json!({
                    "seq": session.next_id.fetch_add(1, Ordering::SeqCst),
                    "type": "response",
                    "request_seq": seq,
                    "success": false,
                    "command": command,
                    "message": "reverse request not supported by HelixCode DAP client"
                });
                let _ = write_message(&session.stdin, &resp);
            }
            _ => {}
        }
    }
}
