"use client";

import { useCallback, useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import {
  fmtTime,
  riskClass,
  sbApi,
  shortHash,
  shortId,
  wrapSeq,
  type Bundle,
  type Design360Data,
  type DesignVersion,
} from "./lib";

type Tab = "overview" | "versions" | "lineage" | "bundle";

const TABS: { key: Tab; label: string }[] = [
  { key: "overview", label: "Overview" },
  { key: "versions", label: "Versions" },
  { key: "lineage", label: "Lineage" },
  { key: "bundle", label: "Bundle" },
];

export function Design360({ id }: { id: string }) {
  const router = useRouter();
  const [data, setData] = useState<Design360Data | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [tab, setTab] = useState<Tab>("overview");
  const [showVersionForm, setShowVersionForm] = useState(false);

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

  const flash = (msg: string) => {
    setNotice(msg);
    setTimeout(() => setNotice(""), 4000);
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
        <button
          className="btn primary"
          style={{ marginLeft: "auto" }}
          onClick={() => {
            setTab("versions");
            setShowVersionForm(true);
          }}
        >
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

      {tab === "overview" && <Overview data={data} latest={latest} />}
      {tab === "versions" && (
        <Versions
          id={id}
          versions={data.versions}
          showForm={showVersionForm}
          setShowForm={setShowVersionForm}
          onCreated={async () => {
            setShowVersionForm(false);
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

function Overview({ data, latest }: { data: Design360Data; latest?: DesignVersion }) {
  const { design, risk_case } = data;
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

  const seq = latest?.sequence_text ?? "";
  const truncated = seq.length > 2000;
  const shown = truncated ? seq.slice(0, 2000) : seq;

  return (
    <>
      {design.description && <p className="lead sb-desc">{design.description}</p>}

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
                <tr key={i}>
                  <td>{c.name}</td>
                  <td>
                    <span className="sb-so">{c.role_so}</span>
                  </td>
                  <td className="num">
                    {c.start}–{c.end}
                  </td>
                  <td className="sb-strand">{c.strand >= 0 ? "→" : "←"}</td>
                  <td className="muted">{c.source}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>

      <section className="panel">
        <div className="panel-head">
          <h2>sequence</h2>
          <span className="muted">
            {latest ? `${latest.sequence_length} bp · v${latest.version}` : "no version"}
          </span>
        </div>
        {!seq ? (
          <p className="muted">No sequence deposited on the current version.</p>
        ) : (
          <>
            <pre className="sb-seq">{wrapSeq(shown)}</pre>
            {truncated && <p className="muted sb-seq-more">… and {seq.length - 2000} more bp</p>}
          </>
        )}
      </section>
    </>
  );
}

/* ————————————————— Versions ————————————————— */

function Versions(props: {
  id: string;
  versions: DesignVersion[];
  showForm: boolean;
  setShowForm: (v: boolean) => void;
  onCreated: () => Promise<void>;
  onError: (m: string) => void;
}) {
  const { id, versions, showForm, setShowForm, onCreated, onError } = props;
  const [busy, setBusy] = useState(false);
  const sorted = [...versions].sort((a, b) => b.version - a.version);
  const latest = sorted[0];

  const submit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const f = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    const body: Record<string, unknown> = { alphabet: f.alphabet, topology: f.topology };
    if (f.sequence_text) body.sequence_text = f.sequence_text;
    if (f.notes) body.notes = f.notes;
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
        <form className="create-form sb-form-wide" onSubmit={submit}>
          <label>
            <span>Alphabet</span>
            <select name="alphabet" defaultValue={latest?.alphabet ?? "dna"}>
              <option value="dna">dna</option>
              <option value="rna">rna</option>
              <option value="protein">protein</option>
            </select>
          </label>
          <label>
            <span>Topology</span>
            <select name="topology" defaultValue={latest?.topology ?? "circular"}>
              <option value="circular">circular</option>
              <option value="linear">linear</option>
            </select>
          </label>
          <label className="sb-span-all">
            <span>Sequence</span>
            <textarea name="sequence_text" rows={4} placeholder="ACGT… (optional)" />
          </label>
          <label className="sb-span-all">
            <span>Notes</span>
            <input name="notes" placeholder="what changed in this version" />
          </label>
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
            <th>content hash</th>
            <th>provenance</th>
            <th>created by</th>
            <th>created</th>
          </tr>
        </thead>
        <tbody>
          {sorted.length === 0 && (
            <tr>
              <td colSpan={8} className="empty">
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
          <h2>lineage edges</h2>
          <span className="muted">{data.edges.length}</span>
        </div>
        {data.edges.length === 0 ? (
          <p className="muted">No lineage edges recorded.</p>
        ) : (
          <table className="etable">
            <thead>
              <tr>
                <th>parent</th>
                <th>relation</th>
                <th>child</th>
                <th>created</th>
              </tr>
            </thead>
            <tbody>
              {data.edges.map((e) => (
                <tr key={e.id}>
                  <td>
                    <span className="sb-kind">{e.parent_kind}</span>{" "}
                    <span className="sb-mono muted">{shortId(e.parent_id)}</span>
                  </td>
                  <td>
                    <span className="sb-rel">{e.relation}</span>
                  </td>
                  <td>
                    <span className="sb-kind">{e.child_kind}</span>{" "}
                    <span className="sb-mono muted">{shortId(e.child_id)}</span>
                  </td>
                  <td className="muted">{fmtTime(e.created_at)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
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
