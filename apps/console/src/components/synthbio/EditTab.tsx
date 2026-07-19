"use client";

import { useMemo, useState } from "react";
import {
  sbApi,
  type Component,
  type Design,
  type DesignVersion,
} from "./lib";
import {
  deleteRange,
  insertAt,
  replaceRange,
  shiftComponents,
  type EditResult,
} from "./lib/seqops";
import {
  digest,
  fragmentSequence,
  ENZYMES,
  type DigestResult,
} from "./lib/enzymes";
import { assembleOverlap, MIN_OVERLAP, type AssemblyResult } from "./lib/assemble";

type EditMode = "features" | "region" | "digest" | "assembly";

const MODES: { key: EditMode; label: string }[] = [
  { key: "features", label: "Feature editor" },
  { key: "region", label: "Region ops" },
  { key: "digest", label: "Digest" },
  { key: "assembly", label: "Assembly" },
];

type DigestState = { ok: true; result: DigestResult } | { ok: false; error: string };

/**
 * Sequence editing suite for the 360 page. Remounted per latest version by
 * the parent (key = version id), so all editor state resets after a save.
 */
export function EditTab({
  id,
  versions,
  onSaved,
  onError,
  onFlash,
}: {
  id: string;
  versions: DesignVersion[];
  onSaved: () => Promise<void>;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const [mode, setMode] = useState<EditMode>("features");
  const [enzymeSel, setEnzymeSel] = useState<string[]>(["EcoRI"]);

  const latest = [...versions].sort((a, b) => b.version - a.version)[0];
  const seq = useMemo(
    () => (latest?.sequence_text ?? "").replace(/\s+/g, "").toUpperCase(),
    [latest],
  );

  const digestState = useMemo<DigestState | null>(() => {
    if (!latest || !seq || enzymeSel.length === 0) return null;
    try {
      return { ok: true, result: digest(seq, latest.topology, enzymeSel) };
    } catch (e) {
      return { ok: false, error: e instanceof Error ? e.message : String(e) };
    }
  }, [latest, seq, enzymeSel]);

  if (!latest || !seq) {
    return (
      <section className="panel sb-panel-flush">
        <div className="panel-head">
          <h2>sequence editing</h2>
        </div>
        <p className="muted">No version with a sequence to edit — commit a version first.</p>
      </section>
    );
  }

  return (
    <section className="panel sb-panel-flush">
      <div className="panel-head">
        <h2>sequence editing</h2>
        <span className="muted">
          editing v{latest.version} · {seq.length.toLocaleString()} bp · {latest.topology}
        </span>
      </div>

      <div className="sb-edit-sub">
        <div className="sb-seg">
          {MODES.map((m) => (
            <button
              key={m.key}
              type="button"
              className={`sb-seg-btn${mode === m.key ? " active" : ""}`}
              onClick={() => setMode(m.key)}
            >
              {m.label}
            </button>
          ))}
        </div>
      </div>

      {mode === "features" && (
        <FeatureEditor id={id} version={latest} seq={seq} onSaved={onSaved} onError={onError} onFlash={onFlash} />
      )}
      {mode === "region" && (
        <RegionOps id={id} version={latest} seq={seq} onSaved={onSaved} onError={onError} onFlash={onFlash} />
      )}
      {mode === "digest" && (
        <DigestView
          seq={seq}
          enzymeSel={enzymeSel}
          setEnzymeSel={setEnzymeSel}
          digestState={digestState}
        />
      )}
      {mode === "assembly" && (
        <AssemblyView
          seq={seq}
          digestState={digestState}
          enzymeSel={enzymeSel}
          onError={onError}
        />
      )}
    </section>
  );
}

/* ————————————————— Feature editor ————————————————— */

const ROLE_OPTIONS: [string, string][] = [
  ["SO:0000167", "promoter"],
  ["SO:0000316", "CDS"],
  ["SO:0000139", "RBS"],
  ["SO:0000141", "terminator"],
  ["SO:0000704", "gene"],
  ["misc", "misc"],
];

type EditComp = Component & { key: number; dropped: boolean };

function FeatureEditor({
  id,
  version,
  seq,
  onSaved,
  onError,
  onFlash,
}: {
  id: string;
  version: DesignVersion;
  seq: string;
  onSaved: () => Promise<void>;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const [rows, setRows] = useState<EditComp[]>(() =>
    version.components.map((c, i) => ({ ...c, key: i, dropped: false })),
  );
  const [nextKey, setNextKey] = useState(version.components.length);
  const [showPreview, setShowPreview] = useState(false);
  const [busy, setBusy] = useState(false);

  const kept = rows.filter((r) => !r.dropped);
  const update = (key: number, patch: Partial<EditComp>) =>
    setRows((rs) => rs.map((r) => (r.key === key ? { ...r, ...patch } : r)));

  const addRow = () => {
    setRows((rs) => [
      ...rs,
      {
        key: nextKey,
        name: "",
        role_so: "misc",
        start: 1,
        end: Math.min(10, seq.length),
        strand: 1,
        source: "feature-editor",
        dropped: false,
      },
    ]);
    setNextKey((k) => k + 1);
  };

  const num = (v: string) => {
    const n = parseInt(v, 10);
    return Number.isNaN(n) ? 0 : n;
  };

  const save = async () => {
    for (const r of kept) {
      if (!r.name.trim()) {
        onError("Every kept component needs a name.");
        return;
      }
      if (r.start < 1 || r.end < r.start || r.end > seq.length) {
        onError(`Component "${r.name}": coordinates must satisfy 1 ≤ start ≤ end ≤ ${seq.length}.`);
        return;
      }
    }
    setBusy(true);
    try {
      await sbApi(`/v1/registry/designs/${id}/versions`, {
        method: "POST",
        body: JSON.stringify({
          alphabet: version.alphabet,
          topology: version.topology,
          sequence_text: seq,
          components: kept.map(({ name, role_so, start, end, strand, source }) => ({
            name: name.trim(),
            role_so,
            start,
            end,
            strand,
            source,
          })),
          provenance: "edited (feature editor)",
          notes: `feature editor: ${kept.length} kept, ${rows.length - kept.length} dropped`,
          source_kind: "edited",
          source_name: "feature-editor",
        }),
      });
      setShowPreview(false);
      onFlash("Edited version committed (feature editor)");
      await onSaved();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      <table className="etable">
        <thead>
          <tr>
            <th>name</th>
            <th>role</th>
            <th className="num">start</th>
            <th className="num">end</th>
            <th>strand</th>
            <th>source</th>
            <th className="acts">drop</th>
          </tr>
        </thead>
        <tbody>
          {rows.length === 0 && (
            <tr>
              <td colSpan={7} className="empty">
                No components on this version — add one below.
              </td>
            </tr>
          )}
          {rows.map((r) => (
            <tr key={r.key} className={r.dropped ? "sb-drop-row" : ""}>
              <td>
                <input
                  className="sb-cell-input"
                  value={r.name}
                  disabled={r.dropped}
                  onChange={(e) => update(r.key, { name: e.target.value })}
                />
              </td>
              <td>
                <select
                  className="sb-cell-input"
                  value={r.role_so}
                  disabled={r.dropped}
                  onChange={(e) => update(r.key, { role_so: e.target.value })}
                >
                  {!ROLE_OPTIONS.some(([v]) => v === r.role_so) && (
                    <option value={r.role_so}>{r.role_so}</option>
                  )}
                  {ROLE_OPTIONS.map(([v, label]) => (
                    <option key={v} value={v}>
                      {label}
                    </option>
                  ))}
                </select>
              </td>
              <td className="num">
                <input
                  className="sb-cell-input sb-cell-num"
                  type="number"
                  min={1}
                  value={r.start}
                  disabled={r.dropped}
                  onChange={(e) => update(r.key, { start: num(e.target.value) })}
                />
              </td>
              <td className="num">
                <input
                  className="sb-cell-input sb-cell-num"
                  type="number"
                  min={1}
                  value={r.end}
                  disabled={r.dropped}
                  onChange={(e) => update(r.key, { end: num(e.target.value) })}
                />
              </td>
              <td>
                <select
                  className="sb-cell-input"
                  value={r.strand >= 0 ? "1" : "-1"}
                  disabled={r.dropped}
                  onChange={(e) => update(r.key, { strand: Number(e.target.value) })}
                >
                  <option value="1">→</option>
                  <option value="-1">←</option>
                </select>
              </td>
              <td className="muted">{r.source}</td>
              <td className="acts">
                <button
                  type="button"
                  className="btn sm ghost"
                  onClick={() => update(r.key, { dropped: !r.dropped })}
                >
                  {r.dropped ? "undo" : "drop"}
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      <div className="sb-edit-actions">
        <button type="button" className="btn" onClick={addRow}>
          + Add component
        </button>
        <button
          type="button"
          className="btn primary"
          disabled={busy}
          onClick={() => setShowPreview((v) => !v)}
        >
          {showPreview ? "Close preview" : "Save as new version"}
        </button>
      </div>

      {showPreview && (
        <div className="sb-staged sb-preview">
          <div className="sb-meta-k">preview — new version (sequence unchanged)</div>
          <p className="muted">
            {kept.length} components kept · {rows.length - kept.length} dropped-marked · sequence:{" "}
            {seq.length.toLocaleString()} bp (unchanged) · provenance: edited (feature editor)
          </p>
          <div className="sb-edit-actions">
            <button type="button" className="btn primary" disabled={busy} onClick={save}>
              {busy ? "Committing…" : "Confirm & commit version"}
            </button>
            <button type="button" className="btn" disabled={busy} onClick={() => setShowPreview(false)}>
              Cancel
            </button>
          </div>
        </div>
      )}
    </>
  );
}

/* ————————————————— Region ops ————————————————— */

type RegionOp = "delete" | "insert" | "replace";

function RegionOps({
  id,
  version,
  seq,
  onSaved,
  onError,
  onFlash,
}: {
  id: string;
  version: DesignVersion;
  seq: string;
  onSaved: () => Promise<void>;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const [op, setOp] = useState<RegionOp>("delete");
  const [start, setStart] = useState(1);
  const [end, setEnd] = useState(1);
  const [insert, setInsert] = useState("");
  const [busy, setBusy] = useState(false);

  const preview = useMemo(() => {
    const ins = insert.replace(/\s+/g, "").toUpperCase();
    try {
      let res: EditResult;
      let spanStart: number;
      let spanEnd: number;
      let delta: number;
      if (op === "delete") {
        res = deleteRange(seq, version.topology, start, end);
        spanStart = start;
        spanEnd = end + 1;
        delta = -(end - start + 1);
      } else if (op === "insert") {
        res = insertAt(seq, version.topology, start, ins);
        spanStart = start;
        spanEnd = start; // zero-width span at the insert point
        delta = ins.length;
      } else {
        res = replaceRange(seq, version.topology, start, end, ins);
        spanStart = start;
        spanEnd = end + 1;
        delta = ins.length - (end - start + 1);
      }
      const shifted = shiftComponents(version.components, spanStart, spanEnd, delta, version.topology);
      const dropped = shifted.filter((c) => c.dropped);
      // ±30 bp snippet around the edit site in the NEW sequence
      const insLen = op === "delete" ? 0 : ins.length;
      const from = Math.max(0, start - 1 - 30);
      const to = Math.min(res.sequence.length, start - 1 + insLen + 30);
      return {
        ok: true as const,
        res,
        shifted,
        dropped,
        snippet: res.sequence.slice(from, to),
        clipLeft: from > 0,
        clipRight: to < res.sequence.length,
      };
    } catch (e) {
      return { ok: false as const, error: e instanceof Error ? e.message : String(e) };
    }
  }, [op, start, end, insert, seq, version]);

  const save = async () => {
    if (!preview.ok) return;
    setBusy(true);
    try {
      await sbApi(`/v1/registry/designs/${id}/versions`, {
        method: "POST",
        body: JSON.stringify({
          alphabet: version.alphabet,
          topology: version.topology,
          sequence_text: preview.res.sequence,
          components: preview.shifted
            .filter((c) => !c.dropped)
            .map(({ name, role_so, start: s, end: en, strand, source }) => ({
              name,
              role_so,
              start: s,
              end: en,
              strand,
              source,
            })),
          provenance: "edited (region op)",
          notes: `${op} ${op === "insert" ? `at ${start}` : `${start}..${end}`}${
            op !== "delete" ? ` (+${preview.res.length - seq.length + (op === "replace" ? end - start + 1 : 0)} bp insert)` : ""
          } · ${preview.dropped.length} components dropped`,
          source_kind: "edited",
          source_name: "region-ops",
        }),
      });
      onFlash("Edited version committed (region op)");
      await onSaved();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy(false);
    }
  };

  const num = (v: string) => {
    const n = parseInt(v, 10);
    return Number.isNaN(n) ? 0 : n;
  };

  return (
    <>
      <div className="create-form sb-form-wide">
        <label>
          <span>Operation</span>
          <select value={op} onChange={(e) => setOp(e.target.value as RegionOp)}>
            <option value="delete">delete</option>
            <option value="insert">insert</option>
            <option value="replace">replace</option>
          </select>
        </label>
        <label>
          <span>{op === "insert" ? "Position (insert before)" : "Start"}</span>
          <input type="number" min={1} value={start} onChange={(e) => setStart(num(e.target.value))} />
        </label>
        {op !== "insert" && (
          <label>
            <span>End</span>
            <input type="number" min={1} value={end} onChange={(e) => setEnd(num(e.target.value))} />
          </label>
        )}
        {op !== "delete" && (
          <label className="sb-span-all">
            <span>Insert sequence *</span>
            <textarea
              rows={3}
              className="sb-mono"
              value={insert}
              placeholder="ACGT…"
              onChange={(e) => setInsert(e.target.value)}
            />
          </label>
        )}
      </div>

      {preview.ok ? (
        <div className="sb-staged sb-preview">
          <div className="sb-meta-k">live preview</div>
          <div className="row">
            <span className="pill">
              length: <b>{seq.length.toLocaleString()}</b> → <b>{preview.res.length.toLocaleString()}</b>
            </span>
            <span className="pill">
              components: <b>{preview.shifted.length - preview.dropped.length}</b> kept
            </span>
            {preview.dropped.length > 0 && (
              <span className="pill sb-bad-text">
                dropped: <b>{preview.dropped.map((c) => c.name).join(", ")}</b>
              </span>
            )}
          </div>
          <div className="sb-meta-k">new sequence ±30 bp around the edit</div>
          <pre className="sb-seq sb-snippet">
            {preview.clipLeft ? "…" : ""}
            {preview.snippet}
            {preview.clipRight ? "…" : ""}
          </pre>
          <div className="sb-edit-actions">
            <button type="button" className="btn primary" disabled={busy} onClick={save}>
              {busy ? "Committing…" : "Save as new version"}
            </button>
          </div>
        </div>
      ) : (
        <p className="sb-bad-text sb-op-err">{preview.error}</p>
      )}
    </>
  );
}

/* ————————————————— Digest ————————————————— */

function DigestView({
  seq,
  enzymeSel,
  setEnzymeSel,
  digestState,
}: {
  seq: string;
  enzymeSel: string[];
  setEnzymeSel: (v: string[]) => void;
  digestState: DigestState | null;
}) {
  const toggle = (name: string) =>
    setEnzymeSel(
      enzymeSel.includes(name) ? enzymeSel.filter((n) => n !== name) : [...enzymeSel, name],
    );

  return (
    <>
      <div className="sb-chips sb-enzyme-chips" role="group" aria-label="enzymes">
        {ENZYMES.map((e) => (
          <button
            key={e.name}
            type="button"
            className={`sb-filter-chip${enzymeSel.includes(e.name) ? " active" : ""}`}
            title={`${e.recognition} (cut ${e.cutTop}/${e.cutBottom})`}
            onClick={() => toggle(e.name)}
          >
            {e.name}
          </button>
        ))}
      </div>

      {enzymeSel.length === 0 && <p className="muted">Select at least one enzyme.</p>}
      {digestState && !digestState.ok && <p className="sb-bad-text">{digestState.error}</p>}

      {digestState && digestState.ok && (
        <>
          <div className="row">
            {digestState.result.perEnzyme.map((p) => (
              <span key={p.enzyme} className="pill">
                {p.enzyme}: <b>{p.cuts.length}</b> cut{p.cuts.length === 1 ? "" : "s"}
                {p.cuts.length > 0 && <span className="muted"> · after {p.cuts.join(", ")}</span>}
              </span>
            ))}
            <span className="pill">
              fragments: <b>{digestState.result.fragments.length}</b>
              {digestState.result.uncut && (
                <span className="muted"> · uncut {digestState.result.circular ? "circle" : "molecule"}</span>
              )}
            </span>
          </div>

          <div className="sb-digest-grid">
            <table className="etable">
              <thead>
                <tr>
                  <th className="num">#</th>
                  <th className="num">start</th>
                  <th className="num">end</th>
                  <th className="num">size (bp)</th>
                  <th>span</th>
                </tr>
              </thead>
              <tbody>
                {digestState.result.fragments.map((f, i) => (
                  <tr key={i}>
                    <td className="num">{i + 1}</td>
                    <td className="num sb-mono">{f.start}</td>
                    <td className="num sb-mono">{f.end}</td>
                    <td className="num">{f.size.toLocaleString()}</td>
                    <td className="muted">{f.start > f.end ? "wraps origin" : "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>

            <GelLane fragments={digestState.result.fragments} uncut={digestState.result.uncut} seqLen={seq.length} />
          </div>
        </>
      )}
    </>
  );
}

/** Horizontal gel lane: bands at log-size positions (large fragments near top). */
function GelLane({
  fragments,
  uncut,
  seqLen,
}: {
  fragments: { size: number }[];
  uncut: boolean;
  seqLen: number;
}) {
  const maxSize = Math.max(seqLen, ...fragments.map((f) => f.size), 1);
  const minL = Math.log10(50);
  const maxL = Math.log10(maxSize);
  const pos = (size: number) => {
    const s = Math.log10(Math.max(size, 50));
    return (1 - (s - minL) / (maxL - minL || 1)) * 100;
  };
  const bands = [...fragments].sort((a, b) => b.size - a.size);
  return (
    <div className="sb-gel" title="simulated gel — bands at log10(size) positions">
      <div className="sb-gel-lane">
        <div className="sb-gel-well" />
        {bands.map((f, i) => (
          <div key={i} className="sb-gel-band" style={{ top: `${pos(f.size)}%` }} />
        ))}
      </div>
      <div className="sb-gel-labels">
        {bands.map((f, i) => (
          <div key={i} className="sb-gel-label" style={{ top: `${pos(f.size)}%` }}>
            {f.size.toLocaleString()} bp
          </div>
        ))}
      </div>
      {uncut && <div className="muted sb-gel-note">uncut — single full-length band</div>}
    </div>
  );
}

/* ————————————————— Assembly ————————————————— */

function AssemblyView({
  seq,
  digestState,
  enzymeSel,
  onError,
}: {
  seq: string;
  digestState: DigestState | null;
  enzymeSel: string[];
  onError: (m: string) => void;
}) {
  const [text, setText] = useState("");
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [created, setCreated] = useState<Design | null>(null);

  const fragments = useMemo(
    () =>
      text
        .split(/\n+/)
        .map((s) => s.replace(/\s+/g, "").toUpperCase())
        .filter(Boolean),
    [text],
  );

  const badFragment = fragments.findIndex((f) => f.length < MIN_OVERLAP || !/^[A-Z]+$/.test(f));

  const assembly = useMemo<{ ok: true; result: AssemblyResult } | { ok: false; error: string } | null>(() => {
    if (fragments.length === 0 || badFragment >= 0) return null;
    try {
      return { ok: true, result: assembleOverlap(fragments) };
    } catch (e) {
      return { ok: false, error: e instanceof Error ? e.message : String(e) };
    }
  }, [fragments, badFragment]);

  const addFragment = (s: string) => {
    setCreated(null);
    setText((t) => (t.trim() ? `${t.trim()}\n${s}` : s));
  };

  const save = async () => {
    if (!name.trim()) {
      onError("Name the new design first.");
      return;
    }
    if (!assembly || !assembly.ok) {
      onError(assembly && !assembly.ok ? assembly.error : "No valid assembly to save.");
      return;
    }
    setBusy(true);
    try {
      const j = await sbApi<{ data: Design }>("/v1/registry/designs", {
        method: "POST",
        body: JSON.stringify({
          name: name.trim(),
          alphabet: "dna",
          topology: assembly.result.topology,
          sequence_text: assembly.result.sequence,
          source_kind: "assembly",
          provenance: "assembled (overlap)",
          notes: `overlap assembly of ${fragments.length} fragments`,
        }),
      });
      setCreated(j.data);
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      {created && (
        <div className="banner ok">
          Design{" "}
          <a href={`/products/helix-synthbio/designs/${created.id}`} className="sb-mono">
            {created.accession}
          </a>{" "}
          assembled ({created.name}) — open its 360.
        </div>
      )}

      <div className="sb-assembly-grid">
        <div>
          <label className="sb-frag-label">
            <span>Fragments (one per line, ≥ {MIN_OVERLAP} bp each)</span>
            <textarea
              rows={8}
              className="sb-mono sb-frag-input"
              value={text}
              placeholder={"ACGTACGTACGT…\nTTTTGGGGCCCC…"}
              onChange={(e) => {
                setCreated(null);
                setText(e.target.value);
              }}
            />
          </label>

          {digestState && digestState.ok && digestState.result.fragments.length > 0 && (
            <div className="sb-from-digest">
              <span className="sb-meta-k">from digest ({enzymeSel.join(", ")})</span>
              <div className="sb-chips">
                {digestState.result.fragments.map((f, i) => (
                  <button
                    key={i}
                    type="button"
                    className="sb-filter-chip"
                    title={`fragment ${i + 1}: ${f.start}..${f.end}`}
                    onClick={() => addFragment(fragmentSequence(seq, f))}
                  >
                    + {f.size.toLocaleString()} bp
                  </button>
                ))}
                <button
                  type="button"
                  className="sb-filter-chip"
                  onClick={() =>
                    digestState.ok &&
                    addFragment(
                      digestState.result.fragments.map((f) => fragmentSequence(seq, f)).join("\n"),
                    )
                  }
                >
                  + all in order
                </button>
              </div>
            </div>
          )}
        </div>

        <div>
          {fragments.length === 0 ? (
            <p className="muted">Paste fragments or add them from the digest to preview the assembly.</p>
          ) : (
            <div className="sb-staged">
              <div className="sb-meta-k">order preview — {fragments.length} fragments</div>
              <table className="etable">
                <thead>
                  <tr>
                    <th className="num">#</th>
                    <th className="num">length</th>
                    <th>join to next</th>
                  </tr>
                </thead>
                <tbody>
                  {fragments.map((f, i) => {
                    const join = assembly && assembly.ok ? assembly.result.joins.find((j) => j.left === i) : undefined;
                    return (
                      <tr key={i} className={i === badFragment ? "sb-drop-row" : ""}>
                        <td className="num">{i}</td>
                        <td className="num">{f.length.toLocaleString()}</td>
                        <td className={join ? "" : "muted"}>
                          {join ? `overlap ${join.overlap} bp → ${join.right}` : i < fragments.length - 1 ? "—" : ""}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
              {badFragment >= 0 && (
                <p className="sb-bad-text">
                  Fragment {badFragment} is invalid — must be ≥ {MIN_OVERLAP} bp, letters only.
                </p>
              )}
              {assembly && !assembly.ok && <p className="sb-bad-text">{assembly.error}</p>}
              {assembly && assembly.ok && (
                <p className="muted">
                  assembled: <b>{assembly.result.sequence.length.toLocaleString()} bp</b> · topology{" "}
                  <b>{assembly.result.topology}</b>
                  {assembly.result.topology === "circular" && " (terminal overlap emitted once)"}
                </p>
              )}
            </div>
          )}

          <div className="sb-edit-actions">
            <input
              className="sb-analyst"
              value={name}
              placeholder="new design name *"
              onChange={(e) => setName(e.target.value)}
            />
            <button
              type="button"
              className="btn primary"
              disabled={busy || !assembly || !assembly.ok}
              onClick={save}
            >
              {busy ? "Assembling…" : "Save as new design"}
            </button>
          </div>
        </div>
      </div>
    </>
  );
}
