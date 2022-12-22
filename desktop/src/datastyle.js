import store from './store.js'

const sources = {}

// for debugging
window.dataStyleSources = sources

// NHTODO
// Handle edge cases where the user removes a source from the main map style or changes style
// We need to detect this and remove the layers not in sources. Right now we just check during initialization.

export function setupDataStyleWithMap(map) {
  // The serialized dataStyle might have layers with a source that is not in the currently loaded map style
  checkForLayersNotInSources()

  map.on('sourcedata', function (e) {

    // Check if we are getting a new vector tile source, and add it to the data style
    checkForNewVectorSource(e)

    const newLayers = new Set()
    const layerIds = e?.tile?.latestFeatureIndex?.layerIds
    if (!Array.isArray(layerIds) || layerIds.length === 0) {
      return
    }

    // We know we already have a set object here from checkForNewVectorSource
    let sourceLayerIds = sources[e.sourceId]

    for (const sourceLayerId of layerIds) {
      if (!sourceLayerIds.has(sourceLayerId)) {
        sourceLayerIds.add(sourceLayerId)
        newLayers.add(sourceLayerId)
      }
    }

    updateStyle(map, e.sourceId, newLayers)
  })
}

function checkForLayersNotInSources() {
  const trackedSources = new Set()
  for (const sourceId in store.mapStyle.sources) {
    const source = store.mapStyle.sources[sourceId]
    const type = source.type
    // only look at vector sources
    if (type === 'background' || type === 'raster' || type === 'hillshade') {
      continue
    }
    trackedSources.add(sourceId)
  }

  const validLayers  = []
  for (const layer of store.dataStyle.layers) {
    // keep the contextual layers
    const type = layer.type
    if (type === 'background' || type === 'raster' || type === 'hillshade') {
      validLayers.push(layer)
    }
    if (trackedSources.has(layer.source)) {
      validLayers.push(layer)
    }
  }
  store.dataStyle.layers = validLayers
}

function checkForNewVectorSource(sourceDataEvent) {
  const type = sourceDataEvent.source.type
  // If must be a vector source
  if (type === 'background' || type === 'raster' || type === 'hillshade') {
    return
  }
  // If we already track it, we don't need to do anything
  if (sources[sourceDataEvent.sourceId]) {
    return
  }

  store.dataStyle.sources[sourceDataEvent.sourceId] = sourceDataEvent.source
  sources[sourceDataEvent.sourceId] = new Set()
}

function findExistingLayerIdsFromStyle(sourceId, style) {
  const sourceLayerIds = new Set()
  for (const layer of style.layers) {
    if (layer.source !== sourceId) {
      continue
    }
    const sourceLayerId = removeSuffix(layer.id)
    sourceLayerIds.add(sourceLayerId)
  }
  return sourceLayerIds
}

function updateStyle(map, sourceId, newLayers) {
  const style = store.dataStyle

  for (const layerId of newLayers) {
    const color = pickColor()

    const lineLayer = {
      id: `${layerId} Line`,
      type: 'line',
      minzoom: 0,
      maxzoom: 23,
      source: sourceId,
      'source-layer': layerId,
      paint: {
        'line-color': color,
        'line-width': 2,
        'line-opacity': 0.8,
      },
    }

    const circleLayer = {
      id: `${layerId} Circle`,
      type: 'circle',
      minzoom: 0,
      maxzoom: 23,
      source: sourceId,
      'source-layer': layerId,
      paint: {
        'circle-radius': 3,
        'circle-color': color,
        'circle-opacity': 1,
        'circle-stroke-width': 1,
        'circle-stroke-color': '#334155',
      },
    }

    const fillLayer = {
      id: `${layerId} Fill`,
      type: 'fill',
      minzoom: 0,
      maxzoom: 23,
      source: sourceId,
      'source-layer': layerId,
      layout: { visibility: 'none' },
      paint: {
        'fill-color': color,
        'fill-opacity': 0.5,
      },
    }

    style.layers.push(lineLayer)
    style.layers.push(circleLayer)
    style.layers.push(fillLayer)

    if (map.getStyle().name === 'Data') {
      map.addLayer(lineLayer)
      map.addLayer(circleLayer)
      map.addLayer(fillLayer)
    }
  }
}

function removeSuffix(layerId) {
  return layerId.replace(/ (Line|Circle|Fill)$/, '')
}

let color_idx = 0
// 35 colors
const colors = [
  '#79ADDC',
  '#caffbf',
  '#ffadad',
  '#ffd6a5',
  '#fdffb6',
  '#bdb2ff',
  '#ffc6ff',
  '#9bf6ff',
  '#c5dedd',
  '#fad2e1',
  '#cdeac0',
  '#ede7b1',
  '#ffd972',
  '#e0e1e9',
  '#e4b4c2',
  '#ddfdfe',
  '#fcefef',
  '#e8ffb7',
  '#e7c8a0',
  '#68b6ef',
  '#fefdf8',
  '#a7d3a6',
  '#eae4e9',
  '#fff1e6',
  '#cfe795',
  '#fde2e4',
  '#becee4',
  '#fad2e1',
  '#e2ece9',
  '#bee1e6',
  '#f0efeb',
  '#fae1dd',
  '#dfe7fd',
  '#ff7497',
  '#cddafd',
]

function pickColor() {
  const i = color_idx++ % colors.length
  return colors[i]
}
