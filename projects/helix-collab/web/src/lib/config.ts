export const API =
  process.env.NEXT_PUBLIC_COLLAB_API ?? "http://127.0.0.1:8101";

export const DEV_USER =
  process.env.NEXT_PUBLIC_DEV_USER ?? "ops@helixforge.local";

export const WS_BASE = API.replace(/^http/, "ws");
