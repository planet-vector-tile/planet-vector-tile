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
