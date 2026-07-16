/**
 * Offline merge UI helpers — compare local IndexedDB cache vs server version.
 */
import { offlineGet, offlineList, offlinePut, type OfflineEntry } from "./offline-store";
import { api, type Doc } from "./api";

export type MergePlan = {
  docId: string;
  local?: OfflineEntry;
  serverVersion?: number;
  serverContent?: string;
  action: "push_local" | "pull_server" | "conflict" | "in_sync" | "local_only";
};

export async function planMerge(docId: string): Promise<MergePlan> {
  const local = await offlineGet(docId);
  try {
    const server = await api<Doc>(`/v1/documents/${docId}`);
    if (!local) {
      return {
        docId,
        serverVersion: server.version,
        serverContent: server.content,
        action: "pull_server",
      };
    }
    if (local.version === server.version && local.content === server.content) {
      return { docId, local, serverVersion: server.version, action: "in_sync" };
    }
    if (local.version > server.version) {
      return {
        docId,
        local,
        serverVersion: server.version,
        serverContent: server.content,
        action: "push_local",
      };
    }
    if (local.version < server.version) {
      return {
        docId,
        local,
        serverVersion: server.version,
        serverContent: server.content,
        action: "pull_server",
      };
    }
    return {
      docId,
      local,
      serverVersion: server.version,
      serverContent: server.content,
      action: "conflict",
    };
  } catch {
    return { docId, local, action: "local_only" };
  }
}

export async function applyPull(plan: MergePlan): Promise<void> {
  if (!plan.serverContent || plan.serverVersion == null) return;
  await offlinePut({
    docId: plan.docId,
    title: plan.local?.title ?? "document",
    content: plan.serverContent,
    client_e2ee: plan.local?.client_e2ee ?? false,
    version: plan.serverVersion,
    updated_at: Date.now(),
  });
}

export async function applyPush(
  plan: MergePlan,
  encrypt?: (plain: string) => Promise<string>,
): Promise<Doc> {
  if (!plan.local) throw new Error("no local");
  let content = plan.local.content;
  if (plan.local.client_e2ee && encrypt && !content.startsWith("HC1.")) {
    content = await encrypt(content);
  }
  return api<Doc>(`/v1/documents/${plan.docId}`, {
    method: "PATCH",
    body: JSON.stringify({
      base_version: plan.serverVersion ?? plan.local.version,
      content,
      title: plan.local.title,
    }),
  });
}

export async function listOfflineDocs(): Promise<OfflineEntry[]> {
  return offlineList();
}
