"use client";

import { useEffect, useState } from "react";

export default function HealthPage() {
  const [payload, setPayload] = useState<string>("Loading gateway /healthz …");

  useEffect(() => {
    const base = process.env.NEXT_PUBLIC_GATEWAY_URL ?? "http://127.0.0.1:8080";
    fetch(`${base}/healthz`)
      .then(async (r) => {
        const text = await r.text();
        setPayload(`${r.status} ${r.statusText}\n\n${text}`);
      })
      .catch((e) => setPayload(`Gateway unreachable: ${e}`));
  }, []);

  return (
    <>
      <h1>Platform health</h1>
      <p className="lead">
        Live probe against the gateway. Start HelixCore with{" "}
        <code>scripts/dev-core.ps1</code> or individual <code>cargo run -p …</code>.
      </p>
      <div className="panel">
        <pre>{payload}</pre>
      </div>
    </>
  );
}
