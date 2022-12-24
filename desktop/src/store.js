// When the app is packaged, the pwd is in the dist dir
let mapStylePath = '../styles/default.json'
let dataStylePath = '../styles/data.json'
// But we are in the desktop dir when running in dev mode
if (process.env.IS_DEV === 'true') {
  mapStylePath = './styles/default.json'
  dataStylePath = './styles/data.json'
}
const defaultMapStyle = require(mapStylePath)
const defaultDataStyle = require(dataStylePath)

import { Page, Info } from './types'

const initialState = {
  nav: {
    page: Page.Map,
    info: Info.None,
  },
  bbox: null,
  mapStyle: defaultMapStyle,
  dataStyle: defaultDataStyle,
  layerPanel: {
    flc: {}, // layer id -> visibility string value
    mapMute: {}, // layer id -> boolean
    dataMute: {}, // source layer id -> boolean
    mapSolo: {}, // layer id -> boolean
    dataSolo: {}, // source layer id -> boolean
  },
}

const properties = Object.keys(initialState)
function init() {
  for (const prop of properties) {
    const val = localStorage.getItem(prop)
    if (val) {
      initialState[prop] = JSON.parse(val)
    }
  }
}
init()

const handler = {
  get: (target, key) => {
    return target[key]
  },
  set: (target, prop, value) => {
    target[prop] = value
    localStorage.setItem(prop, JSON.stringify(value))
    return true
  },
}

window.resetStore = () => {
  for (const prop of properties) {
    localStorage.removeItem(prop)
  }
  init()
}

// Updating values in the state automatically persists to localStorage via a Proxy.
// https://benborgers.com/posts/js-object-changes
// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy
const store = new Proxy(initialState, handler)

// for debugging
window.store = store

export default store
