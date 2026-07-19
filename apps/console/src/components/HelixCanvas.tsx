"use client";

import { useEffect, useRef } from "react";

/** Ambient DNA-helix backdrop. Fixed, non-interactive, sleeps when hidden. */
export function HelixCanvas() {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const c = ref.current;
    if (!c) return;
    const ctx = c.getContext("2d");
    if (!ctx) return;

    let raf = 0;
    let t = 0;
    const A = 46;
    const PERIOD = 150;
    const SPEED = 0.35;

    const size = () => {
      c.width = window.innerWidth;
      c.height = window.innerHeight;
    };
    size();
    window.addEventListener("resize", size);

    const frame = () => {
      t += SPEED;
      const w = c.width;
      const h = c.height;
      const cy = h * 0.42;
      ctx.clearRect(0, 0, w, h);

      for (let strand = 0; strand < 2; strand++) {
        ctx.beginPath();
        for (let x = -20; x <= w + 20; x += 4) {
          const ph = ((x + t) / PERIOD) * Math.PI * 2 + strand * Math.PI;
          const y = cy + Math.sin(ph) * A;
          if (x === -20) ctx.moveTo(x, y);
          else ctx.lineTo(x, y);
        }
        ctx.strokeStyle = strand === 0 ? "rgba(94,234,212,.09)" : "rgba(129,140,248,.08)";
        ctx.lineWidth = 1.4;
        ctx.stroke();
      }

      for (let x = 0; x <= w; x += 34) {
        const ph = ((x + t) / PERIOD) * Math.PI * 2;
        const y1 = cy + Math.sin(ph) * A;
        const y2 = cy + Math.sin(ph + Math.PI) * A;
        const rung = Math.abs(Math.cos(ph));
        ctx.beginPath();
        ctx.moveTo(x, y1);
        ctx.lineTo(x, y2);
        ctx.strokeStyle = `rgba(139,155,184,${0.02 + rung * 0.05})`;
        ctx.lineWidth = 1;
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(x, y1, 1.4, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(94,234,212,${0.05 + rung * 0.1})`;
        ctx.fill();
        ctx.beginPath();
        ctx.arc(x, y2, 1.4, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(129,140,248,${0.04 + rung * 0.08})`;
        ctx.fill();
      }

      raf = requestAnimationFrame(frame);
    };

    const onVis = () => {
      if (document.hidden) {
        cancelAnimationFrame(raf);
      } else {
        raf = requestAnimationFrame(frame);
      }
    };

    raf = requestAnimationFrame(frame);
    document.addEventListener("visibilitychange", onVis);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("resize", size);
      document.removeEventListener("visibilitychange", onVis);
    };
  }, []);

  return <canvas ref={ref} className="helix-canvas" aria-hidden />;
}
