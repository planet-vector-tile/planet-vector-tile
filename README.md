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
pvt -o ~/geodata/extracts/california.osm.pbf ~/geodata/flatdata/california
```

or

```
cargo run -- -o ~/geodata/extracts/california.osm.pbf ~/geodata/flatdata/california
```

## Run the Electron app.

```
npm start
```

## Modify the schema.

If you want to modify the flatdata or flatbuffer schema, you will need to install the corresponding schema compilers.

Install FlatBuffers flatc schema compiler.

Install flatdata flatdata-generator schema compiler.