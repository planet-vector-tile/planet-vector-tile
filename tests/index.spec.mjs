import test from 'ava';

import { loadPlanet } from '../index.js';
import PVT from '../dist/bundle.js';

test('load planet and fetch tile', async t => {
    const planet = loadPlanet(['info']);
    const tile = await planet.tile(9, 82, 199);
    t.truthy(tile);

    const pvt = new PVT(tile);
    t.is(Object.keys(pvt.layers).length, 3);
});

test('check info tile boundary feature', async t => {
    const planet = loadPlanet(['info']);
    const tile = await planet.tile(9, 82, 199);
    const pvt = new PVT(tile);
    const firstFeature = pvt.layers.tile_boundary.feature(0);

    let props = firstFeature.properties;
    let { z, x, y } = props;
    t.is(z, 0);
    t.is(x, 0);
    t.is(y, 0);

    const geom = firstFeature.loadGeometry();
    const points = geom[0];

    t.is(points.length, 5);

    const p0x = points[0].x;
    const p0y = points[0].y;
    const p2x = points[2].x;
    const p2y = points[2].y;

    t.is(p0x, -16384);
    t.is(p0y, -16384);
    t.is(p2x, 16383);
    t.is(p2y, 16383);
});

test('check info tile 0/0/0', async t => {
    const planet = loadPlanet(['info']);
    const tile = await planet.tile(0, 0, 0);
    const pvt = new PVT(tile);
    const firstFeature = pvt.layers.tile_boundary.feature(0);

    let props = firstFeature.properties;
    let { z, x, y } = props;
    t.is(z, 0);
    t.is(x, 0);
    t.is(y, 0);

    const geom = firstFeature.loadGeometry();
    const points = geom[0];

    t.is(points.length, 5);

    const p0x = points[0].x;
    const p0y = points[0].y;
    const p2x = points[2].x;
    const p2y = points[2].y;

    t.is(p0x, 0);
    t.is(p0y, 0);
    t.is(p2x, 8191);
    t.is(p2y, 8191);
});

test('check scotts valley tile with nodes', async t => {
    const planet = loadPlanet(['info', 'tests/fixtures/santacruz/sort']);
    const tile = await planet.tile(12, 659, 1593);
    const pvt = new PVT(tile);

    const len = Object.keys(pvt.layers).length;
    t.is(len, 5);

    t.is(pvt.layers.nodes.length, 2748);

    let nodes = pvt.layers.nodes;
    let firstFeature = nodes.feature(0);
    let props = firstFeature.properties;
    t.is(props.osm_id, 5680698655);
    t.is(props.power, 'pole');

    let geom = firstFeature.loadGeometry();
    let point = geom[0][0];
    t.is(point.x, 162);
    t.is(point.y, 58);

    // NHTODO Fixme: geojson result is not correct.
    // let geojson = firstFeature.toGeoJSON(12, 659, 1593);
    // console.log('geojson', JSON.stringify(geojson, null, 2));
});
