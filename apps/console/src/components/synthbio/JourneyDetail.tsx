"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import {
  fmtTime,
  journeyStageClass,
  listOf,
  riskClass,
  sbApi,
  shortId,
  SAMPLE_KINDS,
  type Design,
  type Design360Data,
  type JourneyDetailData,
  type JourneyStageRow,
  type Pathway,
  type Sample,
} from "./lib";

/**
 * Journey detail: the guided 7-stage pipeline front door.
 * GET /v1/journeys/{id} auto-refreshes server-side on every read; every
 * action below ends with load(), so the pipeline always re-checks.
 */
export function JourneyDetail({ id }: { id: string }) {
  const router = useRouter();
  const [data, setData] = useState<JourneyDetailData | null>(null);
  const [pathway, setPathway] = useState<Pathway | null>(null);
  const [samples, setSamples] = useState<Sample[]>([]);
  const [designs, setDesigns] = useState<Design[]>([]);
  const [designRisk, setDesignRisk] = useState("");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [expanded, setExpanded] = useState<number | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const cardRefs = useRef<(HTMLElement | null)[]>([]);

  const load = useCallback(async () => {
    try {
      const j = await sbApi<{ data: JourneyDetailData }>(`/v1/journeys/${id}`);
      setData(j.data);
      const [pwj, sj, dj] = await Promise.all([
        sbApi("/v1/pathways"),
        sbApi("/v1/inventory/samples"),
        sbApi("/v1/registry/designs"),
      ]);
      setSamples(listOf<Sample>(sj));
      setDesigns(listOf<Design>(dj));
      setPathway(
        listOf<Pathway>(pwj).find((p) => p.key === j.data.journey.pathway_key) ?? null,
      );
      // effective risk of the linked design, for the risk stage card
      const designId = j.data.stages.find((s) => s.stage_key === "design")?.target_id;
      if (designId) {
        try {
          const d360 = await sbApi<{ data: Design360Data }>(`/v1/registry/designs/${designId}`);
          setDesignRisk(d360.data.effective_risk);
        } catch {
          setDesignRisk("");
        }
      } else {
        setDesignRisk("");
      }
      setExpanded((cur) => cur ?? Math.min(j.data.journey.current_stage, 6));
    } catch (e) {
      setError(String(e instanceof Error ? e.message : e));
    }
  }, [id]);

  useEffect(() => {
    load();
  }, [load]);

  // reset per-journey UI state when navigating between journeys
  useEffect(() => {
    setError("");
    setNotice("");
    setExpanded(null);
    setDesignRisk("");
  }, [id]);

  const flash = (msg: string) => {
    setNotice(msg);
    setTimeout(() => setNotice(""), 5000);
  };

  const refresh = async () => {
    setRefreshing(true);
    try {
      await sbApi(`/v1/journeys/${id}/refresh`, { method: "POST" });
      flash("Journey re-checked");
      await load();
    } catch (e) {
      setError(String(e instanceof Error ? e.message : e));
    } finally {
      setRefreshing(false);
    }
  };

  if (error && !data) {
    return (
      <div className="app-down panel">
        <h1>Journey not available</h1>
        <p className="lead">{error}</p>
        <button className="btn" onClick={() => router.push("/products/helix-synthbio")}>
          ← Back to journeys
        </button>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="sb-app">
        <div className="sb-skel" style={{ height: "2rem", width: "55%", marginBottom: "1rem" }} />
        <div className="sb-skel" style={{ height: "3.2rem", marginBottom: "1.2rem" }} />
        {[0, 1, 2].map((i) => (
          <div key={i} className="sb-skel" style={{ height: "5.5rem", marginBottom: "1rem" }} />
        ))}
      </div>
    );
  }

  const { journey } = data;
  const stages = [...data.stages].sort((a, b) => a.stage_index - b.stage_index);
  const stageAt = (key: string) => stages.find((s) => s.stage_key === key);
  const template = (key: string) => pathway?.stages.find((s) => s.stage_key === key);
  const designTarget = stageAt("design")?.target_id ?? null;
  const buildTarget = stageAt("build")?.target_id ?? null;
  const completed = journey.status === "completed";

  const accOf = (kind: string | null, tid: string | null): string => {
    if (!tid) return "";
    if (kind === "sample") return samples.find((s) => s.id === tid)?.accession ?? shortId(tid);
    if (kind === "design") return designs.find((d) => d.id === tid)?.accession ?? shortId(tid);
    return shortId(tid);
  };
  const linkOf = (kind: string | null, tid: string | null): string =>
    kind === "design"
      ? `/products/helix-synthbio/designs/${tid}`
      : `/products/helix-synthbio/samples/${tid}`;

  const gotoStage = (i: number) => {
    setExpanded(i);
    cardRefs.current[i]?.scrollIntoView({ behavior: "smooth", block: "start" });
  };

  return (
    <div className="sb-app">
      <header className="sb-360-head">
        <button className="btn sm ghost" onClick={() => router.push("/products/helix-synthbio")}>
          ← journeys
        </button>
        <div className="sb-360-title">
          <span className="sb-accession">{journey.accession}</span>
          <span className="sb-360-name">{journey.title}</span>
        </div>
        <span className={`pill status s-${journey.status}`}>{journey.status}</span>
        <span className="pill">{pathway?.title ?? journey.pathway_key}</span>
        {journey.route_choice && journey.route_choice !== "undecided" && (
          <span className="pill">route: {journey.route_choice}</span>
        )}
        <button
          className="btn"
          style={{ marginLeft: "auto" }}
          disabled={refreshing}
          onClick={refresh}
        >
          {refreshing ? "Refreshing…" : "Refresh"}
        </button>
      </header>

      {journey.intent && <p className="lead sb-desc">{journey.intent}</p>}

      {notice && <div className="banner ok">{notice}</div>}
      {error && (
        <div className="banner err" onClick={() => setError("")} title="dismiss">
          {error}
        </div>
      )}

      {completed && (
        <div className="sb-complete">
          <b>Journey complete</b> — the full chain is linked and attested:
          <div className="sb-complete-chain">
            <span className="sb-kind">source</span>
            <a className="sb-mono" href={linkOf("sample", stageAt("source")?.target_id ?? "")}>
              {accOf("sample", stageAt("source")?.target_id ?? null)}
            </a>
            <span className="muted">→</span>
            <span className="sb-kind">design</span>
            <a className="sb-mono" href={linkOf("design", designTarget)}>
              {accOf("design", designTarget)}
            </a>
            <span className="muted">→</span>
            <span className="sb-kind">build</span>
            <a className="sb-mono" href={linkOf("sample", buildTarget)}>
              {accOf("sample", buildTarget)}
            </a>
            <span className="muted">→</span>
            <span className="sb-kind">evidence</span>
            <a className="sb-mono" href={linkOf("design", designTarget)}>
              attested claim
            </a>
          </div>
        </div>
      )}

      {/* 7-stage pipeline */}
      <div className="sb-pipe">
        {stages.map((st, i) => (
          <div className="sb-pipe-cell" key={st.id}>
            {i > 0 && <div className={`sb-pipe-link${stages[i - 1].status === "done" ? " fill" : ""}`} />}
            <button
              type="button"
              className={`sb-pipe-node ${st.status === "done" ? "done" : ""}${st.status === "current" ? " cur" : ""}`}
              onClick={() => gotoStage(i)}
              title={`${template(st.stage_key)?.title ?? st.summary} — ${st.status}`}
            >
              <span className="sb-pipe-dot">{st.status === "done" ? "✓" : i + 1}</span>
              <span className="sb-pipe-label">{template(st.stage_key)?.title ?? st.stage_key}</span>
            </button>
          </div>
        ))}
      </div>

      {/* stage cards */}
      {stages.map((st, i) => (
        <section
          key={st.id}
          ref={(el) => {
            cardRefs.current[i] = el;
          }}
          className={`panel sb-stage-card${st.status === "current" ? " cur" : ""}`}
        >
          <div className="panel-head">
            <h2>
              {i + 1}. {template(st.stage_key)?.title ?? st.summary}
            </h2>
            <div className="sb-m-head-tools">
              {st.target_id && (
                <a className="sb-mono sb-target-link" href={linkOf(st.target_kind, st.target_id)}>
                  {accOf(st.target_kind, st.target_id)}
                </a>
              )}
              <span className={journeyStageClass(st.status)}>{st.status}</span>
              <button
                className="btn sm ghost"
                onClick={() => setExpanded(expanded === i ? null : i)}
              >
                {expanded === i ? "Collapse" : "Open"}
              </button>
            </div>
          </div>

          {expanded === i && (
            <>
              {template(st.stage_key) && (
                <p className="muted sb-stage-expl">{template(st.stage_key)!.explanation}</p>
              )}
              {!st.check.met && st.check.missing && (
                <div className="sb-guide">→ {st.check.missing}</div>
              )}
              <StageActions
                st={st}
                jid={id}
                routeChoice={journey.route_choice}
                designTarget={designTarget}
                designRisk={designRisk}
                buildTarget={buildTarget}
                samples={samples}
                designs={designs}
                onDone={load}
                onError={setError}
                onFlash={flash}
              />
            </>
          )}
        </section>
      ))}
    </div>
  );
}

/* ————————————————— per-stage actions ————————————————— */

function StageActions(props: {
  st: JourneyStageRow;
  jid: string;
  routeChoice: string;
  designTarget: string | null;
  designRisk: string;
  buildTarget: string | null;
  samples: Sample[];
  designs: Design[];
  onDone: () => Promise<void>;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const { st, jid, routeChoice, designTarget, designRisk, buildTarget, samples, designs, onDone, onError, onFlash } = props;

  switch (st.stage_key) {
    case "source":
      if (st.status === "done") return null;
      return (
        <LinkSampleForm
          jid={jid}
          index={st.stage_index}
          samples={samples}
          onDone={onDone}
          onError={onError}
          onFlash={onFlash}
        />
      );
    case "route":
      return <RouteChoice jid={jid} chosen={routeChoice} done={st.status === "done"} onDone={onDone} onError={onError} onFlash={onFlash} />;
    case "design":
      if (st.status === "done") return null;
      return (
        <LinkDesignForm
          jid={jid}
          index={st.stage_index}
          designs={designs}
          onDone={onDone}
          onError={onError}
          onFlash={onFlash}
        />
      );
    case "risk":
      if (!designTarget) return null; // dependency guidance already shown
      return (
        <div className="sb-auto-note">
          <p className="muted">
            Current effective risk of the linked design:{" "}
            <span className={riskClass(designRisk || "unknown")}>{designRisk || "…"}</span>
          </p>
          <p className="muted">
            This stage checks itself — record the risk decision in the{" "}
            <a href={`/products/helix-synthbio/designs/${designTarget}`}>design 360 review UI</a>,
            then press <b>Refresh</b> and the journey re-evaluates.
          </p>
        </div>
      );
    case "build":
      if (st.status === "done") return null;
      if (!designTarget) return null; // "link the design first" guidance shown
      return (
        <LinkSampleForm
          jid={jid}
          index={st.stage_index}
          samples={samples}
          designId={designTarget}
          onDone={onDone}
          onError={onError}
          onFlash={onFlash}
        />
      );
    case "test":
      return (
        <div className="sb-auto-note">
          <p className="muted">
            This stage checks itself — it needs an <b>accepted</b> measurement on the build
            sample.
            {buildTarget ? (
              <>
                {" "}
                Record and accept one on the{" "}
                <a href={`/products/helix-synthbio/samples/${buildTarget}`}>build sample page</a>,
                then press <b>Refresh</b>.
              </>
            ) : (
              " Link the build sample first."
            )}
          </p>
        </div>
      );
    case "evidence":
      if (!designTarget) return null; // "link the design first" guidance shown
      return (
        <div className="sb-auto-note">
          <p className="muted">
            Create a claim on the linked design and attest it here — or use the{" "}
            <a href={`/products/helix-synthbio/designs/${designTarget}`}>design 360 Claims tab</a>{" "}
            for the full evidence flow. Then press <b>Refresh</b>.
          </p>
          <ClaimQuickForm designId={designTarget} onDone={onDone} onError={onError} onFlash={onFlash} />
        </div>
      );
    default:
      return null;
  }
}

/** Register a new sample and link it, or link an existing one (build filters to the journey's design). */
function LinkSampleForm({
  jid,
  index,
  samples,
  designId,
  onDone,
  onError,
  onFlash,
}: {
  jid: string;
  index: number;
  samples: Sample[];
  designId?: string | null;
  onDone: () => Promise<void>;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const [mode, setMode] = useState<"create" | "pick">("create");
  const [busy, setBusy] = useState(false);
  const [localErr, setLocalErr] = useState("");
  const candidates = designId ? samples.filter((s) => s.design_id === designId) : samples;

  const link = async (sampleId: string) => {
    await sbApi(`/v1/journeys/${jid}/stages/${index}/link`, {
      method: "POST",
      body: JSON.stringify({ target_kind: "sample", target_id: sampleId }),
    });
  };

  const createAndLink = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const f = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    setBusy(true);
    setLocalErr("");
    try {
      const body: Record<string, unknown> = { name: f.name, kind: f.kind };
      if (f.location) body.location = f.location;
      if (designId) body.design_id = designId;
      const j = await sbApi<{ data: Sample }>("/v1/inventory/samples", {
        method: "POST",
        body: JSON.stringify(body),
      });
      await link(j.data.id);
      onFlash(`Sample ${j.data.accession} registered and linked`);
      await onDone();
    } catch (err) {
      // 422 (e.g. "not built from the journey's design") surfaces inline on the card
      setLocalErr(String(err instanceof Error ? err.message : err));
      setBusy(false);
    }
  };

  const pickAndLink = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const f = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    if (!f.sample_id) {
      setLocalErr("Pick a sample to link.");
      return;
    }
    setBusy(true);
    setLocalErr("");
    try {
      await link(f.sample_id);
      onFlash("Sample linked");
      await onDone();
    } catch (err) {
      setLocalErr(String(err instanceof Error ? err.message : err));
      setBusy(false);
    }
  };

  return (
    <div>
      <div className="sb-seg sb-stage-seg">
        <button
          type="button"
          className={`sb-seg-btn${mode === "create" ? " active" : ""}`}
          onClick={() => setMode("create")}
        >
          Register new
        </button>
        <button
          type="button"
          className={`sb-seg-btn${mode === "pick" ? " active" : ""}`}
          onClick={() => setMode("pick")}
        >
          Link existing
        </button>
      </div>

      {localErr && (
        <div className="banner err" onClick={() => setLocalErr("")} title="dismiss">
          {localErr}
        </div>
      )}

      {mode === "create" ? (
        <form className="create-form sb-form-wide" onSubmit={createAndLink}>
          <label>
            <span>Name *</span>
            <input name="name" placeholder="e.g. lavender batch #1" required />
          </label>
          <label>
            <span>Kind</span>
            <select name="kind" defaultValue={designId ? "plasmid_prep" : "other"}>
              {SAMPLE_KINDS.map((k) => (
                <option key={k} value={k}>
                  {k}
                </option>
              ))}
            </select>
          </label>
          <label>
            <span>Location</span>
            <input name="location" placeholder="e.g. bench-2" />
          </label>
          {designId && (
            <p className="muted sb-span-all sb-build-hint">
              will be registered as derived from the journey&apos;s linked design
            </p>
          )}
          <button className="btn primary" disabled={busy} type="submit">
            {busy ? "Linking…" : "Register & link"}
          </button>
        </form>
      ) : (
        <form className="create-form sb-form-wide" onSubmit={pickAndLink}>
          <label className="sb-span-all">
            <span>Sample{designId ? " (derived from the journey's design)" : ""}</span>
            <select name="sample_id" defaultValue="">
              <option value="">— pick a sample —</option>
              {candidates.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.accession} — {s.name}
                </option>
              ))}
            </select>
          </label>
          <button className="btn primary" disabled={busy} type="submit">
            {busy ? "Linking…" : "Link sample"}
          </button>
        </form>
      )}
    </div>
  );
}

/** Route choice cards (stage 1) — read-only once chosen. */
function RouteChoice({
  jid,
  chosen,
  done,
  onDone,
  onError,
  onFlash,
}: {
  jid: string;
  chosen: string;
  done: boolean;
  onDone: () => Promise<void>;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const [busy, setBusy] = useState("");

  const pick = async (route: "extraction" | "engineered_microbe") => {
    setBusy(route);
    try {
      await sbApi(`/v1/journeys/${jid}/route`, {
        method: "POST",
        body: JSON.stringify({ route }),
      });
      onFlash(`Route set: ${route}`);
      await onDone();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
      setBusy("");
    }
  };

  if (done || (chosen && chosen !== "undecided")) {
    return (
      <p>
        route chosen:{" "}
        <span className="pill">
          {chosen === "extraction" ? "Extract from the source" : "Engineer a microbe"}
        </span>
      </p>
    );
  }

  return (
    <div className="sb-route-cards">
      <button
        type="button"
        className="sb-route-card"
        disabled={busy !== ""}
        onClick={() => pick("extraction")}
      >
        <h3>{busy === "extraction" ? "Choosing…" : "Extract from the source"}</h3>
        <p>Isolate the active compound directly from the starting material — no engineering.</p>
      </button>
      <button
        type="button"
        className="sb-route-card"
        disabled={busy !== ""}
        onClick={() => pick("engineered_microbe")}
      >
        <h3>{busy === "engineered_microbe" ? "Choosing…" : "Engineer a microbe"}</h3>
        <p>Build a strain or cell line that produces the compound — design, review, then prep.</p>
      </button>
    </div>
  );
}

/** Create a design inline and link it, or pick an existing design (stage 2). */
function LinkDesignForm({
  jid,
  index,
  designs,
  onDone,
  onError,
  onFlash,
}: {
  jid: string;
  index: number;
  designs: Design[];
  onDone: () => Promise<void>;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const [mode, setMode] = useState<"create" | "pick">("create");
  const [busy, setBusy] = useState(false);
  const [localErr, setLocalErr] = useState("");

  const link = async (designId: string) => {
    await sbApi(`/v1/journeys/${jid}/stages/${index}/link`, {
      method: "POST",
      body: JSON.stringify({ target_kind: "design", target_id: designId }),
    });
  };

  const createAndLink = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const f = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    setBusy(true);
    setLocalErr("");
    try {
      const body: Record<string, unknown> = {
        name: f.name,
        alphabet: f.alphabet,
        topology: f.topology,
      };
      if (f.sequence_text) body.sequence_text = f.sequence_text;
      if (f.description) body.description = f.description;
      const j = await sbApi<{ data: Design }>("/v1/registry/designs", {
        method: "POST",
        body: JSON.stringify(body),
      });
      await link(j.data.id);
      onFlash(`Design ${j.data.accession} created and linked`);
      await onDone();
    } catch (err) {
      setLocalErr(String(err instanceof Error ? err.message : err));
      setBusy(false);
    }
  };

  const pickAndLink = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const f = Object.fromEntries(new FormData(e.currentTarget).entries()) as Record<string, string>;
    if (!f.design_id) {
      setLocalErr("Pick a design to link.");
      return;
    }
    setBusy(true);
    setLocalErr("");
    try {
      await link(f.design_id);
      onFlash("Design linked");
      await onDone();
    } catch (err) {
      setLocalErr(String(err instanceof Error ? err.message : err));
      setBusy(false);
    }
  };

  return (
    <div>
      <div className="sb-seg sb-stage-seg">
        <button
          type="button"
          className={`sb-seg-btn${mode === "create" ? " active" : ""}`}
          onClick={() => setMode("create")}
        >
          Create new
        </button>
        <button
          type="button"
          className={`sb-seg-btn${mode === "pick" ? " active" : ""}`}
          onClick={() => setMode("pick")}
        >
          Pick existing
        </button>
      </div>

      {localErr && (
        <div className="banner err" onClick={() => setLocalErr("")} title="dismiss">
          {localErr}
        </div>
      )}

      {mode === "create" ? (
        <form className="create-form sb-form-wide" onSubmit={createAndLink}>
          <label>
            <span>Name *</span>
            <input name="name" placeholder="e.g. linalool expression cassette" required />
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
          <label>
            <span>Description</span>
            <input name="description" placeholder="optional" />
          </label>
          <label className="sb-span-all">
            <span>Sequence</span>
            <textarea name="sequence_text" rows={3} className="sb-mono" placeholder="ACGT… (optional)" />
          </label>
          <button className="btn primary" disabled={busy} type="submit">
            {busy ? "Linking…" : "Create & link"}
          </button>
        </form>
      ) : (
        <form className="create-form sb-form-wide" onSubmit={pickAndLink}>
          <label className="sb-span-all">
            <span>Design</span>
            <select name="design_id" defaultValue="">
              <option value="">— pick a design —</option>
              {designs.map((d) => (
                <option key={d.id} value={d.id}>
                  {d.accession} — {d.name}
                </option>
              ))}
            </select>
          </label>
          <button className="btn primary" disabled={busy} type="submit">
            {busy ? "Linking…" : "Link design"}
          </button>
        </form>
      )}
    </div>
  );
}

/** Inline create-claim + attest on the journey's linked design (stage 6). */
function ClaimQuickForm({
  designId,
  onDone,
  onError,
  onFlash,
}: {
  designId: string;
  onDone: () => Promise<void>;
  onError: (m: string) => void;
  onFlash: (m: string) => void;
}) {
  const [statement, setStatement] = useState("");
  const [attestor, setAttestor] = useState("ops@helixforge.local");
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!statement.trim()) {
      onError("Statement is required.");
      return;
    }
    setBusy(true);
    try {
      const j = await sbApi<{ data: { id: string; accession: string } }>("/v1/claims", {
        method: "POST",
        body: JSON.stringify({ design_id: designId, statement: statement.trim() }),
      });
      await sbApi(`/v1/claims/${j.data.id}/attest`, {
        method: "POST",
        body: JSON.stringify({ attestor: attestor.trim() || "ops@helixforge.local" }),
      });
      onFlash(`Claim ${j.data.accession} created and attested`);
      await onDone();
    } catch (err) {
      onError(String(err instanceof Error ? err.message : err));
      setBusy(false);
    }
  };

  return (
    <form className="create-form sb-form-wide" onSubmit={submit}>
      <label className="sb-span-all">
        <span>Claim statement *</span>
        <textarea
          rows={2}
          value={statement}
          placeholder="e.g. this extract soothes dry skin at 2% concentration"
          onChange={(e) => setStatement(e.target.value)}
          required
        />
      </label>
      <label>
        <span>Attestor</span>
        <input value={attestor} onChange={(e) => setAttestor(e.target.value)} />
      </label>
      <button className="btn primary" disabled={busy} type="submit">
        {busy ? "Attesting…" : "Create claim & attest"}
      </button>
    </form>
  );
}
