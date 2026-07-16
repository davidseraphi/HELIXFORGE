import { Catalog } from "@/components/Catalog";

export default function HomePage() {
  return (
    <>
      <h1>HelixForge product catalog</h1>
      <p className="lead">
        Twenty interconnected products on one sovereign HelixCore platform.
        Auth, vault, agents, billing, observability, and audit are shared —
        products own domain logic only.
      </p>
      <Catalog />
    </>
  );
}
