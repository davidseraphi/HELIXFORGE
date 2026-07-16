export type EditorTab = {
  path: string;
  content: string;
  savedContent: string;
};

export type GroupId = "primary" | "secondary";

export type EditorGroup = {
  id: GroupId;
  /** Ordered open paths in this group (subset of docs). */
  tabPaths: string[];
  activePath: string | null;
};

export function isDirty(t: EditorTab): boolean {
  return t.content !== t.savedContent;
}

export function getDoc(docs: EditorTab[], path: string): EditorTab | undefined {
  return docs.find((t) => t.path === path);
}

export function upsertDoc(
  docs: EditorTab[],
  path: string,
  content: string,
  markSaved = true,
): EditorTab[] {
  const idx = docs.findIndex((t) => t.path === path);
  if (idx >= 0) {
    const next = [...docs];
    next[idx] = {
      ...next[idx],
      content,
      savedContent: markSaved ? content : next[idx].savedContent,
    };
    return next;
  }
  return [
    ...docs,
    {
      path,
      content,
      savedContent: content,
    },
  ];
}

export function updateDocContent(
  docs: EditorTab[],
  path: string,
  content: string,
): EditorTab[] {
  return docs.map((t) => (t.path === path ? { ...t, content } : t));
}

export function markDocSaved(
  docs: EditorTab[],
  path: string,
  content?: string,
): EditorTab[] {
  return docs.map((t) =>
    t.path === path
      ? {
          ...t,
          content: content ?? t.content,
          savedContent: content ?? t.content,
        }
      : t,
  );
}

export function markAllDocsSaved(docs: EditorTab[]): EditorTab[] {
  return docs.map((t) => ({ ...t, savedContent: t.content }));
}

export function dirtyDocs(docs: EditorTab[]): EditorTab[] {
  return docs.filter(isDirty);
}

export function emptyGroup(id: GroupId): EditorGroup {
  return { id, tabPaths: [], activePath: null };
}

/** Open path in a group (adds tab if missing). */
export function openInGroup(group: EditorGroup, path: string): EditorGroup {
  const tabPaths = group.tabPaths.includes(path)
    ? group.tabPaths
    : [...group.tabPaths, path];
  return { ...group, tabPaths, activePath: path };
}

/** Close path in one group; returns next group state. */
export function closeInGroup(
  group: EditorGroup,
  path: string,
): EditorGroup {
  const idx = group.tabPaths.indexOf(path);
  if (idx < 0) return group;
  const tabPaths = group.tabPaths.filter((p) => p !== path);
  let activePath = group.activePath;
  if (activePath === path) {
    activePath =
      tabPaths[Math.min(idx, tabPaths.length - 1)] ?? tabPaths[0] ?? null;
  }
  return { ...group, tabPaths, activePath };
}

/** True if path is still open in either group. */
export function pathStillOpen(
  primary: EditorGroup,
  secondary: EditorGroup | null,
  path: string,
): boolean {
  if (primary.tabPaths.includes(path)) return true;
  if (secondary?.tabPaths.includes(path)) return true;
  return false;
}

export function removeDoc(docs: EditorTab[], path: string): EditorTab[] {
  return docs.filter((t) => t.path !== path);
}

/** Move path from one group to the other (open there, close here). */
export function moveTabBetweenGroups(
  from: EditorGroup,
  to: EditorGroup,
  path: string,
): { from: EditorGroup; to: EditorGroup } {
  if (!from.tabPaths.includes(path)) {
    return { from, to: openInGroup(to, path) };
  }
  return {
    from: closeInGroup(from, path),
    to: openInGroup(to, path),
  };
}

export function filterFiles(
  files: string[],
  query: string,
  limit = 40,
): string[] {
  const q = query.trim().toLowerCase();
  if (!q) return files.slice(0, limit);
  const scored = files
    .map((f) => {
      const lower = f.toLowerCase();
      const base = f.split("/").pop()?.toLowerCase() ?? lower;
      let score = 0;
      if (base.startsWith(q)) score = 100;
      else if (base.includes(q)) score = 50;
      else if (lower.includes(q)) score = 10;
      else return null;
      score -= f.length * 0.01;
      return { f, score };
    })
    .filter((x): x is { f: string; score: number } => x != null)
    .sort((a, b) => b.score - a.score);
  return scored.slice(0, limit).map((s) => s.f);
}

// —— legacy aliases used by older imports (kept for safety) ——
export const upsertTab = upsertDoc;
export const updateTabContent = updateDocContent;
export const markTabSaved = markDocSaved;
export const markAllSaved = markAllDocsSaved;
export const dirtyTabs = dirtyDocs;

export function closeTab(
  tabs: EditorTab[],
  path: string,
): { tabs: EditorTab[]; nextActive: string | null } {
  const idx = tabs.findIndex((t) => t.path === path);
  if (idx < 0) return { tabs, nextActive: null };
  const next = tabs.filter((t) => t.path !== path);
  const nextActive =
    next[Math.min(idx, next.length - 1)]?.path ?? next[0]?.path ?? null;
  return { tabs: next, nextActive };
}
