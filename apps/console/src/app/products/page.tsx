import { LauncherGrid } from "@/components/LauncherGrid";

export default function ProductsPage() {
  return (
    <>
      <h1>The suite</h1>
      <p className="lead">
        Twenty-one products, one sovereign platform. Open any of them — every
        one is live behind the console, fully usable: real entities, real
        lifecycle transitions, real guard rails.
      </p>
      <LauncherGrid />
    </>
  );
}
