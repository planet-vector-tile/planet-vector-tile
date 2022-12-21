import store from './store.js'

const sources = {}

// just for debugging
window.datastyle = sources

export function setupDataStyleWithMap(map) {
  map.on('sourcedata', function (e) {
    const newLayers = new Set()
    const layerIds = e?.tile?.latestFeatureIndex?.layerIds
    if (!Array.isArray(layerIds) || layerIds.length === 0) {
      return
    }

    let sourceLayerIds = sources[e.sourceId]
    if (!sourceLayerIds) {
      sourceLayerIds = findExistingLayerIdsFromStyle(e.sourceId, store.dataStyle)
      sources[e.sourceId] = sourceLayerIds
    }

    for (const sourceLayerId of layerIds) {
      if (!sourceLayerIds.has(sourceLayerId)) {
        sourceLayerIds.add(sourceLayerId)
        newLayers.add(sourceLayerId)
      }
    }

    updateStyle(map, e.sourceId, newLayers)
  })
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
    const lineLayer = {
      id: `${layerId} Line`,
      type: 'line',
      minzoom: 0,
      maxzoom: 23,
      source: sourceId,
      'source-layer': layerId,
      paint: {
        'line-color': 'pink',
        'line-width': 2,
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
        'circle-radius': 4,
        'circle-color': 'pink',
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
        'fill-color': 'pink',
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

const colors = [
  '#f94144',
  '#f3722c',
  '#f8961e',
  '#f9844a',
  '#f9c74f',
  '#90be6d',
  '#43aa8b',
  '#4d908e',
  '#577590',
  '#277da1',
  '#5e60ce',
  '#9d4edd',
  '#f3722c',
  '#f8961e',
  '#f9844a',
  '#f9c74f',
  '#90be6d',
  '#43aa8b',
  '#4d908e',
  '#577590',
  '#277da1',
  '#5e60ce',
  '#9d4edd',
  '#f3722c',
  '#f8961e',
  '#f9844a',
  '#f9c74f',
  '#90be6d',
  '#43aa8b',
  '#4d908e',
  '#577590',
  '#277da1',
  '#5e60ce',
  '#9d4edd',
  '#f3722c',
  '#f8961e',
  '#f9844a',
]

const pastelColors = [
  '#fbf8cc',
  '#fde4cf',
  '#ffcfd2',
  '#f1c0e8',
  '#cfbaf0',
  '#a3c4f3',
  '#90dbf4',
  '#8eecf5',
  '#8eecf5',
  '#8eecf5',
  '#9w',
]
