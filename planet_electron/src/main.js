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
  // const cl = app.commandLine
  // win.webContents.send('argv', process.argv)

  if (app.isPackaged) {
    win.loadFile('dist/index.html')
  } else {
    win.loadURL('http://localhost:1234')
    win.webContents.openDevTools({ mode: 'bottom' })
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
