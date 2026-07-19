/**
 * Overlap (Gibson-style) assembly — pure logic.
 *
 * Worked example (verified by hand):
 *   terminalOverlap("ACGTACGTXX", "XXTTTT") → 0
 *   (the longest suffix/prefix match is "XX" = 2 bp < 12, so the join fails
 *   as required and assembleOverlap would raise a validation error)
 */

import { SeqValidationError } from "./seqops";

export const MIN_OVERLAP = 12;
export const MAX_OVERLAP = 50;

/** Longest k ≥ minOverlap with suffix(a, k) === prefix(b, k), else 0. */
export function terminalOverlap(a: string, b: string, minOverlap = MIN_OVERLAP): number {
  return overlapBetween(a, b, minOverlap, Math.min(a.length, b.length));
}

function overlapBetween(a: string, b: string, min: number, max: number): number {
  for (let k = Math.min(max, a.length, b.length); k >= min; k--) {
    if (a.slice(a.length - k) === b.slice(0, k)) return k;
  }
  return 0;
}

export type AssemblyJoin = { left: number; right: number; overlap: number };

export type AssemblyResult = {
  sequence: string;
  topology: "circular" | "linear";
  joins: AssemblyJoin[];
};

/**
 * Merge fragments in order at their max terminal overlap (12..50 bp).
 * Throws a validation error naming the pair index when a join is < 12 bp.
 * When the last fragment's tail also overlaps the first fragment's head,
 * the result is circular and the terminal overlap is emitted only once.
 */
export function assembleOverlap(fragments: string[]): AssemblyResult {
  const frags = fragments.map((f) => f.trim().toUpperCase());
  if (frags.length === 0) throw new SeqValidationError("no fragments to assemble");
  frags.forEach((f, i) => {
    if (!f) throw new SeqValidationError(`fragment ${i + 1} is empty`);
    if (f.length < MIN_OVERLAP) {
      throw new SeqValidationError(
        `fragment ${i + 1} is ${f.length} bp — fragments must be ≥ ${MIN_OVERLAP} bp`,
      );
    }
  });
  if (frags.length === 1) return { sequence: frags[0], topology: "linear", joins: [] };

  const joins: AssemblyJoin[] = [];
  let sequence = frags[0];
  for (let i = 1; i < frags.length; i++) {
    const k = overlapBetween(frags[i - 1], frags[i], MIN_OVERLAP, MAX_OVERLAP);
    if (k < MIN_OVERLAP) {
      throw new SeqValidationError(
        `fragments ${i - 1} and ${i} (pair index ${i - 1}) overlap ${k} bp < ${MIN_OVERLAP} bp — cannot join`,
      );
    }
    joins.push({ left: i - 1, right: i, overlap: k });
    sequence += frags[i].slice(k);
  }

  let topology: "circular" | "linear" = "linear";
  const wrap = overlapBetween(frags[frags.length - 1], frags[0], MIN_OVERLAP, MAX_OVERLAP);
  if (wrap >= MIN_OVERLAP) {
    topology = "circular";
    joins.push({ left: frags.length - 1, right: 0, overlap: wrap });
    sequence = sequence.slice(0, sequence.length - wrap);
  }
  return { sequence, topology, joins };
}
