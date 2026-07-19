import { NextRequest, NextResponse } from "next/server";
import { findProduct } from "@/lib/products";

const DEV_USER = "ops@helixforge.local";

/**
 * Backend-for-frontend proxy: the browser only talks to the console;
 * the console calls product APIs server-side and injects the dev
 * principal — no CORS, no credentials in the client.
 *
 *   /api/p/<slug>/<path...>?query → http://127.0.0.1:<port>/<path...>?query
 */
async function proxy(req: NextRequest, ctx: { params: Promise<{ parts: string[] }> }) {
  const { parts } = await ctx.params;
  const [slug, ...rest] = parts;
  const product = findProduct(slug);
  if (!product) {
    return NextResponse.json({ error: `unknown product: ${slug}` }, { status: 404 });
  }

  const url = `http://127.0.0.1:${product.port}/${rest.join("/")}${req.nextUrl.search}`;
  const init: RequestInit = {
    method: req.method,
    headers: {
      "x-helix-dev-user": DEV_USER,
      "content-type": "application/json",
    },
    cache: "no-store",
  };
  if (req.method !== "GET" && req.method !== "HEAD") {
    init.body = await req.text();
  }

  try {
    const res = await fetch(url, init);
    const body = await res.text();
    return new NextResponse(body, {
      status: res.status,
      headers: { "content-type": res.headers.get("content-type") ?? "application/json" },
    });
  } catch (e) {
    return NextResponse.json(
      { error: `${product.title} API unreachable on :${product.port} (${e})` },
      { status: 502 },
    );
  }
}

export {
  proxy as GET,
  proxy as POST,
  proxy as PATCH,
  proxy as PUT,
  proxy as DELETE,
};
