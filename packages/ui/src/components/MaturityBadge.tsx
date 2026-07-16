"use client";

export type ProductMaturity =
  | "scaffold"
  | "prototype"
  | "alpha"
  | "beta"
  | "production";

const LABELS: Record<ProductMaturity, string> = {
  scaffold: "Scaffold",
  prototype: "Prototype",
  alpha: "Alpha",
  beta: "Beta",
  production: "Production",
};

export function MaturityBadge({ maturity }: { maturity: ProductMaturity | string }) {
  const key =
    (maturity as ProductMaturity) in LABELS ? (maturity as ProductMaturity) : "scaffold";
  return <span className={`maturity-badge maturity-${key}`}>{LABELS[key]}</span>;
}
