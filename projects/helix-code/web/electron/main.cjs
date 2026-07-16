/**
 * HelixCode Electron shell — loads the Code-OSS web UI (Next :3102 by default).
 * Does not embed a forked VS Code; sovereign forge desktop chrome.
 */
const { app, BrowserWindow, Menu, shell, ipcMain } = require("electron");
const path = require("path");

const WEB_URL =
  process.env.HELIX_CODE_WEB_URL ||
  process.env.ELECTRON_START_URL ||
  "http://127.0.0.1:3102";
const API_URL =
  process.env.HELIX_CODE_API_URL ||
  process.env.NEXT_PUBLIC_HELIX_CODE_API ||
  "http://127.0.0.1:8102";

/** @type {BrowserWindow | null} */
let mainWindow = null;

function createWindow() {
  const iconPath = path.join(__dirname, "..", "build", "icon.ico");
  const winOpts = {
    width: 1440,
    height: 900,
    minWidth: 960,
    minHeight: 640,
    title: "HelixCode",
    backgroundColor: "#1e1e1e",
    webPreferences: {
      preload: path.join(__dirname, "preload.cjs"),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  };
  try {
    if (require("fs").existsSync(iconPath)) {
      winOpts.icon = iconPath;
    }
  } catch (_) {
    /* optional icon */
  }
  mainWindow = new BrowserWindow(winOpts);

  mainWindow.loadURL(WEB_URL).catch((err) => {
    console.error("Failed to load web UI:", err.message);
    mainWindow?.loadURL(
      `data:text/html,<h2 style="font-family:system-ui;color:#ccc;background:#1e1e1e;padding:2rem">HelixCode</h2><p style="font-family:system-ui;color:#888;padding:0 2rem">Could not load <code>${WEB_URL}</code>. Start the web app:<br/><code>pnpm --filter @helixforge/helix-code-web dev</code></p>`,
    );
  });

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: "deny" };
  });

  mainWindow.on("closed", () => {
    mainWindow = null;
  });

  buildMenu();
}

function sendMenu(action) {
  mainWindow?.webContents.send("helix-menu", action);
}

function buildMenu() {
  const isMac = process.platform === "darwin";
  const template = [
    ...(isMac
      ? [
          {
            label: app.name,
            submenu: [
              { role: "about" },
              { type: "separator" },
              { role: "quit" },
            ],
          },
        ]
      : []),
    {
      label: "File",
      submenu: [
        {
          label: "Command Palette",
          accelerator: "CmdOrCtrl+Shift+P",
          click: () => sendMenu("palette"),
        },
        {
          label: "Quick Open",
          accelerator: "CmdOrCtrl+P",
          click: () => sendMenu("quickOpen"),
        },
        { type: "separator" },
        isMac ? { role: "close" } : { role: "quit" },
      ],
    },
    {
      label: "View",
      submenu: [
        {
          label: "Split Editor Right",
          accelerator: "CmdOrCtrl+\\",
          click: () => sendMenu("split"),
        },
        {
          label: "Close Split",
          accelerator: "CmdOrCtrl+Shift+\\",
          click: () => sendMenu("unsplit"),
        },
        {
          label: "Focus Other Group",
          accelerator: "CmdOrCtrl+1",
          click: () => sendMenu("focusPrimary"),
        },
        {
          label: "Focus Secondary Group",
          accelerator: "CmdOrCtrl+2",
          click: () => sendMenu("focusSecondary"),
        },
        { type: "separator" },
        { role: "reload" },
        { role: "toggleDevTools" },
        { type: "separator" },
        { role: "togglefullscreen" },
      ],
    },
    {
      label: "Help",
      submenu: [
        {
          label: "Open API healthz",
          click: () => shell.openExternal(`${API_URL}/healthz`),
        },
        {
          label: "About HelixCode",
          click: () => sendMenu("about"),
        },
      ],
    },
  ];
  Menu.setApplicationMenu(Menu.buildFromTemplate(template));
}

ipcMain.handle("helix:get-api-base", () => API_URL);
ipcMain.handle("helix:get-web-url", () => WEB_URL);
ipcMain.handle("helix:get-meta", () => ({
  isElectron: true,
  platform: process.platform,
  versions: {
    electron: process.versions.electron,
    chrome: process.versions.chrome,
    node: process.versions.node,
  },
}));

function setupAutoUpdate() {
  const feed = process.env.HELIX_CODE_UPDATE_URL;
  if (!feed) {
    return;
  }
  try {
    // eslint-disable-next-line global-require
    const { autoUpdater } = require("electron-updater");
    autoUpdater.autoDownload = process.env.HELIX_CODE_UPDATE_AUTO_DOWNLOAD === "1";
    autoUpdater.setFeedURL({ provider: "generic", url: feed.replace(/\/$/, "") });
    autoUpdater.on("error", (err) => console.warn("autoUpdater error:", err?.message || err));
    autoUpdater.on("update-available", (info) =>
      console.log("update available:", info?.version || info),
    );
    autoUpdater.on("update-downloaded", () => {
      console.log("update downloaded; will install on quit");
    });
    autoUpdater.checkForUpdatesAndNotify().catch((e) => {
      console.warn("auto-update check failed:", e.message);
    });
  } catch (e) {
    console.warn("electron-updater not available:", e.message);
  }
}

app.whenReady().then(() => {
  createWindow();
  setupAutoUpdate();
  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});
