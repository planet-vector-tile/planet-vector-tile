# Planet Vector Tile

# Build the entire project.

### Dependencies

-   Rust (cargo and rustc) >= v1.63
-   NodeJS >= v16.17

Build maplibre-gl-js.

```
npm run build
```

## Install the Command Line Interface.

```
cargo install --path .
```

## Convert an OSM PBF to osmflat.

```
pvt ~/geodata/extracts/california.osm.pbf ~/geodata/flatdata/california
```

or

```
cargo run -r -- --overwrite ~/geodata/extracts/california.osm.pbf ~/geodata/flatdata/california
cargo run -r -- --overwrite ~/geodata/extracts/santacruz.pbf ~/geodata/flatdata/santacruz
```

Note that if you are not using a release build, the tool will run much more slowly.

## Run the Electron app.

```
npm start
```

## Modify the schema.

If you want to modify the flatdata or flatbuffer schema, you will need to install the corresponding schema compilers.

Install FlatBuffers flatc schema compiler.

Install flatdata flatdata-generator schema compiler.
