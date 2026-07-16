"use client";

import { useEffect, useState } from "react";

const CORE = [
  { name: "gateway", port: 8080, role: "API edge / catalog" },
  { name: "agent-hub", port: 8081, role: "Agent orchestration" },
  { name: "vault-service", port: 8082, role: "Secrets / envelope encryption" },
  { name: "billing-service", port: 8083, role: "Usage metering" },
  { name: "observability-service", port: 8084, role: "Metrics + audit verify" },
  { name: "auth-adapter", port: 8085, role: "Ory Kratos/Hydra adapter" },
];

export default function CorePage() {
  const [health, setHealth] = useState<Record<string, boolean>>({});

  useEffect(() => {
    CORE.forEach((svc) => {
      fetch(`http://127.0.0.1:${svc.port}/healthz`)
        .then((r) => setHealth((h) => ({ ...h, [svc.name]: r.ok })))
        .catch(() => setHealth((h) => ({ ...h, [svc.name]: false })));
    });
  }, []);

  return (
    <>
      <h1>HelixCore services</h1>
      <p className="lead">
        Shared platform services. All product APIs depend on these via{" "}
        <code>service-kit</code>, NATS subjects, and HTTP clients.
      </p>
      <div className="grid">
        {CORE.map((svc) => (
          <article key={svc.name} className="card">
            <span className="badge standard">core</span>
            <h3>{svc.name}</h3>
            <p>{svc.role}</p>
            <div className="meta">
              <span>localhost:{svc.port}</span>
              <span className={health[svc.name] ? "status-ok" : "status-bad"}>
                {health[svc.name] === undefined
                  ? "…"
                  : health[svc.name]
                    ? "healthy"
                    : "down"}
              </span>
            </div>
          </article>
        ))}
      </div>
    </>
  );
}
