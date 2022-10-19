import test from 'ava';

import { loadPlanet, Planet } from '../index.js';
import pvt from '../dist/planet-vector-tile.js';

console.log('pvt', pvt);

test('load planet and fetch tile', async t => {
    let planet = loadPlanet('info', 0, 14);
    let tile = await planet.tile(9, 82, 199);
    // let pvt = new PlanetVectorTile(tile);
    // t.is(pvt.layers.length, 2);
    t.truthy(tile);
});
