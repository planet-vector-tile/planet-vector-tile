// map.js is not part of the app bundle, so here we pull in NodeJS modules with require.
const { ipcRenderer } = require('electron')

import { createDataStyleFromMapStyle } from './datastyle'
import store from './store'

// To prevent a mess of top-level promise calls for the map, we just expose the main map here
// and make sure we initialize the map before initializing React (in index.jsx).
export let map = null

function clickBBox(point) {
  return [
    [point.x - 3, point.y - 3],
    [point.x + 3, point.y + 3],
  ]
}

function initialize() {
  // We bring in MapLibre from a script tag so that we
  // don't have the massive library in the app bundle.
  const maplibre = window.maplibregl
  let api
  if (process.env.IS_DEV) {
    // In dev mode, we are working from planet-vector-tile/desktop
    api = require('../index')
  } else {
    // In production, we are in the Resources directory of the app bundle,
    // and the NAPI module is in deps, which you can see pulled in by forge.config.json'
    // This require is relative to Resources/dist/index.html
    api = require('../deps/index')
  }

  maplibre.setPlanetVectorTilePlugin(api)

  let style = store.mapStyle
  if (store.nav.page === 'data') {
    style = store.dataStyle
  }

  map = new maplibre.Map({
    container: 'map',
    style: style,
    bounds: store.bbox,
  })

  map.on('moveend', function () {
    // Store the bbox of the map so that the map will be in the same place when the app is restarted
    store.bbox = map.getBounds().toArray()
  })

  const hoverFeatures = new Map() //HashMap
  map.hoverFeatures = hoverFeatures
  const canvasStyle = map.getCanvas().style
  map.on('mousemove', e => {
    for (const f of hoverFeatures.values()) {
      map.setFeatureState(f, { hover: false })
    }
    hoverFeatures.clear()

    const features = map.queryRenderedFeatures(clickBBox(e.point))
    for (const f of features) {
      map.setFeatureState(f, { hover: true })
      hoverFeatures.set(f.id, f)
    }

    if (hoverFeatures.size > 0) {
      canvasStyle.cursor = 'pointer'
    } else {
      canvasStyle.cursor = ''
    }
  })

  const clickFeatures = new Map() //HashMap
  map.clickFeatures = clickFeatures
  map.on('mouseup', e => {
    for (const f of clickFeatures.values()) {
      map.setFeatureState(f, { click: false })
    }
    clickFeatures.clear()

    const features = map.queryRenderedFeatures(clickBBox(e.point))
    for (const f of features) {
      map.setFeatureState(f, { click: true })
      clickFeatures.set(f.id, f)
    }
  })

  ipcRenderer.on('open-style', (_event, style) => {
    map.setStyle(style)
    store.mapStyle = style
    createDataStyleFromMapStyle(style)
  })

  // for debugging
  window.map = map

  return map
}

export function setupMainMap() {
  return new Promise((resolve, _) => {
    if (document.readyState !== 'loading') {
      resolve(initialize())
    } else {
      window.addEventListener('DOMContentLoaded', () => resolve(initialize()))
    }
  })
}
