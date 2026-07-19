"use client";

import { Fragment, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import {
  claimStatusClass,
  evidenceSupportClass,
  fmtTime,
  listOf,
  riskClass,
  sbApi,
  shortHash,
  shortId,
  signatureSealClass,
  EVIDENCE_SUPPORTS,
  EVIDENCE_TARGET_KINDS,
  SIGNATURE_MEANINGS,
  type Bundle,
  type ClaimWithEvidence,
  type Component,
  type Design360Data,
  type DesignVersion,
  type ElnNote,
  type Signature,
} from "./lib";
import { SequenceMap } from "./SequenceMap";
import { LineageGraph } from "./LineageGraph";
import { EditTab } from "./EditTab";
import { AlignTab } from "./AlignTab";
import { scanMotifs, MOTIF_LIB_VERSION } from "./motifs";
import { isCds, translateComponent, translationPreview } from "./translate";

type Tab = "overview" | "versions" | "edit" | "align" | "claims" | "notes" | "lineage" | "bundle";

const TABS: { key: Tab; label: string }[] = [
  { key: "overview", label: "Overview" },
  { key: "versions", label: "Versions" },
  { key: "edit", label: "Edit" },
  { key: "align", label: "Align" },
  { key: "claims", label: "Claims" },
  { key: "notes", label: "Notes" },
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
  const [signatures, setSignatures] = useState<Signature[]>([]);
  const [signingRisk, setSigningRisk] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [tab, setTab] = useState<Tab>("overview");
  const [showVersionForm, setShowVersionForm] = useState(false);
  const [prefill, setPrefill] = useState<VersionPrefill | null>(null);
  const [selected, setSelected] = useState<number | null>(null);

  const load = useCallback(async () => {
    try {
      const [j, gj] = await Promise.all([
        sbApi<{ data: Design360Data }>(`/v1/registry/designs/${id}`),
        sbApi(`/v1/registry/designs/${id}/signatures`),
      ]);
      setData(j.data);
      setSignatures(listOf<Signature>(gj));
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
    setSigningRisk(false);
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
  const rc = data.risk_case;
  // a decided, unlocked risk case can be e-signed; locked cases refuse further signatures (422)
  const canSignDecision = rc != null && rc.state !== "unknown" && !rc.locked_at;

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
        {rc?.locked_at && (
          <span
            className="sb-seal sb-seal-locked"
            title={`decision signed & locked at ${fmtTime(rc.locked_at)}`}
          >
            ✒ signed &amp; locked
          </span>
        )}
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

      {(signatures.length > 0 || canSignDecision) && (
        <div className="sb-sig-strip">
          <span className="sb-meta-k">signatures</span>
          {signatures.map((s) => (
            <span
              key={s.id}
              className={`sb-seal ${signatureSealClass(s.meaning)}`}
              title={`${s.statement ? `${s.statement} · ` : ""}${s.target_kind} ${s.target_id} · hash ${s.content_hash}`}
            >
              <span className="sb-seal-mark">{s.meaning === "approved" ? "✓" : "✒"}</span>
              {s.meaning} · {s.signer}
              <span className="sb-seal-hash sb-mono" title={s.content_hash}>
                {shortHash(s.content_hash, 10)}
              </span>
              <span className="muted">{fmtTime(s.created_at)}</span>
            </span>
          ))}
          {canSignDecision && !signingRisk && (
            <button className="btn sm" style={{ marginLeft: "auto" }} onClick={() => setSigningRisk(true)}>
              Sign decision
            </button>
          )}
          {canSignDecision && signingRisk && (
            <SignForm
              targetKind="risk_case"
              targetId={rc.id}
              defaultMeaning="approved"
              onDone={async () => {
                setSigningRisk(false);
                await load();
              }}
              onError={setError}
              onFlash={flash}
            />
          )}
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
          onSigned={load}
          onError={setError}
          onFlash={flash}
        />
      )}
      {tab === "edit" && (
        <EditTab
          key={latest?.id ?? "no-version"}
          id={id}
          versions={data.versions}
          onSaved={load}
          onError={setError}
          onFlash={flash}
        />
      )}
      {tab === "align" && (
        <AlignTab
          key={latest?.id ?? "no-version"}
          versions={data.versions}
          onError={setError}
          onFlash={flash}
        />
      )}
      {tab === "lineage" && <Lineage data={data} />}
      {tab === "claims" && (
        <ClaimsTab id={id} versions={data.versions} onError={setError} onFlash={flash} />
      )}
      {tab === "notes" && <NotesTab id={id} onError={setError} onFlash={flash} />}
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
        {risk_case.locked_at && (
          <>
            {" "}
            <span className="sb-seal sb-seal-locked" title={`locked at ${fmtTime(risk_case.locked_at)}`}>
              ✒ locked
            </span>
          </>
        )}
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
  onSigned: () => Promise<void>;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const { id, versions, prefill, showForm, setShowForm, onCreated, onSigned, onError, onFlash } = props;
  const [busy, setBusy] = useState(false);
  const [staged, setStaged] = useState<Component[]>(prefill?.components ?? []);
  const [signingId, setSigningId] = useState<string | null>(null);
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
            <th className="acts">sign</th>
          </tr>
        </thead>
        <tbody>
          {sorted.length === 0 && (
            <tr>
              <td colSpan={10} className="empty">
                No versions yet.
              </td>
            </tr>
          )}
          {sorted.map((v) => (
            <Fragment key={v.id}>
              <tr>
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
                <td className="acts">
                  <button
                    className="btn sm ghost"
                    onClick={() => setSigningId(signingId === v.id ? null : v.id)}
                  >
                    {signingId === v.id ? "Close" : "Sign"}
                  </button>
                </td>
              </tr>
              {signingId === v.id && (
                <tr className="child-row">
                  <td colSpan={10}>
                    <SignForm
                      targetKind="design_version"
                      targetId={v.id}
                      defaultMeaning="reviewed"
                      onDone={async () => {
                        setSigningId(null);
                        await onSigned();
                      }}
                      onError={onError}
                      onFlash={onFlash}
                    />
                  </td>
                </tr>
              )}
            </Fragment>
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

/* ————————————————— E-signature form ————————————————— */

/** Inline sign form shared by the risk-decision strip and per-version rows. */
function SignForm({
  targetKind,
  targetId,
  defaultMeaning,
  onDone,
  onError,
  onFlash,
}: {
  targetKind: "design_version" | "risk_case";
  targetId: string;
  defaultMeaning: string;
  onDone: () => Promise<void>;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const [signer, setSigner] = useState("ops@helixforge.local");
  const [meaning, setMeaning] = useState(defaultMeaning);
  const [statement, setStatement] = useState("");
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!signer.trim()) {
      onError("Signer is required — the API rejects an empty signer (422).");
      return;
    }
    setBusy(true);
    try {
      await sbApi("/v1/signatures", {
        method: "POST",
        body: JSON.stringify({
          target_kind: targetKind,
          target_id: targetId,
          signer: signer.trim(),
          meaning,
          ...(statement.trim() ? { statement: statement.trim() } : {}),
        }),
      });
      onFlash(`Signature recorded (${meaning})`);
      await onDone();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <form className="create-form sb-sig-form" onSubmit={submit}>
      <label>
        <span>Signer *</span>
        <input value={signer} onChange={(e) => setSigner(e.target.value)} required />
      </label>
      <label>
        <span>Meaning</span>
        <select value={meaning} onChange={(e) => setMeaning(e.target.value)}>
          {SIGNATURE_MEANINGS.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
        </select>
      </label>
      <label className="sb-span-all">
        <span>Statement</span>
        <input
          value={statement}
          onChange={(e) => setStatement(e.target.value)}
          placeholder="optional — e.g. decision verified against biosafety-v1"
        />
      </label>
      <button className="btn primary" disabled={busy} type="submit">
        {busy ? "Signing…" : "Record signature"}
      </button>
    </form>
  );
}

/* ————————————————— Claims ————————————————— */

function ClaimsTab({
  id,
  versions,
  onError,
  onFlash,
}: {
  id: string;
  versions: DesignVersion[];
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const [items, setItems] = useState<ClaimWithEvidence[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [busy, setBusy] = useState(false);
  const [open, setOpen] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const j = await sbApi(`/v1/registry/designs/${id}/claims`);
      setItems(listOf<ClaimWithEvidence>(j));
    } catch (e) {
      onError(String(e instanceof Error ? e.message : e));
    } finally {
      setLoading(false);
    }
  }, [id, onError]);

  useEffect(() => {
    load();
  }, [load]);

  const submit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const f = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    setBusy(true);
    try {
      const j = await sbApi<{ data: { accession: string } }>("/v1/claims", {
        method: "POST",
        body: JSON.stringify({ design_id: id, statement: f.statement }),
      });
      setShowCreate(false);
      onFlash(`Claim ${j.data.accession} recorded`);
      await load();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel sb-panel-flush">
      <div className="panel-head">
        <h2>claims & evidence</h2>
        <button className="btn primary" onClick={() => setShowCreate((v) => !v)}>
          {showCreate ? "Close" : "New claim"}
        </button>
      </div>

      {showCreate && (
        <form className="create-form sb-form-wide" onSubmit={submit}>
          <label className="sb-span-all">
            <span>Statement *</span>
            <textarea
              name="statement"
              rows={3}
              required
              placeholder="e.g. Construct expresses demo enzyme at useful levels"
            />
          </label>
          <button className="btn primary" disabled={busy} type="submit">
            {busy ? "Recording…" : "Record claim"}
          </button>
        </form>
      )}

      <table className="etable">
        <thead>
          <tr>
            <th>accession</th>
            <th>statement</th>
            <th>status</th>
            <th>attested</th>
            <th>evidence</th>
            <th className="acts">review</th>
          </tr>
        </thead>
        <tbody>
          {!loading && items.length === 0 && (
            <tr>
              <td colSpan={6} className="empty">
                No claims yet — record the first statement.
              </td>
            </tr>
          )}
          {items.map((it) => (
            <ClaimRow
              key={it.claim.id}
              item={it}
              versions={versions}
              isOpen={open === it.claim.id}
              onToggle={() => setOpen(open === it.claim.id ? null : it.claim.id)}
              onChanged={load}
              onError={onError}
              onFlash={onFlash}
            />
          ))}
        </tbody>
      </table>
    </section>
  );
}

/** One claim row with an expandable evidence / attestation section. */
function ClaimRow({
  item,
  versions,
  isOpen,
  onToggle,
  onChanged,
  onError,
  onFlash,
}: {
  item: ClaimWithEvidence;
  versions: DesignVersion[];
  isOpen: boolean;
  onToggle: () => void;
  onChanged: () => Promise<void>;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const c = item.claim;
  const [busy, setBusy] = useState("");
  const [targetKind, setTargetKind] = useState<string>("design_version");
  const [attestor, setAttestor] = useState("ops@helixforge.local");
  const [reason, setReason] = useState("");
  const sortedVersions = [...versions].sort((a, b) => b.version - a.version);

  const linkEvidence = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const f = Object.fromEntries(new FormData(form).entries()) as Record<string, string>;
    const targetId = targetKind === "design_version" ? f.version_id : (f.target_id ?? "").trim();
    if (!targetId) {
      onError(
        targetKind === "design_version"
          ? "No version to link — commit a version first."
          : "Enter the target UUID (bad UUIDs are rejected by the API with 404).",
      );
      return;
    }
    const body: Record<string, unknown> = {
      target_kind: targetKind,
      target_id: targetId,
      support: f.support,
    };
    if (f.note) body.note = f.note;
    setBusy("evidence");
    try {
      await sbApi(`/v1/claims/${c.id}/evidence`, {
        method: "POST",
        body: JSON.stringify(body),
      });
      form.reset();
      onFlash(`Evidence linked to ${c.accession}`);
      await onChanged();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy("");
    }
  };

  const attest = async () => {
    setBusy("attest");
    try {
      await sbApi(`/v1/claims/${c.id}/attest`, {
        method: "POST",
        body: JSON.stringify({ attestor: attestor.trim() || "ops@helixforge.local" }),
      });
      onFlash(`Claim ${c.accession} attested`);
      await onChanged();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy("");
    }
  };

  const challenge = async () => {
    setBusy("challenge");
    try {
      await sbApi(`/v1/claims/${c.id}/challenge`, {
        method: "POST",
        body: JSON.stringify(reason.trim() ? { reason: reason.trim() } : {}),
      });
      setReason("");
      onFlash(`Claim ${c.accession} challenged`);
      await onChanged();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy("");
    }
  };

  return (
    <>
      <tr className={isOpen ? "open" : ""}>
        <td className="sb-mono">{c.accession}</td>
        <td>{c.statement}</td>
        <td>
          <span className={claimStatusClass(c.status)}>{c.status}</span>
        </td>
        <td className="muted">
          {c.attested_by ? `${c.attested_by} · ${fmtTime(c.attested_at)}` : "—"}
        </td>
        <td>
          {item.evidence.length === 0 ? (
            <span className="muted">—</span>
          ) : (
            <span className="sb-evi-list">
              {item.evidence.map((ev, i) => (
                <span
                  key={`${ev.target_kind}-${ev.target_id}-${i}`}
                  className={evidenceSupportClass(ev.support)}
                  title={ev.note || ev.support}
                >
                  {ev.support} · {ev.target_kind} {shortId(ev.target_id)}
                </span>
              ))}
            </span>
          )}
        </td>
        <td className="acts">
          <button className="btn sm" onClick={onToggle}>
            {isOpen ? "Close" : "Review"}
          </button>
        </td>
      </tr>
      {isOpen && (
        <tr className="child-row">
          <td colSpan={6}>
            {item.evidence.length === 0 ? (
              <p className="muted">No evidence linked yet.</p>
            ) : (
              <table className="etable">
                <thead>
                  <tr>
                    <th>support</th>
                    <th>target kind</th>
                    <th>target id</th>
                    <th>note</th>
                    <th>linked</th>
                  </tr>
                </thead>
                <tbody>
                  {item.evidence.map((ev, i) => (
                    <tr key={`${ev.target_kind}-${ev.target_id}-${i}`}>
                      <td>
                        <span className={evidenceSupportClass(ev.support)}>{ev.support}</span>
                      </td>
                      <td>
                        <span className="sb-kind">{ev.target_kind}</span>
                      </td>
                      <td className="sb-mono" title={ev.target_id}>
                        {shortId(ev.target_id)}
                      </td>
                      <td className="muted">{ev.note || "—"}</td>
                      <td className="muted">{fmtTime(ev.created_at)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}

            <form className="create-form sb-form-wide" onSubmit={linkEvidence}>
              <label>
                <span>Support</span>
                <select name="support" defaultValue="supports">
                  {EVIDENCE_SUPPORTS.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                <span>Target kind</span>
                <select value={targetKind} onChange={(e) => setTargetKind(e.target.value)}>
                  {EVIDENCE_TARGET_KINDS.map((k) => (
                    <option key={k} value={k}>
                      {k}
                    </option>
                  ))}
                </select>
              </label>
              {targetKind === "design_version" ? (
                <label>
                  <span>Version</span>
                  <select name="version_id">
                    {sortedVersions.map((v) => (
                      <option key={v.id} value={v.id}>
                        v{v.version} · {shortHash(v.content_hash, 8)}
                      </option>
                    ))}
                  </select>
                </label>
              ) : (
                <label>
                  <span>Target UUID</span>
                  <input name="target_id" className="sb-mono" placeholder="019f…" />
                </label>
              )}
              <label>
                <span>Note</span>
                <input name="note" placeholder="optional" />
              </label>
              <button className="btn primary" disabled={busy === "evidence"} type="submit">
                {busy === "evidence" ? "Linking…" : "Link evidence"}
              </button>
            </form>

            <div className="sb-verdict-bar">
              {(c.status === "draft" || c.status === "under_review") && (
                <>
                  <input
                    className="sb-analyst"
                    value={attestor}
                    onChange={(e) => setAttestor(e.target.value)}
                    title="attestor"
                    placeholder="attestor"
                  />
                  <button className="btn sm primary" disabled={busy !== ""} onClick={attest}>
                    {busy === "attest" ? "Attesting…" : "Attest"}
                  </button>
                </>
              )}
              {c.status !== "challenged" && (
                <>
                  <input
                    className="sb-analyst sb-reason"
                    value={reason}
                    onChange={(e) => setReason(e.target.value)}
                    placeholder="challenge reason (optional)"
                  />
                  <button className="btn sm" disabled={busy !== ""} onClick={challenge}>
                    {busy === "challenge" ? "Challenging…" : "Challenge"}
                  </button>
                </>
              )}
              {c.status === "challenged" && (
                <span className="sb-bad-text">
                  challenged{c.attested_by ? ` · prior attestation by ${c.attested_by} kept` : ""}
                </span>
              )}
            </div>
          </td>
        </tr>
      )}
    </>
  );
}

/* ————————————————— ELN Notes ————————————————— */

function NotesTab({
  id,
  onError,
  onFlash,
}: {
  id: string;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const [notes, setNotes] = useState<ElnNote[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    try {
      const j = await sbApi(`/v1/registry/designs/${id}/notes`);
      setNotes(listOf<ElnNote>(j));
    } catch (e) {
      onError(String(e instanceof Error ? e.message : e));
    } finally {
      setLoading(false);
    }
  }, [id, onError]);

  useEffect(() => {
    load();
  }, [load]);

  const submit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const f = Object.fromEntries(new FormData(form).entries()) as Record<string, string>;
    if (!f.body.trim()) {
      onError("Note body is empty.");
      return;
    }
    setBusy(true);
    try {
      await sbApi(`/v1/registry/designs/${id}/notes`, {
        method: "POST",
        body: JSON.stringify({ body: f.body }),
      });
      form.reset();
      onFlash("Note appended");
      await load();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy(false);
    }
  };

  const sorted = [...notes].sort((a, b) => b.created_at.localeCompare(a.created_at));

  return (
    <section className="panel sb-panel-flush">
      <div className="panel-head">
        <h2>eln notes</h2>
        <span className="muted">{notes.length} notes · append-only</span>
      </div>

      <form className="create-form sb-form-wide" onSubmit={submit}>
        <label className="sb-span-all">
          <span>New note *</span>
          <textarea
            name="body"
            rows={3}
            required
            placeholder="bench observation, decision, context…"
          />
        </label>
        <button className="btn primary" disabled={busy} type="submit">
          {busy ? "Appending…" : "Append note"}
        </button>
        <span className="muted sb-note-hint">
          notes are permanent — append-only, no edit or delete
        </span>
      </form>

      {!loading && sorted.length === 0 ? (
        <p className="muted">No notes yet — write the first bench entry.</p>
      ) : (
        <ol className="sb-timeline">
          {sorted.map((n) => (
            <li className="sb-tl-item" key={n.id}>
              <span className="sb-tl-dot ev-note" />
              <div className="sb-tl-notes">{n.body}</div>
              <div className="sb-tl-meta">
                {n.created_by} · {fmtTime(n.created_at)}
              </div>
            </li>
          ))}
        </ol>
      )}
    </section>
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
