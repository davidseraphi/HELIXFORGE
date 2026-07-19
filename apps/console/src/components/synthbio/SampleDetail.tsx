"use client";

import { Fragment, useCallback, useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import {
  custodyTone,
  fmtTime,
  listOf,
  measurementStatusClass,
  sbApi,
  shortId,
  MEASUREMENT_KINDS,
  MOVE_EVENTS,
  type Measurement,
  type Sample,
  type SampleDetailData,
} from "./lib";

/**
 * Sample detail: header (accession, kind/status, location, linked design),
 * custody actions (move / aliquot), chain-of-custody timeline, lineage edges.
 * Data: GET /v1/inventory/samples/{id} → {sample, custody, edges, design_accession}.
 */
export function SampleDetail({ id }: { id: string }) {
  const router = useRouter();
  const [data, setData] = useState<SampleDetailData | null>(null);
  const [accessions, setAccessions] = useState<Record<string, string>>({});
  const [measurements, setMeasurements] = useState<Measurement[]>([]);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [child, setChild] = useState<Sample | null>(null);
  const [busy, setBusy] = useState<"" | "move" | "aliquot" | "measure">("");
  const [showMeasure, setShowMeasure] = useState(false);
  const [rawOpen, setRawOpen] = useState<string | null>(null);
  const [verdictBusy, setVerdictBusy] = useState("");
  const [analyst, setAnalyst] = useState("ops@helixforge.local");

  const load = useCallback(async () => {
    try {
      const [j, sj, mj] = await Promise.all([
        sbApi<{ data: SampleDetailData }>(`/v1/inventory/samples/${id}`),
        sbApi("/v1/inventory/samples"),
        sbApi(`/v1/inventory/samples/${id}/measurements`),
      ]);
      setData(j.data);
      // resolve accessions of linked samples for the lineage list
      const map: Record<string, string> = {};
      for (const s of listOf<Sample>(sj)) map[s.id] = s.accession;
      setAccessions(map);
      setMeasurements(listOf<Measurement>(mj));
    } catch (e) {
      setError(String(e instanceof Error ? e.message : e));
    }
  }, [id]);

  useEffect(() => {
    load();
  }, [load]);

  // reset per-sample UI state when navigating between samples
  useEffect(() => {
    setNotice("");
    setError("");
    setChild(null);
    setShowMeasure(false);
    setRawOpen(null);
    setVerdictBusy("");
  }, [id]);

  const flash = (msg: string) => {
    setNotice(msg);
    setTimeout(() => setNotice(""), 5000);
  };

  const submitMove = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const f = Object.fromEntries(new FormData(form).entries()) as Record<string, string>;
    const body: Record<string, unknown> = { event: f.event };
    if (f.to_location) body.to_location = f.to_location;
    if (f.notes) body.notes = f.notes;
    setBusy("move");
    setChild(null);
    try {
      await sbApi(`/v1/inventory/samples/${id}/custody`, {
        method: "POST",
        body: JSON.stringify(body),
      });
      form.reset();
      flash(`Custody event "${f.event}" recorded`);
      await load();
    } catch (err) {
      setError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy("");
    }
  };

  const submitAliquot = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const f = Object.fromEntries(new FormData(form).entries()) as Record<string, string>;
    setBusy("aliquot");
    try {
      const j = await sbApi<{ data: Sample }>(`/v1/inventory/samples/${id}/aliquot`, {
        method: "POST",
        body: JSON.stringify({ name: f.name }),
      });
      form.reset();
      setChild(j.data);
      setNotice("");
      await load();
    } catch (err) {
      setError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy("");
    }
  };

  const submitMeasurement = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const f = Object.fromEntries(new FormData(form).entries()) as Record<string, string>;
    const body: Record<string, unknown> = { sample_id: id, kind: f.kind };
    if (f.method) body.method = f.method;
    if (f.unit) body.unit = f.unit;
    if (f.value.trim() !== "") {
      const v = Number(f.value);
      if (Number.isNaN(v)) {
        setError("Value must be a number.");
        return;
      }
      body.value = v;
    }
    if (f.uncertainty.trim() !== "") {
      const u = Number(f.uncertainty);
      if (Number.isNaN(u)) {
        setError("Uncertainty must be a number.");
        return;
      }
      body.uncertainty = u;
    }
    if (f.raw.trim() !== "") {
      let parsed: unknown;
      try {
        parsed = JSON.parse(f.raw);
      } catch {
        setError("Raw JSON does not parse — fix it or clear the field.");
        return;
      }
      if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
        setError("Raw must be a JSON object (e.g. {\"plate\": \"A1\"}).");
        return;
      }
      body.raw = parsed;
    }
    if (body.value === undefined && body.raw === undefined) {
      setError("Provide a value or raw JSON content — empty measurements are rejected (422).");
      return;
    }
    setBusy("measure");
    setChild(null);
    try {
      const j = await sbApi<{ data: Measurement }>("/v1/measurements", {
        method: "POST",
        body: JSON.stringify(body),
      });
      setShowMeasure(false);
      flash(`Measurement ${j.data.accession} recorded as draft`);
      await load();
    } catch (err) {
      setError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy("");
    }
  };

  const verdict = async (m: Measurement, v: "accept" | "reject") => {
    setVerdictBusy(`${m.id}:${v}`);
    setChild(null);
    try {
      await sbApi(`/v1/measurements/${m.id}/${v}`, {
        method: "POST",
        body: JSON.stringify({ analyst: analyst.trim() || undefined }),
      });
      flash(`Measurement ${m.accession} ${v}ed`);
      await load();
    } catch (err) {
      setError(String(err instanceof Error ? err.message : err));
    } finally {
      setVerdictBusy("");
    }
  };

  if (error && !data) {
    return (
      <div className="app-down panel">
        <h1>Sample not available</h1>
        <p className="lead">{error}</p>
        <button className="btn" onClick={() => router.push("/products/helix-synthbio")}>
          ← Back to samples
        </button>
      </div>
    );
  }

  if (!data) {
    return <p className="muted">Loading sample…</p>;
  }

  const { sample } = data;
  const custody = [...data.custody].sort((a, b) => a.created_at.localeCompare(b.created_at));
  const parents = data.edges.filter((e) => e.child_id === sample.id);
  const children = data.edges.filter((e) => e.parent_id === sample.id);

  return (
    <div className="sb-app">
      <header className="sb-360-head">
        <button className="btn sm ghost" onClick={() => router.push("/products/helix-synthbio")}>
          ← samples
        </button>
        <div className="sb-360-title">
          <span className="sb-accession">{sample.accession}</span>
          <span className="sb-360-name">{sample.name}</span>
        </div>
        <span className="sb-kind">{sample.kind}</span>
        <span className={`pill status s-${sample.status}`}>{sample.status}</span>
        <span className="sb-loc" title="current location">
          ⌖ {sample.location || "unlocated"}
        </span>
        {sample.design_id && (
          <span className="pill">
            design:{" "}
            <a href={`/products/helix-synthbio/designs/${sample.design_id}`} className="sb-mono">
              {data.design_accession ?? shortId(sample.design_id)}
            </a>
          </span>
        )}
      </header>

      {notice && <div className="banner ok">{notice}</div>}
      {child && (
        <div className="banner ok">
          Aliquot registered:{" "}
          <a href={`/products/helix-synthbio/samples/${child.id}`} className="sb-mono">
            {child.accession}
          </a>{" "}
          ({child.name}) — derived-from {sample.accession}.
        </div>
      )}
      {error && (
        <div className="banner err" onClick={() => setError("")} title="dismiss">
          {error}
        </div>
      )}

      <section className="panel">
        <div className="panel-head">
          <h2>actions</h2>
          <span className="muted">
            {sample.status === "active" ? "sample is active" : `sample is ${sample.status} — actions may be rejected`}
          </span>
        </div>
        <div className="sb-actions">
          <form className="create-form" onSubmit={submitMove}>
            <label>
              <span>Custody event</span>
              <select name="event" defaultValue="transfer">
                {MOVE_EVENTS.map((ev) => (
                  <option key={ev} value={ev}>
                    {ev}
                  </option>
                ))}
              </select>
            </label>
            <label>
              <span>To location</span>
              <input name="to_location" placeholder={sample.location || "e.g. freezer-A · shelf 2"} />
            </label>
            <label className="sb-span-all">
              <span>Notes</span>
              <input name="notes" placeholder="optional" />
            </label>
            <button className="btn primary" disabled={busy === "move"} type="submit">
              {busy === "move" ? "Recording…" : "Record event"}
            </button>
          </form>

          <form className="create-form" onSubmit={submitAliquot}>
            <label className="sb-span-all">
              <span>Aliquot name *</span>
              <input name="name" placeholder={`${sample.name} · aliquot`} required />
            </label>
            <button className="btn primary" disabled={busy === "aliquot"} type="submit">
              {busy === "aliquot" ? "Creating…" : "Create aliquot"}
            </button>
          </form>
        </div>
      </section>

      <section className="panel">
        <div className="panel-head">
          <h2>measurements</h2>
          <div className="sb-m-head-tools">
            <input
              className="sb-analyst"
              value={analyst}
              onChange={(e) => setAnalyst(e.target.value)}
              placeholder="analyst"
              title="analyst recorded on accept / reject"
            />
            <button className="btn primary" onClick={() => setShowMeasure((v) => !v)}>
              {showMeasure ? "Close" : "Record measurement"}
            </button>
          </div>
        </div>

        {showMeasure && (
          <form className="create-form sb-form-wide" onSubmit={submitMeasurement}>
            <label>
              <span>Kind</span>
              <select name="kind" defaultValue="absorbance">
                {MEASUREMENT_KINDS.map((k) => (
                  <option key={k} value={k}>
                    {k}
                  </option>
                ))}
              </select>
            </label>
            <label>
              <span>Method</span>
              <input name="method" placeholder="e.g. plate reader" />
            </label>
            <label>
              <span>Value</span>
              <input name="value" type="number" step="any" placeholder="e.g. 0.42" />
            </label>
            <label>
              <span>Unit</span>
              <input name="unit" placeholder="e.g. AU" />
            </label>
            <label>
              <span>Uncertainty</span>
              <input name="uncertainty" type="number" step="any" placeholder="e.g. 0.01" />
            </label>
            <label className="sb-span-all">
              <span>Raw JSON</span>
              <textarea
                name="raw"
                rows={3}
                className="sb-mono"
                placeholder='{"plate": "A1"} (optional — required if no value)'
              />
            </label>
            <button className="btn primary" disabled={busy === "measure"} type="submit">
              {busy === "measure" ? "Recording…" : "Record measurement"}
            </button>
          </form>
        )}

        <table className="etable">
          <thead>
            <tr>
              <th>accession</th>
              <th>kind</th>
              <th>method</th>
              <th>value</th>
              <th>uncertainty</th>
              <th>status</th>
              <th>analyst</th>
              <th>created</th>
              <th className="acts">verdict</th>
            </tr>
          </thead>
          <tbody>
            {measurements.length === 0 && (
              <tr>
                <td colSpan={9} className="empty">
                  No measurements yet — record the first observation.
                </td>
              </tr>
            )}
            {measurements.map((m) => {
              const hasRaw = m.raw != null && Object.keys(m.raw).length > 0;
              const isOpen = rawOpen === m.id;
              return (
                <Fragment key={m.id}>
                  <tr
                    className={hasRaw ? "sb-rowlink" : ""}
                    onClick={hasRaw ? () => setRawOpen(isOpen ? null : m.id) : undefined}
                    title={hasRaw ? "click to inspect raw payload" : undefined}
                  >
                    <td className="sb-mono">{m.accession}</td>
                    <td>
                      <span className="sb-kind">{m.kind}</span>
                    </td>
                    <td className="muted">{m.method || "—"}</td>
                    <td className="sb-mono">
                      {m.value != null ? `${m.value}${m.unit ? ` ${m.unit}` : ""}` : "—"}
                    </td>
                    <td className="sb-u">{m.uncertainty != null ? `±${m.uncertainty}` : "—"}</td>
                    <td>
                      <span className={measurementStatusClass(m.status)}>{m.status}</span>
                    </td>
                    <td className="muted">{m.analyst || "—"}</td>
                    <td className="muted">{fmtTime(m.created_at)}</td>
                    <td className="acts">
                      {m.status === "draft" ? (
                        <span className="sb-m-verdict">
                          <button
                            className="btn sm"
                            disabled={verdictBusy !== ""}
                            onClick={(e) => {
                              e.stopPropagation();
                              verdict(m, "accept");
                            }}
                          >
                            {verdictBusy === `${m.id}:accept` ? "…" : "Accept"}
                          </button>
                          <button
                            className="btn sm ghost"
                            disabled={verdictBusy !== ""}
                            onClick={(e) => {
                              e.stopPropagation();
                              verdict(m, "reject");
                            }}
                          >
                            {verdictBusy === `${m.id}:reject` ? "…" : "Reject"}
                          </button>
                        </span>
                      ) : (
                        <span className="muted">—</span>
                      )}
                    </td>
                  </tr>
                  {isOpen && hasRaw && (
                    <tr className="child-row">
                      <td colSpan={9}>
                        <pre className="sb-seq">{JSON.stringify(m.raw, null, 2)}</pre>
                      </td>
                    </tr>
                  )}
                </Fragment>
              );
            })}
          </tbody>
        </table>
      </section>

      <section className="panel">
        <div className="panel-head">
          <h2>chain of custody</h2>
          <span className="muted">{custody.length} events</span>
        </div>
        {custody.length === 0 ? (
          <p className="muted">No custody events recorded.</p>
        ) : (
          <ol className="sb-timeline">
            {custody.map((ev) => (
              <li className="sb-tl-item" key={ev.id}>
                <span className={`sb-tl-dot ev-${custodyTone(ev.event)}`} />
                <div className="sb-tl-head">
                  <span className={`sb-ev sb-ev-${custodyTone(ev.event)}`}>{ev.event}</span>
                  <span className="sb-tl-loc">
                    {ev.from_location || "—"} → {ev.to_location || "—"}
                  </span>
                  <span className="muted sb-tl-when">{fmtTime(ev.created_at)}</span>
                </div>
                <div className="sb-tl-meta">{ev.actor}</div>
                {ev.notes && <div className="sb-tl-notes">{ev.notes}</div>}
              </li>
            ))}
          </ol>
        )}
      </section>

      <section className="panel">
        <div className="panel-head">
          <h2>lineage</h2>
          <span className="muted">{data.edges.length} edges</span>
        </div>
        {data.edges.length === 0 ? (
          <p className="muted">No lineage edges — the sample is not linked to a design or other samples.</p>
        ) : (
          <div>
            {parents.map((e) => (
              <div className="sb-edge-row" key={e.id}>
                <span className="sb-kind">{e.parent_kind}</span>
                {e.parent_kind === "design" ? (
                  <a className="sb-mono" href={`/products/helix-synthbio/designs/${e.parent_id}`}>
                    {data.design_accession ?? shortId(e.parent_id)}
                  </a>
                ) : (
                  <a className="sb-mono" href={`/products/helix-synthbio/samples/${e.parent_id}`}>
                    {accessions[e.parent_id] ?? shortId(e.parent_id)}
                  </a>
                )}
                <span className="sb-rel">—{e.relation}→</span>
                <span className="muted">this sample</span>
              </div>
            ))}
            {children.map((e) => (
              <div className="sb-edge-row" key={e.id}>
                <span className="muted">this sample</span>
                <span className="sb-rel">—{e.relation}→</span>
                <span className="sb-kind">{e.child_kind}</span>
                {e.child_kind === "design" ? (
                  <a className="sb-mono" href={`/products/helix-synthbio/designs/${e.child_id}`}>
                    {shortId(e.child_id)}
                  </a>
                ) : (
                  <a className="sb-mono" href={`/products/helix-synthbio/samples/${e.child_id}`}>
                    {accessions[e.child_id] ?? shortId(e.child_id)}
                  </a>
                )}
              </div>
            ))}
          </div>
        )}
      </section>

      <section className="panel">
        <div className="panel-head">
          <h2>metadata</h2>
        </div>
        <div className="sb-meta-grid">
          <div className="sb-meta-item">
            <div className="sb-meta-k">sample id</div>
            <div className="sb-meta-v sb-mono">{sample.id}</div>
          </div>
          <div className="sb-meta-item">
            <div className="sb-meta-k">registered by</div>
            <div className="sb-meta-v">{sample.created_by}</div>
          </div>
          <div className="sb-meta-item">
            <div className="sb-meta-k">created</div>
            <div className="sb-meta-v">{fmtTime(sample.created_at)}</div>
          </div>
          <div className="sb-meta-item">
            <div className="sb-meta-k">updated</div>
            <div className="sb-meta-v">{fmtTime(sample.updated_at)}</div>
          </div>
        </div>
      </section>
    </div>
  );
}
