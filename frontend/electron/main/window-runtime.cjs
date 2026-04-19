function createWindowRuntime(options) {
    const {
        app,
        BrowserWindow,
        Menu,
        getMainWindow,
        logToFile,
        path,
        screen,
        setMainWindow,
        shell,
        stopAllTerminalBridges,
    } = options;
    const { resolveRendererLoadTarget } = require("./load-target.cjs");

    function sendAppCommand(command) {
        getMainWindow()?.webContents.send('app-command', command);
    }

    function buildAppMenu() {
        return Menu.buildFromTemplate([
            { label: 'File', submenu: [
                { label: 'New Workspace', accelerator: 'Ctrl+Shift+N', click: () => sendAppCommand('new-workspace') },
                { label: 'New Surface', accelerator: 'Ctrl+T', click: () => sendAppCommand('new-surface') },
                { type: 'separator' },
                { label: 'Settings', accelerator: 'Ctrl+,', click: () => sendAppCommand('toggle-settings') },
                { type: 'separator' },
                { role: 'quit', label: 'Exit' },
            ] },
            { label: 'Edit', submenu: [
                { role: 'undo' }, { role: 'redo' }, { type: 'separator' }, { role: 'cut' },
                { label: 'Copy', accelerator: 'Ctrl+C', click: () => { getMainWindow()?.webContents.copy(); sendAppCommand('copy'); } },
                { label: 'Paste', accelerator: 'Ctrl+V', click: () => { getMainWindow()?.webContents.paste(); sendAppCommand('paste'); } },
                { label: 'Select All', accelerator: 'Ctrl+A', click: () => { getMainWindow()?.webContents.selectAll(); sendAppCommand('select-all'); } },
            ] },
            { label: 'View', submenu: [
                { label: 'Command Palette', accelerator: 'Ctrl+Shift+P', click: () => sendAppCommand('toggle-command-palette') },
                { label: 'Search', accelerator: 'Ctrl+Shift+F', click: () => sendAppCommand('toggle-search') },
                { label: 'File Manager', accelerator: 'Ctrl+Shift+E', click: () => sendAppCommand('toggle-file-manager') },
                { label: 'Toggle Sidebar', accelerator: 'Ctrl+B', click: () => sendAppCommand('toggle-sidebar') },
                { type: 'separator' }, { role: 'reload' }, { role: 'forceReload' }, { role: 'toggleDevTools' },
                { type: 'separator' }, { role: 'resetZoom' }, { role: 'zoomIn' }, { role: 'zoomOut' }, { role: 'togglefullscreen' },
            ] },
            { label: 'Features', submenu: [
                { label: 'Mission Console', click: () => sendAppCommand('toggle-mission') },
                { label: 'Command History', click: () => sendAppCommand('toggle-command-history') },
                { label: 'Command Log', click: () => sendAppCommand('toggle-command-log') },
                { label: 'Session Vault', click: () => sendAppCommand('toggle-session-vault') },
                { label: 'System Monitor', click: () => sendAppCommand('toggle-system-monitor') },
                { label: 'Execution Canvas', click: () => sendAppCommand('toggle-canvas') },
                { label: 'Time Travel Snapshots', click: () => sendAppCommand('toggle-time-travel') },
            ] },
            { label: 'Window', submenu: [
                { label: 'Split Right', accelerator: 'Ctrl+D', click: () => sendAppCommand('split-right') },
                { label: 'Split Down', accelerator: 'Ctrl+Shift+D', click: () => sendAppCommand('split-down') },
                { label: 'Zoom Pane', accelerator: 'Ctrl+Shift+Z', click: () => sendAppCommand('toggle-zoom') },
                { type: 'separator' }, { role: 'minimize' }, { role: 'close' },
            ] },
            { label: 'Help', submenu: [{ label: 'About', click: () => sendAppCommand('about') }] },
        ]);
    }

    function setWindowOpacity(opacity) {
        const normalized = Number.isFinite(opacity) ? Math.min(1, Math.max(0.35, Number(opacity))) : 1;
        if (getMainWindow() && typeof getMainWindow().setOpacity === 'function') {
            getMainWindow().setOpacity(normalized);
        }
        return normalized;
    }

    function createWindow() {
        const { width: screenW, height: screenH } = screen.getPrimaryDisplay().workAreaSize;
        const useNativeFrame = process.platform === 'win32';
        const mainWindow = new BrowserWindow({
            width: Math.min(1400, screenW),
            height: Math.min(900, screenH),
            minWidth: 600,
            minHeight: 400,
            frame: useNativeFrame,
            titleBarStyle: useNativeFrame ? 'default' : 'hidden',
            autoHideMenuBar: false,
            titleBarOverlay: useNativeFrame ? undefined : process.platform === 'win32' ? { color: '#181825', symbolColor: '#cdd6f4', height: 36 } : undefined,
            webPreferences: {
                preload: path.join(options.electronDir, 'preload.cjs'),
                nodeIntegration: false,
                contextIsolation: true,
                sandbox: true,
                webviewTag: true,
            },
            title: 'tamux',
            icon: path.join(options.electronDir, '..', 'assets', 'icon.ico'),
            backgroundColor: '#1e1e2e',
            show: false,
            opacity: 1,
        });
        setMainWindow(mainWindow);
        const rendererLoadTarget = resolveRendererLoadTarget({
            app,
            electronDir: options.electronDir,
            env: process.env,
            path,
        });
        if (rendererLoadTarget.kind === "url") {
            mainWindow.loadURL(rendererLoadTarget.value);
        } else {
            mainWindow.loadFile(rendererLoadTarget.value);
        }
        mainWindow.webContents.setWindowOpenHandler(({ url }) => /^https?:\/\//i.test(url) ? (void shell.openExternal(url), { action: 'deny' }) : { action: 'allow' });
        mainWindow.webContents.on('will-navigate', (event, url) => {
            const currentUrl = mainWindow.webContents.getURL();
            if (url !== currentUrl && /^https?:\/\//i.test(url)) {
                event.preventDefault();
                void shell.openExternal(url);
            }
        });
        Menu.setApplicationMenu(buildAppMenu());
        mainWindow.once('ready-to-show', () => mainWindow.show());
        if (!app.isPackaged) mainWindow.webContents.openDevTools();
        mainWindow.on('maximize', () => mainWindow.webContents.send('window-state', 'maximized'));
        mainWindow.on('unmaximize', () => mainWindow.webContents.send('window-state', 'normal'));
        mainWindow.on('closed', () => {
            logToFile('info', 'main window closed');
            stopAllTerminalBridges(true, true);
            setMainWindow(null);
        });
    }

    return { createWindow, setWindowOpacity };
}

module.exports = { createWindowRuntime };
