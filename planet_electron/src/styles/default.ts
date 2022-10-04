import { StyleSpecification } from "../../../maplibre-gl-js/dist/maplibre-gl"

const style = {
  version: 8,
  name: 'Default',
  center: [0, 0],
  zoom: 0,
  bearing: 0,
  pitch: 0,
  sources: {
    sat: {
      type: 'raster',
      tiles: ['https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}'],
      minzoom: 0,
      maxzoom: 23,
    },
  },
  layers: [
    {
      id: 'background',
      type: 'background',
      maxzoom: 24,
      paint: {
        'background-color': 'rgba(0, 0, 0, 1)',
      },
    },
    {
      id: 'sat',
      type: 'raster',
      source: 'sat',
      layout: {
        visibility: 'visible',
      },
      paint: {
        'raster-opacity': {
          stops: [
            [1, 1],
            [5, 1],
          ],
        },
      },
    },
  ],
}

export default style as StyleSpecification
