"use client";

type Props = {
  onWrap: (before: string, after?: string) => void;
  onInsert: (text: string) => void;
  onSave: () => void;
  onTogglePreview: () => void;
  onToggleFocus: () => void;
  preview: boolean;
  focusMode: boolean;
  onExport: () => void;
};

export function EditorToolbar({
  onWrap,
  onInsert,
  onSave,
  onTogglePreview,
  onToggleFocus,
  preview,
  focusMode,
  onExport,
}: Props) {
  return (
    <div className="toolbar" role="toolbar" aria-label="Markdown tools">
      <button type="button" className="tbtn" title="Bold (Ctrl+B)" onClick={() => onWrap("**")}>
        B
      </button>
      <button type="button" className="tbtn" title="Italic (Ctrl+I)" onClick={() => onWrap("*")}>
        <em>I</em>
      </button>
      <button type="button" className="tbtn" title="Code (Ctrl+E)" onClick={() => onWrap("`")}>
        {"</>"}
      </button>
      <button type="button" className="tbtn" title="Heading" onClick={() => onInsert("\n## ")}>
        H2
      </button>
      <button type="button" className="tbtn" title="Bullet list" onClick={() => onInsert("\n- ")}>
        • List
      </button>
      <button
        type="button"
        className="tbtn"
        title="Link"
        onClick={() => onWrap("[", "](https://)")}
      >
        Link
      </button>
      <button type="button" className="tbtn" title="Mention" onClick={() => onInsert("@")}>
        @
      </button>
      <span className="tsep" />
      <button
        type="button"
        className={`tbtn ${preview ? "on" : ""}`}
        title="Toggle preview"
        onClick={onTogglePreview}
      >
        Preview
      </button>
      <button
        type="button"
        className={`tbtn ${focusMode ? "on" : ""}`}
        title="Focus mode"
        onClick={onToggleFocus}
      >
        Focus
      </button>
      <button type="button" className="tbtn" title="Export .md" onClick={onExport}>
        Export
      </button>
      <button type="button" className="tbtn primary" title="Save (Ctrl+S)" onClick={onSave}>
        Save
      </button>
    </div>
  );
}
