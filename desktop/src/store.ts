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

import { Store, Info, Page } from './types'

const initialState: Store = {
  nav: {
    page: Page.Map,
    info: Info.None,
  },
  bbox: null,
  mapStyle: defaultMapStyle,
  dataStyle: defaultDataStyle,
}

const properties = ['nav', 'bbox', 'mapStyle', 'dataStyle']
function init() {
  for (const prop of properties) {
    const val = localStorage.getItem(prop)
    if (val) {
      // @ts-ignore
      initialState[prop] = JSON.parse(val)
    }
  }
}
init()

const handler = {
  get: (target: any, key: any) => {
    return target[key]
  },
  set: (target: any, prop: string, value: any) => {
    target[prop] = value
    localStorage.setItem(prop, JSON.stringify(value))
    return true
  },
}

function resetStore() {
  for (const prop of properties) {
    localStorage.removeItem(prop)
  }
  init()
}
window.resetStore = resetStore

// Updating values in the state automatically persists to localStorage via a Proxy.
// https://benborgers.com/posts/js-object-changes
// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy

const store = new Proxy(initialState, handler)

// for debugging
window.store = store

export default store
