const store = window.store

const sources = {}

// just for debugging
window.datastyle = sources

export default function initFromMap(map) {
  map.on('sourcedata', function (e) {
    const newLayers = []
    const layerIds = e?.tile?.latestFeatureIndex?.layerIds
    if (!Array.isArray(layerIds) || layerIds.length === 0) {
      return
    }

    let sourceLayerIds = sources[e.sourceId]
    if (!sourceLayerIds) {
      sourceLayerIds = findExistingLayerIdsFromStyle(e.sourceId, store.dataStyle)
      sources[e.sourceId] = sourceLayerIds
    }

    for (const name of layerIds) {
      if (!sourceLayerIds.has(name)) {
        sourceLayerIds.add(name)
        newLayers.push(name)
      }
    }

    updateStyle(map, e.sourceId, newLayers)
  })
}

function findExistingLayerIdsFromStyle(sourceId, style) {
  const layerIds = new Set()
  for (const layer of style.layers) {
    if (layer.source === sourceId) {
      layerIds.add(layer.id)
    }
  }
  return layerIds
}

function updateStyle(map, sourceId, newLayers) {
  const style = store.dataStyle

  for (const layerId of newLayers) {
    const lineLayer = {
      id: `${layerId}_line`,
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
      id: `${layerId}_circle`,
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
      id: `${layerId}_fill`,
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

    map.addLayer(lineLayer)
    map.addLayer(circleLayer)
    map.addLayer(fillLayer)
  }
}
