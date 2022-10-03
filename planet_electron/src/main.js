const { app, BrowserWindow } = require('electron')

const createWindow = () => {
  const win = new BrowserWindow({
    width: 1100,
    height: 800,
    webPreferences: {
        nodeIntegration: true,
        contextIsolation: false
    }
  })

  win.loadFile('./static/index.html')
}

app.whenReady().then(() => {
  createWindow()

  // no windows open but app still running in macOS
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})
