export type SemanticState =
  | "active"
  | "waiting_human"
  | "waiting_external"
  | "completed"
  | "failed"
  | "unknown";

export type ProductMaturity =
  | "scaffold"
  | "prototype"
  | "alpha"
  | "beta"
  | "production";

export type ProductTier = "standard" | "frontier";

export type CatalogEntry = {
  order: number;
  slug: string;
  title: string;
  description: string;
  tier: ProductTier;
  maturity: ProductMaturity;
  semantic_state: SemanticState;
  default_port: number;
  upstream: string;
  gateway_prefix?: string;
};

export type CatalogState = {
  slug: string;
  maturity: ProductMaturity;
  semantic_state: SemanticState;
  upstream_reachable: boolean;
  detail: string;
};

export type ApiResponse<T> = {
  success: boolean;
  data: T;
  error?: { code: string; message: string } | null;
};

export class GatewayClient {
  constructor(private baseUrl: string) {}

  async catalog(): Promise<CatalogEntry[]> {
    const r = await fetch(`${this.baseUrl}/v1/catalog`);
    const body: ApiResponse<CatalogEntry[]> = await r.json();
    if (!r.ok || !body.success) {
      throw new Error(body.error?.message ?? `catalog fetch failed: ${r.status}`);
    }
    return body.data;
  }

  async catalogState(slug: string): Promise<CatalogState> {
    const r = await fetch(`${this.baseUrl}/v1/catalog/${slug}/state`);
    const body: ApiResponse<CatalogState> = await r.json();
    if (!r.ok || !body.success) {
      throw new Error(body.error?.message ?? `state fetch failed: ${r.status}`);
    }
    return body.data;
  }
}
