import { TROPHIES } from "@/lib/gates";

export function Trophies() {
  return (
    <div className="trophies">
      {TROPHIES.map((t) => (
        <div key={t.title} className="trophy">
          <div className="trophy-glyph">{t.glyph}</div>
          <h4>{t.title}</h4>
          <p>{t.text}</p>
        </div>
      ))}
    </div>
  );
}
