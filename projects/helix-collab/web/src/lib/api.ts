import { API, DEV_USER } from "./config";

export type Doc = {
  id: string;
  title: string;
  content: string;
  version: number;
  workspace_id?: string | null;
  folder_id?: string | null;
  encrypted?: boolean;
  /** Client-held keys; content is HC1 envelope when true. */
  client_e2ee?: boolean;
  pinned?: boolean;
  archived_at?: string | null;
  updated_at?: string;
  created_at?: string;
};

export type Peer = {
  user_id: string;
  display_name: string;
  cursor_pos: number;
  last_seen?: string;
};

export type Revision = {
  id: string;
  document_id: string;
  version: number;
  content: string;
  author_id?: string | null;
  created_at: string;
};

export type AclEntry = {
  principal_id: string;
  principal_kind: string;
  permissions: string[];
};

export type Workspace = {
  id: string;
  name: string;
  product?: string;
  product_slug?: string;
  created_at?: string;
};

export type Folder = {
  id: string;
  workspace_id: string;
  parent_id?: string | null;
  name: string;
};

export type Comment = {
  id: string;
  document_id: string;
  parent_id?: string | null;
  author_id: string;
  author_label: string;
  body: string;
  anchor_start?: number | null;
  anchor_end?: number | null;
  anchor_quote?: string;
  resolved_at?: string | null;
  created_at: string;
  mentions: Mention[];
};

export type Activity = {
  id: string;
  document_id: string;
  actor_label: string;
  action: string;
  detail: Record<string, unknown>;
  created_at: string;
};

export type Mention = {
  id: string;
  comment_id: string;
  document_id: string;
  mentioned_label: string;
  mentioned_user_id?: string | null;
  created_at: string;
};

export type DomainStatus = {
  durable: boolean;
  features: Record<string, boolean | string>;
  realtime?: { ws?: string; fanout?: string };
};

export type Attachment = {
  id: string;
  document_id: string;
  filename: string;
  content_type: string;
  size_bytes: number;
  object_key: string;
  client_sealed: boolean;
  sha256_hex: string;
  created_at?: string;
};

export function headers(): HeadersInit {
  return {
    "Content-Type": "application/json",
    "x-helix-dev-user": DEV_USER,
  };
}

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API}${path}`, {
    ...init,
    headers: { ...headers(), ...(init?.headers ?? {}) },
  });
  const body = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(body?.error?.message ?? `HTTP ${res.status}`);
  }
  return body.data as T;
}
