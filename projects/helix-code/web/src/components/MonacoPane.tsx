"use client";

import Editor, { type OnMount } from "@monaco-editor/react";
import { useEffect, useRef } from "react";
import type { editor as MonacoEditor, IDisposable } from "monaco-editor";
import type { LspDiagnostic } from "@/lib/api";
import { api } from "@/lib/api";

type Props = {
  path: string | null;
  value: string;
  onChange: (v: string) => void;
  readOnly?: boolean;
  diagnostics?: LspDiagnostic[];
  lspSessionId?: string | null;
  onCursor?: (line: number, character: number) => void;
  onGoTo?: (path: string, line: number, character: number) => void;
  onRevealLine?: (line: number, character: number) => void;
};

function languageFor(path: string | null): string {
  if (!path) return "plaintext";
  const lower = path.toLowerCase();
  if (lower.endsWith(".rs")) return "rust";
  if (lower.endsWith(".ts") || lower.endsWith(".tsx")) return "typescript";
  if (lower.endsWith(".js") || lower.endsWith(".jsx")) return "javascript";
  if (lower.endsWith(".json")) return "json";
  if (lower.endsWith(".md")) return "markdown";
  if (lower.endsWith(".toml")) return "ini";
  if (lower.endsWith(".yml") || lower.endsWith(".yaml")) return "yaml";
  if (lower.endsWith(".css")) return "css";
  if (lower.endsWith(".html")) return "html";
  if (lower.endsWith(".sql")) return "sql";
  if (lower.endsWith(".py")) return "python";
  if (lower.endsWith(".go")) return "go";
  return "plaintext";
}

/** LSP DiagnosticSeverity: 1 Error, 2 Warning, 3 Info, 4 Hint → Monaco 8/4/2/1 */
function toMarkerSeverity(severity: number): number {
  switch (severity) {
    case 1:
      return 8; // Error
    case 2:
      return 4; // Warning
    case 3:
      return 2; // Info
    default:
      return 1; // Hint
  }
}

export function MonacoPane({
  path,
  value,
  onChange,
  readOnly,
  diagnostics = [],
  lspSessionId,
  onCursor,
  onGoTo,
  onRevealLine,
}: Props) {
  const editorRef = useRef<MonacoEditor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<typeof import("monaco-editor") | null>(null);
  const sessionRef = useRef<string | null>(null);
  const pathRef = useRef<string | null>(null);
  const disposables = useRef<IDisposable[]>([]);

  sessionRef.current = lspSessionId ?? null;
  pathRef.current = path;

  const onMount: OnMount = (editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;

    // clean previous providers
    for (const d of disposables.current) d.dispose();
    disposables.current = [];

    editor.onDidChangeCursorPosition((e) => {
      onCursor?.(e.position.lineNumber - 1, e.position.column - 1);
    });

    // Completion from forge LSP
    disposables.current.push(
      monaco.languages.registerCompletionItemProvider(
        { pattern: "**/*" },
        {
          triggerCharacters: [".", ":", ":", "<", '"', "'", "/"],
          provideCompletionItems: async (model, position) => {
            const sid = sessionRef.current;
            const p = pathRef.current;
            if (!sid || !p) return { suggestions: [] };
            try {
              const r = await api.lspCompletion(
                sid,
                p,
                position.lineNumber - 1,
                position.column - 1,
              );
              const suggestions = (r.items ?? []).map((it, i) => ({
                label: it.label,
                kind: mapCompletionKind(monaco, it.kind),
                insertText: it.insert_text || it.label,
                detail: it.detail ?? undefined,
                documentation: it.documentation ?? undefined,
                range: {
                  startLineNumber: position.lineNumber,
                  startColumn: position.column,
                  endLineNumber: position.lineNumber,
                  endColumn: position.column,
                },
                sortText: String(i).padStart(4, "0"),
              }));
              return { suggestions };
            } catch {
              return { suggestions: [] };
            }
          },
        },
      ),
    );

    // Hover via forge LSP (in addition to parent panel)
    disposables.current.push(
      monaco.languages.registerHoverProvider({ pattern: "**/*" }, {
        provideHover: async (model, position) => {
          const sid = sessionRef.current;
          const p = pathRef.current;
          if (!sid || !p) return null;
          try {
            const r = await api.lspHover(
              sid,
              p,
              position.lineNumber - 1,
              position.column - 1,
            );
            if (!r.hover?.contents) return null;
            return {
              contents: [{ value: r.hover.contents }],
            };
          } catch {
            return null;
          }
        },
      }),
    );

    // Go to definition (F12 / Ctrl+Click)
    disposables.current.push(
      monaco.languages.registerDefinitionProvider({ pattern: "**/*" }, {
        provideDefinition: async (model, position) => {
          const sid = sessionRef.current;
          const p = pathRef.current;
          if (!sid || !p) return null;
          try {
            const r = await api.lspDefinition(
              sid,
              p,
              position.lineNumber - 1,
              position.column - 1,
            );
            const items = r.items ?? [];
            if (!items.length) return null;
            const first = items[0];
            if (first.path && first.path !== p) {
              onGoTo?.(
                first.path,
                first.range.start_line,
                first.range.start_character,
              );
            } else {
              onRevealLine?.(
                first.range.start_line,
                first.range.start_character,
              );
            }
            return {
              uri: model.uri,
              range: {
                startLineNumber: first.range.start_line + 1,
                startColumn: first.range.start_character + 1,
                endLineNumber: first.range.end_line + 1,
                endColumn: first.range.end_character + 1,
              },
            };
          } catch {
            return null;
          }
        },
      }),
    );

    // Cmd/Ctrl+Space force completion
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Space, () => {
      editor.trigger("helix", "editor.action.triggerSuggest", {});
    });
  };

  useEffect(() => {
    const monaco = monacoRef.current;
    const editor = editorRef.current;
    if (!monaco || !editor) return;
    const model = editor.getModel();
    if (!model) return;
    const markers = diagnostics.map((d) => ({
      severity: toMarkerSeverity(d.severity),
      message: d.message,
      startLineNumber: d.range.start_line + 1,
      startColumn: d.range.start_character + 1,
      endLineNumber: d.range.end_line + 1,
      endColumn: Math.max(d.range.end_character + 1, d.range.start_character + 2),
      source: d.source || "lsp",
      code: d.code ?? undefined,
    }));
    monaco.editor.setModelMarkers(model, "helix-lsp", markers);
  }, [diagnostics, path, value]);

  // Reveal line when parent requests (problem click)
  useEffect(() => {
    if (!onRevealLine) return;
    // no-op holder; parent uses imperative reveal via key
  }, [onRevealLine]);

  return (
    <div className="editor-host">
      <Editor
        height="100%"
        theme="vs-dark"
        path={path ?? "untitled"}
        language={languageFor(path)}
        value={value}
        onChange={(v) => onChange(v ?? "")}
        onMount={onMount}
        options={{
          readOnly: !!readOnly,
          minimap: { enabled: true },
          fontSize: 13,
          fontFamily: "Cascadia Code, JetBrains Mono, Consolas, monospace",
          wordWrap: "on",
          automaticLayout: true,
          scrollBeyondLastLine: false,
          tabSize: 2,
          quickSuggestions: true,
          suggestOnTriggerCharacters: true,
          definitionLinkOpensInPeek: false,
          links: true,
        }}
      />
    </div>
  );
}

/** Expose reveal for parent via editor instance if needed */
export function revealInEditor(
  editor: MonacoEditor.IStandaloneCodeEditor | null,
  line: number,
  character: number,
) {
  if (!editor) return;
  editor.revealPositionInCenter({
    lineNumber: line + 1,
    column: character + 1,
  });
  editor.setPosition({ lineNumber: line + 1, column: character + 1 });
  editor.focus();
}

function mapCompletionKind(
  monaco: typeof import("monaco-editor"),
  kind?: number | null,
): number {
  // LSP CompletionItemKind → Monaco loosely
  const K = monaco.languages.CompletionItemKind;
  switch (kind) {
    case 1:
      return K.Text;
    case 2:
      return K.Method;
    case 3:
      return K.Function;
    case 4:
      return K.Constructor;
    case 5:
      return K.Field;
    case 6:
      return K.Variable;
    case 7:
      return K.Class;
    case 8:
      return K.Interface;
    case 9:
      return K.Module;
    case 10:
      return K.Property;
    case 12:
      return K.Value;
    case 13:
      return K.Enum;
    case 14:
      return K.Keyword;
    case 15:
      return K.Snippet;
    default:
      return K.Text;
  }
}
