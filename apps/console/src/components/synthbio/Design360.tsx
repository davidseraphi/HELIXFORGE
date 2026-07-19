"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import {
  fmtTime,
  riskClass,
  sbApi,
  shortHash,
  shortId,
  type Bundle,
  type Component,
  type Design360Data,
  type DesignVersion,
} from "./lib";
import { SequenceMap } from "./SequenceMap";
import { LineageGraph } from "./LineageGraph";
import { scanMotifs, MOTIF_LIB_VERSION } from "./motifs";
import { isCds, translateComponent, translationPreview } from "./translate";

type Tab = "overview" | "versions" | "lineage" | "bundle";

const TABS: { key: Tab; label: string }[] = [
  { key: "overview", label: "Overview" },
  { key: "versions", label: "Versions" },
  { key: "lineage", label: "Lineage" },
  { key: "bundle", label: "Bundle" },
];

/** Prefill handed to the New-version form (motif auto-annotation). */
export type VersionPrefill = {
  sequence_text: string;
  alphabet: string;
  topology: string;
  provenance: string;
  notes: string;
  components: Component[];
};

export function Design360({ id }: { id: string }) {
  const router = useRouter();
  const [data, setData] = useState<Design360Data | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [tab, setTab] = useState<Tab>("overview");
  const [showVersionForm, setShowVersionForm] = useState(false);
  const [prefill, setPrefill] = useState<VersionPrefill | null>(null);
  const [selected, setSelected] = useState<number | null>(null);

  const load = useCallback(async () => {
    try {
      const j = await sbApi<{ data: Design360Data }>(`/v1/registry/designs/${id}`);
      setData(j.data);
    } catch (e) {
      setError(String(e instanceof Error ? e.message : e));
    }
  }, [id]);

  useEffect(() => {
    load();
  }, [load]);

  // reset per-design UI state when navigating between designs
  useEffect(() => {
    setTab("overview");
    setSelected(null);
    setPrefill(null);
    setShowVersionForm(false);
  }, [id]);

  const flash = (msg: string) => {
    setNotice(msg);
    setTimeout(() => setNotice(""), 4000);
  };

  const openVersionForm = (p: VersionPrefill | null) => {
    setPrefill(p);
    setShowVersionForm(true);
    setTab("versions");
  };

  if (error && !data) {
    return (
      <div className="app-down panel">
        <h1>Design not available</h1>
        <p className="lead">{error}</p>
        <button className="btn" onClick={() => router.push("/products/helix-synthbio")}>
          ← Back to registry
        </button>
      </div>
    );
  }

  if (!data) {
    return <p className="muted">Loading design 360°…</p>;
  }

  const { design } = data;
  const latest = [...data.versions].sort((a, b) => b.version - a.version)[0];

  return (
    <div className="sb-app">
      <header className="sb-360-head">
        <button className="btn sm ghost" onClick={() => router.push("/products/helix-synthbio")}>
          ← registry
        </button>
        <div className="sb-360-title">
          <span className="sb-accession">{design.accession}</span>
          <span className="sb-360-name">{design.name}</span>
        </div>
        <span className={riskClass(data.effective_risk)}>{data.effective_risk}</span>
        <span className={`pill status s-${design.status}`}>{design.status}</span>
        <button className="btn primary" style={{ marginLeft: "auto" }} onClick={() => openVersionForm(null)}>
          New version
        </button>
      </header>

      {notice && <div className="banner ok">{notice}</div>}
      {error && (
        <div className="banner err" onClick={() => setError("")} title="dismiss">
          {error}
        </div>
      )}

      <nav className="sb-tabs">
        {TABS.map((t) => (
          <button
            key={t.key}
            className={`sb-tab${tab === t.key ? " active" : ""}`}
            onClick={() => setTab(t.key)}
          >
            {t.label}
          </button>
        ))}
      </nav>

      {tab === "overview" && (
        <Overview data={data} latest={latest} selected={selected} onSelect={setSelected} onPrefill={openVersionForm} />
      )}
      {tab === "versions" && (
        <Versions
          id={id}
          versions={data.versions}
          prefill={prefill}
          showForm={showVersionForm}
          setShowForm={setShowVersionForm}
          onCreated={async () => {
            setShowVersionForm(false);
            setPrefill(null);
            flash("Version committed");
            await load();
          }}
          onError={setError}
        />
      )}
      {tab === "lineage" && <Lineage data={data} />}
      {tab === "bundle" && <BundleTab id={id} accession={design.accession} onError={setError} />}
    </div>
  );
}

/* ————————————————— Overview ————————————————— */

function Overview({
  data,
  latest,
  selected,
  onSelect,
  onPrefill,
}: {
  data: Design360Data;
  latest?: DesignVersion;
  selected: number | null;
  onSelect: (idx: number | null) => void;
  onPrefill: (p: VersionPrefill) => void;
}) {
  const { design, risk_case } = data;
  const rowRefs = useRef<(HTMLTableRowElement | null)[]>([]);

  const seq = latest?.sequence_text ?? "";
  const candidates = useMemo(() => {
    if (!seq) return [];
    const existing = new Set(
      (latest?.components ?? []).map((c) => `${c.name}:${c.start}:${c.end}`),
    );
    return scanMotifs(seq).filter((h) => !existing.has(`${h.name}:${h.start}:${h.end}`));
  }, [seq, latest?.components]);

  const selectFromMap = (idx: number | null) => {
    onSelect(idx);
    if (idx != null) {
      rowRefs.current[idx]?.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  };

  const stageMotifs = (hits: typeof candidates) => {
    if (!latest) return;
    const staged: Component[] = [
      ...latest.components,
      ...hits.map((h) => ({
        name: h.name,
        role_so: h.role_so,
        start: h.start,
        end: h.end,
        strand: h.strand as number,
        source: h.source,
      })),
    ];
    onPrefill({
      sequence_text: latest.sequence_text,
      alphabet: latest.alphabet,
      topology: latest.topology,
      provenance: `auto-annotated (${MOTIF_LIB_VERSION})`,
      notes: `auto-annotation: +${hits.map((h) => h.name).join(", ")}`,
      components: staged,
    });
  };

  const meta: [string, React.ReactNode][] = [
    ["alphabet", latest?.alphabet ?? "—"],
    ["topology", latest?.topology ?? "—"],
    ["access class", design.access_class],
    ["source", latest ? `${latest.source_kind}${latest.source_name ? ` · ${latest.source_name}` : ""}` : "—"],
    ["provenance", latest?.provenance ?? "—"],
    ["created", `${fmtTime(design.created_at)} · ${design.created_by}`],
  ];
  if (risk_case) {
    meta.push([
      "risk case",
      <span key="rc">
        <span className={riskClass(risk_case.state)}>{risk_case.state}</span>{" "}
        <span className="muted">
          {risk_case.reviewer} · {risk_case.policy_version || "no policy"}
        </span>
      </span>,
    ]);
  }

  return (
    <>
      {design.description && <p className="lead sb-desc">{design.description}</p>}

      <section className="panel">
        <div className="panel-head">
          <h2>sequence map</h2>
          <span className="muted">
            {latest ? `v${latest.version} · ${latest.sequence_length.toLocaleString()} bp · ${latest.components.length} features` : "no version"}
          </span>
        </div>
        {latest ? (
          <SequenceMap key={latest.id} version={latest} selected={selected} onSelect={selectFromMap} />
        ) : (
          <p className="muted">No version committed yet.</p>
        )}
      </section>

      <section className="panel">
        <div className="panel-head">
          <h2>components</h2>
          <span className="muted">{latest?.components.length ?? 0} on v{latest?.version ?? "—"}</span>
        </div>
        {!latest || latest.components.length === 0 ? (
          <p className="muted">No annotated components on the current version.</p>
        ) : (
          <table className="etable">
            <thead>
              <tr>
                <th>name</th>
                <th>role (SO)</th>
                <th className="num">start–end</th>
                <th>strand</th>
                <th>source</th>
              </tr>
            </thead>
            <tbody>
              {latest.components.map((c, i) => (
                <ComponentRow
                  key={i}
                  refIdx={(el) => {
                    rowRefs.current[i] = el;
                  }}
                  component={c}
                  seq={seq}
                  selected={selected === i}
                  onSelect={() => onSelect(selected === i ? null : i)}
                />
              ))}
            </tbody>
          </table>
        )}
      </section>

      {latest && seq && (
        <section className="panel">
          <div className="panel-head">
            <h2>annotation candidates</h2>
            <span className="muted">
              {candidates.length} motif {candidates.length === 1 ? "hit" : "hits"} · {MOTIF_LIB_VERSION}
            </span>
          </div>
          {candidates.length === 0 ? (
            <p className="muted">No library motifs found on the current version.</p>
          ) : (
            <>
              <table className="etable">
                <thead>
                  <tr>
                    <th>motif</th>
                    <th>role (SO)</th>
                    <th className="num">start–end</th>
                    <th>strand</th>
                    <th className="acts">annotate</th>
                  </tr>
                </thead>
                <tbody>
                  {candidates.map((h, i) => (
                    <tr key={`${h.name}-${h.start}-${i}`}>
                      <td>{h.name}</td>
                      <td>
                        <span className="sb-so">{h.role_so}</span>
                      </td>
                      <td className="num">
                        {h.start}–{h.end}
                      </td>
                      <td className="sb-strand">{h.strand >= 0 ? "→" : "←"}</td>
                      <td className="acts">
                        <button className="btn sm" onClick={() => stageMotifs([h])}>
                          Add as component
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {candidates.length > 1 && (
                <div className="sb-motif-all">
                  <button className="btn" onClick={() => stageMotifs(candidates)}>
                    Add all {candidates.length} as components
                  </button>
                  <span className="muted">opens the New-version form prefilled (provenance: auto-annotated · {MOTIF_LIB_VERSION})</span>
                </div>
              )}
            </>
          )}
        </section>
      )}

      <section className="panel">
        <div className="panel-head">
          <h2>metadata</h2>
        </div>
        <div className="sb-meta-grid">
          {meta.map(([k, v]) => (
            <div key={k} className="sb-meta-item">
              <div className="sb-meta-k">{k}</div>
              <div className="sb-meta-v">{v}</div>
            </div>
          ))}
        </div>
      </section>
    </>
  );
}

/** One component row; CDS rows carry an expandable auto-translation. */
function ComponentRow({
  component: c,
  seq,
  selected,
  onSelect,
  refIdx,
}: {
  component: Component;
  seq: string;
  selected: boolean;
  onSelect: () => void;
  refIdx: (el: HTMLTableRowElement | null) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const protein = isCds(c.role_so) && seq ? translateComponent(seq, c.start, c.end, c.strand) : "";

  return (
    <>
      <tr
        ref={refIdx}
        className={`sb-rowlink${selected ? " sb-row-selected" : ""}`}
        onClick={onSelect}
        title="click to highlight on the map"
      >
        <td>
          {c.name}
          {protein && (
            <button
              type="button"
              className="sb-translation-toggle"
              onClick={(e) => {
                e.stopPropagation();
                setExpanded((v) => !v);
              }}
              title="auto-translation (click to expand)"
            >
              {translationPreview(protein)} {expanded ? "▾" : "▸"}
            </button>
          )}
        </td>
        <td>
          <span className="sb-so">{c.role_so}</span>
        </td>
        <td className="num">
          {c.start}–{c.end}
        </td>
        <td className="sb-strand">{c.strand >= 0 ? "→" : "←"}</td>
        <td className="muted">{c.source}</td>
      </tr>
      {expanded && protein && (
        <tr className="child-row">
          <td colSpan={5}>
            <pre className="sb-seq sb-translation">
              {protein.replace(new RegExp(`(.{60})`, "g"), "$1\n")}
            </pre>
          </td>
        </tr>
      )}
    </>
  );
}

/* ————————————————— Versions ————————————————— */

function Versions(props: {
  id: string;
  versions: DesignVersion[];
  prefill: VersionPrefill | null;
  showForm: boolean;
  setShowForm: (v: boolean) => void;
  onCreated: () => Promise<void>;
  onError: (m: string) => void;
}) {
  const { id, versions, prefill, showForm, setShowForm, onCreated, onError } = props;
  const [busy, setBusy] = useState(false);
  const [staged, setStaged] = useState<Component[]>(prefill?.components ?? []);
  const sorted = [...versions].sort((a, b) => b.version - a.version);
  const latest = sorted[0];

  const submit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const f = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    const body: Record<string, unknown> = { alphabet: f.alphabet, topology: f.topology };
    if (f.sequence_text) body.sequence_text = f.sequence_text;
    if (f.notes) body.notes = f.notes;
    if (f.provenance) body.provenance = f.provenance;
    if (staged.length > 0) body.components = staged;
    setBusy(true);
    try {
      await sbApi(`/v1/registry/designs/${id}/versions`, {
        method: "POST",
        body: JSON.stringify(body),
      });
      await onCreated();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel sb-panel-flush">
      <div className="panel-head">
        <h2>immutable versions</h2>
        <button className="btn primary" onClick={() => setShowForm(!showForm)}>
          {showForm ? "Close" : "New version"}
        </button>
      </div>

      {showForm && (
        <form className="create-form sb-form-wide" key={prefill ? "prefilled" : "blank"} onSubmit={submit}>
          <label>
            <span>Alphabet</span>
            <select name="alphabet" defaultValue={prefill?.alphabet ?? latest?.alphabet ?? "dna"}>
              <option value="dna">dna</option>
              <option value="rna">rna</option>
              <option value="protein">protein</option>
            </select>
          </label>
          <label>
            <span>Topology</span>
            <select name="topology" defaultValue={prefill?.topology ?? latest?.topology ?? "circular"}>
              <option value="circular">circular</option>
              <option value="linear">linear</option>
            </select>
          </label>
          <label className="sb-span-all">
            <span>Sequence</span>
            <textarea
              name="sequence_text"
              rows={4}
              placeholder="ACGT… (optional)"
              defaultValue={prefill?.sequence_text ?? ""}
            />
          </label>
          <label>
            <span>Provenance</span>
            <input name="provenance" placeholder="e.g. benchling import" defaultValue={prefill?.provenance ?? ""} />
          </label>
          <label>
            <span>Notes</span>
            <input name="notes" placeholder="what changed in this version" defaultValue={prefill?.notes ?? ""} />
          </label>

          {staged.length > 0 && (
            <div className="sb-span-all sb-staged">
              <div className="sb-meta-k">components staged on this version ({staged.length})</div>
              <table className="etable">
                <thead>
                  <tr>
                    <th>name</th>
                    <th>role (SO)</th>
                    <th className="num">start–end</th>
                    <th>strand</th>
                    <th>source</th>
                    <th className="acts" />
                  </tr>
                </thead>
                <tbody>
                  {staged.map((c, i) => (
                    <tr key={`${c.name}-${c.start}-${i}`}>
                      <td>{c.name}</td>
                      <td>
                        <span className="sb-so">{c.role_so}</span>
                      </td>
                      <td className="num">
                        {c.start}–{c.end}
                      </td>
                      <td className="sb-strand">{c.strand >= 0 ? "→" : "←"}</td>
                      <td className="muted">{c.source}</td>
                      <td className="acts">
                        <button
                          type="button"
                          className="btn sm ghost"
                          onClick={() => setStaged((s) => s.filter((_, k) => k !== i))}
                        >
                          remove
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          <button className="btn primary" disabled={busy} type="submit">
            {busy ? "Committing…" : "Commit version"}
          </button>
        </form>
      )}

      <table className="etable">
        <thead>
          <tr>
            <th className="num">v</th>
            <th>alphabet / topology</th>
            <th>source</th>
            <th className="num">length</th>
            <th className="num">features</th>
            <th>content hash</th>
            <th>provenance</th>
            <th>created by</th>
            <th>created</th>
          </tr>
        </thead>
        <tbody>
          {sorted.length === 0 && (
            <tr>
              <td colSpan={9} className="empty">
                No versions yet.
              </td>
            </tr>
          )}
          {sorted.map((v) => (
            <tr key={v.id}>
              <td className="num">{v.version}</td>
              <td>
                {v.alphabet} · {v.topology}
              </td>
              <td className="muted">
                {v.source_kind}
                {v.source_name ? ` · ${v.source_name}` : ""}
              </td>
              <td className="num">{v.sequence_length}</td>
              <td className="num">{v.components.length}</td>
              <td className="sb-mono" title={v.content_hash}>
                {shortHash(v.content_hash, 12)}
              </td>
              <td className="muted">{v.provenance || "—"}</td>
              <td className="muted">{v.created_by}</td>
              <td className="muted">{fmtTime(v.created_at)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

/* ————————————————— Lineage ————————————————— */

function Lineage({ data }: { data: Design360Data }) {
  return (
    <>
      <section className="panel">
        <div className="panel-head">
          <h2>lineage graph</h2>
          <span className="muted">{data.edges.length} edges</span>
        </div>
        <LineageGraph data={data} />
      </section>

      <section className="panel">
        <div className="panel-head">
          <h2>event chain</h2>
          <span className="muted">{data.events.length} hash-chained</span>
        </div>
        {data.events.length === 0 ? (
          <p className="muted">No lineage events recorded.</p>
        ) : (
          <table className="etable">
            <thead>
              <tr>
                <th>event</th>
                <th>actor</th>
                <th>content hash</th>
                <th>prev hash</th>
                <th>created</th>
              </tr>
            </thead>
            <tbody>
              {data.events.map((ev) => (
                <tr key={ev.id}>
                  <td>
                    <span className="sb-kind">{ev.event_kind}</span>
                  </td>
                  <td>{ev.actor}</td>
                  <td className="sb-mono" title={ev.content_hash}>
                    {shortHash(ev.content_hash, 16)}
                  </td>
                  <td className="sb-mono muted" title={ev.prev_hash}>
                    {shortHash(ev.prev_hash, 16)}
                  </td>
                  <td className="muted">{fmtTime(ev.created_at)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
    </>
  );
}

/* ————————————————— Bundle ————————————————— */

function BundleTab({ id, accession, onError }: { id: string; accession: string; onError: (m: string) => void }) {
  const [bundle, setBundle] = useState<Bundle | null>(null);
  const [busy, setBusy] = useState(false);

  const fetchBundle = async (download: boolean) => {
    setBusy(true);
    try {
      const j = await sbApi<{ data: Bundle }>(`/v1/registry/designs/${id}/bundle`);
      setBundle(j.data);
      if (download) {
        const blob = new Blob([JSON.stringify(j.data, null, 2)], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `${accession}-evidence-bundle.json`;
        document.body.appendChild(a);
        a.click();
        a.remove();
        URL.revokeObjectURL(url);
      }
    } catch (e) {
      onError(String(e instanceof Error ? e.message : e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel">
      <div className="panel-head">
        <h2>evidence bundle</h2>
        <button className="btn primary" disabled={busy} onClick={() => fetchBundle(true)}>
          {busy ? "Generating…" : "Download evidence bundle"}
        </button>
      </div>

      {!bundle ? (
        <p className="muted">
          Generate the signed evidence bundle for {accession}: design, versions, risk case, lineage
          edges and the hash-chained event log — one JSON artifact for audit.
        </p>
      ) : (
        <>
          <div className="row">
            <span className="pill">
              bundle v<b>{bundle.bundle_version}</b>
            </span>
            <span className="pill">
              generated: <b>{fmtTime(bundle.generated_at)}</b>
            </span>
            <span className="pill">
              versions: <b>{bundle.versions.length}</b>
            </span>
            <span className="pill">
              events: <b>{bundle.events.length}</b>
            </span>
            <span className="pill">
              edges: <b>{bundle.edges.length}</b>
            </span>
            <span className="pill">
              risk: <b className={riskClass(bundle.risk_case?.state ?? "unknown")}>{bundle.risk_case?.state ?? "unknown"}</b>
            </span>
          </div>
          <div className="sb-hashline">
            <span className="sb-meta-k">bundle hash</span>
            <span className="sb-mono">{bundle.bundle_hash}</span>
          </div>
        </>
      )}
    </section>
  );
}
