/**
 * Helix passkey v2 — signs WebAuthn-shaped clientDataJSON bound to user/rp/origin.
 */

import { API, DEV_USER } from "./config";

function headers(): HeadersInit {
  return {
    "Content-Type": "application/json",
    "x-helix-dev-user": DEV_USER,
  };
}

function b64url(buf: ArrayBuffer | Uint8Array): string {
  const bytes = buf instanceof Uint8Array ? buf : new Uint8Array(buf);
  let s = "";
  for (let i = 0; i < bytes.length; i++) s += String.fromCharCode(bytes[i]!);
  return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function b64urlToBytes(s: string): Uint8Array {
  const pad = s.length % 4 === 0 ? "" : "=".repeat(4 - (s.length % 4));
  const b64 = s.replace(/-/g, "+").replace(/_/g, "/") + pad;
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

const DB = "helix-passkey-v2";
const STORE = "keys";
const COUNTER_KEY = "counter";

async function idbPut(key: string, value: CryptoKeyPair | number) {
  return new Promise<void>((resolve, reject) => {
    const req = indexedDB.open(DB, 1);
    req.onupgradeneeded = () => {
      if (!req.result.objectStoreNames.contains(STORE))
        req.result.createObjectStore(STORE);
    };
    req.onsuccess = () => {
      const tx = req.result.transaction(STORE, "readwrite");
      tx.objectStore(STORE).put(value, key);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    };
    req.onerror = () => reject(req.error);
  });
}

async function idbGet<T>(key: string): Promise<T | undefined> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB, 1);
    req.onupgradeneeded = () => {
      if (!req.result.objectStoreNames.contains(STORE))
        req.result.createObjectStore(STORE);
    };
    req.onsuccess = () => {
      const tx = req.result.transaction(STORE, "readonly");
      const r = tx.objectStore(STORE).get(key);
      r.onsuccess = () => resolve(r.result as T | undefined);
      r.onerror = () => reject(r.error);
    };
    req.onerror = () => reject(req.error);
  });
}

export async function registerPasskey(deviceLabel = "browser"): Promise<string> {
  const start = await fetch(`${API}/v1/webauthn/register/start`, {
    method: "POST",
    headers: headers(),
  }).then((r) => r.json());
  if (!start.data?.challenge_b64) throw new Error(start.error?.message ?? "start failed");
  const clientData = b64urlToBytes(start.data.client_data_b64);
  const pair = await crypto.subtle.generateKey(
    { name: "ECDSA", namedCurve: "P-256" },
    true,
    ["sign", "verify"],
  );
  await idbPut("default", pair);
  await idbPut(COUNTER_KEY, 0);
  const spki = await crypto.subtle.exportKey("spki", pair.publicKey);
  // ECDSA+SHA-256 over clientDataJSON bytes (Web Crypto hashes once)
  const sig = await crypto.subtle.sign(
    { name: "ECDSA", hash: "SHA-256" },
    pair.privateKey,
    clientData as BufferSource,
  );
  const finish = await fetch(`${API}/v1/webauthn/register/finish`, {
    method: "POST",
    headers: headers(),
    body: JSON.stringify({
      public_key_spki_b64: b64url(spki),
      signature_b64: b64url(sig),
      client_data_b64: start.data.client_data_b64,
      device_label: deviceLabel,
    }),
  }).then((r) => r.json());
  if (!finish.data?.registered)
    throw new Error(finish.error?.message ?? "register failed");
  return finish.data.device_key_id as string;
}

export async function authenticatePasskey(): Promise<boolean> {
  const pair = await idbGet<CryptoKeyPair>("default");
  if (!pair) throw new Error("no local passkey");
  const start = await fetch(`${API}/v1/webauthn/authenticate/start`, {
    method: "POST",
    headers: headers(),
  }).then((r) => r.json());
  if (!start.data?.client_data_b64)
    throw new Error(start.error?.message ?? "auth start failed");
  const clientData = b64urlToBytes(start.data.client_data_b64);
  const spki = await crypto.subtle.exportKey("spki", pair.publicKey);
  const sig = await crypto.subtle.sign(
    { name: "ECDSA", hash: "SHA-256" },
    pair.privateKey,
    clientData as BufferSource,
  );
  const prev = (await idbGet<number>(COUNTER_KEY)) ?? 0;
  const counter = prev + 1;
  const finish = await fetch(`${API}/v1/webauthn/authenticate/finish`, {
    method: "POST",
    headers: headers(),
    body: JSON.stringify({
      public_key_spki_b64: b64url(spki),
      signature_b64: b64url(sig),
      client_data_b64: start.data.client_data_b64,
      counter,
    }),
  }).then((r) => r.json());
  if (finish.data?.authenticated) {
    await idbPut(COUNTER_KEY, counter);
    return true;
  }
  throw new Error(finish.error?.message ?? "auth failed");
}
