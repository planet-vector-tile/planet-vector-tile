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
    // if (features.length === 0) {
    //     return;
    // }
    document.getElementById('features').innerHTML = JSON.stringify(features, null, 2);
    document.getElementById('features-panel').style.display = 'block';
});

const satSlider = document.getElementById('sat-slider');
satSlider.addEventListener('change', e => {
    const opacity = parseFloat(satSlider.value);
    console.log('sat opacity', opacity);
    map.setPaintProperty('sat', 'raster-opacity', opacity);
});
document.getElementById('close-panel').onclick = () =>
    (document.getElementById('features-panel').style.display = 'none');
