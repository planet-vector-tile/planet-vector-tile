export default {
    input: 'dist/pvt.js',
    output: {
        file: 'dist/bundle.js',
        format: 'umd',
        name: 'planet-vector-tile',
        globals: {
            flatbuffers: 'flatbuffers',
            '@mapbox/point-geometry': 'Point',
        }
    },
    external: ['flatbuffers', '@mapbox/point-geometry'],
};
