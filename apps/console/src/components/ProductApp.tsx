"use client";

import { useCallback, useEffect, useState } from "react";
import type { ActionDef, ProductDef } from "@/lib/products";

type Row = Record<string, any>;

function cell(v: any): string {
  if (v === null || v === undefined) return "—";
  if (typeof v === "string" && v.length > 42) return `${v.slice(0, 39)}…`;
  if (typeof v === "object") return JSON.stringify(v).slice(0, 42);
  return String(v);
}

function shortId(v: any): string {
  const s = String(v ?? "");
  return s.length > 8 ? s.slice(0, 8) : s;
}

function coerce(form: Record<string, string>): Record<string, any> {
  const out: Record<string, any> = {};
  for (const [k, v] of Object.entries(form)) {
    if (v === "") continue;
    out[k] = /^-?\d+$/.test(v) ? Number(v) : v;
  }
  return out;
}

export function ProductApp({ product }: { product: ProductDef }) {
  const parent = product.parent;
  const child = product.child;

  const [health, setHealth] = useState<"checking" | "up" | "down">("checking");
  const [rows, setRows] = useState<Row[]>([]);
  const [children, setChildren] = useState<Record<string, Row[]>>({});
  const [expanded, setExpanded] = useState<string | null>(null);
  const [summary, setSummary] = useState<any>(null);
  const [summaryErr, setSummaryErr] = useState(false);
  const [busy, setBusy] = useState("");
  const [notice, setNotice] = useState("");
  const [error, setError] = useState("");
  const [showCreate, setShowCreate] = useState(false);
  const [createChildFor, setCreateChildFor] = useState<string | null>(null);

  const api = useCallback(
    async (path: string, init?: RequestInit) => {
      const r = await fetch(`/api/p/${product.slug}${path}`, init);
      const text = await r.text();
      let json: any = null;
      try {
        json = text ? JSON.parse(text) : null;
      } catch {
        json = text;
      }
      if (!r.ok) {
        const msg =
          json?.error?.message ??
          (typeof json?.error === "string" ? json.error : undefined) ??
          json?.message ??
          `${r.status} ${r.statusText}`;
        throw new Error(String(msg));
      }
      return json;
    },
    [product.slug],
  );

  const loadRows = useCallback(async () => {
    if (!parent) return;
    const j = await api(parent.path);
    setRows(j?.data?.items ?? j?.data ?? []);
  }, [api, parent]);

  const loadChildren = useCallback(
    async (pid: string) => {
      if (!child) return;
      const j = await api(child.listTemplate.replace("{id}", pid));
      setChildren((c) => ({ ...c, [pid]: j?.data?.items ?? j?.data ?? [] }));
    },
    [api, child],
  );

  const loadSummary = useCallback(async () => {
    if (!product.summaryPath) return;
    try {
      const j = await api(product.summaryPath);
      setSummary(j?.data ?? j);
      setSummaryErr(false);
    } catch {
      setSummaryErr(true);
    }
  }, [api, product.summaryPath]);

  const refresh = useCallback(async () => {
    await loadRows();
    if (expanded) await loadChildren(expanded);
    await loadSummary();
  }, [loadRows, loadChildren, loadSummary, expanded]);

  useEffect(() => {
    let alive = true;
    (async () => {
      try {
        await api("/healthz");
        if (!alive) return;
        setHealth("up");
        await refresh();
      } catch {
        if (alive) setHealth("down");
      }
    })();
    return () => {
      alive = false;
    };
  }, [api, refresh]);

  const flash = (msg: string) => {
    setNotice(msg);
    setError("");
    setTimeout(() => setNotice(""), 3500);
  };
  const fail = (msg: string) => {
    setError(msg);
    setNotice("");
  };

  const submitCreate = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!parent) return;
    const form = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    setBusy("create");
    try {
      await api(parent.path, { method: "POST", body: JSON.stringify(coerce(form)) });
      e.currentTarget.reset();
      setShowCreate(false);
      flash(`${parent.singular} created`);
      await refresh();
    } catch (err) {
      fail(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy("");
    }
  };

  const submitChild = async (pid: string, e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!child?.createTemplate) return;
    const form = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    setBusy(`child-${pid}`);
    try {
      await api(child.createTemplate.replace("{id}", pid), {
        method: "POST",
        body: JSON.stringify(coerce(form)),
      });
      e.currentTarget.reset();
      setCreateChildFor(null);
      flash(`${child.singular} created`);
      await loadChildren(pid);
      await loadSummary();
    } catch (err) {
      fail(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy("");
    }
  };

  const act = async (a: ActionDef, pid: string | null, entityPath: string, id: string) => {
    const key = `${a.action}-${id}`;
    setBusy(key);
    try {
      if (a.method === "delete") {
        await api(`${entityPath}/${id}`, { method: "DELETE" });
      } else {
        await api(`${entityPath}/${id}/${a.action}`, { method: "POST", body: "{}" });
      }
      flash(`${a.label} ✓`);
      if (pid) await loadChildren(pid);
      await loadRows();
      await loadSummary();
    } catch (err) {
      fail(String(err instanceof Error ? err.message : err));
    } finally {
      setBusy("");
    }
  };

  const toggleExpand = async (id: string) => {
    const next = expanded === id ? null : id;
    setExpanded(next);
    if (next && !children[next]) await loadChildren(next);
  };

  if (health === "down") {
    return (
      <div className="app-down panel">
        <h1>{product.title}</h1>
        <p className="lead">
          The {product.title} API is not answering on :{product.port}. Start the
          product suite with <code>scripts/dev-products.ps1</code>, then reload.
        </p>
      </div>
    );
  }

  return (
    <div className="papp">
      <header className="papp-head">
        <div className="app-glyph lg">{product.glyph}</div>
        <div>
          <h1>{product.title}</h1>
          <p className="muted">{product.blurb} · localhost:{product.port}</p>
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

      {parent && (
        <section className="panel">
          <div className="panel-head">
            <h2>{parent.plural}</h2>
            <button className="btn primary" onClick={() => setShowCreate((v) => !v)}>
              {showCreate ? "Close" : `New ${parent.singular}`}
            </button>
          </div>

          {showCreate && parent.createFields && (
            <form className="create-form" onSubmit={submitCreate}>
              {parent.createFields.map((f) => (
                <label key={f.key}>
                  <span>{f.label}</span>
                  <input name={f.key} placeholder={f.placeholder} required={f.required} />
                </label>
              ))}
              <button className="btn primary" disabled={busy === "create"} type="submit">
                {busy === "create" ? "Creating…" : `Create ${parent.singular}`}
              </button>
            </form>
          )}

          <table className="etable">
            <thead>
              <tr>
                {parent.columns.map((c) => (
                  <th key={c.key}>{c.label}</th>
                ))}
                <th className="num">id</th>
                <th className="acts">actions</th>
              </tr>
            </thead>
            <tbody>
              {rows.length === 0 && (
                <tr>
                  <td colSpan={parent.columns.length + 2} className="empty">
                    No {parent.plural} yet — create the first one.
                  </td>
                </tr>
              )}
              {rows.map((r) => {
                const id = String(r.id);
                const isOpen = expanded === id;
                return (
                  <FragmentRow
                    key={id}
                    r={r}
                    id={id}
                    isOpen={isOpen}
                    columns={parent.columns}
                    actions={parent.actions ?? []}
                    busy={busy}
                    hasChild={Boolean(child)}
                    onToggle={() => toggleExpand(id)}
                    onAct={(a) => act(a, null, parent.path, id)}
                  >
                    {isOpen && child && (
                      <tr className="child-row">
                        <td colSpan={parent.columns.length + 2}>
                          <div className="child-panel">
                            <div className="panel-head">
                              <h3>{child.plural}</h3>
                              {child.createTemplate && (
                                <button
                                  className="btn"
                                  onClick={() => setCreateChildFor(createChildFor === id ? null : id)}
                                >
                                  {createChildFor === id ? "Close" : `New ${child.singular}`}
                                </button>
                              )}
                            </div>
                            {createChildFor === id && child.createFields && (
                              <form className="create-form" onSubmit={(e) => submitChild(id, e)}>
                                {child.createFields.map((f) => (
                                  <label key={f.key}>
                                    <span>{f.label}</span>
                                    <input name={f.key} placeholder={f.placeholder} required={f.required} />
                                  </label>
                                ))}
                                <button className="btn primary" disabled={busy === `child-${id}`} type="submit">
                                  {busy === `child-${id}` ? "Creating…" : `Create ${child.singular}`}
                                </button>
                              </form>
                            )}
                            <table className="etable child">
                              <thead>
                                <tr>
                                  {child.columns.map((c) => (
                                    <th key={c.key}>{c.label}</th>
                                  ))}
                                  <th className="num">id</th>
                                  <th className="acts">actions</th>
                                </tr>
                              </thead>
                              <tbody>
                                {(children[id] ?? []).length === 0 && (
                                  <tr>
                                    <td colSpan={child.columns.length + 2} className="empty">
                                      No {child.plural} yet.
                                    </td>
                                  </tr>
                                )}
                                {(children[id] ?? []).map((cr) => {
                                  const cid = String(cr.id);
                                  return (
                                    <tr key={cid}>
                                      {child.columns.map((c) => (
                                        <td key={c.key} className={c.key === "status" ? `status s-${cr[c.key]}` : ""}>
                                          {cell(cr[c.key])}
                                        </td>
                                      ))}
                                      <td className="num muted">{shortId(cid)}</td>
                                      <td className="acts">
                                        {(child.actions ?? []).map((a) => (
                                          <button
                                            key={a.action}
                                            className="btn sm"
                                            disabled={busy === `${a.action}-${cid}`}
                                            onClick={() =>
                                              act(a, id, child.listTemplate.replace("{id}", id), cid)
                                            }
                                          >
                                            {a.label}
                                          </button>
                                        ))}
                                      </td>
                                    </tr>
                                  );
                                })}
                              </tbody>
                            </table>
                          </div>
                        </td>
                      </tr>
                    )}
                  </FragmentRow>
                );
              })}
            </tbody>
          </table>
        </section>
      )}

      {product.summaryPath && !summaryErr && summary && (
        <section className="panel">
          <div className="panel-head">
            <h2>summary</h2>
          </div>
          <SummaryView data={summary} />
        </section>
      )}
    </div>
  );
}

function FragmentRow(props: {
  r: Row;
  id: string;
  isOpen: boolean;
  columns: { key: string; label: string }[];
  actions: ActionDef[];
  busy: string;
  hasChild: boolean;
  onToggle: () => void;
  onAct: (a: ActionDef) => void;
  children?: React.ReactNode;
}) {
  const { r, id, isOpen, columns, actions, busy, hasChild, onToggle, onAct } = props;
  return (
    <>
      <tr className={isOpen ? "open" : ""}>
        {columns.map((c) => (
          <td key={c.key} className={c.key === "status" ? `status s-${r[c.key]}` : ""}>
            {cell(r[c.key])}
          </td>
        ))}
        <td className="num muted">{shortId(id)}</td>
        <td className="acts">
          {actions.map((a) => (
            <button key={a.action} className="btn sm" disabled={busy === `${a.action}-${id}`} onClick={() => onAct(a)}>
              {a.label}
            </button>
          ))}
          {hasChild && (
            <button className="btn sm ghost" onClick={onToggle}>
              {isOpen ? "▾" : "▸"}
            </button>
          )}
        </td>
      </tr>
      {props.children}
    </>
  );
}

function SummaryView({ data }: { data: any }) {
  if (Array.isArray(data)) {
    const rows = data as Record<string, any>[];
    if (rows.length === 0) return <p className="muted">empty</p>;
    const keys: string[] = [];
    rows.forEach((row) => {
      Object.keys(row).forEach((k) => {
        if (!keys.includes(k) && (typeof row[k] !== "object" || row[k] === null)) keys.push(k);
      });
    });
    return (
      <table className="etable child">
        <thead>
          <tr>
            {keys.map((k) => (
              <th key={k}>{k}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.slice(0, 12).map((row, i) => (
            <tr key={i}>
              {keys.map((k) => (
                <td key={k}>{cell(row[k])}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    );
  }
  return (
    <div className="row">
      {Object.entries(data as Record<string, any>)
        .filter(([, v]) => typeof v !== "object" || v === null)
        .map(([k, v]) => (
          <span key={k} className="pill">
            {k}: <b>{String(v)}</b>
          </span>
        ))}
    </div>
  );
}
