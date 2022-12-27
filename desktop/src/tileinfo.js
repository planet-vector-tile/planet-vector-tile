export function tileInfoFillPaint(sourceLayerId) {
  switch (sourceLayerId) {
    case 'Tile Boundary':
      return {
        'fill-color': 'rgb(255, 255, 255)',
        'fill-opacity': ['case', ['boolean', ['feature-state', 'hover'], false], 0.1, 0.001],
      }
    default:
      return null
  }
}

export function tileInfoLinePaint(sourceLayerId) {
  switch (sourceLayerId) {
    case 'Tile Boundary':
      return {
        'line-color': [
          'case',
          ['==', ['get', 'is_render_tile'], true],
          'rgba(255, 255, 255, 0.4)',
          'rgba(255, 255, 255, 0.1)',
        ],
        'line-width': ['case', ['==', ['get', 'is_render_tile'], true], 2, 1],
      }
    case 'Tile Bearing':
      return {
        'line-color': '#ff69b4',
        'line-width': 3,
      }
    default:
      return null
  }
}

export function tileInfoLabelStyle(circleLayerId, sourceId, sourceLayerId, circleLayerVisibility) {
  if (sourceLayerId !== 'Tile Center') {
    return null
  }
  return {
    id: circleLayerId,
    type: 'symbol',
    source: sourceId,
    'source-layer': sourceLayerId,
    layout: {
      'text-field': '{h}',
      'text-size': 14,
      visibility: circleLayerVisibility,
    },
    paint: {
      'text-color': 'white',
    },
  }
}
