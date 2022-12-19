const { Menu, dialog, shell } = require('electron')
const fs = require('fs')

const isMac = process.platform === 'darwin'

const template = [
  { role: 'appMenu' },
  // {role: 'fileMenu'},
  {
    label: 'File',
    submenu: [
      {
        label: 'Open',
        accelerator: 'CmdOrCtrl+O',
        click: () => {
          dialog
            .showOpenDialog({
              properties: ['openFile', 'openDirectory'],
            })
            .then(result => {
              if (!result.canceled) {
                onOpen(result)
              }
            })
            .catch(err => {
              console.log('Error opening file', err)
            })
        },
      },
      { type: 'separator' },
      isMac ? { role: 'close' } : { role: 'quit' },
    ],
  },
  { role: 'editMenu' },
  { role: 'viewMenu' },
  { role: 'windowMenu' },
  {
    role: 'help',
    submenu: [
      {
        label: 'Learn More',
        click: async () => {
          await shell.openExternal('https://planetvectortile.org')
        },
      },
    ],
  },
]
const menu = Menu.buildFromTemplate(template)
Menu.setApplicationMenu(menu)

function onOpen(result) {
  console.log('onOpen', result)

  const { filePaths } = result
  if (filePaths.length === 0) {
    return
  }

  const filePath = filePaths[0]
  fs.readFile(filePath, 'utf-8', (err, data) => {
    if (err) {
      alert('An error ocurred reading the file :' + err.message)
      return
    }

    // Style
    if (filePath.split('.').pop() === 'json') {
    }
    // Manifest
    else if (filePath.split('.').pop() === 'yaml') {
    } else {
      alert('File type not supported')
    }
  })
}
