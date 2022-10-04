import { Map, NavigationControl } from '../../maplibre-gl-js/dist/maplibre-gl'

import '../../maplibre-gl-js/dist/maplibre-gl.css'

let _map: Map | null = null

export default function map(): Promise<Map> {
    if (_map) {
        return Promise.resolve(_map)
    }

    return new Promise((resolve) => {
        const m = _map = new Map({
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
    
        _map.on('load', () => resolve(m))
    }
    )
}
