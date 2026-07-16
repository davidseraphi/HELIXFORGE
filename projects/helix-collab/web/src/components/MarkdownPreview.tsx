"use client";

import { useMemo } from "react";
import { renderMarkdown } from "@/lib/markdown";

export function MarkdownPreview({ source }: { source: string }) {
  const html = useMemo(() => renderMarkdown(source), [source]);
  return (
    <div
      className="md-preview"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
