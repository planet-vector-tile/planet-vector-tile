import store from './store'
import { selectionLayersForDataLayer } from './selection'
import { isVectorType } from './util'
import { tileInfoFillPaint, tileInfoLinePaint, tileInfoLabelStyle } from './tileinfo'

const sources = {}

// for debugging
window.dataStyleSources = sources

// NHTODO
// Handle edge cases where the user removes a source from the main map style or changes style
// We need to detect this and remove the layers not in sources. Right now we just check during initialization.

export function setupDataStyleWithMap(map) {
  // We add the sources from the map style, and we only add layers from local storage
  // that use these sources.
  createDataStyleFromMapStyle()

  map.on('sourcedata', e => {
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

export function createDataStyleFromMapStyle() {
  const vectorSourceIds = new Set()
  const dataStyle = store.dataStyle
  const sources = dataStyle.sources
  for (const sourceId in store.mapStyle.sources) {
    const source = store.mapStyle.sources[sourceId]
    const type = source.type
    // only look at vector sources
    if (!isVectorType(type)) {
      continue
    }
    vectorSourceIds.add(sourceId)
    sources[sourceId] = source
  }

  const layers = []

  // Add contextual layers from local storage so that they have the latest state.
  // Vector layers get added ad hoc as the tiles come in in updateStyle.
  const sourcesWithVectorLayers = new Set()
  for (const layer of store.dataStyle.layers) {
    const type = layer.type
    // Add contextual layers
    if (!isVectorType(type)) {
      layers.push(layer)
    }
  }

  // Add fake layers corresponding to vector sources that do not yet have layers.
  // We need this so that Maplibre fetches tiles from the vector source.
  for (const sourceId of vectorSourceIds) {
    if (!sourcesWithVectorLayers.has(sourceId)) {
      layers.push({
        id: `fake ${sourceId}`,
        type: 'line',
        source: sourceId,
        'source-layer': 'fake',
        layout: {
          visibility: 'visible',
        },
        paint: {
          'line-color': 'red',
          'line-width': 1,
        },
      })
    }
  }

  dataStyle.sources = sources
  dataStyle.layers = layers
  store.dataStyle = dataStyle
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

function updateStyle(map, sourceId, newLayers) {
  const style = store.dataStyle

  for (const sourceLayerId of newLayers) {
    const color = pickColor()

    const isTileInfoLayer = sourceLayerId.indexOf('Tile ') === 0

    const fillLayerId = `${sourceLayerId} Fill`
    const fillLayerVisibility = computeVisibility(sourceLayerId, fillLayerId, 'none')
    const fillLayer = {
      id: fillLayerId,
      type: 'fill',
      source: sourceId,
      'source-layer': sourceLayerId,
      layout: { visibility: fillLayerVisibility },
      paint: {
        'fill-color': color,
        'fill-opacity': 0.5,
      },
    }
    if (isTileInfoLayer) {
      let paint = tileInfoFillPaint(sourceLayerId)
      if (paint) {
        fillLayer.paint = paint
      }
    }

    const lineLayerId = `${sourceLayerId} Line`
    const lineLayerVisibility = computeVisibility(sourceLayerId, lineLayerId, isTileInfoLayer ? 'none' : 'visible')
    const lineLayer = {
      id: lineLayerId,
      type: 'line',
      source: sourceId,
      'source-layer': sourceLayerId,
      layout: {
        'line-join': 'round',
        'line-cap': 'round',
        visibility: lineLayerVisibility,
      },
      paint: {
        'line-color': color,
        'line-width': 2,
        'line-opacity': 0.8,
      },
    }
    if (isTileInfoLayer) {
      let paint = tileInfoLinePaint(sourceLayerId)
      if (paint) {
        lineLayer.paint = paint
      }
    }

    const circleLayerId = `${sourceLayerId} Circle`
    const circleLayerVisibility = computeVisibility(sourceLayerId, circleLayerId, 'none')
    let circleLayer = {
      id: circleLayerId,
      type: 'circle',
      source: sourceId,
      'source-layer': sourceLayerId,
      layout: { visibility: circleLayerVisibility },
      paint: {
        'circle-radius': 3,
        'circle-color': color,
        'circle-opacity': 1,
        'circle-stroke-width': 1,
        'circle-stroke-color': '#334155',
      },
    }
    if (isTileInfoLayer) {
      let layer = tileInfoLabelStyle(circleLayerId, sourceId, sourceLayerId, circleLayerVisibility)
      if (layer) {
        circleLayer = layer
      }
    }

    const hoverVisiblity =
      circleLayerVisibility === 'visible' || lineLayerVisibility === 'visible' || fillLayerVisibility === 'visible'
        ? 'visible'
        : 'none'
    const { hoverLineLayer, clickLineLayer } = selectionLayersForDataLayer(
      sourceId,
      sourceLayerId,
      color,
      hoverVisiblity
    )

    style.layers.push(fillLayer)

    // For now, we are omitting the tile info layers from hover and click due to complexity.
    // Also, it feels like a special layer that shouldn't be thought of as map data.
    if (!isTileInfoLayer) {
      style.layers.push(hoverLineLayer)
      style.layers.push(clickLineLayer)
    }

    style.layers.push(lineLayer)
    style.layers.push(circleLayer)

    store.dataStyle = style

    if (map.getStyle().name === 'Data') {
      map.addLayer(fillLayer)
      if (!isTileInfoLayer) {
        map.addLayer(hoverLineLayer)
        map.addLayer(clickLineLayer)
      }
      map.addLayer(lineLayer)
      map.addLayer(circleLayer)
    }

    // Now look and see if we had layers soloed, and if so, mute the non-soloed layers
    // NHTODO DRY this up with code in Layers.jsx toggleSolo()
    const soloedSourceLayerIdSet = new Set(store.layerPanel.dataSolo)
    if (soloedSourceLayerIdSet.size > 0) {
      for (const layer of map.getStyle().layers) {
        if (!soloedSourceLayerIdSet.has(layer['source-layer']) && isVectorType(layer.type)) {
          map.setLayoutProperty(layer.id, 'visibility', 'none')
        }
      }
    }
  }
}

function computeVisibility(sourceLayerId, layerId, defaultValue) {
  if (store.layerPanel.dataMute[sourceLayerId]) {
    return 'none'
  }
  const visibility = store.layerPanel.flc[layerId]
  if (!visibility) {
    return defaultValue
  }
  return visibility
}

export function dataLayerNameAndType(layerId) {
  const parts = layerId.split(' ')
  const type = parts.pop()
  const name = parts.join(' ')
  return { name, type }
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
