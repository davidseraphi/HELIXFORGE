"use client";

import type { MouseEvent } from "react";
import { MonacoPane } from "@/components/MonacoPane";
import type { LspDiagnostic } from "@/lib/api";
import type { EditorGroup, EditorTab, GroupId } from "@/lib/tabs";
import { getDoc, isDirty } from "@/lib/tabs";

type Props = {
  group: EditorGroup;
  docs: EditorTab[];
  focused: boolean;
  repoName?: string;
  busy?: boolean;
  diagnostics: LspDiagnostic[];
  lspSessionId: string | null;
  split?: boolean;
  onFocus: () => void;
  onSelectTab: (path: string) => void;
  onCloseTab: (path: string, e?: MouseEvent) => void;
  onChange: (path: string, content: string) => void;
  onCursor?: (path: string, line: number, character: number) => void;
  onGoTo?: (path: string) => void;
  onSplit?: () => void;
  onUnsplit?: () => void;
  onMoveToOther?: (path: string) => void;
};

export function EditorGroupPane({
  group,
  docs,
  focused,
  repoName,
  busy,
  diagnostics,
  lspSessionId,
  split,
  onFocus,
  onSelectTab,
  onCloseTab,
  onChange,
  onCursor,
  onGoTo,
  onSplit,
  onUnsplit,
  onMoveToOther,
}: Props) {
  const active = group.activePath
    ? getDoc(docs, group.activePath)
    : undefined;
  const crumbs = (group.activePath ?? "").split("/").filter(Boolean);
  const fileDiags = diagnostics.filter(
    (d) =>
      !group.activePath ||
      d.path === group.activePath ||
      d.path.endsWith(group.activePath),
  );

  return (
    <div
      className={`editor-group${focused ? " focused" : ""}`}
      onMouseDown={onFocus}
      data-group={group.id}
    >
      <div className="tabstrip">
        {group.tabPaths.length === 0 && (
          <span className="empty" style={{ padding: "0.35rem 0.65rem" }}>
            Empty group
          </span>
        )}
        {group.tabPaths.map((path) => {
          const doc = getDoc(docs, path);
          return (
            <button
              key={path}
              type="button"
              className={`tab${path === group.activePath ? " active" : ""}`}
              onClick={() => onSelectTab(path)}
              title={path}
            >
              {doc && isDirty(doc) && <span className="dot" />}
              <span style={{ overflow: "hidden", textOverflow: "ellipsis" }}>
                {path.split("/").pop()}
              </span>
              <span
                className="x"
                role="button"
                tabIndex={0}
                onClick={(e) => onCloseTab(path, e)}
                onKeyDown={(e) => e.key === "Enter" && onCloseTab(path)}
              >
                ×
              </span>
            </button>
          );
        })}
        <span className="tabstrip-actions">
          {onMoveToOther && group.activePath && (
            <button
              type="button"
              title="Move tab to other group"
              onClick={() =>
                group.activePath && onMoveToOther(group.activePath)
              }
            >
              ⇄
            </button>
          )}
          {onSplit && !split && (
            <button type="button" title="Split editor (Ctrl+\\)" onClick={onSplit}>
              ⊞
            </button>
          )}
          {onUnsplit && split && group.id === "secondary" && (
            <button type="button" title="Close split" onClick={onUnsplit}>
              ✕
            </button>
          )}
        </span>
      </div>
      <div className="breadcrumbs">
        <span className="group-label">{labelFor(group.id)}</span>
        {repoName && <span>{repoName}</span>}
        {crumbs.map((c, i) => (
          <span key={`${c}-${i}`}>
            <span className="sep">›</span> {c}
          </span>
        ))}
        {!group.activePath && <span className="sep">—</span>}
      </div>
      {active ? (
        <MonacoPane
          path={active.path}
          value={active.content}
          onChange={(v) => onChange(active.path, v)}
          readOnly={!!busy}
          diagnostics={fileDiags}
          lspSessionId={lspSessionId}
          onCursor={(line, ch) => onCursor?.(active.path, line, ch)}
          onGoTo={(p) => onGoTo?.(p)}
        />
      ) : (
        <div className="empty">
          Open a file into this group.{" "}
          {group.id === "primary" && (
            <>
              Split with <kbd>Ctrl+\</kbd>
            </>
          )}
        </div>
      )}
    </div>
  );
}

function labelFor(id: GroupId): string {
  return id === "primary" ? "1" : "2";
}
