import { RUN_URL, type Gate } from "@/lib/gates";

export function DeployLog({ gates }: { gates: Gate[] }) {
  return (
    <div className="log">
      {gates.map((g) => (
        <div key={g.num} className="log-row">
          <span className="log-check">✓</span>
          <span className="log-name">
            {g.name}
            <span>{g.fix}</span>
          </span>
          <span className="log-tag">gate green</span>
          <a className="log-run" href={RUN_URL(g.run)} target="_blank" rel="noopener noreferrer">
            {g.run}
          </a>
        </div>
      ))}
    </div>
  );
}
