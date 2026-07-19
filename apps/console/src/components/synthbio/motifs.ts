/**
 * Client-side motif auto-annotation.
 *
 * Scans a version sequence against a small curated motif library and reports
 * candidate features with GenBank-style 1-based inclusive coordinates.
 * Both strands are searched (forward pattern + its reverse complement);
 * overlapping hits of the same motif are deduplicated.
 */

import { reverseComplement } from "./translate";

export type MotifDef = {
  name: string;
  role_so: string;
  pattern: string; // exact DNA, matched case-insensitively
};

export type MotifHit = {
  name: string;
  role_so: string;
  start: number; // 1-based inclusive
  end: number; // 1-based inclusive
  strand: 1 | -1;
  source: string;
};

/** motif-lib v1 — curated common parts. */
export const MOTIF_LIBRARY: MotifDef[] = [
  {
    name: "T7 promoter",
    role_so: "SO:0000167",
    pattern: "TAATACGACTCACTATAGGG",
  },
  {
    name: "SP6 promoter",
    role_so: "SO:0000167",
    pattern: "ATTTAGGTGACACTATAG",
  },
  {
    name: "lac promoter",
    role_so: "SO:0000167",
    pattern: "TTTACACTTTATGCTTCCGGCTCG",
  },
  {
    name: "T7 terminator",
    role_so: "SO:0000141",
    pattern: "CTAGCATAACCCCTTGGGGCCTCTAAACGGGTCTTGAGGGGTTTTTTG",
  },
  {
    name: "rrnB T1 terminator",
    role_so: "SO:0000141",
    pattern:
      "AAAGCCACGTTGTGTCTCAAAATCTCTGATGTTACATTGCACAAGATAAAAATATATCATCATGAACAATAAAACTGTCTGCTTACATAAACAGTAATACAAGGGGTGTTATGAGCCATATTCAACGGGAAACGTCTTGCTCGAGGCCGCGATTAAATTCCAACATGGATGCTGATTTATATGGGTATAAATGGGCTCGCGA" +
      "TAATGTCGGGCAATCAGGTGCGACAATCTATCGATTGTATGGGAAGCCCGATGCGCCAGAGTTGTTTCTGAAACATGGCAAAGGTAGCGTTGCCAATGATGTTACAGATGAGATGGTCAGACTAAACTGGCTGACGGAATTTATGCCTCTTCCGACCATCAAGCATTTTATCCGTACTCCTGATGATGCATGGTTA" +
      "CTCACCACTGCGATCCCGGGAAAACAGCATTCCAGGTATTAGAAGAATATCCTGATTCAGGTGAAAATATTGTTGATGCGCTGGCAGTGTTCCTGCGCCGGTTGCATTCGATTCCTGTTTGTAATTGTCCTTTTAACAGCGATCGCGTATTTCGTCTCGCTCAGGCGCAATCACGAATGAATAACGGTTTGGTT" +
      "GATGCGAGTGATTTTGATGACGAGCGTAATGGCTGGCCTGTTGAACAAGTCTGGAAAGAAATGCATAAGCTTTTGCCATTCTCACCGGATTCAGTCGTCACTCATGGTGATTTCTCACTTGATAACCTTATTTTTGACGAGGGGAAATTAATAGGTTGTATTGATGTTGGACGAGTCGGAATCGCAGACCGA" +
      "TACCAGGATCTTGCCATCCTATGGAACTGCCTCGGTGAGTTTTCTCCTTCATTACAGAAACGGCTTTTTCAAAAATATGGTATTGATAATCCTGATATGAATAAATTGCAGTTTCATTTGATGCTCGATGAGTTTTTCTA",
  },
  {
    name: "AmpR bla (segment)",
    role_so: "SO:0000316",
    pattern:
      "ATGAGTATTCAACATTTCCGTGTCGCCCTTATTCCCTTTTTTGCGGCATTTTGCCTTCCTGTTTTTGCTCACCCAGAAACGCTGGTGAAAGTAAAAGATGCTGAAGATCAGTTGGGTGCACGAGTGGGTTACATCGAACTGGATCTCAACAGCGGTAAGATCCTTGAGAGTTTTCGCCCCGAAGAACGTTTT" +
      "CCAATGATGAGCACTTTTAAAGTTCTGCTATGTGGCGCGGTATTATCCCGTATTGACGCCGGGCAAGAGCAACTCGGTCGCCGCATACACTATTCTCAGAATGACTTGGTTGAGTACTCACCAGTCACAGAAAAGCATCTTACGGATGGCATGACAGTAAGAGAATTATGCAGTGCTGCCATAACCATGAGTGA" +
      "TAACACTGCGGCCAACTTACTTCTGACAACGATCGGAGGACCGAAGGAGCTAACCGCTTTTTTGCACAACATGGGGGATCATGTAACTCGCCTTGATCGTTGGGAACCGGAGCTGAATGAAGCCATACCAAACGACGAGCGTGACACCACGATGCCTGTAGCAATGGCAACAACGTTGCGCAAACTATTAAC" +
      "TGGCGAACTACTTACTCTAGCTTCCCGGCAACAATTAATAGACTGGATGGAGGCGGATAAAGTTGCAGGACCACTTCTGCGCTCGGCCCTTCCGGCTGGCTGGTTTATTGCTGATAAATCTGGAGCCGGTGAGCGTGGGTCTCGCGGTATCATTGCAGCACTGGGGCCAGATGGTAAGCCCTCCCGTATCGT" +
      "AGTTATCTACACGACGGGGAGTCAGGCAACTATGGATGAACGAAATAGACAGATCGCTGAGATAGGTGCCTCACTGATTAAGCATTGGTAA",
  },
  {
    name: "pUC ori (segment)",
    role_so: "SO:0000296",
    pattern:
      "TTGAGATCCTTTTTTTCTGCGCGTAATCTGCTGCTTGCAAACAAAAAAACCACCGCTACCAGCGGTGGTTTGTTTGCCGGATCAAGAGCTACCAACTCTTTTTCCGAAGGTAACTGGCTTCAGCAGAGCGCAGATACCAAATACTGTTCTTCTAGTGTAGCCGTAGTTAGGCCACCACTTCAAGAACTCTGT" +
      "AGCACCGCCTACATACCTCGCTCTGCTAATCCTGTTACCAGTGGCTGCTGCCAGTGGCGATAAGTCGTGTCTTACCGGGTTGGACTCAAGACGATAGTTACCGGATAAGGCGCAGCGGTCGGGCTGAACGGGGGGTTCGTGCACACAGCCCAGCTTGGAGCGAACGACCTACACCGAACTGAGATACCTACA" +
      "GCGTGAGCTATGAGAAAGCGCCACGCTTCCCGAAGGGAGAAAGGCGGACAGGTATCCGGTAAGCGGCAGGGTCGGAACAGGAGAGCGCACGAGGGAGCTTCCAGGGGGAAACGCCTGGTATCTTTATAGTCCTGTCGGGTTTCGCCACCTCTGACTTGAGCGTCGATTTTTGTGATGCTCGTCAGGGGGGCGGA" +
      "GCCTATGGAAAAACGCCAGCAACGCGGCCTTTTTACGGTTCCTGGCCTTTTGCTGGCCTTTTGCTCACATGTTCTTTCCTGCGTTATCCCCTGATTCTGTGGATAACCGTATTACCGCCTTTGAGTGAGCTGATACCGCTCGCCGCAGCCGAACGACCGAGCGCAGCGAGTCAGTGAGCGAGGAAGCGGAAG",
  },
];

export const MOTIF_LIB_VERSION = "motif-lib v1";

/** Find every occurrence of `needle` in `haystack` (both uppercase). */
function findAll(haystack: string, needle: string): number[] {
  const out: number[] = [];
  let i = haystack.indexOf(needle);
  while (i !== -1) {
    out.push(i);
    i = haystack.indexOf(needle, i + 1);
  }
  return out;
}

/**
 * Scan `sequence` for every motif in the library, on both strands.
 * Returns hits sorted by start position. Overlapping hits of the same motif
 * are deduplicated (first hit kept).
 */
export function scanMotifs(sequence: string): MotifHit[] {
  const seq = sequence.toUpperCase().replace(/\s+/g, "");
  if (!seq) return [];
  const hits: MotifHit[] = [];

  for (const motif of MOTIF_LIBRARY) {
    const pattern = motif.pattern.toUpperCase();
    const raw: { idx: number; strand: 1 | -1 }[] = [
      ...findAll(seq, pattern).map((idx) => ({ idx, strand: 1 as const })),
      ...findAll(seq, reverseComplement(pattern)).map((idx) => ({ idx, strand: -1 as const })),
    ];
    raw.sort((a, b) => a.idx - b.idx);

    let lastEnd = -1;
    for (const hit of raw) {
      if (hit.idx <= lastEnd) continue; // overlaps a kept hit of the same motif
      lastEnd = hit.idx + pattern.length - 1;
      hits.push({
        name: motif.name,
        role_so: motif.role_so,
        start: hit.idx + 1,
        end: hit.idx + pattern.length,
        strand: hit.strand,
        source: `auto-annotated (${MOTIF_LIB_VERSION})`,
      });
    }
  }

  hits.sort((a, b) => a.start - b.start || a.end - b.end);
  return hits;
}
