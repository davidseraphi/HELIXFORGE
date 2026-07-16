/**
 * HelixCollab WS provider for Yjs.
 *
 * Plain mode: `crdt_sync` / `crdt_update` (server may apply yrs).
 * Sealed mode (client e2ee): `crdt_sealed_sync` / `crdt_sealed_update` —
 * HC1 envelopes of Yjs bytes; server is blind relay only.
 */
import * as Y from "yjs";
import { DEV_USER, WS_BASE } from "./config";
import { decryptBytes, encryptBytes, isUnlocked } from "./client-crypto";

export type ProviderStatus = "connecting" | "open" | "closed" | "error";

export type ProviderHandlers = {
  onStatus?: (s: ProviderStatus) => void;
  onPresence?: (p: {
    user_id: string;
    display_name: string;
    cursor_pos: number;
  }) => void;
  onPeerLeft?: (user_id: string) => void;
  onSnapshot?: (s: {
    version: number;
    content: string;
    title: string;
  }) => void;
  onAck?: (version: number) => void;
  onError?: (message: string) => void;
  onCrdtMode?: (enabled: boolean) => void;
  /** True when using sealed (client-e2ee) CRDT path. */
  onSealedMode?: (enabled: boolean) => void;
  onTyping?: (p: {
    user_id: string;
    display_name: string;
    active: boolean;
  }) => void;
  onCommentEvent?: (e: {
    action: string;
    comment_id: string;
    anchor_start?: number | null;
    anchor_end?: number | null;
  }) => void;
};

export type ProviderOptions = {
  /** Client-held E2EE: encrypt all CRDT payloads with session DEK. */
  sealed?: boolean;
};

function b64ToUint8(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

function uint8ToB64(bytes: Uint8Array): string {
  let s = "";
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    s += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(s);
}

export class HelixYjsProvider {
  readonly ydoc: Y.Doc;
  readonly ytext: Y.Text;
  private ws: WebSocket | null = null;
  private docId: string;
  private handlers: ProviderHandlers;
  private sealed: boolean;
  private applyingRemote = false;
  private destroyed = false;
  private onUpdate: ((update: Uint8Array, origin: unknown) => void) | null =
    null;
  private sealedStatePublished = false;

  constructor(
    docId: string,
    handlers: ProviderHandlers = {},
    options: ProviderOptions = {},
  ) {
    this.docId = docId;
    this.handlers = handlers;
    this.sealed = !!options.sealed;
    this.ydoc = new Y.Doc();
    this.ytext = this.ydoc.getText("content");
  }

  get isSealed() {
    return this.sealed;
  }

  connect() {
    if (this.destroyed) return;
    if (this.sealed && !isUnlocked(this.docId)) {
      this.handlers.onError?.("sealed CRDT requires unlocked client key");
      this.handlers.onStatus?.("error");
      return;
    }
    const qs = new URLSearchParams({ dev_user: DEV_USER });
    const url = `${WS_BASE}/v1/ws/documents/${this.docId}?${qs}`;
    this.handlers.onStatus?.("connecting");
    const ws = new WebSocket(url);
    this.ws = ws;

    ws.onopen = () => {
      this.handlers.onStatus?.("open");
      ws.send(
        JSON.stringify({
          type: "join",
          user_id: DEV_USER,
          display_name: DEV_USER,
        }),
      );
      if (this.sealed) {
        this.handlers.onSealedMode?.(true);
        this.handlers.onCrdtMode?.(true);
        ws.send(JSON.stringify({ type: "crdt_sealed_sync", sealed: "" }));
      } else {
        ws.send(JSON.stringify({ type: "crdt_sync", state_b64: "" }));
      }
    };

    ws.onerror = () => this.handlers.onStatus?.("error");
    ws.onclose = () => this.handlers.onStatus?.("closed");

    ws.onmessage = (ev) => {
      try {
        const msg = JSON.parse(String(ev.data));
        void this.handleMessage(msg);
      } catch {
        /* ignore */
      }
    };

    this.onUpdate = (update, origin) => {
      if (this.applyingRemote || origin === "remote") return;
      if (this.sealed) {
        void this.sendSealedUpdate(update);
      } else {
        this.send({
          type: "crdt_update",
          update_b64: uint8ToB64(update),
          author: DEV_USER,
        });
      }
    };
    this.ydoc.on("update", this.onUpdate);
  }

  private async sendSealedUpdate(update: Uint8Array) {
    try {
      const sealed = await encryptBytes(this.docId, update);
      this.send({
        type: "crdt_sealed_update",
        sealed,
        author: DEV_USER,
      });
      // Periodically publish full sealed state for late joiners.
      if (!this.sealedStatePublished) {
        this.sealedStatePublished = true;
        void this.publishSealedFullState();
        // re-publish full state every 30 local updates via counter would be nicer;
        // also publish shortly after first edit and on interval below.
      }
    } catch (e) {
      this.handlers.onError?.(String(e));
    }
  }

  /** Publish full Yjs state as sealed envelope (late-joiner bootstrap). */
  async publishSealedFullState() {
    if (!this.sealed || !isUnlocked(this.docId)) return;
    try {
      const full = Y.encodeStateAsUpdate(this.ydoc);
      const sealed = await encryptBytes(this.docId, full);
      this.send({ type: "crdt_sealed_sync", sealed });
    } catch (e) {
      this.handlers.onError?.(String(e));
    }
  }

  private async handleMessage(msg: Record<string, unknown>) {
    const type = msg.type as string;
    switch (type) {
      case "crdt_sync": {
        if (this.sealed) break; // ignore plaintext CRDT in sealed mode
        this.handlers.onCrdtMode?.(true);
        const state = msg.state_b64 as string;
        if (state) {
          this.applyingRemote = true;
          try {
            Y.applyUpdate(this.ydoc, b64ToUint8(state), "remote");
          } finally {
            this.applyingRemote = false;
          }
        }
        break;
      }
      case "crdt_update": {
        if (this.sealed) break;
        this.handlers.onCrdtMode?.(true);
        const u = msg.update_b64 as string;
        if (u) {
          this.applyingRemote = true;
          try {
            Y.applyUpdate(this.ydoc, b64ToUint8(u), "remote");
          } finally {
            this.applyingRemote = false;
          }
        }
        break;
      }
      case "crdt_sealed_sync": {
        this.handlers.onSealedMode?.(true);
        this.handlers.onCrdtMode?.(true);
        const sealed = String(msg.sealed ?? "");
        if (!sealed) break;
        try {
          const bytes = await decryptBytes(this.docId, sealed);
          this.applyingRemote = true;
          try {
            Y.applyUpdate(this.ydoc, bytes, "remote");
          } finally {
            this.applyingRemote = false;
          }
        } catch (e) {
          this.handlers.onError?.(`sealed sync decrypt: ${e}`);
        }
        break;
      }
      case "crdt_sealed_update": {
        this.handlers.onSealedMode?.(true);
        this.handlers.onCrdtMode?.(true);
        const sealed = String(msg.sealed ?? "");
        if (!sealed) break;
        // Skip echo of our own if needed — peers still fine
        try {
          const bytes = await decryptBytes(this.docId, sealed);
          this.applyingRemote = true;
          try {
            Y.applyUpdate(this.ydoc, bytes, "remote");
          } finally {
            this.applyingRemote = false;
          }
        } catch (e) {
          this.handlers.onError?.(`sealed update decrypt: ${e}`);
        }
        break;
      }
      case "snapshot":
        this.handlers.onSnapshot?.({
          version: msg.version as number,
          content: msg.content as string,
          title: msg.title as string,
        });
        // Seed Y.Text only in plaintext mode when empty.
        if (
          !this.sealed &&
          this.ytext.length === 0 &&
          typeof msg.content === "string" &&
          !String(msg.content).startsWith("HC1.")
        ) {
          this.applyingRemote = true;
          try {
            this.ydoc.transact(() => {
              this.ytext.insert(0, msg.content as string);
            }, "remote");
          } finally {
            this.applyingRemote = false;
          }
        }
        break;
      case "presence":
        this.handlers.onPresence?.({
          user_id: String(msg.user_id),
          display_name: String(msg.display_name ?? ""),
          cursor_pos: Number(msg.cursor_pos ?? 0),
        });
        break;
      case "peer_left":
        this.handlers.onPeerLeft?.(String(msg.user_id));
        break;
      case "ack":
        this.handlers.onAck?.(Number(msg.version));
        break;
      case "error":
        this.handlers.onError?.(String(msg.message ?? "ws error"));
        break;
      case "typing":
        this.handlers.onTyping?.({
          user_id: String(msg.user_id),
          display_name: String(msg.display_name ?? ""),
          active: Boolean(msg.active),
        });
        break;
      case "comment_event":
        this.handlers.onCommentEvent?.({
          action: String(msg.action),
          comment_id: String(msg.comment_id),
          anchor_start: msg.anchor_start as number | null | undefined,
          anchor_end: msg.anchor_end as number | null | undefined,
        });
        break;
      default:
        break;
    }
  }

  sendPresence(cursorPos: number) {
    this.send({
      type: "presence",
      user_id: DEV_USER,
      display_name: DEV_USER,
      cursor_pos: cursorPos,
    });
  }

  sendTyping(active: boolean) {
    this.send({
      type: "typing",
      user_id: DEV_USER,
      display_name: DEV_USER,
      active,
    });
  }

  sendDurablePatch(baseVersion: number, content: string) {
    this.send({
      type: "patch",
      base_version: baseVersion,
      content,
      author: DEV_USER,
    });
  }

  private send(obj: unknown) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(obj));
    }
  }

  destroy() {
    this.destroyed = true;
    if (this.onUpdate) this.ydoc.off("update", this.onUpdate);
    this.ws?.close();
    this.ws = null;
    this.ydoc.destroy();
  }
}
