let map = null

// We insert features by their feature id into a hash map so that we get only one entry per feature.
// The feature may be in several render layers.
const hoverFeatures = new Map() //HashMap
const clickFeatures = new Map() //HashMap

let showFeaturesPanel = () => {}
let clickFeaturesListener = _clickFeatures => {}

export function listenToMapForSelection(maplibreMap) {
  map = maplibreMap
  const canvasStyle = map.getCanvas().style
  map.on('mousemove', e => {
    for (const f of hoverFeatures.values()) {
      map.setFeatureState(f, { hover: false })
    }
    hoverFeatures.clear()

    const features = map.queryRenderedFeatures(mouseBBox(e.point))
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

  map.on('click', e => {
    clearClickedFeatures()
    const features = map.queryRenderedFeatures(mouseBBox(e.point))
    console.log('features', features)
    for (const f of features) {
      map.setFeatureState(f, { click: true })
      clickFeatures.set(f.id, f)
    }
    if (clickFeatures.size > 0) {
      showFeaturesPanel()
    }
    clickFeaturesListener(Array.from(clickFeatures.values()))
  })
}

export function listenToShowFeaturesPanel(cb) {
  showFeaturesPanel = cb
}

export function listenToClickFeatures(cb) {
  clickFeaturesListener = cb
}

export function listClickFeatures() {
  return Array.from(clickFeatures.values())
}

export function clearClickedFeatures() {
  for (const f of clickFeatures.values()) {
    map.setFeatureState(f, { click: false })
  }
  clickFeatures.clear()
}

export function selectionLayersForDataLayer(source, sourceLayerId, hoverColor, visibility) {
  const hoverLineLayer = {
    id: `${sourceLayerId} Hover`,
    type: 'line',
    source,
    'source-layer': sourceLayerId,
    layout: {
      'line-join': 'round',
      'line-cap': 'round',
      visibility,
    },
    paint: {
      'line-color': hoverColor,
      'line-width': 10,
      'line-blur': 2,
      'line-opacity': ['case', ['boolean', ['feature-state', 'hover'], false], 0.3, 0],
    },
  }

  const clickLineLayer = {
    id: `${sourceLayerId} Click`,
    type: 'line',
    source,
    'source-layer': sourceLayerId,
    layout: {
      'line-join': 'round',
      'line-cap': 'round',
      visibility,
    },
    paint: {
      'line-color': '#a21caf',
      'line-width': 12,
      'line-blur': 2,
      'line-opacity': ['case', ['boolean', ['feature-state', 'click'], false], 0.5, 0],
    },
  }

  // NHTODO Handle circles.

  return {
    hoverLineLayer,
    clickLineLayer,
  }
}

export function hideHoverForDataLayer(sourceLayerId) {
  const hoverLayerId = `${sourceLayerId} Hover`
  const hoverLayer = map.getLayer(hoverLayerId)
  if (hoverLayer) {
    map.setLayoutProperty(hoverLayerId, 'visibility', 'none')
  }
}

export function showHoverForDataLayer(sourceLayerId) {
  const hoverLayerId = `${sourceLayerId} Hover`
  const hoverLayer = map.getLayer(hoverLayerId)
  if (hoverLayer) {
    map.setLayoutProperty(hoverLayerId, 'visibility', 'visible')
  }
}

function mouseBBox(point) {
  return [
    [point.x - 3, point.y - 3],
    [point.x + 3, point.y + 3],
  ]
}
