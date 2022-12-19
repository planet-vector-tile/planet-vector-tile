const defaultMapStyle = require('../styles/default.json')
const defaultDataStyle = require('../styles/data.json')
const { createModuleResolutionCache } = require('typescript')

// Updating values in the state automatically persists to localStorage via a Proxy.
// https://benborgers.com/posts/js-object-changes
// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy

const initialState = {
  nav: {
    page: 'map',
    info: null,
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

const store = new Proxy(initialState, handler)
window.store = store
