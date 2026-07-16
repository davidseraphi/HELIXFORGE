"use client";

import { useEffect, useMemo, useRef, useState } from "react";

export type PaletteCommand = {
  id: string;
  label: string;
  hint?: string;
  run: () => void;
};

type Props = {
  open: boolean;
  commands: PaletteCommand[];
  onClose: () => void;
};

export function CommandPalette({ open, commands, onClose }: Props) {
  const [q, setQ] = useState("");
  const [sel, setSel] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const filtered = useMemo(() => {
    const needle = q.trim().toLowerCase();
    if (!needle) return commands;
    return commands.filter(
      (c) =>
        c.label.toLowerCase().includes(needle) ||
        c.id.toLowerCase().includes(needle) ||
        (c.hint ?? "").toLowerCase().includes(needle),
    );
  }, [commands, q]);

  useEffect(() => {
    if (open) {
      setQ("");
      setSel(0);
      window.setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [open]);

  useEffect(() => {
    setSel(0);
  }, [q]);

  if (!open) return null;

  function activate(i: number) {
    const cmd = filtered[i];
    if (!cmd) return;
    onClose();
    cmd.run();
  }

  return (
    <div className="modal-backdrop" onClick={onClose} role="presentation">
      <div
        className="palette"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-label="Command palette"
      >
        <input
          ref={inputRef}
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder="Type a command…"
          onKeyDown={(e) => {
            if (e.key === "Escape") onClose();
            if (e.key === "ArrowDown") {
              e.preventDefault();
              setSel((s) => Math.min(s + 1, filtered.length - 1));
            }
            if (e.key === "ArrowUp") {
              e.preventDefault();
              setSel((s) => Math.max(s - 1, 0));
            }
            if (e.key === "Enter") {
              e.preventDefault();
              activate(sel);
            }
          }}
        />
        <div className="palette-list">
          {filtered.length === 0 && (
            <div className="palette-empty">No matching commands</div>
          )}
          {filtered.map((c, i) => (
            <button
              key={c.id}
              type="button"
              className={`palette-item${i === sel ? " active" : ""}`}
              onMouseEnter={() => setSel(i)}
              onClick={() => activate(i)}
            >
              <span>{c.label}</span>
              {c.hint && <span className="hint">{c.hint}</span>}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
