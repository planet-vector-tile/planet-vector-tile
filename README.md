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

## Run the Electron app

```
npm start
```

## Modify Schema

If you want to modify the flatdata or flatbuffer schema, you will need to install the corresponding schema compilers.

Install FlatBuffers flatc schema compiler.

Install flatdata flatdata-generator schema compiler.
