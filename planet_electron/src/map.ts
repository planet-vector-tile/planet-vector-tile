import { Map, NavigationControl, StyleSpecification } from '../../maplibre-gl-js/dist/maplibre-gl'
import config from './config'

import '../../maplibre-gl-js/dist/maplibre-gl.css'
import './map.css'

let _map: Map | null = null

export default function map(): Map {
    if (_map) {
        return _map
    }

    _map = new Map({
        container: 'map',
        style: config.style as StyleSpecification,
        center: config.center,
        zoom: config.zoom,
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
