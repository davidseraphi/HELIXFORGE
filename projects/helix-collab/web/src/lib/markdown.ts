/** Minimal safe-ish markdown → HTML for collab preview (no deps). */

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function renderMarkdown(src: string): string {
  const lines = src.replace(/\r\n/g, "\n").split("\n");
  const out: string[] = [];
  let inCode = false;
  let codeBuf: string[] = [];
  let listOpen: "ul" | "ol" | null = null;

  const closeList = () => {
    if (listOpen) {
      out.push(`</${listOpen}>`);
      listOpen = null;
    }
  };

  const inline = (t: string) => {
    let s = escapeHtml(t);
    s = s.replace(/`([^`]+)`/g, "<code>$1</code>");
    s = s.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
    s = s.replace(/\*([^*]+)\*/g, "<em>$1</em>");
    s = s.replace(
      /\[([^\]]+)\]\((https?:\/\/[^)\s]+)\)/g,
      '<a href="$2" target="_blank" rel="noreferrer">$1</a>',
    );
    s = s.replace(
      /(^|[\s(])@([A-Za-z0-9_.@+-]+)/g,
      '$1<span class="md-mention">@$2</span>',
    );
    return s;
  };

  for (const line of lines) {
    if (line.startsWith("```")) {
      if (inCode) {
        out.push(`<pre class="md-code"><code>${escapeHtml(codeBuf.join("\n"))}</code></pre>`);
        codeBuf = [];
        inCode = false;
      } else {
        closeList();
        inCode = true;
      }
      continue;
    }
    if (inCode) {
      codeBuf.push(line);
      continue;
    }

    if (/^#{1,6}\s/.test(line)) {
      closeList();
      const level = line.match(/^#+/)?.[0].length ?? 1;
      const text = line.replace(/^#{1,6}\s+/, "");
      out.push(`<h${level}>${inline(text)}</h${level}>`);
      continue;
    }

    if (/^>\s?/.test(line)) {
      closeList();
      out.push(`<blockquote>${inline(line.replace(/^>\s?/, ""))}</blockquote>`);
      continue;
    }

    if (/^[-*]\s+/.test(line)) {
      if (listOpen !== "ul") {
        closeList();
        out.push("<ul>");
        listOpen = "ul";
      }
      out.push(`<li>${inline(line.replace(/^[-*]\s+/, ""))}</li>`);
      continue;
    }

    if (/^\d+\.\s+/.test(line)) {
      if (listOpen !== "ol") {
        closeList();
        out.push("<ol>");
        listOpen = "ol";
      }
      out.push(`<li>${inline(line.replace(/^\d+\.\s+/, ""))}</li>`);
      continue;
    }

    if (line.trim() === "") {
      closeList();
      out.push("<br/>");
      continue;
    }

    closeList();
    out.push(`<p>${inline(line)}</p>`);
  }
  if (inCode) {
    out.push(`<pre class="md-code"><code>${escapeHtml(codeBuf.join("\n"))}</code></pre>`);
  }
  closeList();
  return out.join("\n");
}
