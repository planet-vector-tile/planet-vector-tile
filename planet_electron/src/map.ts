import { Map, NavigationControl } from '../../maplibre-gl-js/dist/maplibre-gl'

import '../../maplibre-gl-js/dist/maplibre-gl.css'
import './map.css'

let _map: Map | null = null

export default function map(): Map {
    if (_map) {
        return _map
    }

    _map = new Map({
        container: 'map',
        style: 'https://demotiles.maplibre.org/style.json',
        center: [5, 34],
        zoom: 2,
    })

    _map.addControl(
        new NavigationControl({
            showCompass: true,
            showZoom: true,
            visualizePitch: true,
        }),
        'bottom-right'
    )

    return _map
}
