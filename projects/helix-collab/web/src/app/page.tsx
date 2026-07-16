"use client";

import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type RefObject,
} from "react";
import {
  api,
  type AclEntry,
  type Comment,
  type Doc,
  type DomainStatus,
  type Folder,
  type Mention,
  type Peer,
  type Activity,
  type Revision,
  type Workspace,
  type Attachment,
} from "@/lib/api";
import { API, DEV_USER } from "@/lib/config";
import {
  HelixYjsProvider,
  type ProviderStatus,
} from "@/lib/yjs-provider";
import {
  YTextEditor,
  type YTextEditorHandle,
} from "@/components/YTextEditor";
import {
  ProseMirrorEditor,
  type ProseMirrorHandle,
} from "@/components/ProseMirrorEditor";
import { EditorToolbar } from "@/components/EditorToolbar";
import { MarkdownPreview } from "@/components/MarkdownPreview";
import { activityLabel, relativeTime } from "@/lib/format";
import {
  decryptEnvelope,
  encryptPlaintext,
  encryptWithNewKey,
  exportRawDek,
  hasWrappedKey,
  importRawDek,
  isClientEnvelope,
  isUnlocked,
  setupClientKey,
  unlockClientKey,
} from "@/lib/client-crypto";
import { ensureDeviceKey } from "@/lib/device-keys";
import { offlinePut } from "@/lib/offline-store";
import { planMerge, applyPull, applyPush } from "@/lib/offline-merge";
import { registerPasskey, authenticatePasskey } from "@/lib/passkey";

type RailTab = "people" | "share" | "history" | "comments" | "activity";
type Toast = { id: number; kind: "ok" | "error" | "info"; text: string };
type CommentFilter = "open" | "all";
type EditorMode = "rich" | "markdown";
type EditorHandle = YTextEditorHandle | ProseMirrorHandle;

export default function CollabWorkspace() {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [workspaceId, setWorkspaceId] = useState<string | null>(null);
  const [folders, setFolders] = useState<Folder[]>([]);
  const [folderId, setFolderId] = useState<string | null>(null);
  const [docs, setDocs] = useState<Doc[]>([]);
  const [filter, setFilter] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [title, setTitle] = useState("");
  const [version, setVersion] = useState(0);
  const [peers, setPeers] = useState<Peer[]>([]);
  const [aclItems, setAclItems] = useState<AclEntry[]>([]);
  const [revisions, setRevisions] = useState<Revision[]>([]);
  const [comments, setComments] = useState<Comment[]>([]);
  const [commentBody, setCommentBody] = useState("");
  const [mentionSuggest, setMentionSuggest] = useState<string[]>([]);
  const [showSuggest, setShowSuggest] = useState(false);
  const [inbox, setInbox] = useState<Mention[]>([]);
  const [domain, setDomain] = useState<DomainStatus | null>(null);
  const [wsState, setWsState] = useState<ProviderStatus>("closed");
  const [crdtOn, setCrdtOn] = useState(false);
  const [sealedCrdt, setSealedCrdt] = useState(false);
  const [shareId, setShareId] = useState("");
  const [newFolder, setNewFolder] = useState("");
  const [newWs, setNewWs] = useState("");
  const [railTab, setRailTab] = useState<RailTab>("comments");
  const [saving, setSaving] = useState(false);
  const [provider, setProvider] = useState<HelixYjsProvider | null>(null);
  const [fallbackContent, setFallbackContent] = useState("");
  const [liveText, setLiveText] = useState("");
  const [preview, setPreview] = useState(false);
  const [focusMode, setFocusMode] = useState(false);
  const [conflict, setConflict] = useState<string | null>(null);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [expandedFolders, setExpandedFolders] = useState<Record<string, boolean>>(
    {},
  );
  const [typing, setTyping] = useState<
    Record<string, { display_name: string; until: number }>
  >({});
  const [activity, setActivity] = useState<Activity[]>([]);
  const [attachments, setAttachments] = useState<Attachment[]>([]);
  const [encrypted, setEncrypted] = useState(false);
  const [clientE2ee, setClientE2ee] = useState(false);
  const [locked, setLocked] = useState(false);
  const [editorMode, setEditorMode] = useState<EditorMode>("rich");
  const [passphrase, setPassphrase] = useState("");
  const [importDek, setImportDek] = useState("");
  const [pinned, setPinned] = useState(false);
  const [anchorSel, setAnchorSel] = useState<{
    start: number;
    end: number;
    quote: string;
  } | null>(null);
  const [lastSaved, setLastSaved] = useState({ title: "", content: "" });
  const [commentFilter, setCommentFilter] = useState<CommentFilter>("open");
  const [editingCommentId, setEditingCommentId] = useState<string | null>(null);
  const [editingCommentBody, setEditingCommentBody] = useState("");
  const [showKeys, setShowKeys] = useState(false);
  const providerRef = useRef<HelixYjsProvider | null>(null);
  const editorRef = useRef<EditorHandle>(null);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const typingTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const versionRef = useRef(0);
  const toastId = useRef(1);
  const captureAnchorRef = useRef<() => void>(() => undefined);

  const toast = useCallback((text: string, kind: Toast["kind"] = "info") => {
    const id = toastId.current++;
    setToasts((t) => [...t, { id, kind, text }]);
    setTimeout(() => {
      setToasts((t) => t.filter((x) => x.id !== id));
    }, 3200);
  }, []);

  useEffect(() => {
    versionRef.current = version;
  }, [version]);

  const loadWorkspaces = useCallback(async () => {
    const list = await api<Workspace[]>("/v1/workspaces");
    setWorkspaces(list);
    if (!workspaceId && list.length > 0) setWorkspaceId(list[0]!.id);
    return list;
  }, [workspaceId]);

  const loadFolders = useCallback(async (ws: string) => {
    const list = await api<Folder[]>(`/v1/workspaces/${ws}/folders`);
    setFolders(list);
  }, []);

  const loadDocs = useCallback(async () => {
    const params = new URLSearchParams();
    if (workspaceId) params.set("workspace_id", workspaceId);
    if (folderId) params.set("folder_id", folderId);
    else if (workspaceId) params.set("root", "1");
    const q = params.toString();
    const data = await api<{ items: Doc[] }>(
      `/v1/documents${q ? `?${q}` : ""}`,
    );
    setDocs(data.items ?? []);
  }, [workspaceId, folderId]);

  const loadInbox = useCallback(async () => {
    const list = await api<Mention[]>(
      `/v1/mentions/inbox?label=${encodeURIComponent(DEV_USER)}`,
    ).catch(() => [] as Mention[]);
    setInbox(list);
  }, []);

  useEffect(() => {
    Promise.all([
      loadWorkspaces().catch(() => [] as Workspace[]),
      api<DomainStatus>("/v1/domain/status").catch(() => null),
      loadInbox(),
      // Horizon A: register device public key (private never leaves browser).
      ensureDeviceKey(DEV_USER)
        .then((d) =>
          api("/v1/devices", {
            method: "POST",
            body: JSON.stringify(d),
          }).catch(() => null),
        )
        .catch(() => null),
    ])
      .then(([, d]) => {
        if (d) setDomain(d);
      })
      .catch((e) => toast(String(e), "error"));
  }, [loadWorkspaces, loadInbox, toast]);

  useEffect(() => {
    if (!workspaceId) return;
    void loadFolders(workspaceId).catch((e) => toast(String(e), "error"));
    setFolderId(null);
  }, [workspaceId, loadFolders, toast]);

  useEffect(() => {
    void loadDocs().catch((e) => toast(String(e), "error"));
  }, [loadDocs, toast]);

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return docs;
    return docs.filter((d) => d.title.toLowerCase().includes(q));
  }, [docs, filter]);

  const childrenOf = useCallback(
    (parent: string | null) =>
      folders.filter((f) => (f.parent_id ?? null) === parent),
    [folders],
  );

  const closeProvider = useCallback(() => {
    providerRef.current?.destroy();
    providerRef.current = null;
    setProvider(null);
    setSealedCrdt(false);
  }, []);

  const connectProvider = useCallback(
    (id: string, opts: { sealed: boolean; seedMarkdown?: string }) => {
      closeProvider();
      const p = new HelixYjsProvider(
        id,
        {
          onStatus: setWsState,
          onCrdtMode: setCrdtOn,
          onSealedMode: setSealedCrdt,
          onPresence: (peer) => {
            setPeers((prev) => {
              const rest = prev.filter((x) => x.user_id !== peer.user_id);
              return [...rest, peer];
            });
          },
          onPeerLeft: (uid) =>
            setPeers((prev) => prev.filter((x) => x.user_id !== uid)),
          onTyping: (t) => {
            if (t.user_id === DEV_USER) return;
            setTyping((prev) => {
              const next = { ...prev };
              if (t.active) {
                next[t.user_id] = {
                  display_name: t.display_name,
                  until: Date.now() + 2500,
                };
              } else {
                delete next[t.user_id];
              }
              return next;
            });
          },
          onCommentEvent: () => {
            void api<Comment[]>(`/v1/documents/${id}/comments`)
              .then((c) => setComments(c ?? []))
              .catch(() => undefined);
            void api<Activity[]>(`/v1/documents/${id}/activity`)
              .then((a) => setActivity(a ?? []))
              .catch(() => undefined);
          },
          onSnapshot: (s) => {
            setVersion(s.version);
            setTitle(s.title);
            // Sealed: content is HC1 — do not put ciphertext into the editor.
            if (opts.sealed || String(s.content).startsWith("HC1.")) {
              setConflict(null);
              return;
            }
            setFallbackContent(s.content);
            setLiveText(s.content);
            setLastSaved({ title: s.title, content: s.content });
            setConflict(null);
          },
          onAck: (v) => {
            setVersion(v);
            setConflict(null);
          },
          onError: (m) => {
            if (m.includes("CRDT disabled")) {
              setCrdtOn(false);
              return;
            }
            if (m.toLowerCase().includes("conflict") || m.includes("version")) {
              setConflict(m);
              return;
            }
            toast(m, "error");
          },
        },
        { sealed: opts.sealed },
      );
      // Seed local Y.Text from decrypted markdown when sealed room is empty.
      if (opts.sealed && opts.seedMarkdown) {
        p.ydoc.transact(() => {
          if (p.ytext.length === 0 && opts.seedMarkdown) {
            p.ytext.insert(0, opts.seedMarkdown);
          }
        }, "remote");
      }
      providerRef.current = p;
      setProvider(p);
      setSealedCrdt(opts.sealed);
      p.connect();
      return p;
    },
    [closeProvider, toast],
  );

  const openDoc = useCallback(
    async (id: string) => {
      setConflict(null);
      closeProvider();
      setPassphrase("");
      setImportDek("");

      const doc = await api<Doc>(`/v1/documents/${id}`);
      setSelectedId(doc.id);
      setTitle(doc.title);
      setVersion(doc.version);
      setEditingCommentId(null);
      setAnchorSel(null);

      setEncrypted(!!doc.encrypted);
      setClientE2ee(!!doc.client_e2ee);
      setPinned(!!doc.pinned);

      let content = doc.content;
      let isLocked = false;
      if (doc.client_e2ee) {
        if (isUnlocked(id)) {
          try {
            content = await decryptEnvelope(id, doc.content);
          } catch {
            isLocked = true;
            content = "";
          }
        } else {
          isLocked = true;
          content = "";
        }
      }
      setLocked(isLocked);
      setFallbackContent(content);
      setLiveText(content);
      setLastSaved({ title: doc.title, content });

      const [presence, acl, revs, cmts, suggest, act, atts] = await Promise.all([
        api<{ peers: Peer[] }>(`/v1/documents/${id}/presence`).catch(() => ({
          peers: [] as Peer[],
        })),
        api<{ items: AclEntry[] }>(`/v1/documents/${id}/share`).catch(() => ({
          items: [] as AclEntry[],
        })),
        api<Revision[]>(`/v1/documents/${id}/revisions?limit=30`).catch(
          () => [] as Revision[],
        ),
        api<Comment[]>(`/v1/documents/${id}/comments`).catch(
          () => [] as Comment[],
        ),
        api<{ suggestions: string[] }>(
          `/v1/documents/${id}/mention-suggest`,
        ).catch(() => ({ suggestions: [] as string[] })),
        api<Activity[]>(`/v1/documents/${id}/activity`).catch(
          () => [] as Activity[],
        ),
        api<{ items: Attachment[] }>(`/v1/documents/${id}/attachments`).catch(
          () => ({ items: [] as Attachment[] }),
        ),
      ]);
      setPeers(presence.peers ?? []);
      setAclItems(acl.items ?? []);
      setRevisions(revs ?? []);
      setComments(cmts ?? []);
      setMentionSuggest(suggest.suggestions ?? []);
      setActivity(act ?? []);
      setAttachments(atts.items ?? []);

      // Locked client-e2ee: no WS until unlock. Unlocked: sealed CRDT.
      if (doc.client_e2ee) {
        if (!isLocked) {
          const p = connectProvider(id, {
            sealed: true,
            seedMarkdown: content,
          });
          // Publish sealed full state so peers / rejoin can catch up.
          setTimeout(() => void p.publishSealedFullState(), 400);
        } else {
          setWsState("closed");
          setCrdtOn(false);
          setSealedCrdt(false);
        }
        return;
      }

      connectProvider(id, { sealed: false, seedMarkdown: content });
    },
    [closeProvider, toast, connectProvider],
  );

  useEffect(() => () => closeProvider(), [closeProvider]);

  // Global keyboard shortcuts
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      if (mod && e.key.toLowerCase() === "s") {
        e.preventDefault();
        void saveRef.current();
        return;
      }
      if (mod && e.key === "/") {
        e.preventDefault();
        setShowKeys((s) => !s);
        return;
      }
      if (mod && e.shiftKey && e.key.toLowerCase() === "p") {
        e.preventDefault();
        setPreview((p) => !p);
        return;
      }
      if (mod && e.shiftKey && e.key.toLowerCase() === "f") {
        e.preventDefault();
        setFocusMode((f) => !f);
        return;
      }
      if (mod && e.shiftKey && e.key.toLowerCase() === "m") {
        e.preventDefault();
        captureAnchorRef.current();
        return;
      }
      if (e.key === "Escape") {
        if (showKeys) setShowKeys(false);
        else if (focusMode) setFocusMode(false);
        else if (editingCommentId) setEditingCommentId(null);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [showKeys, focusMode, editingCommentId]);

  const ensureWorkspace = async (): Promise<string> => {
    if (workspaceId) return workspaceId;
    const rec = await api<Workspace>("/v1/workspaces", {
      method: "POST",
      body: JSON.stringify({ name: "General" }),
    });
    await loadWorkspaces();
    setWorkspaceId(rec.id);
    return rec.id;
  };

  const createDoc = async (opts?: { clientE2ee?: boolean }) => {
    const ws = await ensureWorkspace();
    const seed =
      "# New document\n\nStart writing…\n\nMention someone with @name\n";
    let content = seed;
    let client_e2ee = false;
    let bind: ((id: string) => Promise<void>) | null = null;
    if (opts?.clientE2ee) {
      const pass = window.prompt(
        "Passphrase for new client-E2EE doc (min 8 chars)",
      );
      if (!pass) return;
      const sealed = await encryptWithNewKey(pass, seed);
      content = sealed.envelope;
      client_e2ee = true;
      bind = sealed.bind;
    }
    const doc = await api<Doc>("/v1/documents", {
      method: "POST",
      body: JSON.stringify({
        title: "Untitled",
        content,
        workspace_id: ws,
        folder_id: folderId,
        client_e2ee,
      }),
    });
    if (bind) await bind(doc.id);
    await loadDocs();
    await openDoc(doc.id);
    toast(
      client_e2ee ? "Client-E2EE document created" : "Document created",
      "ok",
    );
  };

  const createWorkspace = async () => {
    if (!newWs.trim()) return;
    const rec = await api<Workspace>("/v1/workspaces", {
      method: "POST",
      body: JSON.stringify({ name: newWs.trim() }),
    });
    setNewWs("");
    await loadWorkspaces();
    setWorkspaceId(rec.id);
    toast(`Workspace “${rec.name}” created`, "ok");
  };

  const createFolder = async (parentId?: string | null) => {
    if (!workspaceId || !newFolder.trim()) return;
    await api(`/v1/workspaces/${workspaceId}/folders`, {
      method: "POST",
      body: JSON.stringify({
        name: newFolder.trim(),
        parent_id: parentId ?? null,
      }),
    });
    setNewFolder("");
    await loadFolders(workspaceId);
    if (parentId) {
      setExpandedFolders((e) => ({ ...e, [parentId]: true }));
    }
    toast("Folder created", "ok");
  };

  const plainContent = useCallback(() => {
    if (editorMode === "rich" && editorRef.current && "getMarkdown" in editorRef.current) {
      return (editorRef.current as ProseMirrorHandle).getMarkdown();
    }
    if (provider && !clientE2ee && editorMode === "markdown") {
      return provider.ytext.toString();
    }
    if (editorRef.current && "getSelection" in editorRef.current) {
      // Prefer liveText which both editors feed via onChange
      return liveText || fallbackContent;
    }
    return liveText || fallbackContent;
  }, [provider, fallbackContent, liveText, editorMode, clientE2ee]);

  const save = useCallback(async () => {
    if (!selectedId || locked) return;
    setSaving(true);
    try {
      const plain = plainContent();
      let content = plain;
      if (clientE2ee) {
        content = await encryptPlaintext(selectedId, plain);
      }
      if (provider && wsState === "open" && !crdtOn && !clientE2ee) {
        provider.sendDurablePatch(versionRef.current, content);
      }
      const doc = await api<Doc>(`/v1/documents/${selectedId}`, {
        method: "PATCH",
        body: JSON.stringify({
          base_version: versionRef.current,
          content,
          title,
        }),
      });
      setVersion(doc.version);
      setTitle(doc.title);
      // Keep plaintext in editor state for client e2ee
      setFallbackContent(plain);
      setLiveText(plain);
      setLastSaved({ title: doc.title, content: plain });
      setConflict(null);
      toast(`Saved v${doc.version}`, "ok");
      await loadDocs();
      const revs = await api<Revision[]>(
        `/v1/documents/${selectedId}/revisions?limit=30`,
      ).catch(() => [] as Revision[]);
      setRevisions(revs);
      const act = await api<Activity[]>(
        `/v1/documents/${selectedId}/activity`,
      ).catch(() => [] as Activity[]);
      setActivity(act);
    } catch (e) {
      const msg = String(e);
      if (msg.toLowerCase().includes("conflict") || msg.includes("409")) {
        setConflict(msg);
      } else {
        toast(msg, "error");
      }
    } finally {
      setSaving(false);
    }
  }, [
    selectedId,
    locked,
    plainContent,
    provider,
    wsState,
    crdtOn,
    title,
    loadDocs,
    toast,
    clientE2ee,
  ]);

  const saveRef = useRef(save);
  useEffect(() => {
    saveRef.current = save;
  }, [save]);

  const isDirty = useMemo(() => {
    if (!selectedId) return false;
    return title !== lastSaved.title || liveText !== lastSaved.content;
  }, [selectedId, title, liveText, lastSaved]);

  // Debounced autosave on dirty title/content
  useEffect(() => {
    if (!selectedId || !isDirty || conflict || locked) return;
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      void saveRef.current();
    }, 1600);
    return () => {
      if (saveTimer.current) clearTimeout(saveTimer.current);
    };
  }, [title, liveText, selectedId, isDirty, conflict, locked]);

  // CRDT: periodic snapshot to Postgres for durability
  useEffect(() => {
    if (!selectedId || !crdtOn || locked) return;
    const t = setInterval(() => {
      void saveRef.current();
    }, 10000);
    return () => clearInterval(t);
  }, [selectedId, crdtOn, locked]);

  // Sealed CRDT: re-publish full encrypted state for late joiners
  useEffect(() => {
    if (!selectedId || !sealedCrdt || locked || !provider) return;
    const t = setInterval(() => {
      void provider.publishSealedFullState();
    }, 15000);
    return () => clearInterval(t);
  }, [selectedId, sealedCrdt, locked, provider]);

  const reloadDoc = async () => {
    if (!selectedId) return;
    await openDoc(selectedId);
    toast("Reloaded latest version", "ok");
  };

  const deleteDoc = async () => {
    if (!selectedId) return;
    if (!confirm("Delete this document?")) return;
    await api(`/v1/documents/${selectedId}`, { method: "DELETE" });
    closeProvider();
    setSelectedId(null);
    setTitle("");
    setFallbackContent("");
    setLiveText("");
    setPeers([]);
    setAclItems([]);
    setRevisions([]);
    setComments([]);
    await loadDocs();
    toast("Document deleted", "ok");
  };

  const shareDoc = async () => {
    if (!selectedId || !shareId.trim()) return;
    try {
      await api(`/v1/documents/${selectedId}/share`, {
        method: "POST",
        body: JSON.stringify({
          principal_id: shareId.trim(),
          principal_kind: "user",
          permissions: ["read", "write"],
        }),
      });
      toast(`Shared with ${shareId.trim()}`, "ok");
      setShareId("");
      const acl = await api<{ items: AclEntry[] }>(
        `/v1/documents/${selectedId}/share`,
      );
      setAclItems(acl.items ?? []);
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const restoreRev = async (v: number) => {
    if (!selectedId) return;
    if (!confirm(`Restore revision v${v} as a new version?`)) return;
    try {
      await api<Doc>(`/v1/documents/${selectedId}/revisions/${v}/restore`, {
        method: "POST",
      });
      toast(`Restored from v${v}`, "ok");
      await openDoc(selectedId);
      await loadDocs();
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const postComment = async () => {
    if (!selectedId || !commentBody.trim()) return;
    try {
      const c = await api<Comment>(`/v1/documents/${selectedId}/comments`, {
        method: "POST",
        body: JSON.stringify({
          body: commentBody.trim(),
          author_label: DEV_USER,
          anchor_start: anchorSel?.start ?? null,
          anchor_end: anchorSel?.end ?? null,
          anchor_quote: anchorSel?.quote ?? "",
        }),
      });
      setComments((prev) => [...prev, c]);
      setCommentBody("");
      setShowSuggest(false);
      setAnchorSel(null);
      toast(
        c.mentions.length
          ? `Comment · @${c.mentions.map((m) => m.mentioned_label).join(", @")}`
          : c.anchor_quote
            ? "Anchored comment posted"
            : "Comment posted",
        "ok",
      );
      await loadInbox();
      const act = await api<Activity[]>(
        `/v1/documents/${selectedId}/activity`,
      ).catch(() => [] as Activity[]);
      setActivity(act);
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const resolveComment = async (cid: string, resolved: boolean) => {
    if (!selectedId) return;
    try {
      const c = await api<Comment>(
        `/v1/documents/${selectedId}/comments/${cid}/resolve`,
        {
          method: "POST",
          body: JSON.stringify({ resolved }),
        },
      );
      setComments((prev) => prev.map((x) => (x.id === cid ? c : x)));
      toast(resolved ? "Thread resolved" : "Thread reopened", "ok");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const toggleE2ee = async () => {
    if (!selectedId || clientE2ee) {
      toast("Disable client E2EE first for server vault mode", "info");
      return;
    }
    try {
      const doc = await api<Doc>(`/v1/documents/${selectedId}/flags`, {
        method: "POST",
        body: JSON.stringify({ e2ee: !encrypted }),
      });
      setEncrypted(!!doc.encrypted);
      setVersion(doc.version);
      setFallbackContent(doc.content);
      setLiveText(doc.content);
      toast(doc.encrypted ? "Server vault e2ee on" : "Server vault e2ee off", "ok");
      await openDoc(selectedId);
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const enableClientE2ee = async () => {
    if (!selectedId) return;
    if (encrypted && !clientE2ee) {
      toast("Disable server vault e2ee before enabling client E2EE", "info");
      return;
    }
    const pass = window.prompt(
      "Passphrase for client-held keys (min 8 chars). Keys stay in this browser.",
    );
    if (!pass) return;
    try {
      await setupClientKey(selectedId, pass);
      const plain = plainContent();
      const ct = await encryptPlaintext(selectedId, plain);
      const doc = await api<Doc>(`/v1/documents/${selectedId}/flags`, {
        method: "POST",
        body: JSON.stringify({ client_e2ee: true, content: ct }),
      });
      setClientE2ee(!!doc.client_e2ee);
      setEncrypted(!!doc.encrypted);
      setVersion(doc.version);
      setLocked(false);
      setFallbackContent(plain);
      setLiveText(plain);
      setLastSaved({ title: doc.title, content: plain });
      const p = connectProvider(selectedId, {
        sealed: true,
        seedMarkdown: plain,
      });
      setTimeout(() => void p.publishSealedFullState(), 300);
      toast("Client E2EE + sealed CRDT on — server is blind", "ok");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const disableClientE2ee = async () => {
    if (!selectedId || !clientE2ee) return;
    if (locked) {
      toast("Unlock the document first", "info");
      return;
    }
    if (!confirm("Disable client E2EE and store plaintext on server?")) return;
    try {
      const plain = plainContent();
      const doc = await api<Doc>(`/v1/documents/${selectedId}/flags`, {
        method: "POST",
        body: JSON.stringify({ client_e2ee: false, content: plain }),
      });
      setClientE2ee(false);
      setEncrypted(!!doc.encrypted);
      setVersion(doc.version);
      setLocked(false);
      toast("Client E2EE disabled", "ok");
      await openDoc(selectedId);
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const unlockDoc = async () => {
    if (!selectedId) return;
    try {
      if (importDek.trim()) {
        await importRawDek(selectedId, importDek.trim(), passphrase);
      } else if (hasWrappedKey(selectedId)) {
        await unlockClientKey(selectedId, passphrase);
      } else {
        await setupClientKey(selectedId, passphrase);
      }
      const raw = await api<Doc>(`/v1/documents/${selectedId}`);
      const plain = await decryptEnvelope(selectedId, raw.content);
      setFallbackContent(plain);
      setLiveText(plain);
      setLastSaved({ title: raw.title, content: plain });
      setLocked(false);
      setPassphrase("");
      setImportDek("");
      const p = connectProvider(selectedId, {
        sealed: true,
        seedMarkdown: plain,
      });
      setTimeout(() => void p.publishSealedFullState(), 300);
      toast("Unlocked — sealed CRDT live", "ok");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const exportClientKey = async () => {
    if (!selectedId || !clientE2ee || locked) return;
    try {
      const raw = await exportRawDek(selectedId);
      await navigator.clipboard.writeText(raw);
      toast("Raw DEK copied — share out-of-band only", "ok");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const exportBackpack = async () => {
    if (!selectedId) return;
    try {
      const res = await api<{ sha256: string; backpack: Record<string, unknown> }>(
        `/v1/documents/${selectedId}/export`,
      );
      const blob = new Blob([JSON.stringify(res.backpack, null, 2)], {
        type: "application/json",
      });
      const a = document.createElement("a");
      a.href = URL.createObjectURL(blob);
      a.download = `helix-collab-${selectedId.slice(0, 8)}.backpack.json`;
      a.click();
      URL.revokeObjectURL(a.href);
      // Offline cache of durable payload (may be HC1).
      await offlinePut({
        docId: selectedId,
        title,
        content: plainContent(),
        client_e2ee: clientE2ee,
        version,
        updated_at: Date.now(),
      }).catch(() => undefined);
      toast(`Backpack exported · ${res.sha256.slice(0, 12)}…`, "ok");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const agentSuggest = async () => {
    if (!selectedId || locked) return;
    const selection = plainContent().slice(0, 2000);
    try {
      const res = await api<{ suggestion: string; model: string }>(
        `/v1/documents/${selectedId}/agent/suggest`,
        {
          method: "POST",
          body: JSON.stringify({ selection, intent: "summarize" }),
        },
      );
      toast(`${res.model}: ${res.suggestion.slice(0, 120)}…`, "info");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const runOfflineMerge = async () => {
    if (!selectedId) return;
    try {
      const plan = await planMerge(selectedId);
      if (plan.action === "in_sync") {
        toast("Offline cache in sync", "ok");
        return;
      }
      if (plan.action === "pull_server") {
        await applyPull(plan);
        if (plan.serverContent != null) {
          setFallbackContent(
            plan.serverContent.startsWith("HC1.") && clientE2ee
              ? plainContent()
              : plan.serverContent,
          );
        }
        toast(`Pulled server v${plan.serverVersion}`, "ok");
        return;
      }
      if (plan.action === "push_local") {
        const doc = await applyPush(plan, async (plain) => {
          if (!selectedId) return plain;
          const { encryptPlaintext } = await import("@/lib/client-crypto");
          return encryptPlaintext(selectedId, plain);
        });
        setVersion(doc.version);
        toast(`Pushed local → v${doc.version}`, "ok");
        return;
      }
      if (plan.action === "conflict") {
        if (
          confirm(
            `Conflict local v${plan.local?.version} vs server v${plan.serverVersion}. OK=use server, Cancel=keep local`,
          )
        ) {
          await applyPull(plan);
          toast("Resolved with server", "ok");
        } else {
          toast("Kept local — use Push via merge again", "info");
        }
        return;
      }
      toast("Local-only offline copy", "info");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const bindPasskey = async () => {
    try {
      const id = await registerPasskey(DEV_USER);
      toast(`Passkey registered ${id.slice(0, 8)}…`, "ok");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const verifyPasskey = async () => {
    try {
      const ok = await authenticatePasskey();
      toast(ok ? "Passkey OK" : "Passkey failed", ok ? "ok" : "error");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const togglePin = async () => {
    if (!selectedId) return;
    try {
      const doc = await api<Doc>(`/v1/documents/${selectedId}/flags`, {
        method: "POST",
        body: JSON.stringify({ pinned: !pinned }),
      });
      setPinned(!!doc.pinned);
      await loadDocs();
      toast(doc.pinned ? "Pinned" : "Unpinned", "ok");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const archiveDoc = async () => {
    if (!selectedId) return;
    if (!confirm("Archive this document? It will leave the active list.")) return;
    try {
      await api<Doc>(`/v1/documents/${selectedId}/flags`, {
        method: "POST",
        body: JSON.stringify({ archive: true }),
      });
      closeProvider();
      setSelectedId(null);
      setTitle("");
      setFallbackContent("");
      setLiveText("");
      setLastSaved({ title: "", content: "" });
      await loadDocs();
      toast("Document archived", "ok");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const duplicateDoc = async () => {
    if (!selectedId) return;
    try {
      const ws = await ensureWorkspace();
      const doc = await api<Doc>("/v1/documents", {
        method: "POST",
        body: JSON.stringify({
          title: `${title || "Untitled"} (copy)`,
          content: plainContent(),
          workspace_id: ws,
          folder_id: folderId,
          e2ee: encrypted,
        }),
      });
      await loadDocs();
      await openDoc(doc.id);
      toast("Document duplicated", "ok");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const copyDocId = async () => {
    if (!selectedId) return;
    try {
      await navigator.clipboard.writeText(selectedId);
      toast("Document id copied", "ok");
    } catch {
      toast(selectedId, "info");
    }
  };

  const renameFolderUi = async (id: string, current: string) => {
    const name = window.prompt("Rename folder", current);
    if (!name?.trim() || name.trim() === current) return;
    try {
      await api(`/v1/folders/${id}`, {
        method: "PATCH",
        body: JSON.stringify({ name: name.trim() }),
      });
      if (workspaceId) await loadFolders(workspaceId);
      toast("Folder renamed", "ok");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const captureAnchor = () => {
    const sel = editorRef.current?.getSelection();
    if (!sel || sel.start === sel.end) {
      toast("Select text in the editor to anchor a comment", "info");
      return;
    }
    const text = plainContent();
    setAnchorSel({
      start: sel.start,
      end: sel.end,
      quote: text.slice(sel.start, sel.end).slice(0, 200),
    });
    setRailTab("comments");
    toast("Selection anchored — write your comment", "ok");
  };

  captureAnchorRef.current = captureAnchor;

  const jumpToAnchor = (c: Comment) => {
    if (typeof c.anchor_start !== "number") {
      toast("No text anchor on this comment", "info");
      return;
    }
    const end =
      typeof c.anchor_end === "number" ? c.anchor_end : c.anchor_start;
    editorRef.current?.setSelection(c.anchor_start, end);
    editorRef.current?.focus();
    toast("Jumped to anchor", "info");
  };

  const saveCommentEdit = async (cid: string) => {
    if (!selectedId || !editingCommentBody.trim()) return;
    try {
      const c = await api<Comment>(
        `/v1/documents/${selectedId}/comments/${cid}`,
        {
          method: "PATCH",
          body: JSON.stringify({ body: editingCommentBody.trim() }),
        },
      );
      setComments((prev) => prev.map((x) => (x.id === cid ? c : x)));
      setEditingCommentId(null);
      toast("Comment updated", "ok");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const onEditorChange = (text: string) => {
    setLiveText(text);
    provider?.sendTyping(true);
    if (typingTimer.current) clearTimeout(typingTimer.current);
    typingTimer.current = setTimeout(() => provider?.sendTyping(false), 1200);
  };

  const visibleComments = useMemo(() => {
    if (commentFilter === "all") return comments;
    return comments.filter((c) => !c.resolved_at);
  }, [comments, commentFilter]);

  const moveToFolder = async (fid: string | null) => {
    if (!selectedId) return;
    await api(`/v1/documents/${selectedId}/move`, {
      method: "POST",
      body: JSON.stringify({
        folder_id: fid,
        workspace_id: workspaceId,
      }),
    });
    toast(fid ? "Moved into folder" : "Moved to root", "ok");
    await loadDocs();
  };

  const exportMd = () => {
    const blob = new Blob([plainContent()], {
      type: "text/markdown;charset=utf-8",
    });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `${(title || "document").replace(/[^\w.-]+/g, "_")}.md`;
    a.click();
    URL.revokeObjectURL(a.href);
    toast("Exported .md", "ok");
  };

  const filteredSuggest = useMemo(() => {
    const m = commentBody.match(/@([A-Za-z0-9_.@+-]*)$/);
    if (!m) return [];
    const q = (m[1] ?? "").toLowerCase();
    return mentionSuggest
      .filter((s) => s.toLowerCase().includes(q))
      .slice(0, 8);
  }, [commentBody, mentionSuggest]);

  useEffect(() => {
    setShowSuggest(filteredSuggest.length > 0 && commentBody.includes("@"));
  }, [filteredSuggest, commentBody]);

  const applyMention = (label: string) => {
    setCommentBody((b) => b.replace(/@([A-Za-z0-9_.@+-]*)$/, `@${label} `));
    setShowSuggest(false);
  };

  const renderFolder = (parent: string | null, depth: number) =>
    childrenOf(parent).map((f) => {
      const kids = childrenOf(f.id);
      const open = expandedFolders[f.id] ?? depth < 1;
      return (
        <div key={f.id} className={depth ? "folder-node" : undefined}>
          <div className="folder-row">
          <button
            type="button"
            className={`tab ${folderId === f.id ? "active" : ""}`}
            style={{ flex: 1, textAlign: "left", marginBottom: 2 }}
            onClick={() => setFolderId(f.id)}
            onDoubleClick={(e) => {
              e.preventDefault();
              void renameFolderUi(f.id, f.name);
            }}
            title="Double-click to rename"
          >
            <span
              role="button"
              tabIndex={0}
              onClick={(e) => {
                e.stopPropagation();
                if (kids.length) {
                  setExpandedFolders((x) => ({ ...x, [f.id]: !open }));
                }
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.stopPropagation();
                  if (kids.length) {
                    setExpandedFolders((x) => ({ ...x, [f.id]: !open }));
                  }
                }
              }}
              style={{ marginRight: 4, opacity: kids.length ? 1 : 0.3 }}
            >
              {kids.length ? (open ? "▾" : "▸") : "·"}
            </span>
            📂 {f.name}
          </button>
          <button
            type="button"
            className="btn ghost folder-rename"
            title="Rename folder"
            onClick={() => void renameFolderUi(f.id, f.name)}
          >
            ✎
          </button>
          </div>
          {open && renderFolder(f.id, depth + 1)}
        </div>
      );
    });

  const wsLabel =
    wsState === "open"
      ? "live"
      : wsState === "connecting"
        ? "connecting"
        : wsState === "error"
          ? "ws error"
          : "offline";

  const wordCount = useMemo(() => {
    const t = liveText || plainContent();
    const words = t.trim() ? t.trim().split(/\s+/).length : 0;
    return { words, chars: t.length };
  }, [liveText, plainContent]);

  return (
    <>
      <header className="topbar">
        <div className="topbar-left">
          <span className="brand">HelixCollab</span>
          <span className="muted">polished workspace</span>
        </div>
        <div className="topbar-right">
          <button
            type="button"
            className="btn ghost"
            title="Keyboard shortcuts (Ctrl+/)"
            onClick={() => setShowKeys((s) => !s)}
          >
            ⌨
          </button>
          <span className="chip">you: {DEV_USER}</span>
          <span className="chip">
            ws:{" "}
            <span className={wsState === "open" ? "status-ok" : "status-bad"}>
              {wsLabel}
            </span>
          </span>
          <span className="chip">
            mode:{" "}
            <span
              className={
                sealedCrdt
                  ? "status-ok"
                  : crdtOn
                    ? "status-ok"
                    : "status-warn"
              }
            >
              {sealedCrdt
                ? "sealed-crdt"
                : crdtOn
                  ? "yjs/crdt"
                  : "rest+snapshot"}
            </span>
          </span>
          {inbox.length > 0 && (
            <span className="chip live">@{inbox.length} mentions</span>
          )}
          {domain?.durable != null && (
            <span className="chip">
              db:{" "}
              <span className={domain.durable ? "status-ok" : "status-warn"}>
                {domain.durable ? "durable" : "memory"}
              </span>
            </span>
          )}
        </div>
      </header>

      <div className={`layout ${focusMode ? "focus-mode" : ""}`}>
        <aside className="sidebar">
          <div className="panel" style={{ padding: "0.55rem" }}>
            <h3>Workspaces</h3>
            <select
              value={workspaceId ?? ""}
              onChange={(e) => setWorkspaceId(e.target.value || null)}
              style={{
                width: "100%",
                marginBottom: "0.45rem",
                background: "var(--panel-2)",
                color: "var(--text)",
                border: "1px solid var(--border)",
                borderRadius: 8,
                padding: "0.4rem",
              }}
            >
              <option value="">Select workspace…</option>
              {workspaces.map((w) => (
                <option key={w.id} value={w.id}>
                  {w.name}
                </option>
              ))}
            </select>
            <div className="row">
              <input
                type="text"
                placeholder="New workspace"
                value={newWs}
                onChange={(e) => setNewWs(e.target.value)}
                style={{ flex: 1, minWidth: 0 }}
              />
              <button className="btn" type="button" onClick={() => void createWorkspace()}>
                Add
              </button>
            </div>
          </div>

          {workspaceId && (
            <div className="panel" style={{ padding: "0.55rem" }}>
              <h3>Folders</h3>
              <button
                type="button"
                className={`tab ${folderId === null ? "active" : ""}`}
                style={{ width: "100%", marginBottom: 4 }}
                onClick={() => setFolderId(null)}
              >
                📁 Workspace root
              </button>
              {renderFolder(null, 0)}
              <div className="row" style={{ marginTop: 6 }}>
                <input
                  type="text"
                  placeholder={
                    folderId ? "Subfolder name" : "New folder"
                  }
                  value={newFolder}
                  onChange={(e) => setNewFolder(e.target.value)}
                  style={{ flex: 1, minWidth: 0 }}
                />
                <button
                  className="btn"
                  type="button"
                  onClick={() => void createFolder(folderId)}
                >
                  +
                </button>
              </div>
            </div>
          )}

          <div className="row">
            <button className="btn primary" type="button" onClick={() => void createDoc()}>
              New doc
            </button>
            <button
              className="btn"
              type="button"
              title="Create with client-held E2EE"
              onClick={() => void createDoc({ clientE2ee: true })}
            >
              +🔒
            </button>
            <button className="btn" type="button" onClick={() => void loadDocs()}>
              Refresh
            </button>
          </div>
          <input
            className="search"
            type="search"
            placeholder="Filter documents…"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
          />
          <p className="muted" style={{ margin: 0, fontSize: "0.78rem" }}>
            API {API}
          </p>
          <ul className="list">
            {filtered.map((d) => (
              <li key={d.id}>
                <button
                  type="button"
                  className={selectedId === d.id ? "active" : ""}
                  onClick={() => void openDoc(d.id)}
                >
                  <div className="doc-title">
                    {d.pinned ? "📌 " : ""}
                    {d.title}
                    {d.client_e2ee ? " 🔐" : d.encrypted ? " 🔒" : ""}
                  </div>
                  <div className="doc-meta">
                    v{d.version}
                    {d.updated_at ? ` · ${relativeTime(d.updated_at)}` : ""}
                  </div>
                </button>
              </li>
            ))}
            {filtered.length === 0 && (
              <li className="muted" style={{ padding: "0.5rem" }}>
                No documents here
              </li>
            )}
          </ul>
        </aside>

        <main className="main">
          {conflict && (
            <div className="conflict-bar">
              <span>Version conflict — reload latest or save again after merge.</span>
              <button className="btn" type="button" onClick={() => void reloadDoc()}>
                Reload
              </button>
              <button className="btn primary" type="button" onClick={() => void save()}>
                Retry save
              </button>
            </div>
          )}

          {!selectedId ? (
            <div className="empty">
              <h2>Polished collaborative workspace</h2>
              <p>
                Autosave, anchors, e2ee at-rest, folders, @mentions, activity,
                and conflict recovery. Press <kbd>Ctrl</kbd>+<kbd>/</kbd> for
                shortcuts.
              </p>
              <button className="btn primary" type="button" onClick={() => void createDoc()}>
                Create document
              </button>
            </div>
          ) : (
            <>
              <div className="row doc-actions">
                <input
                  className="title-input"
                  type="text"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  aria-label="Document title"
                />
                <button
                  className="btn primary"
                  type="button"
                  disabled={saving || !isDirty}
                  onClick={() => void save()}
                >
                  {saving ? "Saving…" : isDirty ? "Save" : "Saved"}
                </button>
                <button className="btn" type="button" onClick={() => void duplicateDoc()}>
                  Duplicate
                </button>
                <button className="btn" type="button" onClick={() => void copyDocId()}>
                  Copy id
                </button>
                <button className="btn" type="button" onClick={() => void togglePin()}>
                  {pinned ? "Unpin" : "Pin"}
                </button>
                <button
                  className="btn"
                  type="button"
                  disabled={clientE2ee}
                  onClick={() => void toggleE2ee()}
                >
                  {encrypted && !clientE2ee ? "🔓 Vault e2ee" : "🔒 Vault e2ee"}
                </button>
                <button
                  className="btn"
                  type="button"
                  onClick={() =>
                    void (clientE2ee ? disableClientE2ee() : enableClientE2ee())
                  }
                >
                  {clientE2ee ? "🔐 Client E2EE on" : "🔐 Client E2EE"}
                </button>
                {clientE2ee && !locked && (
                  <button className="btn" type="button" onClick={() => void exportClientKey()}>
                    Export key
                  </button>
                )}
                <button className="btn" type="button" onClick={() => void exportBackpack()}>
                  Backpack
                </button>
                <button
                  className="btn"
                  type="button"
                  disabled={locked}
                  onClick={() => void agentSuggest()}
                >
                  Agent
                </button>
                <button className="btn" type="button" onClick={() => void runOfflineMerge()}>
                  Offline merge
                </button>
                <button className="btn" type="button" onClick={() => void bindPasskey()}>
                  Passkey
                </button>
                <button className="btn" type="button" onClick={() => void verifyPasskey()}>
                  Verify key
                </button>
                <button
                  className={`btn ${editorMode === "rich" ? "primary" : ""}`}
                  type="button"
                  onClick={() => setEditorMode("rich")}
                >
                  Rich
                </button>
                <button
                  className={`btn ${editorMode === "markdown" ? "primary" : ""}`}
                  type="button"
                  onClick={() => setEditorMode("markdown")}
                >
                  MD
                </button>
                <button className="btn" type="button" onClick={() => void archiveDoc()}>
                  Archive
                </button>
                <button className="btn" type="button" onClick={captureAnchor}>
                  Comment on selection
                </button>
                <button className="btn danger" type="button" onClick={() => void deleteDoc()}>
                  Delete
                </button>
              </div>

              {locked ? (
                <div className="panel unlock-panel">
                  <h2 style={{ marginTop: 0 }}>🔐 Unlock client E2EE</h2>
                  <p className="muted">
                    Keys never leave this browser. Enter passphrase
                    {hasWrappedKey(selectedId) ? "" : " to import a shared key"}.
                  </p>
                  <label className="muted" htmlFor="pass">
                    Passphrase
                  </label>
                  <input
                    id="pass"
                    type="password"
                    value={passphrase}
                    onChange={(e) => setPassphrase(e.target.value)}
                    placeholder="••••••••"
                  />
                  <label className="muted" htmlFor="dek">
                    Import raw DEK (optional)
                  </label>
                  <input
                    id="dek"
                    type="text"
                    value={importDek}
                    onChange={(e) => setImportDek(e.target.value)}
                    placeholder="base64url DEK from collaborator"
                  />
                  <button
                    className="btn primary"
                    type="button"
                    disabled={passphrase.length < 8}
                    onClick={() => void unlockDoc()}
                  >
                    Unlock
                  </button>
                </div>
              ) : (
              <div className="editor-wrap">
                <EditorToolbar
                  preview={preview}
                  focusMode={focusMode}
                  onTogglePreview={() => setPreview((p) => !p)}
                  onToggleFocus={() => setFocusMode((f) => !f)}
                  onSave={() => void save()}
                  onExport={exportMd}
                  onWrap={(b, a) => editorRef.current?.wrapSelection(b, a)}
                  onInsert={(t) => editorRef.current?.insertAtCursor(t)}
                />
                <div className={`editor-split ${preview ? "" : "single"}`}>
                  {editorMode === "rich" ? (
                    <ProseMirrorEditor
                      key={`${selectedId}-pm`}
                      ref={editorRef as RefObject<ProseMirrorHandle>}
                      className="editor"
                      ydoc={provider && !locked ? provider.ydoc : null}
                      collab={!!provider && crdtOn && !locked}
                      initialMarkdown={fallbackContent}
                      onCursor={(pos) => provider?.sendPresence(pos)}
                      onChange={onEditorChange}
                    />
                  ) : provider && !clientE2ee ? (
                    <YTextEditor
                      key={`${selectedId}-yt`}
                      ref={editorRef as RefObject<YTextEditorHandle>}
                      className="editor"
                      ytext={provider.ytext}
                      onCursor={(pos) => provider.sendPresence(pos)}
                      onChange={onEditorChange}
                    />
                  ) : (
                    <textarea
                      className="editor"
                      value={fallbackContent}
                      onChange={(e) => {
                        setFallbackContent(e.target.value);
                        onEditorChange(e.target.value);
                      }}
                      spellCheck={false}
                    />
                  )}
                  {preview && (
                    <MarkdownPreview source={liveText || plainContent()} />
                  )}
                </div>
                <div className="statusbar">
                  <span
                    className={
                      saving
                        ? "status-warn"
                        : isDirty
                          ? "status-warn"
                          : "status-ok"
                    }
                  >
                    {saving
                      ? "Saving…"
                      : isDirty
                        ? "Unsaved changes"
                        : "All changes saved"}
                  </span>
                  <span
                    className="muted"
                    title={selectedId}
                    style={{ cursor: "pointer" }}
                    onClick={() => void copyDocId()}
                  >
                    id {selectedId.slice(0, 8)}…
                  </span>
                  <span>version {version}</span>
                  {clientE2ee && (
                    <span className="status-ok" title="Client-held keys">
                      client-e2ee
                    </span>
                  )}
                  {sealedCrdt && (
                    <span className="status-ok" title="Encrypted Yjs over WS">
                      sealed-crdt
                    </span>
                  )}
                  {encrypted && !clientE2ee && (
                    <span className="status-ok">vault-e2ee</span>
                  )}
                  <span className="muted">
                    {editorMode === "rich" ? "prosemirror" : "markdown"}
                  </span>
                  {pinned && <span>📌 pinned</span>}
                  <span>
                    {wordCount.words} words · {wordCount.chars} chars
                  </span>
                  <span className="peers">
                    {peers.length === 0 ? (
                      <span className="chip">solo</span>
                    ) : (
                      peers.map((p) => (
                        <span className="chip live" key={p.user_id}>
                          {p.display_name || p.user_id.slice(0, 8)}
                        </span>
                      ))
                    )}
                    {Object.values(typing).map((t) => (
                      <span className="chip" key={t.display_name}>
                        {t.display_name} typing…
                      </span>
                    ))}
                  </span>
                </div>
              </div>
              )}
            </>
          )}
        </main>

        <aside className="rail">
          <div className="tabs">
            {(
              [
                ["comments", "Comments"],
                ["activity", "Activity"],
                ["people", "People"],
                ["share", "Share"],
                ["history", "History"],
              ] as const
            ).map(([k, label]) => (
              <button
                key={k}
                type="button"
                className={`tab ${railTab === k ? "active" : ""}`}
                onClick={() => setRailTab(k)}
              >
                {label}
              </button>
            ))}
          </div>

          {railTab === "comments" && (
            <div className="panel" style={{ position: "relative" }}>
              <div className="row" style={{ justifyContent: "space-between" }}>
                <h3 style={{ margin: 0 }}>Discussion</h3>
                <div className="tabs compact">
                  <button
                    type="button"
                    className={`tab ${commentFilter === "open" ? "active" : ""}`}
                    onClick={() => setCommentFilter("open")}
                  >
                    Open
                  </button>
                  <button
                    type="button"
                    className={`tab ${commentFilter === "all" ? "active" : ""}`}
                    onClick={() => setCommentFilter("all")}
                  >
                    All
                  </button>
                </div>
              </div>
              <div className="rev-list" style={{ maxHeight: 240 }}>
                {visibleComments.length === 0 && (
                  <span className="muted">
                    {comments.length === 0
                      ? "No comments yet"
                      : "No open threads"}
                  </span>
                )}
                {visibleComments.map((c) => (
                  <div
                    key={c.id}
                    className="rev-item"
                    style={{
                      flexDirection: "column",
                      alignItems: "stretch",
                      opacity: c.resolved_at ? 0.55 : 1,
                    }}
                  >
                    <div className="row" style={{ justifyContent: "space-between" }}>
                      <strong style={{ fontSize: "0.85rem" }}>
                        {c.author_label}
                        {c.resolved_at ? " · resolved" : ""}
                      </strong>
                      <span
                        className="muted"
                        style={{ fontSize: "0.7rem" }}
                        title={new Date(c.created_at).toLocaleString()}
                      >
                        {relativeTime(c.created_at)}
                      </span>
                    </div>
                    {c.anchor_quote ? (
                      <button
                        type="button"
                        className="anchor-quote"
                        onClick={() => jumpToAnchor(c)}
                        title="Jump to selection in editor"
                      >
                        “{c.anchor_quote}”
                        {typeof c.anchor_start === "number"
                          ? ` @${c.anchor_start}`
                          : ""}
                      </button>
                    ) : null}
                    {editingCommentId === c.id ? (
                      <div>
                        <textarea
                          value={editingCommentBody}
                          onChange={(e) => setEditingCommentBody(e.target.value)}
                          style={{
                            width: "100%",
                            minHeight: 56,
                            background: "var(--panel-2)",
                            border: "1px solid var(--border)",
                            borderRadius: 8,
                            color: "var(--text)",
                            padding: 6,
                            font: "inherit",
                            fontSize: "0.85rem",
                          }}
                        />
                        <div className="row" style={{ marginTop: 4 }}>
                          <button
                            className="btn primary"
                            type="button"
                            style={{ fontSize: "0.75rem" }}
                            onClick={() => void saveCommentEdit(c.id)}
                          >
                            Save
                          </button>
                          <button
                            className="btn ghost"
                            type="button"
                            style={{ fontSize: "0.75rem" }}
                            onClick={() => setEditingCommentId(null)}
                          >
                            Cancel
                          </button>
                        </div>
                      </div>
                    ) : (
                      <div style={{ fontSize: "0.88rem", whiteSpace: "pre-wrap" }}>
                        {c.body}
                      </div>
                    )}
                    {c.mentions?.length > 0 && (
                      <div className="peers">
                        {c.mentions.map((m) => (
                          <span className="chip live" key={m.id}>
                            @{m.mentioned_label}
                          </span>
                        ))}
                      </div>
                    )}
                    <div className="row comment-actions">
                      {typeof c.anchor_start === "number" && (
                        <button
                          className="btn ghost"
                          type="button"
                          style={{ fontSize: "0.75rem" }}
                          onClick={() => jumpToAnchor(c)}
                        >
                          Jump
                        </button>
                      )}
                      <button
                        className="btn ghost"
                        type="button"
                        style={{ fontSize: "0.75rem" }}
                        onClick={() => {
                          setEditingCommentId(c.id);
                          setEditingCommentBody(c.body);
                        }}
                      >
                        Edit
                      </button>
                      <button
                        className="btn ghost"
                        type="button"
                        style={{ fontSize: "0.75rem" }}
                        onClick={() =>
                          void resolveComment(c.id, !c.resolved_at)
                        }
                      >
                        {c.resolved_at ? "Reopen" : "Resolve"}
                      </button>
                    </div>
                  </div>
                ))}
              </div>
              {anchorSel && (
                <div className="banner info" style={{ marginTop: 8 }}>
                  Anchored: “{anchorSel.quote.slice(0, 80)}
                  {anchorSel.quote.length > 80 ? "…" : ""}”{" "}
                  <button
                    type="button"
                    className="btn ghost"
                    onClick={() => setAnchorSel(null)}
                  >
                    Clear
                  </button>
                </div>
              )}
              <div style={{ position: "relative" }}>
                <textarea
                  value={commentBody}
                  onChange={(e) => setCommentBody(e.target.value)}
                  placeholder="Comment… type @ for mentions"
                  style={{
                    width: "100%",
                    minHeight: 72,
                    marginTop: 8,
                    background: "var(--panel-2)",
                    border: "1px solid var(--border)",
                    borderRadius: 8,
                    color: "var(--text)",
                    padding: 8,
                    font: "inherit",
                  }}
                  disabled={!selectedId}
                />
                {showSuggest && (
                  <div className="suggest" style={{ bottom: 88, left: 0, right: 0 }}>
                    {filteredSuggest.map((s) => (
                      <button
                        key={s}
                        type="button"
                        onClick={() => applyMention(s)}
                      >
                        @{s}
                      </button>
                    ))}
                  </div>
                )}
              </div>
              <button
                className="btn primary"
                type="button"
                style={{ marginTop: 6, width: "100%" }}
                disabled={!selectedId || !commentBody.trim()}
                onClick={() => void postComment()}
              >
                Post comment
              </button>
              {inbox.length > 0 && (
                <>
                  <h3 style={{ marginTop: 12 }}>Your mentions</h3>
                  <div className="peers">
                    {inbox.slice(0, 8).map((m) => (
                      <button
                        key={m.id}
                        type="button"
                        className="chip live"
                        onClick={() => void openDoc(m.document_id)}
                      >
                        @{m.mentioned_label} · {m.document_id.slice(0, 6)}…
                      </button>
                    ))}
                  </div>
                </>
              )}
            </div>
          )}

          {railTab === "activity" && (
            <div className="panel">
              <h3>Activity</h3>
              <div className="rev-list" style={{ maxHeight: 360 }}>
                {activity.length === 0 && (
                  <span className="muted">No activity yet</span>
                )}
                {activity.map((a) => (
                  <div key={a.id} className="rev-item" style={{ flexDirection: "column", alignItems: "stretch" }}>
                    <div className="row" style={{ justifyContent: "space-between" }}>
                      <span style={{ fontSize: "0.82rem" }}>
                        {activityLabel(a.action)}
                      </span>
                      <span
                        className="muted"
                        style={{ fontSize: "0.7rem" }}
                        title={new Date(a.created_at).toLocaleString()}
                      >
                        {relativeTime(a.created_at)}
                      </span>
                    </div>
                    <span className="muted" style={{ fontSize: "0.78rem" }}>
                      {a.actor_label || "system"}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {railTab === "people" && (
            <div className="panel">
              <h3>Presence</h3>
              <div className="peers">
                {peers.length === 0 && (
                  <span className="muted">No other peers in room</span>
                )}
                {peers.map((p) => (
                  <div key={p.user_id} className="chip live">
                    {p.display_name || p.user_id.slice(0, 10)}
                    <span className="muted"> · cursor {p.cursor_pos}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {railTab === "share" && (
            <div className="panel">
              <h3>Invite</h3>
              <div className="row" style={{ marginBottom: "0.65rem" }}>
                <input
                  type="text"
                  placeholder="principal_id / user UUID"
                  value={shareId}
                  onChange={(e) => setShareId(e.target.value)}
                  style={{ flex: 1, minWidth: 0 }}
                />
                <button
                  className="btn primary"
                  type="button"
                  disabled={!selectedId}
                  onClick={() => void shareDoc()}
                >
                  Invite
                </button>
              </div>
              <h3>ACL</h3>
              <div className="peers">
                {aclItems.length === 0 && (
                  <span className="muted">No ACL rows</span>
                )}
                {aclItems.map((a) => (
                  <span
                    className="chip"
                    key={`${a.principal_kind}:${a.principal_id}`}
                  >
                    {a.principal_kind}:{a.principal_id.slice(0, 14)} ·{" "}
                    {a.permissions.join(",")}
                  </span>
                ))}
              </div>
              <h3 style={{ marginTop: 12 }}>Attachments</h3>
              <div className="rev-list" style={{ maxHeight: 160 }}>
                {attachments.length === 0 && (
                  <span className="muted">No files</span>
                )}
                {attachments.map((a) => (
                  <div
                    key={a.id}
                    className="rev-item"
                    style={{ flexDirection: "column", alignItems: "stretch" }}
                  >
                    <div className="row" style={{ justifyContent: "space-between" }}>
                      <span style={{ fontSize: "0.82rem" }}>
                        {a.filename}
                        {a.client_sealed ? " 🔐" : ""}
                      </span>
                      <span className="muted" style={{ fontSize: "0.7rem" }}>
                        {a.size_bytes} B
                      </span>
                    </div>
                    <div className="row" style={{ justifyContent: "flex-end", gap: 4 }}>
                      <button
                        type="button"
                        className="btn ghost"
                        style={{ fontSize: "0.72rem" }}
                        onClick={() => {
                          void (async () => {
                            if (!selectedId) return;
                            try {
                              const res = await api<{
                                data_b64: string;
                                filename: string;
                              }>(
                                `/v1/documents/${selectedId}/attachments/${a.id}/body`,
                              );
                              const bin = atob(res.data_b64);
                              const bytes = new Uint8Array(bin.length);
                              for (let i = 0; i < bin.length; i++)
                                bytes[i] = bin.charCodeAt(i);
                              const blob = new Blob([bytes]);
                              const url = URL.createObjectURL(blob);
                              const el = document.createElement("a");
                              el.href = url;
                              el.download = res.filename || a.filename;
                              el.click();
                              URL.revokeObjectURL(url);
                              toast("Downloaded attachment", "ok");
                            } catch (e) {
                              toast(String(e), "error");
                            }
                          })();
                        }}
                      >
                        Download
                      </button>
                      <button
                        type="button"
                        className="btn danger"
                        style={{ fontSize: "0.72rem" }}
                        onClick={() => {
                          void (async () => {
                            if (!selectedId) return;
                            if (!confirm(`Delete ${a.filename}?`)) return;
                            try {
                              await api(
                                `/v1/documents/${selectedId}/attachments/${a.id}`,
                                { method: "DELETE" },
                              );
                              setAttachments((prev) =>
                                prev.filter((x) => x.id !== a.id),
                              );
                              toast("Attachment deleted", "ok");
                            } catch (e) {
                              toast(String(e), "error");
                            }
                          })();
                        }}
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                ))}
              </div>
              <label className="btn" style={{ marginTop: 8, display: "inline-block" }}>
                Upload
                <input
                  type="file"
                  style={{ display: "none" }}
                  disabled={!selectedId}
                  onChange={(e) => {
                    const file = e.target.files?.[0];
                    e.target.value = "";
                    if (!file || !selectedId) return;
                    void (async () => {
                      try {
                        const buf = new Uint8Array(await file.arrayBuffer());
                        let binary = "";
                        for (let i = 0; i < buf.length; i++)
                          binary += String.fromCharCode(buf[i]!);
                        const data_b64 = btoa(binary);
                        const sealed = clientE2ee;
                        const res = await api<{ attachment: Attachment }>(
                          `/v1/documents/${selectedId}/attachments/upload`,
                          {
                            method: "POST",
                            body: JSON.stringify({
                              filename: file.name,
                              content_type: file.type || "application/octet-stream",
                              data_b64,
                              client_sealed: sealed,
                            }),
                          },
                        );
                        setAttachments((prev) => [res.attachment, ...prev]);
                        toast(
                          sealed
                            ? "Uploaded (flagged sealed)"
                            : "Uploaded to MinIO",
                          "ok",
                        );
                      } catch (err) {
                        toast(String(err), "error");
                      }
                    })();
                  }}
                />
              </label>
              {selectedId && childrenOf(null).length > 0 && (
                <>
                  <h3 style={{ marginTop: 12 }}>Move</h3>
                  <select
                    value=""
                    onChange={(e) => {
                      if (e.target.value === "__root") void moveToFolder(null);
                      else if (e.target.value) void moveToFolder(e.target.value);
                    }}
                    style={{
                      width: "100%",
                      background: "var(--panel-2)",
                      color: "var(--text)",
                      border: "1px solid var(--border)",
                      borderRadius: 8,
                      padding: "0.4rem",
                    }}
                  >
                    <option value="">Move to…</option>
                    <option value="__root">Workspace root</option>
                    {folders.map((f) => (
                      <option key={f.id} value={f.id}>
                        {f.name}
                      </option>
                    ))}
                  </select>
                </>
              )}
            </div>
          )}

          {railTab === "history" && (
            <div className="panel">
              <h3>Revisions</h3>
              <div className="rev-list">
                {revisions.length === 0 && (
                  <span className="muted">No revisions loaded</span>
                )}
                {revisions.map((r) => (
                  <div className="rev-item" key={r.id}>
                    <div>
                      <div>v{r.version}</div>
                      <div className="muted" style={{ fontSize: "0.72rem" }}>
                        {new Date(r.created_at).toLocaleString()}
                      </div>
                    </div>
                    <button
                      className="btn ghost"
                      type="button"
                      onClick={() => void restoreRev(r.version)}
                    >
                      Restore
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}
        </aside>
      </div>

      <div className="toast-stack" aria-live="polite">
        {toasts.map((t) => (
          <div key={t.id} className={`toast ${t.kind === "error" ? "error" : t.kind === "ok" ? "ok" : ""}`}>
            {t.text}
          </div>
        ))}
      </div>

      {showKeys && (
        <div
          className="keys-overlay"
          role="dialog"
          aria-label="Keyboard shortcuts"
          onClick={() => setShowKeys(false)}
        >
          <div className="keys-panel" onClick={(e) => e.stopPropagation()}>
            <div className="row" style={{ justifyContent: "space-between" }}>
              <h3 style={{ margin: 0 }}>Shortcuts</h3>
              <button
                type="button"
                className="btn ghost"
                onClick={() => setShowKeys(false)}
              >
                Esc
              </button>
            </div>
            <ul className="keys-list">
              <li>
                <kbd>Ctrl</kbd>+<kbd>S</kbd> Save
              </li>
              <li>
                <kbd>Ctrl</kbd>+<kbd>B</kbd> / <kbd>I</kbd> / <kbd>E</kbd> Bold /
                italic / code
              </li>
              <li>
                <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>P</kbd> Toggle preview
              </li>
              <li>
                <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>F</kbd> Focus mode
              </li>
              <li>
                <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>M</kbd> Comment on selection
              </li>
              <li>
                <kbd>Ctrl</kbd>+<kbd>/</kbd> This help
              </li>
              <li>
                <kbd>Esc</kbd> Exit focus / close dialogs
              </li>
            </ul>
            <p className="muted" style={{ fontSize: "0.8rem", marginBottom: 0 }}>
              Content autosaves after ~1.6s of idle changes.
            </p>
          </div>
        </div>
      )}
    </>
  );
}
