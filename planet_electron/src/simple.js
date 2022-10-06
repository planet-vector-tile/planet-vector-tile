const api = require('../../planet_node/index')
const style = require('../styles/default.json')

const maplibre = window.maplibregl

maplibre.setPlanetVectorTilePlugin(api)

const map = window.map = new window.maplibregl.Map({
  container: 'map',
  style: style,
})

map.on('mouseup', e => {
  const features = map.queryRenderedFeatures(e.point)
  console.log('features', features)
  document.getElementById('features').innerHTML = JSON.stringify(features, null, 2)
})
