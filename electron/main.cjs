const { app, BrowserWindow, Menu, protocol, shell } = require('electron');
const path = require('path');
const fs = require('fs');

// This allows TypeScript to detect the missing method
protocol.registerSchemesAsPrivileged([
  {
    scheme: 'app',
    privileges: {
      secure: true,
      standard: true,
      supportFetchAPI: true,
      corsEnabled: true,
      bypassCSP: true,
    },
  },
]);

let mainWindow;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      webSecurity: true,
      // Enable features needed for workers and SharedArrayBuffer
      webgl: true,
      experimentalFeatures: true,
    },
    icon: path.join(__dirname, '../dist/favicon.ico'),
  });

  // Create menu for Mac
  const template = [
    {
      label: 'Edit',
      submenu: [
        { role: 'undo' },
        { role: 'redo' },
        { type: 'separator' },
        { role: 'cut' },
        { role: 'copy' },
        { role: 'paste' },
        { role: 'selectall' },
      ],
    },
    {
      label: 'View',
      submenu: [
        { role: 'reload' },
        { role: 'forcereload' },
        { role: 'toggledevtools' },
        { type: 'separator' },
        { role: 'resetzoom' },
        { role: 'zoomin' },
        { role: 'zoomout' },
        { type: 'separator' },
        { role: 'togglefullscreen' },
      ],
    },
    {
      label: 'Window',
      submenu: [{ role: 'minimize' }, { role: 'close' }],
    },
  ];

  if (process.platform === 'darwin') {
    template.unshift({
      label: app.getName(),
      submenu: [
        { role: 'about' },
        { type: 'separator' },
        { role: 'services', submenu: [] },
        { type: 'separator' },
        { role: 'hide' },
        { role: 'hideothers' },
        { role: 'unhide' },
        { type: 'separator' },
        { role: 'quit' },
      ],
    });

    // Window menu
    template[3].submenu = [
      { role: 'close' },
      { role: 'minimize' },
      { role: 'zoom' },
      { type: 'separator' },
      { role: 'front' },
    ];
  }

  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);

  // Load the app
  if (process.env.NODE_ENV === 'development') {
    mainWindow.loadURL('http://localhost:5173');
  } else {
    // Use custom protocol with a base URL structure
    mainWindow.loadURL('app://localhost/');
  }

  // Handle external links - open them in the default browser
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    // If it's an external URL, open it in the default browser
    if (url.startsWith('http://') || url.startsWith('https://')) {
      shell.openExternal(url);
      return { action: 'deny' }; // Prevent opening in Electron
    }
    return { action: 'allow' };
  });

  // Also handle navigation to external links
  mainWindow.webContents.on('will-navigate', (event, url) => {
    // If navigating to an external URL, prevent it and open in browser
    if (!url.startsWith('app://') && !url.startsWith('file://')) {
      if (url.startsWith('http://') || url.startsWith('https://')) {
        event.preventDefault();
        shell.openExternal(url);
      }
    }
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

app.whenReady().then(() => {
  // Register protocol for serving local files with proper headers
  protocol.registerFileProtocol('app', (request, callback) => {
    let url = request.url.replace('app://localhost/', '');

    // Remove any query parameters
    url = url.split('?')[0];

    // Default to index.html if no file specified or just a slash
    if (!url || url === '' || url === '/') {
      url = 'index.html';
    }

    // Remove leading slash if present
    if (url.startsWith('/')) {
      url = url.substring(1);
    }

    const filePath = path.join(__dirname, '../dist', url);

    callback({
      path: filePath,
    });
  });

  createWindow();
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (mainWindow === null) {
    createWindow();
  }
});
