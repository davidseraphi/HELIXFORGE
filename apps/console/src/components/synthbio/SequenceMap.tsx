"use client";

import { useMemo, useState } from "react";
import {
  ROLE_COLORS,
  hexToRgba,
  roleFamily,
  type Component,
  type DesignVersion,
} from "./lib";

type Props = {
  version: DesignVersion;
  selected: number | null; // index into version.components
  onSelect: (idx: number | null) => void;
};

/** Clamp a feature's 1-based inclusive range into [1, len]. */
function clampRange(c: Component, len: number): { lo: number; hi: number } {
  const lo = Math.max(1, Math.min(c.start, c.end));
  const hi = Math.min(len, Math.max(c.start, c.end));
  return { lo, hi: Math.max(lo, hi) };
}

/** Tick step for the ruler: 100 bp for short sequences, scaled up smartly. */
function niceStep(len: number): number {
  const target = len / 8;
  const steps = [100, 200, 250, 500, 1000, 2000, 2500, 5000, 10000, 20000, 25000, 50000, 100000];
  for (const s of steps) if (s >= target) return s;
  return Math.pow(10, Math.ceil(Math.log10(Math.max(target, 1))));
}

export function SequenceMap({ version, selected, onSelect }: Props) {
  const seq = version.sequence_text.replace(/\s+/g, "");
  const len = seq.length || version.sequence_length;
  const [mode, setMode] = useState<"linear" | "circular">(
    version.topology === "circular" ? "circular" : "linear",
  );

  const selectedComp = selected != null ? version.components[selected] : undefined;

  if (!seq) {
    return <p className="muted">No sequence deposited on the current version.</p>;
  }

  return (
    <div className="sb-map">
      <div className="sb-map-toolbar">
        <div className="sb-seg">
          <button
            type="button"
            className={`sb-seg-btn${mode === "linear" ? " active" : ""}`}
            onClick={() => setMode("linear")}
          >
            Linear
          </button>
          <button
            type="button"
            className={`sb-seg-btn${mode === "circular" ? " active" : ""}`}
            onClick={() => setMode("circular")}
          >
            Circular
          </button>
        </div>
        <span className="muted">
          {len.toLocaleString()} bp · {version.topology} · click a feature to select it
        </span>
      </div>

      {mode === "linear" ? (
        <LinearMap version={version} len={len} selected={selected} onSelect={onSelect} />
      ) : (
        <CircularMap version={version} len={len} selected={selected} onSelect={onSelect} />
      )}

      <SequencePanel
        seq={seq}
        feature={
          selectedComp
            ? { ...clampRange(selectedComp, len), color: ROLE_COLORS[roleFamily(selectedComp.role_so, selectedComp.name)] }
            : null
        }
      />
    </div>
  );
}

/* ————————————————— Linear map ————————————————— */

function LinearMap({
  version,
  len,
  selected,
  onSelect,
}: {
  version: DesignVersion;
  len: number;
  selected: number | null;
  onSelect: (idx: number | null) => void;
}) {
  const W = 960;
  const PAD = 10;
  const track = W - PAD * 2;
  const H = 132;
  const rulerY = 30;
  const fwdY = 46; // forward lane top
  const revY = 84; // reverse lane top
  const laneH = 20;
  const backboneY = 78;

  const x = (bp: number) => PAD + ((bp - 1) / Math.max(len, 1)) * track;
  const step = niceStep(len);
  const ticks: number[] = [];
  for (let bp = 1; bp <= len; bp += step) ticks.push(bp);
  if (ticks[ticks.length - 1] !== len) ticks.push(len);

  return (
    <svg className="sb-svg" viewBox={`0 0 ${W} ${H}`} role="img" aria-label="linear sequence map">
      {/* ruler */}
      <line x1={x(1)} y1={rulerY} x2={x(len)} y2={rulerY} className="sb-ruler-line" />
      {ticks.map((bp) => (
        <g key={bp}>
          <line x1={x(bp)} y1={rulerY - 4} x2={x(bp)} y2={rulerY + 4} className="sb-ruler-tick" />
          <text x={x(bp)} y={rulerY - 8} className="sb-ruler-label" textAnchor="middle">
            {bp.toLocaleString()}
          </text>
        </g>
      ))}

      {/* backbone + lane hints */}
      <line x1={x(1)} y1={backboneY} x2={x(len)} y2={backboneY} className="sb-backbone" />
      <text x={PAD} y={fwdY - 6} className="sb-lane-hint">
        ▲ + strand
      </text>
      <text x={PAD} y={revY + laneH + 12} className="sb-lane-hint">
        ▼ − strand
      </text>

      {/* features */}
      {version.components.map((c, i) => {
        const { lo, hi } = clampRange(c, len);
        const fwd = c.strand >= 0;
        const x0 = x(lo);
        const w = Math.max(x(hi) + track / Math.max(len, 1) - x0, 3);
        const y = fwd ? fwdY : revY;
        const fam = roleFamily(c.role_so, c.name);
        const color = ROLE_COLORS[fam];
        const isSel = selected === i;
        const charW = 5.8; // approx width of one 10px char
        const labelW = c.name.length * charW + 8;
        // arrow chevron pointing in the strand direction
        const ax = fwd ? x0 + w - 1 : x0 + 1;
        const chev = fwd
          ? `${ax - 6},${y + 3} ${ax},${y + laneH / 2} ${ax - 6},${y + laneH - 3}`
          : `${ax + 6},${y + 3} ${ax},${y + laneH / 2} ${ax + 6},${y + laneH - 3}`;
        return (
          <g
            key={i}
            className="sb-feature"
            onClick={() => onSelect(isSel ? null : i)}
            opacity={selected == null || isSel ? 1 : 0.55}
          >
            <title>{`${c.name} · ${c.role_so} · ${c.start}–${c.end} (${fwd ? "+" : "−"} strand)`}</title>
            <rect
              x={x0}
              y={y}
              width={w}
              height={laneH}
              rx={3}
              fill={hexToRgba(color, isSel ? 0.95 : 0.8)}
              stroke={isSel ? "#0f1b2d" : color}
              strokeWidth={isSel ? 1.6 : 1}
            />
            <polyline points={chev} fill="none" stroke="#ffffff" strokeWidth={1.6} />
            {w >= 14 && (
              <text
                x={x0 + w / 2}
                y={y + laneH / 2 + 3.5}
                className="sb-feature-label"
                textAnchor="middle"
                textLength={labelW > w - 4 ? Math.max(w - 4, 10) : undefined}
                lengthAdjust="spacingAndGlyphs"
              >
                {c.name}
              </text>
            )}
          </g>
        );
      })}
    </svg>
  );
}

/* ————————————————— Circular map ————————————————— */

function polar(cx: number, cy: number, r: number, deg: number): [number, number] {
  const rad = (deg * Math.PI) / 180;
  return [cx + r * Math.cos(rad), cy + r * Math.sin(rad)];
}

/** Donut-segment arc path between two angles (degrees, clockwise from 12 o'clock). */
function arcPath(
  cx: number,
  cy: number,
  rOuter: number,
  rInner: number,
  a0: number,
  a1: number,
): string {
  const large = a1 - a0 > 180 ? 1 : 0;
  const [x0, y0] = polar(cx, cy, rOuter, a0);
  const [x1, y1] = polar(cx, cy, rOuter, a1);
  const [x2, y2] = polar(cx, cy, rInner, a1);
  const [x3, y3] = polar(cx, cy, rInner, a0);
  return [
    `M ${x0.toFixed(2)} ${y0.toFixed(2)}`,
    `A ${rOuter} ${rOuter} 0 ${large} 1 ${x1.toFixed(2)} ${y1.toFixed(2)}`,
    `L ${x2.toFixed(2)} ${y2.toFixed(2)}`,
    `A ${rInner} ${rInner} 0 ${large} 0 ${x3.toFixed(2)} ${y3.toFixed(2)}`,
    "Z",
  ].join(" ");
}

function CircularMap({
  version,
  len,
  selected,
  onSelect,
}: {
  version: DesignVersion;
  len: number;
  selected: number | null;
  onSelect: (idx: number | null) => void;
}) {
  const S = 460;
  const cx = S / 2;
  const cy = S / 2;
  const rBackbone = 150;
  const rOuter = 162; // forward band outer edge
  const rInner = 138; // reverse band inner edge
  const rTick = 172;

  // position (1-based bp) → degrees clockwise from 12 o'clock
  const ang = (bp: number) => ((bp - 1) / Math.max(len, 1)) * 360;

  // split a feature range at the origin so arcs never wrap past 360°
  const spans = (c: Component): { a0: number; a1: number }[] => {
    const { lo, hi } = clampRange(c, len);
    if (hi - lo >= len - 1) return [{ a0: 0, a1: 359.9 }];
    const a0 = ang(lo);
    const a1 = ang(hi) + 360 / Math.max(len, 1); // inclusive end
    if (a1 <= 360) return [{ a0, a1 }];
    return [
      { a0, a1: 360 },
      { a0: 0, a1: a1 - 360 },
    ];
  };

  const legend = useMemo(
    () =>
      version.components.map((c, i) => ({
        i,
        name: c.name,
        fam: roleFamily(c.role_so, c.name),
      })),
    [version.components],
  );

  return (
    <div className="sb-circular-wrap">
      <svg className="sb-svg sb-svg-circular" viewBox={`0 0 ${S} ${S}`} role="img" aria-label="circular plasmid map">
        {/* degree ticks */}
        {Array.from({ length: 12 }, (_, k) => k * 30).map((deg) => {
          const [tx0, ty0] = polar(cx, cy, rTick, deg);
          const [tx1, ty1] = polar(cx, cy, rTick + 8, deg);
          const [lx, ly] = polar(cx, cy, rTick + 20, deg);
          return (
            <g key={deg}>
              <line x1={tx0} y1={ty0} x2={tx1} y2={ty1} className="sb-ruler-tick" />
              <text x={lx} y={ly + 3} className="sb-ruler-label" textAnchor="middle">
                {deg}°
              </text>
            </g>
          );
        })}

        {/* backbone */}
        <circle cx={cx} cy={cy} r={rBackbone} className="sb-backbone-circle" />

        {/* features: + strand on the outer band, − strand on the inner band */}
        {version.components.map((c, i) => {
          const fwd = c.strand >= 0;
          const fam = roleFamily(c.role_so, c.name);
          const color = ROLE_COLORS[fam];
          const isSel = selected === i;
          const [ro, ri] = fwd ? [rOuter, rBackbone + 2] : [rBackbone - 2, rInner];
          return (
            <g
              key={i}
              className="sb-feature"
              onClick={() => onSelect(isSel ? null : i)}
              opacity={selected == null || isSel ? 1 : 0.5}
            >
              <title>{`${c.name} · ${c.role_so} · ${c.start}–${c.end} (${fwd ? "+" : "−"} strand)`}</title>
              {spans(c).map((s, k) => (
                <path
                  key={k}
                  d={arcPath(cx, cy, ro, ri, s.a0, s.a1)}
                  fill={hexToRgba(color, isSel ? 0.95 : 0.78)}
                  stroke={isSel ? "#0f1b2d" : color}
                  strokeWidth={isSel ? 1.6 : 1}
                />
              ))}
            </g>
          );
        })}
      </svg>

      {legend.length > 0 && (
        <ul className="sb-legend">
          {legend.map((l) => (
            <li key={l.i}>
              <button
                type="button"
                className={`sb-legend-item${selected === l.i ? " active" : ""}`}
                onClick={() => onSelect(selected === l.i ? null : l.i)}
              >
                <span className="sb-swatch" style={{ background: ROLE_COLORS[l.fam] }} />
                {l.name}
                <span className="muted"> · {l.fam}</span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/* ————————————————— Sequence panel ————————————————— */

const SEQ_WINDOW = 4000;
const GROUP = 10;
const PER_LINE = 60;

function SequencePanel({
  seq,
  feature,
}: {
  seq: string;
  feature: { lo: number; hi: number; color: string } | null;
}) {
  const len = seq.length;

  // Window the sequence at SEQ_WINDOW bases, centered on the selected feature.
  let winStart = 1; // 1-based inclusive
  if (len > SEQ_WINDOW) {
    if (feature) {
      const mid = Math.floor((feature.lo + feature.hi) / 2);
      winStart = Math.max(1, Math.min(mid - Math.floor(SEQ_WINDOW / 2), len - SEQ_WINDOW + 1));
    }
  }
  const winEnd = Math.min(len, winStart + SEQ_WINDOW - 1);

  const lines: { start: number; text: string }[] = [];
  for (let s = winStart; s <= winEnd; s += PER_LINE) {
    lines.push({ start: s, text: seq.slice(s - 1, Math.min(s - 1 + PER_LINE, winEnd)) });
  }

  // merge consecutive bases with the same highlight state into spans
  const renderLine = (line: { start: number; text: string }) => {
    const out: { text: string; hl: boolean; key: number }[] = [];
    let cur = "";
    let curHl: boolean | null = null;
    for (let i = 0; i < line.text.length; i++) {
      const pos = line.start + i; // 1-based
      const hl = feature != null && pos >= feature.lo && pos <= feature.hi;
      if (curHl === null || hl === curHl) {
        cur += line.text[i];
        curHl = hl;
      } else {
        out.push({ text: cur, hl: curHl, key: i });
        cur = line.text[i];
        curHl = hl;
      }
    }
    if (cur) out.push({ text: cur, hl: curHl ?? false, key: line.text.length });
    // insert group spacing: rebuild spans with a space every GROUP bases
    const spaced: { text: string; hl: boolean; key: string }[] = [];
    let col = 0;
    for (const seg of out) {
      for (const ch of seg.text) {
        if (col > 0 && col % GROUP === 0) spaced.push({ text: " ", hl: false, key: `sp-${line.start}-${col}` });
        spaced.push({ text: ch, hl: seg.hl, key: `c-${line.start + col}` });
        col++;
      }
    }
    return spaced;
  };

  return (
    <div className="sb-seq-panel">
      <div className="sb-seq-meta muted">
        {len > SEQ_WINDOW
          ? `showing bases ${winStart.toLocaleString()}–${winEnd.toLocaleString()} of ${len.toLocaleString()} bp`
          : `${len.toLocaleString()} bp`}
        {feature ? ` · selection ${feature.lo.toLocaleString()}–${feature.hi.toLocaleString()}` : ""}
      </div>
      <pre className="sb-seq">
        {lines.map((line) => (
          <div key={line.start} className="sb-seq-line">
            <span className="sb-seq-gutter">{line.start.toLocaleString()}</span>
            <span className="sb-seq-bases">
              {renderLine(line).map((seg) =>
                seg.hl && feature ? (
                  <span key={seg.key} style={{ background: hexToRgba(feature.color, 0.28) }}>
                    {seg.text}
                  </span>
                ) : (
                  <span key={seg.key}>{seg.text}</span>
                ),
              )}
            </span>
          </div>
        ))}
      </pre>
    </div>
  );
}
