const api = require('../index');
const style = require('../styles/default.json');

const maplibre = window.maplibregl;

maplibre.setPlanetVectorTilePlugin(api);

const map = (window.map = new window.maplibregl.Map({
    container: 'map',
    style: style,
}));

map.getCanvas().style.cursor = 'crosshair';

let pvt = (window.pvt = {
    clickedFeatures: null,
    selectedFeature: null,
});

map.on('mouseup', e => {
    const features = (pvt.clickedFeatures = map.queryRenderedFeatures(e.point));

    const infos = features
        .map(f => {
            const info = {};
            info.id = f.id;
            info.layer = f.layer.id;
            info.layerType = f.layer.type;
            info.sourceLayer = f.sourceLayer;
            info.geometryType = f.geometry.type;
            info.properties = f.properties;
            const json = JSON.stringify(info, null, 2);

            const title = info.properties?.tile || `${info.layer} - ${info.id}`;

            return `<details><summary>${title}<button onclick="select(${f.id})">Select</button></summary><pre>${json}</pre></details>`;
        })
        .join('');

    document.getElementById('features').innerHTML = infos;
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

function select(id) {
    console.log('selected feature id', id);
    if (pvt.selectedFeature) {
        map.setFeatureState(
            {
                source: 'planet',
                sourceLayer: 'tile_boundary',
                id: pvt.selectedFeature.id,
            },
            {
                selected: false,
            }
        );
    }
    const feature = (pvt.selectedFeature = pvt.clickedFeatures.find(f => f.id === id));
    map.setFeatureState(
        {
            source: 'planet',
            sourceLayer: 'tile_boundary',
            id: feature.id,
        },
        {
            selected: true,
        }
    );
    map.setFilter('tile_bearing', ['==', 'z', feature.properties.z]);
    map.setFilter('tile_bearing_arrow', ['==', 'z', feature.properties.z]);
    map.setFilter('tile_center_label', ['==', 'z', feature.properties.z]);
}
