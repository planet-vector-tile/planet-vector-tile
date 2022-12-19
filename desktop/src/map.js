// map.js is not part of the app bundle, so here we pull in NodeJS modules with require.
const { ipcRenderer } = require('electron')

const maplibre = window.maplibregl
let api
if (process.env.IS_DEV) {
  api = require('../index')
} else {
  api = require('../deps/index')
}

maplibre.setPlanetVectorTilePlugin(api)

ipcRenderer.on('open-style', (_event, style) => {
  window.map.setStyle(style)
  store.mapStyle = style
  // NHTODO also pull out business logic to derive a new data style and update that in the store
})

function initMap() {
  let style = store.mapStyle
  if (store.nav.page === 'data') {
    style = dataStyle
  }

  map = window.map = new maplibre.Map({
    container: 'map',
    style: style,
    bounds: store.bbox,
  })

  map.on('moveend', function () {
    store.bbox = map.getBounds().toArray()
  })

  map.on('mouseup', e => {
    // give a little bit of space so we are more likely to select what we want
    const bbox = [
      [e.point.x - 5, e.point.y - 5],
      [e.point.x + 5, e.point.y + 5],
    ]

    const features = map.queryRenderedFeatures(bbox)
    console.log('features', features)
  })
}

if (document.readyState !== 'loading') {
  initMap()
} else {
  window.addEventListener('DOMContentLoaded', () => initMap())
}
