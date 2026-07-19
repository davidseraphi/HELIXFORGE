/**
 * Region operations on sequences — pure, unit-testable logic.
 *
 * Coordinates are GenBank-style 1-based inclusive. Circular topology is
 * supported for display ops; edit ops require NON-wrapping ranges
 * (start ≤ end) even on circular molecules, and validate that.
 *
 * Character validation: letters A–Z only (case-insensitive). IUPAC
 * nucleotide letters are the intended content and the UI normalizes to
 * uppercase, but the check deliberately accepts any letter so the worked
 * examples below (plain-alphabet strings, lowercase insert) stay valid:
 *
 *   deleteRange("ABCDEFGH", "linear", 3, 5)        → { sequence: "ABFGH", length: 5 }
 *   insertAt("ABCD", "linear", 3, "XY")            → { sequence: "ABXYCD", length: 6 }
 *   replaceRange("ABCDEF", "linear", 2, 4, "z")    → { sequence: "AzEF", length: 6 }
 *
 * shiftComponents uses a half-open edit span [start, end) plus a signed
 * length delta. A delete of 1-based 3..5 inclusive is the span [3, 6)
 * with delta −3:
 *
 *   shiftComponents([{10..20}, {1..2}, {4..9}], 3, 6, −3)
 *     → {7..17} (fully after, shifted by −3)
 *       {1..2}  (fully before, untouched)
 *       {4..9}  (boundary cut → { dropped: true })
 */

export type Topology = string; // "circular" | "linear"

export type EditResult = { sequence: string; length: number };

/** Typed validation failure: `{code: "validation", message}` as an Error. */
export class SeqValidationError extends Error {
  readonly code = "validation";
  constructor(message: string) {
    super(message);
    this.name = "SeqValidationError";
  }
}

const LETTERS_RE = /^[A-Za-z]*$/;

function checkSeq(seq: string, label: string): void {
  if (!LETTERS_RE.test(seq)) {
    throw new SeqValidationError(
      `${label} must contain IUPAC letters only (A–Z, no digits/whitespace/symbols)`,
    );
  }
}

function checkRange(seq: string, topology: Topology, start: number, end: number): void {
  if (!Number.isInteger(start) || !Number.isInteger(end)) {
    throw new SeqValidationError(`range bounds must be integers (got ${start}..${end})`);
  }
  if (start > end) {
    throw new SeqValidationError(
      `range ${start}..${end} wraps the origin or is inverted — edit ops require start ≤ end` +
        (topology === "circular" ? " even on circular molecules" : ""),
    );
  }
  if (start < 1) {
    throw new SeqValidationError(`range start ${start} is out of bounds (1-based, minimum 1)`);
  }
  if (end > seq.length) {
    throw new SeqValidationError(
      `range end ${end} exceeds the sequence length ${seq.length}`,
    );
  }
}

/** Removes [start..end] (1-based inclusive). */
export function deleteRange(
  seq: string,
  topology: Topology,
  start: number,
  end: number,
): EditResult {
  checkSeq(seq, "sequence");
  checkRange(seq, topology, start, end);
  const sequence = seq.slice(0, start - 1) + seq.slice(end);
  return { sequence, length: sequence.length };
}

/** Inserts `insert` before `position` (1-based; 1..len+1). */
export function insertAt(
  seq: string,
  topology: Topology,
  position: number,
  insert: string,
): EditResult {
  checkSeq(seq, "sequence");
  checkSeq(insert, "insert");
  if (insert.length === 0) {
    throw new SeqValidationError("insert must be non-empty");
  }
  if (!Number.isInteger(position) || position < 1 || position > seq.length + 1) {
    throw new SeqValidationError(
      `insert position ${position} is out of bounds (allowed 1..${seq.length + 1})`,
    );
  }
  const sequence = seq.slice(0, position - 1) + insert + seq.slice(position - 1);
  return { sequence, length: sequence.length };
}

/** Delete [start..end] and insert `insert` at the same site. */
export function replaceRange(
  seq: string,
  topology: Topology,
  start: number,
  end: number,
  insert: string,
): EditResult {
  const del = deleteRange(seq, topology, start, end);
  return insertAt(del.sequence, topology, start, insert);
}

export type FeatureSpan = { start: number; end: number };

/**
 * Adjust component coordinates for an edit spanning the half-open range
 * [start, end) whose net length change is `delta` (negative = shrink).
 * For a zero-width insert at position p use start = end = p.
 *
 * - component fully before the edit (end < start)  → unchanged
 * - component fully after the edit (start ≥ end)   → shifted by delta
 * - component cut by an edit boundary              → marked dropped: true
 *   (safe default: never emit a corrupted span into a new version)
 */
export function shiftComponents<T extends FeatureSpan>(
  components: T[],
  start: number,
  end: number,
  delta: number,
  topology: Topology = "linear",
): (T & { dropped?: true })[] {
  if (!Number.isInteger(start) || !Number.isInteger(end) || start < 1 || start > end) {
    throw new SeqValidationError(
      `edit span [${start}, ${end}) is invalid — need 1 ≤ start ≤ end (non-wrapping` +
        (topology === "circular" ? ", even on circular molecules" : "") +
        ")",
    );
  }
  if (!Number.isInteger(delta)) {
    throw new SeqValidationError(`delta must be an integer (got ${delta})`);
  }
  return components.map((c) => {
    if (c.end < start) return { ...c };
    if (c.start >= end) return { ...c, start: c.start + delta, end: c.end + delta };
    return { ...c, dropped: true as const };
  });
}
