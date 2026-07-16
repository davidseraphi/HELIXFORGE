"use client";

import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
} from "react";
import type * as Y from "yjs";

export type YTextEditorHandle = {
  wrapSelection: (before: string, after?: string) => void;
  insertAtCursor: (text: string) => void;
  focus: () => void;
  getSelection: () => { start: number; end: number };
  /** Jump caret / highlight a range (e.g. comment anchors). */
  setSelection: (start: number, end: number) => void;
};

type Props = {
  ytext: Y.Text;
  className?: string;
  onCursor?: (pos: number) => void;
  onChange?: (text: string) => void;
  readOnly?: boolean;
};

/**
 * Collaborative plain/markdown editor bound to Y.Text.
 */
export const YTextEditor = forwardRef<YTextEditorHandle, Props>(
  function YTextEditor({ ytext, className, onCursor, onChange, readOnly }, ref) {
    const elRef = useRef<HTMLTextAreaElement>(null);
    const remote = useRef(false);

    useImperativeHandle(ref, () => ({
      wrapSelection(before, after = before) {
        const el = elRef.current;
        if (!el || readOnly) return;
        const start = el.selectionStart;
        const end = el.selectionEnd;
        const full = ytext.toString();
        const selected = full.slice(start, end) || "text";
        const next =
          full.slice(0, start) + before + selected + after + full.slice(end);
        ytext.doc?.transact(() => {
          ytext.delete(0, ytext.length);
          ytext.insert(0, next);
        });
        requestAnimationFrame(() => {
          const pos = start + before.length + selected.length + after.length;
          el.focus();
          el.setSelectionRange(start + before.length, pos - after.length);
        });
      },
      insertAtCursor(text) {
        const el = elRef.current;
        if (!el || readOnly) return;
        const start = el.selectionStart;
        const end = el.selectionEnd;
        const full = ytext.toString();
        const next = full.slice(0, start) + text + full.slice(end);
        ytext.doc?.transact(() => {
          ytext.delete(0, ytext.length);
          ytext.insert(0, next);
        });
        requestAnimationFrame(() => {
          const pos = start + text.length;
          el.focus();
          el.setSelectionRange(pos, pos);
        });
      },
      focus() {
        elRef.current?.focus();
      },
      getSelection() {
        const el = elRef.current;
        return {
          start: el?.selectionStart ?? 0,
          end: el?.selectionEnd ?? 0,
        };
      },
      setSelection(start, end) {
        const el = elRef.current;
        if (!el) return;
        const len = el.value.length;
        const s = Math.max(0, Math.min(start, len));
        const e = Math.max(s, Math.min(end, len));
        el.focus();
        el.setSelectionRange(s, e);
        // scroll selection into view (best-effort)
        try {
          const lineH = 24;
          const before = el.value.slice(0, s);
          const lines = before.split("\n").length;
          el.scrollTop = Math.max(0, (lines - 3) * lineH);
        } catch {
          /* ignore */
        }
      },
    }));

    useEffect(() => {
      const el = elRef.current;
      if (!el) return;

      const syncFromY = () => {
        const next = ytext.toString();
        if (el.value !== next) {
          const start = el.selectionStart;
          const end = el.selectionEnd;
          remote.current = true;
          el.value = next;
          try {
            el.setSelectionRange(
              Math.min(start, next.length),
              Math.min(end, next.length),
            );
          } catch {
            /* ignore */
          }
          remote.current = false;
          onChange?.(next);
        }
      };

      syncFromY();
      const observer = () => syncFromY();
      ytext.observe(observer);
      return () => ytext.unobserve(observer);
    }, [ytext, onChange]);

    return (
      <textarea
        ref={elRef}
        className={className}
        readOnly={readOnly}
        spellCheck={false}
        defaultValue={ytext.toString()}
        onChange={(e) => {
          if (remote.current || readOnly) return;
          const next = e.target.value;
          const prev = ytext.toString();
          if (next === prev) return;
          ytext.doc?.transact(() => {
            ytext.delete(0, ytext.length);
            if (next) ytext.insert(0, next);
          });
          onChange?.(next);
        }}
        onSelect={(e) => {
          const t = e.currentTarget;
          onCursor?.(t.selectionStart);
        }}
        onKeyUp={(e) => onCursor?.(e.currentTarget.selectionStart)}
        onClick={(e) => onCursor?.(e.currentTarget.selectionStart)}
        onKeyDown={(e) => {
          if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "b") {
            e.preventDefault();
            ref &&
              typeof ref !== "function" &&
              ref.current?.wrapSelection("**");
          }
          if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "i") {
            e.preventDefault();
            ref &&
              typeof ref !== "function" &&
              ref.current?.wrapSelection("*");
          }
          if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "e") {
            e.preventDefault();
            ref &&
              typeof ref !== "function" &&
              ref.current?.wrapSelection("`");
          }
        }}
      />
    );
  },
);
