import test from 'ava';

import { loadPlanet, Planet } from '../index.js';
import { PlanetVectorTile } from '../dist/bundle.js';

test('load planet and fetch tile', async t => {
    const planet = loadPlanet(['info']);
    const tile = await planet.tile(9, 82, 199);
    t.truthy(tile);

    const pvt = new PlanetVectorTile(tile);
    t.is(Object.keys(pvt.layers).length, 3);
});

test('check info tile boundary feature', async t => {
    const planet = loadPlanet(['info']);
    const tile = await planet.tile(9, 82, 199);
    const pvt = new PlanetVectorTile(tile);
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
    const pvt = new PlanetVectorTile(tile);
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
