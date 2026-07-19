import { Counter } from "@/components/Counter";
import { HelixCanvas } from "@/components/HelixCanvas";
import { GateWall } from "@/components/GateWall";
import { Trophies } from "@/components/Trophies";
import { DeployLog } from "@/components/DeployLog";
import { GATES, HARDENING, STATS } from "@/lib/gates";

export default function ObservatoryPage() {
  return (
    <>
      <HelixCanvas />
      <section className="obs-hero">
        <div className="obs-kicker">Foundation Integrity · Durability Gate Program</div>
        <h1 className="obs-h1">
          Every write survives <span className="obs-grad">its own crash.</span>
        </h1>
        <div className="obs-counter">
          <Counter target={21} duration={650} />
          <span className="obs-of"> / 21</span>
        </div>
        <p className="lead">
          <strong>21 products. One gate.</strong> Each one raced 8 writers against
          itself, was killed mid-acknowledgement, dumped and restored row-for-row —
          and came back green, locally and in CI. This is the proof wall.
        </p>
        <div className="obs-pill">
          <span className="obs-dot" />
          All systems gate-proven
        </div>
      </section>

      <section className="obs-stats">
        {STATS.map((s) => (
          <div key={s.label} className="obs-stat">
            <div className="obs-stat-v">
              <Counter target={s.value} duration={900} />
            </div>
            <div className="obs-stat-l">{s.label}</div>
          </div>
        ))}
      </section>

      <section className="obs-sec">
        <div className="obs-sec-head">
          <h2>The ritual</h2>
          <span>same gate, every product</span>
        </div>
        <div className="ritual">
          <div className="step">
            <div className="step-num">01 / RACE</div>
            <h3>8 writers, 1 row</h3>
            <p>
              Eight concurrent writes fire at one invariant. Exactly one may win;
              the rest must conflict cleanly. No silent losers.
            </p>
          </div>
          <div className="step">
            <div className="step-num">02 / KILL</div>
            <h3>SIGKILL mid-ack</h3>
            <p>
              The API acknowledges a write, then is force-killed on the spot. On
              restart, the acknowledged state must be fully present.
            </p>
          </div>
          <div className="step">
            <div className="step-num">03 / RESTORE</div>
            <h3>Dump &amp; compare</h3>
            <p>
              The product schema is pg_dump&apos;d into a scratch database. Row
              counts and md5 content hashes must match exactly.
            </p>
          </div>
          <div className="step">
            <div className="step-num">04 / PROVE</div>
            <h3>CI or it didn&apos;t happen</h3>
            <p>
              Every gate runs as its own GitHub Actions job — race tests plus the
              kill-and-restore script, on Ubuntu, on record.
            </p>
          </div>
        </div>
      </section>

      <section className="obs-sec">
        <div className="obs-sec-head">
          <h2>The gate wall</h2>
          <span>21 / 21 green</span>
        </div>
        <GateWall gates={GATES} />
      </section>

      <section className="obs-sec">
        <div className="obs-sec-head">
          <h2>Hardening tier</h2>
          <span>beyond the gate</span>
        </div>
        <GateWall gates={HARDENING} />
      </section>

      <section className="obs-sec">
        <div className="obs-sec-head">
          <h2>Trophies</h2>
          <span>real bugs, killed on record</span>
        </div>
        <Trophies />
      </section>

      <section className="obs-sec">
        <div className="obs-sec-head">
          <h2>Deploy log</h2>
          <span>chronological record</span>
        </div>
        <DeployLog gates={[...GATES, ...HARDENING]} />
      </section>
    </>
  );
}
