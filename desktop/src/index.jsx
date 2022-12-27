import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { setupMainMap } from './map'
import { setupHotKeys } from './hotkeys'
import { setupDataStyleWithMap } from './datastyle'

import './index.css'

// Initialize map, then initialize React.
setupMainMap()
  .then(map => {
    setupHotKeys(map)
    setupDataStyleWithMap(map)

    ReactDOM.createRoot(document.getElementById('root')).render(
      <React.StrictMode>
        <App />
      </React.StrictMode>
    )

    // ReactDOM.createRoot(document.getElementById('root')).render(<App />)
  })
  .catch(err => {
    console.error('Unable to initialize map.', err)
  })
