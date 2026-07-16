"use client";

export type SemanticState =
  | "active"
  | "waiting_human"
  | "waiting_external"
  | "completed"
  | "failed"
  | "unknown";

const LABELS: Record<SemanticState, string> = {
  active: "Active",
  waiting_human: "Waiting for you",
  waiting_external: "Waiting",
  completed: "Completed",
  failed: "Failed",
  unknown: "Unknown",
};

export function SemanticBadge({ state }: { state: SemanticState | string }) {
  const key = (state as SemanticState) in LABELS ? (state as SemanticState) : "unknown";
  return (
    <span className={`semantic-badge semantic-${key}`} title={state}>
      {LABELS[key]}
    </span>
  );
}
