"use client";

import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type MouseEvent,
} from "react";
import {
  api,
  type AgentJob,
  type CodeRepo,
  type CommitInfo,
  type DomainStatus,
  type LspDiagnostic,
  type PipelineRun,
  type TreeEntry,
} from "@/lib/api";
import { API } from "@/lib/config";
import { CommandPalette, type PaletteCommand } from "@/components/CommandPalette";
import { QuickOpen } from "@/components/QuickOpen";
import { EditorGroupPane } from "@/components/EditorGroupPane";
import { desktopPlatform, isElectronShell } from "@/lib/desktop";
import {
  closeInGroup,
  dirtyDocs,
  emptyGroup,
  getDoc,
  isDirty,
  markAllDocsSaved,
  markDocSaved,
  moveTabBetweenGroups,
  openInGroup,
  pathStillOpen,
  removeDoc,
  updateDocContent,
  upsertDoc,
  type EditorGroup,
  type EditorTab,
  type GroupId,
} from "@/lib/tabs";

type Toast = { kind: "ok" | "error" | "info"; text: string };
type Activity =
  | "explorer"
  | "search"
  | "scm"
  | "run"
  | "agents"
  | "mls"
  | "collab"
  | "terminal"
  | "debug"
  | "ext"
  | "settings";
type BottomTab = "problems" | "output" | "history" | "terminal";

export default function CodeOssWorkspace() {
  const [domain, setDomain] = useState<DomainStatus | null>(null);
  const [repos, setRepos] = useState<CodeRepo[]>([]);
  const [repoId, setRepoId] = useState<string | null>(null);
  const [treePath, setTreePath] = useState("");
  const [entries, setEntries] = useState<TreeEntry[]>([]);
  const [fileIndex, setFileIndex] = useState<string[]>([]);
  const [docs, setDocs] = useState<EditorTab[]>([]);
  const [primary, setPrimary] = useState<EditorGroup>(() => emptyGroup("primary"));
  const [secondary, setSecondary] = useState<EditorGroup | null>(null);
  const [focusedGroup, setFocusedGroup] = useState<GroupId>("primary");
  const [splitRatio, setSplitRatio] = useState(0.5);
  const [draggingSplit, setDraggingSplit] = useState(false);
  const [message, setMessage] = useState("chore: update via HelixCode");
  const [branch, setBranch] = useState("main");
  const [commits, setCommits] = useState<CommitInfo[]>([]);
  const [newRepo, setNewRepo] = useState("");
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<Toast | null>(null);
  const [lspSessionId, setLspSessionId] = useState<string | null>(null);
  const [lspAvailable, setLspAvailable] = useState(false);
  const [diagnostics, setDiagnostics] = useState<LspDiagnostic[]>([]);
  const [hoverText, setHoverText] = useState<string | null>(null);
  const [activity, setActivity] = useState<Activity>("explorer");
  const [bottomTab, setBottomTab] = useState<BottomTab>("problems");
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [quickOpen, setQuickOpen] = useState(false);
  const [searchQ, setSearchQ] = useState("");
  const [searchHits, setSearchHits] = useState<
    { path: string; line: number; preview: string }[]
  >([]);
  const [lastRun, setLastRun] = useState<PipelineRun | null>(null);
  const [issues, setIssues] = useState<unknown[]>([]);
  const [pulls, setPulls] = useState<unknown[]>([]);
  const [termId, setTermId] = useState<string | null>(null);
  const [termLog, setTermLog] = useState("");
  const [termCmd, setTermCmd] = useState("echo helix-term");
  const [settingsJson, setSettingsJson] = useState("{}");
  const [extensions, setExtensions] = useState<unknown[]>([]);
  const [debugInfo, setDebugInfo] = useState("");
  const [deployKeys, setDeployKeys] = useState<unknown[]>([]);
  const [quotasText, setQuotasText] = useState("");
  const [lastJob, setLastJob] = useState<AgentJob | null>(null);
  const [mlsInfo, setMlsInfo] = useState("");
  const [output, setOutput] = useState("HelixCode Code-OSS shell ready.\n");
  const [electron, setElectron] = useState(false);
  const changeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const splitRowRef = useRef<HTMLDivElement | null>(null);

  const focused =
    focusedGroup === "secondary" && secondary ? secondary : primary;
  const activePath = focused.activePath;
  const activeTab = activePath ? getDoc(docs, activePath) : undefined;
  const dirtyCount = useMemo(() => dirtyDocs(docs).length, [docs]);
  const repo = useMemo(
    () => repos.find((r) => r.id === repoId) ?? null,
    [repos, repoId],
  );
  const errCount = diagnostics.filter((d) => d.severity === 1).length;
  const warnCount = diagnostics.filter((d) => d.severity === 2).length;
  const isSplit = !!secondary;

  const flash = useCallback((kind: Toast["kind"], text: string) => {
    setToast({ kind, text });
    window.setTimeout(() => setToast(null), 4000);
  }, []);

  const logOut = useCallback((line: string) => {
    setOutput((o) => `${o}${line}\n`);
  }, []);

  const refreshRepos = useCallback(async () => {
    const items = await api.listRepos();
    setRepos(items);
    return items;
  }, []);

  const loadTree = useCallback(async (id: string, path = "", rev = "main") => {
    const data = await api.tree(id, rev, path);
    setEntries(data.entries ?? []);
    setTreePath(path);
  }, []);

  const loadLog = useCallback(async (id: string, rev = "main") => {
    const data = await api.log(id, rev, 20);
    setCommits(data.commits ?? []);
  }, []);

  const loadFileIndex = useCallback(async (id: string, rev = "main") => {
    try {
      const data = await api.listFiles(id, rev, 3000);
      setFileIndex(data.files ?? []);
    } catch {
      setFileIndex([]);
    }
  }, []);

  const ensureLsp = useCallback(async (id: string, rev: string) => {
    try {
      const st = await api.lspStatus();
      setLspAvailable(!!st.available);
      if (!st.available) {
        setLspSessionId(null);
        return null;
      }
      const sess = await api.lspOpenSession(id, rev);
      setLspSessionId(sess.session_id);
      return sess.session_id;
    } catch {
      setLspAvailable(false);
      setLspSessionId(null);
      return null;
    }
  }, []);

  useEffect(() => {
    setElectron(isElectronShell());
  }, []);

  useEffect(() => {
    (async () => {
      try {
        const [st, items] = await Promise.all([
          api.domainStatus(),
          refreshRepos(),
        ]);
        setDomain(st);
        setLspAvailable(!!st.planes?.lsp_available);
        if (items[0]) setRepoId(items[0].id);
      } catch (e) {
        flash("error", e instanceof Error ? e.message : String(e));
      }
    })();
  }, [flash, refreshRepos]);

  useEffect(() => {
    if (!repoId) return;
    (async () => {
      try {
        setDocs([]);
        setPrimary(emptyGroup("primary"));
        setSecondary(null);
        setFocusedGroup("primary");
        setTreePath("");
        setDiagnostics([]);
        setHoverText(null);
        setSearchHits([]);
        await Promise.all([
          loadTree(repoId, "", branch),
          loadLog(repoId, branch),
          loadFileIndex(repoId, branch),
        ]);
        await ensureLsp(repoId, branch);
      } catch (e) {
        flash("error", e instanceof Error ? e.message : String(e));
      }
    })();
  }, [repoId, branch, loadTree, loadLog, loadFileIndex, flash, ensureLsp]);

  // Debounced LSP for focused active tab
  useEffect(() => {
    if (!lspSessionId || !activeTab) return;
    if (changeTimer.current) clearTimeout(changeTimer.current);
    changeTimer.current = setTimeout(() => {
      api
        .lspDidChange(lspSessionId, activeTab.path, activeTab.content)
        .then((r) => setDiagnostics(r.diagnostics ?? []))
        .catch(() => {});
    }, 500);
    return () => {
      if (changeTimer.current) clearTimeout(changeTimer.current);
    };
  }, [activeTab?.content, activeTab?.path, lspSessionId]);

  function setGroup(id: GroupId, next: EditorGroup) {
    if (id === "primary") setPrimary(next);
    else setSecondary(next);
  }

  function getGroup(id: GroupId): EditorGroup {
    if (id === "secondary" && secondary) return secondary;
    return primary;
  }

  function splitEditor() {
    if (secondary) {
      setFocusedGroup("secondary");
      return;
    }
    const path = primary.activePath;
    const g = emptyGroup("secondary");
    setSecondary(path ? openInGroup(g, path) : g);
    setFocusedGroup("secondary");
    setSplitRatio(0.5);
    logOut("split editor (secondary group)");
  }

  function unsplitEditor() {
    if (!secondary) return;
    // Merge secondary tabs into primary
    let p = primary;
    for (const path of secondary.tabPaths) {
      p = openInGroup(p, path);
    }
    setPrimary(p);
    setSecondary(null);
    setFocusedGroup("primary");
    logOut("closed split");
  }

  function moveToOtherGroup(path: string) {
    if (!secondary) {
      // Create secondary with this path; keep in primary as well until explicit close (VS Code-like clone)
      const g = openInGroup(emptyGroup("secondary"), path);
      setSecondary(g);
      setFocusedGroup("secondary");
      setSplitRatio(0.5);
      logOut(`split + open ${path} in group 2`);
      return;
    }
    if (focusedGroup === "primary") {
      const { from, to } = moveTabBetweenGroups(primary, secondary, path);
      setPrimary(from);
      setSecondary(to);
      setFocusedGroup("secondary");
    } else {
      const { from, to } = moveTabBetweenGroups(secondary, primary, path);
      setSecondary(from);
      setPrimary(to);
      setFocusedGroup("primary");
    }
  }

  // Global keybindings
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const mod = e.ctrlKey || e.metaKey;
      if (mod && e.shiftKey && e.key.toLowerCase() === "p") {
        e.preventDefault();
        setPaletteOpen(true);
        setQuickOpen(false);
      } else if (mod && !e.shiftKey && e.key.toLowerCase() === "p") {
        e.preventDefault();
        setQuickOpen(true);
        setPaletteOpen(false);
      } else if (mod && e.key === "\\") {
        e.preventDefault();
        if (e.shiftKey) {
          setSecondary((s) => {
            if (!s) return s;
            setPrimary((p) => {
              let next = p;
              for (const path of s.tabPaths) next = openInGroup(next, path);
              return next;
            });
            setFocusedGroup("primary");
            return null;
          });
        } else {
          setSecondary((s) => {
            if (s) {
              setFocusedGroup("secondary");
              return s;
            }
            setPrimary((p) => {
              const path = p.activePath;
              setSecondary(
                path
                  ? openInGroup(emptyGroup("secondary"), path)
                  : emptyGroup("secondary"),
              );
              setFocusedGroup("secondary");
              setSplitRatio(0.5);
              return p;
            });
            return s;
          });
        }
      } else if (mod && e.key === "1") {
        e.preventDefault();
        setFocusedGroup("primary");
      } else if (mod && e.key === "2") {
        e.preventDefault();
        setFocusedGroup((fg) => fg);
        setSecondary((s) => {
          if (s) setFocusedGroup("secondary");
          return s;
        });
      } else if (e.key === "Escape") {
        setPaletteOpen(false);
        setQuickOpen(false);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Electron menu bridge
  useEffect(() => {
    const bridge = window.helixDesktop;
    if (!bridge?.onMenu) return;
    return bridge.onMenu((action) => {
      switch (action) {
        case "palette":
          setPaletteOpen(true);
          break;
        case "quickOpen":
          setQuickOpen(true);
          break;
        case "split":
          setSecondary((s) => {
            if (s) {
              setFocusedGroup("secondary");
              return s;
            }
            setPrimary((p) => {
              const path = p.activePath;
              const g = path
                ? openInGroup(emptyGroup("secondary"), path)
                : emptyGroup("secondary");
              // schedule setSecondary via nested is messy; use direct
              queueMicrotask(() => {
                setSecondary(g);
                setFocusedGroup("secondary");
                setSplitRatio(0.5);
              });
              return p;
            });
            return s;
          });
          break;
        case "unsplit":
          setSecondary((s) => {
            if (!s) return s;
            setPrimary((p) => {
              let next = p;
              for (const path of s.tabPaths) next = openInGroup(next, path);
              return next;
            });
            setFocusedGroup("primary");
            return null;
          });
          break;
        case "focusPrimary":
          setFocusedGroup("primary");
          break;
        case "focusSecondary":
          setSecondary((s) => {
            if (s) setFocusedGroup("secondary");
            return s;
          });
          break;
        case "about":
          flash(
            "info",
            `HelixCode Electron · ${desktopPlatform() ?? "desktop"} · ${API}`,
          );
          break;
        default:
          break;
      }
    });
  }, [flash]);

  // Split sash drag
  useEffect(() => {
    if (!draggingSplit) return;
    function onMove(e: globalThis.MouseEvent) {
      const el = splitRowRef.current;
      if (!el) return;
      const rect = el.getBoundingClientRect();
      const ratio = (e.clientX - rect.left) / rect.width;
      setSplitRatio(Math.min(0.8, Math.max(0.2, ratio)));
    }
    function onUp() {
      setDraggingSplit(false);
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
    return () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
  }, [draggingSplit]);

  async function openFileAt(path: string, targetGroup?: GroupId) {
    if (!repoId) return;
    const gid =
      targetGroup ??
      (focusedGroup === "secondary" && secondary ? "secondary" : "primary");
    const existing = getDoc(docs, path);
    if (existing) {
      setGroup(gid, openInGroup(getGroup(gid), path));
      setFocusedGroup(gid);
      if (lspSessionId) {
        try {
          const opened = await api.lspDidOpen(
            lspSessionId,
            path,
            existing.content,
          );
          setDiagnostics(opened.diagnostics ?? []);
        } catch {}
      }
      return;
    }
    setBusy(true);
    try {
      const blob = await api.blob(repoId, path, branch);
      setDocs((d) => upsertDoc(d, path, blob.content, true));
      setGroup(gid, openInGroup(getGroup(gid), path));
      setFocusedGroup(gid);
      setHoverText(null);
      let sid = lspSessionId;
      if (!sid) sid = await ensureLsp(repoId, branch);
      if (sid) {
        const opened = await api.lspDidOpen(sid, path, blob.content);
        setDiagnostics(opened.diagnostics ?? []);
      }
    } catch (e) {
      flash("error", e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onOpenEntry(entry: TreeEntry) {
    if (!repoId) return;
    if (entry.kind === "tree") {
      try {
        await loadTree(repoId, entry.path, branch);
      } catch (e) {
        flash("error", e instanceof Error ? e.message : String(e));
      }
      return;
    }
    await openFileAt(entry.path);
  }

  async function onUp() {
    if (!repoId || !treePath) return;
    const parts = treePath.split("/").filter(Boolean);
    parts.pop();
    await loadTree(repoId, parts.join("/"), branch);
  }

  async function onCreateRepo() {
    const name = newRepo.trim();
    if (!name) return;
    setBusy(true);
    try {
      const created = await api.createRepo(name, "created from Code-OSS shell");
      await api.createWorkspace(created.id, "default", "main").catch(() => null);
      setNewRepo("");
      const items = await refreshRepos();
      setRepoId(created.id ?? items.find((r) => r.name === name)?.id ?? null);
      flash("ok", `repo ${name}`);
      logOut(`created repo ${name}`);
    } catch (e) {
      flash("error", e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onCommitActive() {
    if (!repoId || !activeTab) return;
    if (!isDirty(activeTab)) {
      flash("info", "no changes in active tab");
      return;
    }
    setBusy(true);
    try {
      const res = await api.commit(repoId, {
        path: activeTab.path,
        content: activeTab.content,
        message: message.trim() || "chore: update via HelixCode",
        branch,
      });
      setDocs((d) => markDocSaved(d, activeTab.path));
      await Promise.all([
        loadLog(repoId, branch),
        refreshRepos(),
        loadFileIndex(repoId, branch),
      ]);
      flash("ok", `committed ${res.commit_sha.slice(0, 8)}`);
      logOut(`commit ${res.commit_sha.slice(0, 8)} ${activeTab.path}`);
    } catch (e) {
      flash("error", e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onCommitAll() {
    if (!repoId) return;
    const dirty = dirtyDocs(docs);
    if (!dirty.length) {
      flash("info", "no dirty docs");
      return;
    }
    setBusy(true);
    try {
      const res = await api.commitBatch(repoId, {
        files: dirty.map((t) => ({ path: t.path, content: t.content })),
        message: message.trim() || `chore: save ${dirty.length} files`,
        branch,
      });
      setDocs((d) => markAllDocsSaved(d));
      await Promise.all([
        loadLog(repoId, branch),
        refreshRepos(),
        loadFileIndex(repoId, branch),
      ]);
      flash("ok", `batch ${res.commit_sha.slice(0, 8)} (${res.count} files)`);
      logOut(`batch commit ${res.commit_sha.slice(0, 8)} files=${res.count}`);
    } catch (e) {
      flash("error", e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onSearch() {
    if (!repoId || !searchQ.trim()) return;
    setBusy(true);
    try {
      const r = await api.search(repoId, searchQ.trim(), branch, 80);
      setSearchHits(r.hits ?? []);
      setActivity("search");
      setBottomTab("output");
      logOut(`search "${searchQ}" → ${r.count} hits`);
    } catch (e) {
      flash("error", e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onRunCi() {
    if (!repoId) return;
    setBusy(true);
    setActivity("run");
    try {
      let pipes = await api.listPipelines(repoId);
      let pipeId = pipes.items?.[0]?.id;
      if (!pipeId) {
        const created = await api.createPipeline(repoId, "web-ci", {
          version: 1,
          steps: [
            { name: "hello", run: "echo helix-code-ci" },
            { name: "rev", run: "git rev-parse HEAD" },
          ],
          artifacts: ["helix-ci.log"],
        });
        pipeId = created.id;
      }
      const run = await api.triggerPipeline(pipeId, `refs/heads/${branch}`);
      setLastRun(run);
      setBottomTab("output");
      logOut(
        `CI ${run.status} exit=${run.exit_code ?? "?"} isolation=${run.isolation ?? "?"}`,
      );
      if (run.log_text) logOut(run.log_text.slice(0, 2000));
      flash(run.status === "succeeded" ? "ok" : "error", `CI ${run.status}`);
    } catch (e) {
      flash("error", e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onRunAgent() {
    if (!repoId) return;
    setBusy(true);
    setActivity("agents");
    try {
      const stamp = Date.now();
      const job = await api.createAgentJob(repoId, {
        prompt: `code-oss agent ${stamp}`,
        kind: "mesh",
        branch,
        commit: true,
        commit_message: `feat: oss agent ${stamp}`,
        patches: [
          {
            path: `src/oss_agent_${stamp}.rs`,
            content: `pub const OSS_AGENT: u64 = ${stamp};\n`,
            create: true,
          },
        ],
        agents: ["helix-code-assistant"],
      });
      setLastJob(job);
      await Promise.all([
        loadTree(repoId, treePath, branch),
        loadLog(repoId, branch),
        loadFileIndex(repoId, branch),
      ]);
      logOut(
        `agent ${job.status} iso=${job.isolation ?? "?"} ${job.result_summary ?? ""}`,
      );
      flash(job.status === "succeeded" ? "ok" : "error", `agent ${job.status}`);
    } catch (e) {
      flash("error", e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onMlsProbe() {
    setBusy(true);
    setActivity("mls");
    try {
      const st = await api.mlsStatus();
      await api.mlsIdentity("forge");
      const g = await api.mlsCreateGroup(`oss-${Date.now()}`, repoId ?? undefined);
      const text = `openmls=${st.openmls}\ngroup=${g.group_id}\nepoch=${g.epoch} members=${g.member_count}`;
      setMlsInfo(text);
      logOut(text.replace(/\n/g, " · "));
      flash("ok", "OpenMLS group created");
    } catch (e) {
      setMlsInfo(e instanceof Error ? e.message : String(e));
      flash("error", e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  function onCloseTab(groupId: GroupId, path: string, e?: MouseEvent) {
    e?.stopPropagation();
    const doc = getDoc(docs, path);
    if (doc && isDirty(doc)) {
      if (!window.confirm(`Close dirty tab ${path}?`)) return;
    }
    const g = getGroup(groupId);
    const nextG = closeInGroup(g, path);
    setGroup(groupId, nextG);
    if (groupId === "secondary" && nextG.tabPaths.length === 0) {
      // keep empty secondary until user closes split
    }
    const other = groupId === "primary" ? secondary : primary;
    const p = groupId === "primary" ? nextG : primary;
    const s = groupId === "secondary" ? nextG : secondary;
    if (!pathStillOpen(p, s, path)) {
      setDocs((d) => removeDoc(d, path));
    }
    void other;
  }

  const commands: PaletteCommand[] = useMemo(
    () => [
      {
        id: "quickOpen",
        label: "Go to File…",
        hint: "Ctrl+P",
        run: () => setQuickOpen(true),
      },
      {
        id: "split",
        label: "Split Editor Right",
        hint: "Ctrl+\\",
        run: () => splitEditor(),
      },
      {
        id: "unsplit",
        label: "Close Split",
        hint: "Ctrl+Shift+\\",
        run: () => unsplitEditor(),
      },
      {
        id: "focus1",
        label: "Focus Primary Group",
        hint: "Ctrl+1",
        run: () => setFocusedGroup("primary"),
      },
      {
        id: "focus2",
        label: "Focus Secondary Group",
        hint: "Ctrl+2",
        run: () => secondary && setFocusedGroup("secondary"),
      },
      {
        id: "moveTab",
        label: "Move Active Tab to Other Group",
        run: () => activePath && moveToOtherGroup(activePath),
      },
      {
        id: "save",
        label: "Save / Commit Active File",
        run: () => void onCommitActive(),
      },
      {
        id: "saveAll",
        label: "Save All Dirty (Batch Commit)",
        run: () => void onCommitAll(),
      },
      {
        id: "search",
        label: "Search in Repository",
        run: () => setActivity("search"),
      },
      { id: "ci", label: "Run CI Pipeline", run: () => void onRunCi() },
      { id: "agent", label: "Run Agent Mesh Job", run: () => void onRunAgent() },
      { id: "mls", label: "OpenMLS Probe", run: () => void onMlsProbe() },
      {
        id: "explorer",
        label: "Show Explorer",
        run: () => setActivity("explorer"),
      },
      { id: "scm", label: "Show Source Control", run: () => setActivity("scm") },
      {
        id: "problems",
        label: "Show Problems Panel",
        run: () => setBottomTab("problems"),
      },
      {
        id: "refresh",
        label: "Refresh Repos",
        run: () => void refreshRepos(),
      },
    ],
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [repoId, activePath, docs, secondary, message, branch],
  );

  const isolation = String(
    domain?.planes?.isolation_mode ?? domain?.ci?.isolation ?? "?",
  );

  const groupPaneProps = {
    docs,
    busy,
    diagnostics,
    lspSessionId,
    repoName: repo?.name,
    split: isSplit,
    onChange: (path: string, content: string) =>
      setDocs((d) => updateDocContent(d, path, content)),
    onCursor: async (path: string, line: number, ch: number) => {
      if (!lspSessionId) return;
      try {
        const r = await api.lspHover(lspSessionId, path, line, ch);
        setHoverText(r.hover?.contents ?? null);
      } catch {
        setHoverText(null);
      }
    },
    onGoTo: (p: string) => void openFileAt(p),
    onSplit: splitEditor,
    onUnsplit: unsplitEditor,
    onMoveToOther: moveToOtherGroup,
  };

  return (
    <div className="oss">
      <div className="oss-main">
        <nav className="activity" aria-label="Activity bar">
          {(
            [
              ["explorer", "EX"],
              ["search", "SR"],
              ["scm", "SC"],
              ["collab", "PR"],
              ["run", "CI"],
              ["agents", "AG"],
              ["terminal", "TR"],
              ["debug", "DB"],
              ["mls", "ML"],
              ["ext", "XT"],
              ["settings", "ST"],
            ] as const
          ).map(([id, label]) => (
            <button
              key={id}
              type="button"
              title={id}
              className={activity === id ? "active" : ""}
              onClick={() => setActivity(id)}
            >
              {label}
            </button>
          ))}
          <div className="spacer" />
          <button
            type="button"
            title="Split editor"
            onClick={() => splitEditor()}
          >
            ⊞
          </button>
          <button
            type="button"
            title="Command Palette"
            onClick={() => setPaletteOpen(true)}
          >
            ⌘
          </button>
        </nav>

        <aside className="sidebar">
          {activity === "explorer" && (
            <>
              <div className="sidebar-h">
                <span>Explorer</span>
                <button type="button" onClick={() => void refreshRepos()}>
                  ↻
                </button>
              </div>
              <div className="create-row">
                <input
                  placeholder="new-repo"
                  value={newRepo}
                  onChange={(e) => setNewRepo(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && void onCreateRepo()}
                />
                <button
                  type="button"
                  className="primary"
                  disabled={busy}
                  onClick={() => void onCreateRepo()}
                >
                  +
                </button>
              </div>
              <div className="sidebar-body">
                <div className="side-section">
                  <strong>Repositories</strong>
                </div>
                {repos.map((r) => (
                  <button
                    key={r.id}
                    type="button"
                    className={`list-item${r.id === repoId ? " active" : ""}`}
                    onClick={() => setRepoId(r.id)}
                  >
                    {r.name}
                    <span className="sub">
                      {(r.head_sha ?? "").slice(0, 8) || "—"}
                    </span>
                  </button>
                ))}
                <div className="side-section" style={{ marginTop: "0.5rem" }}>
                  <strong>
                    {repo?.name ?? "—"} {treePath ? `/ ${treePath}` : ""}
                  </strong>
                  <button
                    type="button"
                    disabled={!treePath}
                    onClick={() => void onUp()}
                  >
                    ↑ up
                  </button>
                </div>
                {entries.map((e) => (
                  <button
                    key={e.oid + e.path}
                    type="button"
                    className={`tree-entry${activePath === e.path ? " active" : ""}`}
                    onClick={() => void onOpenEntry(e)}
                    onDoubleClick={() => {
                      if (e.kind !== "tree" && secondary) {
                        void openFileAt(e.path, "secondary");
                      }
                    }}
                    title="Double-click opens in secondary when split"
                  >
                    <span className="kind">
                      {e.kind === "tree" ? "dir" : "  "}
                    </span>
                    <span>{e.path.split("/").pop()}</span>
                  </button>
                ))}
              </div>
            </>
          )}

          {activity === "search" && (
            <>
              <div className="sidebar-h">Search</div>
              <div className="search-box">
                <input
                  placeholder="Search in files…"
                  value={searchQ}
                  onChange={(e) => setSearchQ(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && void onSearch()}
                />
              </div>
              <div className="side-actions">
                <button
                  type="button"
                  className="primary"
                  disabled={busy}
                  onClick={() => void onSearch()}
                >
                  Search
                </button>
              </div>
              <div className="sidebar-body">
                {searchHits.map((h, i) => (
                  <button
                    key={`${h.path}-${h.line}-${i}`}
                    type="button"
                    className="hit-row"
                    onClick={() => void openFileAt(h.path)}
                  >
                    <div className="meta">
                      {h.path}:{h.line + 1}
                    </div>
                    <div>{h.preview}</div>
                  </button>
                ))}
                {!searchHits.length && (
                  <div className="empty">Enter a query and press Enter.</div>
                )}
              </div>
            </>
          )}

          {activity === "scm" && (
            <>
              <div className="sidebar-h">Source Control</div>
              <div className="side-section">
                Branch <strong>{branch}</strong> · dirty{" "}
                <strong>{dirtyCount}</strong>
              </div>
              <div className="create-row">
                <input
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                  placeholder="commit message"
                />
              </div>
              <div className="side-actions">
                <button
                  type="button"
                  className="primary"
                  disabled={!dirtyCount || busy}
                  onClick={() => void onCommitAll()}
                >
                  Commit All
                </button>
                <button
                  type="button"
                  disabled={!activeTab || busy || !isDirty(activeTab)}
                  onClick={() => void onCommitActive()}
                >
                  Commit Active
                </button>
              </div>
              <div className="sidebar-body">
                {dirtyDocs(docs).map((t) => (
                  <button
                    key={t.path}
                    type="button"
                    className="list-item"
                    onClick={() => void openFileAt(t.path)}
                  >
                    M {t.path}
                  </button>
                ))}
                {!dirtyCount && (
                  <div className="empty">No uncommitted editor changes.</div>
                )}
                <div className="side-section">
                  <strong>History</strong>
                </div>
                {commits.map((c) => (
                  <div key={c.sha} className="commit-row">
                    <div className="meta">{c.sha.slice(0, 8)}</div>
                    <div className="msg">{c.message}</div>
                  </div>
                ))}
              </div>
            </>
          )}

          {activity === "run" && (
            <>
              <div className="sidebar-h">Run & CI</div>
              <div className="side-actions">
                <button
                  type="button"
                  className="primary"
                  disabled={!repoId || busy}
                  onClick={() => void onRunCi()}
                >
                  Run Pipeline
                </button>
              </div>
              <div className="sidebar-body">
                <div className="side-section">
                  Isolation: <strong>{isolation}</strong>
                </div>
                {lastRun ? (
                  <div className="empty" style={{ whiteSpace: "pre-wrap" }}>
                    status={lastRun.status}
                    {"\n"}exit={lastRun.exit_code ?? "—"}
                    {"\n"}iso={lastRun.isolation ?? "—"}
                  </div>
                ) : (
                  <div className="empty">No recent CI run.</div>
                )}
              </div>
            </>
          )}

          {activity === "agents" && (
            <>
              <div className="sidebar-h">Agents</div>
              <div className="side-actions">
                <button
                  type="button"
                  className="primary"
                  disabled={!repoId || busy}
                  onClick={() => void onRunAgent()}
                >
                  Run Mesh Job
                </button>
              </div>
              <div className="sidebar-body">
                {lastJob ? (
                  <div className="empty" style={{ whiteSpace: "pre-wrap" }}>
                    {lastJob.kind} · {lastJob.status}
                    {"\n"}
                    {lastJob.result_summary}
                  </div>
                ) : (
                  <div className="empty">No agent job yet.</div>
                )}
              </div>
            </>
          )}

          {activity === "mls" && (
            <>
              <div className="sidebar-h">OpenMLS</div>
              <div className="side-actions">
                <button
                  type="button"
                  className="primary"
                  disabled={busy}
                  onClick={() => void onMlsProbe()}
                >
                  Create Group
                </button>
                <button
                  type="button"
                  disabled={busy}
                  onClick={() =>
                    void api
                      .mlsRegisterDevice(`web-${Date.now()}`, "web")
                      .then(() => api.mlsDevices())
                      .then((r) =>
                        setMlsInfo(
                          `devices=${JSON.stringify(r.items ?? []).slice(0, 400)}`,
                        ),
                      )
                      .catch((e) => setMlsInfo(String(e)))
                  }
                >
                  Register device
                </button>
              </div>
              <div className="sidebar-body">
                <div className="empty" style={{ whiteSpace: "pre-wrap" }}>
                  {mlsInfo || "Probe forge OpenMLS identity + group."}
                </div>
              </div>
            </>
          )}

          {activity === "collab" && (
            <>
              <div className="sidebar-h">Collab</div>
              <div className="side-actions">
                <button
                  type="button"
                  disabled={!repoId || busy}
                  onClick={() =>
                    void (async () => {
                      if (!repoId) return;
                      await api.createIssue(repoId, `issue ${Date.now()}`);
                      const [i, p] = await Promise.all([
                        api.listIssues(repoId),
                        api.listPulls(repoId),
                      ]);
                      setIssues(i.items ?? []);
                      setPulls(p.items ?? []);
                    })()
                  }
                >
                  New issue
                </button>
                <button
                  type="button"
                  disabled={!repoId || busy}
                  onClick={() =>
                    void (async () => {
                      if (!repoId) return;
                      const [i, p, k] = await Promise.all([
                        api.listIssues(repoId),
                        api.listPulls(repoId),
                        api.listDeployKeys?.(repoId).catch(() => ({ items: [] })),
                      ]);
                      setIssues(i.items ?? []);
                      setPulls(p.items ?? []);
                      setDeployKeys(
                        (k as { items?: unknown[] })?.items ?? [],
                      );
                    })()
                  }
                >
                  Refresh
                </button>
                <button
                  type="button"
                  disabled={!repoId || busy}
                  onClick={() =>
                    void (async () => {
                      if (!repoId) return;
                      const issued = await api.createDeployKey?.(repoId, "ui-key", "read");
                      flash(
                        "ok",
                        `deploy key token (copy once): ${(issued as { token?: string })?.token ?? "?"}`,
                      );
                      const k = await api.listDeployKeys?.(repoId);
                      setDeployKeys(k?.items ?? []);
                    })()
                  }
                >
                  Deploy key
                </button>
              </div>
              <div className="sidebar-body">
                <div className="side-section">
                  <strong>Issues ({issues.length})</strong>
                </div>
                {issues.map((raw, idx) => {
                  const it = raw as { number?: number; title?: string; state?: string };
                  return (
                    <div key={idx} className="commit-row">
                      <div className="meta">
                        #{it.number} · {it.state}
                      </div>
                      <div className="msg">{it.title}</div>
                    </div>
                  );
                })}
                <div className="side-section">
                  <strong>PRs ({pulls.length})</strong>
                </div>
                {pulls.map((raw, idx) => {
                  const it = raw as {
                    number?: number;
                    title?: string;
                    state?: string;
                    source_branch?: string;
                  };
                  return (
                    <div key={idx} className="commit-row">
                      <div className="meta">
                        #{it.number} · {it.state} · {it.source_branch}
                      </div>
                      <div className="msg">{it.title}</div>
                    </div>
                  );
                })}
                <div className="side-section">
                  <strong>Deploy keys ({deployKeys.length})</strong>
                </div>
                {deployKeys.map((raw, idx) => {
                  const it = raw as { name?: string; token_prefix?: string; scope?: string };
                  return (
                    <div key={idx} className="commit-row">
                      <div className="meta">
                        {it.name} · {it.scope}
                      </div>
                      <div className="msg mono">{it.token_prefix}…</div>
                    </div>
                  );
                })}
              </div>
            </>
          )}

          {activity === "terminal" && (
            <>
              <div className="sidebar-h">Terminal</div>
              <div className="side-actions">
                <button
                  type="button"
                  className="primary"
                  disabled={!repoId || busy}
                  onClick={() =>
                    void (async () => {
                      if (!repoId) return;
                      const t = await api.createTerminal(repoId, branch);
                      setTermId(t.terminal_id);
                      setTermLog("opened\n");
                      setBottomTab("terminal");
                    })()
                  }
                >
                  Open
                </button>
                <button
                  type="button"
                  disabled={!termId || busy}
                  onClick={() =>
                    void (async () => {
                      if (!termId) return;
                      const r = await api.termWrite(termId, termCmd);
                      setTermLog(r.log);
                      setBottomTab("terminal");
                    })()
                  }
                >
                  Run
                </button>
              </div>
              <div className="create-row">
                <input
                  value={termCmd}
                  onChange={(e) => setTermCmd(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && termId) {
                      void api.termWrite(termId, termCmd).then((r) => setTermLog(r.log));
                    }
                  }}
                />
              </div>
              <div className="sidebar-body">
                <pre className="empty" style={{ fontFamily: "var(--mono)", fontSize: "0.72rem" }}>
                  {termLog || "Open a terminal for the active repo."}
                </pre>
              </div>
            </>
          )}

          {activity === "debug" && (
            <>
              <div className="sidebar-h">Debug</div>
              <div className="side-actions">
                <button
                  type="button"
                  className="primary"
                  disabled={!repoId || busy}
                  onClick={() =>
                    void (async () => {
                      if (!repoId) return;
                      const d = (await api.debugLaunch(repoId)) as {
                        session_id?: string;
                        adapter?: string;
                        status?: string;
                      };
                      setDebugInfo(
                        `session=${d.session_id}\nadapter=${d.adapter}\nstatus=${d.status}`,
                      );
                      if (d.session_id) {
                        await api.setBreakpoints?.(d.session_id, [
                          { path: "src/lib.rs", line: 1 },
                        ]);
                        await api.debugContinue?.(d.session_id);
                        setDebugInfo((s) => `${s}\nbreakpoints set + continue`);
                      }
                    })()
                  }
                >
                  Launch
                </button>
              </div>
              <div className="sidebar-body">
                <div className="empty" style={{ whiteSpace: "pre-wrap" }}>
                  {debugInfo || "Launch a debug session (DAP-ready adapters)."}
                </div>
              </div>
            </>
          )}

          {activity === "ext" && (
            <>
              <div className="sidebar-h">Extensions</div>
              <div className="side-actions">
                <button
                  type="button"
                  onClick={() =>
                    void api.listExtensions().then((r) => setExtensions(r.items ?? []))
                  }
                >
                  Refresh
                </button>
              </div>
              <div className="sidebar-body">
                {extensions.map((raw, idx) => {
                  const e = raw as { id?: string; name?: string; version?: string };
                  return (
                    <div key={idx} className="commit-row">
                      <div className="meta">{e.id}</div>
                      <div className="msg">
                        {e.name} · v{e.version}
                      </div>
                    </div>
                  );
                })}
                {!extensions.length && (
                  <div className="empty">Load the built-in extension registry.</div>
                )}
              </div>
            </>
          )}

          {activity === "settings" && (
            <>
              <div className="sidebar-h">Settings</div>
              <div className="side-actions">
                <button
                  type="button"
                  onClick={() =>
                    void api.getSettings().then((r) =>
                      setSettingsJson(JSON.stringify(r.settings ?? {}, null, 2)),
                    )
                  }
                >
                  Load
                </button>
                <button
                  type="button"
                  className="primary"
                  onClick={() =>
                    void (async () => {
                      const settings = JSON.parse(settingsJson || "{}") as Record<
                        string,
                        unknown
                      >;
                      await api.putSettings(settings);
                      flash("ok", "settings saved");
                    })()
                  }
                >
                  Save
                </button>
                <button
                  type="button"
                  onClick={() =>
                    void api.quotas().then((q) =>
                      setQuotasText(JSON.stringify(q, null, 2)),
                    )
                  }
                >
                  Quotas
                </button>
              </div>
              <div className="sidebar-body">
                <textarea
                  style={{
                    width: "100%",
                    minHeight: "12rem",
                    background: "var(--bg)",
                    color: "var(--text)",
                    border: "1px solid var(--border)",
                    fontFamily: "var(--mono)",
                    fontSize: "0.75rem",
                    padding: "0.5rem",
                  }}
                  value={settingsJson}
                  onChange={(e) => setSettingsJson(e.target.value)}
                />
                {quotasText && (
                  <pre className="empty" style={{ fontSize: "0.7rem" }}>
                    {quotasText}
                  </pre>
                )}
              </div>
            </>
          )}
        </aside>

        <section className="editor-col">
          <div className="editor-toolbar">
            <input
              className="message"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              placeholder="commit message"
            />
            <input
              style={{ width: "5.5rem" }}
              value={branch}
              onChange={(e) => setBranch(e.target.value)}
              title="branch"
            />
            {dirtyCount > 0 && (
              <span className="dirty">{dirtyCount} modified</span>
            )}
            <button
              type="button"
              title="Split editor"
              onClick={() => splitEditor()}
            >
              Split
            </button>
            {isSplit && (
              <button type="button" onClick={() => unsplitEditor()}>
                Unsplit
              </button>
            )}
            <button
              type="button"
              className="primary"
              disabled={!activeTab || busy || !isDirty(activeTab)}
              onClick={() => void onCommitActive()}
            >
              Commit
            </button>
            <button
              type="button"
              disabled={!dirtyCount || busy}
              onClick={() => void onCommitAll()}
            >
              Commit All
            </button>
          </div>

          <div
            className={`split-row${isSplit ? " split" : ""}`}
            ref={splitRowRef}
          >
            <div
              className="split-pane"
              style={
                isSplit
                  ? { width: `${splitRatio * 100}%`, flex: "none" }
                  : undefined
              }
            >
              <EditorGroupPane
                {...groupPaneProps}
                group={primary}
                focused={focusedGroup === "primary"}
                onFocus={() => setFocusedGroup("primary")}
                onSelectTab={(path) => {
                  setPrimary((g) => ({ ...g, activePath: path }));
                  setFocusedGroup("primary");
                }}
                onCloseTab={(path, e) => onCloseTab("primary", path, e)}
              />
            </div>
            {isSplit && secondary && (
              <>
                <div
                  className="split-sash"
                  onMouseDown={() => setDraggingSplit(true)}
                  title="Drag to resize"
                />
                <div className="split-pane" style={{ flex: 1, minWidth: 0 }}>
                  <EditorGroupPane
                    {...groupPaneProps}
                    group={secondary}
                    focused={focusedGroup === "secondary"}
                    onFocus={() => setFocusedGroup("secondary")}
                    onSelectTab={(path) => {
                      setSecondary((g) =>
                        g ? { ...g, activePath: path } : g,
                      );
                      setFocusedGroup("secondary");
                    }}
                    onCloseTab={(path, e) => onCloseTab("secondary", path, e)}
                  />
                </div>
              </>
            )}
          </div>

          {!docs.length && !isSplit && (
            <div className="empty welcome-overlay">
              <strong>HelixCode · Code-OSS shell</strong>
              <p>
                Multi-tab + <strong>split groups</strong> ·{" "}
                <kbd>Ctrl+\</kbd> split · <kbd>Ctrl+P</kbd> quick open ·{" "}
                <kbd>Ctrl+Shift+P</kbd> palette
              </p>
              <p>
                {electron
                  ? `Electron desktop · ${desktopPlatform()}`
                  : "Browser shell — run Electron via pnpm electron:dev"}{" "}
                · API {API}
              </p>
            </div>
          )}

          {hoverText && <div className="hover-strip">{hoverText}</div>}

          <div className="bottom-panel">
            <div className="bottom-tabs">
              {(
                [
                  ["problems", `Problems (${diagnostics.length})`],
                  ["output", "Output"],
                  ["history", "History"],
                ] as const
              ).map(([id, label]) => (
                <button
                  key={id}
                  type="button"
                  className={bottomTab === id ? "active" : ""}
                  onClick={() => setBottomTab(id)}
                >
                  {label}
                </button>
              ))}
            </div>
            <div className="bottom-body">
              {bottomTab === "problems" &&
                (diagnostics.length === 0 ? (
                  <div className="empty">No problems.</div>
                ) : (
                  diagnostics.map((d, i) => (
                    <button
                      key={`${d.path}-${i}`}
                      type="button"
                      className="problem-row"
                      onClick={() => void openFileAt(d.path || activePath || "")}
                    >
                      <div className="meta">
                        {d.path}:{d.range.start_line + 1} sev{d.severity}
                      </div>
                      <div className="msg">{d.message}</div>
                    </button>
                  ))
                ))}
              {bottomTab === "output" && (
                <pre
                  className="empty"
                  style={{
                    margin: 0,
                    whiteSpace: "pre-wrap",
                    fontFamily: "var(--mono)",
                    fontSize: "0.72rem",
                  }}
                >
                  {output}
                </pre>
              )}
              {bottomTab === "history" &&
                commits.map((c) => (
                  <div key={c.sha} className="commit-row">
                    <div className="meta">
                      {c.sha.slice(0, 10)} · {c.author}
                    </div>
                    <div className="msg">{c.message}</div>
                  </div>
                ))}
            </div>
          </div>
        </section>
      </div>

      <footer className="status">
        <span>HelixCode</span>
        <span className="muted">{domain?.phase ?? "…"}</span>
        {electron && <span title="Electron shell">⚡ electron</span>}
        <span>{branch}</span>
        <span className="muted">
          {String(domain?.planes?.git_backend ?? "git")}
        </span>
        <span className="muted">
          groups={isSplit ? 2 : 1} · focus={focusedGroup === "primary" ? "1" : "2"}
        </span>
        <span className="muted">iso={isolation}</span>
        <span className="muted">
          lsp={lspAvailable ? (lspSessionId ? "on" : "ready") : "off"}
        </span>
        {errCount > 0 && <span className="err">✕ {errCount}</span>}
        {warnCount > 0 && <span className="warn">⚠ {warnCount}</span>}
        <span className="spacer" />
        <span className="muted">
          docs={docs.length} · dirty={dirtyCount} · files={fileIndex.length}
        </span>
        <span className="muted">{API}</span>
      </footer>

      <CommandPalette
        open={paletteOpen}
        commands={commands}
        onClose={() => setPaletteOpen(false)}
      />
      <QuickOpen
        open={quickOpen}
        files={fileIndex}
        onClose={() => setQuickOpen(false)}
        onPick={(p) => void openFileAt(p)}
      />
      {toast && <div className={`toast ${toast.kind}`}>{toast.text}</div>}
    </div>
  );
}
