const { app, BrowserWindow } = require('electron')

function createWindow() {
  const win = new BrowserWindow({
    width: 1100,
    height: 800,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false,
    },
  })

  // https://www.electronjs.org/docs/latest/api/command-line#commandlinegetswitchvalueswitch
  if (app.commandLine.hasSwitch('simple')) {
    win.loadFile('src/simple.html')
  } else if (app.isPackaged) {
    win.loadFile('dist/index.html')
  } else {
    win.loadURL('http://localhost:1234')
  }

  if (app.commandLine.hasSwitch('dbg')) {
    win.webContents.openDevTools({ mode: 'bottom' })
    setTimeout(() => win.reload(), 500)
  }
}

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit()
})

app.whenReady().then(() => {
  createWindow()

  // no windows open but app still running in macOS
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})
