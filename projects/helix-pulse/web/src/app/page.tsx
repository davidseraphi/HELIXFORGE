export default function Home() {
  return (
    <main style={{ fontFamily: "system-ui", padding: "2rem", maxWidth: 720 }}>
      <h1>HelixPulse</h1>
      <p>
        Sovereign distributed memory &amp; cluster data plane (modern Redis-class).
      </p>
      <p>
        <strong>Build priority: last</strong> — after HelixCore and products 1–20.
      </p>
      <ul>
        <li>API scaffold: port 8121</li>
        <li>Cluster: not implemented (P3)</li>
        <li>Docs: <code>projects/helix-pulse/VISION.md</code></li>
      </ul>
    </main>
  );
}
