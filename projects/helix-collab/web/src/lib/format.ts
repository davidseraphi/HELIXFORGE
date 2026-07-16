/** Compact relative time for activity / comments. */
export function relativeTime(iso: string, now = Date.now()): string {
  const t = new Date(iso).getTime();
  if (Number.isNaN(t)) return iso;
  const d = Math.max(0, now - t);
  if (d < 45_000) return "just now";
  if (d < 3_600_000) return `${Math.floor(d / 60_000)}m ago`;
  if (d < 86_400_000) return `${Math.floor(d / 3_600_000)}h ago`;
  if (d < 7 * 86_400_000) return `${Math.floor(d / 86_400_000)}d ago`;
  return new Date(iso).toLocaleDateString();
}

/** Human label for activity action codes. */
export function activityLabel(action: string): string {
  const map: Record<string, string> = {
    "document.patched": "Edited document",
    "document.create": "Created document",
    "comment.created": "Added comment",
    "comment.resolved": "Resolved thread",
    "comment.unresolved": "Reopened thread",
    "comment.deleted": "Deleted comment",
    created: "Added comment",
    resolved: "Resolved thread",
    unresolved: "Reopened thread",
    deleted: "Deleted comment",
  };
  return map[action] ?? action.replace(/\./g, " · ");
}
