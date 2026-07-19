/**
 * Shared client for the HelixSynthBio registry UI.
 * Talks to the console BFF proxy: /api/p/helix-synthbio/<path>.
 */

export type Design = {
  id: string;
  accession: string;
  name: string;
  description: string;
  status: string;
  access_class: string;
  current_version: number;
  created_by: string;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
};

export type Component = {
  name: string;
  role_so: string;
  start: number;
  end: number;
  strand: number;
  source: string;
};

export type DesignVersion = {
  id: string;
  design_id: string;
  version: number;
  alphabet: string;
  topology: string;
  source_kind: string;
  source_name: string;
  sequence_length: number;
  sequence_text: string;
  components: Component[];
  content_hash: string;
  provenance: string;
  notes: string;
  created_by: string;
  created_at: string;
};

export type RiskCase = {
  id: string;
  design_id: string;
  design_version_id: string;
  state: string;
  reviewer: string;
  intended_use: string;
  policy_version: string;
  reasons: string[];
  conditions: string;
  expires_at: string | null;
  decided_at: string | null;
  created_at: string;
  updated_at: string;
};

export type LineageEdge = {
  id: string;
  parent_kind: string;
  parent_id: string;
  child_kind: string;
  child_id: string;
  relation: string;
  created_at: string;
};

export type LineageEvent = {
  id: string;
  event_kind: string;
  entity_kind: string;
  entity_id: string;
  actor: string;
  details: Record<string, unknown>;
  content_hash: string;
  prev_hash: string;
  created_at: string;
};

export type Design360Data = {
  design: Design;
  versions: DesignVersion[];
  risk_case: RiskCase | null;
  effective_risk: string;
  edges: LineageEdge[];
  events: LineageEvent[];
};

export type QueueItem = { case: RiskCase; accession: string };

export type ImportManifest = {
  total_records: number;
  accepted_count: number;
  rejected_count: number;
  accepted: Design[];
  rejected: { record: string; line: number; reason: string }[];
};

export type Bundle = {
  bundle_version: string;
  generated_at: string;
  design: Design;
  versions: DesignVersion[];
  risk_case: RiskCase | null;
  events: LineageEvent[];
  edges: LineageEdge[];
  bundle_hash: string;
};

/** Fetch through the BFF proxy; throws Error with the server's message on failure. */
export async function sbApi<T = unknown>(path: string, init?: RequestInit): Promise<T> {
  const r = await fetch(`/api/p/helix-synthbio${path}`, init);
  const text = await r.text();
  let json: unknown = null;
  try {
    json = text ? JSON.parse(text) : null;
  } catch {
    json = text;
  }
  if (!r.ok) {
    const j = json as {
      error?: { code?: string; message?: string } | string;
      message?: string;
    } | null;
    const msg =
      (typeof j?.error === "object" ? j.error?.message : undefined) ??
      (typeof j?.error === "string" ? j.error : undefined) ??
      j?.message ??
      `${r.status} ${r.statusText}`;
    throw new Error(String(msg));
  }
  return json as T;
}

export function listOf<T>(json: unknown): T[] {
  const d = (json as { data?: { items?: T[] } | T[] })?.data;
  if (Array.isArray(d)) return d;
  return d?.items ?? [];
}

export function shortHash(h: string, n = 12): string {
  if (!h) return "—";
  return h.length > n ? h.slice(0, n) : h;
}

export function shortId(id: string): string {
  return id.length > 8 ? id.slice(0, 8) : id;
}

export function fmtTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  const pad = (v: number) => String(v).padStart(2, "0");
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())} ${pad(
    d.getUTCHours(),
  )}:${pad(d.getUTCMinutes())}Z`;
}

/** Wrap a sequence at `width` chars per line. */
export function wrapSeq(seq: string, width = 60): string {
  const clean = seq.replace(/\s+/g, "");
  const lines: string[] = [];
  for (let i = 0; i < clean.length; i += width) lines.push(clean.slice(i, i + width));
  return lines.join("\n");
}

export const RISK_STATES = ["unknown", "allowed", "restricted", "blocked"] as const;

export function riskClass(state: string): string {
  return `sb-chip sb-risk-${RISK_STATES.includes(state as (typeof RISK_STATES)[number]) ? state : "unknown"}`;
}

/* ——— Feature role families (sequence map + components table) ——— */

export type RoleFamily = "promoter" | "cds" | "rbs" | "terminator" | "origin" | "other";

const SO_FAMILY: Record<string, RoleFamily> = {
  "SO:0000167": "promoter", // promoter
  "SO:0000316": "cds", // CDS
  "SO:0000139": "rbs", // ribosome_entry_site
  "SO:0000141": "terminator", // terminator
  "SO:0000296": "origin", // origin_of_replication
};

export function roleFamily(roleSo: string, name = ""): RoleFamily {
  const hit = SO_FAMILY[roleSo];
  if (hit) return hit;
  const n = name.toLowerCase();
  if (/promoter|^p[A-Z]/.test(n) || n.includes("prom")) return "promoter";
  if (n.includes("rbs") || n.includes("ribosome")) return "rbs";
  if (n.includes("term")) return "terminator";
  if (n.includes("ori") || n.includes("origin")) return "origin";
  if (roleSo === "SO:0000316" || n.includes("cds") || n.includes("enzyme")) return "cds";
  return "other";
}

export const ROLE_COLORS: Record<RoleFamily, string> = {
  promoter: "#0d9488",
  cds: "#2563eb",
  rbs: "#7c3aed",
  terminator: "#dc2626",
  origin: "#d97706",
  other: "#64748b",
};

export function hexToRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

/** Small valid 2-record GenBank demo for the import tab. */
export const DEMO_GENBANK = `LOCUS       DEMOPLASMA1            96 bp    DNA     circular SYN 19-JUL-2026
DEFINITION  Demo plasmid backbone alpha.
ACCESSION   DEMOPLASMA1
FEATURES             Location/Qualifiers
     promoter        10..40
                     /label="pTet"
ORIGIN
        1 acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt
       61 acgtacgtac gtacgtacgt acgtacgtac gtacgt
//
LOCUS       DEMOPLASMA2            60 bp    DNA     linear   SYN 19-JUL-2026
DEFINITION  Demo linear insert beta.
ACCESSION   DEMOPLASMA2
ORIGIN
        1 tttgacagct agctcagtcc taggtatagt gctagcggcc gcttctagag
//
`;
