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
  return <ProductApp product={product} />;
}
