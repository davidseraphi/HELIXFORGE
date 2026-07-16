const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("helixDesktop", {
  isElectron: true,
  platform: process.platform,
  versions: {
    electron: process.versions.electron,
    chrome: process.versions.chrome,
    node: process.versions.node,
  },
  getApiBase: () => ipcRenderer.invoke("helix:get-api-base"),
  getWebUrl: () => ipcRenderer.invoke("helix:get-web-url"),
  getMeta: () => ipcRenderer.invoke("helix:get-meta"),
  onMenu: (handler) => {
    const listener = (_event, action) => handler(action);
    ipcRenderer.on("helix-menu", listener);
    return () => ipcRenderer.removeListener("helix-menu", listener);
  },
});
