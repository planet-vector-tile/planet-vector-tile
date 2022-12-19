const maplibre = window.maplibregl
let api
if (process.env.IS_DEV) {
  api = require('../index')
} else {
  api = require('../deps/index')
}
const style = require('../styles/data.json')
maplibre.setPlanetVectorTilePlugin(api)
window.maplibre = maplibre

let bbox = null
try {
  const bboxStr = localStorage.getItem('bbox')
  bbox = JSON.parse(bboxStr)
} catch (e) {
  console.log('No stored bbox.', e)
}

function initMap() {
  const map = (window.map = new maplibre.Map({
    container: 'map',
    style: style,
    bounds: bbox,
  }))

  // map.addControl(
  //   new maplibregl.NavigationControl({
  //     showCompass: true,
  //     showZoom: true,
  //     visualizePitch: true,
  //   }),
  //   'bottom-left'
  // )

  map.on('moveend', function () {
    const bbox = JSON.stringify(map.getBounds().toArray())
    localStorage.setItem('bbox', bbox)
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
