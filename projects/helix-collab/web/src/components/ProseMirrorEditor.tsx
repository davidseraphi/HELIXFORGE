"use client";

import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
} from "react";
import * as Y from "yjs";
import { EditorState, TextSelection } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import { keymap } from "prosemirror-keymap";
import { baseKeymap, toggleMark } from "prosemirror-commands";
import { history, undo, redo } from "prosemirror-history";
import {
  ySyncPlugin,
  yUndoPlugin,
  prosemirrorToYXmlFragment,
} from "y-prosemirror";
import {
  collabSchema,
  docToMarkdown,
  markdownToDoc,
} from "@/lib/pm-schema";

export type ProseMirrorHandle = {
  focus: () => void;
  getMarkdown: () => string;
  setMarkdown: (md: string) => void;
  wrapSelection: (before: string, after?: string) => void;
  insertAtCursor: (text: string) => void;
  getSelection: () => { start: number; end: number };
  setSelection: (start: number, end: number) => void;
};

type Props = {
  ydoc?: Y.Doc | null;
  className?: string;
  initialMarkdown?: string;
  onChange?: (markdown: string) => void;
  onCursor?: (pos: number) => void;
  readOnly?: boolean;
  /** Bind to Y.XmlFragment("prosemirror") for multiplayer CRDT. */
  collab?: boolean;
};

/**
 * ProseMirror rich editor. Optional y-prosemirror collab on Y.XmlFragment.
 * Durable / e2ee path uses markdown serialization.
 */
export const ProseMirrorEditor = forwardRef<ProseMirrorHandle, Props>(
  function ProseMirrorEditor(
    {
      ydoc,
      className,
      initialMarkdown = "",
      onChange,
      onCursor,
      readOnly,
      collab = false,
    },
    ref,
  ) {
    const hostRef = useRef<HTMLDivElement>(null);
    const viewRef = useRef<EditorView | null>(null);
    const onChangeRef = useRef(onChange);
    onChangeRef.current = onChange;
    const onCursorRef = useRef(onCursor);
    onCursorRef.current = onCursor;
    const seeded = useRef(false);

    useImperativeHandle(ref, () => ({
      focus() {
        viewRef.current?.focus();
      },
      getMarkdown() {
        const v = viewRef.current;
        if (!v) return "";
        return docToMarkdown(v.state.doc);
      },
      setMarkdown(md: string) {
        const v = viewRef.current;
        if (!v) return;
        const doc = markdownToDoc(md);
        const tr = v.state.tr.replaceWith(
          0,
          v.state.doc.content.size,
          doc.content,
        );
        v.dispatch(tr);
      },
      wrapSelection(before, after = before) {
        const v = viewRef.current;
        if (!v || readOnly) return;
        const { from, to } = v.state.selection;
        const text = v.state.doc.textBetween(from, to, "\n") || "text";
        v.dispatch(v.state.tr.insertText(before + text + after, from, to));
        v.focus();
      },
      insertAtCursor(text) {
        const v = viewRef.current;
        if (!v || readOnly) return;
        const { from, to } = v.state.selection;
        v.dispatch(v.state.tr.insertText(text, from, to));
        v.focus();
      },
      getSelection() {
        const v = viewRef.current;
        if (!v) return { start: 0, end: 0 };
        return { start: v.state.selection.from, end: v.state.selection.to };
      },
      setSelection(start, end) {
        const v = viewRef.current;
        if (!v) return;
        const max = v.state.doc.content.size;
        const s = Math.max(1, Math.min(start, max));
        const e = Math.max(s, Math.min(end, max));
        try {
          const sel = TextSelection.create(v.state.doc, s, e);
          v.dispatch(v.state.tr.setSelection(sel));
          v.focus();
        } catch {
          /* invalid pos */
        }
      },
    }));

    useEffect(() => {
      const host = hostRef.current;
      if (!host) return;
      seeded.current = false;

      const plugins = [
        history(),
        keymap({
          "Mod-z": undo,
          "Mod-y": redo,
          "Mod-Shift-z": redo,
          "Mod-b": toggleMark(collabSchema.marks.strong!),
          "Mod-i": toggleMark(collabSchema.marks.em!),
          "Mod-e": toggleMark(collabSchema.marks.code!),
        }),
        keymap(baseKeymap),
      ];

      let state: EditorState;

      if (collab && ydoc) {
        const type = ydoc.getXmlFragment("prosemirror");
        if (type.length === 0 && initialMarkdown.trim()) {
          try {
            const doc = markdownToDoc(initialMarkdown);
            prosemirrorToYXmlFragment(doc, type);
            seeded.current = true;
          } catch {
            /* seed best-effort */
          }
        }
        plugins.unshift(ySyncPlugin(type), yUndoPlugin());
        state = EditorState.create({ schema: collabSchema, plugins });
      } else {
        state = EditorState.create({
          doc: markdownToDoc(initialMarkdown),
          schema: collabSchema,
          plugins,
        });
      }

      const view = new EditorView(host, {
        state,
        editable: () => !readOnly,
        dispatchTransaction(tr) {
          const next = view.state.apply(tr);
          view.updateState(next);
          if (tr.docChanged) {
            onChangeRef.current?.(docToMarkdown(next.doc));
          }
          if (tr.selectionSet) {
            onCursorRef.current?.(next.selection.from);
          }
        },
      });
      viewRef.current = view;

      return () => {
        view.destroy();
        viewRef.current = null;
      };
      // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [ydoc, collab, readOnly]);

    useEffect(() => {
      if (collab && ydoc) return;
      const v = viewRef.current;
      if (!v) return;
      const current = docToMarkdown(v.state.doc);
      if (current.trim() === (initialMarkdown || "").trim()) return;
      const doc = markdownToDoc(initialMarkdown || "");
      v.dispatch(
        v.state.tr.replaceWith(0, v.state.doc.content.size, doc.content),
      );
    }, [initialMarkdown, collab, ydoc]);

    return (
      <div
        ref={hostRef}
        className={`pm-editor ${className ?? ""}`}
        data-readonly={readOnly ? "1" : "0"}
      />
    );
  },
);
