const hoverFeatures = new Map() //HashMap
const clickFeatures = new Map() //HashMap
const clickFeatureListeners = []

function listenToMap(map) {
  const canvasStyle = map.getCanvas().style
  map.on('mousemove', e => {
    for (const f of hoverFeatures.values()) {
      map.setFeatureState(f, { hover: false })
    }
    hoverFeatures.clear()

    const features = map.queryRenderedFeatures(clickBBox(e.point))
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
    const features = map.queryRenderedFeatures(clickBBox(e.point))
    if (features.length > 0) {
      clearClickedFeatures()
    }
    for (const f of features) {
      map.setFeatureState(f, { click: true })
      clickFeatures.set(f.id, f)
    }
  })
}

function listenForClickedFeatures(cb) {
  clickFeatureListeners.push(cb)
}

function removeClickdFeatureListener(cb) {
  const index = clickFeatureListeners.indexOf(cb)
  if (index > -1) {
    clickFeatureListeners.splice(index, 1)
  }
}

function clearClickedFeatures() {
  for (const f of clickFeatures.values()) {
    map.setFeatureState(f, { click: false })
  }
  clickFeatures.clear()
}

function clickBBox(point) {
  return [
    [point.x - 3, point.y - 3],
    [point.x + 3, point.y + 3],
  ]
}

export default {
  listenToMap,
  listenForClickedFeatures,
  removeClickdFeatureListener,
  clearClickedFeatures,
}
