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
  locked_at: string | null;
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

/* ——— Inventory (samples) ——— */

export type Sample = {
  id: string;
  accession: string;
  name: string;
  kind: string;
  design_id: string | null;
  status: string;
  location: string | null;
  created_by: string;
  created_at: string;
  updated_at: string;
};

export type CustodyEvent = {
  id: string;
  sample_id: string;
  event: string;
  from_location: string | null;
  to_location: string | null;
  actor: string;
  notes: string;
  created_at: string;
};

export type SampleDetailData = {
  sample: Sample;
  custody: CustodyEvent[];
  edges: LineageEdge[];
  design_accession: string | null;
};

export const SAMPLE_KINDS = [
  "strain",
  "plasmid_prep",
  "oligo",
  "protein",
  "cell_line",
  "other",
] as const;

/** All custody event kinds the API accepts/returns. */
export const CUSTODY_EVENT_KINDS = [
  "register",
  "transfer",
  "process",
  "consume",
  "store",
  "dispose",
  "aliquot",
  "reconcile",
] as const;

/**
 * Events offered in the Move form: register happens at creation and aliquot
 * has its own action, so neither is posted as a plain custody event.
 */
export const MOVE_EVENTS = [
  "transfer",
  "process",
  "store",
  "consume",
  "dispose",
  "reconcile",
] as const;

/** Color tone key for a custody event chip/dot; unknown kinds fall back to slate. */
export function custodyTone(ev: string): string {
  return (CUSTODY_EVENT_KINDS as readonly string[]).includes(ev) ? ev : "store";
}

/* ——— Measurements ——— */

export type Measurement = {
  id: string;
  accession: string;
  sample_id: string;
  kind: string;
  method: string | null;
  value: number | null;
  unit: string | null;
  uncertainty: number | null;
  raw: Record<string, unknown> | null;
  status: string;
  analyst: string;
  created_at: string;
};

export const MEASUREMENT_KINDS = [
  "absorbance",
  "fluorescence",
  "qpcr",
  "gel",
  "ngs_qc",
  "other",
] as const;

/** Status chip class for a measurement; unknown statuses fall back to draft slate. */
export function measurementStatusClass(status: string): string {
  return `sb-chip sb-ms-${["draft", "accepted", "rejected"].includes(status) ? status : "draft"}`;
}

/* ——— Claims & ELN notes ——— */

export type EvidenceLink = {
  target_kind: string;
  target_id: string;
  support: string;
  note: string | null;
  created_at: string;
};

export type Claim = {
  id: string;
  accession: string;
  design_id: string;
  statement: string;
  status: string;
  attested_by: string | null;
  attested_at: string | null;
  created_by: string;
  created_at: string;
};

export type ClaimWithEvidence = { claim: Claim; evidence: EvidenceLink[] };

export type ElnNote = {
  id: string;
  body: string;
  created_by: string;
  created_at: string;
};

export const CLAIM_STATUSES = ["draft", "under_review", "accepted", "challenged"] as const;

/** Status chip class for a claim; unknown statuses fall back to draft slate. */
export function claimStatusClass(status: string): string {
  return `sb-chip sb-cl-${(CLAIM_STATUSES as readonly string[]).includes(status) ? status : "draft"}`;
}

export const EVIDENCE_SUPPORTS = ["supports", "conflicts", "missing"] as const;
export const EVIDENCE_TARGET_KINDS = ["design_version", "measurement", "analysis"] as const;

/** Support chip class for an evidence link; unknown values fall back to amber. */
export function evidenceSupportClass(support: string): string {
  return `sb-evi sb-evi-${(EVIDENCE_SUPPORTS as readonly string[]).includes(support) ? support : "missing"}`;
}

/* ——— E-signatures ——— */

export type Signature = {
  id: string;
  target_kind: string;
  target_id: string;
  signer: string;
  meaning: string;
  statement: string | null;
  content_hash: string;
  created_at: string;
};

export const SIGNATURE_MEANINGS = ["approved", "witnessed", "reviewed"] as const;

/** Seal color modifier for a signature meaning; unknown meanings fall back to reviewed violet. */
export function signatureSealClass(meaning: string): string {
  return `sb-seal-${(SIGNATURE_MEANINGS as readonly string[]).includes(meaning) ? meaning : "reviewed"}`;
}

/* ——— Journeys (guided pathways) ——— */

export type PathwayStage = {
  stage_key: string;
  title: string;
  explanation: string;
  mode: string;
};

export type Pathway = {
  key: string;
  title: string;
  description: string;
  stages: PathwayStage[];
};

export type Journey = {
  id: string;
  accession: string;
  title: string;
  intent: string;
  pathway_key: string;
  route_choice: string;
  status: string;
  current_stage: number;
  created_by: string;
  created_at: string;
  updated_at: string;
};

export type JourneyCheck = { met: boolean; missing: string };

/** One stage row inside the journey detail payload (check is embedded). */
export type JourneyStageRow = {
  id: string;
  journey_id: string;
  stage_index: number;
  stage_key: string;
  status: string;
  summary: string;
  target_kind: string | null;
  target_id: string | null;
  check: JourneyCheck;
  created_at: string;
  updated_at: string;
};

export type JourneyDetailData = { journey: Journey; stages: JourneyStageRow[] };

export const JOURNEY_STAGE_KEYS = [
  "source",
  "route",
  "design",
  "risk",
  "build",
  "test",
  "evidence",
] as const;

/** Status chip class for a journey stage; unknown statuses fall back to pending slate. */
export function journeyStageClass(status: string): string {
  return `sb-chip sb-st-${["done", "current", "pending"].includes(status) ? status : "pending"}`;
}

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
