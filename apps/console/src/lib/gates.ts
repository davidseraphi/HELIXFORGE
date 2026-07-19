export type Gate = {
  num: string;
  name: string;
  fix: string;
  tag: string;
  tagClass?: "gold" | "violet";
  run: string;
};

export const GATES: Gate[] = [
  { num: "01", name: "HelixCollab", fix: "Torn create welded into one atomic write", tag: "atomic insert", run: "29661659103" },
  { num: "02", name: "HelixCapital", fix: "Ledger writes guarded at the statement level", tag: "atomic insert", run: "29662883748" },
  { num: "03", name: "HelixCommerce", fix: "Pool deadlock in cancel_order eliminated — items loaded inside the tx", tag: "deadlock killed", run: "29664024211" },
  { num: "04", name: "HelixFlow", fix: "Unbootable API restored; terminal runs made immutable (finished_at guard)", tag: "boot + terminal", run: "29665124925" },
  { num: "05", name: "HelixInsights", fix: "Points on deleted metrics rejected; 8 concurrent records all land", tag: "atomic insert", run: "29666090622" },
  { num: "06", name: "HelixEdu", fix: "8 racing enrolls → 1 winner; draft course rejects all comers", tag: "atomic insert", run: "29667121757" },
  { num: "07", name: "HelixWell", fix: "Paused habit rejects 8 racing logs; active habit lands all 8", tag: "atomic insert", run: "29667399976" },
  { num: "08", name: "HelixNetwork", fix: "request_connection: four statements welded into one FOR UPDATE transaction", tag: "transaction", run: "29668195166" },
  { num: "09", name: "HelixForge Studio", fix: "Deleted app rejects 8 racing pages; 8 racing publishes → 1", tag: "atomic insert", run: "29669148679" },
  { num: "10", name: "HelixSynthBio", fix: "Deleted design rejects 8 racing sims; 8 racing approves → 1", tag: "guarded update", run: "29669804701" },
  { num: "11", name: "HelixLex Prime", fix: "Deleted matter rejects 8 racing filings; 8 racing closes → 1", tag: "guarded update", run: "29670279394" },
  { num: "12", name: "HelixCura Prime", fix: "Deleted case rejects 8 racing notes; signed notes immutable under race", tag: "guarded update", run: "29670866072" },
  { num: "13", name: "HelixTerra Prime", fix: "Deleted field rejects 8 racing observations; 8 racing retires → 1", tag: "guarded update", run: "29671334631" },
  { num: "14", name: "HelixClimate Prime", fix: "Deleted scenario rejects 8 racing scores; 8 racing archives → 1", tag: "guarded update", run: "29671780109" },
  { num: "15", name: "HelixOrbit Prime", fix: "Deleted asset rejects 8 racing passes; 8 racing decommissions → 1", tag: "guarded update", run: "29672257327" },
  { num: "16", name: "HelixQuantum Forge", fix: "Deleted job rejects 8 racing circuits; 8 racing submits → 1", tag: "guarded update", run: "29672764891" },
  { num: "17", name: "HelixVita Prime", fix: "Deleted study rejects 8 racing cohorts; 8 racing completes → 1", tag: "guarded update", run: "29673285395" },
  { num: "18", name: "HelixGrid Prime", fix: "Deleted site rejects 8 racing readings; 8 racing offlines → 1", tag: "guarded update", run: "29685116830" },
  { num: "19", name: "HelixNova Labs", fix: "Deleted experiment rejects 8 racing findings; 8 racing concludes → 1", tag: "guarded update", run: "29685681271" },
  { num: "20", name: "HelixPulse", fix: "Deleted monitor rejects 8 racing incidents; 8 racing pauses → 1", tag: "guarded update", run: "29686421129" },
  { num: "21", name: "HelixCode", fix: "Terminal finishes guarded; FK-500s became clean 404s; boot bug killed", tag: "terminal guard", run: "29687099450" },
];

export const HARDENING: Gate[] = [
  { num: "H1", name: "Atomic Counters", fix: "16 racing issues + 16 racing events — all distinct, zero errors, zero MAX+1 window", tag: "row-locked upsert", tagClass: "violet", run: "29688633026" },
  { num: "H2", name: "Ref Compare-and-Swap", fix: "8 racing commits → clean conflicts; branch history is exactly seed + winners", tag: "cas at git + mirror", tagClass: "violet", run: "29688633026" },
];

export type Trophy = { glyph: string; title: string; text: string };

export const TROPHIES: Trophy[] = [
  { glyph: "◉", title: "The pool deadlock", text: "cancel_order loaded items outside its transaction — 8 racing cancels hung the pool. Killed in HelixCommerce." },
  { glyph: "◈", title: "The unbootable API", text: "nest_service at root panicked under current axum. Flow shipped dead; now it boots and its terminal runs are immutable." },
  { glyph: "◐", title: "The four-statement window", text: "request_connection checked profiles, blocks, and duplicates in separate reads. Now one transaction, rows locked." },
  { glyph: "◍", title: "The mutable signature", text: "A sign landing between read and write let an edit overwrite a signed note. signed_immutable now holds under race." },
  { glyph: "◎", title: "The second unbootable", text: "HelixCode had the same axum boot bug plus unguarded terminal finishes. Both found by the gate, both buried." },
];

export type Stat = { value: number; label: string };

export const STATS: Stat[] = [
  { value: 21, label: "products gated" },
  { value: 44, label: "race proofs" },
  { value: 21, label: "forced kills survived" },
  { value: 21, label: "restores hash-matched" },
  { value: 23, label: "windows welded shut" },
  { value: 22, label: "green CI runs" },
];

export const RUN_URL = (id: string) =>
  `https://github.com/davidseraphi/HELIXFORGE/actions/runs/${id}`;
