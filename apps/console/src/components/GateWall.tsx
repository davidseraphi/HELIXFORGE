import { RUN_URL, type Gate } from "@/lib/gates";

type GateWallProps = {
  gates: Gate[];
  startDelay?: number;
};

export function GateWall({ gates, startDelay = 0.1 }: GateWallProps) {
  return (
    <div className="wall">
      {gates.map((g, i) => (
        <article
          key={g.num}
          className="gtile"
          style={{ animationDelay: `${startDelay + i * 0.022}s` }}
        >
          <div className="gtile-top">
            <span className="gtile-num">{g.num}</span>
            <span className="gtile-check" aria-hidden>
              <svg viewBox="0 0 24 24">
                <path d="M4 12.5l5 5L20 6.5" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </span>
          </div>
          <h3>{g.name}</h3>
          <p className="gtile-fix">{g.fix}</p>
          <div className="gtile-foot">
            <span className={`gtag ${g.tagClass ?? ""}`}>{g.tag}</span>
            <a className="grun" href={RUN_URL(g.run)} target="_blank" rel="noopener noreferrer">
              run {g.run} ↗
            </a>
          </div>
        </article>
      ))}
    </div>
  );
}
