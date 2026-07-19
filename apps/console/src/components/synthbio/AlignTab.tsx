"use client";

import { Fragment, useEffect, useState } from "react";
import {
  listOf,
  sbApi,
  wrapSeq,
  type DesignVersion,
  type Measurement,
  type Sample,
} from "./lib";
import { alignReads, SEED_LENGTH, type AlignResult, type AlignSummary } from "./lib/align";

type RunResult = { results: AlignResult[]; summary: AlignSummary };

/** Small valid 2-read demo derived from the reference: one exact substring + one with 2 deliberate mismatches (seed 12-mer kept intact). */
function demoReads(seq: string): string {
  const mut = (b: string) => ({ A: "C", C: "G", G: "T", T: "A" })[b] ?? "A";
  const r1 = seq.slice(0, Math.min(48, seq.length));
  const start2 = seq.length > 96 ? 24 : 0;
  const r2len = Math.min(48, seq.length - start2);
  const r2 = seq.slice(start2, start2 + r2len).split("");
  const p1 = Math.min(20, r2len - 1);
  const p2 = Math.min(35, r2len - 1);
  // mutate two bases past the seed 12-mer so the demo exercises seed-and-extend
  if (p1 > SEED_LENGTH) r2[p1] = mut(r2[p1]);
  if (p2 > SEED_LENGTH && p2 !== p1) r2[p2] = mut(r2[p2]);
  return [
    ">demo-exact substring of reference",
    wrapSeq(r1, 60),
    ">demo-2mm two deliberate mismatches",
    wrapSeq(r2.join(""), 60),
  ].join("\n");
}

function identityClass(pct: number): string {
  if (pct >= 99) return "sb-id-hi";
  if (pct >= 95) return "sb-id-mid";
  return "sb-id-lo";
}

/**
 * Read-to-reference alignment against the latest version's sequence.
 * Remounted per version by the parent (key = version id).
 */
export function AlignTab({
  versions,
  onError,
  onFlash,
}: {
  versions: DesignVersion[];
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const latest = [...versions].sort((a, b) => b.version - a.version)[0];
  const seq = (latest?.sequence_text ?? "").replace(/\s+/g, "").toUpperCase();

  const [fasta, setFasta] = useState("");
  const [result, setResult] = useState<RunResult | null>(null);
  const [openId, setOpenId] = useState<string | null>(null);
  const [samples, setSamples] = useState<Sample[]>([]);
  const [sampleId, setSampleId] = useState("");
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState<{ msr: string; smp: string; smpId: string } | null>(null);

  // samples for the "save as measurement" target picker
  useEffect(() => {
    let alive = true;
    sbApi("/v1/inventory/samples")
      .then((j) => {
        if (!alive) return;
        const items = listOf<Sample>(j).sort((a, b) => a.accession.localeCompare(b.accession));
        setSamples(items);
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  }, []);

  const run = () => {
    setSaved(null);
    setOpenId(null);
    try {
      setResult(alignReads(seq, latest.topology, fasta));
    } catch (e) {
      onError(String(e instanceof Error ? e.message : e));
    }
  };

  const save = async () => {
    if (!result) return;
    if (!sampleId) {
      onError("Pick a sample to save the result — measurements require a sample_id.");
      return;
    }
    setSaving(true);
    try {
      const j = await sbApi<{ data: Measurement }>("/v1/measurements", {
        method: "POST",
        body: JSON.stringify({
          sample_id: sampleId,
          kind: "ngs_qc",
          method: "seed-align (client)",
          value: result.summary.meanIdentityPct,
          unit: "% identity",
          raw: {
            ...result.summary,
            reference_length: seq.length,
            topology: latest.topology,
            design_version_id: latest.id,
            results: result.results.map((r) => ({
              ...r,
              mismatchPositions: r.mismatchPositions.slice(0, 20),
            })),
          },
        }),
      });
      const smp = samples.find((s) => s.id === sampleId);
      setSaved({ msr: j.data.accession, smp: smp?.accession ?? sampleId, smpId: sampleId });
      onFlash(`Alignment saved as ${j.data.accession}`);
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setSaving(false);
    }
  };

  if (!latest || !seq) {
    return (
      <section className="panel sb-panel-flush">
        <div className="panel-head">
          <h2>read alignment</h2>
        </div>
        <p className="muted">No version with a sequence to align against — commit a version first.</p>
      </section>
    );
  }

  return (
    <section className="panel sb-panel-flush">
      <div className="panel-head">
        <h2>read alignment</h2>
        <span className="muted">
          reference: v{latest.version} · {seq.length.toLocaleString()} bp · {latest.topology}
        </span>
      </div>

      {saved && (
        <div className="banner ok">
          Alignment saved as measurement <span className="sb-mono">{saved.msr}</span> on sample{" "}
          <a href={`/products/helix-synthbio/samples/${saved.smpId}`} className="sb-mono">
            {saved.smp}
          </a>{" "}
          — open the sample page.
        </div>
      )}

      <label className="sb-frag-label">
        <span>Reads (FASTA, one or more records)</span>
        <textarea
          rows={8}
          className="sb-mono sb-frag-input"
          value={fasta}
          placeholder={">read-1\nACGTACGTACGT…"}
          onChange={(e) => {
            setFasta(e.target.value);
            setSaved(null);
          }}
        />
      </label>
      <div className="sb-edit-actions">
        <button type="button" className="btn" onClick={() => setFasta(demoReads(seq))}>
          Load demo reads
        </button>
        <button type="button" className="btn primary" onClick={run}>
          Run alignment
        </button>
      </div>

      {result && (
        <>
          <div className="row sb-align-summary">
            <span className="pill">
              total: <b>{result.summary.total}</b>
            </span>
            <span className="pill sb-ok-text">
              exact: <b>{result.summary.exact}</b>
            </span>
            <span className="pill">
              aligned: <b>{result.summary.aligned}</b>
            </span>
            <span className="pill sb-bad-text">
              failed: <b>{result.summary.failed}</b>
            </span>
            <span className="pill">
              mean identity: <b>{result.summary.meanIdentityPct}%</b>
            </span>
          </div>

          <table className="etable">
            <thead>
              <tr>
                <th>read</th>
                <th>strand</th>
                <th className="num">offset</th>
                <th className="num">aligned</th>
                <th className="num">identity</th>
                <th>mismatches</th>
                <th>status</th>
              </tr>
            </thead>
            <tbody>
              {result.results.map((r) => {
                const scorable = r.status === "exact" || r.status === "aligned";
                const isOpen = openId === r.readName;
                return (
                  <Fragment key={r.readName}>
                    <tr>
                      <td>{r.readName}</td>
                      <td>{scorable ? <span className="sb-strand">{r.strand}</span> : <span className="muted">—</span>}</td>
                      <td className="num sb-mono">{scorable ? r.offset : "—"}</td>
                      <td className="num">{scorable ? `${r.alignedLength} bp` : "—"}</td>
                      <td className="num">
                        {scorable ? (
                          <span className={identityClass(r.identityPct)}>{r.identityPct}%</span>
                        ) : (
                          "—"
                        )}
                      </td>
                      <td>
                        {!scorable || r.mismatches === 0 ? (
                          <span className="muted">{scorable ? "0" : "—"}</span>
                        ) : (
                          <button
                            type="button"
                            className="btn sm ghost"
                            onClick={() => setOpenId(isOpen ? null : r.readName)}
                          >
                            {r.mismatches} {isOpen ? "▾" : "▸"}
                          </button>
                        )}
                      </td>
                      <td>
                        <span className={`sb-chip sb-al-${r.status}`}>{r.status}</span>
                      </td>
                    </tr>
                    {isOpen && r.mismatches > 0 && (
                      <tr className="child-row">
                        <td colSpan={7}>
                          <div className="sb-meta-k">
                            mismatch positions (1-based reference
                            {r.mismatches > r.mismatchPositions.length
                              ? `, first ${r.mismatchPositions.length} of ${r.mismatches}`
                              : ""}
                            )
                          </div>
                          <span className="sb-mono">{r.mismatchPositions.join(", ")}</span>
                        </td>
                      </tr>
                    )}
                  </Fragment>
                );
              })}
            </tbody>
          </table>

          <div className="sb-edit-actions">
            <select
              className="sb-sort sb-sample-select"
              value={sampleId}
              onChange={(e) => setSampleId(e.target.value)}
              aria-label="target sample"
            >
              <option value="">— pick a sample —</option>
              {samples.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.accession} — {s.name}
                </option>
              ))}
            </select>
            <button type="button" className="btn primary" disabled={saving} onClick={save}>
              {saving ? "Saving…" : "Save as measurement"}
            </button>
            {!sampleId && <span className="muted">pick a sample to save the result</span>}
          </div>
        </>
      )}
    </section>
  );
}
