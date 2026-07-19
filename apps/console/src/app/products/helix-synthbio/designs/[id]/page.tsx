import { Design360 } from "@/components/synthbio/Design360";

export default async function Design360Page({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  return <Design360 id={id} />;
}
