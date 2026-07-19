"use client";

import { useEffect, useRef, useState } from "react";

type CounterProps = {
  target: number;
  duration?: number;
  className?: string;
};

/** Count-up that starts the first time it scrolls into view (easeOutExpo). */
export function Counter({ target, duration = 800, className }: CounterProps) {
  const ref = useRef<HTMLSpanElement>(null);
  const started = useRef(false);
  const [val, setVal] = useState(0);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const io = new IntersectionObserver(
      (entries) => {
        if (!entries[0]?.isIntersecting || started.current) return;
        started.current = true;
        const t0 = performance.now();
        const tick = (t: number) => {
          const p = Math.min((t - t0) / duration, 1);
          const e = 1 - Math.pow(2, -10 * p);
          setVal(Math.round(target * e));
          if (p < 1) requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
      },
      { threshold: 0.3 },
    );
    io.observe(el);
    return () => io.disconnect();
  }, [target, duration]);

  return (
    <span ref={ref} className={className}>
      {val}
    </span>
  );
}
