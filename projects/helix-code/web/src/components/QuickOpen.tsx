"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { filterFiles } from "@/lib/tabs";

type Props = {
  open: boolean;
  files: string[];
  onClose: () => void;
  onPick: (path: string) => void;
};

export function QuickOpen({ open, files, onClose, onPick }: Props) {
  const [q, setQ] = useState("");
  const [sel, setSel] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const filtered = useMemo(() => filterFiles(files, q, 50), [files, q]);

  useEffect(() => {
    if (open) {
      setQ("");
      setSel(0);
      window.setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [open]);

  useEffect(() => setSel(0), [q]);

  if (!open) return null;

  function pick(i: number) {
    const path = filtered[i];
    if (!path) return;
    onClose();
    onPick(path);
  }

  return (
    <div className="modal-backdrop" onClick={onClose} role="presentation">
      <div
        className="palette"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-label="Quick open"
      >
        <input
          ref={inputRef}
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder="Go to file…"
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
              pick(sel);
            }
          }}
        />
        <div className="palette-list">
          {filtered.length === 0 && (
            <div className="palette-empty">No files match</div>
          )}
          {filtered.map((f, i) => (
            <button
              key={f}
              type="button"
              className={`palette-item${i === sel ? " active" : ""}`}
              onMouseEnter={() => setSel(i)}
              onClick={() => pick(i)}
            >
              <span className="mono">{f}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
