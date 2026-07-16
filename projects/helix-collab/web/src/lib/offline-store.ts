/**
 * Offline-first Y.Doc / markdown cache (Horizon B).
 * Sealed docs store HC1 only; never plaintext at rest without unlock.
 */

const DB = "helix-collab-offline-v1";
const STORE = "docs";

export type OfflineEntry = {
  docId: string;
  title: string;
  content: string; // may be HC1
  client_e2ee: boolean;
  version: number;
  updated_at: number;
  yjs_update_b64?: string;
};

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB, 1);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE)) {
        db.createObjectStore(STORE, { keyPath: "docId" });
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

export async function offlinePut(entry: OfflineEntry): Promise<void> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readwrite");
    tx.objectStore(STORE).put(entry);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

export async function offlineGet(
  docId: string,
): Promise<OfflineEntry | undefined> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readonly");
    const r = tx.objectStore(STORE).get(docId);
    r.onsuccess = () => resolve(r.result as OfflineEntry | undefined);
    r.onerror = () => reject(r.error);
  });
}

export async function offlineList(): Promise<OfflineEntry[]> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readonly");
    const r = tx.objectStore(STORE).getAll();
    r.onsuccess = () => resolve((r.result as OfflineEntry[]) ?? []);
    r.onerror = () => reject(r.error);
  });
}
