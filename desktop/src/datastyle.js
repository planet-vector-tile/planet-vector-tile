const store = window.store

const sources = {}

// just for debugging
window.datastyle = sources

export default function initFromMap(map) {
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
