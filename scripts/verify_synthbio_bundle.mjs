#!/usr/bin/env node
// HelixSynthBio evidence bundle verifier — recompute every hash in a bundle
// on a clean machine and report PASS/FAIL per section. No repo access needed.
//
// Usage:
//   node scripts/verify_synthbio_bundle.mjs bundle.json
//   (bundle.json = the `data` field of GET /v1/registry/designs/{id}/bundle)

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const sha256 = (s) => createHash("sha256").update(s, "utf8").digest("hex");

const normalizeSeq = (s) =>
  (s ?? "")
    .split("")
    .filter((c) => /[a-zA-Z]/.test(c))
    .map((c) => c.toUpperCase())
    .join("");

function fail(msg) {
  console.error(`FAIL: ${msg}`);
  process.exit(1);
}

const file = process.argv[2];
if (!file) fail("usage: node scripts/verify_synthbio_bundle.mjs bundle.json");

const bundle = JSON.parse(readFileSync(file, "utf8"));
const b = bundle.data ?? bundle;

// 1. Version content hashes.
let versionsOk = 0;
for (const v of b.versions ?? []) {
  const canonicalSeq = normalizeSeq(v.sequence_text);
  const expected = sha256(
    `${v.alphabet}|${v.topology}|${JSON.stringify(v.components)}|${canonicalSeq}`,
  );
  if (expected !== v.content_hash) {
    fail(
      `version ${v.version}: content_hash mismatch (bundle=${v.content_hash}, recomputed=${expected})`,
    );
  }
  if (v.sequence_length !== canonicalSeq.length) {
    fail(`version ${v.version}: sequence_length ${v.sequence_length} != ${canonicalSeq.length}`);
  }
  versionsOk++;
}

// 2. Event chain hashes (per entity, in ledger order).
let eventsOk = 0;
let prev = "";
for (const e of b.events ?? []) {
  const expected = sha256(
    `${prev}|${e.entity_kind}|${e.entity_id}|${e.event_kind}|${e.actor}|${JSON.stringify(e.details)}`,
  );
  if (expected !== e.content_hash) {
    fail(
      `event ${e.event_kind} (${e.created_at}): content_hash mismatch (bundle=${e.content_hash}, recomputed=${expected})`,
    );
  }
  if (e.prev_hash !== prev) {
    fail(`event ${e.event_kind}: prev_hash chain broken`);
  }
  prev = e.content_hash;
  eventsOk++;
}

// 3. Bundle hash.
let input = b.design?.accession ?? "";
for (const v of b.versions ?? []) input += `|${v.content_hash}`;
if (b.risk_case) {
  input += `|${b.risk_case.state}${b.risk_case.reviewer ?? ""}`;
}
for (const e of b.events ?? []) input += `|${e.content_hash}`;
const expectedBundle = sha256(input);
if (expectedBundle !== b.bundle_hash) {
  fail(`bundle_hash mismatch (bundle=${b.bundle_hash}, recomputed=${expectedBundle})`);
}

console.log(`PASS: ${versionsOk} version hashes, ${eventsOk} chained events, bundle hash ${b.bundle_hash.slice(0, 16)}… verified`);
