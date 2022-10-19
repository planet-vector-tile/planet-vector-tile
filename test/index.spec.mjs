import test from 'ava';

import { loadPlanet, Planet } from '../index.js';
import { PlanetVectorTile } from '../dist/bundle.js';

test('load planet and fetch tile', async t => {
    let planet = loadPlanet('info', 0, 14);
    let tile = await planet.tile(9, 82, 199);
    t.truthy(tile);

    let pvt = new PlanetVectorTile(tile);
    t.is(Object.keys(pvt.layers).length, 2);
});

test('check info tile boundary feature', async t => {
    let planet = loadPlanet('info', 0, 14);
    let tile = await planet.tile(9, 82, 199);
    let pvt = new PlanetVectorTile(tile);
    let firstFeature = pvt.layers.tile_boundary.feature(0);
    let geom = firstFeature.loadGeometry();
    let points = geom[0];

    t.is(points.length, 5);

    let p0x = points[0].x;
    let p0y = points[0].y;
    let p2x = points[2].x;
    let p2y = points[2].y;

    t.is(p0x, 0);
    t.is(p0y, 0);
    t.is(p2x, 8192);
    t.is(p2y, 8192);
});
