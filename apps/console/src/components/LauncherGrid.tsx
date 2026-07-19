"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { PRODUCTS } from "@/lib/products";

type Health = "checking" | "up" | "down";

export function LauncherGrid() {
  const [health, setHealth] = useState<Record<string, Health>>({});

  useEffect(() => {
    PRODUCTS.forEach((p, i) => {
      const t = setTimeout(() => {
        fetch(`/api/p/${p.slug}/healthz`, { cache: "no-store" })
          .then((r) => setHealth((h) => ({ ...h, [p.slug]: r.ok ? "up" : "down" })))
          .catch(() => setHealth((h) => ({ ...h, [p.slug]: "down" })));
      }, i * 60);
      return () => clearTimeout(t);
    });
  }, []);

  return (
    <div className="launcher">
      {PRODUCTS.map((p, i) => {
        const state = health[p.slug] ?? "checking";
        const href = p.external ?? `/products/${p.slug}`;
        const external = Boolean(p.external);
        const card = (
          <article className="app-tile" style={{ animationDelay: `${0.05 + i * 0.03}s` }}>
            <div className="app-glyph">{p.glyph}</div>
            <div className="app-meta">
              <h3>{p.title}</h3>
              <p>{p.blurb}</p>
            </div>
            <div className={`app-state ${state}`}>
              <span className="app-dot" />
              {state === "checking" ? "…" : state === "up" ? (external ? "app" : "live") : "down"}
            </div>
          </article>
        );
        return external ? (
          <a key={p.slug} href={href} target="_blank" rel="noreferrer" className="app-link">
            {card}
          </a>
        ) : (
          <Link key={p.slug} href={href} className="app-link">
            {card}
          </Link>
        );
      })}
    </div>
  );
}
