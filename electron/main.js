const { app, BrowserWindow } = require('electron');

// boilerplates
// https://www.electronjs.org/docs/latest/tutorial/boilerplates-and-clis
// https://www.electronforge.io/guides/framework-integration/react-with-typescript

// Consider electron-react-boilerplate
// https://github.com/electron-react-boilerplate/electron-react-boilerplate

// Also
// https://www.electron.build/

function createWindow() {
    const win = new BrowserWindow({
        width: 1100,
        height: 800,
        webPreferences: {
            nodeIntegration: true,
            contextIsolation: false,
        },
    });

    if (app.commandLine.hasSwitch('bench')) {
        win.loadFile('bench.html');
    } else {
        win.loadFile('simple.html');
    }

    // https://www.electronjs.org/docs/latest/api/command-line#commandlinegetswitchvalueswitch
    if (app.commandLine.hasSwitch('dbg')) {
        win.webContents.openDevTools({ mode: 'bottom' });
    }
}

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit();
});

app.whenReady().then(() => {
    createWindow();

    // no windows open but app still running in macOS
    app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) createWindow();
    });
});
