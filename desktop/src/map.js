const maplibre = window.maplibregl;
let api;
if (process.env.IS_DEV) {
    api = require('../index');
} else {
    api = require('../deps/index');
}
const style = require('../styles/data.json');
maplibre.setPlanetVectorTilePlugin(api);
window.maplibre = maplibre;

let bbox = null;
try {
    const bboxStr = localStorage.getItem('bbox');
    bbox = JSON.parse(bboxStr);
} catch (e) {
    console.log('No stored bbox.', e);
}

function initMap() {
    window.map = new maplibre.Map({
        container: 'map',
        style: style,
        bounds: bbox,
    });
}

if (document.readyState !== 'loading') {
    initMap();
} else {
    window.addEventListener('DOMContentLoaded', () => initMap());
}
