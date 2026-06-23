const path = require("node:path");
const { app, BrowserWindow, shell } = require("electron");

let mainWindow = null;
let promptServer = null;

async function closePromptServer() {
  if (!promptServer) {
    return;
  }

  await new Promise((resolve) => {
    promptServer.close(() => resolve());
  });
  promptServer = null;
}

async function createMainWindow() {
  // 桌面版必须把缓存写到用户数据目录，避免打包后的应用目录只读。
  process.env.PROMPT_LIBRARY_CACHE_DIR = path.join(app.getPath("userData"), "github-markdown-cache");

  const { startServer } = require("../server");
  promptServer = await startServer(0, { log: false });

  const isMac = process.platform === "darwin";

  mainWindow = new BrowserWindow({
    width: 1720,
    height: 1120,
    minWidth: 1100,
    minHeight: 760,
    show: false,
    backgroundColor: "#f6f5f1",
    title: "Prompt Library Viewer",
    titleBarStyle: "hidden",
    titleBarOverlay: isMac ? false : {
      color: "#f6f5f1",
      symbolColor: "#202124",
      height: 38
    },
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });

  mainWindow.maximize();
  mainWindow.once("ready-to-show", () => {
    mainWindow.show();
  });

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: "deny" };
  });

  await mainWindow.loadURL(promptServer.localUrl);
}

app.whenReady().then(createMainWindow).catch((error) => {
  console.error(error);
  app.quit();
});

app.on("activate", () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createMainWindow().catch((error) => {
      console.error(error);
      app.quit();
    });
  }
});

app.on("before-quit", () => {
  if (mainWindow) {
    mainWindow.removeAllListeners("close");
  }
});

app.on("window-all-closed", () => {
  closePromptServer().finally(() => {
    if (process.platform !== "darwin") {
      app.quit();
    }
  });
});
