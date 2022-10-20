const { InferencePriority } = require('typescript');
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
    
    const infos = features.map(f => {
        const info = {}
        info.id = f.id;
        info.layer = f.layer.id;
        info.layerType = f.layer.type;
        info.sourceLayer = f.sourceLayer;
        info.geometryType = f.geometry.type;
        info.properties = f.properties;
        return info;
    })
    document.getElementById('features').innerHTML = JSON.stringify(infos, null, 2);
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
