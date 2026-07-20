"use client";

import { useEffect, useState } from "react";
import { MaturityBadge, SemanticBadge } from "@helixforge/ui";
import { GatewayClient, type CatalogEntry, type SemanticState } from "@helixforge/sdk-ts";

const FALLBACK: CatalogEntry[] = [
  { order: 1, slug: "helix-collab", title: "HelixCollab", description: "Real-time collaborative workspace", tier: "standard", maturity: "beta", semantic_state: "active", default_port: 8101, upstream: "http://127.0.0.1:8101" },
  { order: 2, slug: "helix-code", title: "HelixCode", description: "AI-native collaborative IDE", tier: "standard", maturity: "beta", semantic_state: "active", default_port: 8102, upstream: "http://127.0.0.1:8102" },
  { order: 3, slug: "helix-flow", title: "HelixFlow", description: "Agentic automation & workflow engine", tier: "standard", maturity: "alpha", semantic_state: "active", default_port: 8103, upstream: "http://127.0.0.1:8103" },
  { order: 4, slug: "helix-insights", title: "HelixInsights", description: "Predictive analytics & decision OS", tier: "standard", maturity: "scaffold", semantic_state: "unknown", default_port: 8104, upstream: "http://127.0.0.1:8104" },
  { order: 5, slug: "helix-commerce", title: "HelixCommerce", description: "AI e-commerce & digital marketplace builder", tier: "standard", maturity: "scaffold", semantic_state: "unknown", default_port: 8105, upstream: "http://127.0.0.1:8105" },
  { order: 6, slug: "helix-edu", title: "HelixEdu", description: "Adaptive AI learning & certification platform", tier: "standard", maturity: "scaffold", semantic_state: "unknown", default_port: 8106, upstream: "http://127.0.0.1:8106" },
  { order: 7, slug: "helix-capital", title: "HelixCapital", description: "AI financial operating system", tier: "standard", maturity: "scaffold", semantic_state: "unknown", default_port: 8107, upstream: "http://127.0.0.1:8107" },
  { order: 8, slug: "helix-well", title: "HelixWell", description: "AI personal & team wellness platform", tier: "standard", maturity: "scaffold", semantic_state: "unknown", default_port: 8108, upstream: "http://127.0.0.1:8108" },
  { order: 9, slug: "helix-network", title: "HelixNetwork", description: "AI professional networking & opportunity engine", tier: "standard", maturity: "scaffold", semantic_state: "unknown", default_port: 8109, upstream: "http://127.0.0.1:8109" },
  { order: 10, slug: "helix-forge-studio", title: "HelixForge Studio", description: "No-code/low-code AI app & internal tool builder", tier: "standard", maturity: "scaffold", semantic_state: "unknown", default_port: 8110, upstream: "http://127.0.0.1:8110" },
  { order: 11, slug: "helix-synthbio", title: "HelixSynthBio", description: "Synthetic biology design & virtual wet-lab", tier: "frontier", maturity: "scaffold", semantic_state: "unknown", default_port: 8111, upstream: "http://127.0.0.1:8111" },
  { order: 12, slug: "helix-lex-prime", title: "HelixLexPrime", description: "Autonomous legal & regulatory intelligence", tier: "frontier", maturity: "scaffold", semantic_state: "unknown", default_port: 8112, upstream: "http://127.0.0.1:8112" },
  { order: 13, slug: "helix-cura-prime", title: "HelixCuraPrime", description: "Enterprise clinical AI platform", tier: "frontier", maturity: "scaffold", semantic_state: "unknown", default_port: 8113, upstream: "http://127.0.0.1:8113" },
  { order: 14, slug: "helix-terra-prime", title: "HelixTerraPrime", description: "Precision agriculture & climate-smart farming OS", tier: "frontier", maturity: "scaffold", semantic_state: "unknown", default_port: 8114, upstream: "http://127.0.0.1:8114" },
  { order: 15, slug: "helix-climate-prime", title: "HelixClimatePrime", description: "Planetary-scale climate risk modeling & net-zero orchestration", tier: "frontier", maturity: "scaffold", semantic_state: "unknown", default_port: 8115, upstream: "http://127.0.0.1:8115" },
  { order: 16, slug: "helix-orbit-prime", title: "HelixOrbitPrime", description: "Commercial space operations & satellite intelligence", tier: "frontier", maturity: "scaffold", semantic_state: "unknown", default_port: 8116, upstream: "http://127.0.0.1:8116" },
  { order: 17, slug: "helix-quantum-forge", title: "HelixQuantumForge", description: "Hybrid quantum-classical computing platform", tier: "frontier", maturity: "scaffold", semantic_state: "unknown", default_port: 8117, upstream: "http://127.0.0.1:8117" },
  { order: 18, slug: "helix-vita-prime", title: "HelixVitaPrime", description: "Precision medicine & longevity research platform", tier: "frontier", maturity: "scaffold", semantic_state: "unknown", default_port: 8118, upstream: "http://127.0.0.1:8118" },
  { order: 19, slug: "helix-grid-prime", title: "HelixGridPrime", description: "Autonomous smart energy systems & renewable optimization", tier: "frontier", maturity: "scaffold", semantic_state: "unknown", default_port: 8119, upstream: "http://127.0.0.1:8119" },
  { order: 20, slug: "helix-nova-labs", title: "HelixNovaLabs", description: "Open scientific discovery accelerator", tier: "frontier", maturity: "scaffold", semantic_state: "unknown", default_port: 8120, upstream: "http://127.0.0.1:8120" },
  { order: 21, slug: "helix-pulse", title: "HelixPulse", description: "Sovereign distributed memory & cluster data plane (modern Redis-class) — build last", tier: "frontier", maturity: "scaffold", semantic_state: "unknown", default_port: 8121, upstream: "http://127.0.0.1:8121" },
];

function productWebUrl(slug: string, port: number): string | null {
  if (slug === "helix-collab") {
    return process.env.NEXT_PUBLIC_COLLAB_WEB ?? "http://127.0.0.1:3101";
  }
  if (slug === "helix-synthbio") {
    return process.env.NEXT_PUBLIC_SYNTHBIO_WEB ?? "http://127.0.0.1:3201";
  }
  void port;
  return null;
}

export function Catalog() {
  const [items, setItems] = useState<CatalogEntry[]>(FALLBACK);
  const [states, setStates] = useState<Record<string, SemanticState>>({});
  const [source, setSource] = useState<"gateway" | "fallback">("fallback");

  useEffect(() => {
    const base = process.env.NEXT_PUBLIC_GATEWAY_URL ?? "http://127.0.0.1:8080";
    const client = new GatewayClient(base);
    client
      .catalog()
      .then((data) => {
        setItems(data);
        setSource("gateway");
      })
      .catch(() => setSource("fallback"));
  }, []);

  useEffect(() => {
    if (source !== "gateway") return;
    const base = process.env.NEXT_PUBLIC_GATEWAY_URL ?? "http://127.0.0.1:8080";
    const client = new GatewayClient(base);
    let cancelled = false;
    Promise.all(
      items.map(async (p) => {
        try {
          const s = await client.catalogState(p.slug);
          return [p.slug, s.semantic_state] as const;
        } catch {
          return [p.slug, p.semantic_state] as const;
        }
      })
    ).then((pairs) => {
      if (cancelled) return;
      setStates(Object.fromEntries(pairs));
    });
    return () => {
      cancelled = true;
    };
  }, [items, source]);

  return (
    <>
      <div className="row">
        <span className="pill">
          Source: {source === "gateway" ? "live gateway" : "offline catalog"}
        </span>
        <span className="pill">{items.length} products</span>
      </div>
      <div className="grid">
        {items.map((p) => {
          const web = productWebUrl(p.slug, p.default_port);
          const api = p.upstream || `http://127.0.0.1:${p.default_port}`;
          const state = states[p.slug] ?? p.semantic_state;
          return (
            <article key={p.slug} className="card">
              <div className="row" style={{ marginBottom: "0.55rem", gap: "0.4rem" }}>
                <span className={`badge ${p.tier === "frontier" ? "frontier" : "standard"}`}>
                  #{p.order} · {p.tier}
                </span>
                <MaturityBadge maturity={p.maturity} />
                <SemanticBadge state={state} />
              </div>
              <h3>{p.title}</h3>
              <p>{p.description}</p>
              <div className="meta">
                <code>{p.slug}</code>
                <span>:{p.default_port}</span>
              </div>
              <div className="row" style={{ marginTop: "0.75rem", gap: "0.5rem" }}>
                {web && (
                  <a className="pill" href={web} target="_blank" rel="noreferrer">
                    Open UI
                  </a>
                )}
                <a className="pill" href={api} target="_blank" rel="noreferrer">
                  API
                </a>
                {p.slug === "helix-collab" && (
                  <a
                    className="pill"
                    href={`${api}/v1/domain/status`}
                    target="_blank"
                    rel="noreferrer"
                  >
                    Status
                  </a>
                )}
              </div>
            </article>
          );
        })}
      </div>
    </>
  );
}
