import { JourneyDetail } from "@/components/synthbio/JourneyDetail";

export default async function JourneyDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  return (
    <div className="sb-theme">
      <JourneyDetail id={id} />
    </div>
  );
}
