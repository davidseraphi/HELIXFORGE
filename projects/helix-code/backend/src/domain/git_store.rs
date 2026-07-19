//! Git object plane for HelixCode extreme (E0/E1).
//!
//! **Dual plane:**
//! 1. **Object / metadata plane** — bare repos on disk; reads via **gitoxide (`gix`)**,
//!    with system-`git` fallback if gix cannot open/peel. Writes (init/commit) still use
//!    system `git` porcelain (reliable worktree path until pure-gix commit lands).
//! 2. **Smart HTTP plane** — `git upload-pack` / `receive-pack` `--stateless-rpc`.
//!
//! Layout: `{HELIX_CODE_REPO_ROOT}/{tenant_uuid}/{repo_name}.git`

use gix::bstr::ByteSlice;
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use std::path::{Path, PathBuf};
use std::process::Command;
// serde_json via workspace dependency graph

#[derive(Debug, Clone)]
pub struct GitStore {
    root: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RefInfo {
    pub name: String,
    pub target_sha: String,
    pub is_symbolic: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TreeEntry {
    pub path: String,
    pub kind: String,
    pub mode: String,
    pub oid: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CommitInfo {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub time: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchHit {
    pub path: String,
    pub line: u32,
    pub preview: String,
}

impl GitStore {
    pub fn from_env() -> Self {
        let root = std::env::var("HELIX_CODE_REPO_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(".data/helix-code/repos"));
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn backend_id(&self) -> &'static str {
        "gix+cli"
    }

    /// Whether gix can open a path (object plane preferred).
    pub fn gix_open_ok(path: &Path) -> bool {
        gix::open(path).is_ok()
    }

    pub fn repo_path(&self, tenant_id: TenantId, name: &str) -> PathBuf {
        self.root
            .join(tenant_id.as_uuid().to_string())
            .join(format!("{name}.git"))
    }

    /// Create bare repo, seed README on main, return HEAD sha.
    pub fn init_bare_with_seed(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
    ) -> HelixResult<(PathBuf, String)> {
        let path = self.repo_path(tenant_id, name);
        if path.exists() {
            return Err(HelixError::conflict(format!(
                "git storage already exists: {}",
                path.display()
            )));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| HelixError::internal(format!("create repo parent: {e}")))?;
        }

        let bare_s = path_str(&path)?;
        run_git(&["init", "--bare", bare_s])?;

        let work =
            tempfile::tempdir().map_err(|e| HelixError::internal(format!("tempdir: {e}")))?;
        let work_s = path_str(work.path())?;
        run_git(&["clone", bare_s, work_s])?;
        let readme = format!("# {name}\n\n{description}\n\nForged on HelixCode extreme (E0).\n");
        std::fs::write(work.path().join("README.md"), readme)
            .map_err(|e| HelixError::internal(format!("write README: {e}")))?;
        run_git_in(
            work.path(),
            &["config", "user.email", "forge@helixforge.local"],
        )?;
        run_git_in(work.path(), &["config", "user.name", "HelixCode Forge"])?;
        run_git_in(work.path(), &["add", "README.md"])?;
        run_git_in(work.path(), &["commit", "-m", "chore: initial forge seed"])?;
        let _ = run_git_in(work.path(), &["branch", "-M", "main"]);
        run_git_in(work.path(), &["push", "-u", "origin", "HEAD:main"])?;
        let _ = run_git(&["-C", bare_s, "symbolic-ref", "HEAD", "refs/heads/main"]);

        let head = head_sha(&path)?;
        // Prove gix opens the new bare repo
        let _ = open_gix(&path)?;
        Ok((path, head))
    }

    pub fn list_refs(&self, tenant_id: TenantId, name: &str) -> HelixResult<Vec<RefInfo>> {
        let path = self.repo_path(tenant_id, name);
        ensure_repo(&path)?;
        match list_refs_gix(&path) {
            Ok(r) if !r.is_empty() => Ok(r),
            Ok(_) | Err(_) => list_refs_cli(&path),
        }
    }

    pub fn head_sha(&self, tenant_id: TenantId, name: &str) -> HelixResult<String> {
        head_sha(&self.repo_path(tenant_id, name))
    }

    /// Resolve a ref or SHA to a full object id.
    pub fn rev_parse(&self, tenant_id: TenantId, name: &str, rev: &str) -> HelixResult<String> {
        let path = self.repo_path(tenant_id, name);
        ensure_repo(&path)?;
        let s = git_stdout(&path, &["rev-parse", rev])?;
        let s = s.trim().to_string();
        if s.len() < 7 {
            return Err(HelixError::not_found(format!("rev not found: {rev}")));
        }
        Ok(s)
    }

    pub fn list_tree(
        &self,
        tenant_id: TenantId,
        name: &str,
        rev: &str,
        path: &str,
    ) -> HelixResult<Vec<TreeEntry>> {
        let repo = self.repo_path(tenant_id, name);
        ensure_repo(&repo)?;
        match list_tree_gix(&repo, rev, path) {
            Ok(e) => Ok(e),
            Err(e) => list_tree_cli(&repo, rev, path).map_err(|cli| {
                HelixError::dependency(format!("gix tree failed ({e}); cli also failed: {cli}"))
            }),
        }
    }

    pub fn read_blob(
        &self,
        tenant_id: TenantId,
        name: &str,
        rev: &str,
        path: &str,
    ) -> HelixResult<String> {
        if path.is_empty() {
            return Err(HelixError::validation("path required"));
        }
        let repo = self.repo_path(tenant_id, name);
        ensure_repo(&repo)?;
        match read_blob_gix(&repo, rev, path) {
            Ok(s) => Ok(s),
            Err(e) => {
                let spec = format!("{rev}:{path}");
                git_stdout(&repo, &["show", &spec]).map_err(|cli| {
                    HelixError::not_found(format!("blob not found (gix: {e}; cli: {cli})"))
                })
            }
        }
    }

    /// Flat recursive file paths (Code-OSS Quick Open index).
    pub fn list_files_recursive(
        &self,
        tenant_id: TenantId,
        name: &str,
        rev: &str,
        max: usize,
    ) -> HelixResult<Vec<String>> {
        let repo = self.repo_path(tenant_id, name);
        ensure_repo(&repo)?;
        let max = max.clamp(1, 10_000);
        // Prefer git ls-tree -r --name-only (fast on bare)
        let out = git_stdout(&repo, &["ls-tree", "-r", "--name-only", rev]).or_else(|_| {
            git_stdout(
                &repo,
                &["ls-tree", "-r", "--name-only", &format!("{rev}^{{tree}}")],
            )
        })?;
        let mut files: Vec<String> = out
            .lines()
            .map(|l| l.trim().replace('\\', "/"))
            .filter(|l| !l.is_empty() && !l.contains(".."))
            .take(max)
            .collect();
        files.sort();
        Ok(files)
    }

    /// Content search across UTF-8 blobs (Code-OSS Search view). Bounded for forge safety.
    pub fn search_content(
        &self,
        tenant_id: TenantId,
        name: &str,
        rev: &str,
        query: &str,
        max_hits: usize,
        max_files: usize,
    ) -> HelixResult<Vec<SearchHit>> {
        let q = query.trim();
        if q.is_empty() {
            return Err(HelixError::validation("query required"));
        }
        if q.len() > 200 {
            return Err(HelixError::validation("query too long"));
        }
        let max_hits = max_hits.clamp(1, 200);
        let max_files = max_files.clamp(1, 500);
        let files = self.list_files_recursive(tenant_id, name, rev, max_files)?;
        let q_lower = q.to_ascii_lowercase();
        let mut hits = Vec::new();
        for path in files {
            // skip heavy / non-text by extension
            let lower = path.to_ascii_lowercase();
            if lower.ends_with(".png")
                || lower.ends_with(".jpg")
                || lower.ends_with(".jpeg")
                || lower.ends_with(".gif")
                || lower.ends_with(".webp")
                || lower.ends_with(".pdf")
                || lower.ends_with(".zip")
                || lower.ends_with(".wasm")
                || lower.ends_with(".exe")
                || lower.ends_with(".dll")
            {
                continue;
            }
            let content = match self.read_blob(tenant_id, name, rev, &path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if content.len() > 512 * 1024 {
                continue;
            }
            for (i, line) in content.lines().enumerate() {
                if line.to_ascii_lowercase().contains(&q_lower) {
                    hits.push(SearchHit {
                        path: path.clone(),
                        line: i as u32,
                        preview: line.chars().take(200).collect(),
                    });
                    if hits.len() >= max_hits {
                        return Ok(hits);
                    }
                }
            }
        }
        Ok(hits)
    }

    pub fn commit_file(
        &self,
        tenant_id: TenantId,
        name: &str,
        branch: &str,
        path: &str,
        content: &str,
        message: &str,
    ) -> HelixResult<String> {
        if path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
            return Err(HelixError::validation("invalid path"));
        }
        let bare = self.repo_path(tenant_id, name);
        ensure_repo(&bare)?;
        let bare_s = path_str(&bare)?;
        let work =
            tempfile::tempdir().map_err(|e| HelixError::internal(format!("tempdir: {e}")))?;
        let work_s = path_str(work.path())?;
        if run_git(&["clone", "--branch", branch, bare_s, work_s]).is_err() {
            run_git(&["clone", bare_s, work_s])?;
            run_git_in(work.path(), &["checkout", "-B", branch])?;
        }
        let dest = work.path().join(path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| HelixError::internal(format!("mkdir: {e}")))?;
        }
        std::fs::write(&dest, content)
            .map_err(|e| HelixError::internal(format!("write file: {e}")))?;
        run_git_in(
            work.path(),
            &["config", "user.email", "forge@helixforge.local"],
        )?;
        run_git_in(work.path(), &["config", "user.name", "HelixCode Forge"])?;
        run_git_in(work.path(), &["add", "--", path])?;
        run_git_in(work.path(), &["commit", "-m", message])?;
        run_git_push(work.path(), &["push", "origin", &format!("HEAD:{branch}")])?;
        head_sha(&bare)
    }

    /// Multi-file commit in one worktree (Code-OSS Save All → single commit).
    pub fn commit_files(
        &self,
        tenant_id: TenantId,
        name: &str,
        branch: &str,
        files: &[(String, String)],
        message: &str,
    ) -> HelixResult<String> {
        if files.is_empty() {
            return Err(HelixError::validation("files required"));
        }
        if files.len() > 50 {
            return Err(HelixError::validation("max 50 files per batch commit"));
        }
        for (path, _) in files {
            if path.contains("..")
                || path.starts_with('/')
                || path.starts_with('\\')
                || path.is_empty()
            {
                return Err(HelixError::validation(format!("invalid path: {path}")));
            }
        }
        let bare = self.repo_path(tenant_id, name);
        ensure_repo(&bare)?;
        let bare_s = path_str(&bare)?;
        let work =
            tempfile::tempdir().map_err(|e| HelixError::internal(format!("tempdir: {e}")))?;
        let work_s = path_str(work.path())?;
        if run_git(&["clone", "--branch", branch, bare_s, work_s]).is_err() {
            run_git(&["clone", bare_s, work_s])?;
            run_git_in(work.path(), &["checkout", "-B", branch])?;
        }
        for (path, content) in files {
            let dest = work.path().join(path);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| HelixError::internal(format!("mkdir: {e}")))?;
            }
            std::fs::write(&dest, content)
                .map_err(|e| HelixError::internal(format!("write {path}: {e}")))?;
            run_git_in(work.path(), &["add", "--", path])?;
        }
        run_git_in(
            work.path(),
            &["config", "user.email", "forge@helixforge.local"],
        )?;
        run_git_in(work.path(), &["config", "user.name", "HelixCode Forge"])?;
        run_git_in(work.path(), &["commit", "-m", message])?;
        run_git_push(work.path(), &["push", "origin", &format!("HEAD:{branch}")])?;
        head_sha(&bare)
    }

    pub fn log(
        &self,
        tenant_id: TenantId,
        name: &str,
        rev: &str,
        limit: usize,
    ) -> HelixResult<Vec<CommitInfo>> {
        let repo = self.repo_path(tenant_id, name);
        ensure_repo(&repo)?;
        let limit = limit.clamp(1, 100);
        match log_gix(&repo, rev, limit) {
            Ok(c) if !c.is_empty() => Ok(c),
            Ok(_) | Err(_) => log_cli(&repo, rev, limit),
        }
    }

    pub fn path_for_smart_http(&self, tenant_id: TenantId, name: &str) -> HelixResult<PathBuf> {
        let p = self.repo_path(tenant_id, name);
        ensure_repo(&p)?;
        Ok(p)
    }

    /// Create branch from existing ref (e.g. main → feature).
    pub fn create_branch(
        &self,
        tenant_id: TenantId,
        name: &str,
        branch: &str,
        from: &str,
    ) -> HelixResult<String> {
        let bare = self.repo_path(tenant_id, name);
        ensure_repo(&bare)?;
        let bare_s = path_str(&bare)?;
        let work =
            tempfile::tempdir().map_err(|e| HelixError::internal(format!("tempdir: {e}")))?;
        let work_s = path_str(work.path())?;
        run_git(&["clone", bare_s, work_s])?;
        run_git_in(work.path(), &["checkout", from])?;
        run_git_in(work.path(), &["checkout", "-B", branch])?;
        run_git_push(work.path(), &["push", "-u", "origin", branch])?;
        let sha = rev_parse_work(work.path())?;
        Ok(sha)
    }

    /// Merge source into target (merge commit) and push bare.
    pub fn merge_branch(
        &self,
        tenant_id: TenantId,
        name: &str,
        source: &str,
        target: &str,
        message: &str,
    ) -> HelixResult<String> {
        let bare = self.repo_path(tenant_id, name);
        ensure_repo(&bare)?;
        let bare_s = path_str(&bare)?;
        let work =
            tempfile::tempdir().map_err(|e| HelixError::internal(format!("tempdir: {e}")))?;
        let work_s = path_str(work.path())?;
        run_git(&["clone", bare_s, work_s])?;
        run_git_in(work.path(), &["checkout", target])
            .or_else(|_| run_git_in(work.path(), &["checkout", "-B", target]))?;
        run_git_in(
            work.path(),
            &["config", "user.email", "forge@helixforge.local"],
        )?;
        run_git_in(work.path(), &["config", "user.name", "HelixCode Forge"])?;
        // fetch all branches
        let _ = run_git_in(work.path(), &["fetch", "origin", source]);
        let merge_ref = format!("origin/{source}");
        if run_git_in(
            work.path(),
            &["merge", "--no-ff", "-m", message, &merge_ref],
        )
        .is_err()
        {
            run_git_in(work.path(), &["merge", "--no-ff", "-m", message, source])?;
        }
        run_git_push(work.path(), &["push", "origin", &format!("HEAD:{target}")])?;
        rev_parse_work(work.path())
    }

    /// Summary of refs + head for SCM UI.
    pub fn status_summary(
        &self,
        tenant_id: TenantId,
        name: &str,
    ) -> HelixResult<serde_json::Value> {
        let refs = self.list_refs(tenant_id, name)?;
        let head = self.head_sha(tenant_id, name).ok();
        Ok(serde_json::json!({
            "head": head,
            "refs": refs,
            "dirty_workspace": false,
            "note": "forge bare status — editor dirty state is client-side"
        }))
    }

    pub fn diff_path(
        &self,
        tenant_id: TenantId,
        name: &str,
        rev: &str,
        path: &str,
    ) -> HelixResult<String> {
        let bare = self.repo_path(tenant_id, name);
        ensure_repo(&bare)?;
        if path.is_empty() {
            return git_stdout(&bare, &["show", rev, "--stat"]);
        }
        // show file at rev with parent diff if possible
        let out = git_stdout(&bare, &["log", "-1", "-p", rev, "--", path])
            .or_else(|_| git_stdout(&bare, &["show", &format!("{rev}:{path}")]))?;
        Ok(out)
    }
}

fn rev_parse_work(dir: &Path) -> HelixResult<String> {
    let s = git_stdout(dir, &["rev-parse", "HEAD"])?;
    Ok(s.trim().to_string())
}

fn open_gix(path: &Path) -> HelixResult<gix::Repository> {
    gix::open(path).map_err(|e| HelixError::dependency(format!("gix open {}: {e}", path.display())))
}

fn list_refs_gix(path: &Path) -> HelixResult<Vec<RefInfo>> {
    let repo = open_gix(path)?;
    let platform = repo
        .references()
        .map_err(|e| HelixError::dependency(format!("gix references: {e}")))?;
    let all = platform
        .all()
        .map_err(|e| HelixError::dependency(format!("gix refs all: {e}")))?;
    let mut refs = Vec::new();
    for r in all {
        let r = r.map_err(|e| HelixError::dependency(format!("gix ref: {e}")))?;
        let name = r.name().as_bstr().to_str_lossy().into_owned();
        let target_sha = r.id().to_string();
        let is_symbolic = matches!(r.target(), gix::refs::TargetRef::Symbolic(_));
        refs.push(RefInfo {
            name,
            target_sha,
            is_symbolic,
        });
    }
    if let Ok(head) = repo.head() {
        if let Some(name) = head.referent_name() {
            let target = repo
                .head_id()
                .map(|id| id.to_string())
                .unwrap_or_else(|_| name.as_bstr().to_str_lossy().into_owned());
            refs.push(RefInfo {
                name: "HEAD".into(),
                target_sha: target,
                is_symbolic: true,
            });
            let _ = name;
        }
    }
    refs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(refs)
}

fn list_refs_cli(path: &Path) -> HelixResult<Vec<RefInfo>> {
    let out = git_stdout(path, &["show-ref", "--head"])?;
    let mut refs = Vec::new();
    for line in out.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(sha) = parts.next() else { continue };
        let Some(refname) = parts.next() else {
            continue;
        };
        refs.push(RefInfo {
            name: refname.into(),
            target_sha: sha.into(),
            is_symbolic: false,
        });
    }
    if let Ok(head) = git_stdout(path, &["symbolic-ref", "HEAD"]) {
        let name = head.trim().to_string();
        if !name.is_empty() {
            let target = git_stdout(path, &["rev-parse", "HEAD"])
                .unwrap_or_default()
                .trim()
                .to_string();
            refs.push(RefInfo {
                name: "HEAD".into(),
                target_sha: if target.is_empty() {
                    name.clone()
                } else {
                    target
                },
                is_symbolic: true,
            });
        }
    }
    refs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(refs)
}

fn list_tree_gix(repo_path: &Path, rev: &str, path: &str) -> HelixResult<Vec<TreeEntry>> {
    let repo = open_gix(repo_path)?;
    let spec = if path.is_empty() {
        format!("{rev}^{{tree}}")
    } else {
        format!("{rev}:{path}")
    };
    let id = repo
        .rev_parse_single(spec.as_str())
        .map_err(|e| HelixError::not_found(format!("gix rev_parse {spec}: {e}")))?;
    let object = repo
        .find_object(id)
        .map_err(|e| HelixError::dependency(format!("gix find_object: {e}")))?;
    let tree = object
        .try_into_tree()
        .map_err(|e| HelixError::validation(format!("not a tree: {e}")))?;
    let mut entries = Vec::new();
    for entry in tree.iter() {
        let entry = entry.map_err(|e| HelixError::dependency(format!("gix tree entry: {e}")))?;
        let filename = entry.filename().to_str_lossy().into_owned();
        let mode = format!("{:?}", entry.mode());
        let kind = if entry.mode().is_tree() {
            "tree"
        } else if entry.mode().is_blob() || entry.mode().is_executable() {
            "blob"
        } else if entry.mode().is_link() {
            "link"
        } else {
            "other"
        };
        let rel = if path.is_empty() {
            filename
        } else {
            format!("{path}/{filename}")
        };
        entries.push(TreeEntry {
            path: rel,
            kind: kind.into(),
            mode,
            oid: entry.oid().to_string(),
        });
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

fn list_tree_cli(repo: &Path, rev: &str, path: &str) -> HelixResult<Vec<TreeEntry>> {
    let spec = if path.is_empty() {
        rev.to_string()
    } else {
        format!("{rev}:{path}")
    };
    let out = git_stdout(repo, &["ls-tree", "-z", &spec])
        .or_else(|_| git_stdout(repo, &["ls-tree", &spec]))?;
    let mut entries = Vec::new();
    if out.contains('\0') {
        for chunk in out.split('\0') {
            if chunk.is_empty() {
                continue;
            }
            if let Some(e) = parse_ls_tree_line(chunk, path) {
                entries.push(e);
            }
        }
    } else {
        for line in out.lines() {
            if let Some(e) = parse_ls_tree_line(line, path) {
                entries.push(e);
            }
        }
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

fn read_blob_gix(repo_path: &Path, rev: &str, path: &str) -> HelixResult<String> {
    let repo = open_gix(repo_path)?;
    let spec = format!("{rev}:{path}");
    let id = repo
        .rev_parse_single(spec.as_str())
        .map_err(|e| HelixError::not_found(format!("gix blob {spec}: {e}")))?;
    let object = repo
        .find_object(id)
        .map_err(|e| HelixError::dependency(format!("gix find blob: {e}")))?;
    String::from_utf8(object.data.clone()).map_err(|_| {
        HelixError::validation("blob is not valid UTF-8 (binary files: raw API later)")
    })
}

fn log_gix(repo_path: &Path, rev: &str, limit: usize) -> HelixResult<Vec<CommitInfo>> {
    let repo = open_gix(repo_path)?;
    let id = repo
        .rev_parse_single(rev)
        .map_err(|e| HelixError::not_found(format!("gix rev: {e}")))?;
    let walk = repo
        .rev_walk([id])
        .all()
        .map_err(|e| HelixError::dependency(format!("gix rev_walk: {e}")))?;
    let mut out = Vec::new();
    for item in walk.take(limit) {
        let info = item.map_err(|e| HelixError::dependency(format!("gix walk: {e}")))?;
        // In gix 0.85, rev_walk Info::object() yields a Commit.
        let commit = info
            .object()
            .map_err(|e| HelixError::dependency(format!("gix commit obj: {e}")))?;
        let message = commit
            .message_raw()
            .map(|m| {
                let s = m.to_str().unwrap_or("(binary message)");
                s.lines().next().unwrap_or(s).to_string()
            })
            .unwrap_or_else(|_| "(no message)".into());
        let author = commit
            .author()
            .map(|a| a.name.to_str().unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "unknown".into());
        let time = commit
            .author()
            .map(|a| a.time.to_string())
            .unwrap_or_default();
        out.push(CommitInfo {
            sha: info.id().to_string(),
            message,
            author,
            time,
        });
    }
    Ok(out)
}

fn log_cli(repo: &Path, rev: &str, limit: usize) -> HelixResult<Vec<CommitInfo>> {
    let n = limit.to_string();
    let out = git_stdout(
        repo,
        &["log", rev, "-n", &n, "--format=%H%x1f%an%x1f%aI%x1f%s"],
    )?;
    let mut commits = Vec::new();
    for line in out.lines() {
        let mut parts = line.split('\x1f');
        let Some(sha) = parts.next() else { continue };
        let author = parts.next().unwrap_or("").to_string();
        let time = parts.next().unwrap_or("").to_string();
        let message = parts.next().unwrap_or("").to_string();
        commits.push(CommitInfo {
            sha: sha.into(),
            message,
            author,
            time,
        });
    }
    Ok(commits)
}

fn parse_ls_tree_line(line: &str, prefix: &str) -> Option<TreeEntry> {
    // mode SP type SP object TAB name
    let (meta, name) = line.split_once('\t')?;
    let mut parts = meta.split_whitespace();
    let mode = parts.next()?.to_string();
    let kind = parts.next()?.to_string();
    let oid = parts.next()?.to_string();
    let filename = name.to_string();
    let path = if prefix.is_empty() {
        filename
    } else {
        format!("{prefix}/{filename}")
    };
    Some(TreeEntry {
        path,
        kind,
        mode,
        oid,
    })
}

fn ensure_repo(path: &Path) -> HelixResult<()> {
    if !path.exists() {
        return Err(HelixError::not_found(format!(
            "git repo missing on disk: {}",
            path.display()
        )));
    }
    Ok(())
}

fn head_sha(path: &Path) -> HelixResult<String> {
    ensure_repo(path)?;
    if let Ok(repo) = open_gix(path) {
        if let Ok(id) = repo.head_id() {
            return Ok(id.to_string());
        }
    }
    let s = git_stdout(path, &["rev-parse", "HEAD"])?;
    let s = s.trim().to_string();
    if s.len() < 7 {
        return Err(HelixError::dependency("empty HEAD"));
    }
    Ok(s)
}

fn path_str(p: &Path) -> HelixResult<&str> {
    p.to_str()
        .ok_or_else(|| HelixError::internal("path not utf8"))
}

fn run_git(args: &[&str]) -> HelixResult<()> {
    let out = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| HelixError::dependency(format!("git spawn: {e}")))?;
    if !out.status.success() {
        return Err(HelixError::dependency(format!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(())
}

fn run_git_in(dir: &Path, args: &[&str]) -> HelixResult<()> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|e| HelixError::dependency(format!("git spawn: {e}")))?;
    if !out.status.success() {
        return Err(HelixError::dependency(format!(
            "git {:?} in {} failed: {}",
            args,
            dir.display(),
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(())
}

/// Run `git push` and map the fast-forward / stale-ref rejections git
/// already enforces into a clean conflict: the ref moved between our
/// clone and our push, which is compare-and-swap at the git layer —
/// the loser is told to retry instead of seeing a raw 500.
fn run_git_push(dir: &Path, args: &[&str]) -> HelixResult<()> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|e| HelixError::dependency(format!("git spawn: {e}")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        if stderr.contains("non-fast-forward")
            || stderr.contains("fetch first")
            || stderr.contains("stale info")
            || stderr.contains("failed to push some refs")
        {
            return Err(HelixError::conflict(
                "ref moved concurrently; fetch and retry",
            ));
        }
        return Err(HelixError::dependency(format!(
            "git {:?} in {} failed: {}",
            args,
            dir.display(),
            stderr
        )));
    }
    Ok(())
}

fn git_stdout(repo: &Path, args: &[&str]) -> HelixResult<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo).args(args);
    let out = cmd
        .output()
        .map_err(|e| HelixError::dependency(format!("git spawn: {e}")))?;
    if !out.status.success() {
        return Err(HelixError::dependency(format!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Map HTTP service name (`git-upload-pack`) → git subcommand (`upload-pack`).
fn pack_subcommand(service: &str) -> HelixResult<&'static str> {
    match service {
        "git-upload-pack" | "upload-pack" => Ok("upload-pack"),
        "git-receive-pack" | "receive-pack" => Ok("receive-pack"),
        _ => Err(HelixError::validation(format!(
            "unknown pack service: {service}"
        ))),
    }
}

/// Run git pack server for smart HTTP (stateless-rpc).
pub fn git_pack_advertise(repo: &Path, service: &str) -> HelixResult<Vec<u8>> {
    let sub = pack_subcommand(service)?;
    let out = Command::new("git")
        .arg(sub)
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(repo)
        .output()
        .map_err(|e| HelixError::dependency(format!("git {sub}: {e}")))?;
    if !out.status.success() {
        return Err(HelixError::dependency(format!(
            "git {sub} advertise: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(out.stdout)
}

pub fn git_pack_rpc(repo: &Path, service: &str, body: &[u8]) -> HelixResult<Vec<u8>> {
    use std::io::Write;
    use std::process::Stdio;
    let sub = pack_subcommand(service)?;
    let mut child = Command::new("git")
        .arg(sub)
        .arg("--stateless-rpc")
        .arg(repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| HelixError::dependency(format!("git {sub} spawn: {e}")))?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| HelixError::internal("no stdin"))?;
        stdin
            .write_all(body)
            .map_err(|e| HelixError::internal(format!("write stdin: {e}")))?;
    }
    let out = child
        .wait_with_output()
        .map_err(|e| HelixError::dependency(format!("git {sub} wait: {e}")))?;
    if !out.status.success() {
        return Err(HelixError::dependency(format!(
            "git {sub} rpc: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(out.stdout)
}

pub fn pkt_service_header(service: &str) -> Vec<u8> {
    let payload = format!("# service={service}\n");
    let mut out = pkt_line(payload.as_bytes());
    out.extend_from_slice(b"0000");
    out
}

fn pkt_line(data: &[u8]) -> Vec<u8> {
    let len = data.len() + 4;
    let mut v = format!("{len:04x}").into_bytes();
    v.extend_from_slice(data);
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn init_list_tree_commit_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = GitStore {
            root: dir.path().to_path_buf(),
        };
        let tenant = TenantId::from_uuid(Uuid::nil());
        let (path, head) = store
            .init_bare_with_seed(tenant, "demo", "test repo")
            .expect("init");
        assert!(path.exists());
        assert!(head.len() >= 7);
        let refs = store.list_refs(tenant, "demo").unwrap();
        assert!(refs.iter().any(|r| r.name.contains("main")));
        let tree = store.list_tree(tenant, "demo", "main", "").unwrap();
        assert!(tree
            .iter()
            .any(|e| e.path == "README.md" || e.path.ends_with("README.md")));
        let blob = store
            .read_blob(tenant, "demo", "main", "README.md")
            .unwrap();
        assert!(blob.contains("demo"));
        let sha = store
            .commit_file(
                tenant,
                "demo",
                "main",
                "src/hello.rs",
                "fn main() {}\n",
                "feat: add hello",
            )
            .unwrap();
        assert_ne!(sha, head);
        let blob2 = store
            .read_blob(tenant, "demo", "main", "src/hello.rs")
            .unwrap();
        assert!(blob2.contains("fn main"));
        let log = store.log(tenant, "demo", "main", 5).unwrap();
        assert!(log.len() >= 2);

        // Code-OSS: recursive index + search + batch commit
        let files = store
            .list_files_recursive(tenant, "demo", "main", 100)
            .unwrap();
        assert!(files.iter().any(|f| f.ends_with("hello.rs")));
        let hits = store
            .search_content(tenant, "demo", "main", "fn main", 20, 100)
            .unwrap();
        assert!(hits.iter().any(|h| h.path.contains("hello.rs")));
        let batch_sha = store
            .commit_files(
                tenant,
                "demo",
                "main",
                &[
                    ("src/a.rs".into(), "pub const A: u8 = 1;\n".into()),
                    ("src/b.rs".into(), "pub const B: u8 = 2;\n".into()),
                ],
                "feat: batch",
            )
            .unwrap();
        assert_ne!(batch_sha, sha);
        assert!(store
            .read_blob(tenant, "demo", "main", "src/a.rs")
            .unwrap()
            .contains("A:"));
    }

    #[test]
    fn concurrent_commit_same_branch_cas_holds() {
        let dir = tempfile::tempdir().unwrap();
        let tenant = TenantId::from_uuid(Uuid::nil());
        GitStore {
            root: dir.path().to_path_buf(),
        }
        .init_bare_with_seed(tenant, "race", "cas repo")
        .expect("init");

        // 8 racing single-file commits on main: every winner advances the
        // branch by one commit; every loser is a clean conflict from the
        // git-layer compare-and-swap (non-fast-forward), never a 500.
        let mut handles = Vec::new();
        for i in 0..8u32 {
            let root = dir.path().to_path_buf();
            handles.push(std::thread::spawn(move || {
                let store = GitStore { root };
                store.commit_file(
                    tenant,
                    "race",
                    "main",
                    &format!("race/{i}.txt"),
                    &format!("winner {i}\n"),
                    &format!("race commit {i}"),
                )
            }));
        }
        let mut winners = 0usize;
        let mut conflicts = 0usize;
        for h in handles {
            match h.join().expect("commit task panicked") {
                Ok(_) => winners += 1,
                Err(e) if e.code == shared_core::ErrorCode::Conflict => conflicts += 1,
                Err(e) => panic!("unexpected commit error: {e}"),
            }
        }
        assert!(winners >= 1, "at least one commit must win");
        assert_eq!(conflicts, 8 - winners, "every loser must conflict");

        // main holds exactly the seed plus one commit per winner.
        let store = GitStore {
            root: dir.path().to_path_buf(),
        };
        let log = store.log(tenant, "race", "main", 64).unwrap();
        assert_eq!(
            log.len(),
            1 + winners,
            "branch history is exactly the winners' commits"
        );
    }
}
