# PlanetVectorTile

## Contributing

You are welcome to submit pull requests to this repo for core PlanetVectorTile changes. If you need to modify MapLibre, you can submit a pull request to the planet branch of:

https://github.com/planet-vector-tile/maplibre-gl-js

## Dependencies

-   Rust >= v1.65
-   NodeJS >= v16.17

Install Rust

    curl https://sh.rustup.rs -sSf | sh -s -- -y

Install NodeJS

https://nodejs.org/en/download/

## Clone Repo

    git clone --recursive git@github.com:planet-vector-tile/planet-vector-tile.git

## Install

You can run the install script that will build and install the CLI, MapLibre, and the Electron app.

    ./install.sh

## Run the Electron app.

```
npm start
```

## Run tests.

First you need to generate the test fixtures.

```
cargo run -r --bin fixtures
```

Then,

```
cargo test
npm test
```

If you are debugging tests and you want to see full output, run:

```
cargo test -- --nocapture
```

## Modify the schema.

If you want to modify the flatdata or flatbuffer schema, you will need to install the corresponding schema compilers.

### 1. Install flatdata flatc schema compiler.

https://github.com/heremaps/flatdata/tree/master/flatdata-generator

    pip3 install flatdata-generator

### 2. Build the flatbuffer schema compiler.

    brew install flatbuffers

or

https://google.github.io/flatbuffers/flatbuffers_guide_building.html

### 3. Regenerate schema

Then run to regnerate schema files:

    npm run generate:schema
