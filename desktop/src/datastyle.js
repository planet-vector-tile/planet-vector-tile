const store = window.store

const sources = {}

// just for debugging
window.datastyle = sources

export default function initFromMap(map) {
  map.on('sourcedata', function (e) {
    let styleNeedsUpdate = false
    const tileData = e?.tile?.latestFeatureIndex?.vtLayers
    if (!tileData) {
      return
    }
    console.log('sourcedata', e.sourceId)
    const layerKeys = Object.keys(tileData)
    let source = sources[e.sourceId]
    if (!source) {
      source = {}
      sources[e.sourceId] = source
    }

    for (const layerId of layerKeys) {
      const layer = source[layerId]
      // this is a new layer for us to track
      if (!layer) {
        source[layerId] = determineGeometryType(tileData[layerId])
        styleNeedsUpdate = true
      }
      // we know about the layer, but we don't know what type it is
      if (layer === 'Unknown') {
        const geomType = determineGeometryType(tileData[layerId])
        if (geomType !== 'Unknown') {
          source[layerId] = geomType
          styleNeedsUpdate = true
        }
      }
    }

    if (styleNeedsUpdate) {
      updateStyle()
    }
  })
}

function determineGeometryType(layer) {
  if (layer.features.length > 0) {
    const feature = layer.features[0]
    if (feature.type === 1) {
      return 'Point'
    }
    if (feature.type === 2) {
      return 'LineString'
    }
    if (feature.type === 3) {
      return 'Polygon'
    }
  }
  return 'Unknown'
}

function updateStyle() {
  console.log('updateStyle')
  console.log('sources', sources)
}
