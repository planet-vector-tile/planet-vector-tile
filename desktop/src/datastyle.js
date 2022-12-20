const intialStyle = {
  version: 8,
  name: 'Data',
  center: [0, 0],
  zoom: 0,
  bearing: 0,
  pitch: 0,
  glyphs: 'https://demotiles.maplibre.org/font/{fontstack}/{range}.pbf',
  sprite: 'https://storage.googleapis.com/hellopvt3/sprites',
  sources: {
    sat: {
      type: 'raster',
      tiles: ['https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}'],
      minzoom: 0,
      maxzoom: 23,
    },
    osm: {
      type: 'raster',
      tiles: ['https://tile.openstreetmap.org/{z}/{x}/{y}.png'],
      minzoom: 0,
      maxzoom: 23,
    },
    planet: {
      type: 'planet',
      tiles: ['info', '/Users/n/code/planet-vector-tile/manifests/full_planet.yaml'],
      sprite: 'https://storage.googleapis.com/hellopvt3/sprites',
      minzoom: 0,
      maxzoom: 14,
    },
  },
  layers: [],
}

export default class DataStyle {
  constructor(map) {
    const sources = {}

    this.sources = sources
    this.map = map

    map.on('sourcedata', function (e) {
      let styleNeedsUpdate = false
      const tileData = e?.tile?.latestFeatureIndex?.vtLayers
      if (!tileData) {
        return
      }
      //   console.log('sourcedata', e)
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

      this.styleNeedsUpdate = styleNeedsUpdate
    })
  }

  updateStyle() {

    this.styleNeedsUpdate = false
  }
}

function determineGeometryType(layer) {
  console.log('layer', layer)
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
