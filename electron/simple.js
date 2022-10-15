const api = require('../index');
const style = require('../styles/default.json');

const maplibre = window.maplibregl;

maplibre.setPlanetVectorTilePlugin(api);

const map = (window.map = new window.maplibregl.Map({
    container: 'map',
    style: style,
}));

map.on('mouseup', e => {
    const features = map.queryRenderedFeatures(e.point);
    console.log('features', features);
    const el = document.getElementById('features')
    document.getElementById('features').innerHTML = JSON.stringify(features, null, 2);
    document.getElementById('features-panel').style.display = 'block'
});

document.getElementById('close-panel').onclick = () => document.getElementById('features-panel').style.display = 'none'
