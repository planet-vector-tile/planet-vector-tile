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

window.addEventListener('DOMContentLoaded', () => {
    const map = new window.maplibregl.Map({
        container: 'map',
        style: style,
        bounds: bbox,
    });
    window.map = map;
});
