/**
 * Client-side DNA → protein translation (NCBI standard genetic code, table 1).
 *
 * Component coordinates are GenBank-style: 1-based, inclusive on both ends.
 * Strand −1 means the feature is read from the reverse complement.
 */

/** Full 64-codon standard genetic code. `*` marks stop codons. */
const CODON_TABLE: Record<string, string> = {
  TTT: "F", TTC: "F", TTA: "L", TTG: "L",
  CTT: "L", CTC: "L", CTA: "L", CTG: "L",
  ATT: "I", ATC: "I", ATA: "I", ATG: "M",
  GTT: "V", GTC: "V", GTA: "V", GTG: "V",
  TCT: "S", TCC: "S", TCA: "S", TCG: "S",
  CCT: "P", CCC: "P", CCA: "P", CCG: "P",
  ACT: "T", ACC: "T", ACA: "T", ACG: "T",
  GCT: "A", GCC: "A", GCA: "A", GCG: "A",
  TAT: "Y", TAC: "Y", TAA: "*", TAG: "*",
  CAT: "H", CAC: "H", CAA: "Q", CAG: "Q",
  AAT: "N", AAC: "N", AAA: "K", AAG: "K",
  GAT: "D", GAC: "D", GAA: "E", GAG: "E",
  TGT: "C", TGC: "C", TGA: "*", TGG: "W",
  CGT: "R", CGC: "R", CGA: "R", CGG: "R",
  AGT: "S", AGC: "S", AGA: "R", AGG: "R",
  GGT: "G", GGC: "G", GGA: "G", GGG: "G",
};

const COMPLEMENT: Record<string, string> = {
  A: "T", T: "A", G: "C", C: "G",
  U: "A", R: "Y", Y: "R", S: "S", W: "W",
  K: "M", M: "K", B: "V", D: "H", H: "D",
  V: "B", N: "N",
};

/** Reverse complement of a DNA sequence (IUPAC-aware, uppercases output). */
export function reverseComplement(seq: string): string {
  const up = seq.toUpperCase().replace(/\s+/g, "");
  let out = "";
  for (let i = up.length - 1; i >= 0; i--) {
    out += COMPLEMENT[up[i]] ?? "N";
  }
  return out;
}

/**
 * Translate a DNA reading frame into amino acids.
 * Only complete codons are translated; a trailing partial codon is dropped.
 * Unknown codons (gaps / ambiguous bases) become "X".
 */
export function translateDna(dna: string): string {
  const up = dna.toUpperCase().replace(/\s+/g, "").replace(/U/g, "T");
  let protein = "";
  for (let i = 0; i + 3 <= up.length; i += 3) {
    const codon = up.slice(i, i + 3);
    protein += CODON_TABLE[codon] ?? "X";
  }
  return protein;
}

/**
 * Translate a component's span of a version sequence, respecting strand.
 * `start`/`end` are 1-based inclusive and clamped to the sequence bounds.
 * Returns "" when the span is invalid or too short for one codon.
 */
export function translateComponent(
  sequence: string,
  start: number,
  end: number,
  strand: number,
): string {
  const len = sequence.length;
  if (len === 0) return "";
  const lo = Math.max(1, Math.min(start, end));
  const hi = Math.min(len, Math.max(start, end));
  if (hi < lo) return "";
  const sub = sequence.slice(lo - 1, hi);
  return translateDna(strand < 0 ? reverseComplement(sub) : sub);
}

/** True for CDS-role components (SO:0000316). */
export function isCds(roleSo: string): boolean {
  return roleSo === "SO:0000316";
}

/** One-line preview of a translation: first residues + length, e.g. "MNKTW… (29 aa)". */
export function translationPreview(protein: string, head = 5): string {
  if (!protein) return "";
  const shown = protein.slice(0, head);
  return protein.length > head
    ? `${shown}… (${protein.length} aa)`
    : `${shown} (${protein.length} aa)`;
}
