/**
 * Client-held E2EE for HelixCollab.
 *
 * Envelope: HC1.<iv_b64url>.<ct_b64url>  (AES-256-GCM)
 * DEK is random; wrapped with PBKDF2-derived KEK from passphrase and stored in localStorage.
 * Server never sees the DEK or plaintext.
 */

export const CLIENT_ENVELOPE_PREFIX = "HC1.";

const STORAGE_KEY = "helix.collab.clientKeys.v1";
const PBKDF2_ITERS = 210_000;

export type WrappedKeyRecord = {
  docId: string;
  salt_b64: string;
  iv_b64: string;
  wrapped_dek_b64: string;
  iterations: number;
};

/** In-memory unlocked DEKs for this tab session. */
const sessionKeys = new Map<string, CryptoKey>();

function b64urlEncode(bytes: Uint8Array): string {
  let s = "";
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    s += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function b64urlDecode(s: string): Uint8Array {
  const pad = s.length % 4 === 0 ? "" : "=".repeat(4 - (s.length % 4));
  const b64 = s.replace(/-/g, "+").replace(/_/g, "/") + pad;
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

export function isClientEnvelope(s: string): boolean {
  return s.trimStart().startsWith(CLIENT_ENVELOPE_PREFIX);
}

function loadKeyring(): Record<string, WrappedKeyRecord> {
  if (typeof localStorage === "undefined") return {};
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    return JSON.parse(raw) as Record<string, WrappedKeyRecord>;
  } catch {
    return {};
  }
}

function saveKeyring(ring: Record<string, WrappedKeyRecord>) {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(ring));
}

export function hasWrappedKey(docId: string): boolean {
  return !!loadKeyring()[docId];
}

export function isUnlocked(docId: string): boolean {
  return sessionKeys.has(docId);
}

export function lockSession(docId?: string) {
  if (docId) sessionKeys.delete(docId);
  else sessionKeys.clear();
}

async function deriveKek(
  passphrase: string,
  salt: Uint8Array,
  iterations: number,
): Promise<CryptoKey> {
  const enc = new TextEncoder();
  const base = await crypto.subtle.importKey(
    "raw",
    enc.encode(passphrase),
    "PBKDF2",
    false,
    ["deriveKey"],
  );
  return crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: salt as BufferSource,
      iterations,
      hash: "SHA-256",
    },
    base,
    { name: "AES-GCM", length: 256 },
    false,
    ["wrapKey", "unwrapKey", "encrypt", "decrypt"],
  );
}

async function generateDek(): Promise<CryptoKey> {
  return crypto.subtle.generateKey({ name: "AES-GCM", length: 256 }, true, [
    "encrypt",
    "decrypt",
  ]);
}

async function wrapDek(
  dek: CryptoKey,
  passphrase: string,
): Promise<Omit<WrappedKeyRecord, "docId">> {
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const kek = await deriveKek(passphrase, salt, PBKDF2_ITERS);
  const wrapped = new Uint8Array(
    await crypto.subtle.wrapKey("raw", dek, kek, {
      name: "AES-GCM",
      iv,
    }),
  );
  return {
    salt_b64: b64urlEncode(salt),
    iv_b64: b64urlEncode(iv),
    wrapped_dek_b64: b64urlEncode(wrapped),
    iterations: PBKDF2_ITERS,
  };
}

async function unwrapDek(
  record: WrappedKeyRecord,
  passphrase: string,
): Promise<CryptoKey> {
  const salt = b64urlDecode(record.salt_b64);
  const iv = b64urlDecode(record.iv_b64);
  const wrapped = b64urlDecode(record.wrapped_dek_b64);
  const kek = await deriveKek(passphrase, salt, record.iterations);
  return crypto.subtle.unwrapKey(
    "raw",
    wrapped as BufferSource,
    kek,
    { name: "AES-GCM", iv: iv as BufferSource },
    { name: "AES-GCM", length: 256 },
    true,
    ["encrypt", "decrypt"],
  );
}

/** Create a new DEK for a doc, wrap with passphrase, unlock session. */
export async function setupClientKey(
  docId: string,
  passphrase: string,
): Promise<void> {
  if (!passphrase || passphrase.length < 8) {
    throw new Error("Passphrase must be at least 8 characters");
  }
  const dek = await generateDek();
  const wrapped = await wrapDek(dek, passphrase);
  const ring = loadKeyring();
  ring[docId] = { docId, ...wrapped };
  saveKeyring(ring);
  sessionKeys.set(docId, dek);
}

/** Unlock an existing wrapped DEK into session memory. */
export async function unlockClientKey(
  docId: string,
  passphrase: string,
): Promise<void> {
  const rec = loadKeyring()[docId];
  if (!rec) throw new Error("No local key for this document — import or re-setup");
  try {
    const dek = await unwrapDek(rec, passphrase);
    sessionKeys.set(docId, dek);
  } catch {
    throw new Error("Wrong passphrase or corrupt key");
  }
}

/** Import a raw DEK (base64url) and wrap with passphrase. */
export async function importRawDek(
  docId: string,
  rawDekB64: string,
  passphrase: string,
): Promise<void> {
  const raw = b64urlDecode(rawDekB64.trim());
  const dek = await crypto.subtle.importKey(
    "raw",
    raw as BufferSource,
    { name: "AES-GCM", length: 256 },
    true,
    ["encrypt", "decrypt"],
  );
  const wrapped = await wrapDek(dek, passphrase);
  const ring = loadKeyring();
  ring[docId] = { docId, ...wrapped };
  saveKeyring(ring);
  sessionKeys.set(docId, dek);
}

/** Export raw DEK for sharing with a collaborator (treat as secret). */
export async function exportRawDek(docId: string): Promise<string> {
  const dek = sessionKeys.get(docId);
  if (!dek) throw new Error("Document key not unlocked");
  const raw = new Uint8Array(await crypto.subtle.exportKey("raw", dek));
  return b64urlEncode(raw);
}

/** Encrypt raw bytes (Yjs updates) to HC1 envelope. */
export async function encryptBytes(
  docId: string,
  bytes: Uint8Array,
): Promise<string> {
  const dek = sessionKeys.get(docId);
  if (!dek) throw new Error("Document key not unlocked");
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const ct = new Uint8Array(
    await crypto.subtle.encrypt(
      { name: "AES-GCM", iv },
      dek,
      bytes as BufferSource,
    ),
  );
  return `${CLIENT_ENVELOPE_PREFIX}${b64urlEncode(iv)}.${b64urlEncode(ct)}`;
}

/** Decrypt HC1 envelope to raw bytes. */
export async function decryptBytes(
  docId: string,
  envelope: string,
): Promise<Uint8Array> {
  if (!isClientEnvelope(envelope)) {
    throw new Error("Not an HC1 envelope");
  }
  const dek = sessionKeys.get(docId);
  if (!dek) throw new Error("Document key not unlocked");
  const body = envelope.trim().slice(CLIENT_ENVELOPE_PREFIX.length);
  const [ivB64, ctB64] = body.split(".");
  if (!ivB64 || !ctB64) throw new Error("Invalid HC1 envelope");
  const iv = b64urlDecode(ivB64);
  const ct = b64urlDecode(ctB64);
  const plain = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv: iv as BufferSource },
    dek,
    ct as BufferSource,
  );
  return new Uint8Array(plain);
}

export async function encryptPlaintext(
  docId: string,
  plaintext: string,
): Promise<string> {
  const enc = new TextEncoder();
  return encryptBytes(docId, enc.encode(plaintext));
}

export async function decryptEnvelope(
  docId: string,
  envelope: string,
): Promise<string> {
  if (!isClientEnvelope(envelope)) return envelope;
  const bytes = await decryptBytes(docId, envelope);
  return new TextDecoder().decode(bytes);
}

/** Provisional encrypt for create-before-id: returns { envelope, dek } then bind. */
export async function encryptWithNewKey(
  passphrase: string,
  plaintext: string,
): Promise<{ envelope: string; bind: (docId: string) => Promise<void> }> {
  if (!passphrase || passphrase.length < 8) {
    throw new Error("Passphrase must be at least 8 characters");
  }
  const dek = await generateDek();
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const enc = new TextEncoder();
  const ct = new Uint8Array(
    await crypto.subtle.encrypt(
      { name: "AES-GCM", iv },
      dek,
      enc.encode(plaintext),
    ),
  );
  const envelope = `${CLIENT_ENVELOPE_PREFIX}${b64urlEncode(iv)}.${b64urlEncode(ct)}`;
  const wrapped = await wrapDek(dek, passphrase);
  return {
    envelope,
    bind: async (docId: string) => {
      const ring = loadKeyring();
      ring[docId] = { docId, ...wrapped };
      saveKeyring(ring);
      sessionKeys.set(docId, dek);
    },
  };
}
