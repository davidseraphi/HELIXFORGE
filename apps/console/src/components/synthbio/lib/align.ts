/**
 * Read-to-reference alignment — pure client logic.
 *
 * Per read: exact substring fast path (forward, then reverse complement),
 * otherwise an ungapped seed-and-extend alignment: the read's first 12-mer
 * is located in the reference (the last 12-mer is tried only when the first
 * has NO occurrence), each occurrence is extended ungapped across the full
 * read length, and the placement with the fewest mismatches wins. The whole
 * search runs on both strands; ties prefer "+".
 *
 * Order of gates (chosen so the spec's own spot-checks hold):
 *   1. exact match (any read length — a short exact substring is unambiguous)
 *   2. read < 12 bp → "too-short" (the seed path needs a full 12-mer)
 *   3. no 12-mer occurrence on either strand → "no-seed"
 *
 * Circular references: the search space is ref + ref[0..r-1], so reads
 * spanning the origin align; offsets/mismatch positions are reported in
 * 1-based reference coordinates (mod length). Reads longer than the
 * reference cannot place ungapped and report "no-seed".
 *
 * Worked checks (verified by hand):
 *   ref "ACGTACGTACGT", read "CGTA"            → exact, "+", offset 3, 100%
 *   ref "TTTTAAAACCCCGGGG", read "CCCCGGGGTTTT"
 *     → forward: no exact, no seed; revcomp(read) = "AAAACCCCGGGG" found at
 *       0-based 4 → exact, strand "-", offset 5, 100%
 *   ref "AAAACCCCGGGGTTTT", read "AAAACCCCGGGGTTCA"
 *     → seed "AAAACCCCGGGG" at 0, extend: mismatches at read bases 14/15
 *       (ref positions 15, 16) → aligned, "+", offset 1, 14/16 = 87.5%
 *   ref "ACGTACGTACGT" circular, read "TACGTACGTACG"
 *     → linear would be "no-seed"; circular search space finds it at
 *       0-based 11 → exact, "+", offset 12 (wraps the origin)
 */

import { SeqValidationError } from "./seqops";
import { reverseComplement } from "./enzymes";

export const SEED_LENGTH = 12;
const MISMATCH_CAP = 50;

export type AlignStatus = "exact" | "aligned" | "no-seed" | "too-short";

export type AlignResult = {
  readName: string;
  strand: "+" | "-";
  offset: number; // 1-based start on the reference
  alignedLength: number;
  matches: number;
  mismatches: number;
  identityPct: number; // matches/alignedLength*100, 1 decimal
  mismatchPositions: number[]; // 1-based reference positions, cap 50
  status: AlignStatus;
};

export type AlignSummary = {
  total: number;
  exact: number;
  aligned: number;
  failed: number;
  meanIdentityPct: number; // mean over exact+aligned reads, 1 decimal (0 when none)
};

export type FastaRecord = { name: string; sequence: string };

/** Split a FASTA string into records (`>` headers + wrapped lines, uppercased). */
export function parseFasta(fasta: string): FastaRecord[] {
  const records: FastaRecord[] = [];
  let name: string | null = null;
  let chunks: string[] = [];
  const flush = () => {
    if (name != null) {
      records.push({ name, sequence: chunks.join("").toUpperCase() });
    }
  };
  for (const raw of fasta.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line) continue;
    if (line.startsWith(">")) {
      flush();
      name = line.slice(1).trim() || `read-${records.length + 1}`;
      chunks = [];
    } else {
      chunks.push(line);
    }
  }
  flush();
  return records;
}

type Candidate = { offset0: number; mismatches: number; mismatchPositions: number[] };

/** Best ungapped placement of `read` inside `space` via 12-mer seeding (0-based offset). */
function bestUngapped(space: string, read: string): Candidate | null {
  const r = read.length;
  const seedStarts = r > SEED_LENGTH ? [0, r - SEED_LENGTH] : [0];
  let best: Candidate | null = null;
  for (const p of seedStarts) {
    const seed = read.slice(p, p + SEED_LENGTH);
    let j = space.indexOf(seed);
    let found = false;
    while (j !== -1) {
      found = true;
      const off = j - p;
      if (off >= 0 && off + r <= space.length) {
        let mm = 0;
        const pos: number[] = [];
        for (let k = 0; k < r; k++) {
          if (read[k] !== space[off + k]) {
            mm++;
            if (pos.length < MISMATCH_CAP) pos.push(off + k + 1); // 1-based in space coords
          }
        }
        if (!best || mm < best.mismatches) {
          best = { offset0: off, mismatches: mm, mismatchPositions: pos };
        }
      }
      j = space.indexOf(seed, j + 1);
    }
    if (found) break; // the trailing 12-mer is tried only when the first has no occurrence
  }
  return best;
}

function round1(x: number): number {
  return Math.round(x * 10) / 10;
}

function alignOne(ref: string, circular: boolean, rec: FastaRecord): AlignResult {
  const n = ref.length;
  const read = rec.sequence;
  const r = read.length;
  const blank: Omit<AlignResult, "readName" | "status"> = {
    strand: "+",
    offset: 0,
    alignedLength: 0,
    matches: 0,
    mismatches: 0,
    identityPct: 0,
    mismatchPositions: [],
  };
  if (r === 0) return { readName: rec.name, ...blank, status: "too-short" };

  const space = circular && r <= n ? ref + ref.slice(0, r - 1) : ref;
  const rc = reverseComplement(read);

  // 1) exact fast path (forward first)
  let idx = space.indexOf(read);
  if (idx >= 0) {
    return {
      readName: rec.name,
      ...blank,
      strand: "+",
      offset: (idx % n) + 1,
      alignedLength: r,
      matches: r,
      identityPct: 100,
      status: "exact",
    };
  }
  idx = space.indexOf(rc);
  if (idx >= 0) {
    return {
      readName: rec.name,
      ...blank,
      strand: "-",
      offset: (idx % n) + 1,
      alignedLength: r,
      matches: r,
      identityPct: 100,
      status: "exact",
    };
  }

  // 2) too short to seed
  if (r < SEED_LENGTH) return { readName: rec.name, ...blank, status: "too-short" };

  // 3) ungapped seed-and-extend on both strands
  const fwd = bestUngapped(space, read);
  const rev = bestUngapped(space, rc);
  if (!fwd && !rev) return { readName: rec.name, ...blank, status: "no-seed" };

  const useRev = rev != null && (fwd == null || rev.mismatches < fwd.mismatches);
  const best = (useRev ? rev : fwd) as Candidate;
  const matches = r - best.mismatches;
  return {
    readName: rec.name,
    strand: useRev ? "-" : "+",
    offset: (best.offset0 % n) + 1,
    alignedLength: r,
    matches,
    mismatches: best.mismatches,
    identityPct: round1((matches / r) * 100),
    mismatchPositions: best.mismatchPositions.map((p) => ((p - 1) % n) + 1),
    status: "aligned",
  };
}

/** Align every FASTA read against the reference; returns per-read results + summary. */
export function alignReads(
  reference: string,
  topology: string,
  fasta: string,
): { results: AlignResult[]; summary: AlignSummary } {
  const ref = reference.toUpperCase().replace(/\s+/g, "");
  if (!ref) throw new SeqValidationError("reference sequence is empty");
  if (!/^[A-Z]+$/.test(ref)) {
    throw new SeqValidationError("reference must contain letters only");
  }
  const records = parseFasta(fasta);
  if (records.length === 0) {
    throw new SeqValidationError("no FASTA records found — paste reads starting with a >header line");
  }
  records.forEach((rec, i) => {
    if (!/^[A-Z]*$/.test(rec.sequence)) {
      throw new SeqValidationError(`read ${i + 1} (${rec.name}) contains non-letter characters`);
    }
  });

  const circular = topology === "circular";
  const results = records.map((rec) => alignOne(ref, circular, rec));
  const exact = results.filter((r) => r.status === "exact").length;
  const aligned = results.filter((r) => r.status === "aligned").length;
  const scorable = results.filter((r) => r.status === "exact" || r.status === "aligned");
  const meanIdentityPct = scorable.length
    ? round1(scorable.reduce((a, r) => a + r.identityPct, 0) / scorable.length)
    : 0;
  return {
    results,
    summary: {
      total: results.length,
      exact,
      aligned,
      failed: results.length - exact - aligned,
      meanIdentityPct,
    },
  };
}
