// The store is the contents of localStorage inflated into JavaScript objects.
// We are using a Proxy to intercept writes to the store and persist them to localStorage.
// The removes the need of constantly having to explicitly serde JSON for the local storage key values.
// Note that the Proxy only applies to the top level items of the store object, so you do need to
// explicitly assign a value to the store object for the persistenct to occur. e.g.:
//
//    store.nav = nav
//
// Mutating a sub object will not cause a write.
//
//    store.nav.something = 'foo' // no write to localStore
//
// But then doing this will cause a write:
//
//    store.nav = store.nav
//
// Keeping this explicit with one level of Proxy is important, because JSON serde does have a cost.

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
    dataMute: {},
    mapSolo: {},
    dataSolo: {},
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
