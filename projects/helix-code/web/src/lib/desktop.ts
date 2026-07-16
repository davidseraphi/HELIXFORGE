/** HelixCode Electron preload bridge (optional). */

export type HelixDesktopBridge = {
  isElectron: true;
  platform: string;
  versions: { electron?: string; chrome?: string; node?: string };
  getApiBase: () => Promise<string>;
  onMenu?: (handler: (action: string) => void) => () => void;
};

declare global {
  interface Window {
    helixDesktop?: HelixDesktopBridge;
  }
}

export function isElectronShell(): boolean {
  if (typeof window === "undefined") return false;
  return !!window.helixDesktop?.isElectron;
}

export function desktopPlatform(): string | null {
  if (typeof window === "undefined") return null;
  return window.helixDesktop?.platform ?? null;
}
