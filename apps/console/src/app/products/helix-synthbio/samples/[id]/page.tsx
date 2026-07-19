import { SampleDetail } from "@/components/synthbio/SampleDetail";

export default async function SampleDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  return (
    <div className="sb-theme">
      <SampleDetail id={id} />
    </div>
  );
}
