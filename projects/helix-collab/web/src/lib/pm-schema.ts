import { Schema } from "prosemirror-model";
import { schema as basicSchema } from "prosemirror-schema-basic";
import { addListNodes } from "prosemirror-schema-list";

/** HelixCollab rich-text schema: basic + lists. */
export const collabSchema = new Schema({
  nodes: addListNodes(basicSchema.spec.nodes, "paragraph block*", "block"),
  marks: basicSchema.spec.marks,
});

/** Serialize PM doc to simple markdown-ish plain text for storage / e2ee. */
export function docToMarkdown(doc: import("prosemirror-model").Node): string {
  const parts: string[] = [];
  doc.forEach((node) => {
    parts.push(blockToMd(node));
  });
  return parts.join("\n\n").trimEnd() + (parts.length ? "\n" : "");
}

function blockToMd(node: import("prosemirror-model").Node): string {
  const text = inlineToMd(node);
  switch (node.type.name) {
    case "heading": {
      const level = (node.attrs.level as number) || 1;
      return `${"#".repeat(Math.min(6, level))} ${text}`;
    }
    case "blockquote":
      return text
        .split("\n")
        .map((l) => `> ${l}`)
        .join("\n");
    case "code_block":
      return "```\n" + node.textContent + "\n```";
    case "bullet_list": {
      const items: string[] = [];
      node.forEach((li) => {
        items.push(`- ${li.textContent}`);
      });
      return items.join("\n");
    }
    case "ordered_list": {
      const items: string[] = [];
      let i = 1;
      node.forEach((li) => {
        items.push(`${i}. ${li.textContent}`);
        i += 1;
      });
      return items.join("\n");
    }
    case "horizontal_rule":
      return "---";
    case "paragraph":
    default:
      return text;
  }
}

function inlineToMd(node: import("prosemirror-model").Node): string {
  let out = "";
  node.forEach((child) => {
    if (child.isText) {
      let t = child.text || "";
      const marks = child.marks.map((m) => m.type.name);
      if (marks.includes("code")) t = `\`${t}\``;
      if (marks.includes("strong")) t = `**${t}**`;
      if (marks.includes("em")) t = `*${t}*`;
      if (marks.includes("link")) {
        const href = child.marks.find((m) => m.type.name === "link")?.attrs
          .href as string;
        t = `[${t}](${href || ""})`;
      }
      out += t;
    } else if (child.type.name === "hard_break") {
      out += "\n";
    } else {
      out += child.textContent;
    }
  });
  return out;
}

/** Best-effort markdown → PM document (paragraphs + headings). */
export function markdownToDoc(md: string): import("prosemirror-model").Node {
  const lines = (md || "").replace(/\r\n/g, "\n").split("\n");
  const nodes: import("prosemirror-model").Node[] = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i] ?? "";
    if (!line.trim()) {
      i += 1;
      continue;
    }
    const h = /^(#{1,6})\s+(.*)$/.exec(line);
    if (h) {
      nodes.push(
        collabSchema.nodes.heading!.create(
          { level: h[1]!.length },
          h[2] ? collabSchema.text(h[2]) : undefined,
        ),
      );
      i += 1;
      continue;
    }
    if (line.startsWith("```")) {
      i += 1;
      const code: string[] = [];
      while (i < lines.length && !lines[i]!.startsWith("```")) {
        code.push(lines[i]!);
        i += 1;
      }
      if (i < lines.length) i += 1;
      nodes.push(
        collabSchema.nodes.code_block!.create(
          null,
          code.length ? collabSchema.text(code.join("\n")) : undefined,
        ),
      );
      continue;
    }
    // accumulate paragraph
    const para: string[] = [line];
    i += 1;
    while (
      i < lines.length &&
      lines[i]!.trim() &&
      !lines[i]!.startsWith("#") &&
      !lines[i]!.startsWith("```")
    ) {
      para.push(lines[i]!);
      i += 1;
    }
    nodes.push(
      collabSchema.nodes.paragraph!.create(
        null,
        collabSchema.text(para.join("\n")),
      ),
    );
  }
  if (nodes.length === 0) {
    nodes.push(collabSchema.nodes.paragraph!.create());
  }
  return collabSchema.node("doc", null, nodes);
}
