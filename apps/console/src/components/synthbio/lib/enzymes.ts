/**
 * Restriction digest simulation — pure logic.
 *
 * Sites are searched on the forward sequence AND on its reverse complement
 * (a site can sit on either strand orientation); revcomp hits are mapped
 * back to forward coordinates and de-duplicated by site span. Cut positions
 * are expressed as "cut index" c = number of bases before the cut, i.e. the
 * cut falls between 1-based bases c and c+1. Only top-strand cuts are used
 * for fragment math.
 *
 * Orientation: a forward-scan hit at 0-based i cuts at i + cutTop; a
 * reverse-scan hit (mapped to forward start f) cuts at f + (L − cutBottom).
 * For palindromic recognitions both scans find the same span and the
 * forward formula wins, so no double-counting.
 *
 * Worked example (verified by hand):
 *   EcoRI on "GGAATTCC" — site GAATTC at 0-based start 1, cutTop 1 → cut index 2
 *   (the cut falls between 1-based bases 2 and 3, G^G wait: G2 ^ A3):
 *     linear:   fragments [1..2] = 2 bp ("GG") and [3..8] = 6 bp ("AATTCC")
 *     circular: single cut → ONE fragment start=3, end=2 (wrapping the
 *               origin), size 8 — a linearized full-length band.
 */

import { SeqValidationError, type Topology } from "./seqops";

export type Enzyme = {
  name: string;
  recognition: string; // 5'→3', may include N
  cutTop: number; // bases before the cut on the top strand, from the site start
  cutBottom: number; // same for the bottom strand
};

export const ENZYMES: Enzyme[] = [
  { name: "EcoRI", recognition: "GAATTC", cutTop: 1, cutBottom: 5 },
  { name: "BamHI", recognition: "GGATCC", cutTop: 1, cutBottom: 5 },
  { name: "HindIII", recognition: "AAGCTT", cutTop: 1, cutBottom: 5 },
  { name: "NotI", recognition: "GCGGCCGC", cutTop: 2, cutBottom: 6 },
  { name: "XbaI", recognition: "TCTAGA", cutTop: 1, cutBottom: 5 },
  { name: "SpeI", recognition: "ACTAGT", cutTop: 1, cutBottom: 5 },
  { name: "PstI", recognition: "CTGCAG", cutTop: 5, cutBottom: 1 },
  { name: "NdeI", recognition: "CATATG", cutTop: 2, cutBottom: 4 },
  { name: "XhoI", recognition: "CTCGAG", cutTop: 1, cutBottom: 5 },
  { name: "KpnI", recognition: "GGTACC", cutTop: 4, cutBottom: 1 },
  { name: "SacI", recognition: "GAGCTC", cutTop: 1, cutBottom: 5 },
  { name: "SalI", recognition: "GTCGAC", cutTop: 1, cutBottom: 5 },
  { name: "EcoRV", recognition: "GATATC", cutTop: 3, cutBottom: 3 },
  { name: "SmaI", recognition: "CCCGGG", cutTop: 3, cutBottom: 3 },
  { name: "NcoI", recognition: "CCATGG", cutTop: 1, cutBottom: 5 },
  { name: "BglII", recognition: "AGATCT", cutTop: 1, cutBottom: 5 },
  { name: "NheI", recognition: "GCTAGC", cutTop: 1, cutBottom: 5 },
  { name: "AgeI", recognition: "ACCGGT", cutTop: 1, cutBottom: 5 },
  { name: "PacI", recognition: "TTAATTAA", cutTop: 3, cutBottom: 5 },
  { name: "PmeI", recognition: "GTTTAAAC", cutTop: 3, cutBottom: 5 },
  // Type IIS — simplified per spec: scan the recognition pattern only and
  // display a single top-strand cut at +1 from the site start (the real
  // downstream N-offset is ignored).
  { name: "BsaI", recognition: "GGTCTC", cutTop: 1, cutBottom: 1 },
  { name: "BsmBI", recognition: "CGTCTC", cutTop: 1, cutBottom: 1 },
];

export type DigestFragment = { start: number; end: number; size: number }; // 1-based inclusive; start > end = wraps origin (circular)

export type EnzymeCuts = { enzyme: string; cuts: number[] }; // sorted cut indices ("cut after 1-based base c")

export type DigestResult = {
  perEnzyme: EnzymeCuts[];
  cuts: number[]; // sorted unique cut indices across all enzymes
  fragments: DigestFragment[];
  circular: boolean;
  uncut: boolean;
};

const COMPLEMENT: Record<string, string> = {
  A: "T", T: "A", U: "A", C: "G", G: "C",
  R: "Y", Y: "R", S: "S", W: "W", K: "M", M: "K",
  B: "V", V: "B", D: "H", H: "D", N: "N",
};

export function reverseComplement(seq: string): string {
  let out = "";
  for (let i = seq.length - 1; i >= 0; i--) out += COMPLEMENT[seq[i]] ?? "N";
  return out;
}

/** IUPAC code → matching base letters (used to build site-scan regexes). */
const IUPAC_CLASS: Record<string, string> = {
  A: "A", C: "C", G: "G", T: "T", U: "U",
  R: "AGR", Y: "CTY", S: "GCS", W: "ATW", K: "GTK", M: "ACM",
  B: "CGTB", D: "AGTD", H: "ACTH", V: "ACGV", N: "ACGTN",
};

/** 0-based start indices of every (possibly overlapping) recognition match. */
function siteStarts(seq: string, recognition: string): number[] {
  const pat = recognition
    .split("")
    .map((ch) => `[${IUPAC_CLASS[ch] ?? ch}]`)
    .join("");
  const re = new RegExp(`(?=(${pat}))`, "g");
  const starts: number[] = [];
  let m: RegExpExecArray | null;
  while ((m = re.exec(seq)) !== null) starts.push(m.index);
  return starts;
}

/**
 * Digest `sequence` with the named enzymes (≥1, all must be known).
 * Fragments are 1-based inclusive; circular fragments may wrap the origin.
 * 0 cuts → uncut (one full-length fragment; a circle when circular).
 */
export function digest(
  sequence: string,
  topology: Topology,
  enzymeNames: string[],
): DigestResult {
  const seq = sequence.toUpperCase().replace(/\s+/g, "");
  if (seq.length === 0) throw new SeqValidationError("cannot digest an empty sequence");
  if (!/^[A-Z]+$/.test(seq)) {
    throw new SeqValidationError("sequence must contain letters only");
  }
  if (!enzymeNames || enzymeNames.length === 0) {
    throw new SeqValidationError("select at least one enzyme");
  }
  const enzymes = enzymeNames.map((n) => {
    const e = ENZYMES.find((x) => x.name.toLowerCase() === n.toLowerCase());
    if (!e) throw new SeqValidationError(`unknown enzyme: ${n}`);
    return e;
  });

  const len = seq.length;
  const rc = reverseComplement(seq);
  const circular = topology === "circular";

  const perEnzyme: EnzymeCuts[] = enzymes.map((e) => {
    const L = e.recognition.length;
    // span start → orientation; forward hits win on ties (palindromes)
    const spans = new Map<number, "fwd" | "rev">();
    for (const i of siteStarts(seq, e.recognition)) spans.set(i, "fwd");
    for (const r of siteStarts(rc, e.recognition)) {
      const f = len - r - L; // revcomp hit mapped back to forward 0-based start
      if (!spans.has(f)) spans.set(f, "rev");
    }
    const cuts = [...spans.entries()]
      .map(([i, ori]) => (ori === "fwd" ? i + e.cutTop : i + (L - e.cutBottom)))
      .filter((c) => c >= 0 && c <= len)
      .sort((a, b) => a - b);
    return { enzyme: e.name, cuts };
  });

  const cuts = [...new Set(perEnzyme.flatMap((p) => p.cuts))].sort((a, b) => a - b);
  const uncut = cuts.length === 0;

  const fragments: DigestFragment[] = [];
  if (uncut) {
    fragments.push({ start: 1, end: len, size: len });
  } else if (!circular) {
    const bounds = [0, ...cuts, len];
    for (let i = 0; i + 1 < bounds.length; i++) {
      fragments.push({ start: bounds[i] + 1, end: bounds[i + 1], size: bounds[i + 1] - bounds[i] });
    }
  } else {
    for (let i = 0; i < cuts.length; i++) {
      const a = cuts[i];
      const b = i + 1 < cuts.length ? cuts[i + 1] : cuts[0];
      if (i + 1 < cuts.length) {
        fragments.push({ start: a + 1, end: b, size: b - a });
      } else {
        // wrap fragment: last cut → first cut across the origin
        fragments.push({ start: a + 1, end: b, size: len - a + b });
      }
    }
  }

  return { perEnzyme, cuts, fragments, circular, uncut };
}

/** Extract a fragment's sequence (handles circular origin wrap). */
export function fragmentSequence(seq: string, f: DigestFragment): string {
  if (f.start <= f.end) return seq.slice(f.start - 1, f.end);
  return seq.slice(f.start - 1) + seq.slice(0, f.end);
}
