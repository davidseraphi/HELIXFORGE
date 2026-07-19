"use client";

import { useMemo } from "react";
import { ROLE_COLORS, shortId, type Design360Data } from "./lib";

type Node = {
  id: string;
  kind: "design" | "version" | "risk";
  label: string;
  sub: string;
  x: number;
  y: number;
  w: number;
  h: number;
};

type Edge = { from: string; to: string; label: string };

/**
 * Lineage graph: design centered-left, versions in a chain to the right,
 * risk case below. Edges come from the registry's lineage_edges and are
 * labeled contains / derived-from / reviews with direction arrows.
 */
export function LineageGraph({ data }: { data: Design360Data }) {
  const { nodes, edges, width, height } = useMemo(() => {
    const versions = [...data.versions].sort((a, b) => a.version - b.version);
    const nodes: Node[] = [];
    const byId = new Map<string, Node>();

    const designNode: Node = {
      id: data.design.id,
      kind: "design",
      label: data.design.accession,
      sub: "design",
      x: 30,
      y: 96,
      w: 128,
      h: 44,
    };
    nodes.push(designNode);
    byId.set(designNode.id, designNode);

    versions.forEach((v, i) => {
      const n: Node = {
        id: v.id,
        kind: "version",
        label: `v${v.version}`,
        sub: `${v.sequence_length.toLocaleString()} bp · ${shortId(v.id)}`,
        x: 250 + i * 170,
        y: 100,
        w: 120,
        h: 38,
      };
      nodes.push(n);
      byId.set(n.id, n);
    });

    let riskNode: Node | null = null;
    if (data.risk_case) {
      riskNode = {
        id: data.risk_case.id,
        kind: "risk",
        label: data.risk_case.state,
        sub: "risk case",
        x: 30,
        y: 230,
        w: 128,
        h: 40,
      };
      nodes.push(riskNode);
      byId.set(riskNode.id, riskNode);
    }

    const edges: Edge[] = [];
    for (const e of data.edges) {
      if (!byId.has(e.parent_id) || !byId.has(e.child_id)) continue;
      edges.push({ from: e.parent_id, to: e.child_id, label: e.relation });
    }
    // a decided/open risk case reviews its design even when no explicit edge exists
    if (riskNode && !edges.some((e) => e.from === riskNode.id || e.to === riskNode.id)) {
      edges.push({ from: riskNode.id, to: designNode.id, label: "reviews" });
    }

    const width = Math.max(560, 250 + versions.length * 170 + 30);
    const height = riskNode ? 310 : 190;
    return { nodes, edges, width, height };
  }, [data]);

  const byId = new Map(nodes.map((n) => [n.id, n]));

  // anchor points on a node's border toward another node
  const anchor = (a: Node, b: Node): { x0: number; y0: number; x1: number; y1: number } => {
    const acx = a.x + a.w / 2;
    const acy = a.y + a.h / 2;
    const bcx = b.x + b.w / 2;
    const bcy = b.y + b.h / 2;
    const dx = bcx - acx;
    const dy = bcy - acy;
    if (Math.abs(dx) > Math.abs(dy)) {
      const s = Math.sign(dx) || 1;
      return { x0: acx + (s * a.w) / 2, y0: acy, x1: bcx - (s * b.w) / 2, y1: bcy };
    }
    const s = Math.sign(dy) || 1;
    return { x0: acx, y0: acy + (s * a.h) / 2, x1: bcx, y1: bcy - (s * b.h) / 2 };
  };

  const nodeClass = (n: Node) =>
    n.kind === "design" ? "sb-lg-node-design" : n.kind === "risk" ? `sb-lg-node-risk sb-risk-${n.label}` : "sb-lg-node-version";

  return (
    <div className="sb-lg-wrap">
      <svg
        className="sb-svg"
        viewBox={`0 0 ${width} ${height}`}
        role="img"
        aria-label="lineage graph"
      >
        <defs>
          <marker
            id="sb-lg-arrow"
            viewBox="0 0 10 10"
            refX="9"
            refY="5"
            markerWidth="7"
            markerHeight="7"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 10 5 L 0 10 z" className="sb-lg-arrowhead" />
          </marker>
        </defs>

        {edges.map((e, i) => {
          const a = byId.get(e.from);
          const b = byId.get(e.to);
          if (!a || !b) return null;
          const { x0, y0, x1, y1 } = anchor(a, b);
          const mx = (x0 + x1) / 2;
          const my = (y0 + y1) / 2;
          return (
            <g key={i}>
              <line x1={x0} y1={y0} x2={x1} y2={y1} className="sb-lg-edge" markerEnd="url(#sb-lg-arrow)" />
              <text x={mx} y={my - 5} className="sb-lg-edge-label" textAnchor="middle">
                {e.label}
              </text>
            </g>
          );
        })}

        {nodes.map((n) => (
          <g key={n.id} className="sb-lg-node">
            <rect x={n.x} y={n.y} width={n.w} height={n.h} rx={8} className={nodeClass(n)} />
            <text x={n.x + n.w / 2} y={n.y + n.h / 2 - 2} className="sb-lg-node-label" textAnchor="middle">
              {n.label}
            </text>
            <text x={n.x + n.w / 2} y={n.y + n.h / 2 + 12} className="sb-lg-node-sub" textAnchor="middle">
              {n.sub}
            </text>
          </g>
        ))}
      </svg>

      <div className="sb-legend-row">
        <span className="sb-swatch" style={{ background: "#0d9488" }} /> design
        <span className="sb-swatch" style={{ background: "#2563eb" }} /> version
        <span className="sb-swatch" style={{ background: ROLE_COLORS.other }} /> risk case
      </div>
    </div>
  );
}
