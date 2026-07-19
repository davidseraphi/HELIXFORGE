"use client";

import { useCallback, useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import {
  DEMO_GENBANK,
  fmtTime,
  listOf,
  riskClass,
  sbApi,
  type Design,
  type ImportManifest,
  type QueueItem,
} from "./lib";

type Tab = "registry" | "risk" | "import";

const TABS: { key: Tab; label: string }[] = [
  { key: "registry", label: "Registry" },
  { key: "risk", label: "Risk Review" },
  { key: "import", label: "Import" },
];

export function SynthBioApp() {
  const [tab, setTab] = useState<Tab>("registry");
  const [health, setHealth] = useState<"checking" | "up" | "down">("checking");
  const [notice, setNotice] = useState("");
  const [error, setError] = useState("");

  const flash = (msg: string) => {
    setNotice(msg);
    setError("");
    setTimeout(() => setNotice(""), 4000);
  };
  const fail = (msg: string) => {
    setError(msg);
    setNotice("");
  };

  useEffect(() => {
    let alive = true;
    sbApi("/healthz")
      .then(() => alive && setHealth("up"))
      .catch(() => alive && setHealth("down"));
    return () => {
      alive = false;
    };
  }, []);

  if (health === "down") {
    return (
      <div className="app-down panel">
        <h1>HelixSynthBio</h1>
        <p className="lead">
          The HelixSynthBio API is not answering on :8111. Start the product
          suite with <code>scripts/dev-products.ps1</code>, then reload.
        </p>
      </div>
    );
  }

  return (
    <div className="sb-app">
      <header className="papp-head">
        <div className="app-glyph lg">Sb</div>
        <div>
          <h1>HelixSynthBio</h1>
          <p className="muted">Sequence design registry · localhost:8111</p>
        </div>
        <div className={`app-state ${health}`} style={{ marginLeft: "auto" }}>
          <span className="app-dot" />
          {health === "checking" ? "…" : "live"}
        </div>
      </header>

      {notice && <div className="banner ok">{notice}</div>}
      {error && (
        <div className="banner err" onClick={() => setError("")} title="dismiss">
          {error}
        </div>
      )}

      <div className="sb-shell">
        <nav className="sb-rail">
          {TABS.map((t) => (
            <button
              key={t.key}
              className={`sb-rail-tab${tab === t.key ? " active" : ""}`}
              onClick={() => setTab(t.key)}
            >
              {t.label}
            </button>
          ))}
        </nav>
        <main className="sb-main">
          {tab === "registry" && <RegistryTab onError={fail} onFlash={flash} />}
          {tab === "risk" && <RiskTab onError={fail} onFlash={flash} />}
          {tab === "import" && <ImportTab onError={fail} />}
        </main>
      </div>
    </div>
  );
}

/* ————————————————— Registry ————————————————— */

function RegistryTab({ onError, onFlash }: { onError: (m: string) => void; onFlash: (m: string) => void }) {
  const router = useRouter();
  const [rows, setRows] = useState<Design[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    try {
      const j = await sbApi("/v1/registry/designs");
      setRows(listOf<Design>(j));
    } catch (e) {
      onError(String(e instanceof Error ? e.message : e));
    } finally {
      setLoading(false);
    }
  }, [onError]);

  useEffect(() => {
    load();
  }, [load]);

  const submit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const f = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    const body: Record<string, unknown> = {
      name: f.name,
      alphabet: f.alphabet,
      topology: f.topology,
    };
    if (f.description) body.description = f.description;
    if (f.sequence_text) body.sequence_text = f.sequence_text;
    if (f.notes) body.notes = f.notes;
    setBusy(true);
    try {
      const j = await sbApi<{ data: Design }>("/v1/registry/designs", {
        method: "POST",
        body: JSON.stringify(body),
      });
      setShowCreate(false);
      onFlash(`Design ${j.data.accession} created`);
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
        <h2>design registry</h2>
        <button className="btn primary" onClick={() => setShowCreate((v) => !v)}>
          {showCreate ? "Close" : "New design"}
        </button>
      </div>

      {showCreate && (
        <form className="create-form sb-form-wide" onSubmit={submit}>
          <label>
            <span>Name *</span>
            <input name="name" placeholder="e.g. pTet-GFP backbone" required />
          </label>
          <label>
            <span>Description</span>
            <input name="description" placeholder="optional" />
          </label>
          <label>
            <span>Alphabet</span>
            <select name="alphabet" defaultValue="dna">
              <option value="dna">dna</option>
              <option value="rna">rna</option>
              <option value="protein">protein</option>
            </select>
          </label>
          <label>
            <span>Topology</span>
            <select name="topology" defaultValue="circular">
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
            <input name="notes" placeholder="optional" />
          </label>
          <button className="btn primary" disabled={busy} type="submit">
            {busy ? "Creating…" : "Create design"}
          </button>
        </form>
      )}

      <table className="etable">
        <thead>
          <tr>
            <th>accession</th>
            <th>name</th>
            <th className="num">v</th>
            <th>status</th>
            <th>updated</th>
          </tr>
        </thead>
        <tbody>
          {!loading && rows.length === 0 && (
            <tr>
              <td colSpan={5} className="empty">
                No designs yet — create the first one.
              </td>
            </tr>
          )}
          {rows.map((d) => (
            <tr
              key={d.id}
              className="sb-rowlink"
              onClick={() => router.push(`/products/helix-synthbio/designs/${d.id}`)}
            >
              <td className="sb-mono">{d.accession}</td>
              <td>{d.name}</td>
              <td className="num">{d.current_version}</td>
              <td className={`status s-${d.status}`}>{d.status}</td>
              <td className="muted">{fmtTime(d.updated_at)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

/* ————————————————— Risk Review ————————————————— */

function RiskTab({ onError, onFlash }: { onError: (m: string) => void; onFlash: (m: string) => void }) {
  const router = useRouter();
  const [items, setItems] = useState<QueueItem[]>([]);
  const [names, setNames] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [open, setOpen] = useState<string | null>(null);
  const [busy, setBusy] = useState("");

  const load = useCallback(async () => {
    try {
      const [qj, dj] = await Promise.all([
        sbApi("/v1/registry/risk/queue"),
        sbApi("/v1/registry/designs"),
      ]);
      setItems(listOf<QueueItem>(qj));
      const map: Record<string, string> = {};
      for (const d of listOf<Design>(dj)) map[d.id] = d.name;
      setNames(map);
    } catch (e) {
      onError(String(e instanceof Error ? e.message : e));
    } finally {
      setLoading(false);
    }
  }, [onError]);

  useEffect(() => {
    load();
  }, [load]);

  const submitReview = async (item: QueueItem, e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const f = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    const reasons = f.reasons
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
    const body: Record<string, unknown> = {
      state: f.state,
      reviewer: f.reviewer,
    };
    if (f.intended_use) body.intended_use = f.intended_use;
    if (f.policy_version) body.policy_version = f.policy_version;
    if (reasons.length > 0) body.reasons = reasons;
    if (f.conditions) body.conditions = f.conditions;
    body.expires_at = f.expires_at ? new Date(`${f.expires_at}T00:00:00Z`).toISOString() : null;

    const key = item.case.id;
    setBusy(key);
    try {
      await sbApi(`/v1/registry/designs/${item.case.design_id}/risk/review`, {
        method: "POST",
        body: JSON.stringify(body),
      });
      onFlash(`Review recorded for ${item.accession} → ${f.state}`);
      setOpen(null);
      await load();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy("");
    }
  };

  return (
    <section className="panel sb-panel-flush">
      <div className="panel-head">
        <h2>risk review queue</h2>
        <span className="muted">{items.length} awaiting decision</span>
      </div>

      <table className="etable">
        <thead>
          <tr>
            <th>accession</th>
            <th>design</th>
            <th>state</th>
            <th>queued</th>
            <th className="acts">decision</th>
          </tr>
        </thead>
        <tbody>
          {!loading && items.length === 0 && (
            <tr>
              <td colSpan={5} className="empty">
                Queue is clear — no undecided risk cases.
              </td>
            </tr>
          )}
          {items.map((it) => {
            const c = it.case;
            const isOpen = open === c.id;
            return (
              <ReviewRow
                key={c.id}
                item={it}
                name={names[c.design_id]}
                isOpen={isOpen}
                busy={busy === c.id}
                onToggle={() => setOpen(isOpen ? null : c.id)}
                onOpenDesign={() => router.push(`/products/helix-synthbio/designs/${c.design_id}`)}
                onSubmit={(e) => submitReview(it, e)}
              />
            );
          })}
        </tbody>
      </table>
    </section>
  );
}

function ReviewRow(props: {
  item: QueueItem;
  name?: string;
  isOpen: boolean;
  busy: boolean;
  onToggle: () => void;
  onOpenDesign: () => void;
  onSubmit: (e: React.FormEvent<HTMLFormElement>) => void;
}) {
  const { item, name, isOpen, busy, onToggle, onOpenDesign, onSubmit } = props;
  const c = item.case;
  return (
    <>
      <tr className={isOpen ? "open" : ""}>
        <td className="sb-mono">
          <a
            href={`/products/helix-synthbio/designs/${c.design_id}`}
            onClick={(e) => {
              e.preventDefault();
              onOpenDesign();
            }}
          >
            {item.accession}
          </a>
        </td>
        <td>{name ?? "—"}</td>
        <td>
          <span className={riskClass(c.state)}>{c.state}</span>
        </td>
        <td className="muted">{fmtTime(c.created_at)}</td>
        <td className="acts">
          <button className="btn sm" onClick={onToggle}>
            {isOpen ? "Close" : "Decide"}
          </button>
        </td>
      </tr>
      {isOpen && (
        <tr className="child-row">
          <td colSpan={5}>
            <form className="create-form sb-form-wide" onSubmit={onSubmit}>
              <label>
                <span>State *</span>
                <select name="state" defaultValue="allowed">
                  <option value="allowed">allowed</option>
                  <option value="restricted">restricted</option>
                  <option value="blocked">blocked</option>
                </select>
              </label>
              <label>
                <span>Reviewer *</span>
                <input name="reviewer" defaultValue="ops@helixforge.local" required />
              </label>
              <label>
                <span>Intended use</span>
                <input name="intended_use" placeholder="e.g. bench research" />
              </label>
              <label>
                <span>Policy version</span>
                <input name="policy_version" placeholder="e.g. biosafety-v1" />
              </label>
              <label className="sb-span-all">
                <span>Reasons (comma-separated)</span>
                <input name="reasons" placeholder="e.g. public backbone, no SOC hits" />
              </label>
              <label className="sb-span-all">
                <span>Conditions</span>
                <input name="conditions" placeholder="optional constraints on use" />
              </label>
              <label>
                <span>Expires at</span>
                <input name="expires_at" type="date" />
              </label>
              <button className="btn primary" disabled={busy} type="submit">
                {busy ? "Recording…" : "Record decision"}
              </button>
            </form>
          </td>
        </tr>
      )}
    </>
  );
}

/* ————————————————— Import ————————————————— */

function ImportTab({ onError }: { onError: (m: string) => void }) {
  const [manifest, setManifest] = useState<ImportManifest | null>(null);
  const [content, setContent] = useState("");
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const f = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    if (!f.content.trim()) {
      onError("Paste file content first (or load the demo GenBank).");
      return;
    }
    const body: Record<string, unknown> = { format: f.format, content: f.content };
    if (f.filename) body.filename = f.filename;
    setBusy(true);
    try {
      const j = await sbApi<{ data: ImportManifest }>("/v1/registry/import", {
        method: "POST",
        body: JSON.stringify(body),
      });
      setManifest(j.data);
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel sb-panel-flush">
      <div className="panel-head">
        <h2>sequence import</h2>
        <button
          className="btn"
          type="button"
          onClick={() => {
            setContent(DEMO_GENBANK);
            setManifest(null);
          }}
        >
          Load demo GenBank
        </button>
      </div>

      <form className="create-form sb-form-wide" onSubmit={submit}>
        <label>
          <span>Format</span>
          <select name="format" defaultValue="auto">
            <option value="auto">auto</option>
            <option value="genbank">genbank</option>
            <option value="fasta">fasta</option>
          </select>
        </label>
        <label>
          <span>Filename</span>
          <input name="filename" placeholder="e.g. backbones.gb" />
        </label>
        <label className="sb-span-all">
          <span>File content *</span>
          <textarea
            name="content"
            rows={12}
            className="sb-mono"
            placeholder="Paste GenBank or FASTA content…"
            value={content}
            onChange={(e) => setContent(e.target.value)}
          />
        </label>
        <button className="btn primary" disabled={busy} type="submit">
          {busy ? "Importing…" : "Import"}
        </button>
      </form>

      {manifest && (
        <div className="sb-manifest">
          <div className="row">
            <span className="pill">
              total: <b>{manifest.total_records}</b>
            </span>
            <span className="pill sb-ok-text">
              accepted: <b>{manifest.accepted_count}</b>
            </span>
            <span className="pill sb-bad-text">
              rejected: <b>{manifest.rejected_count}</b>
            </span>
            <span className="pill muted">
              {manifest.accepted_count} + {manifest.rejected_count} = {manifest.total_records}
            </span>
          </div>

          {manifest.accepted.length > 0 && (
            <>
              <h3 className="sb-subhead">accepted</h3>
              <table className="etable">
                <thead>
                  <tr>
                    <th>accession</th>
                    <th>name</th>
                    <th className="num">v</th>
                    <th>status</th>
                  </tr>
                </thead>
                <tbody>
                  {manifest.accepted.map((d) => (
                    <tr key={d.id}>
                      <td className="sb-mono">
                        <a href={`/products/helix-synthbio/designs/${d.id}`}>{d.accession}</a>
                      </td>
                      <td>{d.name}</td>
                      <td className="num">{d.current_version}</td>
                      <td className={`status s-${d.status}`}>{d.status}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}

          {manifest.rejected.length > 0 && (
            <>
              <h3 className="sb-subhead">rejected</h3>
              <table className="etable">
                <thead>
                  <tr>
                    <th>record</th>
                    <th className="num">line</th>
                    <th>reason</th>
                  </tr>
                </thead>
                <tbody>
                  {manifest.rejected.map((r, i) => (
                    <tr key={i}>
                      <td className="sb-mono">{r.record || "—"}</td>
                      <td className="num">{r.line}</td>
                      <td className="sb-bad-text">{r.reason}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
        </div>
      )}
    </section>
  );
}
