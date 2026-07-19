import { notFound, redirect } from "next/navigation";
import { findProduct } from "@/lib/products";
import { ProductApp } from "@/components/ProductApp";

export default async function ProductPage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const product = findProduct(slug);
  if (!product) notFound();
  if (product.external) redirect(product.external);
  // HelixSynthBio has a bespoke app under the static route; never fall back
  // to the generic engine for this slug.
  if (slug === "helix-synthbio") redirect("/products/helix-synthbio");
  return <ProductApp product={product} />;
}
