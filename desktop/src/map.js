const api = require('../index');
const style = require('../styles/data.json');
const maplibre = window.maplibregl;

maplibre.setPlanetVectorTilePlugin(api);
window.maplibre = maplibre;

let bbox = null;
try {
    const bboxStr = localStorage.getItem('bbox');
    bbox = JSON.parse(bboxStr);
} catch (e) {
    console.log('No stored bbox.', e);
}

const map = new window.maplibregl.Map({
    container: 'map',
    style: style,
    bounds: bbox,
});
window.map = map;
