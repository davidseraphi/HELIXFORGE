/**
 * Device key registry (Horizon A) — ECDSA P-256; public key registered server-side.
 * Private key never leaves IndexedDB.
 */

const DB = "helix-collab-devices-v1";
const STORE = "keys";

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB, 1);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE)) db.createObjectStore(STORE);
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function idbGet(key: string): Promise<CryptoKeyPair | undefined> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readonly");
    const r = tx.objectStore(STORE).get(key);
    r.onsuccess = () => resolve(r.result as CryptoKeyPair | undefined);
    r.onerror = () => reject(r.error);
  });
}

async function idbSet(key: string, value: CryptoKeyPair): Promise<void> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readwrite");
    tx.objectStore(STORE).put(value, key);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

function b64(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf);
  let s = "";
  for (let i = 0; i < bytes.length; i++) s += String.fromCharCode(bytes[i]!);
  return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

/** Ensure a local device keypair exists; return SPKI public key b64url. */
export async function ensureDeviceKey(
  deviceLabel = "browser",
): Promise<{ public_key_b64: string; algorithm: string; device_label: string }> {
  const id = "default";
  let pair = await idbGet(id);
  if (!pair) {
    pair = await crypto.subtle.generateKey(
      { name: "ECDSA", namedCurve: "P-256" },
      false,
      ["sign", "verify"],
    );
    await idbSet(id, pair);
  }
  const spki = await crypto.subtle.exportKey("spki", pair.publicKey);
  return {
    public_key_b64: b64(spki),
    algorithm: "ECDSA_P256",
    device_label: deviceLabel,
  };
}
